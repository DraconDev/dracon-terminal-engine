#![allow(missing_docs)]
//! Widget Gallery — all interactive widgets shown simultaneously in a polished grid.
//!
//! Every widget is rendered in its own card with border, title, and live state.
//! Click or use Enter to interact with each widget.
//!
//! Controls:
//!   ↑/↓/←/→  — navigate between widget cards
//!   Enter    — activate focused widget
//!   Tab      — cycle theme
//!   q        — quit

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::hitzone::ScopedZoneRegistry;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, Checkbox, ProgressBar, Radio, SearchInput, Select, Slider, Spinner, Toggle,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Widget slot positions in the grid (row, col, name, icon)
const WIDGET_SLOTS: &[(usize, usize, &str, &str)] = &[
    (0, 0, "Checkbox", "󰄵"),
    (0, 1, "Radio", "󰑃"),
    (0, 2, "Toggle", "󰔡"),
    (0, 3, "Spinner", "󰝥"),
    (1, 0, "Slider", "󰜈"),
    (1, 1, "Select", "󰑇"),
    (1, 2, "Search Input", "󰍉"),
    (2, 0, "Progress Bar", "󰖎"),
    (2, 1, "Button", "󰔂"),
];

struct WidgetGallery {
    id: WidgetId,
    selected: usize,
    checkbox: Checkbox,
    radio: Radio,
    slider: Slider,
    spinner: Spinner,
    toggle: Toggle,
    select: Select,
    search: SearchInput,
    progress: ProgressBar,
    button: Button,
    area: Rect,
    theme: Theme,
    should_quit: Arc<AtomicBool>,
    show_help: bool,
    zones: RefCell<ScopedZoneRegistry<usize>>,
    keybindings: KeybindingSet,
}

impl WidgetGallery {
    fn new(quit: Arc<AtomicBool>, theme: Theme) -> Self {
        let id = WidgetId::new(1);
        Self {
            id,
            selected: 0,
            checkbox: Checkbox::new(WidgetId::new(10), "Enable Feature"),
            radio: Radio::new(WidgetId::new(11), "Selected"),
            slider: Slider::new(WidgetId::new(12)).with_range(0.0, 100.0),
            spinner: Spinner::new(WidgetId::new(13)),
            toggle: Toggle::new(WidgetId::new(14), "Dark Mode"),
            select: Select::new(WidgetId::new(15)).with_options(vec![
                "Red".to_string(),
                "Green".to_string(),
                "Blue".to_string(),
            ]),
            search: SearchInput::new(WidgetId::new(16)),
            progress: ProgressBar::new(WidgetId::new(17)),
            button: Button::with_id(WidgetId::new(18), "Click Me!"),
            area: Rect::new(0, 0, 80, 24),
            theme,
            should_quit: quit,
            show_help: false,
            zones: RefCell::new(ScopedZoneRegistry::new()),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.checkbox.on_theme_change(&self.theme);
        self.radio.on_theme_change(&self.theme);
        self.slider.on_theme_change(&self.theme);
        self.spinner.on_theme_change(&self.theme);
        self.toggle.on_theme_change(&self.theme);
        self.select.on_theme_change(&self.theme);
        self.search.on_theme_change(&self.theme);
        self.progress.on_theme_change(&self.theme);
        self.button.on_theme_change(&self.theme);
    }

    fn widget_mut(&mut self, slot: usize) -> &mut dyn Widget {
        match slot {
            0 => &mut self.checkbox,
            1 => &mut self.radio,
            2 => &mut self.toggle,
            3 => &mut self.spinner,
            4 => &mut self.slider,
            5 => &mut self.select,
            6 => &mut self.search,
            7 => &mut self.progress,
            8 => &mut self.button,
            _ => &mut self.checkbox,
        }
    }

    fn slot_rect(&self, slot: usize, area: Rect) -> Rect {
        let (row, col, _name, _icon) = WIDGET_SLOTS[slot];
        let rows = 3u16;
        let cols = if row == 0 {
            4u16
        } else if row == 1 {
            3u16
        } else {
            2u16
        };

        let card_w = area.width.saturating_sub(2) / cols;
        let card_h = area.height.saturating_sub(4) / rows;

        let x = area.x + 1 + col as u16 * card_w;
        let y = area.y + 2 + row as u16 * card_h;

        Rect::new(x, y, card_w.saturating_sub(1), card_h.saturating_sub(1))
    }

    fn update_bg(&self, cell: &mut Cell) {
        cell.bg = self.theme.bg;
        cell.fg = self.theme.fg;
        cell.transparent = false;
    }
}

impl Widget for WidgetGallery {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn area(&self) -> Rect {
        self.area
    }
    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }
    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.checkbox.on_theme_change(theme);
        self.radio.on_theme_change(theme);
        self.slider.on_theme_change(theme);
        self.spinner.on_theme_change(theme);
        self.toggle.on_theme_change(theme);
        self.select.on_theme_change(theme);
        self.search.on_theme_change(theme);
        self.progress.on_theme_change(theme);
        self.button.on_theme_change(theme);
    }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            self.update_bg(cell);
        }

        // Header
        let title = " Widget Gallery ";
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, 2, 0, title, t.primary, t.bg, true);
        draw_text(
            &mut plane,
            area.width.saturating_sub(theme_label.len() as u16 + 1),
            0,
            &theme_label,
            t.secondary,
            t.bg,
            false,
        );

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Widget cards
        self.zones.borrow_mut().clear();
        for (slot, &(_row, _col, name, icon)) in WIDGET_SLOTS.iter().enumerate() {
            let rect = self.slot_rect(slot, area);
            let is_selected = slot == self.selected;

            render_card_border(&mut plane, rect, t, is_selected);

            // Title with icon
            let title = format!("{} {}", icon, name);
            draw_text(
                &mut plane,
                rect.x + 1,
                rect.y + 1,
                &title,
                t.primary,
                t.surface,
                true,
            );

            // Render the widget into its card
            let widget_area = Rect::new(
                rect.x + 1,
                rect.y + 2,
                rect.width.saturating_sub(2),
                rect.height.saturating_sub(3),
            );
            if widget_area.width >= 4 && widget_area.height >= 1 {
                // Register hit zone for this widget's interactive area
                self.zones.borrow_mut().register(
                    slot,
                    widget_area.x,
                    widget_area.y,
                    widget_area.width,
                    widget_area.height,
                );

                let mut w_plane = match slot {
                    0 => self.checkbox.render(widget_area),
                    1 => self.radio.render(widget_area),
                    2 => self.toggle.render(widget_area),
                    3 => self.spinner.render(widget_area),
                    4 => self.slider.render(widget_area),
                    5 => self.select.render(widget_area),
                    6 => self.search.render(widget_area),
                    7 => self.progress.render(widget_area),
                    8 => self.button.render(widget_area),
                    _ => Plane::new(0, 0, 0),
                };
                blit_to(
                    &mut plane,
                    &mut w_plane,
                    widget_area.x as usize,
                    widget_area.y as usize,
                );

                // Show widget state below
                let state_y = widget_area.y + widget_area.height + 1;
                if state_y < rect.y + rect.height - 1 {
                    let state = match slot {
                        0 => format!("checked: {}", self.checkbox.is_checked()),
                        1 => format!("selected: {}", self.radio.is_selected()),
                        2 => format!("on: {}", self.toggle.is_on()),
                        3 => format!("frame: '{}'", self.spinner.current_frame()),
                        4 => format!("value: {:.0}", self.slider.value()),
                        5 => format!(
                            "selected: {}",
                            self.select.selected_label().unwrap_or("none")
                        ),
                        6 => format!("query: '{}'", self.search.query()),
                        7 => format!("progress: {:.0}%", self.progress.progress() * 100.0),
                        8 => String::from("[Click me]"),
                        _ => String::new(),
                    };
                    draw_text(
                        &mut plane,
                        rect.x + 1,
                        state_y,
                        &state,
                        t.fg_muted,
                        t.surface,
                        false,
                    );
                }
            }
        }

        // Footer
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
        let nav_text = " ↑↓←→ nav | Enter activate | Tab: theme | ?: help | Esc: dismiss | q quit ";
        draw_text(&mut plane, 2, footer_y, nav_text, t.fg_muted, t.bg, false);

        // Help overlay
        if self.show_help {
            let hw = 44u16.min(area.width.saturating_sub(4));
            let hh = 12u16.min(area.height.saturating_sub(4));
            let hx = (area.width - hw) / 2;
            let hy = (area.height - hh) / 2;
            for y in hy..hy + hh {
                for x in hx..hx + hw {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
            for x in hx..hx + hw {
                let top_idx = (hy * area.width + x) as usize;
                let bot_idx = ((hy + hh - 1) * area.width + x) as usize;
                if top_idx < plane.cells.len() {
                    plane.cells[top_idx].char = '─';
                    plane.cells[top_idx].fg = t.outline;
                }
                if bot_idx < plane.cells.len() {
                    plane.cells[bot_idx].char = '─';
                    plane.cells[bot_idx].fg = t.outline;
                }
            }
            for y in hy..hy + hh {
                let left_idx = (y * area.width + hx) as usize;
                let right_idx = (y * area.width + hx + hw - 1) as usize;
                if left_idx < plane.cells.len() {
                    plane.cells[left_idx].char = '│';
                    plane.cells[left_idx].fg = t.outline;
                }
                if right_idx < plane.cells.len() {
                    plane.cells[right_idx].char = '│';
                    plane.cells[right_idx].fg = t.outline;
                }
            }
            let title = "Widget Gallery Help";
            let tx = hx + (hw - title.len() as u16) / 2;
            draw_text(
                &mut plane,
                tx,
                hy + 1,
                title,
                t.primary,
                t.surface_elevated,
                true,
            );
            let shortcuts = [
                ("↑↓←→", "Navigate cards"),
                ("Enter", "Activate widget"),
                (self.keybindings.display(actions::THEME).unwrap_or("t"), "Cycle theme"),
                (self.keybindings.display(actions::HELP).unwrap_or("?"), "Toggle help"),
                (self.keybindings.display(actions::BACK).unwrap_or("Esc"), "Dismiss help"),
                (self.keybindings.display(actions::QUIT).unwrap_or("q"), "Quit"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                draw_text(
                    &mut plane,
                    hx + 2,
                    row,
                    key,
                    t.primary,
                    t.surface_elevated,
                    false,
                );
                draw_text(
                    &mut plane,
                    hx + 14,
                    row,
                    desc,
                    t.fg,
                    t.surface_elevated,
                    false,
                );
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
            }
            return true;
        }

        if self.keybindings.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            self.cycle_theme();
            return true;
        }

        match key.code {
            KeyCode::Right | KeyCode::Down => {
                self.selected = (self.selected + 1) % WIDGET_SLOTS.len();
                true
            }
            KeyCode::Left | KeyCode::Up => {
                self.selected = if self.selected == 0 {
                    WIDGET_SLOTS.len() - 1
                } else {
                    self.selected - 1
                };
                true
            }
            KeyCode::Enter | KeyCode::Char(' ') if key.modifiers.is_empty() => self.widget_mut(self.selected).handle_key(key),
            _ => self.widget_mut(self.selected).handle_key(key),
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Check if click is in a widget zone using ScopedZoneRegistry
        let zone_slot = self.zones.borrow().dispatch(col, row);
        if let Some(slot) = zone_slot {
            if let MouseEventKind::Down(MouseButton::Left) = kind {
                self.selected = slot;
            }
            let rect = self.slot_rect(slot, self.area);
            let rel_col = col - rect.x - 1;
            let rel_row = row - rect.y - 2;
            return self.widget_mut(slot).handle_mouse(kind, rel_col, rel_row);
        }

        // Check if click is in any card (for selection, not widget interaction)
        for (slot, &(..)) in WIDGET_SLOTS.iter().enumerate() {
            let rect = self.slot_rect(slot, self.area);
            if col >= rect.x
                && col < rect.x + rect.width
                && row >= rect.y
                && row < rect.y + rect.height
            {
                if let MouseEventKind::Down(MouseButton::Left) = kind {
                    self.selected = slot;
                }
                return true;
            }
        }
        false
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RENDERING HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = (y * plane.width + x + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch,
                fg,
                bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false,
                skip: false,
            };
        }
    }
}

fn render_card_border(plane: &mut Plane, rect: Rect, t: &Theme, selected: bool) {
    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
    let border = if selected { t.primary } else { t.outline };
    let bg = if selected {
        t.surface_elevated
    } else {
        t.surface
    };
    if w < 3 || h < 3 {
        return;
    }
    for row in y..y + h {
        for col in x..x + w {
            let idx = (row * plane.width + col) as usize;
            if idx >= plane.cells.len() {
                continue;
            }
            plane.cells[idx].bg = bg;
            let is_border = row == y || row == y + h - 1 || col == x || col == x + w - 1;
            if is_border {
                plane.cells[idx].fg = border;
                plane.cells[idx].char = if row == y && col == x {
                    '╭'
                } else if row == y && col == x + w - 1 {
                    '╮'
                } else if row == y + h - 1 && col == x {
                    '╰'
                } else if row == y + h - 1 && col == x + w - 1 {
                    '╯'
                } else if row == y || row == y + h - 1 {
                    '─'
                } else {
                    '│'
                };
            } else {
                plane.cells[idx].char = ' ';
                plane.cells[idx].fg = t.fg;
            }
        }
    }
}

fn blit_to(dest: &mut Plane, src: &mut Plane, offset_x: usize, offset_y: usize) {
    // Copy src cells into dest at the given offset
    // Only copy non-transparent, non-null cells
    for i in 0..src.cells.len() {
        let cell = &src.cells[i];
        if cell.char == '\0' || cell.transparent {
            continue;
        }
        let w = src.width as usize;
        let row = i / w;
        let col = i % w;
        let dy = offset_y + row;
        let dx = offset_x + col;
        if dy >= dest.height as usize || dx >= dest.width as usize {
            continue;
        }
        let idx = dy * dest.width as usize + dx;
        if idx < dest.cells.len() {
            dest.cells[idx] = *cell;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("Widget Gallery — All widgets demo | ↑↓←→ nav | Enter interact | t theme | q quit");
    std::thread::sleep(std::time::Duration::from_millis(300));

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let env_theme = Theme::from_env_or(Theme::nord());

    let gallery = WidgetGallery::new(running_clone.clone(), env_theme.clone());

    let mut app = App::new()?
        .title("Widget Gallery")
        .fps(30)
        .theme(env_theme.clone());

    app.add_widget(Box::new(gallery), Rect::new(0, 0, 80, 24));

    app.on_tick(move |ctx, _| {
        if !running_clone.load(Ordering::SeqCst) {
            ctx.stop();
        }
    })
    .run(|_ctx| {})
}

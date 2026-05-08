//! Embedded Widget Gallery scene for the showcase.
//!
//! Runs inside the showcase process via SceneRouter instead of launching
//! an external binary. Press `B` or `Esc` to return to the showcase grid.

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::hitzone::ScopedZoneRegistry;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, Checkbox, ProgressBar, Radio, SearchInput, Select, Slider, Spinner, Toggle,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

// ── Widget slot positions in the grid (row, col, name, icon) ────────────────

const SLOTS: &[(usize, usize, &str, &str)] = &[
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

// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET GALLERY SCENE
// ═══════════════════════════════════════════════════════════════════════════════

pub struct WidgetGalleryScene {
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
    theme: Theme,
    show_help: bool,
    zones: RefCell<ScopedZoneRegistry<usize>>,
    area: Rect,
}

impl WidgetGalleryScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            selected: 0,
            checkbox: Checkbox::new(WidgetId::new(10), "Enable Feature"),
            radio: Radio::new(WidgetId::new(11), "Selected"),
            slider: Slider::new(WidgetId::new(12)).with_range(0.0, 100.0),
            spinner: Spinner::new(WidgetId::new(13)),
            toggle: Toggle::new(WidgetId::new(14), "Dark Mode"),
            select: Select::new(WidgetId::new(15))
                .with_options(vec!["Red".into(), "Green".into(), "Blue".into()]),
            search: SearchInput::new(WidgetId::new(16)),
            progress: ProgressBar::new(WidgetId::new(17)),
            button: Button::with_id(WidgetId::new(18), "Click Me!"),
            theme,
            show_help: false,
            zones: RefCell::new(ScopedZoneRegistry::new()),
            area: Rect::new(0, 0, 80, 24),
        }
    }

    fn widget_mut(&mut self, slot: usize) -> &mut dyn dracon_terminal_engine::framework::widget::Widget {
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
        let (row, col, ..) = SLOTS[slot];
        let rows = 3u16;
        let cols = if row == 0 { 4 } else if row == 1 { 3 } else { 2 };

        let card_w = area.width.saturating_sub(2) / cols;
        let card_h = area.height.saturating_sub(4) / rows;

        let x = area.x + 1 + col as u16 * card_w;
        let y = area.y + 2 + row as u16 * card_h;

        Rect::new(x, y, card_w.saturating_sub(1), card_h.saturating_sub(1))
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::dark(), Theme::light(), Theme::cyberpunk(), Theme::dracula(),
            Theme::nord(), Theme::catppuccin_mocha(), Theme::gruvbox_dark(),
            Theme::tokyo_night(), Theme::solarized_dark(), Theme::solarized_light(),
            Theme::one_dark(), Theme::rose_pine(), Theme::kanagawa(),
            Theme::everforest(), Theme::monokai(), Theme::warm(),
            Theme::cool(), Theme::forest(), Theme::sunset(), Theme::mono(),
        ];
        let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];
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
}

impl Scene for WidgetGalleryScene {
    fn scene_id(&self) -> &str { "widget_gallery" }

    fn render(&self, area: Rect) -> Plane {
        let t = self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        let title = " Widget Gallery ";
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, 2, 0, title, t.primary, t.bg, true);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 2), 0,
                  &theme_label, t.secondary, t.bg, false);

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
        for (slot, &(_row, _col, name, icon)) in SLOTS.iter().enumerate() {
            let rect = self.slot_rect(slot, area);
            let is_selected = slot == self.selected;
            render_card_border(&mut plane, rect, t, is_selected);

            let title = format!("{} {}", icon, name);
            draw_text(&mut plane, rect.x + 1, rect.y + 1, &title, t.primary, t.surface, true);

            let widget_area = Rect::new(
                rect.x + 1, rect.y + 2,
                rect.width.saturating_sub(2), rect.height.saturating_sub(3),
            );
            if widget_area.width >= 4 && widget_area.height >= 1 {
                self.zones.borrow_mut().register(
                    slot, widget_area.x, widget_area.y, widget_area.width, widget_area.height,
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
                blit_to(&mut plane, &mut w_plane, widget_area.x as usize, widget_area.y as usize);

                let state_y = widget_area.y + widget_area.height + 1;
                if state_y < rect.y + rect.height - 1 {
                    let state = match slot {
                        0 => format!("checked: {}", self.checkbox.is_checked()),
                        1 => format!("selected: {}", self.radio.is_selected()),
                        2 => format!("on: {}", self.toggle.is_on()),
                        3 => format!("frame: '{}'", self.spinner.current_frame()),
                        4 => format!("value: {:.0}", self.slider.value()),
                        5 => format!("selected: {}", self.select.selected_label().unwrap_or("none")),
                        6 => format!("query: '{}'", self.search.query()),
                        7 => format!("progress: {:.0}%", self.progress.progress() * 100.0),
                        8 => String::from("[Click me]"),
                        _ => String::new(),
                    };
                    draw_text(&mut plane, rect.x + 1, state_y, &state, t.fg_muted, t.surface, false);
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
        let nav = " ↑↓←→ nav | Enter activate | Tab: theme | B: back | ?: help ";
        draw_text(&mut plane, 2, footer_y, nav, t.fg_muted, t.bg, false);

        // Help overlay
        if self.show_help {
            draw_help_overlay(&mut plane, area, t);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                self.show_help = false;
            }
            return true;
        }

        match key.code {
            KeyCode::Char('?') => { self.show_help = true; true }
            KeyCode::Tab | KeyCode::Char('t') => { self.cycle_theme(); true }
            KeyCode::Right | KeyCode::Down => {
                self.selected = (self.selected + 1) % SLOTS.len();
                true
            }
            KeyCode::Left | KeyCode::Up => {
                self.selected = if self.selected == 0 { SLOTS.len() - 1 } else { self.selected - 1 };
                true
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.widget_mut(self.selected).handle_key(key)
            }
            _ => self.widget_mut(self.selected).handle_key(key),
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let zone_slot = self.zones.borrow().dispatch(col, row);
        if let Some(slot) = zone_slot {
            if let MouseEventKind::Down(MouseButton::Left) = kind {
                self.selected = slot;
            }
            let rect = self.slot_rect(slot, self.area());
            let rel_col = col.saturating_sub(rect.x + 1);
            let rel_row = row.saturating_sub(rect.y + 2);
            return self.widget_mut(slot).handle_mouse(kind, rel_col, rel_row);
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
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

    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
}

// ═══════════════════════════════════════════════════════════════════════════════
// RENDERING HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = (y * plane.width + x + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch, fg, bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false, skip: false,
            };
        }
    }
}

fn render_card_border(plane: &mut Plane, rect: Rect, t: Theme, selected: bool) {
    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
    let border = if selected { t.primary } else { t.outline };
    let bg = if selected { t.surface_elevated } else { t.surface };
    if w < 3 || h < 3 { return; }
    for row in y..y + h {
        for col in x..x + w {
            let idx = (row * plane.width + col) as usize;
            if idx >= plane.cells.len() { continue; }
            plane.cells[idx].bg = bg;
            let is_border = row == y || row == y + h - 1 || col == x || col == x + w - 1;
            if is_border {
                plane.cells[idx].fg = border;
                plane.cells[idx].char = match (row == y, row == y + h - 1, col == x, col == x + w - 1) {
                    (true, _, true, _) => '╭',
                    (true, _, _, true) => '╮',
                    (_, true, true, _) => '╰',
                    (_, true, _, true) => '╯',
                    (true, true, _, _) | (_, _, true, true) => '─',
                    _ => '│',
                };
            } else {
                plane.cells[idx].char = ' ';
                plane.cells[idx].fg = t.fg;
            }
        }
    }
}

fn blit_to(dest: &mut Plane, src: &mut Plane, offset_x: usize, offset_y: usize) {
    for i in 0..src.cells.len() {
        let cell = &src.cells[i];
        if cell.char == '\0' || cell.transparent { continue; }
        let row = i / src.width as usize;
        let col = i % src.width as usize;
        let dy = offset_y + row;
        let dx = offset_x + col;
        if dy >= dest.height as usize || dx >= dest.width as usize { continue; }
        let idx = dy * dest.width as usize + dx;
        if idx < dest.cells.len() {
            dest.cells[idx] = cell.clone();
        }
    }
}

fn draw_help_overlay(plane: &mut Plane, area: Rect, t: Theme) {
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
        let top = (hy * area.width + x) as usize;
        let bot = ((hy + hh - 1) * area.width + x) as usize;
        if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
        if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
    }
    for y in hy..hy + hh {
        let left = (y * area.width + hx) as usize;
        let right = (y * area.width + hx + hw - 1) as usize;
        if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
        if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
    }

    let title = "Widget Gallery Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

    let shortcuts = [
        ("↑↓←→", "Navigate cards"),
        ("Enter", "Activate widget"),
        ("Tab/t", "Cycle theme"),
        ("B/Esc", "Back to showcase"),
        ("?", "Toggle help"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = hy + 3 + i as u16;
        draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
        draw_text(plane, hx + 14, row, desc, t.fg, t.surface_elevated, false);
    }
}

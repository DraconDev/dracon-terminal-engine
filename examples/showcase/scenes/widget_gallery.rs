//! Embedded Widget Workshop scene for the showcase.
//!
//! Interactive playground: select a widget from the sidebar, interact with it
//! in the main panel, and see live state updates in the properties inspector.
//! Like Storybook for TUI widgets.

use crate::scenes::shared_helpers::{
    blit_to, draw_focus_ring, draw_text, draw_text_clipped, render_help_overlay,
};
use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::hitzone::ScopedZoneRegistry;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, Checkbox, ColorPicker, ProgressBar, ProgressRing, Radio, SearchInput, Select, Slider,
    Spinner, TagsInput, Toggle, StatusBar, StatusSegment,
};
use dracon_terminal_engine::input::event::{
    KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind,
};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;

// ── Widget catalog (icon, name, description) ────────────────────────────────

const WIDGETS: &[(&str, &str, &str)] = &[
    ("☑", "Checkbox", "Toggleable on/off state"),
    ("◎", "Radio", "Single selection from options"),
    ("◑", "Toggle", "Boolean switch with label"),
    ("◌", "Spinner", "Animated loading indicator"),
    ("━━", "Slider", "Continuous range control"),
    ("▼", "Select", "Dropdown selection list"),
    ("⌕", "SearchInput", "Text input with submit"),
    ("▓", "ProgressBar", "Linear progress indicator"),
    ("▣", "Button", "Clickable action button"),
    ("◐", "ColorPicker", "Interactive color selection"),
    ("◉", "ProgressRing", "Circular progress indicator"),
    ("#️⃣", "TagsInput", "Multi-tag composition"),
];

const SIDEBAR_WIDTH: u16 = 20;

pub struct WidgetGalleryScene {
    selected: usize,
    hovered: Option<usize>,
    checkbox: Checkbox,
    radio: Radio,
    slider: Slider,
    spinner: Spinner,
    toggle: Toggle,
    select: Select,
    search: SearchInput,
    progress: ProgressBar,
    button: Button,
    color_picker: ColorPicker,
    progress_ring: ProgressRing,
    tags_input: TagsInput,
    theme: Theme,
    show_help: bool,
    dirty: bool,
    zones: RefCell<ScopedZoneRegistry<usize>>,
    area: std::cell::Cell<Rect>,
    keybindings: KeybindingSet,
    button_clicks: u32,
    button_bridge: Rc<RefCell<bool>>,
    status_bar: RefCell<StatusBar>,
}

impl WidgetGalleryScene {
    pub fn new(theme: Theme) -> Self {
        let button_bridge = Rc::new(RefCell::new(false));
        let button_bridge_cb = Rc::clone(&button_bridge);
        let status_bar = StatusBar::new(WidgetId::new(2018))
            .add_segment(StatusSegment::new(
                "↑↓:navigate | Enter:interact | Type:input | F1:help | Esc:back",
            ))
            .with_theme(theme.clone());
        Self {
            selected: 0,
            hovered: None,
            checkbox: Checkbox::new(WidgetId::new(10), "Enable Feature"),
            radio: Radio::new(WidgetId::new(11), "Option A"),
            slider: Slider::new(WidgetId::new(12)).with_range(0.0, 100.0),
            spinner: Spinner::new(WidgetId::new(13)),
            toggle: Toggle::new(WidgetId::new(14), "Dark Mode"),
            select: Select::new(WidgetId::new(15)).with_options(vec![
                "Red".into(),
                "Green".into(),
                "Blue".into(),
            ]),
            search: SearchInput::new(WidgetId::new(16)),
            progress: ProgressBar::new(WidgetId::new(17)),
            button: Button::with_id(WidgetId::new(18), "  Click Me!  ").on_click(move || {
                *button_bridge_cb.borrow_mut() = true;
            }),
            color_picker: ColorPicker::new().with_theme(theme.clone()),
            progress_ring: ProgressRing::new(0.65),
            tags_input: TagsInput::new(vec!["rust".to_string(), "tui".to_string()])
                .with_theme(theme.clone()),
            theme,
            show_help: false,
            dirty: true,
            zones: RefCell::new(ScopedZoneRegistry::new()),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            button_clicks: 0,
            button_bridge,
            status_bar: RefCell::new(status_bar),
        }
    }

    fn sync_button_bridge(&mut self) {
        if *self.button_bridge.borrow() {
            *self.button_bridge.borrow_mut() = false;
            self.button_clicks += 1;
        }
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
            9 => &mut self.color_picker,
            10 => &mut self.progress_ring,
            11 => &mut self.tags_input,
            _ => &mut self.checkbox,
        }
    }

    fn widget_state(&self) -> String {
        match self.selected {
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
            8 => format!("clicks: {}", self.button_clicks),
            9 => format!("hex: {}", self.color_picker.hex()),
            10 => format!("progress: {:.0}%", self.progress_ring.progress() * 100.0),
            11 => format!("tags: {}", self.tags_input.tags().len()),
            _ => String::new(),
        }
    }

    fn render_sidebar(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let max_x = SIDEBAR_WIDTH;

        // Sidebar header
        draw_text_clipped(
            plane,
            1,
            0,
            " Widgets ",
            max_x,
            t.fg_on_accent,
            t.primary,
            true,
        );

        // Widget list
        for (i, (icon, name, _desc)) in WIDGETS.iter().enumerate() {
            let row = i as u16 + 1;
            if row >= area.height.saturating_sub(1) {
                break;
            }

            let is_selected = i == self.selected;
            let is_hovered = self.hovered == Some(i);

            // Background
            let bg = if is_selected {
                t.primary
            } else if is_hovered {
                t.hover_bg
            } else {
                t.surface
            };
            let fg = if is_selected { t.fg_on_accent } else { t.fg };
            let style = if is_selected {
                Styles::BOLD
            } else {
                Styles::empty()
            };

            // Fill row background
            for x in 0..SIDEBAR_WIDTH {
                let idx = (row * plane.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ' ';
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].style = style;
                    plane.cells[idx].transparent = false;
                }
            }

            // Hit zone
            self.zones
                .borrow_mut()
                .register(i, 0, row, SIDEBAR_WIDTH, 1);

            // Icon + name
            let entry = format!(" {} {}", icon, name);
            draw_text(plane, 1, row, &entry, fg, bg, is_selected);

            // Selected indicator
            if is_selected {
                let idx = (row * plane.width) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '▸';
                    plane.cells[idx].fg = t.fg_on_accent;
                    plane.cells[idx].transparent = false;
                }
            }
        }
    }

    fn render_demo(&self, plane: &mut Plane, area: Rect, div_x: u16) {
        let t = &self.theme;
        let panel_x = div_x + 1;
        let panel_w = area.width.saturating_sub(panel_x + 1);
        let max_x = panel_x + panel_w;

        let (icon, name, desc) = WIDGETS[self.selected];

        // Title: icon + name + description
        let title = format!("{} {} ", icon, name);
        draw_text_clipped(plane, panel_x, 0, &title, max_x, t.primary, t.bg, true);
        draw_text_clipped(
            plane,
            panel_x + title.len() as u16,
            0,
            desc,
            max_x,
            t.fg_muted,
            t.bg,
            false,
        );

        // Divider
        for x in panel_x..max_x.min(area.width) {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── Interactive Widget Area ────────────────────────────
        let demo_y = 2;
        let demo_h = area.height.saturating_sub(8);

        // Demo card background
        let card_x = panel_x;
        let card_w = panel_w;
        for y in demo_y..demo_y + demo_h {
            for x in card_x..card_x + card_w {
                if x >= area.width {
                    break;
                }
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Card border
        draw_focus_ring(plane, card_x, demo_y, card_w, demo_h, t.outline);

        // Render the actual widget
        let widget_area = Rect::new(
            card_x + 2,
            demo_y + 1,
            card_w.saturating_sub(4),
            demo_h.saturating_sub(2),
        );

        if widget_area.width >= 4 && widget_area.height >= 1 {
            let w_plane = match self.selected {
                0 => self.checkbox.render(widget_area),
                1 => self.radio.render(widget_area),
                2 => self.toggle.render(widget_area),
                3 => self.spinner.render(widget_area),
                4 => self.slider.render(widget_area),
                5 => self.select.render(widget_area),
                6 => self.search.render(widget_area),
                7 => self.progress.render(widget_area),
                8 => self.button.render(widget_area),
                9 => self.color_picker.render(widget_area),
                10 => self.progress_ring.render(widget_area),
                11 => self.tags_input.render(widget_area),
                _ => Plane::new(0, 0, 0),
            };
            blit_to(
                plane,
                &w_plane,
                widget_area.x as usize,
                widget_area.y as usize,
            );
        }

        // ── Properties Inspector ───────────────────────────────
        let props_y = demo_y + demo_h + 1;
        if props_y < area.height.saturating_sub(2) {
            draw_text_clipped(
                plane, panel_x, props_y, "State: ", max_x, t.fg_muted, t.bg, false,
            );
            draw_text_clipped(
                plane,
                panel_x + 7,
                props_y,
                &self.widget_state(),
                max_x,
                t.primary,
                t.bg,
                true,
            );

            // Keyboard hints
            let hints = self.keyboard_hints();
            let hint_y = props_y + 1;
            if hint_y < area.height.saturating_sub(2) {
                draw_text_clipped(
                    plane,
                    panel_x,
                    hint_y,
                    "Interact: ",
                    max_x,
                    t.fg_muted,
                    t.bg,
                    false,
                );
                draw_text_clipped(plane, panel_x + 10, hint_y, hints, max_x, t.fg, t.bg, false);
            }
        }
    }

    fn keyboard_hints(&self) -> &'static str {
        match self.selected {
            0 => "Space: toggle",
            1 => "Space: select",
            2 => "Space: toggle",
            3 => "Animating automatically",
            4 => "←/→: adjust  |  PgUp/Dn: step 10",
            5 => "←/→: change selection",
            6 => "Type: input  |  Enter: submit",
            7 => "Progress demo (static)",
            8 => "Enter/Click: activate",
            9 => "Tab: cycle slider  |  ←/→: adjust",
            10 => "Progress demo (static)",
            11 => "Type: tag  |  Enter: add  |  Backspace: remove",
            _ => "",
        }
    }
}

impl Scene for WidgetGalleryScene {

    fn on_enter(&mut self) {
        self.show_help = false;
        self.dirty = true;
    }

    fn on_exit(&mut self) {
        self.show_help = false;
    }


    fn scene_id(&self) -> &str {
        "widget_gallery"
    }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        draw_text(&mut plane, 2, 0, " Widget Workshop ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(
            &mut plane,
            area.width.saturating_sub(theme_label.len() as u16 + 2),
            0,
            &theme_label,
            t.secondary,
            t.bg,
            false,
        );

        // Header divider
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── Left Sidebar ──────────────────────────────────────
        self.zones.borrow_mut().clear();
        self.render_sidebar(&mut plane, area);

        // Sidebar divider
        let div_x = SIDEBAR_WIDTH;
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * area.width + div_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // ── Right Panel ───────────────────────────────────────
        self.render_demo(&mut plane, area, div_x);

        // ── Footer ────────────────────────────────────────────
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        let nav = " ↑/↓: navigate  |  Enter: interact  |  B/Esc: back  |  ?: help ";
        draw_text(&mut plane, 2, footer_y, nav, t.fg_muted, t.surface, false);
        let count = format!(" {} widgets ", WIDGETS.len());
        draw_text(
            &mut plane,
            area.width.saturating_sub(count.len() as u16 + 2),
            footer_y,
            &count,
            t.fg_muted,
            t.surface,
            false,
        );

        // Help overlay
        if self.show_help {
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(
                &mut plane,
                area,
                t,
                "Widget Workshop Help",
                &[
                    ("↑/↓", "Navigate widget list"),
                    ("Enter", "Interact with selected widget"),
                    ("Type", "Input into text widgets"),
                    (back_key, "Back"),
                    ("?", "Toggle this help"),
                ],
            );
        }

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_area = ratatui::layout::Rect::new(0, sb_y, area.width, 1);
        self.status_bar.borrow_mut().set_area(sb_area);
        let sb_plane = self.status_bar.borrow().render(sb_area);
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        match key.code {
            KeyCode::Up if key.modifiers.is_empty() => {
                self.selected = if self.selected == 0 {
                    WIDGETS.len() - 1
                } else {
                    self.selected - 1
                };
                self.hovered = None;
                true
            }
            KeyCode::Down if key.modifiers.is_empty() => {
                self.selected = (self.selected + 1) % WIDGETS.len();
                self.hovered = None;
                true
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let result = self.widget_mut(self.selected).handle_key(key);
                self.sync_button_bridge();
                result
            }
            _ => {
                let result = self.widget_mut(self.selected).handle_key(key);
                self.sync_button_bridge();
                result
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Check sidebar zones first
        let zone_slot = self.zones.borrow().dispatch(col, row);
        if let Some(slot) = zone_slot {
            if let MouseEventKind::Down(MouseButton::Left) = kind {
                self.selected = slot;
            }
            if let MouseEventKind::Moved = kind {
                self.hovered = Some(slot);
            }
            return true;
        }

        // Clear hover if mouse is outside sidebar
        if col < SIDEBAR_WIDTH && self.hovered.is_some() {
            self.hovered = None;
        }

        // Handle widget interaction in demo area
        let area = self.area.get();
        let demo_x = SIDEBAR_WIDTH + 1;
        if col >= demo_x && row >= 2 {
            let demo_w = area.width.saturating_sub(demo_x + 1);
            let card_x = demo_x;
            let card_w = demo_w;
            let demo_h = area.height.saturating_sub(8);
            let card_y = 2;

            if col >= card_x + 2
                && col < card_x + card_w - 2
                && row > card_y
                && row < card_y + demo_h - 1
            {
                let rel_col = col.saturating_sub(card_x + 2);
                let rel_row = row.saturating_sub(card_y + 1);
                let result = self
                    .widget_mut(self.selected)
                    .handle_mouse(kind, rel_col, rel_row);
                self.sync_button_bridge();
                return result;
            }
        }

        false
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
        self.color_picker.on_theme_change(theme);
        self.progress_ring.on_theme_change(theme);
        self.tags_input.on_theme_change(theme);
        self.status_bar.borrow_mut().on_theme_change(theme);
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

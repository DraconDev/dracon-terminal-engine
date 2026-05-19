//! Embedded Widget Workshop scene for the showcase.
//!
//! Interactive playground: select a widget type, modify properties
//! via keyboard controls, and see live render updates. Like Storybook.

use crate::scenes::shared_helpers::{blit_to, draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::button::Button;
use dracon_terminal_engine::framework::widgets::checkbox::Checkbox;
use dracon_terminal_engine::framework::widgets::progress_bar::ProgressBar;
use dracon_terminal_engine::framework::widgets::progress_ring::ProgressRing;
use dracon_terminal_engine::framework::widgets::radio::Radio;
use dracon_terminal_engine::framework::widgets::slider::Slider;
use dracon_terminal_engine::framework::widgets::spinner::Spinner;
use dracon_terminal_engine::framework::widgets::toggle::Toggle;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::{Cell, RefCell};

#[derive(Clone, Copy, PartialEq)]
enum WidgetType {
    Button,
    Checkbox,
    Toggle,
    Radio,
    Slider,
    ProgressBar,
    ProgressRing,
    Spinner,
}

const WIDGET_NAMES: &[(&str, WidgetType)] = &[
    ("Button", WidgetType::Button),
    ("Checkbox", WidgetType::Checkbox),
    ("Toggle", WidgetType::Toggle),
    ("Radio", WidgetType::Radio),
    ("Slider", WidgetType::Slider),
    ("ProgressBar", WidgetType::ProgressBar),
    ("ProgressRing", WidgetType::ProgressRing),
    ("Spinner", WidgetType::Spinner),
];

pub struct WorkshopScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    selected_widget: Cell<usize>,
    // Live widget props
    prop_int: Cell<i32>,    // generic int prop (progress, slider value, etc.)
    prop_bool: Cell<bool>,  // generic bool prop (checked, toggled, etc.)
    prop_label: Cell<&'static str>,
    // Widgets (RefCell for render(&self) pattern)
    button: RefCell<Button>,
    checkbox: RefCell<Checkbox>,
    toggle: RefCell<Toggle>,
    radio: RefCell<Radio>,
    slider: RefCell<Slider>,
    progress_bar: RefCell<ProgressBar>,
    progress_ring: RefCell<ProgressRing>,
    spinner: RefCell<Spinner>,
    dirty: bool,
    area: Cell<Rect>,
}

impl WorkshopScene {
    pub fn new(theme: Theme) -> Self {
        let button = RefCell::new(Button::new("Click Me").with_theme(theme.clone()));
        let checkbox = RefCell::new(Checkbox::new(WidgetId::new(2), "Enable feature").with_theme(theme.clone()));
        let toggle = RefCell::new(Toggle::new(WidgetId::new(3), "Dark mode").with_theme(theme.clone()));
        let radio = RefCell::new(Radio::new(WidgetId::new(4), "Option A").with_theme(theme.clone()));
        let slider = RefCell::new(Slider::new(WidgetId::new(5)).with_theme(theme.clone()));
        let progress_bar = RefCell::new(ProgressBar::new(WidgetId::new(6)).with_theme(theme.clone()));
        let progress_ring = RefCell::new(ProgressRing::new(0.5).with_theme(theme.clone()).with_size(8).show_percentage(true));
        let spinner = RefCell::new(Spinner::new(WidgetId::new(8)).with_theme(theme.clone()));

        Self {
            theme,
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            selected_widget: Cell::new(0),
            prop_int: Cell::new(50),
            prop_bool: Cell::new(false),
            prop_label: Cell::new("Click Me"),
            button,
            checkbox,
            toggle,
            radio,
            slider,
            progress_bar,
            progress_ring,
            spinner,
            dirty: true,
            area: Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn current_widget_type(&self) -> WidgetType {
        WIDGET_NAMES[self.selected_widget.get()].1
    }

    fn render_widget_list(&self, plane: &mut Plane, x: u16, y: u16) {
        let t = &self.theme;
        draw_text(plane, x, y, "Widgets", t.primary, t.bg, true);

        for (i, (name, _)) in WIDGET_NAMES.iter().enumerate() {
            let wy = y + 2 + i as u16;
            let is_selected = i == self.selected_widget.get();
            let fg = if is_selected { t.primary } else { t.fg_muted };
            let prefix = if is_selected { "► " } else { "  " };
            draw_text(plane, x, wy, &format!("{}{}", prefix, name), fg, t.bg, is_selected);
        }
    }

    fn render_props(&self, plane: &mut Plane, x: u16, y: u16, w: u16) {
        let t = &self.theme;
        draw_text(plane, x, y, "Properties", t.primary, t.bg, true);

        let wt = self.current_widget_type();
        let py = y + 2;

        // Common: label
        draw_text(plane, x, py, &format!("Label: {}", self.prop_label.get()), t.fg, t.bg, false);

        // Widget-specific props
        match wt {
            WidgetType::Button => {
                draw_text(plane, x, py + 1, &format!("Pressed: {}", self.prop_bool.get()), t.fg, t.bg, false);
                draw_text(plane, x, py + 2, "↑/↓: change label | Enter: press", t.fg_muted, t.bg, false);
            }
            WidgetType::Checkbox => {
                draw_text(plane, x, py + 1, &format!("Checked: {}", self.prop_bool.get()), t.fg, t.bg, false);
                draw_text(plane, x, py + 2, "Space: toggle | ↑/↓: change label", t.fg_muted, t.bg, false);
            }
            WidgetType::Toggle => {
                draw_text(plane, x, py + 1, &format!("On: {}", self.prop_bool.get()), t.fg, t.bg, false);
                draw_text(plane, x, py + 2, "Space: toggle | ↑/↓: change label", t.fg_muted, t.bg, false);
            }
            WidgetType::Radio => {
                draw_text(plane, x, py + 1, &format!("Selected: {}", self.prop_bool.get()), t.fg, t.bg, false);
                draw_text(plane, x, py + 2, "Space: select | ↑/↓: change label", t.fg_muted, t.bg, false);
            }
            WidgetType::Slider => {
                draw_text(plane, x, py + 1, &format!("Value: {}", self.prop_int.get()), t.primary, t.bg, true);
                // Visual slider
                let bar_w = (w as usize).saturating_sub(2);
                let filled = (self.prop_int.get() as f32 / 100.0 * bar_w as f32) as usize;
                for bx in 0..bar_w {
                    let idx = (py + 2) as usize * plane.width as usize + x as usize + bx;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = if bx < filled { '█' } else { '░' };
                        plane.cells[idx].fg = if bx < filled { t.primary } else { t.fg_muted };
                        plane.cells[idx].transparent = false;
                    }
                }
                draw_text(plane, x, py + 3, "←/→: adjust value", t.fg_muted, t.bg, false);
            }
            WidgetType::ProgressBar => {
                draw_text(plane, x, py + 1, &format!("Progress: {}%", self.prop_int.get()), t.primary, t.bg, true);
                draw_text(plane, x, py + 2, "←/→: adjust progress", t.fg_muted, t.bg, false);
            }
            WidgetType::ProgressRing => {
                draw_text(plane, x, py + 1, &format!("Progress: {}%", self.prop_int.get()), t.primary, t.bg, true);
                draw_text(plane, x, py + 2, "←/→: adjust progress", t.fg_muted, t.bg, false);
            }
            WidgetType::Spinner => {
                draw_text(plane, x, py + 1, "Spinning...", t.primary, t.bg, false);
                draw_text(plane, x, py + 2, "Always animating", t.fg_muted, t.bg, false);
            }
        }
    }

    fn render_preview(&self, plane: &mut Plane, x: u16, y: u16, w: u16, h: u16) {
        let t = &self.theme;

        // Preview border
        draw_text(plane, x, y, "Preview", t.primary, t.bg, true);
        for bx in 0..w {
            let idx = ((y + 1) * plane.width + x + bx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Background for preview area
        for py in 0..h.saturating_sub(2) {
            for px in 0..w {
                let idx = ((y + 2 + py) * plane.width + x + px) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        let preview_y = y + 3;
        let preview_x = x + 4;
        let wt = self.current_widget_type();

        // Sync widget state
        self.sync_widget_state();

        match wt {
            WidgetType::Button => {
                let area = Rect::new(preview_x, preview_y, 16, 3);
                let widget_plane = self.button.borrow().render(area);
                blit_to(plane, &widget_plane, area.x as usize, area.y as usize);
            }
            WidgetType::Checkbox => {
                let area = Rect::new(preview_x, preview_y, 20, 3);
                let widget_plane = self.checkbox.borrow().render(area);
                blit_to(plane, &widget_plane, area.x as usize, area.y as usize);
            }
            WidgetType::Toggle => {
                let area = Rect::new(preview_x, preview_y, 18, 3);
                let widget_plane = self.toggle.borrow().render(area);
                blit_to(plane, &widget_plane, area.x as usize, area.y as usize);
            }
            WidgetType::Radio => {
                let area = Rect::new(preview_x, preview_y, 16, 3);
                let widget_plane = self.radio.borrow().render(area);
                blit_to(plane, &widget_plane, area.x as usize, area.y as usize);
            }
            WidgetType::Slider => {
                let area = Rect::new(preview_x, preview_y, 24, 3);
                let widget_plane = self.slider.borrow().render(area);
                blit_to(plane, &widget_plane, area.x as usize, area.y as usize);
            }
            WidgetType::ProgressBar => {
                let area = Rect::new(preview_x, preview_y, w.saturating_sub(8), 3);
                let widget_plane = self.progress_bar.borrow().render(area);
                blit_to(plane, &widget_plane, area.x as usize, area.y as usize);
            }
            WidgetType::ProgressRing => {
                let area = Rect::new(preview_x, preview_y, 12, 4);
                let widget_plane = self.progress_ring.borrow().render(area);
                blit_to(plane, &widget_plane, area.x as usize, area.y as usize);
            }
            WidgetType::Spinner => {
                let area = Rect::new(preview_x, preview_y, 8, 1);
                let widget_plane = self.spinner.borrow().render(area);
                blit_to(plane, &widget_plane, area.x as usize, area.y as usize);
            }
        }
    }

    fn sync_widget_state(&self) {
        let progress = self.prop_int.get() as f32 / 100.0;
        let checked = self.prop_bool.get();

        self.progress_bar.borrow_mut().set_progress(progress);
        self.progress_ring.borrow_mut().set_progress(progress as f64);
        self.slider.borrow_mut().set_value(self.prop_int.get() as f32 / 100.0);

        if checked {
            self.checkbox.borrow_mut().check();
            if !self.toggle.borrow().is_on() {
                self.toggle.borrow_mut().toggle();
            }
            self.radio.borrow_mut().select();
        } else {
            self.checkbox.borrow_mut().uncheck();
            if self.toggle.borrow().is_on() {
                self.toggle.borrow_mut().toggle();
            }
            self.radio.borrow_mut().deselect();
        }
    }
}

impl Scene for WorkshopScene {
    fn scene_id(&self) -> &str { "workshop" }

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

        // Left panel: widget list
        self.render_widget_list(&mut plane, 2, 2);

        // Vertical divider
        let div_x = 16u16;
        for y in 2..area.height.saturating_sub(1) {
            let idx = (y * area.width + div_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Middle panel: properties
        let props_x = div_x + 2;
        let props_w = 24u16;
        self.render_props(&mut plane, props_x, 2, props_w);

        // Second vertical divider
        let div2_x = props_x + props_w + 2;
        for y in 2..area.height.saturating_sub(1) {
            let idx = (y * area.width + div2_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Right panel: live preview
        let preview_x = div2_x + 2;
        let preview_w = area.width.saturating_sub(preview_x + 2);
        let preview_h = area.height.saturating_sub(4);
        self.render_preview(&mut plane, preview_x, 2, preview_w, preview_h);

        // Footer
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let _current_name = WIDGET_NAMES[self.selected_widget.get()].0;
        let footer = format!(
            " ↑/↓:pick widget | ←/→:adjust | Space:toggle | {}:help | {}:back ",
            help_key, back_key,
        );
        let fy = area.height.saturating_sub(1);
        for (i, c) in footer.chars().enumerate() {
            let idx = (fy * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        if self.show_help {
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(&mut plane, area, &self.theme, "Widget Workshop — Help", &[
                ("↑/↓", "Switch between widgets"),
                ("←/→", "Adjust numeric properties"),
                ("Space", "Toggle boolean properties"),
                ("Enter", "Activate/press widget"),
                (back_key, "Back"),
            ]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        match key.code {
            KeyCode::Up => {
                let cur = self.selected_widget.get();
                if cur > 0 {
                    self.selected_widget.set(cur - 1);
                    self.reset_props();
                    self.dirty = true;
                }
                true
            }
            KeyCode::Down => {
                let cur = self.selected_widget.get();
                if cur + 1 < WIDGET_NAMES.len() {
                    self.selected_widget.set(cur + 1);
                    self.reset_props();
                    self.dirty = true;
                }
                true
            }
            KeyCode::Left => {
                let val = self.prop_int.get().saturating_sub(5).max(0);
                self.prop_int.set(val);
                self.dirty = true;
                true
            }
            KeyCode::Right => {
                let val = self.prop_int.get().saturating_add(5).min(100);
                self.prop_int.set(val);
                self.dirty = true;
                true
            }
            KeyCode::Char(' ') => {
                self.prop_bool.set(!self.prop_bool.get());
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                self.prop_bool.set(!self.prop_bool.get());
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();

        match kind {
            MouseEventKind::Down(_) => {
                // Widget list: rows 4..4+8 (y=2 start, +2 header)
                let list_y = 4u16;
                if (2..16).contains(&col) && (list_y..list_y + WIDGET_NAMES.len() as u16).contains(&row) {
                    let idx = (row - list_y) as usize;
                    if idx < WIDGET_NAMES.len() && idx != self.selected_widget.get() {
                        self.selected_widget.set(idx);
                        self.reset_props();
                        self.dirty = true;
                    }
                    return true;
                }

                // Properties panel slider bar (only for Slider widget)
                if self.current_widget_type() == WidgetType::Slider {
                    let props_x = 18u16; // div_x(16) + 2
                    let bar_y = 6u16; // y=2 + py+2 = 2+2+2 = 6
                    if col >= props_x && col < props_x + 24 && row == bar_y {
                        let rel = (col - props_x) as i32;
                        let bar_w = 22i32; // w-2
                        let val = (rel as f32 / bar_w as f32 * 100.0) as i32;
                        self.prop_int.set(val.clamp(0, 100));
                        self.dirty = true;
                        return true;
                    }
                }

                // Preview panel: click to interact with widget
                let div2_x = 18u16 + 24u16 + 2u16; // props_x + props_w + 2 = 44
                let preview_x = div2_x + 2; // 46
                if col >= preview_x && row >= 4 && row < area.height.saturating_sub(2) {
                    let wt = self.current_widget_type();
                    match wt {
                        WidgetType::Button | WidgetType::Checkbox | WidgetType::Toggle | WidgetType::Radio => {
                            self.prop_bool.set(!self.prop_bool.get());
                            self.dirty = true;
                        }
                        WidgetType::Slider => {
                            let rel = (col.saturating_sub(preview_x)) as i32;
                            let bar_w = (area.width.saturating_sub(preview_x + 2)) as i32;
                            if bar_w > 0 {
                                let val = (rel as f32 / bar_w as f32 * 100.0) as i32;
                                self.prop_int.set(val.clamp(0, 100));
                                self.dirty = true;
                            }
                        }
                        WidgetType::ProgressBar | WidgetType::ProgressRing => {
                            let rel = (col.saturating_sub(preview_x)) as i32;
                            let bar_w = (area.width.saturating_sub(preview_x + 2)) as i32;
                            if bar_w > 0 {
                                let val = (rel as f32 / bar_w as f32 * 100.0) as i32;
                                self.prop_int.set(val.clamp(0, 100));
                                self.dirty = true;
                            }
                        }
                        WidgetType::Spinner => {} // display only
                    }
                    return true;
                }

                false
            }
            MouseEventKind::ScrollUp => {
                let cur = self.selected_widget.get();
                if cur > 0 {
                    self.selected_widget.set(cur - 1);
                    self.reset_props();
                    self.dirty = true;
                }
                true
            }
            MouseEventKind::ScrollDown => {
                let cur = self.selected_widget.get();
                if cur + 1 < WIDGET_NAMES.len() {
                    self.selected_widget.set(cur + 1);
                    self.reset_props();
                    self.dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.button.borrow_mut().on_theme_change(theme);
        self.checkbox.borrow_mut().on_theme_change(theme);
        self.toggle.borrow_mut().on_theme_change(theme);
        self.radio.borrow_mut().on_theme_change(theme);
        self.slider.borrow_mut().on_theme_change(theme);
        self.progress_bar.borrow_mut().on_theme_change(theme);
        self.progress_ring.borrow_mut().on_theme_change(theme);
        self.spinner.borrow_mut().on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { self.dirty || self.current_widget_type() == WidgetType::Spinner }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl WorkshopScene {
    fn reset_props(&self) {
        self.prop_int.set(50);
        self.prop_bool.set(false);
        match self.current_widget_type() {
            WidgetType::Button => self.prop_label.set("Click Me"),
            WidgetType::Checkbox => self.prop_label.set("Enable feature"),
            WidgetType::Toggle => self.prop_label.set("Dark mode"),
            WidgetType::Radio => self.prop_label.set("Option A"),
            WidgetType::Slider => self.prop_label.set("Volume"),
            WidgetType::ProgressBar => self.prop_label.set("Loading"),
            WidgetType::ProgressRing => self.prop_label.set("Progress"),
            WidgetType::Spinner => self.prop_label.set("Working"),
        }
    }
}



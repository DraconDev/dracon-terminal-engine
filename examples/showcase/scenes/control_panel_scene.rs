//! Control Panel scene — Select + Toggle + Checkbox + StatusBar + Profiler.
//!
//! A settings-like control panel with dropdown selects, toggle switches,
//! checkboxes, and a status bar showing live state.

#![allow(dead_code)]

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    Checkbox, Divider, Label, Profiler, Select, StatusBar, StatusSegment, Toggle,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::time::Duration;

/// Track select state externally since Select doesn't have select_next/prev
struct SelectState {
    widget: RefCell<Select>,
    index: RefCell<usize>,
    options: Vec<String>,
}

impl SelectState {
    fn new(widget: Select, options: Vec<String>) -> Self {
        Self {
            widget: RefCell::new(widget),
            index: RefCell::new(0),
            options,
        }
    }
    fn current_label(&self) -> String {
        let idx = *self.index.borrow();
        self.options.get(idx).cloned().unwrap_or_else(|| "?".into())
    }
    fn next(&self) {
        let mut idx = self.index.borrow_mut();
        *idx = (*idx + 1) % self.options.len();
        self.widget.borrow_mut().set_selected(*idx);
    }
    fn prev(&self) {
        let mut idx = self.index.borrow_mut();
        if *idx == 0 { *idx = self.options.len() - 1; } else { *idx -= 1; }
        self.widget.borrow_mut().set_selected(*idx);
    }
}

pub struct ControlPanelScene {
    theme: Theme,
    keybindings: KeybindingSet,
    // Selects (state tracked externally)
    theme_select: SelectState,
    font_select: SelectState,
    lang_select: SelectState,
    // Toggles
    dark_mode: RefCell<Toggle>,
    auto_save: RefCell<Toggle>,
    telemetry: RefCell<Toggle>,
    // Checkboxes
    line_numbers: RefCell<Checkbox>,
    word_wrap: RefCell<Checkbox>,
    minimap: RefCell<Checkbox>,
    // Profiler overlay
    profiler: RefCell<Profiler>,
    show_profiler: bool,
    // Status bar
    status_bar: RefCell<StatusBar>,
    // UI state
    focus_field: usize,
    show_help: bool,
    dirty: bool,
}

impl ControlPanelScene {
    pub fn new(theme: Theme) -> Self {
        let theme_opts = vec!["Nord".into(), "Cyberpunk".into(), "Dracula".into(), "Solarized".into(), "Gruvbox".into()];
        let font_opts = vec!["JetBrains Mono".into(), "Fira Code".into(), "Cascadia Code".into(), "Source Code Pro".into()];
        let lang_opts = vec!["Rust".into(), "TypeScript".into(), "Python".into(), "Go".into(), "C++".into()];

        let theme_select = SelectState::new(
            Select::new(WidgetId::new(1000)).with_options(theme_opts.clone()).with_theme(theme.clone()),
            theme_opts,
        );
        let font_select = SelectState::new(
            Select::new(WidgetId::new(1001)).with_options(font_opts.clone()).with_theme(theme.clone()),
            font_opts,
        );
        let lang_select = SelectState::new(
            Select::new(WidgetId::new(1002)).with_options(lang_opts.clone()).with_theme(theme.clone()),
            lang_opts,
        );

        let dark_mode = Toggle::new(WidgetId::new(1010), "Dark Mode").with_theme(theme.clone());
        let auto_save = Toggle::new(WidgetId::new(1011), "Auto Save").with_theme(theme.clone());
        let telemetry = Toggle::new(WidgetId::new(1012), "Telemetry").with_theme(theme.clone());

        let line_numbers = Checkbox::new(WidgetId::new(1020), "Line Numbers").with_theme(theme.clone());
        let word_wrap = Checkbox::new(WidgetId::new(1021), "Word Wrap").with_theme(theme.clone());
        let minimap = Checkbox::new(WidgetId::new(1022), "Minimap").with_theme(theme.clone());

        let profiler = Profiler::new(WidgetId::new(1030)).with_theme(theme.clone());

        let status_bar = StatusBar::new(WidgetId::new(1040))
            .add_segment(StatusSegment::new("Tab: focus | Space/↑/↓: change | P: profiler | F1: help | Esc: back"))
            .add_segment(StatusSegment::new("Select · Toggle · Checkbox · StatusBar"))
            .with_theme(theme.clone());

        Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            theme_select,
            font_select,
            lang_select,
            dark_mode: RefCell::new(dark_mode),
            auto_save: RefCell::new(auto_save),
            telemetry: RefCell::new(telemetry),
            line_numbers: RefCell::new(line_numbers),
            word_wrap: RefCell::new(word_wrap),
            minimap: RefCell::new(minimap),
            profiler: RefCell::new(profiler),
            show_profiler: false,
            status_bar: RefCell::new(status_bar),
            focus_field: 0,
            show_help: false,
            dirty: true,
        }
    }

    fn field_count() -> usize { 9 }
}

impl Scene for ControlPanelScene {
    fn on_enter(&mut self) {}
    fn on_exit(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // Title
        let title = Label::new("Control Panel").with_style(Styles::BOLD).with_theme(t.clone());
        let title_plane = title.render(Rect::new(0, 0, 15, 1));
        blit_to(&mut plane, &title_plane, 1, 0);
        draw_text(&mut plane, 17, 0, "— Select · Toggle · Checkbox · StatusBar", t.fg_muted, t.bg, false);

        // Divider: Appearance
        let div = Divider::new().with_label("Appearance").with_theme(t.clone());
        let div_plane = div.render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &div_plane, 0, 1);

        // Select fields (rows 2-4): render label + current value
        let select_data: [(&SelectState, &str); 3] = [
            (&self.theme_select, "Theme"),
            (&self.font_select, "Font"),
            (&self.lang_select, "Language"),
        ];
        for (i, (sel, label)) in select_data.iter().enumerate() {
            let y = 2 + i as u16;
            let is_focused = self.focus_field == i;
            let lbl_color = if is_focused { t.primary } else { t.fg_muted };
            let lbl_text = if is_focused { format!("▸ {:<10}", label) } else { format!("  {:<10}", label) };
            draw_text(&mut plane, 2, y, &lbl_text, lbl_color, t.bg, is_focused);

            // Render actual Select widget (collapsed, shows current value + ▼)
            let sel_area = Rect::new(14, y, 20, 1);
            sel.widget.borrow_mut().set_area(sel_area);
            let sel_plane = sel.widget.borrow().render(sel_area);
            blit_to(&mut plane, &sel_plane, 14, y as usize);
        }

        // Divider: Toggles
        let div2_y = 5;
        let div2 = Divider::new().with_label("Toggles").with_theme(t.clone());
        let div2_plane = div2.render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &div2_plane, 0, div2_y as usize);

        // Toggle fields (rows 6-8)
        let toggle_data: [(&RefCell<Toggle>, usize); 3] = [
            (&self.dark_mode, 3),
            (&self.auto_save, 4),
            (&self.telemetry, 8),
        ];
        for (i, (tog, field_idx)) in toggle_data.iter().enumerate() {
            let y = 6 + i as u16;
            let is_focused = self.focus_field == *field_idx;
            let t_area = Rect::new(2, y, 22, 1);
            tog.borrow_mut().set_area(t_area);
            let t_plane = tog.borrow().render(t_area);
            blit_to(&mut plane, &t_plane, 2, y as usize);
            if is_focused {
                draw_text(&mut plane, 25, y, "◀", t.primary, t.bg, false);
            }
        }

        // Divider: Checkboxes
        let div3_y = 9;
        let div3 = Divider::new().with_label("Editor").with_theme(t.clone());
        let div3_plane = div3.render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &div3_plane, 0, div3_y as usize);

        // Checkbox fields (rows 10-12)
        let cb_data: [(&RefCell<Checkbox>, usize); 3] = [
            (&self.line_numbers, 5),
            (&self.word_wrap, 6),
            (&self.minimap, 7),
        ];
        for (i, (cb, field_idx)) in cb_data.iter().enumerate() {
            let y = 10 + i as u16;
            let is_focused = self.focus_field == *field_idx;
            let cb_area = Rect::new(2, y, 22, 1);
            cb.borrow_mut().set_area(cb_area);
            let cb_plane = cb.borrow().render(cb_area);
            blit_to(&mut plane, &cb_plane, 2, y as usize);
            if is_focused {
                draw_text(&mut plane, 25, y, "◀", t.primary, t.bg, false);
            }
        }

        // Live state summary (right panel)
        let summary_x = area.width / 2 + 2;
        draw_text(&mut plane, summary_x, 2, "Current Settings", t.fg, t.bg, true);
        let lines = [
            format!("Theme:     {}", self.theme_select.current_label()),
            format!("Font:      {}", self.font_select.current_label()),
            format!("Language:  {}", self.lang_select.current_label()),
            format!("Dark Mode: {}", if self.dark_mode.borrow().is_on() { "ON" } else { "OFF" }),
            format!("Auto Save: {}", if self.auto_save.borrow().is_on() { "ON" } else { "OFF" }),
            format!("Lines:     {}", if self.line_numbers.borrow().is_checked() { "ON" } else { "OFF" }),
            format!("Word Wrap: {}", if self.word_wrap.borrow().is_checked() { "ON" } else { "OFF" }),
            format!("Minimap:   {}", if self.minimap.borrow().is_checked() { "ON" } else { "OFF" }),
            format!("Telemetry: {}", if self.telemetry.borrow().is_on() { "ON" } else { "OFF" }),
        ];
        for (i, line) in lines.iter().enumerate() {
            draw_text(&mut plane, summary_x, 4 + i as u16, line, t.fg_muted, t.bg, false);
        }

        // Profiler overlay
        if self.show_profiler {
            self.profiler.borrow_mut().record("render", Duration::from_micros(450), 1);
            self.profiler.borrow_mut().record("layout", Duration::from_micros(120), 1);
            self.profiler.borrow_mut().record("input", Duration::from_micros(50), 1);
            let prof_w = 30u16.min(area.width.saturating_sub(4));
            let prof_h = 8u16.min(area.height.saturating_sub(4));
            let prof_x = area.width.saturating_sub(prof_w + 2);
            let p_area = Rect::new(0, 0, prof_w, prof_h);
            self.profiler.borrow_mut().set_area(p_area);
            let p_plane = self.profiler.borrow().render(p_area);
            blit_to(&mut plane, &p_plane, prof_x as usize, 2);
        }

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self.status_bar.borrow().render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        if self.show_help {
            render_help_overlay(&mut plane, area, t, "Control Panel — Help", &[("Tab", "Cycle focus through fields"), ("Space", "Toggle / cycle select"), ("Up/Dn", "Change select option"), ("P", "Toggle profiler overlay"), ("Click", "Any field to interact"), ("F1", "Toggle this help"), ("Esc", "Back to showcase")]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::HELP, &key) || self.keybindings.matches(actions::BACK, &key) {
                self.show_help = false; self.dirty = true; return true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) { self.show_help = !self.show_help; self.dirty = true; return true; }
        if self.keybindings.matches(actions::BACK, &key) { return false; }

        match key.code {
            KeyCode::Tab => { self.focus_field = (self.focus_field + 1) % Self::field_count(); self.dirty = true; true }
            KeyCode::BackTab => { self.focus_field = (self.focus_field + Self::field_count() - 1) % Self::field_count(); self.dirty = true; true }
            KeyCode::Char(' ') => {
                match self.focus_field {
                    0 => self.theme_select.next(),
                    1 => self.font_select.next(),
                    2 => self.lang_select.next(),
                    3 => self.dark_mode.borrow_mut().toggle(),
                    4 => self.auto_save.borrow_mut().toggle(),
                    5 => self.line_numbers.borrow_mut().toggle(),
                    6 => self.word_wrap.borrow_mut().toggle(),
                    7 => self.minimap.borrow_mut().toggle(),
                    8 => self.telemetry.borrow_mut().toggle(),
                    _ => {}
                }
                self.dirty = true; true
            }
            KeyCode::Up => {
                match self.focus_field {
                    0 => self.theme_select.prev(),
                    1 => self.font_select.prev(),
                    2 => self.lang_select.prev(),
                    _ => {}
                }
                self.dirty = true; true
            }
            KeyCode::Down => {
                match self.focus_field {
                    0 => self.theme_select.next(),
                    1 => self.font_select.next(),
                    2 => self.lang_select.next(),
                    _ => {}
                }
                self.dirty = true; true
            }
            KeyCode::Char('p') if key.modifiers.is_empty() => { self.show_profiler = !self.show_profiler; self.dirty = true; true }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if let MouseEventKind::Down(_) = kind {
            // Select clicks (rows 2-4)
            for i in 0..3u16 {
                if row == 2 + i && col >= 14 {
                    self.focus_field = i as usize;
                    match i {
                        0 => self.theme_select.next(),
                        1 => self.font_select.next(),
                        2 => self.lang_select.next(),
                        _ => {}
                    }
                    self.dirty = true; return true;
                }
            }
            // Toggle clicks (rows 6-8)
            for i in 0..3u16 {
                if row == 6 + i && (2..24).contains(&col) {
                    self.focus_field = 3 + i as usize;
                    match i {
                        0 => self.dark_mode.borrow_mut().toggle(),
                        1 => self.auto_save.borrow_mut().toggle(),
                        2 => self.telemetry.borrow_mut().toggle(),
                        _ => {}
                    }
                    self.dirty = true; return true;
                }
            }
            // Checkbox clicks (rows 10-12)
            for i in 0..3u16 {
                if row == 10 + i && (2..24).contains(&col) {
                    self.focus_field = 5 + i as usize;
                    match i {
                        0 => self.line_numbers.borrow_mut().toggle(),
                        1 => self.word_wrap.borrow_mut().toggle(),
                        2 => self.minimap.borrow_mut().toggle(),
                        _ => {}
                    }
                    self.dirty = true; return true;
                }
            }
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.theme_select.widget.borrow_mut().on_theme_change(theme);
        self.font_select.widget.borrow_mut().on_theme_change(theme);
        self.lang_select.widget.borrow_mut().on_theme_change(theme);
        self.dark_mode.borrow_mut().on_theme_change(theme);
        self.auto_save.borrow_mut().on_theme_change(theme);
        self.telemetry.borrow_mut().on_theme_change(theme);
        self.line_numbers.borrow_mut().on_theme_change(theme);
        self.word_wrap.borrow_mut().on_theme_change(theme);
        self.minimap.borrow_mut().on_theme_change(theme);
        self.profiler.borrow_mut().on_theme_change(theme);
        self.status_bar.borrow_mut().on_theme_change(theme);
        self.dirty = true;
    }

    fn scene_id(&self) -> &str { "control_panel" }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}


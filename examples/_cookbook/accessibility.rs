//! Accessibility demo  -  demonstrates screen reader announcements via OSC 99.
//!
//! This example shows how widgets can implement the `Accessibility` trait
//! to expose metadata to screen readers (NVDA, VoiceOver, Orca).
//!
//! # Usage
//!
//! Run this in a terminal that supports OSC 99 announcements (most modern
//! terminals do). With a screen reader active, navigate with Tab/Shift+Tab
//! and interact with the controls to hear announcements.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::visuals::accessibility::{Accessibility, Role};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A simple accessible button widget.
struct AccessibleButton {
    id: WidgetId,
    area: Rect,
    label: String,
    pressed: bool,
    theme: Theme,
    dirty: bool,
}

impl AccessibleButton {
    fn new(id: WidgetId, label: &str, theme: Theme) -> Self {
        Self {
            id,
            area: Rect::default(),
            label: label.to_string(),
            pressed: false,
            theme,
            dirty: true,
        }
    }
}

impl Accessibility for AccessibleButton {
    fn role(&self) -> Role {
        Role::Button
    }

    fn label(&self) -> Option<&str> {
        Some(&self.label)
    }

    fn description(&self) -> Option<&str> {
        Some(if self.pressed {
            "pressed"
        } else {
            "not pressed"
        })
    }

    fn disabled(&self) -> bool {
        false
    }
}

impl Widget for AccessibleButton {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(if self.pressed {
            self.theme.primary
        } else {
            self.theme.surface
        });

        // Draw button border
        let label_len = self.label.len() as u16;
        let start_x = (area.width.saturating_sub(label_len + 2)) / 2;

        for x in 0..area.width {
            plane.cells[x as usize].char = '─';
            plane.cells[x as usize].fg = self.theme.outline;
            plane.cells[x as usize].transparent = false;

            let last_row_idx = ((area.height - 1) * area.width + x) as usize;
            if last_row_idx < plane.cells.len() {
                plane.cells[last_row_idx].char = '─';
                plane.cells[last_row_idx].fg = self.theme.outline;
                plane.cells[last_row_idx].transparent = false;
            }
        }

        for y in 0..area.height {
            plane.cells[(y * area.width) as usize].char = '│';
            plane.cells[(y * area.width) as usize].fg = self.theme.outline;
            plane.cells[(y * area.width) as usize].transparent = false;

            let right_idx = (y * area.width + area.width - 1) as usize;
            if right_idx < plane.cells.len() {
                plane.cells[right_idx].char = '│';
                plane.cells[right_idx].fg = self.theme.outline;
                plane.cells[right_idx].transparent = false;
            }
        }

        // Draw label
        let text_fg = if self.pressed {
            self.theme.fg_on_accent
        } else {
            self.theme.fg
        };
        for (i, c) in self.label.chars().enumerate() {
            let idx = ((area.height / 2) * area.width + start_x + 1 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = text_fg;
                plane.cells[idx].style = Styles::BOLD;
                plane.cells[idx].transparent = false;
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Enter && key.kind == KeyEventKind::Press {
            self.pressed = true;
            self.dirty = true;
            true
        } else if key.code == KeyCode::Enter && key.kind == KeyEventKind::Release {
            self.pressed = false;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if col >= self.area.width || row >= self.area.height {
            return false;
        }

        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.pressed = true;
                self.dirty = true;
                true
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.pressed = false;
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.dirty = true;
    }
}

/// A toggle/checkbox widget with accessibility support.
struct AccessibleToggle {
    id: WidgetId,
    area: Rect,
    label: String,
    checked: bool,
    theme: Theme,
    dirty: bool,
}

impl AccessibleToggle {
    fn new(id: WidgetId, label: &str, checked: bool, theme: Theme) -> Self {
        Self {
            id,
            area: Rect::default(),
            label: label.to_string(),
            checked,
            theme,
            dirty: true,
        }
    }
}

impl Accessibility for AccessibleToggle {
    fn role(&self) -> Role {
        Role::CheckBox
    }

    fn label(&self) -> Option<&str> {
        Some(&self.label)
    }

    fn description(&self) -> Option<&str> {
        Some(if self.checked { "checked" } else { "not checked" })
    }

    fn checked(&self) -> Option<bool> {
        Some(self.checked)
    }

    fn disabled(&self) -> bool {
        false
    }

    fn keyboard_shortcut(&self) -> Option<&str> {
        Some("Space")
    }
}

impl Widget for AccessibleToggle {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        let checkbox = if self.checked { "[x]" } else { "[ ]" };
        let text = format!("{} {}", checkbox, self.label);

        for (i, c) in text.chars().enumerate() {
            let idx = (2 * area.width + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if self.checked {
                    self.theme.success
                } else {
                    self.theme.fg
                };
                plane.cells[idx].transparent = false;
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Char(' ') && key.kind == KeyEventKind::Press {
            self.checked = !self.checked;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.dirty = true;
    }
}

/// Main accessibility demo app.
struct AccessibilityDemo {
    id: WidgetId,
    area: Rect,
    theme: Theme,
    show_help: bool,
    dirty: bool,
    should_quit: Arc<AtomicBool>,
    keybindings: KeybindingSet,
    submit_btn: AccessibleButton,
    cancel_btn: AccessibleButton,
    notifications_toggle: AccessibleToggle,
    sound_toggle: AccessibleToggle,
    toast_msg: RefCell<Option<String>>,
    toast_time: RefCell<Option<Instant>>,
}

impl AccessibilityDemo {
    fn new(theme: Theme, should_quit: Arc<AtomicBool>) -> Self {
        Self {
            id: WidgetId::new(1),
            area: Rect::default(),
            theme: theme.clone(),
            show_help: false,
            dirty: true,
            should_quit,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            submit_btn: AccessibleButton::new(WidgetId::new(2), "Submit", theme.clone()),
            cancel_btn: AccessibleButton::new(WidgetId::new(3), "Cancel", theme.clone()),
            notifications_toggle: AccessibleToggle::new(
                WidgetId::new(4),
                "Enable notifications",
                true,
                theme.clone(),
            ),
            sound_toggle: AccessibleToggle::new(
                WidgetId::new(5),
                "Enable sound effects",
                false,
                theme,
            ),
            toast_msg: RefCell::new(None),
            toast_time: RefCell::new(None),
        }
    }
}

impl Widget for AccessibilityDemo {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
        let btn_y = area.y + area.height / 2;
        self.submit_btn.set_area(Rect::new(area.x + 5, btn_y, 12, 3));
        self.cancel_btn.set_area(Rect::new(area.x + 20, btn_y, 10, 3));
        self.notifications_toggle.set_area(Rect::new(area.x + 2, area.y + 5, 30, 1));
        self.sound_toggle.set_area(Rect::new(area.x + 2, area.y + 7, 30, 1));
    }

    fn needs_render(&self) -> bool {
        let expired = self.toast_time.borrow().map_or(false, |t| t.elapsed() >= Duration::from_secs(2));
        if expired && self.toast_msg.borrow().is_some() {
            *self.toast_msg.borrow_mut() = None;
            *self.toast_time.borrow_mut() = None;
            return true;
        }
        self.dirty
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        // Title
        let title = "Accessibility Demo";
        let tx = (area.width - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (2 * area.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Subtitle
        let subtitle = "Press Tab to navigate, Enter/Space to activate";
        let sx = (area.width - subtitle.len() as u16) / 2;
        for (i, c) in subtitle.chars().enumerate() {
            let idx = (4 * area.width + sx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
            }
        }

        // Separator
        let sep_y = 5u16;
        for x in 2..area.width - 2 {
            let idx = (sep_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = self.theme.outline;
            }
        }

        // Render sub-widgets
        let btn_plane = self.submit_btn.render(self.submit_btn.area);
        for (i, cell) in btn_plane.cells.iter().enumerate() {
            let local_y = i / btn_plane.width as usize;
            let local_x = i % btn_plane.width as usize;
            let abs_x = self.submit_btn.area.x + local_x as u16;
            let abs_y = self.submit_btn.area.y + local_y as u16;
            let dest_idx = (abs_y * area.width + abs_x) as usize;
            if dest_idx < plane.cells.len() && !cell.transparent {
                plane.cells[dest_idx] = *cell;
            }
        }

        let btn_plane = self.cancel_btn.render(self.cancel_btn.area);
        for (i, cell) in btn_plane.cells.iter().enumerate() {
            let local_y = i / btn_plane.width as usize;
            let local_x = i % btn_plane.width as usize;
            let abs_x = self.cancel_btn.area.x + local_x as u16;
            let abs_y = self.cancel_btn.area.y + local_y as u16;
            let dest_idx = (abs_y * area.width + abs_x) as usize;
            if dest_idx < plane.cells.len() && !cell.transparent {
                plane.cells[dest_idx] = *cell;
            }
        }

        let toggle_plane = self.notifications_toggle.render(self.notifications_toggle.area);
        for (i, cell) in toggle_plane.cells.iter().enumerate() {
            let local_y = i / toggle_plane.width as usize;
            let local_x = i % toggle_plane.width as usize;
            let abs_x = self.notifications_toggle.area.x + local_x as u16;
            let abs_y = self.notifications_toggle.area.y + local_y as u16;
            let dest_idx = (abs_y * area.width + abs_x) as usize;
            if dest_idx < plane.cells.len() && !cell.transparent {
                plane.cells[dest_idx] = *cell;
            }
        }

        let toggle_plane = self.sound_toggle.render(self.sound_toggle.area);
        for (i, cell) in toggle_plane.cells.iter().enumerate() {
            let local_y = i / toggle_plane.width as usize;
            let local_x = i % toggle_plane.width as usize;
            let abs_x = self.sound_toggle.area.x + local_x as u16;
            let abs_y = self.sound_toggle.area.y + local_y as u16;
            let dest_idx = (abs_y * area.width + abs_x) as usize;
            if dest_idx < plane.cells.len() && !cell.transparent {
                plane.cells[dest_idx] = *cell;
            }
        }

        // Status bar
        let kb_quit = self.keybindings.display(actions::QUIT).unwrap_or("Ctrl+Q");
        let kb_help = self.keybindings.display(actions::HELP).unwrap_or("F1");
        let kb_back = self.keybindings.display(actions::BACK).unwrap_or("Esc");
        let status = format!("Tab: navigate | Enter/Space: activate | {kb_help}: help | {kb_back}: dismiss | {kb_quit}: quit");
        for (i, c) in status.chars().enumerate() {
            let idx = ((area.height - 1) * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
                plane.cells[idx].bg = self.theme.surface;
                plane.cells[idx].transparent = false;
            }
        }

        if let (Some(msg), Some(time)) = (self.toast_msg.borrow().as_ref(), self.toast_time.borrow().as_ref()) {
            if time.elapsed() < Duration::from_secs(2) {
                let msg_len = msg.len() as u16;
                let pad = 2u16;
                let bar_w = msg_len + pad * 2;
                let bar_x = (area.width - bar_w) / 2;
                let bar_y = area.height.saturating_sub(4);
                let bg = if msg == "Submitted!" {
                    self.theme.success
                } else {
                    self.theme.error
                };
                for x in bar_x..bar_x + bar_w {
                    let idx = (bar_y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = bg;
                        plane.cells[idx].fg = self.theme.fg_on_accent;
                        plane.cells[idx].transparent = false;
                    }
                }
                for (i, c) in msg.chars().enumerate() {
                    let idx = (bar_y * area.width + bar_x + pad + i as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = self.theme.fg_on_accent;
                        plane.cells[idx].style = Styles::BOLD;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }

        if self.show_help {
            let t = &self.theme;
            let hw = 40u16.min(area.width.saturating_sub(4));
            let hh = 10u16.min(area.height.saturating_sub(4));
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
            let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = *ch; plane.cells[idx].fg = t.outline; }
            }
            for x in hx + 1..hx + hw - 1 {
                let top = (hy * area.width + x) as usize;
                let bot = ((hy + hh - 1) * area.width + x) as usize;
                if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
                if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
            }
            for y in hy + 1..hy + hh - 1 {
                let left = (y * area.width + hx) as usize;
                let right = (y * area.width + hx + hw - 1) as usize;
                if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
                if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
            }
            let title = "Accessibility Help";
            let tx = hx + (hw - title.len() as u16) / 2;
            for (i, c) in title.chars().enumerate() {
                let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; plane.cells[idx].style = Styles::BOLD; }
            }
            let kb_theme = self.keybindings.display(actions::THEME).unwrap_or("Ctrl+T");
            let shortcuts = [
                ("Tab", "Navigate controls"),
                ("Enter/Spc", "Activate button"),
                (kb_theme, "Cycle theme"),
                (kb_help, "Toggle help"),
                (kb_back, "Dismiss help"),
                (kb_quit, "Quit app"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * area.width + hx + 2 + j as u16) as usize;
                    if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * area.width + hx + 14 + j as u16) as usize;
                    if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.fg; }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.keybindings.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            let themes = Theme::all();
            let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
            self.theme = themes[(idx + 1) % themes.len()].clone();
            self.submit_btn.on_theme_change(&self.theme);
            self.cancel_btn.on_theme_change(&self.theme);
            self.notifications_toggle.on_theme_change(&self.theme);
            self.sound_toggle.on_theme_change(&self.theme);
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            true
        } else if self.keybindings.matches(actions::BACK, &key) && self.show_help {
            self.show_help = false;
            self.dirty = true;
            true
        } else {
            let was_submit = self.submit_btn.pressed;
            let was_cancel = self.cancel_btn.pressed;
            let handled = self.submit_btn.handle_key(key)
                || self.cancel_btn.handle_key(key)
                || self.notifications_toggle.handle_key(key)
                || self.sound_toggle.handle_key(key);
            if was_submit && !self.submit_btn.pressed {
                *self.toast_msg.borrow_mut() = Some("Submitted!".to_string());
                *self.toast_time.borrow_mut() = Some(Instant::now());
            }
            if was_cancel && !self.cancel_btn.pressed {
                *self.toast_msg.borrow_mut() = Some("Cancelled".to_string());
                *self.toast_time.borrow_mut() = Some(Instant::now());
            }
            if handled {
                self.dirty = true;
            }
            handled
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let was_submit = self.submit_btn.pressed;
        if self.submit_btn.handle_mouse(kind, col, row) {
            if was_submit && !self.submit_btn.pressed {
                *self.toast_msg.borrow_mut() = Some("Submitted!".to_string());
                *self.toast_time.borrow_mut() = Some(Instant::now());
            }
            self.dirty = true;
            return true;
        }
        let was_cancel = self.cancel_btn.pressed;
        if self.cancel_btn.handle_mouse(kind, col, row) {
            if was_cancel && !self.cancel_btn.pressed {
                *self.toast_msg.borrow_mut() = Some("Cancelled".to_string());
                *self.toast_time.borrow_mut() = Some(Instant::now());
            }
            self.dirty = true;
            return true;
        }
        if self.notifications_toggle.handle_mouse(kind, col, row) {
            self.dirty = true;
            return true;
        }
        if self.sound_toggle.handle_mouse(kind, col, row) {
            self.dirty = true;
            return true;
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.submit_btn.on_theme_change(theme);
        self.cancel_btn.on_theme_change(theme);
        self.notifications_toggle.on_theme_change(theme);
        self.sound_toggle.on_theme_change(theme);
        self.dirty = true;
    }

    fn current_theme(&self) -> Option<Theme> {
        Some(self.theme.clone())
    }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::from_env_or(Theme::nord());
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);
    let demo = AccessibilityDemo::new(theme.clone(), should_quit);

    let mut app = App::new()?
        .title("Accessibility Demo")
        .fps(30)
        .theme(theme);
    app.add_widget(Box::new(demo), Rect::new(0, 0, 80, 24));
    app.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
        }
    })
    .run(|_| {})
}
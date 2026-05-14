//! Accessibility demo — demonstrates screen reader announcements via OSC 99.
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
use dracon_terminal_engine::visuals::accessibility::{Accessibility, Role};

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

        let checkbox = if self.checked { "☑" } else { "☐" };
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
    // Sub-widgets
    submit_btn: AccessibleButton,
    cancel_btn: AccessibleButton,
    notifications_toggle: AccessibleToggle,
    sound_toggle: AccessibleToggle,
}

impl AccessibilityDemo {
    fn new(theme: Theme) -> Self {
        Self {
            id: WidgetId::new(1),
            area: Rect::default(),
            theme: theme.clone(),
            show_help: false,
            dirty: true,
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
        // Position sub-widgets
        let btn_y = area.y + area.height / 2;
        self.submit_btn.set_area(Rect::new(area.x + 5, btn_y, 12, 3));
        self.cancel_btn.set_area(Rect::new(area.x + 20, btn_y, 10, 3));
        self.notifications_toggle.set_area(Rect::new(area.x + 2, area.y + 5, 30, 1));
        self.sound_toggle.set_area(Rect::new(area.x + 2, area.y + 7, 30, 1));
    }

    fn needs_render(&self) -> bool {
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
        let status = "Tab: navigate | Enter/Space: activate | ?: help | Ctrl+Q: quit";
        for (i, c) in status.chars().enumerate() {
            let idx = ((area.height - 1) * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
                plane.cells[idx].bg = self.theme.surface;
                plane.cells[idx].transparent = false;
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Char('?') && key.modifiers.is_empty() {
            self.show_help = !self.show_help;
            self.dirty = true;
            true
        } else if key.code == KeyCode::Esc && self.show_help {
            self.show_help = false;
            self.dirty = true;
            true
        } else {
            // Delegate to sub-widgets
            let handled = self.submit_btn.handle_key(key)
                || self.cancel_btn.handle_key(key)
                || self.notifications_toggle.handle_key(key)
                || self.sound_toggle.handle_key(key);
            if handled {
                self.dirty = true;
            }
            handled
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.submit_btn.handle_mouse(kind, col, row) {
            self.dirty = true;
            return true;
        }
        if self.cancel_btn.handle_mouse(kind, col, row) {
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
}

fn main() -> std::io::Result<()> {
    let theme = Theme::from_env_or(Theme::nord());
    let demo = AccessibilityDemo::new(theme.clone());

    let mut app = App::new()?
        .title("Accessibility Demo")
        .fps(30)
        .theme(theme);
    app.add_widget(Box::new(demo), Rect::new(0, 0, 80, 24));
    app.run(|_| {})
}
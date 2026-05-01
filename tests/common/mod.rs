//! Shared test utilities and helpers used across integration tests.

#![allow(dead_code)]

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;
use std::cell::Cell;
use std::rc::Rc;

/// Creates a press KeyEvent for the given KeyCode.
pub fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        kind: KeyEventKind::Press,
        code,
        modifiers: KeyModifiers::empty(),
    }
}

/// Creates a repeat KeyEvent for the given KeyCode.
pub fn make_key_repeat(code: KeyCode) -> KeyEvent {
    KeyEvent {
        kind: KeyEventKind::Repeat,
        code,
        modifiers: KeyModifiers::empty(),
    }
}

/// Creates a key event with modifiers.
pub fn make_key_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        kind: KeyEventKind::Press,
        code,
        modifiers,
    }
}

/// Returns a standard 80x20 test area.
pub fn dummy_area() -> Rect {
    Rect::new(0, 0, 80, 20)
}

/// Returns a test area with custom dimensions.
pub fn make_area(w: u16, h: u16) -> Rect {
    Rect::new(0, 0, w, h)
}

/// Creates a Rect with the given coordinates and dimensions.
pub fn rect(x: u16, y: u16, w: u16, h: u16) -> Rect {
    Rect::new(x, y, w, h)
}

/// Asserts that a theme color field matches the expected RGB values.
pub fn assert_rgb(t: &Theme, field: &str, r: u8, g: u8, b: u8) {
    let expected = Color::Rgb(r, g, b);
    let actual = match field {
        "bg" => t.bg,
        "fg" => t.fg,
        "accent" => t.accent,
        "selection_bg" => t.selection_bg,
        "selection_fg" => t.selection_fg,
        "border" => t.border,
        "scrollbar_track" => t.scrollbar_track,
        "scrollbar_thumb" => t.scrollbar_thumb,
        "hover_bg" => t.hover_bg,
        "active_bg" => t.active_bg,
        "inactive_fg" => t.inactive_fg,
        "input_bg" => t.input_bg,
        "input_fg" => t.input_fg,
        "error_fg" => t.error_fg,
        "success_fg" => t.success_fg,
        "warning_fg" => t.warning_fg,
        "disabled_fg" => t.disabled_fg,
        _ => panic!("unknown field: {}", field),
    };
    assert_eq!(actual, expected, "Theme.{} mismatch", field);
}

/// A mock widget that tracks on_theme_change calls and exposes the current theme name.
#[derive(Default)]
pub struct TrackingWidget {
    pub id: dracon_terminal_engine::framework::widget::WidgetId,
    pub theme_changes: Rc<Cell<usize>>,
    pub current_theme: Rc<Cell<Option<&'static str>>>,
    pub focus_count: Rc<Cell<usize>>,
    pub blur_count: Rc<Cell<usize>>,
    pub area: std::cell::Cell<Rect>,
}

impl TrackingWidget {
    pub fn new(id: usize) -> Self {
        Self {
            id: dracon_terminal_engine::framework::widget::WidgetId::new(id),
            theme_changes: Rc::new(Cell::new(0)),
            current_theme: Rc::new(Cell::new(None)),
            focus_count: Rc::new(Cell::new(0)),
            blur_count: Rc::new(Cell::new(0)),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    pub fn theme_change_count(&self) -> usize {
        self.theme_changes.get()
    }

    pub fn focus_count(&self) -> usize {
        self.focus_count.get()
    }

    pub fn blur_count(&self) -> usize {
        self.blur_count.get()
    }
}

impl Widget for TrackingWidget {
    fn id(&self) -> dracon_terminal_engine::framework::widget::WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn render(&self, _area: Rect) -> dracon_terminal_engine::compositor::Plane {
        dracon_terminal_engine::compositor::Plane::new(0, 80, 24)
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme_changes.set(self.theme_changes.get() + 1);
        self.current_theme.set(Some(theme.name));
    }

    fn on_focus(&mut self) {
        self.focus_count.set(self.focus_count.get() + 1);
    }

    fn on_blur(&mut self) {
        self.blur_count.set(self.blur_count.get() + 1);
    }
}

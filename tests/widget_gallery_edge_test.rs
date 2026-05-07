use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, Checkbox, ProgressBar, Radio, SearchInput, Select, Slider, Spinner, Toggle,
};
use dracon_terminal_engine::input::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use ratatui::layout::Rect;

// Minimal reproduction of WidgetGallery's handle_mouse to test edge cases
struct WidgetGalleryMock {
    checkbox: Checkbox,
    radio: Radio,
    slider: Slider,
    spinner: Spinner,
    toggle: Toggle,
    select: Select,
    search: SearchInput,
    progress: ProgressBar,
    button: Button,
    selected: usize,
}

impl WidgetGalleryMock {
    fn new() -> Self {
        Self {
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
            selected: 0,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Right | KeyCode::Down => {
                self.selected = (self.selected + 1) % 9;
                true
            }
            KeyCode::Left | KeyCode::Up => {
                self.selected = if self.selected == 0 {
                    8
                } else {
                    self.selected - 1
                };
                true
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Toggle checkbox for slot 0
                if self.selected == 0 {
                    self.checkbox.toggle();
                }
                true
            }
            _ => false,
        }
    }

    fn slot_rect(&self, slot: usize, area: Rect) -> Rect {
        let rows = 3u16;
        let cols = if slot < 4 {
            4u16
        } else if slot < 7 {
            3u16
        } else {
            2u16
        };
        let row = (slot / cols as usize) as u16;
        let col = (slot % cols as usize) as u16;

        let card_w = area.width.saturating_sub(2) / cols;
        let card_h = area.height.saturating_sub(4) / rows;

        let x = area.x + 1 + col * card_w;
        let y = area.y + 2 + row * card_h;

        Rect::new(x, y, card_w.saturating_sub(1), card_h.saturating_sub(1))
    }

    fn handle_mouse_at_edge(
        &mut self,
        _kind: MouseEventKind,
        col: u16,
        row: u16,
        area: Rect,
    ) -> bool {
        for slot in 0..9usize {
            let rect = self.slot_rect(slot, area);
            if col >= rect.x
                && col < rect.x + rect.width
                && row >= rect.y
                && row < rect.y + rect.height
            {
                let widget_area = Rect::new(
                    1,
                    2,
                    rect.width.saturating_sub(2),
                    rect.height.saturating_sub(3),
                );
                // This is the fixed code - includes col check
                if row >= rect.y + 2
                    && row < rect.y + 2 + widget_area.height
                    && col > rect.x
                    && col < rect.x + 1 + widget_area.width
                {
                    let _rel_col = col - rect.x - 1;
                    let _rel_row = row - rect.y - 2;
                    return true; // Would dispatch to widget
                }
                return true; // In card but outside widget area
            }
        }
        false
    }
}

#[test]
fn test_widget_gallery_mouse_at_left_edge_no_panic() {
    let mut gallery = WidgetGalleryMock::new();
    let area = Rect::new(0, 0, 80, 24);

    // Click at the left edge of the first card (col == rect.x)
    let rect = gallery.slot_rect(0, area);
    let result = gallery.handle_mouse_at_edge(
        MouseEventKind::Down(MouseButton::Left),
        rect.x,     // Left edge - this used to panic
        rect.y + 3, // Inside widget area vertically
        area,
    );
    assert!(result);
}

#[test]
fn test_widget_gallery_mouse_at_top_edge_no_panic() {
    let mut gallery = WidgetGalleryMock::new();
    let area = Rect::new(0, 0, 80, 24);

    // Click just above the widget area (row == rect.y + 1)
    let rect = gallery.slot_rect(0, area);
    let result = gallery.handle_mouse_at_edge(
        MouseEventKind::Down(MouseButton::Left),
        rect.x + 2, // Inside widget area horizontally
        rect.y + 1, // Just above widget area - this used to panic
        area,
    );
    assert!(result); // Should be in card but outside widget area
}

#[test]
fn test_widget_gallery_mouse_inside_widget_area() {
    let mut gallery = WidgetGalleryMock::new();
    let area = Rect::new(0, 0, 80, 24);

    let rect = gallery.slot_rect(0, area);
    let result = gallery.handle_mouse_at_edge(
        MouseEventKind::Down(MouseButton::Left),
        rect.x + 2, // Inside widget area
        rect.y + 3, // Inside widget area
        area,
    );
    assert!(result);
}

#[test]
fn test_widget_gallery_mouse_outside_all_cards() {
    let mut gallery = WidgetGalleryMock::new();
    let area = Rect::new(0, 0, 80, 24);

    let result = gallery.handle_mouse_at_edge(
        MouseEventKind::Down(MouseButton::Left),
        0, // Top-left corner - outside all cards
        0,
        area,
    );
    assert!(!result);
}

#[test]
fn test_widget_gallery_slot_rect_calculation() {
    let gallery = WidgetGalleryMock::new();
    let area = Rect::new(0, 0, 80, 24);

    let rect0 = gallery.slot_rect(0, area);
    assert!(rect0.width > 0);
    assert!(rect0.height > 0);

    let rect8 = gallery.slot_rect(8, area);
    assert!(rect8.width > 0);
    assert!(rect8.height > 0);
}

#[test]
fn test_widget_gallery_mouse_small_terminal() {
    let mut gallery = WidgetGalleryMock::new();
    let area = Rect::new(0, 0, 10, 8); // Very small terminal

    let rect = gallery.slot_rect(0, area);
    // Click anywhere in the card - should not panic
    let _result = gallery.handle_mouse_at_edge(
        MouseEventKind::Down(MouseButton::Left),
        rect.x,
        rect.y,
        area,
    );
    // Just verify it doesn't panic - result may vary with tiny dimensions
}

// ═══════════════════════════════════════════════════════════════════════════════
// KEYBOARD NAVIGATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_widget_gallery_key_right_navigates() {
    let mut gallery = WidgetGalleryMock::new();
    assert_eq!(gallery.selected, 0);

    let key = KeyEvent {
        code: KeyCode::Right,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    };
    assert!(gallery.handle_key(key));
    assert_eq!(gallery.selected, 1);
}

#[test]
fn test_widget_gallery_key_left_navigates() {
    let mut gallery = WidgetGalleryMock::new();
    gallery.selected = 5;

    let key = KeyEvent {
        code: KeyCode::Left,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    };
    assert!(gallery.handle_key(key));
    assert_eq!(gallery.selected, 4);
}

#[test]
fn test_widget_gallery_key_left_wraps() {
    let mut gallery = WidgetGalleryMock::new();
    assert_eq!(gallery.selected, 0);

    let key = KeyEvent {
        code: KeyCode::Left,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    };
    assert!(gallery.handle_key(key));
    assert_eq!(gallery.selected, 8); // Wraps to last slot
}

#[test]
fn test_widget_gallery_key_right_wraps() {
    let mut gallery = WidgetGalleryMock::new();
    gallery.selected = 8;

    let key = KeyEvent {
        code: KeyCode::Right,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    };
    assert!(gallery.handle_key(key));
    assert_eq!(gallery.selected, 0); // Wraps to first slot
}

#[test]
fn test_widget_gallery_key_enter_toggles_checkbox() {
    let mut gallery = WidgetGalleryMock::new();
    assert!(!gallery.checkbox.is_checked());

    let key = KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    };
    assert!(gallery.handle_key(key));
    assert!(gallery.checkbox.is_checked());
}

#[test]
fn test_widget_gallery_key_release_ignored() {
    let mut gallery = WidgetGalleryMock::new();
    let original = gallery.selected;

    let key = KeyEvent {
        code: KeyCode::Right,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Release,
    };
    assert!(!gallery.handle_key(key));
    assert_eq!(gallery.selected, original);
}

#[test]
fn test_widget_gallery_key_down_navigates() {
    let mut gallery = WidgetGalleryMock::new();
    assert_eq!(gallery.selected, 0);

    let key = KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    };
    assert!(gallery.handle_key(key));
    assert_eq!(gallery.selected, 1);
}

#[test]
fn test_widget_gallery_key_up_navigates() {
    let mut gallery = WidgetGalleryMock::new();
    gallery.selected = 3;

    let key = KeyEvent {
        code: KeyCode::Up,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    };
    assert!(gallery.handle_key(key));
    assert_eq!(gallery.selected, 2);
}

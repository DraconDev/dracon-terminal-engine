//! Tests for theme propagation through the widget system.

use std::cell::Cell;
use std::rc::Rc;

use dracon_terminal_engine::compositor::{Color, Plane};
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::split::Orientation;
use dracon_terminal_engine::framework::widgets::SplitPane;

/// A mock widget that tracks on_theme_change calls.
#[derive(Default)]
struct MockWidget {
    id: WidgetId,
    theme_changes: Rc<Cell<usize>>,
    current_theme: Rc<Cell<Option<&'static str>>>,
    area: std::cell::Cell<ratatui::layout::Rect>,
}

impl MockWidget {
    fn new(id: usize) -> Self {
        Self {
            id: WidgetId::new(id),
            theme_changes: Rc::new(Cell::new(0)),
            current_theme: Rc::new(Cell::new(None)),
            area: std::cell::Cell::new(ratatui::layout::Rect::new(0, 0, 80, 24)),
        }
    }

    fn theme_change_count(&self) -> usize {
        self.theme_changes.get()
    }
}

impl Widget for MockWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> ratatui::layout::Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: ratatui::layout::Rect) {
        self.area.set(area);
    }

    fn render(&self, _area: ratatui::layout::Rect) -> Plane {
        Plane::new(0, 80, 24)
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme_changes.set(self.theme_changes.get() + 1);
        self.current_theme.set(Some(theme.name));
    }
}

// === SplitPane on_theme_change ===

#[test]
fn test_splitpane_on_theme_change_updates_divider_color() {
    let mut split = SplitPane::new(Orientation::Horizontal);

    let original_color = split.divider_color;
    assert!(matches!(original_color, Color::Rgb(_, _, _)));

    split.on_theme_change(&Theme::cyberpunk());

    assert_eq!(
        split.divider_color,
        Theme::cyberpunk().inactive_fg,
        "divider_color should update to theme.inactive_fg"
    );
}

#[test]
fn test_splitpane_on_theme_change_dracula() {
    let mut split = SplitPane::new(Orientation::Vertical);

    split.on_theme_change(&Theme::dracula());

    assert_eq!(
        split.divider_color,
        Theme::dracula().inactive_fg,
        "divider_color should update for dracula theme"
    );
}

#[test]
fn test_splitpane_on_theme_change_light() {
    let mut split = SplitPane::new(Orientation::Horizontal);

    split.on_theme_change(&Theme::light());

    assert_eq!(
        split.divider_color,
        Theme::light().inactive_fg,
        "divider_color should update for light theme"
    );
}

#[test]
fn test_splitpane_theme_change_idempotent() {
    let mut split = SplitPane::new(Orientation::Horizontal);

    split.on_theme_change(&Theme::dark());
    let first = split.divider_color;
    split.on_theme_change(&Theme::dark());
    let second = split.divider_color;

    assert_eq!(first, second, "calling on_theme_change twice with same theme should be idempotent");
}

// === Mock Widget on_theme_change tracking ===

#[test]
fn test_mock_widget_tracks_theme_changes() {
    let widget = MockWidget::new(1);
    assert_eq!(widget.theme_change_count(), 0);

    let mut w = MockWidget::new(1);
    w.on_theme_change(&Theme::dark());
    assert_eq!(w.theme_change_count(), 1);

    w.on_theme_change(&Theme::light());
    assert_eq!(w.theme_change_count(), 2);
}

#[test]
fn test_mock_widget_records_theme_name() {
    let mut w = MockWidget::new(1);
    assert!(w.current_theme.get().is_none());

    w.on_theme_change(&Theme::dracula());
    assert_eq!(w.current_theme.get(), Some("dracula"));

    w.on_theme_change(&Theme::nord());
    assert_eq!(w.current_theme.get(), Some("nord"));
}

// === Default Widget trait on_theme_change ===

struct NoopWidget;

impl Widget for NoopWidget {
    fn id(&self) -> WidgetId {
        WidgetId::new(99)
    }
    fn area(&self) -> ratatui::layout::Rect {
        ratatui::layout::Rect::new(0, 0, 80, 24)
    }
    fn set_area(&mut self, _area: ratatui::layout::Rect) {}
    fn render(&self, _area: ratatui::layout::Rect) -> Plane {
        Plane::new(0, 80, 24)
    }
}

#[test]
fn test_default_widget_on_theme_change_is_noop() {
    let mut w = NoopWidget;
    w.on_theme_change(&Theme::cyberpunk());
}

// === Theme switching correctness ===

#[test]
fn test_all_themes_produce_different_divider_colors() {
    let themes = [
        Theme::dark(),
        Theme::light(),
        Theme::cyberpunk(),
        Theme::dracula(),
        Theme::nord(),
        Theme::catppuccin_mocha(),
        Theme::gruvbox_dark(),
        Theme::tokyo_night(),
        Theme::solarized_dark(),
        Theme::solarized_light(),
        Theme::one_dark(),
        Theme::rose_pine(),
        Theme::kanagawa(),
        Theme::everforest(),
        Theme::monokai(),
    ];

    let colors: Vec<_> = themes
        .iter()
        .map(|t| t.inactive_fg)
        .collect();

    for (i, c1) in colors.iter().enumerate() {
        for (j, c2) in colors.iter().enumerate() {
            if i != j {
                assert_ne!(
                    c1, c2,
                    "themes at index {} and {} have same inactive_fg: {:?}",
                    i, j, c1
                );
            }
        }
    }
}

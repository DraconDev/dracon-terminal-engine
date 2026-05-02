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
        Theme::cyberpunk().outline,
        "divider_color should update to theme.outline"
    );
}

#[test]
fn test_splitpane_on_theme_change_dracula() {
    let mut split = SplitPane::new(Orientation::Vertical);

    split.on_theme_change(&Theme::dracula());

    assert_eq!(
        split.divider_color,
        Theme::dracula().outline,
        "divider_color should update for dracula theme"
    );
}

#[test]
fn test_splitpane_on_theme_change_light() {
    let mut split = SplitPane::new(Orientation::Horizontal);

    split.on_theme_change(&Theme::light());

    assert_eq!(
        split.divider_color,
        Theme::light().outline,
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

    assert_eq!(
        first, second,
        "calling on_theme_change twice with same theme should be idempotent"
    );
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

// === App::set_theme integration ===

use dracon_terminal_engine::framework::app::App;
use ratatui::layout::Rect;

struct TrackingWidget {
    id: WidgetId,
    theme_call_count: Rc<Cell<usize>>,
    dirty_flag: Rc<Cell<bool>>,
    area: std::cell::Cell<Rect>,
}

impl TrackingWidget {
    fn new(id_val: usize) -> (Self, Rc<Cell<usize>>) {
        let theme_call_count = Rc::new(Cell::new(0));
        let dirty_flag = Rc::new(Cell::new(true));
        let tw = Self {
            id: WidgetId::new(id_val),
            theme_call_count: theme_call_count.clone(),
            dirty_flag: dirty_flag.clone(),
            area: std::cell::Cell::new(Rect::new(0, 0, 10, 10)),
        };
        (tw, theme_call_count)
    }
}

impl Widget for TrackingWidget {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn area(&self) -> Rect {
        self.area.get()
    }
    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn needs_render(&self) -> bool {
        self.dirty_flag.get()
    }
    fn mark_dirty(&mut self) {
        self.dirty_flag.set(true);
    }
    fn clear_dirty(&mut self) {
        self.dirty_flag.set(false);
    }
    fn render(&self, area: Rect) -> Plane {
        Plane::new(0, area.width, area.height)
    }
    fn on_theme_change(&mut self, _theme: &Theme) {
        self.theme_call_count.set(self.theme_call_count.get() + 1);
    }
}

#[test]
fn test_app_set_theme_calls_on_theme_change_on_all_widgets() {
    let mut app = App::new().unwrap();

    let (tw1, count1) = TrackingWidget::new(1);
    let (tw2, count2) = TrackingWidget::new(2);
    let (tw3, count3) = TrackingWidget::new(3);

    app.add_widget(Box::new(tw1), Rect::new(0, 0, 10, 10));
    app.add_widget(Box::new(tw2), Rect::new(10, 0, 10, 10));
    app.add_widget(Box::new(tw3), Rect::new(0, 10, 20, 10));

    app.set_theme(Theme::nord());

    assert_eq!(
        count1.get(),
        1,
        "widget 1 should have received 1 theme change"
    );
    assert_eq!(
        count2.get(),
        1,
        "widget 2 should have received 1 theme change"
    );
    assert_eq!(
        count3.get(),
        1,
        "widget 3 should have received 1 theme change"
    );
}

#[test]
fn test_app_set_theme_multiple_times_accumulates() {
    let mut app = App::new().unwrap();
    let (tw, count) = TrackingWidget::new(1);
    app.add_widget(Box::new(tw), Rect::new(0, 0, 10, 10));

    app.set_theme(Theme::dark());
    app.set_theme(Theme::light());
    app.set_theme(Theme::cyberpunk());

    assert_eq!(
        count.get(),
        3,
        "widget should have received 3 theme change calls"
    );
}

#[test]
fn test_app_widget_persists_after_theme_change() {
    let mut app = App::new().unwrap();
    app.add_widget(Box::new(TrackingWidget::new(1).0), Rect::new(0, 0, 10, 10));

    assert_eq!(app.widget_count(), 1, "one widget should be added");
    app.set_theme(Theme::cyberpunk());
    assert_eq!(
        app.widget_count(),
        1,
        "widget count should remain 1 after theme change"
    );
}

#[test]
fn test_app_remove_widget_after_theme_change() {
    let mut app = App::new().unwrap();
    let id1 = app.add_widget(Box::new(TrackingWidget::new(1).0), Rect::new(0, 0, 10, 10));
    app.add_widget(Box::new(TrackingWidget::new(2).0), Rect::new(10, 0, 10, 10));

    assert_eq!(app.widget_count(), 2, "two widgets should be added");
    app.set_theme(Theme::nord());
    app.remove_widget(id1);

    assert_eq!(
        app.widget_count(),
        1,
        "one widget should remain after removal"
    );
    assert!(
        app.widget(id1).is_none(),
        "removed widget should not be found"
    );
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

    let colors: Vec<_> = themes.iter().map(|t| t.fg_muted).collect();

    for (i, c1) in colors.iter().enumerate() {
        for (j, c2) in colors.iter().enumerate() {
            if i != j {
                assert_ne!(
                    c1, c2,
                    "themes at index {} and {} have same fg_muted: {:?}",
                    i, j, c1
                );
            }
        }
    }
}

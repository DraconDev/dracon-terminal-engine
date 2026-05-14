//! Tests for widget components.
//!
//! Note: Most widget tests live in widget_test.rs.
//! This file contains additional tests for framework widgets.

mod common;

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::split::Orientation;
use dracon_terminal_engine::framework::widgets::table::Column;
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, Button, Checkbox, CommandItem, CommandPalette, ConfirmDialog, ContextAction,
    ContextMenu, Form, Gauge, Hud, Label, List, LogViewer, MenuBar, MenuEntry, MenuItem, Modal,
    ProgressBar, Radio, Select, Slider, Spinner, SplitPane, StatusBadge, StatusBar, StatusSegment,
    TabBar, Table, Toast, ToastKind, Toggle, Tooltip, Tree, TreeNode,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_button_new() {
    let btn = Button::new("Click me");
    let area = Rect::new(0, 0, 10, 1);
    let _plane = btn.render(area);
}

#[test]
fn test_button_with_id() {
    let id = WidgetId::new(99);
    let btn = Button::with_id(id, "Label");
    assert_eq!(btn.id(), id);
}

#[test]
fn test_button_with_theme() {
    let btn = Button::new("Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 10, 1);
    let _plane = btn.render(area);
}

#[test]
fn test_button_render() {
    let btn = Button::new("Hi");
    let area = Rect::new(0, 0, 20, 1);
    let plane = btn.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_button_render_brackets() {
    let btn = Button::new("OK");
    let area = Rect::new(0, 0, 10, 1);
    let plane = btn.render(area);
    assert_eq!(plane.cells[0].char, '[');
    let label_end = 1 + 2;
    assert_eq!(plane.cells[label_end].char, ']');
}

#[test]
fn test_button_default_area() {
    let btn = Button::new("test");
    let area = btn.area();
    assert_eq!(area.width, 10);
    assert_eq!(area.height, 1);
}

#[test]
fn test_button_clear_dirty() {
    let mut btn = Button::new("test");
    assert!(btn.needs_render());
    btn.clear_dirty();
    assert!(!btn.needs_render());
}

#[test]
fn test_button_mark_dirty() {
    let mut btn = Button::new("test");
    btn.clear_dirty();
    btn.mark_dirty();
    assert!(btn.needs_render());
}

#[test]
fn test_button_set_area() {
    let mut btn = Button::new("test");
    assert!(btn.needs_render());
    btn.set_area(Rect::new(0, 0, 5, 1));
    assert!(btn.needs_render());
}

#[test]
fn test_button_z_index() {
    let btn = Button::new("test");
    assert_eq!(btn.z_index(), 0);
}

#[test]
fn test_button_cursor_position_returns_none() {
    let btn = Button::new("test");
    assert!(btn.cursor_position().is_none());
}

#[test]
fn test_button_handle_key_enter_triggers_callback() {
    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();
    {
        let mut btn = Button::new("OK").on_click(move || {
            called_clone.set(true);
        });
        btn.handle_key(KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Enter,
            modifiers: Default::default(),
        });
    }
    assert!(called.get());
}

#[test]
fn test_button_handle_key_non_enter_returns_false() {
    let mut btn = Button::new("OK").on_click(|| {});
    let result = btn.handle_key(KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Backspace,
        modifiers: Default::default(),
    });
    assert!(!result);
}

#[test]
fn test_toggle_new() {
    let toggle = Toggle::new(WidgetId::default_id(), "Enable");
    assert!(!toggle.is_on());
}

#[test]
fn test_toggle_with_theme() {
    let toggle = Toggle::new(WidgetId::default_id(), "Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 20, 1);
    let _plane = toggle.render(area);
}

#[test]
fn test_toggle_render() {
    let toggle = Toggle::new(WidgetId::default_id(), "Test");
    let area = Rect::new(0, 0, 20, 1);
    let plane = toggle.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_toggle_toggle() {
    let mut toggle = Toggle::new(WidgetId::default_id(), "Test");
    assert!(!toggle.is_on());
    toggle.toggle();
    assert!(toggle.is_on());
    toggle.toggle();
    assert!(!toggle.is_on());
}

#[test]
fn test_toggle_clear_dirty() {
    let mut toggle = Toggle::new(WidgetId::default_id(), "Test");
    assert!(toggle.needs_render());
    toggle.clear_dirty();
    assert!(!toggle.needs_render());
}

#[test]
fn test_checkbox_new() {
    let cb = Checkbox::new(WidgetId::default_id(), "Agree");
    assert!(!cb.is_checked());
}

#[test]
fn test_checkbox_with_theme() {
    let cb = Checkbox::new(WidgetId::default_id(), "Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 20, 1);
    let _plane = cb.render(area);
}

#[test]
fn test_checkbox_render() {
    let cb = Checkbox::new(WidgetId::default_id(), "Test");
    let area = Rect::new(0, 0, 20, 1);
    let plane = cb.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_checkbox_toggle() {
    let mut cb = Checkbox::new(WidgetId::default_id(), "Test");
    assert!(!cb.is_checked());
    cb.toggle();
    assert!(cb.is_checked());
    cb.toggle();
    assert!(!cb.is_checked());
}

#[test]
fn test_checkbox_clear_dirty() {
    let mut cb = Checkbox::new(WidgetId::default_id(), "Test");
    assert!(cb.needs_render());
    cb.clear_dirty();
    assert!(!cb.needs_render());
}

#[test]
fn test_radio_new() {
    let radio = Radio::new(WidgetId::default_id(), "Option");
    assert!(!radio.is_selected());
}

#[test]
fn test_radio_with_theme() {
    let radio = Radio::new(WidgetId::default_id(), "Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 20, 1);
    let _plane = radio.render(area);
}

#[test]
fn test_radio_render() {
    let radio = Radio::new(WidgetId::default_id(), "Test");
    let area = Rect::new(0, 0, 20, 1);
    let plane = radio.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_radio_select() {
    let mut radio = Radio::new(WidgetId::default_id(), "Test");
    assert!(!radio.is_selected());
    radio.select();
    assert!(radio.is_selected());
}

#[test]
fn test_radio_clear_dirty() {
    let mut radio = Radio::new(WidgetId::default_id(), "Test");
    assert!(radio.needs_render());
    radio.clear_dirty();
    assert!(!radio.needs_render());
}

#[test]
fn test_label_new() {
    let label = Label::new("Hello");
    let area = Rect::new(0, 0, 40, 1);
    let _plane = label.render(area);
}

#[test]
fn test_label_with_theme() {
    let label = Label::new("Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 1);
    let _plane = label.render(area);
}

#[test]
fn test_label_render() {
    let label = Label::new("Hello");
    let area = Rect::new(0, 0, 40, 1);
    let plane = label.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_label_render_chars() {
    let label = Label::new("ABC");
    let area = Rect::new(0, 0, 40, 1);
    let plane = label.render(area);
    assert_eq!(plane.cells[0].char, 'A');
    assert_eq!(plane.cells[1].char, 'B');
    assert_eq!(plane.cells[2].char, 'C');
}

#[test]
fn test_label_set_text() {
    let mut label = Label::new("Hello");
    label.set_text("World");
    let area = Rect::new(0, 0, 40, 1);
    let plane = label.render(area);
    assert_eq!(plane.cells[0].char, 'W');
    assert_eq!(plane.cells[1].char, 'o');
}

#[test]
fn test_label_clear_dirty() {
    let mut label = Label::new("Test");
    assert!(label.needs_render());
    label.clear_dirty();
    assert!(!label.needs_render());
}

#[test]
fn test_label_focusable_returns_false() {
    let label = Label::new("Test");
    assert!(!label.focusable());
}

#[test]
fn test_spinner_new() {
    let spinner = Spinner::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 10, 1);
    let _plane = spinner.render(area);
}

#[test]
fn test_spinner_with_theme() {
    let spinner = Spinner::new(WidgetId::default_id()).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 10, 1);
    let _plane = spinner.render(area);
}

#[test]
fn test_spinner_render() {
    let spinner = Spinner::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 10, 1);
    let plane = spinner.render(area);
    assert_eq!(plane.width, 10);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_spinner_clear_dirty() {
    let mut spinner = Spinner::new(WidgetId::default_id());
    assert!(spinner.needs_render());
    spinner.clear_dirty();
    assert!(!spinner.needs_render());
}

#[test]
fn test_spinner_tick_advances_frame() {
    use std::thread::sleep;
    use std::time::Duration;

    let mut spinner = Spinner::new(WidgetId::default_id());
    assert_eq!(spinner.current_frame(), '|');

    sleep(Duration::from_millis(150));
    spinner.tick();
    assert_eq!(spinner.current_frame(), '/');

    sleep(Duration::from_millis(150));
    spinner.tick();
    assert_eq!(spinner.current_frame(), '-');
}

#[test]
fn test_spinner_tick_no_advance_before_100ms() {
    let mut spinner = Spinner::new(WidgetId::default_id());
    assert_eq!(spinner.current_frame(), '|');
    spinner.tick(); // < 100ms since creation, should not advance
    assert_eq!(spinner.current_frame(), '|');
}

#[test]
fn test_spinner_with_frames() {
    let spinner = Spinner::new(WidgetId::default_id()).with_frames(vec!['◐', '◓', '◑', '◒']);
    assert_eq!(spinner.current_frame(), '◐');

    let spinner = Spinner::new(WidgetId::default_id()).with_frames(vec![]);
    assert_eq!(spinner.current_frame(), '|'); // empty falls back to default
}

#[test]
fn test_spinner_tick_cycles() {
    use std::thread::sleep;
    use std::time::Duration;

    let mut spinner = Spinner::new(WidgetId::default_id());
    // 4 default frames: | / - \
    sleep(Duration::from_millis(150));
    spinner.tick(); // -> /
    sleep(Duration::from_millis(150));
    spinner.tick(); // -> -
    sleep(Duration::from_millis(150));
    spinner.tick(); // -> \
    sleep(Duration::from_millis(150));
    spinner.tick(); // -> | (wraps)
    assert_eq!(spinner.current_frame(), '|');
}

#[test]
fn test_progress_bar_new() {
    let pb = ProgressBar::new(WidgetId::default_id());
    assert_eq!(pb.progress(), 0.0);
}

#[test]
fn test_progress_bar_with_theme() {
    let pb = ProgressBar::new(WidgetId::default_id()).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 1);
    let _plane = pb.render(area);
}

#[test]
fn test_progress_bar_render() {
    let pb = ProgressBar::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 40, 1);
    let plane = pb.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_progress_bar_set_progress() {
    let mut pb = ProgressBar::new(WidgetId::default_id());
    pb.set_progress(0.5);
    assert_eq!(pb.progress(), 0.5);
}

#[test]
fn test_progress_bar_clamp() {
    let mut pb = ProgressBar::new(WidgetId::default_id());
    pb.set_progress(1.5);
    assert_eq!(pb.progress(), 1.0);
    pb.set_progress(-0.5);
    assert_eq!(pb.progress(), 0.0);
}

#[test]
fn test_progress_bar_clear_dirty() {
    let mut pb = ProgressBar::new(WidgetId::default_id());
    assert!(pb.needs_render());
    pb.clear_dirty();
    assert!(!pb.needs_render());
}

#[test]
fn test_slider_new() {
    let slider = Slider::new(WidgetId::default_id());
    assert_eq!(slider.value(), 0.5);
}

#[test]
fn test_slider_with_theme() {
    let slider = Slider::new(WidgetId::default_id()).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 1);
    let _plane = slider.render(area);
}

#[test]
fn test_slider_render() {
    let slider = Slider::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 40, 1);
    let plane = slider.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_slider_range() {
    let slider = Slider::new(WidgetId::default_id()).with_range(0.0, 100.0);
    assert_eq!(slider.value(), 50.0);
}

#[test]
fn test_slider_set_value() {
    let mut slider = Slider::new(WidgetId::default_id());
    slider.set_value(0.75);
    assert_eq!(slider.value(), 0.75);
}

#[test]
fn test_slider_clamp() {
    let mut slider = Slider::new(WidgetId::default_id()).with_range(0.0, 100.0);
    slider.set_value(150.0);
    assert_eq!(slider.value(), 100.0);
    slider.set_value(-10.0);
    assert_eq!(slider.value(), 0.0);
}

#[test]
fn test_slider_clear_dirty() {
    let mut slider = Slider::new(WidgetId::default_id());
    assert!(slider.needs_render());
    slider.clear_dirty();
    assert!(!slider.needs_render());
}

#[test]
fn test_gauge_new() {
    let gauge = Gauge::new("CPU");
    assert_eq!(gauge.label, "CPU");
    assert_eq!(gauge.value(), 0.0);
}

#[test]
fn test_gauge_with_theme() {
    let gauge = Gauge::new("Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 30, 3);
    let _plane = gauge.render(area);
}

#[test]
fn test_gauge_render() {
    let gauge = Gauge::new("CPU");
    let area = Rect::new(0, 0, 30, 3);
    let plane = gauge.render(area);
    assert_eq!(plane.width, 30);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_gauge_set_value() {
    let mut gauge = Gauge::new("CPU");
    gauge.set_value(50.0);
    assert_eq!(gauge.value(), 50.0);
}

#[test]
fn test_gauge_percentage() {
    let mut gauge = Gauge::new("CPU");
    gauge.set_value(50.0);
    assert_eq!(gauge.percentage(), 50.0);
}

#[test]
fn test_gauge_fill_color() {
    let mut gauge = Gauge::new("CPU");
    gauge.set_value(50.0);
    let color = gauge.fill_color();
    assert_eq!(color, Theme::default().success);
}

#[test]
fn test_gauge_warn_threshold() {
    let mut gauge = Gauge::new("CPU");
    gauge.set_value(75.0);
    let color = gauge.fill_color();
    assert_eq!(color, Theme::default().warning);
}

#[test]
fn test_gauge_crit_threshold() {
    let mut gauge = Gauge::new("CPU");
    gauge.set_value(95.0);
    let color = gauge.fill_color();
    assert_eq!(color, Theme::default().error);
}

#[test]
fn test_gauge_clear_dirty() {
    let mut gauge = Gauge::new("CPU");
    assert!(gauge.needs_render());
    gauge.clear_dirty();
    assert!(!gauge.needs_render());
}

#[test]
fn test_breadcrumbs_new() {
    let crumbs = vec!["home".to_string(), "user".to_string()];
    let bc = Breadcrumbs::new(crumbs.clone());
    let area = Rect::new(0, 0, 80, 1);
    let _plane = bc.render(area);
}

#[test]
fn test_breadcrumbs_render() {
    let crumbs = vec!["home".to_string(), "user".to_string()];
    let bc = Breadcrumbs::new(crumbs);
    let area = Rect::new(0, 0, 80, 1);
    let plane = bc.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_breadcrumbs_clear_dirty() {
    let crumbs = vec!["home".to_string()];
    let mut bc = Breadcrumbs::new(crumbs);
    assert!(bc.needs_render());
    bc.clear_dirty();
    assert!(!bc.needs_render());
}

#[test]
fn test_status_badge_new() {
    let badge = StatusBadge::new(WidgetId::new(1));
    assert_eq!(badge.id(), WidgetId::new(1));
}

#[test]
fn test_status_badge_with_theme() {
    let badge = StatusBadge::new(WidgetId::new(1)).with_theme(Theme::cyberpunk());
    assert_eq!(&*badge.theme.name, "cyberpunk");
}

#[test]
fn test_status_badge_render() {
    let badge = StatusBadge::new(WidgetId::new(1)).with_status("OK");
    let plane = badge.render(Rect::new(0, 0, 6, 1));
    assert_eq!(plane.cells[0].char, '[');
    assert_eq!(plane.cells[1].char, 'O');
    assert_eq!(plane.cells[2].char, 'K');
    assert_eq!(plane.cells[3].char, ']');
}

#[test]
fn test_status_badge_clear_dirty() {
    let mut badge = StatusBadge::new(WidgetId::new(1));
    assert!(badge.needs_render());
    badge.clear_dirty();
    assert!(!badge.needs_render());
}

#[test]
fn test_status_bar_new() {
    let bar = StatusBar::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 80, 1);
    let _plane = bar.render(area);
}

#[test]
fn test_status_bar_with_theme() {
    let bar = StatusBar::new(WidgetId::default_id()).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 80, 1);
    let _plane = bar.render(area);
}

#[test]
fn test_status_bar_render() {
    let bar = StatusBar::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 80, 1);
    let plane = bar.render(area);
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_status_bar_add_segment() {
    let bar =
        StatusBar::new(WidgetId::default_id()).add_segment(StatusSegment::new("Test segment"));
    let area = Rect::new(0, 0, 80, 1);
    let plane = bar.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_select_new() {
    let select = Select::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 20, 1);
    let _plane = select.render(area);
}

#[test]
fn test_select_with_options() {
    let options = vec!["Option A".to_string(), "Option B".to_string()];
    let select = Select::new(WidgetId::default_id()).with_options(options);
    let area = Rect::new(0, 0, 20, 1);
    let _plane = select.render(area);
}

#[test]
fn test_select_with_theme() {
    let select = Select::new(WidgetId::default_id()).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 20, 1);
    let _plane = select.render(area);
}

#[test]
fn test_select_render() {
    let options = vec!["A".to_string(), "B".to_string()];
    let select = Select::new(WidgetId::default_id()).with_options(options);
    let area = Rect::new(0, 0, 20, 1);
    let plane = select.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_select_selected_index() {
    let options = vec!["A".to_string(), "B".to_string()];
    let select = Select::new(WidgetId::default_id()).with_options(options);
    assert_eq!(select.selected_index(), 0);
}

#[test]
fn test_select_clear_dirty() {
    let mut select = Select::new(WidgetId::default_id());
    assert!(select.needs_render());
    select.clear_dirty();
    assert!(!select.needs_render());
}

#[test]
fn test_form_new() {
    let form = Form::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 40, 10);
    let _plane = form.render(area);
}

#[test]
fn test_form_add_field() {
    let form = Form::new(WidgetId::default_id()).add_field("Name");
    let area = Rect::new(0, 0, 40, 10);
    let plane = form.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_form_with_theme() {
    let form = Form::new(WidgetId::default_id()).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 10);
    let _plane = form.render(area);
}

#[test]
fn test_form_render() {
    let form = Form::new(WidgetId::default_id()).add_field("Name");
    let area = Rect::new(0, 0, 40, 10);
    let plane = form.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 10);
}

#[test]
fn test_form_set_field_value() {
    let mut form = Form::new(WidgetId::default_id()).add_field("Name");
    form.set_field_value(0, "Alice");
}

#[test]
fn test_form_set_field_error() {
    let mut form = Form::new(WidgetId::default_id()).add_field("Email");
    form.set_field_error(0, "Invalid email");
}

#[test]
fn test_form_clear_dirty() {
    let mut form = Form::new(WidgetId::default_id());
    assert!(form.needs_render());
    form.clear_dirty();
    assert!(!form.needs_render());
}

#[test]
fn test_log_viewer_new() {
    let viewer = LogViewer::new();
    let area = Rect::new(0, 0, 80, 20);
    let _plane = viewer.render(area);
}

#[test]
fn test_log_viewer_with_id() {
    let viewer = LogViewer::with_id(WidgetId::new(5));
    assert_eq!(viewer.id, WidgetId::new(5));
}

#[test]
fn test_log_viewer_with_theme() {
    let viewer = LogViewer::new().with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 80, 20);
    let _plane = viewer.render(area);
}

#[test]
fn test_log_viewer_render() {
    let viewer = LogViewer::new();
    let area = Rect::new(0, 0, 80, 20);
    let plane = viewer.render(area);
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 20);
}

#[test]
fn test_log_viewer_max_lines() {
    let viewer = LogViewer::new().max_lines(100);
    assert_eq!(viewer.max_lines, 100);
}

#[test]
fn test_log_viewer_clear_dirty() {
    let mut viewer = LogViewer::new();
    assert!(viewer.needs_render());
    viewer.clear_dirty();
    assert!(!viewer.needs_render());
}

#[test]
fn test_split_pane_new() {
    let split = SplitPane::new(Orientation::Horizontal);
    let area = Rect::new(0, 0, 80, 24);
    let _plane = split.render(area);
}

#[test]
fn test_split_pane_render() {
    let split = SplitPane::new(Orientation::Horizontal);
    let area = Rect::new(0, 0, 80, 24);
    let plane = split.render(area);
    assert!(plane.width > 0);
    assert_eq!(plane.height, 24);
}

#[test]
fn test_split_pane_ratio() {
    let split = SplitPane::new(Orientation::Horizontal).ratio(0.3);
    assert_eq!(split.get_ratio(), 0.3);
}

#[test]
fn test_split_pane_split_horizontal() {
    let split = SplitPane::new(Orientation::Horizontal);
    let area = Rect::new(0, 0, 80, 24);
    let (left, right) = split.split(area);
    assert!(left.width > 0);
    assert!(right.width > 0);
    assert_eq!(left.width + right.width, 80);
}

#[test]
fn test_split_pane_split_vertical() {
    let split = SplitPane::new(Orientation::Vertical);
    let area = Rect::new(0, 0, 80, 24);
    let (top, bottom) = split.split(area);
    assert!(top.height > 0);
    assert!(bottom.height > 0);
    assert_eq!(top.height + bottom.height, 24);
}

#[test]
fn test_split_pane_clear_dirty() {
    let mut split = SplitPane::new(Orientation::Horizontal);
    assert!(split.needs_render());
    split.clear_dirty();
    assert!(!split.needs_render());
}

#[test]
fn test_list_new() {
    let items = vec!["a", "b", "c"];
    let list = List::new(items);
    assert_eq!(list.len(), 3);
}

#[test]
fn test_list_render() {
    let items = vec!["Item 1", "Item 2", "Item 3"];
    let list = List::new(items);
    let area = Rect::new(0, 0, 40, 10);
    let plane = list.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 10);
}

#[test]
fn test_list_with_theme() {
    let items = vec!["a", "b"];
    let list = List::new(items).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 20, 5);
    let plane = list.render(area);
    assert_eq!(plane.width, 20);
}

#[test]
fn test_list_selected_index() {
    let items = vec!["a", "b", "c"];
    let list = List::new(items);
    assert_eq!(list.selected_index(), 0);
}

#[test]
fn test_list_clear_dirty() {
    let items = vec!["a", "b"];
    let mut list = List::new(items);
    assert!(list.needs_render());
    list.clear_dirty();
    assert!(!list.needs_render());
}

#[test]
fn test_table_new() {
    let table: Table<String> = Table::new(vec![]);
    assert_eq!(table.len(), 0);
}

#[test]
fn test_table_with_theme() {
    let table: Table<String> = Table::new(vec![]).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 80, 20);
    let _plane = table.render(area);
}

#[test]
fn test_table_render() {
    let cols = vec![
        Column {
            header: "Name".to_string(),
            width: 20,
        },
        Column {
            header: "Age".to_string(),
            width: 10,
        },
    ];
    let table: Table<String> = Table::new(cols);
    let area = Rect::new(0, 0, 80, 20);
    let plane = table.render(area);
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_table_clear_dirty() {
    let cols = vec![Column {
        header: "Name".to_string(),
        width: 20,
    }];
    let mut table: Table<String> = Table::new(cols);
    assert!(table.needs_render());
    table.clear_dirty();
    assert!(!table.needs_render());
}

#[test]
fn test_tabbar_new() {
    let tabs = vec!["Tab A", "Tab B", "Tab C"];
    let tabbar = TabBar::new(tabs.clone());
    assert_eq!(tabbar.active(), 0);
}

#[test]
fn test_tabbar_with_theme() {
    let tabs = vec!["Tab A", "Tab B"];
    let tabbar = TabBar::new(tabs).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 80, 3);
    let _plane = tabbar.render(area);
}

#[test]
fn test_tabbar_render() {
    let tabs = vec!["Tab A", "Tab B"];
    let tabbar = TabBar::new(tabs);
    let area = Rect::new(0, 0, 80, 3);
    let plane = tabbar.render(area);
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_tabbar_set_active() {
    let tabs = vec!["Tab A", "Tab B", "Tab C"];
    let mut tabbar = TabBar::new(tabs);
    tabbar.set_active(2);
    assert_eq!(tabbar.active(), 2);
}

#[test]
fn test_tabbar_clear_dirty() {
    let tabs = vec!["Tab A", "Tab B"];
    let mut tabbar = TabBar::new(tabs);
    assert!(tabbar.needs_render());
    tabbar.clear_dirty();
    assert!(!tabbar.needs_render());
}

#[test]
fn test_tree_new() {
    let tree = Tree::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 40, 20);
    let _plane = tree.render(area);
}

#[test]
fn test_tree_with_root() {
    let root = TreeNode::new("Root");
    let tree = Tree::new(WidgetId::default_id()).with_root(vec![root]);
    let area = Rect::new(0, 0, 40, 20);
    let _plane = tree.render(area);
}

#[test]
fn test_tree_with_theme() {
    let tree = Tree::new(WidgetId::default_id()).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 20);
    let _plane = tree.render(area);
}

#[test]
fn test_tree_render() {
    let root = TreeNode::new("Root");
    let tree = Tree::new(WidgetId::default_id()).with_root(vec![root]);
    let area = Rect::new(0, 0, 40, 20);
    let plane = tree.render(area);
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_tree_node_new() {
    let node = TreeNode::new("Test");
    assert_eq!(node.label, "Test");
    assert!(!node.expanded);
    assert!(node.children.is_empty());
}

#[test]
fn test_tree_node_add_child() {
    let mut node = TreeNode::new("Parent");
    node.add_child(TreeNode::new("Child"));
    assert_eq!(node.children.len(), 1);
}

#[test]
fn test_tree_clear_dirty() {
    let tree = Tree::new(WidgetId::default_id());
    assert!(tree.needs_render());
}

#[test]
fn test_command_palette_new() {
    let commands = vec![CommandItem {
        id: "test",
        name: "Test Command",
        category: "Testing",
    }];
    let palette = CommandPalette::new(commands);
    let area = Rect::new(0, 0, 40, 20);
    let _plane = palette.render(area);
}

#[test]
fn test_command_palette_with_theme() {
    let commands = vec![CommandItem {
        id: "test",
        name: "Test",
        category: "Test",
    }];
    let palette = CommandPalette::new(commands).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 20);
    let _plane = palette.render(area);
}

#[test]
fn test_command_palette_render() {
    let commands = vec![CommandItem {
        id: "test",
        name: "Test",
        category: "Test",
    }];
    let palette = CommandPalette::new(commands);
    let area = Rect::new(0, 0, 40, 20);
    let plane = palette.render(area);
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_command_palette_show_hide() {
    let commands = vec![CommandItem {
        id: "test",
        name: "Test",
        category: "Test",
    }];
    let mut palette = CommandPalette::new(commands);
    assert!(!palette.is_visible());
    palette.show();
    assert!(palette.is_visible());
    palette.hide();
    assert!(!palette.is_visible());
}

#[test]
fn test_command_palette_clear_dirty() {
    let commands = vec![CommandItem {
        id: "test",
        name: "Test",
        category: "Test",
    }];
    let mut palette = CommandPalette::new(commands);
    palette.show();
    palette.clear_dirty();
    assert!(!palette.needs_render());
}

#[test]
fn test_confirm_dialog_new() {
    let dialog = ConfirmDialog::new("Title", "Message");
    assert_eq!(dialog.title, "Title");
    assert_eq!(dialog.message, "Message");
}

#[test]
fn test_confirm_dialog_with_id() {
    let dialog = ConfirmDialog::with_id(WidgetId::new(5), "Title", "Message");
    assert_eq!(dialog.id, WidgetId::new(5));
}

#[test]
fn test_confirm_dialog_with_theme() {
    let dialog = ConfirmDialog::new("Title", "Message").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 7);
    let _plane = dialog.render(area);
}

#[test]
fn test_confirm_dialog_render() {
    let dialog = ConfirmDialog::new("Title", "Message");
    let area = Rect::new(0, 0, 40, 7);
    let plane = dialog.render(area);
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_confirm_dialog_clear_dirty() {
    let mut dialog = ConfirmDialog::new("Title", "Message");
    assert!(dialog.needs_render());
    dialog.clear_dirty();
    assert!(!dialog.needs_render());
}

#[test]
fn test_modal_new() {
    let modal = Modal::new("Test Title");
    let area = Rect::new(0, 0, 40, 5);
    let _plane = modal.render(area);
}

#[test]
fn test_modal_with_theme() {
    let modal = Modal::new("Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 5);
    let _plane = modal.render(area);
}

#[test]
fn test_modal_render() {
    let modal = Modal::new("Test");
    let area = Rect::new(0, 0, 40, 5);
    let plane = modal.render(area);
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_modal_clear_dirty() {
    let mut modal = Modal::new("Test");
    assert!(modal.needs_render());
    modal.clear_dirty();
    assert!(!modal.needs_render());
}

#[test]
fn test_hud_new() {
    let hud = Hud::new(50);
    assert!(hud.is_visible());
}

#[test]
fn test_hud_with_theme() {
    let hud = Hud::new(50).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 30, 10);
    let _plane = hud.render(area);
}

#[test]
fn test_hud_render() {
    let hud = Hud::new(50);
    let area = Rect::new(0, 0, 30, 10);
    let plane = hud.render(area);
    assert_eq!(plane.width, 30);
    assert_eq!(plane.height, 10);
}

#[test]
fn test_hud_show_hide() {
    let mut hud = Hud::new(50);
    assert!(hud.is_visible());
    hud.hide();
    assert!(!hud.is_visible());
    hud.show();
    assert!(hud.is_visible());
}

#[test]
fn test_hud_new_with_id() {
    let hud = Hud::new_with_id(WidgetId::new(42), 100);
    assert!(hud.is_visible());
    assert_eq!(hud.position(), (0, 0));
}

#[test]
fn test_hud_with_size() {
    let hud = Hud::new_with_id(WidgetId::new(1), 10).with_size(50, 20);
    let area = Rect::new(0, 0, 50, 20);
    let plane = hud.render(area);
    assert_eq!(plane.width, 50);
    assert_eq!(plane.height, 20);
}

#[test]
fn test_hud_render_text() {
    let hud = Hud::new(10);
    let plane = hud.render_text(0, 0, "HUD", Color::Ansi(15), Color::Ansi(0));
    assert_eq!(plane.width, 30); // default width
    assert_eq!(plane.height, 10); // default height
    assert_eq!(plane.cells[0].char, 'H');
    assert_eq!(plane.cells[0].fg, Color::Ansi(15));
    assert_eq!(plane.cells[0].bg, Color::Ansi(0));
}

#[test]
fn test_hud_render_gauge() {
    let hud = Hud::new(10).with_theme(Theme::cyberpunk());
    let plane = hud.render_gauge(0, 0, "CPU:", 50.0, 100.0, 20);
    assert_eq!(plane.width, 30);
    assert_eq!(plane.height, 10);
    assert_eq!(plane.cells[0].char, 'C');
    assert_eq!(plane.cells[1].char, 'P');
    assert_eq!(plane.cells[2].char, 'U');
    assert_eq!(plane.cells[3].char, ':');
}

#[test]
fn test_context_menu_new() {
    let menu = ContextMenu::new(vec![("Open", ContextAction::Open)]);
    let area = Rect::new(0, 0, 20, 10);
    let _plane = menu.render(area);
}

#[test]
fn test_context_menu_with_theme() {
    let menu = ContextMenu::new(vec![("Open", ContextAction::Open)]).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 20, 10);
    let _plane = menu.render(area);
}

#[test]
fn test_context_menu_render() {
    let menu = ContextMenu::new(vec![("Open", ContextAction::Open)]);
    let area = Rect::new(0, 0, 20, 10);
    let plane = menu.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_context_menu_clear_dirty() {
    let mut menu = ContextMenu::new(vec![("Open", ContextAction::Open)]);
    assert!(menu.needs_render());
    menu.clear_dirty();
    assert!(!menu.needs_render());
}

#[test]
fn test_toast_new() {
    let toast = Toast::new(WidgetId::new(1), "Hello");
    assert_eq!(toast.message(), "Hello");
}

#[test]
fn test_toast_with_kind() {
    let toast = Toast::new(WidgetId::new(1), "Error").with_kind(ToastKind::Error);
    let area = Rect::new(0, 0, 40, 1);
    let _plane = toast.render(area);
}

#[test]
fn test_toast_with_theme() {
    let toast = Toast::new(WidgetId::new(1), "Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 1);
    let _plane = toast.render(area);
}

#[test]
fn test_toast_render() {
    let toast = Toast::new(WidgetId::new(1), "Hello");
    let area = Rect::new(0, 0, 40, 1);
    let plane = toast.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_toast_not_expired_immediately() {
    let toast = Toast::new(WidgetId::new(1), "Hello");
    assert!(!toast.is_expired());
}

#[test]
fn test_tooltip_new() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Hint");
    assert_eq!(tooltip.text(), "Hint");
}

#[test]
fn test_tooltip_with_theme() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Hint").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 30, 3);
    let _plane = tooltip.render(area);
}

#[test]
fn test_tooltip_render() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Hint");
    let area = Rect::new(0, 0, 30, 3);
    let plane = tooltip.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_tooltip_z_index() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Hint");
    assert_eq!(tooltip.z_index(), 100);
}

#[test]
fn test_menu_bar_new() {
    let menu_bar = MenuBar::new(WidgetId::new(1));
    let area = Rect::new(0, 0, 80, 1);
    let _plane = menu_bar.render(area);
}

#[test]
fn test_menu_bar_with_theme() {
    let menu_bar = MenuBar::new(WidgetId::new(1)).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 80, 1);
    let _plane = menu_bar.render(area);
}

#[test]
fn test_menu_bar_render() {
    let menu_bar = MenuBar::new(WidgetId::new(1));
    let area = Rect::new(0, 0, 80, 1);
    let plane = menu_bar.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_menu_bar_with_entries() {
    let entry = MenuEntry::new("File").add_item(MenuItem::new("Open"));
    let menu_bar = MenuBar::new(WidgetId::new(1)).with_entries(vec![entry]);
    let area = Rect::new(0, 0, 80, 1);
    let plane = menu_bar.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_menu_bar_clear_dirty() {
    let mut menu_bar = MenuBar::new(WidgetId::new(1));
    assert!(menu_bar.needs_render());
    menu_bar.clear_dirty();
    assert!(!menu_bar.needs_render());
}

#[test]
fn test_menu_item_new() {
    let item = MenuItem::new("Open");
    assert!(item.enabled);
}

#[test]
fn test_menu_item_with_enabled() {
    let item = MenuItem::new("Open").with_enabled(false);
    assert!(!item.enabled);
}

#[test]
fn test_all_themes_have_unique_names() {
    let themes = vec![
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
        Theme::warm(),
        Theme::cool(),
        Theme::forest(),
        Theme::sunset(),
        Theme::mono(),
    ];
    let mut names: Vec<String> = themes.iter().map(|t| t.name.to_string()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 20, "Expected 20 unique theme names");
}

#[test]
fn test_all_themes_are_dark_or_light() {
    let themes = vec![
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
        Theme::warm(),
        Theme::cool(),
        Theme::forest(),
        Theme::sunset(),
        Theme::mono(),
    ];
    for theme in themes {
        assert!(
            theme.kind == dracon_terminal_engine::framework::theme::ThemeKind::Dark
                || theme.kind == dracon_terminal_engine::framework::theme::ThemeKind::Light,
            "Theme '{}' must be Dark or Light",
            theme.name
        );
    }
}

#[test]
fn test_theme_dark_is_default() {
    let default = Theme::default();
    assert_eq!(&*default.name, "dark");
}

mod common;

use dracon_terminal_engine::framework::command::ParsedOutput;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::gauge::Gauge;
use ratatui::layout::Rect;

#[test]
fn test_gauge_new() {
    let g = Gauge::new("CPU");
    assert_eq!(g.label, "CPU");
    assert_eq!(g.value, 0.0);
    assert_eq!(g.max, 100.0);
}

#[test]
fn test_gauge_with_id() {
    let g = Gauge::with_id(dracon_terminal_engine::framework::widget::WidgetId::new(7), "RAM");
    assert_eq!(g.id, dracon_terminal_engine::framework::widget::WidgetId::new(7));
    assert_eq!(g.label, "RAM");
}

#[test]
fn test_gauge_max() {
    let g = Gauge::new("Disk").max(1000.0);
    assert_eq!(g.max, 1000.0);
}

#[test]
fn test_gauge_warn_threshold() {
    let g = Gauge::new("CPU").warn_threshold(60.0);
    assert_eq!(g.warn_threshold, 60.0);
}

#[test]
fn test_gauge_crit_threshold() {
    let g = Gauge::new("CPU").crit_threshold(95.0);
    assert_eq!(g.crit_threshold, 95.0);
}

#[test]
fn test_gauge_bind_command() {
    use dracon_terminal_engine::framework::command::BoundCommand;
    let cmd = BoundCommand::new("cpu --percent").label("cpu");
    let g = Gauge::new("CPU").bind_command(cmd);
    assert_eq!(g.commands().len(), 1);
}

#[test]
fn test_gauge_set_value() {
    let mut g = Gauge::new("CPU");
    g.set_value(50.0);
    assert_eq!(g.value, 50.0);
}

#[test]
fn test_gauge_set_value_clamped() {
    let mut g = Gauge::new("CPU");
    g.set_value(150.0);
    assert_eq!(g.value, 100.0);
}

#[test]
fn test_gauge_percentage() {
    let mut g = Gauge::new("CPU");
    g.set_value(75.0);
    assert!((g.percentage() - 75.0).abs() < 0.001);
}

#[test]
fn test_gauge_percentage_zero_max() {
    let mut g = Gauge::new("CPU").max(0.0);
    g.set_value(50.0);
    assert_eq!(g.percentage(), 0.0);
}

#[test]
fn test_gauge_fill_color_normal() {
    let mut g = Gauge::new("CPU");
    g.set_value(50.0);
    assert_eq!(g.fill_color(), g.theme.success);
}

#[test]
fn test_gauge_render() {
    let mut g = Gauge::new("CPU");
    g.set_value(50.0);
    let plane = g.render(Rect::new(0, 0, 20, 3));
    assert_eq!(plane.cells[0].char, 'C');
}

#[test]
fn test_gauge_render_bar_chars() {
    let mut g = Gauge::new("CPU");
    g.set_value(50.0);
    let plane = g.render(Rect::new(0, 0, 20, 3));
    let bar_cell = &plane.cells[21];
    assert_eq!(bar_cell.char, '█');
}

#[test]
fn test_gauge_dirty_lifecycle() {
    let mut g = Gauge::new("CPU");
    assert!(g.needs_render());
    g.clear_dirty();
    assert!(!g.needs_render());
    g.set_value(25.0);
    assert!(g.needs_render());
}

#[test]
fn test_gauge_with_theme() {
    let theme = Theme::nord();
    let g = Gauge::new("CPU").with_theme(theme);
    assert_eq!(g.theme.name, "nord");
}

#[test]
fn test_gauge_value() {
    let mut g = Gauge::new("RAM");
    g.set_value(42.5);
    assert!((g.value() - 42.5).abs() < 0.001);
}

#[test]
fn test_gauge_apply_command_output_scalar() {
    let mut g = Gauge::new("CPU");
    g.apply_command_output(&ParsedOutput::Scalar("75.5".to_string()));
    assert!((g.value() - 75.5).abs() < 0.001);
}

#[test]
fn test_gauge_apply_command_output_ignores_non_scalar() {
    let mut g = Gauge::new("CPU");
    g.set_value(50.0);
    g.apply_command_output(&ParsedOutput::None);
    assert!((g.value() - 50.0).abs() < 0.001);
}

#[test]
fn test_gauge_apply_command_output_parses_invalid_as_zero() {
    let mut g = Gauge::new("CPU");
    g.apply_command_output(&ParsedOutput::Scalar("not-a-number".to_string()));
    assert_eq!(g.value(), 0.0);
}
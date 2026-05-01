mod common;

use dracon_terminal_engine::framework::command::ParsedOutput;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::status_badge::StatusBadge;
use ratatui::layout::Rect;

#[test]
fn test_status_badge_new() {
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1));
    assert_eq!(badge.id(), dracon_terminal_engine::framework::widget::WidgetId::new(1));
    assert_eq!(badge.status, "UNKNOWN");
}

#[test]
fn test_status_badge_with_status() {
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1)).with_status("OK");
    assert_eq!(badge.status, "OK");
}

#[test]
fn test_status_badge_with_label() {
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1)).with_label("Disk OK");
    assert_eq!(badge.label, "Disk OK");
}

#[test]
fn test_status_badge_render_ok() {
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1)).with_status("OK");
    let plane = badge.render(Rect::new(0, 0, 6, 1));
    assert_eq!(plane.cells[0].char, '[');
    assert_eq!(plane.cells[1].char, 'O');
    assert_eq!(plane.cells[2].char, 'K');
    assert_eq!(plane.cells[3].char, ']');
}

#[test]
fn test_status_badge_render_error() {
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1)).with_status("ERROR");
    let plane = badge.render(Rect::new(0, 0, 10, 1));
    assert_eq!(plane.cells[0].char, '[');
    assert_eq!(plane.cells[1].char, 'E');
    assert_eq!(plane.cells[3].char, 'R');
}

#[test]
fn test_status_badge_render_warn() {
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1)).with_status("WARNING");
    let plane = badge.render(Rect::new(0, 0, 10, 1));
    assert_eq!(plane.cells[0].char, '[');
    assert_eq!(plane.cells[1].char, 'W');
}

#[test]
fn test_status_badge_numeric_ok() {
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1)).with_status("1");
    let plane = badge.render(Rect::new(0, 0, 6, 1));
    assert_eq!(plane.cells[1].char, 'O');
}

#[test]
fn test_status_badge_numeric_zero() {
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1)).with_status("0");
    let plane = badge.render(Rect::new(0, 0, 10, 1));
    assert_eq!(plane.cells[1].char, 'E');
}

#[test]
fn test_status_badge_dirty_lifecycle() {
    let mut badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1));
    assert!(badge.needs_render());
    badge.clear_dirty();
    assert!(!badge.needs_render());
    badge.set_status("OK");
    assert!(badge.needs_render());
}

#[test]
fn test_status_badge_commands() {
    use dracon_terminal_engine::framework::command::BoundCommand;
    let cmd = BoundCommand::new("test-cmd --json").label("test");
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1)).bind_command(cmd);
    let cmds = badge.commands();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].command, "test-cmd --json");
}

#[test]
fn test_status_badge_empty_status() {
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1)).with_status("");
    let plane = badge.render(Rect::new(0, 0, 10, 1));
    assert_eq!(plane.cells[1].char, 'E');
    assert_eq!(plane.cells[2].char, 'M');
    assert_eq!(plane.cells[3].char, 'P');
    assert_eq!(plane.cells[4].char, 'T');
}

#[test]
fn test_status_badge_focusable_returns_true_by_default() {
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1));
    assert!(badge.focusable());
}

#[test]
fn test_status_badge_z_index() {
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1));
    assert_eq!(badge.z_index(), 0);
}

#[test]
fn test_status_badge_with_theme() {
    let theme = Theme::cyberpunk();
    let badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1)).with_theme(theme);
    assert_eq!(badge.theme.name, "cyberpunk");
}

#[test]
fn test_status_badge_set_area() {
    let mut badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1));
    badge.set_area(Rect::new(5, 5, 20, 2));
    let area = badge.area();
    assert_eq!(area.x, 5);
    assert_eq!(area.y, 5);
    assert_eq!(area.width, 20);
    assert_eq!(area.height, 2);
}

#[test]
fn test_status_badge_apply_command_output_scalar() {
    let mut badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1));
    badge.apply_command_output(&ParsedOutput::Scalar("OK".to_string()));
    assert_eq!(badge.status, "OK");
}

#[test]
fn test_status_badge_apply_command_output_ignores_non_scalar() {
    let mut badge = StatusBadge::new(dracon_terminal_engine::framework::widget::WidgetId::new(1));
    badge.set_status("OK");
    badge.apply_command_output(&ParsedOutput::None);
    assert_eq!(badge.status, "OK");
}
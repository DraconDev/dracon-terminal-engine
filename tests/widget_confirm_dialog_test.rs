mod common;

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::confirm_dialog::{ConfirmDialog, ConfirmResult};
use ratatui::layout::Rect;

#[test]
fn test_confirm_dialog_new() {
    let dlg = ConfirmDialog::new("Title", "Message");
    assert_eq!(dlg.title, "Title");
    assert_eq!(dlg.message, "Message");
}

#[test]
fn test_confirm_dialog_with_id() {
    let dlg = ConfirmDialog::with_id(dracon_terminal_engine::framework::widget::WidgetId::new(5), "Title", "Msg");
    assert_eq!(dlg.id, dracon_terminal_engine::framework::widget::WidgetId::new(5));
}

#[test]
fn test_confirm_dialog_confirm_label() {
    let dlg = ConfirmDialog::new("t", "m").confirm_label("Delete");
    assert_eq!(dlg.confirm_label, "Delete");
}

#[test]
fn test_confirm_dialog_cancel_label() {
    let dlg = ConfirmDialog::new("t", "m").cancel_label("Abort");
    assert_eq!(dlg.cancel_label, "Abort");
}

#[test]
fn test_confirm_dialog_danger() {
    let dlg = ConfirmDialog::new("t", "m").danger(true);
    assert!(dlg.danger);
}

#[test]
fn test_confirm_dialog_bind_command() {
    use dracon_terminal_engine::framework::command::BoundCommand;
    let cmd = BoundCommand::new("rm -rf /").label("dangerous");
    let dlg = ConfirmDialog::new("t", "m").bind_command(cmd);
    assert_eq!(dlg.commands().len(), 1);
}

#[test]
fn test_confirm_dialog_result_starts_none() {
    let dlg = ConfirmDialog::new("t", "m");
    assert_eq!(dlg.confirmed(), None);
}

#[test]
fn test_confirm_dialog_clear_result() {
    let mut dlg = ConfirmDialog::new("t", "m");
    dlg.result = Some(ConfirmResult::Confirmed);
    dlg.clear_result();
    assert_eq!(dlg.confirmed(), None);
}

#[test]
fn test_confirm_dialog_render_box() {
    let dlg = ConfirmDialog::new("Confirm?", "Are you sure?");
    let plane = dlg.render(Rect::new(0, 0, 30, 7));
    assert_eq!(plane.cells[0].char, '┌');
    assert_eq!(plane.cells[29].char, '┐');
}

#[test]
fn test_confirm_dialog_render_title() {
    let dlg = ConfirmDialog::new("Delete All", "This cannot be undone");
    let plane = dlg.render(Rect::new(0, 0, 40, 7));
    let title_chars: Vec<char> = plane.cells[40..80].iter().map(|c| c.char).collect();
    let title_str: String = title_chars.into_iter().collect();
    assert!(title_str.contains("Delete All"));
}

#[test]
fn test_confirm_dialog_danger_border_color() {
    let dlg = ConfirmDialog::new("Danger", "Very bad").danger(true);
    let plane = dlg.render(Rect::new(0, 0, 30, 7));
    assert_eq!(plane.cells[0].fg, dlg.theme.error);
}

#[test]
fn test_confirm_dialog_focusable() {
    let dlg = ConfirmDialog::new("t", "m");
    assert!(dlg.focusable());
}

#[test]
fn test_confirm_dialog_dirty_lifecycle() {
    let mut dlg = ConfirmDialog::new("t", "m");
    assert!(dlg.needs_render());
    dlg.clear_dirty();
    assert!(!dlg.needs_render());
}

#[test]
fn test_confirm_dialog_with_theme() {
    let theme = Theme::cyberpunk();
    let dlg = ConfirmDialog::new("t", "m").with_theme(theme);
    assert_eq!(dlg.theme.name, "cyberpunk");
}
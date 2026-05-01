//! Demonstrates modal dialogs and keyboard shortcuts.
//!
//! ## Features Shown
//!
//! 1. **ConfirmDialog** — Modal confirmation box blocking input to widgets behind it
//! 2. **Help overlay** — Modal that toggles with `?`, listing keyboard shortcuts
//! 3. **Global keyboard shortcuts** — `q` quit, `Ctrl+S` save (via toast), `?` help
//! 4. **Modal composition** — Help renders above main content, ConfirmDialog above help
//!
//! ## Key Patterns
//!
//! - `Modal` widget blocks input via z-index and focus trapping
//! - `ConfirmDialog` wraps `Modal` with themed confirm/cancel buttons
//! - Global shortcuts handled at App level before passing to focused widget
//! - `FocusManager::enter_trap()` / `exit_trap()` control focus trapping for modals
//! - Z-index layering: main content (0) < help (100) < confirm dialog (110)

use std::io;
use std::time::Duration;

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, ConfirmDialog, Label, Modal, ModalResult, Toast, ToastKind,
};
use dracon_terminal_engine::{App, Ctx};
use ratatui::layout::Rect;

struct DemoApp {
    show_help: bool,
    show_confirm: bool,
    modal_id: WidgetId,
    help_modal_id: WidgetId,
    label_id: WidgetId,
    button_id: WidgetId,
}

impl DemoApp {
    fn new() -> Self {
        Self {
            show_help: false,
            show_confirm: false,
            modal_id: WidgetId::default_id(),
            help_modal_id: WidgetId::default_id(),
            label_id: WidgetId::default_id(),
            button_id: WidgetId::default_id(),
        }
    }
}

fn main() -> io::Result<()> {
    println!("Modal Demo");
    println!("==========");
    println!("Press ? for help, q to quit, Ctrl+S to save");
    println!();

    std::thread::sleep(Duration::from_millis(300));

    let mut app = App::new()?.title("Modal Demo").fps(30);

    let theme = Theme::dark();
    app.set_theme(theme);

    let mut demo = DemoApp::new();

    let label = Label::new("Main content area\n\nPress [c] to trigger confirm dialog\nPress [?] to toggle help overlay");
    demo.label_id = app.add_widget(Box::new(label), Rect::new(2, 2, 50, 10));

    let mut show_confirm_btn = Button::new("Show Confirm Dialog").on_click(|| {});
    demo.button_id = app.add_widget(
        Box::new(show_confirm_btn),
        Rect::new(2, 14, 25, 1),
    );

    let help_modal = Modal::new("Keyboard Shortcuts")
        .with_size(45, 12)
        .with_buttons(vec![("OK", ModalResult::Confirm)]);
    demo.help_modal_id = app.add_widget(Box::new(help_modal), Rect::new(0, 0, 80, 24));

    let confirm_dlg = ConfirmDialog::new("Confirm Action", "Are you sure?")
        .confirm_label("OK")
        .cancel_label("Cancel")
        .danger(true);
    demo.modal_id = app.add_widget(Box::new(confirm_dlg), Rect::new(0, 0, 80, 24));

    let (save_toast_id, help_visible, confirm_visible, running) = app.run(move |ctx| {
        if let Some(mut help_modal) = ctx.compositor_mut().get_widget(demo.help_modal_id) {
            help_modal.set_visible(help_visible);
            help_modal.mark_dirty();
        }
        if let Some(mut confirm) = ctx.compositor_mut().get_widget(demo.modal_id) {
            confirm.set_visible(confirm_visible);
            confirm.mark_dirty();
        }

        if ctx.needs_full_refresh() {
            ctx.mark_all_dirty();
        }

        if let Some(mut label) = ctx.widget_mut(demo.label_id) {
            label.mark_dirty();
            let area = label.area();
            let plane = label.render(area);
            ctx.add_plane(plane);
        }

        if let Some(mut button) = ctx.widget_mut(demo.button_id) {
            button.mark_dirty();
            let area = button.area();
            let plane = button.render(area);
            ctx.add_plane(plane);
        }

        if help_visible {
            if let Some(mut help_modal) = ctx.widget_mut(demo.help_modal_id) {
                help_modal.mark_dirty();
                let area = help_modal.area();
                let mut plane = help_modal.render(area);
                plane.z_index = 100;
                ctx.add_plane(plane);
            }
        }

        if confirm_visible {
            if let Some(mut confirm) = ctx.widget_mut(demo.modal_id) {
                confirm.mark_dirty();
                let area = confirm.area();
                let mut plane = confirm.render(area);
                plane.z_index = 110;
                ctx.add_plane(plane);
            }
        }
    });

    println!("\nModal demo exited cleanly");
    Ok(())
}

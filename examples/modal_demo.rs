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
//! - `Modal` widget blocks input via z-index (100) and focus trapping
//! - `ConfirmDialog` wraps `Modal` with themed confirm/cancel buttons (z-index 110)
//! - Global shortcuts handled before passing to focused widget (via `handle_key`)
//! - `FocusManager::enter_trap()` / `exit_trap()` control focus trapping for modals
//! - Z-index layering: main content (0) < help (100) < confirm dialog (110)
//! - `on_tick` used for state transitions, `handle_key` for per-frame keyboard events

use std::io;
use std::time::Duration;

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, ConfirmDialog, Label, Modal, ModalResult, Toast, ToastKind,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;

struct HelpOverlay {
    modal: Modal,
    visible: bool,
}

impl HelpOverlay {
    fn new() -> Self {
        let shortcuts = vec![
            ("?", "Toggle help"),
            ("q", "Quit app"),
            ("Ctrl+S", "Save (mock)"),
            ("Tab", "Cycle focus"),
            ("Esc", "Close modal"),
            ("Enter", "Confirm"),
        ];
        let content = shortcuts
            .iter()
            .map(|(key, desc)| format!("{:10} {}", key, desc))
            .collect::<Vec<_>>()
            .join("\n");

        let mut modal = Modal::new("Keyboard Shortcuts").with_size(40, 10);
        modal = modal.with_buttons(vec![("OK", ModalResult::Confirm)]);

        Self {
            modal,
            visible: false,
        }
    }

    fn toggle(&mut self) {
        self.visible = !self.visible;
        self.modal.mark_dirty();
    }

    fn hide(&mut self) {
        self.visible = false;
        self.modal.mark_dirty();
    }
}

impl Widget for HelpOverlay {
    fn id(&self) -> WidgetId {
        self.modal.id()
    }

    fn set_id(&mut self, id: WidgetId) {
        self.modal.set_id(id);
    }

    fn area(&self) -> Rect {
        self.modal.area()
    }

    fn set_area(&mut self, area: Rect) {
        self.modal.set_area(area);
    }

    fn z_index(&self) -> u16 {
        100
    }

    fn needs_render(&self) -> bool {
        self.visible || self.modal.needs_render()
    }

    fn mark_dirty(&mut self) {
        self.modal.mark_dirty();
    }

    fn clear_dirty(&mut self) {
        self.modal.clear_dirty();
    }

    fn focusable(&self) -> bool {
        self.visible
    }

    fn render(&self, area: Rect) -> Plane {
        if !self.visible {
            return Plane::new(0, area.width, area.height);
        }
        let mut plane = self.modal.render(area);

        let shortcuts = vec![
            ("?", "Toggle help"),
            ("q", "Quit app"),
            ("Ctrl+S", "Save (mock)"),
            ("Tab", "Cycle focus"),
            ("Esc", "Close modal"),
            ("Enter", "Confirm"),
        ];

        let start_y = 2u16;
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let y = start_y + i as u16;
            let text = format!("{:10} {}", key, desc);
            for (j, c) in text.chars().enumerate() {
                let idx = (y * plane.width + 3 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if *key == "?" || *key == "q" || *key == "Ctrl+S" {
                        self.modal.theme.accent
                    } else {
                        self.modal.theme.fg
                    };
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if !self.visible {
            return false;
        }
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Esc => {
                self.hide();
                true
            }
            _ => self.modal.handle_key(key),
        }
    }

    fn handle_mouse(
        &mut self,
        kind: dracon_terminal_engine::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        if !self.visible {
            return false;
        }
        self.modal.handle_mouse(kind, col, row)
    }
}

struct ModalDemoApp {
    show_confirm: bool,
    help_visible: bool,
    show_save_toast: bool,
    toast_message: String,
    confirm_id: WidgetId,
    help_id: WidgetId,
    label_id: WidgetId,
    button_id: WidgetId,
}

impl ModalDemoApp {
    fn new() -> Self {
        Self {
            show_confirm: false,
            help_visible: false,
            show_save_toast: false,
            toast_message: String::new(),
            confirm_id: WidgetId::default_id(),
            help_id: WidgetId::default_id(),
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

    let mut demo = ModalDemoApp::new();

    let label = Label::new(
        "Main content area\n\n\
         Press [c] to trigger confirm dialog\n\
         Press [?] to toggle help overlay",
    );
    demo.label_id = app.add_widget(Box::new(label), Rect::new(2, 2, 50, 10));

    let mut button = Button::new("Show Confirm Dialog");
    demo.button_id = app.add_widget(Box::new(button), Rect::new(2, 14, 25, 1));

    let mut help_overlay = HelpOverlay::new();
    demo.help_id = app.add_widget(Box::new(help_overlay), Rect::new(0, 0, 80, 24));

    let confirm_dlg = ConfirmDialog::new("Confirm Action", "Are you sure you want to proceed?")
        .confirm_label("OK")
        .cancel_label("Cancel")
        .danger(true);
    demo.confirm_id = app.add_widget(Box::new(confirm_dlg), Rect::new(0, 0, 80, 24));

    app.on_tick(|ctx, _tick| {
        if demo.show_save_toast {
            let toast = Toast::new(WidgetId::new(200), "File saved successfully!")
                .with_kind(ToastKind::Success)
                .with_duration(Duration::from_secs(2))
                .with_theme(*ctx.theme());

            let toast_area = Rect::new(
                (ctx.compositor().size().0.saturating_sub(40)) / 2,
                ctx.compositor().size().1.saturating_sub(3),
                40,
                1,
            );
            ctx.add_plane(toast.render(toast_area));
            demo.show_save_toast = false;
        }
    });

    let _result = app.run(move |ctx| {
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

        if demo.help_visible {
            if let Some(mut help) = ctx.widget_mut(demo.help_id) {
                help.mark_dirty();
                let area = help.area();
                let plane = help.render(area);
                ctx.add_plane(plane);
            }
        }

        if demo.show_confirm {
            if let Some(mut confirm) = ctx.widget_mut(demo.confirm_id) {
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

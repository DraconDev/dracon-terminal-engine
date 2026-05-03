#![allow(missing_docs)]
//! Demonstrates modal dialogs and keyboard shortcuts.
//!
//! ## Features Shown
//!
//! 1. **ConfirmDialog** — Modal confirmation box with OK/Cancel, ESC→Cancel, Enter→Confirm
//! 2. **Help overlay** — Modal toggled via button, listing keyboard shortcuts
//! 3. **Modal composition** — Help renders above main content, ConfirmDialog above help
//! 4. **Toast notifications** — Success toast after confirm
//!
//! ## Key Patterns
//!
//! - `Modal` widget blocks input via z-index (100) and focus trapping
//! - `ConfirmDialog` wraps `Modal` with themed confirm/cancel buttons (z-index 110)
//! - Focus trapping via `FocusManager::enter_trap()` / `exit_trap()`
//! - Z-index layering: main content (0) < help (100) < confirm dialog (110)
//! - Widgets handle their own keyboard events when focused via `handle_key`
//! - Buttons respond to Enter key and mouse clicks
//!
//! ## Event Flow
//!
//! The App routes keyboard events to the focused widget. The typical flow:
//! 1. Widget receives `handle_key` event when it has focus
//! 2. If widget doesn't handle it, event is not consumed (bubbles up)
//! 3. For modal composition, higher-z-index widgets render on top

use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, ConfirmDialog, ConfirmResult, Label, Modal, Toast, ToastKind,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

struct HelpOverlay<'a> {
    modal: Modal<'a>,
    visible: bool,
}

impl<'a> HelpOverlay<'a> {
    fn new() -> Self {
        let modal = Modal::new("Keyboard Shortcuts").with_size(40, 12);

        Self {
            modal,
            visible: false,
        }
    }
}

impl<'a> Widget for HelpOverlay<'a> {
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

        let shortcuts = [("?", "Toggle help"),
            ("q", "Quit app"),
            ("Ctrl+S", "Save (mock)"),
            ("Tab", "Cycle focus"),
            ("Esc", "Close modal"),
            ("Enter", "Confirm")];

        let start_y = 2u16;
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let y = start_y + i as u16;
            let text = format!("{:10} {}", key, desc);
            for (j, c) in text.chars().enumerate() {
                let idx = (y * plane.width + 3 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
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
                self.visible = false;
                self.modal.mark_dirty();
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

struct ModalDemoApp<'a> {
    show_confirm: bool,
    help_visible: bool,
    show_save_toast: bool,
    toast_message: String,
    label: Label,
    confirm_dialog: ConfirmDialog,
    help_overlay: HelpOverlay<'a>,
    confirm_btn: Button,
    help_btn: Button,
    area: Rect,
    should_quit: Arc<AtomicBool>,
}

impl<'a> ModalDemoApp<'a> {
    fn new(should_quit: Arc<AtomicBool>) -> Self {
        let label = Label::new(
            "Modal Demo\n\
             ──────────\n\
             Click buttons below to interact\n\
             \n\
             This demonstrates:\n\
             • ConfirmDialog (z=110) renders above help (z=100)\n\
             • Help overlay renders above main content (z=0)\n\
             • ESC closes modals, Enter confirms",
        );

        let confirm_dialog = ConfirmDialog::new("Confirm Action", "Are you sure?")
            .confirm_label("OK")
            .cancel_label("Cancel")
            .danger(true);

        let help_overlay = HelpOverlay::new();

        let confirm_btn = Button::new("Show Confirm Dialog");
        let help_btn = Button::new("Show Help (?)");

        Self {
            show_confirm: false,
            help_visible: false,
            show_save_toast: false,
            toast_message: String::new(),
            label,
            confirm_dialog,
            help_overlay,
            confirm_btn,
            help_btn,
            area: Rect::new(0, 0, 80, 24),
            should_quit,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Delegate to confirm dialog if visible
        if self.show_confirm {
            if self.confirm_dialog.handle_key(key) {
                if self.confirm_dialog.confirmed() == Some(ConfirmResult::Confirmed) {
                    self.show_confirm = false;
                    self.show_save_toast = true;
                    self.toast_message = "Action confirmed!".to_string();
                    self.confirm_dialog.clear_result();
                } else if self.confirm_dialog.confirmed() == Some(ConfirmResult::Cancelled) {
                    self.show_confirm = false;
                    self.confirm_dialog.clear_result();
                }
                return true;
            }
            if key.code == KeyCode::Esc {
                self.show_confirm = false;
                return true;
            }
            return false;
        }

        // Delegate to help overlay if visible
        if self.help_visible {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                self.help_visible = false;
                return true;
            }
            return self.help_overlay.handle_key(key);
        }

        // Global shortcuts
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            KeyCode::Char('?') => {
                self.help_visible = true;
                true
            }
            KeyCode::Enter => {
                self.show_confirm = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.show_confirm {
            if self.confirm_dialog.handle_mouse(kind, col, row) {
                if let MouseEventKind::Down(_) = kind {
                    if self.confirm_dialog.confirmed() == Some(ConfirmResult::Confirmed) {
                        self.show_confirm = false;
                        self.show_save_toast = true;
                        self.toast_message = "Action confirmed!".to_string();
                        self.confirm_dialog.clear_result();
                    } else if self.confirm_dialog.confirmed() == Some(ConfirmResult::Cancelled) {
                        self.show_confirm = false;
                        self.confirm_dialog.clear_result();
                    }
                }
                return true;
            }
            return false;
        }

        if self.help_visible {
            return self.help_overlay.handle_mouse(kind, col, row);
        }

        // Check button clicks
        let confirm_btn_area = Rect::new(2, 16, 25, 1);
        let help_btn_area = Rect::new(30, 16, 18, 1);

        if let MouseEventKind::Down(_) = kind {
            if col >= confirm_btn_area.x
                && col < confirm_btn_area.x + confirm_btn_area.width
                && row >= confirm_btn_area.y
                && row < confirm_btn_area.y + confirm_btn_area.height
            {
                self.show_confirm = true;
                return true;
            }
            if col >= help_btn_area.x
                && col < help_btn_area.x + help_btn_area.width
                && row >= help_btn_area.y
                && row < help_btn_area.y + help_btn_area.height
            {
                self.help_visible = true;
                return true;
            }
        }

        false
    }
}

struct ModalDemoRouter {
    target: Rc<RefCell<ModalDemoApp<'static>>>,
    id: WidgetId,
    area: Rect,
}

impl Widget for ModalDemoRouter {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn area(&self) -> Rect {
        self.area
    }
    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        false
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }
    fn render(&self, _area: Rect) -> Plane {
        Plane::new(0, 0, 0)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.target.borrow_mut().handle_key(key)
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.target.borrow_mut().handle_mouse(kind, col, row)
    }
}

fn main() -> io::Result<()> {
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let demo = Rc::new(RefCell::new(ModalDemoApp::new(should_quit)));
    let demo_for_render = Rc::clone(&demo);
    let demo_for_input = Rc::clone(&demo);

    let mut app = App::new()?.title("Modal Demo").fps(30);

    let theme = Theme::dark();
    app.set_theme(theme);

    // Register input router so keyboard/mouse events reach the demo
    let router = ModalDemoRouter {
        target: demo_for_input,
        id: WidgetId::new(100),
        area: Rect::new(0, 0, 80, 24),
    };
    app.add_widget(Box::new(router), Rect::new(0, 0, 80, 24));

    app.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
        }
    })
    .run(move |ctx| {
        let mut demo = demo_for_render.borrow_mut();

        if ctx.needs_full_refresh() {
            ctx.mark_all_dirty();
        }

        let (w, h) = ctx.compositor().size();
        demo.area = Rect::new(0, 0, w, h);

        let label_area = Rect::new(
            2,
            2,
            55.min(w.saturating_sub(4)),
            12.min(h.saturating_sub(4)),
        );
        demo.label.mark_dirty();
        let label_plane = demo.label.render(label_area);
        ctx.add_plane(label_plane);

        let confirm_btn_area = Rect::new(2, 16.min(h.saturating_sub(2)), 25, 1);
        demo.confirm_btn.mark_dirty();
        let btn_plane = demo.confirm_btn.render(confirm_btn_area);
        ctx.add_plane(btn_plane);

        let help_btn_area = Rect::new(
            30.min(w.saturating_sub(20)),
            16.min(h.saturating_sub(2)),
            18,
            1,
        );
        demo.help_btn.mark_dirty();
        let help_btn_plane = demo.help_btn.render(help_btn_area);
        ctx.add_plane(help_btn_plane);

        if demo.help_visible {
            demo.help_overlay.mark_dirty();
            let help_area = Rect::new(0, 0, w, h);
            let mut help_plane = demo.help_overlay.render(help_area);
            help_plane.z_index = 100;
            ctx.add_plane(help_plane);
        }

        if demo.show_confirm {
            demo.confirm_dialog.mark_dirty();
            let confirm_area = Rect::new(0, 0, w, h);
            let mut confirm_plane = demo.confirm_dialog.render(confirm_area);
            confirm_plane.z_index = 110;
            ctx.add_plane(confirm_plane);
        }

        if demo.show_save_toast {
            let toast = Toast::new(WidgetId::new(200), &demo.toast_message)
                .with_kind(ToastKind::Success)
                .with_duration(Duration::from_secs(2))
                .with_theme(Theme::dark());

            let toast_area = Rect::new((w.saturating_sub(40)) / 2, h.saturating_sub(3), 40, 1);
            ctx.add_plane(toast.render(toast_area));
            demo.show_save_toast = false;
        }
    })?;

    eprintln!("\nModal demo exited cleanly");
    Ok(())
}

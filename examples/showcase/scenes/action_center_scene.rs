//! Action Center scene — ContextMenu + ConfirmDialog + Toast demonstration.
//!
//! Right-click for context menu, confirm before delete, toast notifications.
//! All three interaction-pattern widgets working together in a realistic app.

#![allow(dead_code)]

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    ConfirmDialog, ConfirmResult, ContextMenu, ContextMenuItem, StatusBar, StatusSegment, Toast,
    ToastKind,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone, Debug)]
struct FileItem {
    name: String,
    kind: String,
    size: String,
}

pub struct ActionCenterScene {
    theme: Theme,
    keybindings: KeybindingSet,
    files: Vec<FileItem>,
    selected: Option<usize>,
    hovered: Option<usize>,
    context_menu: RefCell<ContextMenu>,
    confirm_dialog: RefCell<Option<ConfirmDialog>>,
    toasts: Vec<Toast>,
    status_bar: RefCell<StatusBar>,
    action_bridge: Rc<RefCell<Option<String>>>,
    show_help: bool,
    dirty: bool,
}

impl ActionCenterScene {
    pub fn new(theme: Theme) -> Self {
        let files = vec![
            FileItem { name: "src".into(), kind: "dir".into(), size: "—".into() },
            FileItem { name: "Cargo.toml".into(), kind: "file".into(), size: "1.2K".into() },
            FileItem { name: "README.md".into(), kind: "file".into(), size: "4.5K".into() },
            FileItem { name: "LICENSE".into(), kind: "file".into(), size: "1.1K".into() },
            FileItem { name: ".gitignore".into(), kind: "file".into(), size: "0.2K".into() },
            FileItem { name: "tests".into(), kind: "dir".into(), size: "—".into() },
            FileItem { name: "examples".into(), kind: "dir".into(), size: "—".into() },
            FileItem { name: "Makefile".into(), kind: "file".into(), size: "0.8K".into() },
            FileItem { name: "CHANGELOG.md".into(), kind: "file".into(), size: "12K".into() },
            FileItem { name: "deno.json".into(), kind: "file".into(), size: "0.5K".into() },
        ];

        // Bridge for context menu on_select callback
        let action_bridge = Rc::new(RefCell::new(None::<String>));
        let action_bridge_cb = Rc::clone(&action_bridge);

        let context_menu = ContextMenu::new(vec![
            ContextMenuItem::new("open", "Open"),
            ContextMenuItem::new("copy", "Copy Path"),
            ContextMenuItem::new("rename", "Rename"),
            ContextMenuItem::separator(),
            ContextMenuItem::new("delete", "Delete"),
        ])
        .with_theme(theme.clone())
        .with_width(22)
        .on_select(Box::new(move |id: &str| { *action_bridge_cb.borrow_mut() = Some(id.to_string()); }));

        let status_bar = StatusBar::new(WidgetId::new(603))
            .add_segment(StatusSegment::new("Right-click: menu | Del: delete | F1: help | Esc: back"))
            .with_theme(theme.clone());

        Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            files,
            selected: None,
            hovered: None,
            context_menu: RefCell::new(context_menu),
            confirm_dialog: RefCell::new(None),
            toasts: Vec::new(),
            status_bar: RefCell::new(status_bar),
            action_bridge,
            show_help: false,
            dirty: true,
        }
    }

    fn show_context_menu(&mut self, col: u16, row: u16) {
        self.context_menu.borrow_mut().set_anchor(col, row);
        self.context_menu.borrow_mut().show();
        self.dirty = true;
    }

    fn hide_context_menu(&mut self) {
        self.context_menu.borrow_mut().hide();
        self.dirty = true;
    }

    /// Check if context menu on_select fired via bridge, and execute
    fn sync_action_bridge(&mut self) {
        let action = self.action_bridge.borrow_mut().take();
        if let Some(id) = action {
            self.hide_context_menu();
            self.execute_context_action(&id);
        }
    }

    fn execute_context_action(&mut self, id: &str) {
        self.hide_context_menu();
        match id {
            "open" => {
                if let Some(idx) = self.selected {
                    if idx < self.files.len() {
                        self.add_toast(ToastKind::Info, &format!("Opened {}", self.files[idx].name));
                    }
                }
            }
            "copy" => {
                if let Some(idx) = self.selected {
                    if idx < self.files.len() {
                        self.add_toast(ToastKind::Info, &format!("Copied path: {}", self.files[idx].name));
                    }
                }
            }
            "rename" => {
                self.add_toast(ToastKind::Info, "Rename not yet implemented");
            }
            "delete" => {
                if let Some(idx) = self.selected {
                    if idx < self.files.len() {
                        let name = self.files[idx].name.clone();
                        *self.confirm_dialog.borrow_mut() = Some(
                            ConfirmDialog::new("Delete File", &format!("Delete \"{}\"?", name))
                                .danger(true)
                                .with_theme(self.theme.clone())
                        );
                    }
                }
            }
            _ => {}
        }
    }

    fn add_toast(&mut self, kind: ToastKind, message: &str) {
        let toast = Toast::new(WidgetId::next(), message)
            .with_kind(kind)
            .with_duration(Duration::from_secs(3))
            .with_theme(self.theme.clone());
        self.toasts.push(toast);
        self.dirty = true;
    }

    fn expire_toasts(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }
}

impl Scene for ActionCenterScene {
    fn on_enter(&mut self) {}
    fn on_exit(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // ── Title bar ──────────────────────────────────────────────
        draw_text(&mut plane, 1, 0, "Action Center", t.primary, t.bg, true);
        draw_text(&mut plane, 16, 0, "— ContextMenu + ConfirmDialog + Toast", t.fg_muted, t.bg, false);

        // ── File list ──────────────────────────────────────────────
        let list_y = 2u16;
        // Header
        draw_text(&mut plane, 1, list_y, "  Name           Kind  Size", t.fg_muted, t.bg, true);
        draw_text(&mut plane, 1, list_y + 1, "─".repeat(area.width as usize / 2).as_str(), t.outline, t.bg, false);

        for (i, file) in self.files.iter().enumerate() {
            let y = list_y + 2 + i as u16;
            if y >= area.height.saturating_sub(1) { break; }

            let is_selected = self.selected == Some(i);
            let is_hovered = self.hovered == Some(i);
            let bg = if is_selected { t.selection_bg } else if is_hovered { t.hover_bg } else { t.bg };
            let fg = if is_selected || is_hovered { t.fg } else { t.fg_muted };

            // Fill row background
            for x in 0..area.width.min(40) {
                let idx = (y as usize) * area.width as usize + x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].transparent = false;
                }
            }

            let icon = if file.kind == "dir" { "📁" } else { "📄" };
            let line = format!("  {} {:14} {:4} {:6}", icon, file.name, file.kind, file.size);
            draw_text(&mut plane, 1, y, &line, fg, bg, false);
        }

        // ── Info panel (right side) ────────────────────────────────
        let info_x = area.width / 2 + 2;
        draw_text(&mut plane, info_x, 2, "Interaction Patterns", t.primary, t.bg, true);
        draw_text(&mut plane, info_x, 4, "This scene demonstrates three", t.fg, t.bg, false);
        draw_text(&mut plane, info_x, 5, "interaction pattern widgets:", t.fg, t.bg, false);
        draw_text(&mut plane, info_x, 7, "ContextMenu", t.primary, t.bg, true);
        draw_text(&mut plane, info_x, 8, "  Right-click a file to open", t.fg_muted, t.bg, false);
        draw_text(&mut plane, info_x, 9, "  a context menu with actions", t.fg_muted, t.bg, false);
        draw_text(&mut plane, info_x, 11, "ConfirmDialog", t.primary, t.bg, true);
        draw_text(&mut plane, info_x, 12, "  Select Delete from menu", t.fg_muted, t.bg, false);
        draw_text(&mut plane, info_x, 13, "  to see a confirmation dialog", t.fg_muted, t.bg, false);
        draw_text(&mut plane, info_x, 15, "Toast", t.primary, t.bg, true);
        draw_text(&mut plane, info_x, 16, "  Actions produce toast", t.fg_muted, t.bg, false);
        draw_text(&mut plane, info_x, 17, "  notifications (3s auto-dismiss)", t.fg_muted, t.bg, false);

        if let Some(idx) = self.selected {
            if idx < self.files.len() {
                let f = &self.files[idx];
                draw_text(&mut plane, info_x, 20, "Selected File", t.primary, t.bg, true);
                draw_text(&mut plane, info_x, 21, &format!("  {} ({})", f.name, f.kind), t.fg, t.bg, false);
                draw_text(&mut plane, info_x, 22, &format!("  Size: {}", f.size), t.fg_muted, t.bg, false);
            }
        }

        // ── Status bar ─────────────────────────────────────────────
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self.status_bar.borrow().render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        // ── Context menu overlay ────────────────────────────────────
        if self.context_menu.borrow().is_visible() {
            let cm = self.context_menu.borrow();
            let cm_plane = cm.render(area);
            let anchor_x = cm.anchor_x();
            let anchor_y = cm.anchor_y();
            blit_to(&mut plane, &cm_plane, anchor_x as usize, anchor_y as usize);
        }

        // ── Confirm dialog overlay ─────────────────────────────────
        {
            let mut cd = self.confirm_dialog.borrow_mut();
            if let Some(ref mut dialog) = *cd {
                let dw = 40u16.min(area.width.saturating_sub(4));
                let dh = 8u16.min(area.height.saturating_sub(4));
                let dx = (area.width.saturating_sub(dw)) / 2;
                let dy = (area.height.saturating_sub(dh)) / 2;
                dialog.set_area(Rect::new(dx, dy, dw, dh));
                let cd_plane = dialog.render(Rect::new(0, 0, dw, dh));
                blit_to(&mut plane, &cd_plane, dx as usize, dy as usize);
            }
        }

        // ── Toast notifications (bottom-right) ──────────────────────
        let mut toast_y = area.height.saturating_sub(3);
        for toast in self.toasts.iter().rev().take(3) {
            let msg = toast.message();
            let tw = (msg.len() as u16 + 4).min(area.width.saturating_sub(4));
            let tx = area.width.saturating_sub(tw + 2);
            // Toast background
            for y in toast_y..toast_y + 1 {
                for x in tx..tx + tw {
                    let idx = (y as usize) * area.width as usize + x as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
            // Toast text
            draw_text(&mut plane, tx + 2, toast_y, msg, t.fg, t.surface_elevated, false);
            toast_y = toast_y.saturating_sub(1);
        }

        // ── Help overlay ───────────────────────────────────────────
        if self.show_help {
            render_help_overlay(&mut plane, area, t, "Action Center — Help", &[("Up/Dn", "Navigate file list"), ("Right-click", "Open context menu"), ("Del", "Delete selected file"), ("N", "New file (placeholder)"), ("Click file", "Select file"), ("Click menu", "Execute action"), ("F1", "Toggle this help"), ("Esc", "Back to showcase")]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        // Confirm dialog takes priority
        if self.confirm_dialog.borrow().is_some() {
            let result = {
                let mut cd = self.confirm_dialog.borrow_mut();
                if let Some(ref mut dialog) = *cd {
                    dialog.handle_key(key);
                    dialog.confirmed()
                } else { None }
            };
            if let Some(result) = result {
                match result {
                    ConfirmResult::Confirmed => {
                        if let Some(idx) = self.selected.take() {
                            if idx < self.files.len() {
                                let name = self.files[idx].name.clone();
                                self.files.remove(idx);
                                self.add_toast(ToastKind::Success, &format!("Deleted {}", name));
                                if !self.files.is_empty() {
                                    self.selected = Some(idx.min(self.files.len() - 1));
                                }
                            }
                        }
                    }
                    ConfirmResult::Cancelled => {
                        self.add_toast(ToastKind::Info, "Delete cancelled");
                    }
                }
                *self.confirm_dialog.borrow_mut() = None;
                self.dirty = true;
            }
            return true;
        }

        // Context menu takes priority
        if self.context_menu.borrow().is_visible() {
            let _handled = self.context_menu.borrow_mut().handle_key(key);
            self.sync_action_bridge();
            // Also handle Enter key for context menu actions
            if key.code == KeyCode::Enter {
                let id = self.context_menu.borrow().selected_id().map(|s| s.to_string());
                if let Some(id) = id {
                    self.execute_context_action(&id);
                }
            }
            if self.keybindings.matches(actions::BACK, &key) {
                self.hide_context_menu();
            }
            self.dirty = true;
            return true;
        }

        if self.show_help {
            if self.keybindings.matches(actions::HELP, &key) || self.keybindings.matches(actions::BACK, &key) {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        match key.code {
            KeyCode::Up => {
                if let Some(idx) = self.selected {
                    self.selected = Some(idx.saturating_sub(1));
                } else if !self.files.is_empty() {
                    self.selected = Some(0);
                }
                self.dirty = true;
                true
            }
            KeyCode::Down => {
                if let Some(idx) = self.selected {
                    self.selected = Some((idx + 1).min(self.files.len() - 1));
                } else if !self.files.is_empty() {
                    self.selected = Some(0);
                }
                self.dirty = true;
                true
            }
            KeyCode::Delete => {
                if self.selected.is_some() {
                    self.execute_context_action("delete");
                }
                true
            }
            KeyCode::Char('n') if key.modifiers.is_empty() => {
                self.add_toast(ToastKind::Info, "New file placeholder");
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Confirm dialog
        {
            let mut cd = self.confirm_dialog.borrow_mut();
            if let Some(ref mut dialog) = *cd {
                return dialog.handle_mouse(kind, col, row);
            }
        }

        // Context menu
        if self.context_menu.borrow().is_visible() {
            let handled = self.context_menu.borrow_mut().handle_mouse(kind, col, row);
            if !handled {
                // Click outside — dismiss
                self.hide_context_menu();
            }
            self.sync_action_bridge();
            self.dirty = true;
            return true;
        }

        // Right-click → context menu
        if let MouseEventKind::Down(MouseButton::Right) = kind {
            self.show_context_menu(col, row);
            return true;
        }

        // File list hover/click
        let list_y = 2u16;
        let list_end = list_y + 2 + self.files.len() as u16;
        if row >= list_y + 2 && row < list_end && col < area_width_hint() {
            let idx = (row - list_y - 2) as usize;
            if idx < self.files.len() {
                match kind {
                    MouseEventKind::Moved => {
                        if self.hovered != Some(idx) {
                            self.hovered = Some(idx);
                            self.dirty = true;
                        }
                        true
                    }
                    MouseEventKind::Down(MouseButton::Left) => {
                        self.selected = Some(idx);
                        self.dirty = true;
                        true
                    }
                    _ => false,
                }
            } else {
                false
            }
        } else if let MouseEventKind::Moved = kind {
            if self.hovered.is_some() {
                self.hovered = None;
                self.dirty = true;
            }
            false
        } else {
            false
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.context_menu.borrow_mut().on_theme_change(theme);
        if let Some(ref mut cd) = *self.confirm_dialog.borrow_mut() {
            cd.on_theme_change(theme);
        }
        self.status_bar.borrow_mut().on_theme_change(theme);
        self.dirty = true;
    }

    fn scene_id(&self) -> &str { "action_center" }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

fn area_width_hint() -> u16 { 40 }


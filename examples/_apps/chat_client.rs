#![allow(missing_docs)]
//! Chat Client — rich chat UI demo using List, TextInput, Toast, Modal, and StatusBar.
//!
//! Features: Custom message rendering, unread highlighting, emoji picker, settings modal,
//! status bar, auto-scroll, and toast notifications.
//!
//! Pattern: InputRouter (Pattern 2) — all rendering in `on_tick`, input via Widget trait.

use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dracon_terminal_engine::compositor::{Cell, Plane, Styles};
use dracon_terminal_engine::framework::hitzone::ScopedZoneRegistry;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Modal, Toast, ToastKind};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

#[derive(Clone)]
struct Message {
    sender: String,
    text: String,
    time: String,
    is_read: bool,
}

impl Message {
    fn new(sender: &str, text: &str, time: &str, is_read: bool) -> Self {
        Self {
            sender: sender.to_string(),
            text: text.to_string(),
            time: time.to_string(),
            is_read,
        }
    }
}

struct ChatState {
    messages: Vec<Message>,
    input_text: String,
    cursor_pos: usize,
    show_emoji_modal: bool,
    show_settings_modal: bool,
    show_help: bool,
    emoji_modal: Modal<'static>,
    settings_modal: Modal<'static>,
    notifications_enabled: bool,
    show_toast: bool,
    toast_message: String,
    scroll_offset: usize,
    area: std::cell::Cell<Rect>,
    should_quit: Arc<AtomicBool>,
    theme: Theme,
    dirty: bool,
    zones: RefCell<ScopedZoneRegistry<usize>>,
}

// Zone IDs for mouse dispatch
const ZONE_EMOJI_BTN: usize = 1;
const ZONE_SEND_BTN: usize = 2;
const ZONE_SETTINGS_BTN: usize = 3;

impl ChatState {
    fn new(should_quit: Arc<AtomicBool>, theme: Theme) -> Self {
        let now = chrono_lite_timestamp();
        Self {
            messages: vec![
                Message::new("Alice", "Hey, how's the project going?", &now, true),
                Message::new("Bob", "Going well! Just finished the new widget.", &now, true),
                Message::new("Alice", "Nice! Can you send me the code?", &now, true),
                Message::new("Bob", "Sure, I'll share it after review.", &now, true),
                Message::new("Alice", "Perfect, thanks!", &now, false),
            ],
            input_text: String::new(),
            cursor_pos: 0,
            show_emoji_modal: false,
            show_settings_modal: false,
            show_help: false,
            emoji_modal: Modal::new("Emoji Picker")
                .with_size(30, 10)
                .with_buttons(vec![("Close", ModalResult::Cancel)]),
            settings_modal: Modal::new("Settings")
                .with_size(35, 10)
                .with_buttons(vec![("Done", ModalResult::Confirm)]),
            notifications_enabled: true,
            show_toast: false,
            toast_message: String::new(),
            scroll_offset: 0,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            should_quit,
            theme,
            dirty: true,
            zones: RefCell::new(ScopedZoneRegistry::new()),
        }
    }

    fn scroll_to_bottom(&mut self) {
        let visible = (self.area.get().height.saturating_sub(5)).max(1) as usize;
        self.scroll_offset = self.messages.len().saturating_sub(visible);
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::nord(),
            Theme::cyberpunk(),
            Theme::dracula(),
            Theme::gruvbox_dark(),
            Theme::tokyo_night(),
            Theme::solarized_dark(),
            Theme::solarized_light(),
            Theme::monokai(),
            Theme::one_dark(),
            Theme::catppuccin_mocha(),
            Theme::rose_pine(),
            Theme::kanagawa(),
            Theme::everforest(),
            Theme::light(),
            Theme::warm(),
            Theme::cool(),
            Theme::forest(),
            Theme::sunset(),
            Theme::mono(),
        ];
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];
        self.dirty = true;
    }

    fn unread_count(&self) -> usize {
        self.messages.iter().filter(|m| !m.is_read).count()
    }

    fn send_message(&mut self) {
        if self.input_text.trim().is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.input_text);
        self.messages.push(Message::new("You", &text, "Now", true));
        self.cursor_pos = 0;
        self.scroll_to_bottom();
        self.show_toast = true;
        self.toast_message = "Message sent!".to_string();
        self.dirty = true;
    }

    // ── Input Handling ────────────────────────────────────────────────────────

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Modal capture: when any modal is open, only Esc closes it
        if self.show_emoji_modal || self.show_settings_modal {
            match key.code {
                KeyCode::Esc => {
                    self.show_emoji_modal = false;
                    self.show_settings_modal = false;
                    self.dirty = true;
                    return true;
                }
                _ => return true, // Capture everything else
            }
        }

        // Help overlay: Esc or ? dismisses
        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => {
                    self.show_help = false;
                    self.dirty = true;
                    return true;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Esc => {
                self.show_help = false;
                self.dirty = true;
                true
            }
            KeyCode::Char('q') => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            KeyCode::Char('t') => {
                self.cycle_theme();
                true
            }
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                self.send_message();
                true
            }
            KeyCode::Backspace if !self.input_text.is_empty() => {
                self.input_text.pop();
                self.cursor_pos = self.input_text.len();
                self.dirty = true;
                true
            }
            KeyCode::Char(ch) => {
                self.input_text.push(ch);
                self.cursor_pos = self.input_text.len();
                self.dirty = true;
                true
            }
            KeyCode::Left if self.cursor_pos > 0 => {
                self.cursor_pos -= 1;
                self.dirty = true;
                true
            }
            KeyCode::Right if self.cursor_pos < self.input_text.len() => {
                self.cursor_pos += 1;
                self.dirty = true;
                true
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                self.dirty = true;
                true
            }
            KeyCode::End => {
                self.cursor_pos = self.input_text.len();
                self.dirty = true;
                true
            }
            KeyCode::Up => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Down => {
                let visible = (self.area.get().height.saturating_sub(5)).max(1) as usize;
                if self.scroll_offset + visible < self.messages.len() {
                    self.scroll_offset += 1;
                    self.dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Modal click dismissal
        if self.show_emoji_modal || self.show_settings_modal {
            if let MouseEventKind::Down(_) = kind {
                self.show_emoji_modal = false;
                self.show_settings_modal = false;
                self.dirty = true;
                return true;
            }
            return false;
        }

        // Dispatch via zones — collect result first, then act
        if let MouseEventKind::Down(MouseButton::Left) = kind {
            let zone = self.zones.borrow().dispatch(col, row);
            match zone {
                Some(ZONE_EMOJI_BTN) => {
                    self.show_emoji_modal = true;
                    self.dirty = true;
                    return true;
                }
                Some(ZONE_SEND_BTN) => {
                    self.send_message();
                    return true;
                }
                Some(ZONE_SETTINGS_BTN) => {
                    self.show_settings_modal = true;
                    self.dirty = true;
                    return true;
                }
                _ => {}
            }
        }

        false
    }

    // ── Rendering ─────────────────────────────────────────────────────────────

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(t.bg);

        let (input_h, status_h, header_h) = (3u16, 1u16, 1u16);
        let list_h = area.height.saturating_sub(input_h + status_h + header_h);
        let input_row = header_h + list_h;

        // ── Header ────────────────────────────────────────────────────────────
        for (i, c) in "Chat Client".chars().enumerate() {
            if i < plane.cells.len() {
                plane.cells[i] = Cell {
                    char: c,
                    fg: t.fg_on_accent,
                    bg: t.primary,
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }

        // Online status
        let status_x = (area.width as usize).saturating_sub(12);
        for (i, c) in "Online".chars().enumerate() {
            let idx = status_x + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.success;
            }
        }

        // Settings button zone
        let settings_x = area.width.saturating_sub(6);
        self.zones.borrow_mut().register(ZONE_SETTINGS_BTN, settings_x, 0, 6, 1);
        for (i, c) in "[⚙]".chars().enumerate() {
            let idx = settings_x as usize + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.secondary;
            }
        }

        // Header divider
        for col in 0..area.width {
            let idx = (header_h * area.width + col) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── Message List ──────────────────────────────────────────────────────
        let visible_count = (list_h as usize).saturating_sub(2).max(1);
        let start = self.scroll_offset;
        let end = (start + visible_count).min(self.messages.len());

        for (i, msg) in self.messages[start..end].iter().enumerate() {
            let row = header_h + 1 + i as u16;
            let base_idx = (row * area.width) as usize;
            let is_me = msg.sender == "You";
            let bg = if !msg.is_read { t.primary_active } else { t.surface };

            // Avatar (first letter of sender)
            let avatar = msg.sender.chars().next().unwrap_or('?');
            let avatar_color = match msg.sender.as_str() {
                "Alice" => t.secondary,
                "Bob" => t.info,
                "You" => t.success,
                _ => t.fg_muted,
            };
            let avatar_x = if is_me { area.width as usize - 4 } else { 1 };
            if base_idx + avatar_x < plane.cells.len() {
                plane.cells[base_idx + avatar_x] = Cell {
                    char: avatar,
                    fg: t.fg_on_accent,
                    bg: avatar_color,
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }

            // Message bubble
            let bubble_start = if is_me { 6usize } else { 4usize };
            let bubble_end = if is_me {
                area.width as usize - 5
            } else {
                area.width as usize - 3
            };
            for col in bubble_start..bubble_end {
                let idx = base_idx + col;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = bg;
                }
            }

            // Sender name
            let sender_color = match msg.sender.as_str() {
                "Alice" => t.secondary,
                "Bob" => t.info,
                "You" => t.success,
                _ => t.fg_muted,
            };
            let sender_x = if is_me {
                bubble_end - msg.sender.len() - 1
            } else {
                bubble_start + 1
            };
            for (j, c) in msg.sender.chars().enumerate() {
                let idx = base_idx + sender_x + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = sender_color;
                    plane.cells[idx].style = Styles::BOLD;
                    plane.cells[idx].bg = bg;
                }
            }

            // Message text
            let text_start = if is_me {
                sender_x
            } else {
                bubble_start + msg.sender.len() + 3
            };
            let text_limit = if is_me {
                text_start.saturating_sub(2)
            } else {
                bubble_end.saturating_sub(8)
            };
            let text_len = text_limit.saturating_sub(text_start);
            for (j, c) in msg.text.chars().take(text_len).enumerate() {
                let idx = if is_me {
                    base_idx + text_start.saturating_sub(msg.text.len().min(text_len)) + j
                } else {
                    base_idx + text_start + j
                };
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if !msg.is_read { t.fg } else { t.fg_muted };
                    plane.cells[idx].bg = bg;
                }
            }

            // Timestamp
            let time_x = if is_me {
                1usize
            } else {
                (area.width as usize).saturating_sub(6)
            };
            for (j, c) in msg.time.chars().enumerate() {
                let idx = base_idx + time_x + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg_subtle;
                    plane.cells[idx].bg = t.bg;
                }
            }
        }

        // List bottom divider
        for col in 0..area.width {
            let idx = ((header_h + list_h.saturating_sub(1)) * area.width + col) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── Input Area ────────────────────────────────────────────────────────
        let input_w = area.width.saturating_sub(2);
        if input_w > 4 {
            let input_y = input_row;
            // Rounded border
            for col in 1..area.width.saturating_sub(1) {
                let idx_top = (input_y * area.width + col) as usize;
                let idx_bottom = ((input_y + 2) * area.width + col) as usize;
                if idx_top < plane.cells.len() {
                    plane.cells[idx_top].char = '─';
                    plane.cells[idx_top].fg = t.outline;
                }
                if idx_bottom < plane.cells.len() {
                    plane.cells[idx_bottom].char = '─';
                    plane.cells[idx_bottom].fg = t.outline;
                }
            }
            let corners = [
                (input_y, 1u16, '╭'),
                (input_y, area.width.saturating_sub(2), '╮'),
                (input_y + 2, 1u16, '╰'),
                (input_y + 2, area.width.saturating_sub(2), '╯'),
            ];
            for (r, c, ch) in corners.iter() {
                let idx = (r * area.width + c) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = *ch;
                    plane.cells[idx].fg = t.primary;
                }
            }
            for row in input_y + 1..input_y + 2 {
                let idx_left = (row * area.width + 1) as usize;
                let idx_right = (row * area.width + area.width.saturating_sub(2)) as usize;
                if idx_left < plane.cells.len() {
                    plane.cells[idx_left].char = '│';
                    plane.cells[idx_left].fg = t.outline;
                }
                if idx_right < plane.cells.len() {
                    plane.cells[idx_right].char = '│';
                    plane.cells[idx_right].fg = t.outline;
                }
            }
        }

        // Emoji button zone
        let base_idx = ((input_row + 1) * area.width + 3) as usize;
        self.zones.borrow_mut().register(ZONE_EMOJI_BTN, 3, input_row + 1, 4, 1);
        for (i, c) in "[📎]".chars().enumerate() {
            let idx = base_idx + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.secondary;
            }
        }

        // Input text
        let display = if self.input_text.is_empty() {
            "Message..."
        } else {
            &self.input_text
        };
        for (j, c) in display
            .chars()
            .take((area.width as usize).saturating_sub(10))
            .enumerate()
        {
            let idx = base_idx + 4 + j;
            if idx < plane.cells.len() {
                let is_cursor = j == self.cursor_pos && !self.input_text.is_empty();
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if is_cursor || self.input_text.is_empty() {
                    t.fg_muted
                } else {
                    t.input_fg
                };
                plane.cells[idx].bg = if is_cursor {
                    t.input_border
                } else {
                    t.input_bg
                };
            }
        }

        // Send button zone
        let send_x = (area.width as usize).saturating_sub(5);
        self.zones.borrow_mut().register(ZONE_SEND_BTN, send_x as u16, input_row + 1, 5, 1);
        for (i, c) in "[➤]".chars().enumerate() {
            let idx = base_idx + send_x + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if i == 1 { t.primary } else { t.secondary };
                plane.cells[idx].style = if i == 1 { Styles::BOLD } else { Styles::empty() };
            }
        }

        // Input area bottom divider
        let divider_row = input_row + 1;
        for col in 0..area.width {
            let idx = (divider_row * area.width + col) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── Status Bar ────────────────────────────────────────────────────────
        let status_base = ((area.height - status_h) * area.width) as usize;

        // Left segment: participants
        for (i, c) in "Alice, Bob".chars().enumerate() {
            let idx = status_base + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.secondary;
                plane.cells[idx].bg = t.surface;
            }
        }

        // Middle segment: unread count
        let seg2 = if self.unread_count() > 0 {
            format!("{} unread", self.unread_count())
        } else {
            "All read".to_string()
        };
        for (i, c) in seg2.chars().enumerate() {
            let idx = status_base + 15 + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if self.unread_count() > 0 { t.warning } else { t.success };
                plane.cells[idx].bg = t.surface;
            }
        }

        // Right segment: hints
        let hint = "t: theme | ?: help | q: quit";
        let hint_x = (area.width as usize).saturating_sub(hint.len() + 2);
        for (i, c) in hint.chars().enumerate() {
            let idx = status_base + hint_x + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_subtle;
                plane.cells[idx].bg = t.surface;
            }
        }

        // ── Scrollbar ─────────────────────────────────────────────────────────
        if self.messages.len() > visible_count {
            let sb_x = area.width - 2;
            let content_h = list_h.saturating_sub(2);
            let thumb_h = (visible_count as f32 / self.messages.len() as f32 * content_h as f32).max(1.0) as u16;
            let thumb_y = (self.scroll_offset as f32
                / self.messages.len().saturating_sub(visible_count).max(1) as f32
                * (content_h - thumb_h) as f32) as u16
                + header_h
                + 1;
            for i in 0..thumb_h {
                let y = thumb_y + i;
                if y > header_h && y < input_row.saturating_sub(1) {
                    let idx = (y * area.width + sb_x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '▐';
                        plane.cells[idx].fg = t.primary;
                    }
                }
            }
        }

        // ── Help Overlay ──────────────────────────────────────────────────────
        if self.show_help {
            self.render_help(&mut plane, area, t);
        }

        plane
    }

    fn render_help(&self, plane: &mut Plane, area: Rect, t: &Theme) {
        let hw = 36u16.min(area.width.saturating_sub(4));
        let hh = 13u16.min(area.height.saturating_sub(4));
        let hx = (area.width - hw) / 2;
        let hy = (area.height - hh) / 2;

        // Background fill
        for y in hy..hy + hh {
            for x in hx..hx + hw {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Rounded border
        let corners = [
            ('╭', hx, hy),
            ('╮', hx + hw - 1, hy),
            ('╰', hx, hy + hh - 1),
            ('╯', hx + hw - 1, hy + hh - 1),
        ];
        for (ch, cx, cy) in corners.iter() {
            let idx = (cy * area.width + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = *ch;
                plane.cells[idx].fg = t.outline;
            }
        }
        for x in hx + 1..hx + hw - 1 {
            let top_idx = (hy * area.width + x) as usize;
            let bot_idx = ((hy + hh - 1) * area.width + x) as usize;
            if top_idx < plane.cells.len() {
                plane.cells[top_idx].char = '─';
                plane.cells[top_idx].fg = t.outline;
            }
            if bot_idx < plane.cells.len() {
                plane.cells[bot_idx].char = '─';
                plane.cells[bot_idx].fg = t.outline;
            }
        }
        for y in hy + 1..hy + hh - 1 {
            let left_idx = (y * area.width + hx) as usize;
            let right_idx = (y * area.width + hx + hw - 1) as usize;
            if left_idx < plane.cells.len() {
                plane.cells[left_idx].char = '│';
                plane.cells[left_idx].fg = t.outline;
            }
            if right_idx < plane.cells.len() {
                plane.cells[right_idx].char = '│';
                plane.cells[right_idx].fg = t.outline;
            }
        }

        // Title
        let title = "Chat Help";
        let tx = hx + (hw - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Shortcuts
        let shortcuts = [
            ("↑/↓", "Scroll messages"),
            ("Enter", "Send message"),
            ("Type", "Compose"),
            ("t", "Cycle theme"),
            ("?", "Toggle help"),
            ("q", "Quit"),
            ("Click", "Emoji / Send / Settings"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            for (j, c) in key.chars().enumerate() {
                let idx = (row * area.width + hx + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                }
            }
            for (j, c) in desc.chars().enumerate() {
                let idx = (row * area.width + hx + 14 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                }
            }
        }
    }

    // ── Render overlays (modals, toasts) ────────────────────────────────────

    fn render_overlays(&mut self, ctx: &mut Ctx, w: u16, h: u16) {
        if self.show_emoji_modal {
            let mut mp = self.emoji_modal.render(Rect::new(0, 0, w, h));
            let emojis = [
                "😀", "😃", "😄", "😁", "😊", "🙂", "🙃", "😍", "🤔", "🤨", "😅", "😂", "🤣",
            ];
            let (sx, sy) = (((w as i32 - 30) / 2) as u16, ((h as i32 - 10) / 2) as u16);
            for (i, e) in emojis.iter().enumerate() {
                let (x, y) = (sx + (i as u16 % 7) * 4, sy + (i as u16 / 7) * 2);
                if y < h && x < w {
                    for (j, c) in e.chars().enumerate() {
                        let idx = (y * w + x + j as u16) as usize;
                        if idx < mp.cells.len() {
                            mp.cells[idx].char = c;
                            mp.cells[idx].fg = self.theme.secondary;
                        }
                    }
                }
            }
            for (j, c) in "Click or ESC".chars().enumerate() {
                let idx = ((sy + 6) * w + sx + 8 + j as u16) as usize;
                if idx < mp.cells.len() {
                    mp.cells[idx].char = c;
                    mp.cells[idx].fg = self.theme.fg_muted;
                }
            }
            ctx.add_plane(mp);
        }

        if self.show_settings_modal {
            let mut mp = self.settings_modal.render(Rect::new(0, 0, w, h));
            let (sx, sy) = (((w as i32 - 35) / 2) as u16, ((h as i32 - 10) / 2) as u16);
            for (i, c) in format!(
                "Notifications: {}",
                if self.notifications_enabled { "ON" } else { "OFF" }
            )
            .chars()
            .enumerate()
            {
                let idx = ((sy + 2) * w + sx + 2 + i as u16) as usize;
                if idx < mp.cells.len() {
                    mp.cells[idx].char = c;
                    mp.cells[idx].fg = if self.notifications_enabled {
                        self.theme.success
                    } else {
                        self.theme.error
                    };
                }
            }
            for (i, c) in format!("Theme: {}", self.theme.name).chars().enumerate() {
                let idx = ((sy + 3) * w + sx + 2 + i as u16) as usize;
                if idx < mp.cells.len() {
                    mp.cells[idx].char = c;
                    mp.cells[idx].fg = self.theme.secondary;
                }
            }
            for (i, c) in "Clear Chat History".chars().enumerate() {
                let idx = ((sy + 5) * w + sx + 8 + i as u16) as usize;
                if idx < mp.cells.len() {
                    mp.cells[idx].char = c;
                    mp.cells[idx].fg = self.theme.error;
                }
            }
            ctx.add_plane(mp);
        }

        if self.show_toast {
            let toast_msg = std::mem::take(&mut self.toast_message);
            ctx.add_plane(
                Toast::new(WidgetId::new(200), &toast_msg)
                    .with_kind(ToastKind::Success)
                    .with_duration(Duration::from_secs(2))
                    .with_theme(self.theme)
                    .render(Rect::new(w.saturating_sub(25).max(1), h.saturating_sub(4), 20, 1)),
            );
            self.show_toast = false;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// INPUT ROUTER
// ═══════════════════════════════════════════════════════════════════════════════

struct ChatInputRouter {
    target: Rc<RefCell<ChatState>>,
    id: WidgetId,
    area: std::cell::Cell<Rect>,
}

impl Widget for ChatInputRouter {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn area(&self) -> Rect {
        self.area.get()
    }
    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
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

// ═══════════════════════════════════════════════════════════════════════════════
// UTILITIES
// ═══════════════════════════════════════════════════════════════════════════════

fn chrono_lite_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, s)
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> io::Result<()> {
    println!("Chat Client Demo - Enter to send | Click 📎 for emojis | Click ⚙ for settings");
    std::thread::sleep(Duration::from_millis(300));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app = App::new()?.title("Chat Client").fps(30);
    let chat = Rc::new(RefCell::new(ChatState::new(should_quit, Theme::default())));
    let chat_for_tick = Rc::clone(&chat);
    let chat_for_input = Rc::clone(&chat);

    let router = ChatInputRouter {
        target: chat_for_input,
        id: WidgetId::new(100),
        area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
    };
    app.add_widget(Box::new(router), Rect::new(0, 0, 80, 24));

    app.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
            return;
        }

        let mut chat = chat_for_tick.borrow_mut();
        let (w, h) = ctx.compositor().size();
        let area = Rect::new(0, 0, w, h);

        // Update area if resized
        if chat.area.get() != area {
            chat.area.set(area);
            chat.dirty = true;
        }

        if chat.dirty {
            let plane = chat.render(area);
            ctx.add_plane(plane);
            chat.dirty = false;
        }

        chat.render_overlays(ctx, w, h);
    })
    .run(|_| {})?;

    println!("\nChat client exited cleanly");
    Ok(())
}

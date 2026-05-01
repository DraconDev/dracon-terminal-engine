//! Chat Client — rich chat UI demo using List, TextInput, Toast, Modal, and StatusBar.
//!
//! Features: Custom message rendering, unread highlighting, emoji picker, settings modal,
//! status bar, auto-scroll, and toast notifications.
//!
//! Layout:
//! ┌─────────────────────────────────────────────────────────┐
//! │ Chat Client                             [👤 Online]  [⚙]│
//! ├─────────────────────────────────────────────────────────┤
//! │ [Alice] Hey, how's the project going?  14:32            │
//! │ [Bob] Going well! Just finished the new widget.        │
//! │ ...                                                    │
//! ├─────────────────────────────────────────────────────────┤
//! │ [📎] [Message input...___________________________] [➤]│
//! ├─────────────────────────────────────────────────────────┤
//! │ [Alice, Bob] | 3 unread | Press Enter to send           │
//! └─────────────────────────────────────────────────────────┘

use std::io;
use std::time::Duration;

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::{Modal, Toast, ToastKind};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind, MouseButton};
use ratatui::layout::Rect;

#[derive(Clone)]
struct Message {
    sender: &'static str,
    text: &'static str,
    time: &'static str,
    is_read: bool,
}

static MESSAGES: &[Message] = &[
    Message { sender: "Alice", text: "Hey, how's the project going?", time: "14:32", is_read: true },
    Message { sender: "Bob", text: "Going well! Just finished the new widget.", time: "14:33", is_read: true },
    Message { sender: "Alice", text: "Nice! Can you send me the code?", time: "14:34", is_read: true },
    Message { sender: "Bob", text: "Sure, I'll share it after review.", time: "14:35", is_read: true },
    Message { sender: "Alice", text: "Perfect, thanks!", time: "14:36", is_read: false },
];

struct ChatState {
    messages: Vec<Message>,
    input_text: String,
    cursor_pos: usize,
    show_emoji_modal: bool,
    show_settings_modal: bool,
    emoji_modal: Modal<'static>,
    settings_modal: Modal<'static>,
    notifications_enabled: bool,
    theme_mode: &'static str,
    show_toast: bool,
    scroll_offset: usize,
}

impl ChatState {
    fn new() -> Self {
        Self {
            messages: MESSAGES.iter().cloned().map(|m| m.clone()).collect(),
            input_text: String::new(),
            cursor_pos: 0,
            show_emoji_modal: false,
            show_settings_modal: false,
            emoji_modal: Modal::new("Emoji Picker").with_size(30, 10).with_buttons(vec![("Close", ModalResult::Cancel)]),
            settings_modal: Modal::new("Settings").with_size(35, 10).with_buttons(vec![("Done", ModalResult::Confirm)]),
            notifications_enabled: true,
            theme_mode: "Dark",
            show_toast: false,
            scroll_offset: 0,
        }
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.messages.len().saturating_sub(6).max(0);
    }

    fn unread_count(&self) -> usize {
        self.messages.iter().filter(|m| !m.is_read).count()
    }

    fn send_message(&mut self) {
        if self.input_text.trim().is_empty() { return; }
        let text = std::mem::take(&mut self.input_text);
        self.messages.push(Message { sender: "You", text: Box::leak(text.into_boxed_str()), time: "Now", is_read: true });
        self.cursor_pos = 0;
        self.scroll_to_bottom();
        self.show_toast = true;
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        match key.code {
            KeyCode::Esc => {
                self.show_emoji_modal = false;
                self.show_settings_modal = false;
                true
            }
            KeyCode::Enter if !self.show_emoji_modal && !self.show_settings_modal => { self.send_message(); true }
            KeyCode::Backspace if !self.input_text.is_empty() => { self.input_text.pop(); self.cursor_pos = self.input_text.len(); true }
            KeyCode::Char(ch) if !self.show_emoji_modal && !self.show_settings_modal => { self.input_text.push(ch); self.cursor_pos = self.input_text.len(); true }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let (h, input_h, status_h, header_h) = (24u16, 3u16, 1u16, 1u16);
        let list_h = h - input_h - status_h - header_h;
        let input_row = header_h + list_h;

        if self.show_emoji_modal || self.show_settings_modal {
            if let MouseEventKind::Down(_) = kind { self.show_emoji_modal = false; self.show_settings_modal = false; return true; }
            return false;
        }

        if let MouseEventKind::Down(btn) = kind {
            if btn == MouseButton::Left {
                if col >= 1 && col <= 3 && row >= input_row && row < input_row + 1 { self.show_emoji_modal = true; return true; }
                if col >= 74 && col <= 77 && row < 1 { self.show_settings_modal = true; return true; }
                if col >= 75 && col <= 78 && row >= input_row && row < input_row + 1 { self.send_message(); return true; }
            }
        }
        false
    }
}

fn render_chat(chat: &ChatState, area: Rect) -> Plane {
    let mut plane = Plane::new(0, area.width, area.height);
    plane.z_index = 10;
    let (input_h, status_h, header_h) = (3u16, 1u16, 1u16);
    let list_h = area.height - input_h - status_h - header_h;

    for cell in &mut plane.cells {
        *cell = Cell { char: ' ', fg: Color::Reset, bg: Color::Ansi(17), style: Styles::empty(), transparent: false, skip: false };
    }

    for (i, c) in "Chat Client".chars().enumerate() {
        if i < plane.cells.len() { plane.cells[i] = Cell { char: c, fg: Color::Rgb(255, 255, 255), style: Styles::BOLD, ..Default::default() }; }
    }

    let status_x = (area.width as usize).saturating_sub(12);
    for (i, c) in "Online".chars().enumerate() {
        let idx = status_x + i;
        if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = Color::Ansi(2); }
    }

    for (i, c) in "[⚙]".chars().enumerate() {
        let idx = ((area.width as usize) - 6) + i;
        if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = Color::Ansi(3); }
    }

    for col in 0..area.width {
        let idx = (header_h * area.width + col) as usize;
        if idx < plane.cells.len() { plane.cells[idx].char = '─'; plane.cells[idx].fg = Color::Ansi(8); }
    }

    let visible_count = (list_h as usize).saturating_sub(2).max(1);
    let start = chat.scroll_offset;
    let end = (start + visible_count).min(chat.messages.len());

    for (i, msg) in chat.messages[start..end].iter().enumerate() {
        let row = header_h + 1 + i as u16;
        let base_idx = (row * area.width) as usize;
        let bg = if !msg.is_read { Color::Ansi(24) } else { Color::Reset };

        for col in 0..area.width {
            let idx = base_idx + col as usize;
            if idx < plane.cells.len() { plane.cells[idx].bg = bg; plane.cells[idx].fg = Color::Reset; }
        }

        let sender_color = match msg.sender {
            "Alice" => Color::Ansi(5), "Bob" => Color::Ansi(6), "You" => Color::Ansi(2), _ => Color::Ansi(3),
        };
        let sender_len = msg.sender.len();

        for (j, c) in msg.sender.chars().enumerate() {
            let idx = base_idx + j;
            if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = sender_color; plane.cells[idx].style = Styles::BOLD; }
        }
        for (j, c) in "] ".chars().enumerate() {
            let idx = base_idx + sender_len + j;
            if idx < plane.cells.len() { plane.cells[idx].char = c; }
        }

        let text_start = sender_len + 2;
        for (j, c) in msg.text.chars().take((area.width as usize).saturating_sub(text_start + 10)).enumerate() {
            let idx = base_idx + text_start + j;
            if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = if !msg.is_read { Color::Ansi(15) } else { Color::Reset }; }
        }

        let time_x = (area.width as usize).saturating_sub(6);
        for (j, c) in msg.time.chars().enumerate() {
            let idx = base_idx + time_x + j;
            if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = Color::Ansi(8); }
        }
    }

    for col in 0..area.width {
        let idx = ((header_h + list_h - 1) * area.width + col) as usize;
        if idx < plane.cells.len() { plane.cells[idx].char = '─'; plane.cells[idx].fg = Color::Ansi(8); }
    }

    let input_row = header_h + list_h;
    let base_idx = (input_row * area.width) as usize;
    for col in 0..area.width {
        let idx = base_idx + col as usize;
        if idx < plane.cells.len() { plane.cells[idx].bg = Color::Ansi(8); }
    }

    for (i, c) in "[📎]".chars().enumerate() {
        let idx = base_idx + i;
        if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = Color::Ansi(6); }
    }

    let display = if chat.input_text.is_empty() { "Message..." } else { &chat.input_text };
    for (j, c) in display.chars().take((area.width as usize).saturating_sub(10)).enumerate() {
        let idx = base_idx + 4 + j;
        if idx < plane.cells.len() {
            let is_cursor = j == chat.cursor_pos && !chat.input_text.is_empty();
            plane.cells[idx].char = c;
            plane.cells[idx].fg = if is_cursor || chat.input_text.is_empty() { Color::Ansi(8) } else { Color::Reset };
            plane.cells[idx].bg = if is_cursor { Color::Reset } else { Color::Ansi(8) };
        }
    }

    let send_x = (area.width as usize).saturating_sub(5);
    for (i, c) in "[➤]".chars().enumerate() {
        let idx = base_idx + send_x + i;
        if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = if i == 1 { Color::Ansi(2) } else { Color::Ansi(6) }; plane.cells[idx].style = if i == 1 { Styles::BOLD } else { Styles::empty() }; }
    }

    for col in 0..area.width {
        let idx = ((input_row + 1) * area.width + col) as usize;
        if idx < plane.cells.len() { plane.cells[idx].char = '─'; plane.cells[idx].fg = Color::Ansi(8); }
    }

    let status_base = ((area.height - status_h) * area.width) as usize;
    for (i, c) in "Alice, Bob".chars().enumerate() {
        let idx = status_base + i;
        if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = Color::Ansi(6); plane.cells[idx].bg = Color::Ansi(17); }
    }

    let seg2 = if chat.unread_count() > 0 { format!("{} unread", chat.unread_count()) } else { "All read".to_string() };
    for (i, c) in seg2.chars().enumerate() {
        let idx = status_base + 15 + i;
        if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = if chat.unread_count() > 0 { Color::Ansi(3) } else { Color::Ansi(2) }; plane.cells[idx].bg = Color::Ansi(17); }
    }

    for (i, c) in "Press Enter to send".chars().enumerate() {
        let idx = status_base + 30 + i;
        if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = Color::Ansi(8); plane.cells[idx].bg = Color::Ansi(17); }
    }

    plane
}

fn main() -> io::Result<()> {
    println!("Chat Client Demo - Enter to send | Click 📎 for emojis | Click ⚙ for settings");
    std::thread::sleep(Duration::from_millis(300));

    let mut app = App::new()?.title("Chat Client").fps(30);
    app.set_theme(Theme::dark());
    let mut chat = ChatState::new();

    let _ = app.run(move |ctx| {
        if ctx.needs_full_refresh() { ctx.mark_all_dirty(); }
        let (w, h) = ctx.compositor().size();
        ctx.add_plane(render_chat(&chat, Rect::new(0, 0, w, h)));

        if chat.show_emoji_modal {
            let mut mp = chat.emoji_modal.render(Rect::new(0, 0, w, h));
            let emojis = ["😀", "😃", "😄", "😁", "😊", "🙂", "🙃", "😍", "🤔", "🤨", "😅", "😂", "🤣"];
            let (sx, sy) = (((w as i32 - 30) / 2) as u16, ((h as i32 - 10) / 2) as u16);
            for (i, e) in emojis.iter().enumerate() {
                let (x, y) = (sx + (i as u16 % 7) * 4, sy + (i as u16 / 7) * 2);
                if y < h && x < w { for (j, c) in e.chars().enumerate() { let idx = (y * w + x + j as u16) as usize; if idx < mp.cells.len() { mp.cells[idx].char = c; mp.cells[idx].fg = Color::Ansi(3); } } }
            }
            for (j, c) in "Click or ESC".chars().enumerate() { let idx = ((sy + 6) * w + sx + 8 + j as u16) as usize; if idx < mp.cells.len() { mp.cells[idx].char = c; mp.cells[idx].fg = Color::Ansi(8); } }
            mp.z_index = 100; ctx.add_plane(mp);
        }

        if chat.show_settings_modal {
            let mut mp = chat.settings_modal.render(Rect::new(0, 0, w, h));
            let (sx, sy) = (((w as i32 - 35) / 2) as u16, ((h as i32 - 10) / 2) as u16);
            for (i, c) in format!("Notifications: {}", if chat.notifications_enabled { "ON" } else { "OFF" }).chars().enumerate() { let idx = ((sy + 2) * w + sx + 2 + i as u16) as usize; if idx < mp.cells.len() { mp.cells[idx].char = c; mp.cells[idx].fg = if chat.notifications_enabled { Color::Ansi(2) } else { Color::Ansi(1) }; } }
            for (i, c) in format!("Theme: {}", chat.theme_mode).chars().enumerate() { let idx = ((sy + 3) * w + sx + 2 + i as u16) as usize; if idx < mp.cells.len() { mp.cells[idx].char = c; mp.cells[idx].fg = Color::Ansi(6); } }
            for (i, c) in "Clear Chat History".chars().enumerate() { let idx = ((sy + 5) * w + sx + 8 + i as u16) as usize; if idx < mp.cells.len() { mp.cells[idx].char = c; mp.cells[idx].fg = Color::Ansi(1); } }
            mp.z_index = 100; ctx.add_plane(mp);
        }

        if chat.show_toast {
            ctx.add_plane(Toast::new(WidgetId::new(200), "Message sent!").with_kind(ToastKind::Success).with_duration(Duration::from_secs(2)).with_theme(Theme::dark()).render(Rect::new(30, h - 4, 20, 1)));
            chat.show_toast = false;
        }
    });

    println!("\nChat client exited cleanly");
    Ok(())
}
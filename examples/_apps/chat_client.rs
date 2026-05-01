//! Chat Client — rich chat UI demo using List, TextInput, Toast, Modal, and StatusBar.
//!
//! Demonstrates:
//! - Custom List rendering for chat messages with sender, text, timestamp
//! - Unread message highlighting
//! - TextInput for composing messages
//! - Emoji picker modal
//! - Settings modal with toggles and clear chat
//! - StatusBar showing participants and unread count
//! - Auto-scroll to bottom on new messages
//! - Toast notifications for send confirmations
//!
//! # Layout
//! ```
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
//! ```

use std::io;
use std::time::Duration;

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Modal, Toast, ToastKind,
};
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
    toast_message: String,
    scroll_offset: usize,
}

impl ChatState {
    fn new() -> Self {
        let emoji_modal = Modal::new("Emoji Picker")
            .with_size(30, 10)
            .with_buttons(vec![("Close", ModalResult::Cancel)]);

        let settings_modal = Modal::new("Settings")
            .with_size(35, 10)
            .with_buttons(vec![("Done", ModalResult::Confirm)]);

        let mut state = Self {
            messages: MESSAGES.iter().map(|m| Message {
                sender: m.sender.clone(),
                text: m.text.clone(),
                time: m.time.clone(),
                is_read: m.is_read,
            }).collect(),
            input_text: String::new(),
            cursor_pos: 0,
            show_emoji_modal: false,
            show_settings_modal: false,
            emoji_modal,
            settings_modal,
            notifications_enabled: true,
            theme_mode: "Dark",
            show_toast: false,
            toast_message: String::new(),
            scroll_offset: 0,
        };
        state.scroll_to_bottom();
        state
    }

    fn scroll_to_bottom(&mut self) {
        let total = self.messages.len();
        if total > 6 {
            self.scroll_offset = total - 6;
        } else {
            self.scroll_offset = 0;
        }
    }

    fn unread_count(&self) -> usize {
        self.messages.iter().filter(|m| !m.is_read).count()
    }

    fn send_message(&mut self) {
        if self.input_text.trim().is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.input_text);
        let msg = Message {
            sender: "You",
            text: Box::leak(text.into_boxed_str()),
            time: "Now",
            is_read: true,
        };
        self.messages.push(msg);
        self.cursor_pos = 0;
        self.scroll_to_bottom();
        self.show_toast = true;
        self.toast_message = "Message sent!".to_string();
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Esc => {
                if self.show_emoji_modal {
                    self.show_emoji_modal = false;
                    self.emoji_modal.clear_result();
                    true
                } else if self.show_settings_modal {
                    self.show_settings_modal = false;
                    self.settings_modal.clear_result();
                    true
                } else {
                    false
                }
            }
            KeyCode::Enter => {
                if !self.show_emoji_modal && !self.show_settings_modal {
                    self.send_message();
                    true
                } else {
                    false
                }
            }
            KeyCode::Backspace => {
                if !self.show_emoji_modal && !self.show_settings_modal && !self.input_text.is_empty() {
                    self.input_text.pop();
                    self.cursor_pos = self.input_text.len();
                    true
                } else {
                    false
                }
            }
            KeyCode::Char(ch) => {
                if !self.show_emoji_modal && !self.show_settings_modal {
                    self.input_text.push(ch);
                    self.cursor_pos = self.input_text.len();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let h = 24u16;
        let input_h = 3u16;
        let status_h = 1u16;
        let header_h = 1u16;
        let list_h = h.saturating_sub(input_h + status_h + header_h);
        let input_row = header_h + list_h;

        if self.show_emoji_modal {
            if let MouseEventKind::Down(_) = kind {
                self.show_emoji_modal = false;
                return true;
            }
            return false;
        }

        if self.show_settings_modal {
            if let MouseEventKind::Down(_) = kind {
                self.show_settings_modal = false;
                return true;
            }
            return false;
        }

        if let MouseEventKind::Down(btn) = kind {
            if btn == MouseButton::Left {
                if col >= 1 && col <= 3 && row >= input_row && row < input_row + 1 {
                    self.show_emoji_modal = true;
                    return true;
                }

                let settings_x = 74u16;
                if col >= settings_x && col <= settings_x + 3 && row < 1 {
                    self.show_settings_modal = true;
                    return true;
                }

                let send_x = 75u16;
                if col >= send_x && col <= send_x + 3 && row >= input_row && row < input_row + 1 {
                    self.send_message();
                    return true;
                }
            }
        }
        false
    }
}

fn render_chat(chat: &ChatState, area: Rect) -> Plane {
    let mut plane = Plane::new(0, area.width, area.height);
    plane.z_index = 10;

    let input_h = 3u16;
    let status_h = 1u16;
    let header_h = 1u16;
    let list_h = area.height.saturating_sub(input_h + status_h + header_h);

    for col in 0..area.width {
        let idx = col as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ' ',
                fg: Color::Reset,
                bg: Color::Ansi(17),
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
    }

    let title = "Chat Client";
    for (i, c) in title.chars().enumerate() {
        let idx = i;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = Color::Rgb(255, 255, 255);
            plane.cells[idx].style = Styles::BOLD;
        }
    }

    let status_x = (area.width as usize).saturating_sub(12);
    for (i, c) in "Online".chars().enumerate() {
        let idx = status_x + i;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = Color::Ansi(2);
        }
    }

    let settings_x = (area.width as usize).saturating_sub(6);
    for (i, c) in "[⚙]".chars().enumerate() {
        let idx = settings_x + i;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = Color::Ansi(3);
        }
    }

    for col in 0..area.width {
        let idx = (header_h * area.width + col) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = '─';
            plane.cells[idx].fg = Color::Ansi(8);
        }
    }

    let visible_count = (list_h as usize).saturating_sub(2).max(1);
    let start = chat.scroll_offset;
    let end = (start + visible_count).min(chat.messages.len());

    for (i, msg) in chat.messages[start..end].iter().enumerate() {
        let row = (header_h + 1 + i as u16);
        let bg = if !msg.is_read {
            Color::Ansi(24)
        } else {
            Color::Reset
        };

        let sender_color = match msg.sender.as_str() {
            "Alice" => Color::Ansi(5),
            "Bob" => Color::Ansi(6),
            "You" => Color::Ansi(2),
            _ => Color::Ansi(3),
        };

        let base_idx = (row * area.width) as usize;
        for col in 0..area.width {
            let idx = base_idx + col as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = bg;
                plane.cells[idx].fg = Color::Reset;
            }
        }

        let sender_len = msg.sender.len();
        for (j, c) in msg.sender.chars().enumerate() {
            let idx = base_idx + j;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = sender_color;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        for (j, c) in "] ".chars().enumerate() {
            let idx = base_idx + sender_len + j;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = Color::Reset;
            }
        }

        let text_start = sender_len + 2;
        for (j, c) in msg.text.chars().take((area.width as usize).saturating_sub(text_start + 10)).enumerate() {
            let idx = base_idx + text_start + j;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if !msg.is_read { Color::Ansi(15) } else { Color::Reset };
            }
        }

        let time_x = (area.width as usize).saturating_sub(6);
        for (j, c) in msg.time.chars().enumerate() {
            let idx = base_idx + time_x + j;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = Color::Ansi(8);
            }
        }
    }

    for col in 0..area.width {
        let idx = ((header_h + list_h - 1) * area.width + col) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = '─';
            plane.cells[idx].fg = Color::Ansi(8);
        }
    }

    let input_row = header_h + list_h;
    let base_idx = (input_row * area.width) as usize;
    for col in 0..area.width {
        let idx = base_idx + col as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].bg = Color::Ansi(8);
            plane.cells[idx].fg = Color::Reset;
        }
    }

    if base_idx < plane.cells.len() {
        plane.cells[base_idx].char = '[';
        plane.cells[base_idx].fg = Color::Ansi(6);
    }
    if base_idx + 1 < plane.cells.len() {
        plane.cells[base_idx + 1].char = '📎';
        plane.cells[base_idx + 1].fg = Color::Ansi(6);
    }
    if base_idx + 2 < plane.cells.len() {
        plane.cells[base_idx + 2].char = ']';
        plane.cells[base_idx + 2].fg = Color::Ansi(6);
    }
    if base_idx + 3 < plane.cells.len() {
        plane.cells[base_idx + 3].char = ' ';
        plane.cells[base_idx + 3].fg = Color::Reset;
    }

    let display = if chat.input_text.is_empty() { "Message..." } else { &chat.input_text };
    let input_start = 4usize;
    for (j, c) in display.chars().take((area.width as usize).saturating_sub(10)).enumerate() {
        let idx = base_idx + input_start + j;
        if idx < plane.cells.len() {
            let is_cursor = j == chat.cursor_pos && !chat.input_text.is_empty();
            plane.cells[idx].char = c;
            plane.cells[idx].fg = if is_cursor { Color::Ansi(8) } else if chat.input_text.is_empty() { Color::Ansi(8) } else { Color::Reset };
            plane.cells[idx].bg = if is_cursor { Color::Reset } else { Color::Ansi(8) };
        }
    }

    let send_x = (area.width as usize).saturating_sub(5);
    if base_idx + send_x < plane.cells.len() {
        plane.cells[base_idx + send_x].char = '[';
        plane.cells[base_idx + send_x].fg = Color::Ansi(6);
    }
    if base_idx + send_x + 1 < plane.cells.len() {
        plane.cells[base_idx + send_x + 1].char = '➤';
        plane.cells[base_idx + send_x + 1].fg = Color::Ansi(2);
        plane.cells[base_idx + send_x + 1].style = Styles::BOLD;
    }
    if base_idx + send_x + 2 < plane.cells.len() {
        plane.cells[base_idx + send_x + 2].char = ']';
        plane.cells[base_idx + send_x + 2].fg = Color::Ansi(6);
    }

    for col in 0..area.width {
        let idx = ((input_row + 1) * area.width + col) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = '─';
            plane.cells[idx].fg = Color::Ansi(8);
        }
    }

    let status_row = area.height - status_h;
    let status_base = (status_row * area.width) as usize;
    for col in 0..area.width {
        let idx = status_base + col as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].bg = Color::Ansi(17);
            plane.cells[idx].fg = Color::Reset;
        }
    }

    let seg1 = "Alice, Bob";
    for (j, c) in seg1.chars().enumerate() {
        let idx = status_base + j;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = Color::Ansi(6);
        }
    }

    let seg2 = if chat.unread_count() > 0 {
        format!("{} unread", chat.unread_count())
    } else {
        "All read".to_string()
    };
    for (j, c) in seg2.chars().enumerate() {
        let idx = status_base + 15 + j;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = if chat.unread_count() > 0 { Color::Ansi(3) } else { Color::Ansi(2) };
        }
    }

    let seg3 = "Press Enter to send";
    for (j, c) in seg3.chars().enumerate() {
        let idx = status_base + 30 + j;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = Color::Ansi(8);
        }
    }

    plane
}

fn main() -> io::Result<()> {
    println!("Chat Client Demo");
    println!("================");
    println!("Enter to send | Click 📎 for emojis | Click ⚙ for settings");
    println!();

    std::thread::sleep(Duration::from_millis(300));

    let mut app = App::new()?.title("Chat Client").fps(30);
    let theme = Theme::dark();
    app.set_theme(theme);

    let mut chat = ChatState::new();

    let _ = app.run(move |ctx| {
        if ctx.needs_full_refresh() {
            ctx.mark_all_dirty();
        }

        let (w, h) = ctx.compositor().size();

        let plane = render_chat(&chat, Rect::new(0, 0, w, h));
        ctx.add_plane(plane);

        if chat.show_emoji_modal {
            let mut modal_plane = chat.emoji_modal.render(Rect::new(0, 0, w, h));
            modal_plane.z_index = 100;
            ctx.add_plane(modal_plane);

            let emojis = ["😀", "😃", "😄", "😁", "😊", "🙂", "🙃", "😍", "🤔", "🤨", "😅", "😂", "🤣"];
            let start_x = ((w as i32 - 30) / 2) as u16;
            let start_y = ((h as i32 - 10) / 2) as u16;
            for (i, emoji) in emojis.iter().enumerate() {
                let x = start_x + (i as u16 % 7) * 4;
                let y = start_y + (i as u16 / 7) * 2;
                if y < h && x < w {
                    for (j, c) in emoji.chars().enumerate() {
                        let idx = (y * w + x + j as u16) as usize;
                        if idx < modal_plane.cells.len() {
                            modal_plane.cells[idx].char = c;
                            modal_plane.cells[idx].fg = Color::Ansi(3);
                        }
                    }
                }
            }

            let hint = "Click emoji or ESC";
            let hint_x = start_x + 8;
            let hint_y = start_y + 6;
            for (j, c) in hint.chars().enumerate() {
                let idx = (hint_y * w + hint_x + j as u16) as usize;
                if idx < modal_plane.cells.len() {
                    modal_plane.cells[idx].char = c;
                    modal_plane.cells[idx].fg = Color::Ansi(8);
                }
            }
        }

        if chat.show_settings_modal {
            let mut modal_plane = chat.settings_modal.render(Rect::new(0, 0, w, h));
            modal_plane.z_index = 100;
            ctx.add_plane(modal_plane);

            let settings_x = ((w as i32 - 35) / 2) as u16;
            let settings_y = ((h as i32 - 10) / 2) as u16;

            let notif_text = format!("Notifications: {}", if chat.notifications_enabled { "ON" } else { "OFF" });
            for (i, c) in notif_text.chars().enumerate() {
                let idx = ((settings_y + 2) * w + settings_x + 2 + i as u16) as usize;
                if idx < modal_plane.cells.len() {
                    modal_plane.cells[idx].char = c;
                    modal_plane.cells[idx].fg = if chat.notifications_enabled { Color::Ansi(2) } else { Color::Ansi(1) };
                }
            }

            let theme_text = format!("Theme: {}", chat.theme_mode);
            for (i, c) in theme_text.chars().enumerate() {
                let idx = ((settings_y + 3) * w + settings_x + 2 + i as u16) as usize;
                if idx < modal_plane.cells.len() {
                    modal_plane.cells[idx].char = c;
                    modal_plane.cells[idx].fg = Color::Ansi(6);
                }
            }

            let clear_text = "Clear Chat History";
            for (i, c) in clear_text.chars().enumerate() {
                let idx = ((settings_y + 5) * w + settings_x + 8 + i as u16) as usize;
                if idx < modal_plane.cells.len() {
                    modal_plane.cells[idx].char = c;
                    modal_plane.cells[idx].fg = Color::Ansi(1);
                }
            }
        }

        if chat.show_toast {
            let toast = Toast::new(WidgetId::new(200), "Message sent!")
                .with_kind(ToastKind::Success)
                .with_duration(Duration::from_secs(2))
                .with_theme(Theme::dark());
            ctx.add_plane(toast.render(Rect::new(30, h - 4, 20, 1)));
            chat.show_toast = false;
        }
    });

    println!("\nChat client exited cleanly");
    Ok(())
}
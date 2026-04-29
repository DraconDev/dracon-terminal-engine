//! Chat demo — shows Input, Editor, List, theme, and App event loop.
//!
//! A simple chat interface with message list, input bar, and theme.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widgets::List;
use ratatui::layout::Rect;

struct Message {
    sender: String,
    text: String,
    timestamp: String,
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Message {
            sender: self.sender.clone(),
            text: self.text.clone(),
            timestamp: self.timestamp.clone(),
        }
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        format!("[{}] {}: {}", self.timestamp, self.sender, self.text)
    }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();

    let now = chrono_lite_timestamp();
    let messages = vec![
        Message {
            sender: "Alice".to_string(),
            text: "Hey, has anyone tried the new framework yet?".to_string(),
            timestamp: now.clone(),
        },
        Message {
            sender: "Bob".to_string(),
            text: "Yeah! It's pretty slick. One import and you're building apps.".to_string(),
            timestamp: now.clone(),
        },
        Message {
            sender: "Carol".to_string(),
            text: "The hit zone system is really nice. No more manual rect checking.".to_string(),
            timestamp: now.clone(),
        },
        Message {
            sender: "Dave".to_string(),
            text: "Just built a file manager with Breadcrumbs + List + SplitPane. Took 20 minutes.".to_string(),
            timestamp: now.clone(),
        },
    ];

    let input_text = String::from("");
    let chat_history = messages.clone();

    App::new()?
        .title("Framework Chat")
        .fps(30)
        .theme(theme)
        .run(move |ctx| {
            let (w, h) = ctx.compositor().size();

            let input_height = 3u16;
            let list_height = h.saturating_sub(input_height);

            let list_rect = Rect::new(0, 0, w, list_height);
            let _input_rect = Rect::new(0, list_height, w, input_height);

            let mut list = List::new(chat_history.clone());
            list.set_visible_count((list_rect.height as usize).saturating_sub(2).max(1));
            let list_plane = list.render(list_rect);
            ctx.add_plane(list_plane);

            let mut input_plane = Plane::new(1, w, input_height);
            input_plane.z_index = 10;

            let placeholder = "Type a message... (Enter to send)";
            let display_text = if input_text.is_empty() {
                placeholder
            } else {
                &input_text
            };

            let prompt = "> ";
            let mut x = 1u16;
            for ch in prompt.chars() {
                let idx = x as usize;
                if idx < input_plane.cells.len() {
                    input_plane.cells[idx].char = ch;
                    input_plane.cells[idx].fg = Color::Rgb(0, 255, 136);
                }
                x += 1;
            }

            let mut text_color = Color::Rgb(200, 200, 200);
            if input_text.is_empty() {
                text_color = Color::Rgb(100, 100, 100);
            }
            for (i, ch) in display_text.chars().take(w as usize - 3).enumerate() {
                let idx = x as usize + i;
                if idx < input_plane.cells.len() {
                    input_plane.cells[idx].char = ch;
                    input_plane.cells[idx].fg = text_color;
                    input_plane.cells[idx].transparent = false;
                }
            }

            let border_y = list_height;
            for col in 0..w {
                let idx = (border_y * w + col) as usize;
                if idx < input_plane.cells.len() {
                    input_plane.cells[idx].char = '─';
                    input_plane.cells[idx].fg = Color::Rgb(60, 60, 80);
                }
            }

            ctx.add_plane(input_plane);
        })
}

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
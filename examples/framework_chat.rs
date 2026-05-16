#![allow(missing_docs)]
//! Chat demo  -  shows Input, Editor, List, theme, and App event loop.
//!
//! A simple chat interface with message list, input bar, and theme.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::List;
use ratatui::layout::Rect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
struct Message {
    sender: String,
    text: String,
    timestamp: String,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.timestamp, self.sender, self.text)
    }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::from_env_or(Theme::cyberpunk());

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
            text: "Just built a file manager with Breadcrumbs + List + SplitPane. Took 20 minutes."
                .to_string(),
            timestamp: now.clone(),
        },
    ];

    let input_text = String::from("");
    let chat_history = messages.clone();

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);
    let show_help = Arc::new(AtomicBool::new(false));
    let show_help_input = Arc::clone(&show_help);
    let show_help_render = Arc::clone(&show_help);

    App::new()?
        .title("Framework Chat")
        .fps(30)
        .theme(theme)
        .on_input(move |key| {
            if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) && key.kind == KeyEventKind::Press {
                should_quit.store(true, Ordering::SeqCst);
                return true;
            }
            if key.kind != KeyEventKind::Press { return false; }
            match key.code {
                KeyCode::F(1) | KeyCode::Char('?') => {
                    show_help_input.store(!show_help_input.load(Ordering::SeqCst), Ordering::SeqCst);
                    true
                }
                KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    true
                }
                KeyCode::Esc => {
                    if show_help_input.load(Ordering::SeqCst) {
                        show_help_input.store(false, Ordering::SeqCst);
                        true
                    } else { false }
                }
                _ => false,
            }
        })
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
            }
        })
        .run(move |ctx| {
            let (w, h) = ctx.compositor().size();
            let theme = ctx.theme().clone();

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
                    input_plane.cells[idx].fg = theme.primary;
                }
                x += 1;
            }

            let mut text_color = theme.fg;
            if input_text.is_empty() {
                text_color = theme.fg_muted;
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
                    input_plane.cells[idx].fg = theme.outline;
                }
            }

            if show_help_render.load(Ordering::SeqCst) {
                let hw = 40u16.min(w.saturating_sub(4));
                let hh = 10u16.min(h.saturating_sub(4));
                let _hx = (w - hw) / 2;
                let _hy = (h - hh) / 2;

                let mut help_plane = Plane::new(100, hw, hh);
                help_plane.z_index = 50;
                for cell in help_plane.cells.iter_mut() {
                    cell.bg = theme.surface_elevated;
                    cell.transparent = false;
                }

                for x in 1..hw - 1 {
                    let top = x as usize;
                    let bot = ((hh - 1) * hw + x) as usize;
                    if top < help_plane.cells.len() { help_plane.cells[top].char = '─'; help_plane.cells[top].fg = theme.outline; }
                    if bot < help_plane.cells.len() { help_plane.cells[bot].char = '─'; help_plane.cells[bot].fg = theme.outline; }
                }
                for y in 1..hh - 1 {
                    let left = (y * hw) as usize;
                    let right = (y * hw + hw - 1) as usize;
                    if left < help_plane.cells.len() { help_plane.cells[left].char = '│'; help_plane.cells[left].fg = theme.outline; }
                    if right < help_plane.cells.len() { help_plane.cells[right].char = '│'; help_plane.cells[right].fg = theme.outline; }
                }
                let corners = [('╭', 0, 0), ('╮', hw - 1, 0), ('╰', 0, hh - 1), ('╯', hw - 1, hh - 1)];
                for (ch, cx, cy) in corners {
                    let idx = (cy * hw + cx) as usize;
                    if idx < help_plane.cells.len() { help_plane.cells[idx].char = ch; help_plane.cells[idx].fg = theme.outline; }
                }

                let help_title = "Framework Chat Help";
                let tx = (hw - help_title.len() as u16) / 2;
                for (i, c) in help_title.chars().enumerate() {
                    let idx = (1 * hw + tx + i as u16) as usize;
                    if idx < help_plane.cells.len() {
                        help_plane.cells[idx].char = c;
                        help_plane.cells[idx].fg = theme.primary;
                        help_plane.cells[idx].style = Styles::BOLD;
                    }
                }

                let shortcuts = [
                    ("Ctrl+T", "Cycle theme"),
                    ("F1 / ?", "Toggle help"),
                    ("Esc", "Dismiss help"),
                    ("Ctrl+Q", "Quit"),
                ];
                for (i, (key, desc)) in shortcuts.iter().enumerate() {
                    let row = 3 + i as u16;
                    for (j, c) in key.chars().enumerate() {
                        let idx = (row * hw + 2 + j as u16) as usize;
                        if idx < help_plane.cells.len() { help_plane.cells[idx].char = c; help_plane.cells[idx].fg = theme.primary; }
                    }
                    for (j, c) in desc.chars().enumerate() {
                        let idx = (row * hw + 14 + j as u16) as usize;
                        if idx < help_plane.cells.len() { help_plane.cells[idx].char = c; help_plane.cells[idx].fg = theme.fg; }
                    }
                }

                ctx.add_plane(help_plane);
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

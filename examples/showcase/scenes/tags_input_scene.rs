//! Embedded Tags Input scene for the showcase.
//!
//! Demonstrates the TagsInput widget with autocomplete suggestions:
//!   - Email-style tag composition
//!   - Autocomplete suggestions from a preset list
//!   - Tag add/remove with keyboard and mouse
//!   - Live tag count and validation

use crate::scenes::shared_helpers::draw_text;
use dracon_terminal_engine::compositor::plane::{Color, Plane};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widgets::tags_input::TagsInput;
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

pub struct TagsInputScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    tags_input: TagsInput,
    tag_log: Vec<String>,
    dirty: bool,
    area: std::cell::Cell<Rect>,
}

impl TagsInputScene {
    pub fn new(theme: Theme) -> Self {
        let suggestions = vec![
            "rust", "python", "javascript", "typescript", "go", "java",
            "csharp", "ruby", "swift", "kotlin", "dart", "elixir",
            "haskell", "clojure", "scala", "perl", "lua", "zig",
        ];

        let tags_input = TagsInput::new(vec!["rust".into(), "terminal".into()])
            .with_theme(theme.clone())
            .with_placeholder("Add a tag...")
            .with_width(50)
            .with_max_tags(8)
            .with_suggestions(suggestions)
            .on_tag_add(|tag| {
                // Tag add callback (can't modify external state from here)
                let _ = tag;
            })
            .on_tag_remove(|_idx| {});

        Self {
            theme: theme.clone(),
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            tags_input,
            tag_log: vec!["rust added".into(), "terminal added".into()],
            dirty: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }
}

impl Scene for TagsInputScene {
    fn scene_id(&self) -> &str { "tags_input" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        let title = " Tags Input ";
        draw_text(&mut plane, 2, 0, title, t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 2), 0,
                  &theme_label, t.secondary, t.bg, false);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── Description ────────────────────────────────────────────────────
        draw_text(&mut plane, 2, 2, "Email-style tag input with autocomplete.", t.fg, t.bg, false);
        draw_text(&mut plane, 2, 3, "Type and press Enter/Tab to add tags.", t.fg_muted, t.bg, false);
        draw_text(&mut plane, 2, 4, "Backspace removes the last tag.", t.fg_muted, t.bg, false);

        // ── Tags Input Widget ──────────────────────────────────────────────
        let input_y = 6;
        let input_area = Rect::new(2, input_y, area.width.saturating_sub(4), 3);
        let input_plane = self.tags_input.render(input_area);
        for y in 0..input_plane.height.min(input_area.height) {
            for x in 0..input_plane.width.min(input_area.width) {
                let src_idx = (y * input_plane.width + x) as usize;
                let dst_idx = ((input_area.y + y) * area.width + input_area.x + x) as usize;
                if src_idx < input_plane.cells.len() && dst_idx < plane.cells.len() {
                    let src = &input_plane.cells[src_idx];
                    if !src.transparent {
                        plane.cells[dst_idx] = *src;
                    }
                }
            }
        }

        // ── Tag Stats ──────────────────────────────────────────────────────
        let tags = self.tags_input.tags();
        let stats_y = 10;
        let count_text = format!("Tags: {}/{}", tags.len(), 8);
        draw_text(&mut plane, 2, stats_y, &count_text, t.fg, t.bg, true);

        if tags.len() >= 8 {
            draw_text(&mut plane, 15, stats_y, "MAX", Color::Rgb(200, 80, 80), t.bg, true);
        }

        // Current tags list
        if !tags.is_empty() {
            draw_text(&mut plane, 2, stats_y + 1, "Current:", t.fg_muted, t.bg, false);
            let tag_str = tags.join(" · ");
            draw_text(&mut plane, 11, stats_y + 1, &tag_str, t.primary, t.bg, false);
        }

        // ── Activity Log ──────────────────────────────────────────────────
        let log_y = stats_y + 3;
        draw_text(&mut plane, 2, log_y, "Activity Log:", t.fg, t.bg, true);
        for (i, entry) in self.tag_log.iter().rev().take(5).enumerate() {
            draw_text(&mut plane, 2, log_y + 1 + i as u16, &format!("• {}", entry), t.fg_muted, t.bg, false);
        }

        // ── Suggestions Preview ───────────────────────────────────────────
        let sug_y = log_y + 7;
        if sug_y < area.height.saturating_sub(2) {
            draw_text(&mut plane, 2, sug_y, "Available suggestions: rust, python, javascript, ...", t.fg_muted, t.bg, false);
        }

        // ── Footer ────────────────────────────────────────────────────────
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " {}:help | {}:back | Enter/Tab:add | Bksp:remove ",
            help_key, back_key,
        );
        let fy = area.height.saturating_sub(1);
        for (i, c) in footer.chars().enumerate() {
            let idx = (fy * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        if self.show_help {
            self.render_help(&mut plane, area);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.keybindings.matches(actions::BACK, &key) {
            if self.show_help {
                self.show_help = false;
            }
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            return true;
        }
        if self.show_help {
            return false;
        }

        let prev_count = self.tags_input.tags().len();

        if self.tags_input.handle_key(key) {
            let new_count = self.tags_input.tags().len();
            if new_count > prev_count {
                // Tag was added
                if let Some(tag) = self.tags_input.tags().last() {
                    self.tag_log.push(format!("{} added", tag));
                }
            } else if new_count < prev_count {
                self.tag_log.push("tag removed".into());
            }
            self.dirty = true;
            return true;
        }

        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();
        let input_y = 6;
        let input_area = Rect::new(2, input_y, area.width.saturating_sub(4), 3);

        if col >= input_area.x && col < input_area.x + input_area.width
            && row >= input_area.y && row < input_area.y + input_area.height
        {
            let local_col = col - input_area.x;
            let local_row = row - input_area.y;
            if self.tags_input.handle_mouse(kind, local_col, local_row) {
                self.dirty = true;
                return true;
            }
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.tags_input.on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl TagsInputScene {
    fn render_help(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 40u16.min(area.width.saturating_sub(4));
        let hh = 11u16.min(area.height.saturating_sub(4));
        let hx = (area.width - hw) / 2;
        let hy = (area.height - hh) / 2;

        for y in hy..hy + hh {
            for x in hx..hx + hw {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");

        let lines = [
            ("╭────────────────────────────────────╮", true),
            ("│       Tags Input Help              │", true),
            ("├────────────────────────────────────┤", true),
            ("│  Enter/Tab  Add typed tag          │", false),
            ("│  Backspace  Remove last tag        │", false),
            ("│  ↑/↓        Select suggestion      │", false),
            ("│  Type       Filter suggestions     │", false),
            (&format!("│  {:<10} Toggle this help          │", help_key), false),
            (&format!("│  {:<10} Dismiss / go back        │", back_key), false),
            ("╰────────────────────────────────────╯", true),
        ];

        for (i, (line, is_border)) in lines.iter().enumerate() {
            let ly = hy + i as u16;
            let lx = (area.width - line.len() as u16) / 2;
            for (j, ch) in line.chars().enumerate() {
                let px = lx + j as u16;
                if px < area.width && ly < area.height {
                    let idx = (ly * area.width + px) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = if *is_border || "│╭╮├┤╰╯─".contains(ch) { t.outline } else { t.fg };
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }
    }
}

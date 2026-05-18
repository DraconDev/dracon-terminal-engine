//! Embedded Tags Input scene for the showcase.
//!
//! Demonstrates the TagsInput widget with autocomplete suggestions:
//!   - Email-style tag composition with colored pills
//!   - Autocomplete suggestions from a preset list
//!   - Tag add/remove with keyboard and mouse
//!   - Shortcut legend panel, category breakdown, tag stats

use crate::scenes::shared_helpers::draw_text;
use dracon_terminal_engine::compositor::plane::{Color, Plane};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widgets::tags_input::TagsInput;
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

struct TagCategory {
    name: &'static str,
    items: &'static [&'static str],
    color: Color,
}

fn tag_categories() -> Vec<TagCategory> {
    // These are static — colors will be filled in render with theme
    vec![
        TagCategory { name: "Systems", items: &["rust", "go", "zig", "c"], color: Color::Rgb(136, 192, 208) },
        TagCategory { name: "Scripting", items: &["python", "ruby", "perl", "lua"], color: Color::Rgb(208, 135, 112) },
        TagCategory { name: "Web", items: &["javascript", "typescript", "dart"], color: Color::Rgb(163, 190, 140) },
        TagCategory { name: "JVM", items: &["java", "kotlin", "scala", "clojure"], color: Color::Rgb(235, 203, 139) },
        TagCategory { name: "Functional", items: &["haskell", "elixir", "swift"], color: Color::Rgb(180, 142, 173) },
    ]
}

fn tag_color(tag: &str, t: &Theme) -> Color {
    for cat in tag_categories() {
        if cat.items.contains(&tag) { return cat.color; }
    }
    t.fg_muted
}

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
            .on_tag_add(|tag| { let _ = tag; })
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
        draw_text(&mut plane, 2, 0, " Tags Input ", t.primary, t.bg, true);
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

        // Description
        draw_text(&mut plane, 2, 2, "Email-style tag input with autocomplete.", t.fg, t.bg, false);

        // ── Tags Input Widget ──────────────────────────────────────────────
        let input_y = 4;
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

        // ── Left panel: Tag pills + stats ─────────────────────────────────
        let left_w = area.width / 2;
        let tags = self.tags_input.tags();

        // Colored tag pills
        let pill_y = 8;
        draw_text(&mut plane, 2, pill_y, "Current Tags", t.primary, t.bg, true);
        for dx in 0..left_w.saturating_sub(4) {
            let idx = ((pill_y + 1) * plane.width + 2 + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        if tags.is_empty() {
            draw_text(&mut plane, 4, pill_y + 2, "No tags yet", t.fg_muted, t.bg, false);
        } else {
            let mut px = 4u16;
            let mut py = pill_y + 2;
            for tag in tags.iter() {
                let color = tag_color(tag, t);
                let pill_len = tag.len() as u16 + 2; // " tag "

                // Check if we need to wrap
                if px + pill_len > left_w {
                    px = 4;
                    py += 1;
                    if py >= area.height.saturating_sub(2) { break; }
                }

                // Pill background
                for dx in 0..pill_len {
                    let idx = (py * plane.width + px + dx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = color;
                        plane.cells[idx].transparent = false;
                    }
                }

                // Pill text (dark fg on colored bg)
                draw_text(&mut plane, px + 1, py, tag, Color::Rgb(30, 30, 30), color, true);
                px += pill_len + 1;
            }
        }

        // Tag stats
        let stats_y = pill_y + 5;
        if stats_y + 4 < area.height.saturating_sub(2) {
            draw_text(&mut plane, 2, stats_y, "Statistics", t.secondary, t.bg, true);
            for dx in 0..left_w.saturating_sub(4) {
                let idx = ((stats_y + 1) * plane.width + 2 + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }

            let max_tags = 8;
            let count = tags.len();
            let count_text = format!("Count: {}/{}", count, max_tags);
            draw_text(&mut plane, 4, stats_y + 2, &count_text, t.fg, t.bg, false);

            // Capacity bar
            let bar_y = stats_y + 3;
            let bar_w = left_w.saturating_sub(8);
            let filled = bar_w as usize * count / max_tags.max(1);
            for dx in 0..bar_w {
                let idx = (bar_y * plane.width + 4 + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = if (dx as usize) < filled { '█' } else { '░' };
                    plane.cells[idx].fg = if (dx as usize) < filled { t.primary } else { t.fg_muted };
                    plane.cells[idx].transparent = false;
                }
            }

            if count >= max_tags {
                draw_text(&mut plane, 4, bar_y + 1, "Maximum capacity reached", t.warning, t.bg, false);
            }
        }

        // ── Right panel: Shortcuts + categories + log ─────────────────────
        let right_x = left_w + 2;
        let right_w = area.width.saturating_sub(left_w + 2);

        // Shortcuts
        draw_text(&mut plane, right_x, 8, "Shortcuts", t.primary, t.bg, true);
        for dx in 0..right_w.saturating_sub(2) {
            let idx = (9 * plane.width + right_x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
        let shortcuts = [
            ("Enter/Tab", "Add tag"),
            ("Backspace", "Remove last"),
            ("↑/↓", "Pick suggestion"),
            ("Type", "Filter list"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let sy = 10 + i as u16;
            draw_text(&mut plane, right_x, sy, key, t.primary, t.bg, false);
            draw_text(&mut plane, right_x + 14, sy, desc, t.fg_muted, t.bg, false);
        }

        // Categories
        let cat_y = 15;
        if cat_y + 5 < area.height.saturating_sub(2) {
            draw_text(&mut plane, right_x, cat_y, "Categories", t.secondary, t.bg, true);
            for dx in 0..right_w.saturating_sub(2) {
                let idx = ((cat_y + 1) * plane.width + right_x + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }
            let cats = tag_categories();
            for (i, cat) in cats.iter().enumerate() {
                let cy = cat_y + 2 + i as u16;
                if cy >= area.height.saturating_sub(2) { break; }

                // Color swatch
                let swatch_idx = (cy * plane.width + right_x) as usize;
                if swatch_idx < plane.cells.len() {
                    plane.cells[swatch_idx].char = '■';
                    plane.cells[swatch_idx].fg = cat.color;
                    plane.cells[swatch_idx].transparent = false;
                }

                // Category name + items count
                let label = format!("{} ({} tags)", cat.name, cat.items.len());
                draw_text(&mut plane, right_x + 2, cy, &label, t.fg, t.bg, false);

                // Highlight if any current tag belongs
                let has_match = tags.iter().any(|t| cat.items.contains(&t.as_str()));
                if has_match {
                    draw_text(&mut plane, right_x + right_w.saturating_sub(2), cy, "●", t.success, t.bg, false);
                }
            }
        }

        // Activity log (bottom)
        let log_y = area.height.saturating_sub(5);
        if log_y > cat_y + 8 {
            draw_text(&mut plane, 2, log_y, "Activity Log", t.secondary, t.bg, true);
            for dx in 0..area.width.saturating_sub(4) {
                let idx = ((log_y + 1) * plane.width + 2 + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }
            for (i, entry) in self.tag_log.iter().rev().take(3).enumerate() {
                draw_text(&mut plane, 4, log_y + 2 + i as u16, &format!("• {}", entry), t.fg_muted, t.bg, false);
            }
        }

        // Vertical divider
        for y in 8..area.height.saturating_sub(2) {
            let idx = (y * plane.width + left_w) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Footer
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " Enter/Tab:add | Bksp:remove | {}:help | {}:back ",
            self.keybindings.display(actions::HELP).unwrap_or("f1"),
            back_key,
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
        if key.kind != KeyEventKind::Press { return false; }

        if self.keybindings.matches(actions::BACK, &key) {
            if self.show_help { self.show_help = false; }
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) || key.code == dracon_terminal_engine::input::event::KeyCode::Char('?') {
            self.show_help = !self.show_help;
            return true;
        }
        if self.show_help { return true; }

        let prev_count = self.tags_input.tags().len();
        if self.tags_input.handle_key(key) {
            let new_count = self.tags_input.tags().len();
            if new_count > prev_count {
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
        let input_y = 4;
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
        let hw = 44u16.min(area.width.saturating_sub(4));
        let hh = 12u16.min(area.height.saturating_sub(4));
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
        for x in hx + 1..hx + hw - 1 {
            let top = (hy * plane.width + x) as usize;
            let bot = ((hy + hh - 1) * plane.width + x) as usize;
            if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
            if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
        }
        for y in hy + 1..hy + hh - 1 {
            let left = (y * plane.width + hx) as usize;
            let right = (y * plane.width + hx + hw - 1) as usize;
            if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
            if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
        }
        for (ch, cx, cy) in [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)] {
            let idx = (cy * plane.width + cx) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = t.outline; }
        }

        let title = "Tags Input Help";
        let tx = hx + (hw - title.len() as u16) / 2;
        draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

        let shortcuts = [
            ("Enter/Tab", "Add typed tag"),
            ("Backspace", "Remove last tag"),
            ("↑/↓", "Select suggestion"),
            ("Type", "Filter suggestions"),
            ("?", "Toggle this help"),
            ("B/Esc", "Back to showcase"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
            draw_text(plane, hx + 16, row, desc, t.fg, t.surface_elevated, false);
        }
    }
}

//! Embedded Tags Input scene for the showcase.
//!
//! Demonstrates the TagsInput widget with autocomplete suggestions:
//!   - Email-style tag composition with colored pills
//!   - Autocomplete suggestions from a preset list
//!   - Tag add/remove with keyboard and mouse
//!   - Tag cloud grouped by category with interactive highlighting

use crate::scenes::shared_helpers::{draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::{Color, Plane};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widgets::tags_input::TagsInput;
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

const SIDEBAR_W: u16 = 22;
const DIV_X: u16 = SIDEBAR_W + 2;

struct TagCategory {
    name: &'static str,
    icon: char,
    items: &'static [&'static str],
    color: Color,
}

fn tag_categories() -> Vec<TagCategory> {
    vec![
        TagCategory { name: "Systems", icon: '⚙', items: &["rust", "go", "zig", "c"], color: Color::Rgb(136, 192, 208) },
        TagCategory { name: "Scripting", icon: '⌨', items: &["python", "ruby", "perl", "lua"], color: Color::Rgb(208, 135, 112) },
        TagCategory { name: "Web", icon: '◇', items: &["javascript", "typescript", "dart"], color: Color::Rgb(163, 190, 140) },
        TagCategory { name: "JVM", icon: '☕', items: &["java", "kotlin", "scala", "clojure"], color: Color::Rgb(235, 203, 139) },
        TagCategory { name: "Functional", icon: 'λ', items: &["haskell", "elixir", "swift"], color: Color::Rgb(180, 142, 173) },
    ]
}

fn tag_category_color(tag: &str) -> Color {
    for cat in tag_categories() {
        if cat.items.contains(&tag) { return cat.color; }
    }
    Color::Rgb(128, 128, 128)
}

fn tag_icon(tag: &str) -> char {
    match tag {
        "rust" => '⚙',
        "go" => '▶',
        "python" => '*',
        "javascript" => '*',
        "typescript" => '*',
        "java" => '☕',
        "haskell" => 'λ',
        "swift" => '◆',
        "zig" => '⚡',
        "dart" => '◉',
        _ => '●',
    }
}

pub struct TagsInputScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    tags_input: TagsInput,
    tag_log: Vec<String>,
    dirty: bool,
    area: std::cell::Cell<Rect>,
    hovered_tag: Option<usize>,
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
            .on_tag_add(|_tag| {})
            .on_tag_remove(|_idx| {});

        Self {
            theme: theme.clone(),
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            tags_input,
            tag_log: vec!["rust added".into(), "terminal added".into()],
            dirty: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            hovered_tag: None,
        }
    }

    fn brighten(&self, color: Color) -> Color {
        match color {
            Color::Rgb(r, g, b) => Color::Rgb(r.saturating_add(30), g.saturating_add(30), b.saturating_add(30)),
            _ => color,
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
        draw_text(&mut plane, 2, 0, " Tag Manager ", t.primary, t.bg, true);
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

        // ── Left sidebar: categories ─────────────────────────────────────
        self.render_sidebar(&mut plane, area, t);

        // Vertical divider
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * area.width + DIV_X) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // ── Main area ───────────────────────────────────────────────────
        let main_x = DIV_X + 2;
        let main_w = area.width.saturating_sub(main_x + 2);
        let tags = self.tags_input.tags();

        // TagsInput widget at top
        let input_y = 2;
        let input_area = Rect::new(main_x, input_y, main_w, 3);
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

        // ── Active Tags Panel ─────────────────────────────────────────────
        let tags_y = 6;
        draw_text(&mut plane, main_x, tags_y, "Active Tags", t.primary, t.bg, true);
        let tags_hr_y = tags_y + 1;
        for dx in 0..main_w {
            let idx = (tags_hr_y * area.width + main_x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        if tags.is_empty() {
            draw_text(&mut plane, main_x + 2, tags_y + 3, "(no tags — type above or click a category)", t.fg_muted, t.bg, false);
        } else {
            // Colored tag pills
            let mut px = main_x + 2;
            let mut py = tags_y + 3;
            let pill_h = 2u16;

            for (i, tag) in tags.iter().enumerate() {
                let color = tag_category_color(tag);
                let tag_icon = tag_icon(tag);
                let pill_len = (tag.len() + 4) as u16; // " ● tag "

                if px + pill_len > main_x + main_w {
                    px = main_x + 2;
                    py += pill_h;
                }
                if py + pill_h > area.height.saturating_sub(4) { break; }

                let is_hovered = self.hovered_tag == Some(i);

                // Pill background
                let pill_bg = if is_hovered { self.brighten(color) } else { color };
                for dy in 0..pill_h {
                    for dx in 0..pill_len {
                        let idx = ((py + dy) * area.width + px + dx) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].bg = pill_bg;
                            plane.cells[idx].transparent = false;
                        }
                    }
                }

                // Pill text
                let text_color = Color::Rgb(20, 20, 20);
                let icon_idx = (py * area.width + px + 1) as usize;
                if icon_idx < plane.cells.len() {
                    plane.cells[icon_idx].char = tag_icon;
                    plane.cells[icon_idx].fg = text_color;
                }
                draw_text(&mut plane, px + 3, py, tag, text_color, color, true);

                // × button on right
                let close_x = px + pill_len - 2;
                let close_idx = (py * area.width + close_x) as usize;
                if close_idx < plane.cells.len() {
                    plane.cells[close_idx].char = '×';
                    plane.cells[close_idx].fg = text_color;
                }

                px += pill_len + 1;
            }

            // ── Capacity indicator ───────────────────────────────────────
            let cap_y = py + pill_h + 1;
            if cap_y < area.height.saturating_sub(4) {
                let max_tags = 8;
                let count = tags.len();
                let cap_text = format!("{}/{} tags used", count, max_tags);
                draw_text(&mut plane, main_x, cap_y, &cap_text, t.fg_muted, t.bg, false);

                let bar_y = cap_y + 1;
                let bar_w = main_w / 2;
                let filled = (bar_w as usize * count / max_tags).min(bar_w as usize);
                for dx in 0..bar_w {
                    let idx = (bar_y * area.width + main_x + dx) as usize;
                    if idx < plane.cells.len() {
                        let bar_color = if count >= max_tags { t.warning } else { t.primary };
                        plane.cells[idx].char = if (dx as usize) < filled { '█' } else { '░' };
                        plane.cells[idx].fg = bar_color;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }

        // ── Activity Log ────────────────────────────────────────────────
        let log_y = area.height.saturating_sub(8);
        if log_y > tags_y + 12 {
            draw_text(&mut plane, main_x, log_y, "Activity Log", t.secondary, t.bg, true);
            for dx in 0..main_w {
                let idx = ((log_y + 1) * area.width + main_x + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }
            for (i, entry) in self.tag_log.iter().rev().take(3).enumerate() {
                let color = if entry.contains("removed") { t.warning } else { t.success };
                draw_text(&mut plane, main_x, log_y + 2 + i as u16, &format!("• {}", entry), color, t.bg, false);
            }
        }

        // ── Footer ─────────────────────────────────────────────────────
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " Enter/Tab:add | Bksp:remove | Click tag:remove | Click cat:add | {}:help | {}:back ",
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
            let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(&mut plane, area, &self.theme, "Tag Manager — Help", &[
                ("Enter/Tab", "Add typed tag"),
                ("Backspace", "Remove last tag"),
                ("↑/↓", "Select suggestion"),
                ("Type", "Filter suggestions"),
                ("Click tag", "Remove that tag"),
                ("Click cat", "Add first available"),
                (help_key, "Toggle this help"),
                (back_key, "Back"),
            ]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.keybindings.matches(actions::BACK, &key) {
            if self.show_help { self.show_help = false; self.dirty = true; return true; }
            return false;
        }
        if self.keybindings.matches(actions::HELP, &key) || key.code == dracon_terminal_engine::input::event::KeyCode::Char('?') {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.show_help { return true; }

        let prev_count = self.tags_input.tags().len();
        if self.tags_input.handle_key(key) {
            let new_count = self.tags_input.tags().len();
            if new_count > prev_count {
                if let Some(tag) = self.tags_input.tags().last() {
                    self.tag_log.push(format!("+ {} added", tag));
                }
            } else if new_count < prev_count {
                self.tag_log.push("- tag removed".into());
            }
            self.dirty = true;
            return true;
        }

        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();

        // TagsInput widget area
        let input_area = Rect::new(DIV_X + 2, 2, area.width.saturating_sub(DIV_X + 4), 3);
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

        match kind {
            MouseEventKind::Moved => {
                // Track hovered tag pill
                self.hovered_tag = None;
                if col > DIV_X + 2 && row > 8 {
                    let tags = self.tags_input.tags();
                    let mut px = DIV_X + 4;
                    let mut py = 9u16;
                    let pill_h = 2u16;

                    let tags_count = tags.len();
                    for (i, tag) in tags.iter().enumerate().take(tags_count) {
                        let pill_len = (tag.len() + 4) as u16;

                        if px + pill_len > area.width.saturating_sub(DIV_X + 4) {
                            px = DIV_X + 4;
                            py += pill_h;
                        }
                        if row >= py && row < py + pill_h && col >= px && col < px + pill_len {
                            self.hovered_tag = Some(i);
                            self.dirty = true;
                            return true;
                        }
                        px += pill_len + 1;
                    }
                }
                false
            }
            MouseEventKind::Down(_) => {
                // Click on active tag pill → remove it
                if col > DIV_X + 2 && row > 8 {
                    let tags = self.tags_input.tags();
                    let tags_count = tags.len();
                    let mut px = DIV_X + 4;
                    let mut py = 9u16;
                    let pill_h = 2u16;

                    for (i, tag) in tags.iter().enumerate().take(tags_count) {
                        let pill_len = (tag.len() + 4) as u16;

                        if px + pill_len > area.width.saturating_sub(DIV_X + 4) {
                            px = DIV_X + 4;
                            py += pill_h;
                        }
                        if row >= py && row < py + pill_h && col >= px && col < px + pill_len {
                            let tag_name = tag.to_string();
                            self.tags_input.remove_tag(i);
                            self.tag_log.push(format!("- {} removed", tag_name));
                            self.dirty = true;
                            return true;
                        }
                        px += pill_len + 1;
                    }
                }

                // Click on sidebar category → add first available tag
                if col < DIV_X && row > 3 {
                    let cats = tag_categories();
                    let idx = ((row - 4) as usize) / 5;
                    if idx < cats.len() {
                        let cat = &cats[idx];
                        let current_tags = self.tags_input.tags();
                        if let Some(item) = cat.items.iter().find(|it| !current_tags.contains(&it.to_string())) {
                            self.tags_input.add_tag(item.to_string());
                            self.tag_log.push(format!("+ {} added ({})", item, cat.name));
                            self.dirty = true;
                            return true;
                        }
                    }
                }

                false
            }
            _ => false,
        }
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
    fn render_sidebar(&self, plane: &mut Plane, area: Rect, t: &Theme) {
        let sx = 2u16;

        // Title
        draw_text(plane, sx, 2, "Categories", t.primary, t.bg, true);

        let cats = tag_categories();
        let current_tags: Vec<String> = self.tags_input.tags().to_vec();

        for (i, cat) in cats.iter().enumerate() {
            let y = 4 + i as u16 * 5;

            // Category header
            let icon_idx = (y * plane.width + sx) as usize;
            if icon_idx < plane.cells.len() {
                plane.cells[icon_idx].char = cat.icon;
                plane.cells[icon_idx].fg = cat.color;
            }
            draw_text(plane, sx + 2, y, cat.name, cat.color, t.bg, true);

            // Tags in this category
            for (j, tag) in cat.items.iter().enumerate() {
                let ty = y + 1 + j as u16;
                if ty >= area.height.saturating_sub(2) { break; }

                let is_active = current_tags.contains(&tag.to_string());
                let bg = if is_active { cat.color } else { t.surface };
                let fg = if is_active { Color::Rgb(20, 20, 20) } else { t.fg };

                // Fill row
                for cx in 0..SIDEBAR_W {
                    let idx = (ty * plane.width + sx + cx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = bg;
                        plane.cells[idx].transparent = false;
                    }
                }

                // Status indicator
                let ind_char = if is_active { '●' } else { '○' };
                let ind_color = if is_active { t.success } else { t.fg_muted };
                draw_text_clipped(plane, sx + 1, ty, &ind_char.to_string(), sx + SIDEBAR_W, ind_color, bg, false);
                draw_text_clipped(plane, sx + 3, ty, tag, sx + SIDEBAR_W, fg, bg, false);
            }

            // Divider between categories
            let div_y = y + 5;
            if div_y < area.height.saturating_sub(4) {
                for dx in 0..SIDEBAR_W {
                    let idx = (div_y * plane.width + sx + dx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '─';
                        plane.cells[idx].fg = t.outline;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }

        // Stats at bottom of sidebar
        let stats_y = area.height.saturating_sub(5);
        if stats_y > 4 + cats.len() as u16 * 5 + 2 {
            draw_text(plane, sx, stats_y, "Summary", t.secondary, t.bg, true);

            // Count tags per category
            let mut cat_counts = Vec::new();
            for cat in &cats {
                let count = cat.items.iter().filter(|t| current_tags.contains(&t.to_string())).count();
                if count > 0 {
                    cat_counts.push((cat.name, count, cat.color));
                }
            }

            if cat_counts.is_empty() {
                draw_text(plane, sx, stats_y + 2, "No categories used", t.fg_muted, t.bg, false);
            } else {
                for (i, (name, count, color)) in cat_counts.iter().enumerate() {
                    let sy = stats_y + 2 + i as u16;
                    let swatch_idx = (sy * plane.width + sx) as usize;
                    if swatch_idx < plane.cells.len() {
                        plane.cells[swatch_idx].char = '■';
                        plane.cells[swatch_idx].fg = *color;
                    }
                    draw_text_clipped(plane, sx + 2, sy, &format!("{}:{}", name, count), sx + SIDEBAR_W, t.fg, t.bg, false);
                }
            }
        }
    }
}
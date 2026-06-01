//! Embedded RichText scene for the showcase.
//!
//! Demonstrates the RichText widget with tabbed markdown documents,
//! a document tree sidebar, and live preview.

use crate::scenes::shared_helpers::{blit_to, draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::{Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widgets::RichText;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

const SIDEBAR_W: u16 = 22;
const DIV_X: u16 = SIDEBAR_W + 2;

const DOC_README: &str = r#"# RichText Widget

A powerful Markdown renderer with **bold**, *italic*, `inline code`, and more.

## Features

- **Bold** and *italic* text support
- `inline code` blocks
- [Links](https://example.com) with underline styling
- Unordered lists with bullet markers
- Multi-level headers (H1-H6)
- Word wrapping for long content

### Code Block Demo

```rust
fn main() {
    println!("Hello, RichText!");
}
```

### Links and Lists

- [Documentation](https://docs.example.com)
- [GitHub Repository](https://github.com/example)
- [API Reference](https://api.example.com)"#;

const DOC_CHANGELOG: &str = r#"# Changelog

## v0.3.0 — 2026-05-17

### Added
- **Color Picker** widget with HSL and hex input
- **Tags Input** widget with autocomplete suggestions
- **Animation scene** in showcase with easing curves
- Visual gauges in CellPool scene

### Changed
- Rewrote chat client to use framework widgets
- Upgraded notification center with filtering

## v0.2.0 — 2026-04-01

### Added
- **Widget decomposition** Phase 1 (sub-traits)
- `KeyCode::Unsupported(u32)` variant
- Media key support in keybindings

## v0.1.0 — 2026-01-15

- Initial release with 50+ widgets
- 20+ built-in themes
- Mouse-first interaction model"#;

const DOC_GUIDE: &str = r#"# Quick Start Guide

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
dracon-terminal-engine = "0.3"
```

## Hello World

```rust
use dracon_terminal_engine::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?.title("Hello");
    app.add_widget(Box::new(my_widget), Rect::new(0, 0, 0, 0));
    app.run(|_| {})
}
```

## Themes

Use built-in themes or create custom ones:

```rust
let theme = Theme::nord();
let theme = Theme::from_env_or(Theme::dark());
```

## Keybindings

Override in `dracon.toml`:

```toml
[keybindings]
quit = "ctrl+q"
help = "f1"
theme = "ctrl+t"
```"#;

const DOC_API: &str = r#"# API Reference

## Theme API

```rust
pub struct Theme {
    pub name: String,
    pub fg: Color,
    pub bg: Color,
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}
```

## Widget API

```rust
pub trait Widget {
    fn render(&self, area: Rect) -> Plane;
    fn handle_key(&mut self, key: KeyEvent) -> bool;
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool;
}
```

## Color API

```rust
pub enum Color {
    Reset,
    Black, Red, Green, Yellow, Blue, Magenta, Cyan, White,
    BrightBlack, BrightRed, BrightGreen, BrightYellow,
    BrightBlue, BrightMagenta, BrightCyan, BrightWhite,
    Indexed(u8),
    Rgb(u8, u8, u8),
}
```"#;

struct DocInfo {
    label: &'static str,
    content: &'static str,
    icon: char,
    category: &'static str,
}

const DOCUMENTS: &[DocInfo] = &[
    DocInfo {
        label: "README",
        content: DOC_README,
        icon: '📄',
        category: "Overview",
    },
    DocInfo {
        label: "Changelog",
        content: DOC_CHANGELOG,
        icon: '📋',
        category: "Overview",
    },
    DocInfo {
        label: "Guide",
        content: DOC_GUIDE,
        icon: '📖',
        category: "Learning",
    },
    DocInfo {
        label: "API",
        content: DOC_API,
        icon: '⚙',
        category: "Reference",
    },
];

pub struct RichTextScene {
    theme: Theme,
    show_help: bool,
    selected_doc: usize,
    rich_text: RefCell<RichText>,
    keybindings: KeybindingSet,
    dirty: bool,
}

impl RichTextScene {
    pub fn new(theme: Theme) -> Self {
        let rich_text = RefCell::new(RichText::new(DOC_README).with_theme(theme.clone()));

        Self {
            theme,
            show_help: false,
            selected_doc: 0,
            rich_text,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            dirty: true,
        }
    }

    fn switch_doc(&mut self, idx: usize) {
        if idx < DOCUMENTS.len() && idx != self.selected_doc {
            self.selected_doc = idx;
            self.rich_text
                .borrow_mut()
                .set_content(DOCUMENTS[idx].content);
            self.dirty = true;
        }
    }

    fn current_doc(&self) -> &DocInfo {
        &DOCUMENTS[self.selected_doc]
    }

    fn doc_stats(&self, doc: &str) -> (usize, usize, usize) {
        let words = doc.split_whitespace().count();
        let lines = doc.lines().count();
        let chars = doc.len();
        (lines, words, chars)
    }
}

impl Scene for RichTextScene {
    fn scene_id(&self) -> &str {
        "rich_text"
    }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // ── Header ──────────────────────────────────────────────────────
        draw_text(&mut plane, 2, 0, " Document Viewer ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(
            &mut plane,
            area.width.saturating_sub(theme_label.len() as u16 + 2),
            0,
            &theme_label,
            t.secondary,
            t.bg,
            false,
        );

        // Divider
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── Left sidebar ───────────────────────────────────────────────
        self.render_sidebar(&mut plane, area, t);

        // Vertical divider
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * plane.width + DIV_X) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // ── Main area ───────────────────────────────────────────────────
        let main_x = DIV_X + 2;
        let main_w = area.width.saturating_sub(main_x + 2);

        // Tab bar row
        let tab_y = 2;
        self.render_tab_bar(&mut plane, main_x, tab_y, main_w, t);

        // Document info bar
        let doc = self.current_doc().content;
        let (lines, words, chars) = self.doc_stats(doc);
        let info = format!("{} lines  {} words  {} chars", lines, words, chars);
        draw_text_clipped(
            &mut plane,
            main_x,
            tab_y + 2,
            &info,
            main_x + main_w,
            t.fg_muted,
            t.bg,
            false,
        );

        // RichText content area
        let content_area = Rect::new(
            main_x,
            tab_y + 4,
            main_w,
            area.height.saturating_sub(tab_y + 6),
        );
        let content_plane = self.rich_text.borrow().render(content_area);
        blit_to(
            &mut plane,
            &content_plane,
            main_x as usize,
            (tab_y + 4) as usize,
        );

        // ── Footer ─────────────────────────────────────────────────────
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " Tab:switch | 1-4:jump | ↑↓:scroll | {}:help | {}:back ",
            help_key, back_key,
        );
        let fy = area.height.saturating_sub(1);
        for (i, c) in footer.chars().enumerate() {
            let idx = (fy * plane.width + i as u16) as usize;
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
            render_help_overlay(
                &mut plane,
                area,
                &self.theme,
                "Document Viewer — Help",
                &[
                    ("Tab", "Next document"),
                    ("Shift+Tab", "Previous document"),
                    ("1/2/3/4", "Jump to document"),
                    ("↑/↓/PgUp/PgDn", "Scroll content"),
                    ("Click tab", "Select document"),
                    (help_key, "Toggle this help"),
                    (back_key, "Back"),
                ],
            );
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        match key.code {
            KeyCode::Tab => {
                self.switch_doc((self.selected_doc + 1) % DOCUMENTS.len());
                true
            }
            KeyCode::BackTab => {
                self.switch_doc(if self.selected_doc == 0 {
                    DOCUMENTS.len() - 1
                } else {
                    self.selected_doc - 1
                });
                true
            }
            KeyCode::Char('1') if key.modifiers.is_empty() => {
                self.switch_doc(0);
                true
            }
            KeyCode::Char('2') if key.modifiers.is_empty() => {
                self.switch_doc(1);
                true
            }
            KeyCode::Char('3') if key.modifiers.is_empty() => {
                self.switch_doc(2);
                true
            }
            KeyCode::Char('4') if key.modifiers.is_empty() => {
                self.switch_doc(3);
                true
            }
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Home
            | KeyCode::End => {
                self.rich_text.borrow_mut().handle_key(key);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Sidebar clicks
        let sidebar_click = col < DIV_X && row >= 3;
        if sidebar_click
            && matches!(
                kind,
                MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left)
            )
        {
            let mut y = 3u16;
            for (i, doc) in DOCUMENTS.iter().enumerate() {
                let prev_category = if i > 0 { DOCUMENTS[i - 1].category } else { "" };
                let row_h = if doc.category != prev_category { 3 } else { 1 };
                if row >= y && row < y + row_h {
                    self.switch_doc(i);
                    return true;
                }
                y += row_h;
            }
        }

        // Tab bar clicks (main area, row 2)
        if row == 2 && col >= DIV_X + 2 {
            let main_x = DIV_X + 2;
            let main_w = 80; // approximate
            let tab_w = main_w / DOCUMENTS.len() as u16;
            let tab_idx = (col.saturating_sub(main_x) / tab_w) as usize;
            if tab_idx < DOCUMENTS.len() {
                self.switch_doc(tab_idx);
                return true;
            }
        }

        // Content scroll
        if row > 4 && col > DIV_X {
            let scrolled = self.rich_text.borrow_mut().handle_mouse(kind, col, row);
            if scrolled {
                self.dirty = true;
            }
            return scrolled;
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.rich_text.borrow_mut().on_theme_change(theme);
    }

    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

impl RichTextScene {
    fn render_sidebar(&self, plane: &mut Plane, area: Rect, t: &Theme) {
        let sx = 2u16;

        // Title
        draw_text(plane, sx, 2, "Documents", t.primary, t.bg, true);

        // Document list
        let mut y = 3u16;
        let mut current_category = "";

        for (i, doc) in DOCUMENTS.iter().enumerate() {
            // New category header
            if doc.category != current_category {
                current_category = doc.category;
                let header_text = format!(" {}", current_category);
                let idx = (y * plane.width + sx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface;
                    plane.cells[idx].transparent = false;
                }
                for (j, c) in header_text.chars().enumerate() {
                    let idx = (y * plane.width + sx + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.secondary;
                        plane.cells[idx].bg = t.surface;
                        plane.cells[idx].transparent = false;
                    }
                }
                y += 1;
            }

            // Document row
            let is_selected = i == self.selected_doc;
            let bg = if is_selected { t.primary } else { t.bg };
            let fg = if is_selected { t.fg_on_accent } else { t.fg };
            let row_text = format!(" {} {}", doc.icon, doc.label);

            // Background
            for (j, _) in row_text.chars().enumerate() {
                let idx = (y * plane.width + sx + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].transparent = false;
                }
            }

            // Text
            for (j, c) in row_text.chars().enumerate() {
                let idx = (y * plane.width + sx + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].transparent = false;
                }
            }

            y += 1;
            if y >= area.height.saturating_sub(8) {
                break;
            }
        }

        // Divider
        let div_y = y + 1;
        for dx in 0..SIDEBAR_W {
            let idx = (div_y * plane.width + sx + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Current document details
        let details_y = div_y + 2;
        if details_y < area.height.saturating_sub(4) {
            draw_text(plane, sx, details_y, "Current Doc", t.secondary, t.bg, true);

            let current = self.current_doc();
            let (lines, words, chars) = self.doc_stats(current.content);

            let stats = [
                ("Name", current.label),
                ("Category", current.category),
                ("Lines", &lines.to_string()),
                ("Words", &words.to_string()),
                ("Chars", &chars.to_string()),
            ];

            for (i, (label, value)) in stats.iter().enumerate() {
                let sy = details_y + 1 + i as u16;
                if sy >= area.height.saturating_sub(4) {
                    break;
                }
                draw_text_clipped(plane, sx, sy, label, sx + 8, t.fg_muted, t.bg, false);
                draw_text_clipped(plane, sx + 9, sy, value, sx + SIDEBAR_W, t.fg, t.bg, false);
            }
        }

        // Quick stats at bottom
        let stats_y = area.height.saturating_sub(6);
        if stats_y > details_y + 7 {
            draw_text(plane, sx, stats_y, "Quick Stats", t.secondary, t.bg, true);

            let total_docs = DOCUMENTS.len();
            let total_lines: usize = DOCUMENTS.iter().map(|d| d.content.lines().count()).sum();
            let total_words: usize = DOCUMENTS
                .iter()
                .map(|d| d.content.split_whitespace().count())
                .sum();

            let quick_stats = [
                ("Docs", &total_docs.to_string()),
                ("Total Lines", &total_lines.to_string()),
                ("Total Words", &total_words.to_string()),
            ];

            for (i, (label, value)) in quick_stats.iter().enumerate() {
                let sy = stats_y + 1 + i as u16;
                draw_text(plane, sx, sy, label, t.fg_muted, t.bg, false);
                draw_text_clipped(plane, sx + 12, sy, value, sx + SIDEBAR_W, t.fg, t.bg, false);
            }
        }
    }

    fn render_tab_bar(&self, plane: &mut Plane, x: u16, y: u16, w: u16, t: &Theme) {
        let tab_w = w / DOCUMENTS.len() as u16;

        for (i, doc) in DOCUMENTS.iter().enumerate() {
            let is_selected = i == self.selected_doc;
            let bg = if is_selected { t.primary } else { t.surface };
            let fg = if is_selected {
                t.fg_on_accent
            } else {
                t.fg_muted
            };
            let style = if is_selected {
                Styles::BOLD
            } else {
                Styles::empty()
            };

            let tab_x = x + (i as u16) * tab_w;
            let label = format!(" {} ", doc.label);

            for (j, ch) in label.chars().enumerate() {
                let idx = (y * plane.width + tab_x + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].style = style;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Fill remaining area
        let end_x = x + (DOCUMENTS.len() as u16) * tab_w;
        for cx in end_x..x + w {
            let idx = (y * plane.width + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
    }
}

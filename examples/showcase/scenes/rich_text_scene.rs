//! Embedded RichText scene for the showcase.
//!
//! Demonstrates the RichText widget with tabbed markdown documents.

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::{Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widgets::RichText;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

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

struct TabInfo {
    label: &'static str,
    content: &'static str,
}

pub struct RichTextScene {
    tabs: Vec<TabInfo>,
    selected_tab: usize,
    rich_text: RichText,
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    dirty: bool,
}

impl RichTextScene {
    pub fn new(theme: Theme) -> Self {
        let tabs = vec![
            TabInfo { label: "README", content: DOC_README },
            TabInfo { label: "Changelog", content: DOC_CHANGELOG },
            TabInfo { label: "Guide", content: DOC_GUIDE },
        ];
        let rich_text = RichText::new(DOC_README).with_theme(theme.clone());

        Self {
            tabs,
            selected_tab: 0,
            rich_text,
            theme,
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            dirty: true,
        }
    }

    fn switch_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() && idx != self.selected_tab {
            self.selected_tab = idx;
            self.rich_text = RichText::new(self.tabs[idx].content).with_theme(self.theme.clone());
            self.dirty = true;
        }
    }

    fn render_tab_bar(&self, plane: &mut Plane, y: u16, area: Rect) {
        let t = &self.theme;
        let mut x = 2u16;
        for (i, tab) in self.tabs.iter().enumerate() {
            let is_selected = i == self.selected_tab;
            let label = format!(" {} ", tab.label);
            let bg = if is_selected { t.primary } else { t.surface };
            let fg = if is_selected { t.fg_on_accent } else { t.fg_muted };
            let style = if is_selected { Styles::BOLD } else { Styles::empty() };

            for (j, ch) in label.chars().enumerate() {
                let idx = (y * area.width + x + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].style = style;
                    plane.cells[idx].transparent = false;
                }
            }
            x += label.len() as u16;
        }
        // Fill remaining tab bar area
        for cx in x..area.width {
            let idx = (y * area.width + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
    }
}

impl Scene for RichTextScene {
    fn scene_id(&self) -> &str { "rich_text" }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        draw_text(&mut plane, 2, 0, " RichText ", t.primary, t.bg, true);
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

        // Tab bar
        self.render_tab_bar(&mut plane, 2, area);

        // Tab content indicator
        let tab_info = format!("Document {} of {}", self.selected_tab + 1, self.tabs.len());
        draw_text(&mut plane, area.width.saturating_sub(tab_info.len() as u16 + 2), 2, &tab_info, t.fg_muted, t.bg, false);

        // Document info bar (word count, line count)
        let doc = match self.selected_tab {
            0 => DOC_README,
            1 => DOC_CHANGELOG,
            _ => DOC_GUIDE,
        };
        let word_count = doc.split_whitespace().count();
        let line_count = doc.lines().count();
        let char_count = doc.len();
        let info = format!("{} lines | {} words | {} chars", line_count, word_count, char_count);
        draw_text(&mut plane, 2, 3, &info, t.fg_muted, t.bg, false);

        // Selected tab indicator on right
        let tab_names = ["README", "Changelog", "Guide"];
        if self.selected_tab < tab_names.len() {
            draw_text(&mut plane, area.width.saturating_sub(tab_names[self.selected_tab].len() as u16 + 2), 3,
                      tab_names[self.selected_tab], t.primary, t.bg, true);
        }

        // RichText content area
        let content_area = Rect::new(area.x + 2, area.y + 4, area.width.saturating_sub(4), area.height.saturating_sub(7));
        let content_plane = self.rich_text.render(content_area);
        blit_to(&mut plane, &content_plane, content_area.x as usize, content_area.y as usize);

        // Footer
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " Tab:switch docs | {}:help | {}:back ",
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
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(&mut plane, area, &self.theme, "RichText — Help", &[
                ("Tab", "Next document"),
                ("Shift+Tab", "Previous document"),
                ("1/2/3", "Jump to document"),
                ("Click tab", "Select document"),
                (back_key, "Back to showcase"),
            ]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
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
                self.switch_tab((self.selected_tab + 1) % self.tabs.len());
                true
            }
            KeyCode::BackTab => {
                self.switch_tab(if self.selected_tab == 0 { self.tabs.len() - 1 } else { self.selected_tab - 1 });
                true
            }
            KeyCode::Char('1') if key.modifiers.is_empty() => { self.switch_tab(0); true }
            KeyCode::Char('2') if key.modifiers.is_empty() => { self.switch_tab(1); true }
            KeyCode::Char('3') if key.modifiers.is_empty() => { self.switch_tab(2); true }
            // Scroll keys forwarded to RichText widget
            KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                self.rich_text.handle_key(key);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Tab bar clicks (row 2)
        if row == 2
            && matches!(kind, MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left)) {
                let mut x = 2u16;
                for (i, tab) in self.tabs.iter().enumerate() {
                    let tab_w = tab.label.len() as u16 + 2;
                    if col >= x && col < x + tab_w {
                        self.switch_tab(i);
                        return true;
                    }
                    x += tab_w;
                }
            }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.rich_text.on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}



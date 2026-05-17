use dracon_terminal_engine::compositor::{Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

const DOC_FEATURES: &str = r#"# Rich Text / Markdown Demo

This widget renders **styled text** with *minimal* Markdown support.

## Features

- **Headers**  -  like the one above
- **Bold text**  -  using double asterisks
- *Italic text*  -  using single asterisks or underscores
- `Inline code`  -  using backticks
- [Links](https://github.com)  -  click-supported (OSC 8)
- List items  -  using dashes

## Example Paragraph

The quick brown fox jumps over the lazy dog. This is a longer paragraph that demonstrates word wrapping within the widget boundaries. The renderer uses `unicode-width` for accurate character measurement and breaks at word boundaries when possible.

## Code Example

You can write `let x = 42;` inline or refer to `functions` and `types`.

*Try cycling themes with Ctrl+T to see how colors adapt.*"#;

const DOC_GUIDE: &str = r#"# Markdown Syntax Guide

A quick reference for the **RichText** widget's supported syntax.

## Text Styles

- **Bold**: wrap text in double asterisks `**like this**`
- *Italic*: wrap text in single asterisks `*like this*`
- `Code`: wrap text in backticks

## Headers

Use `#` symbols for headers:
- `# H1` — Level 1
- `## H2` — Level 2
- `### H3` — Level 3

## Lists

Use dashes or asterisks for bullet items:
- First item
- Second item
- Third item with **bold** and *italic* mixed

## Links

Write links as `[text](url)`:
- [GitHub](https://github.com)
- [Rust docs](https://doc.rust-lang.org)

## Word Wrapping

Long paragraphs automatically wrap at word boundaries. The quick brown fox jumps over the lazy dog. Sphinx of black quartz, judge my vow. Pack my box with five dozen liquor jugs.

*Scroll with mouse wheel or arrow keys.*"#;

const DOC_CHANGES: &str = r#"# Changelog

## v0.3.0 — Current

- **Mouse scroll**: scroll content with mouse wheel
- **Document switching**: press `Tab` or `1`–`3` to switch documents
- Improved word wrapping accuracy

## v0.2.0

- Added `*italic*` and `_italic_` support
- Added `[link](url)` rendering with underline style
- Fixed header spacing after paragraphs

## v0.1.0

- Initial release
- `#` headers (H1–H6)
- `**bold**` text
- `` `inline code` `` blocks
- `- ` unordered lists
- Word wrapping with `unicode-width`

*Press Tab to cycle through documents.*"#;

const DOCS: [&str; 3] = [DOC_FEATURES, DOC_GUIDE, DOC_CHANGES];
const DOC_TITLES: [&str; 3] = ["Features", "Guide", "Changelog"];

struct RichTextApp {
    id: WidgetId,
    area: Rect,
    theme: Theme,
    rich_text: RichText,
    show_help: bool,
    dirty: bool,
    keybindings: KeybindingSet,
    should_quit: Rc<AtomicBool>,
    scroll_offset: u16,
    content_height: Cell<u16>,
    doc_index: usize,
}

impl RichTextApp {
    fn new(theme: Theme, should_quit: Rc<AtomicBool>) -> Self {
        let rich_text = RichText::with_id(WidgetId::new(1), DOCS[0]).with_theme(theme.clone());
        Self {
            id: WidgetId::new(0),
            area: Rect::default(),
            theme,
            rich_text,
            show_help: false,
            dirty: true,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            should_quit,
            scroll_offset: 0,
            content_height: Cell::new(0),
            doc_index: 0,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.rich_text.on_theme_change(&self.theme);
        self.dirty = true;
    }

    fn switch_doc(&mut self, idx: usize) {
        if idx >= DOCS.len() || idx == self.doc_index {
            return;
        }
        self.doc_index = idx;
        self.scroll_offset = 0;
        self.rich_text.set_content(DOCS[idx]);
        self.dirty = true;
    }

    fn visible_height(&self) -> u16 {
        self.area.height.saturating_sub(5)
    }

    fn max_scroll(&self) -> u16 {
        self.content_height.get().saturating_sub(self.visible_height())
    }

    fn scroll_up(&mut self, n: u16) {
        let new = self.scroll_offset.saturating_sub(n);
        if new != self.scroll_offset {
            self.scroll_offset = new;
            self.dirty = true;
        }
    }

    fn scroll_down(&mut self, n: u16) {
        let new = (self.scroll_offset + n).min(self.max_scroll());
        if new != self.scroll_offset {
            self.scroll_offset = new;
            self.dirty = true;
        }
    }
}

impl Widget for RichTextApp {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; self.dirty = true; }
    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
    fn focusable(&self) -> bool { true }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        let t = &self.theme;

        let mut tab_x = 1u16;
        for (i, title) in DOC_TITLES.iter().enumerate() {
            let is_active = i == self.doc_index;
            let fg = if is_active { t.primary } else { t.fg_muted };
            let bg = if is_active { t.selection_bg } else { t.bg };
            let style = if is_active { Styles::BOLD } else { Styles::empty() };
            let label = format!(" {} ", title);
            for (j, c) in label.chars().enumerate() {
                let idx = (tab_x + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].style = style;
                    plane.cells[idx].transparent = false;
                }
            }
            tab_x += label.len() as u16 + 2;
        }
        for i in 1..DOC_TITLES.len() {
            let num_label = format!("{}", i + 1);
            for (j, c) in num_label.chars().enumerate() {
                let idx = (tab_x + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg_muted;
                    plane.cells[idx].transparent = false;
                }
            }
            tab_x += num_label.len() as u16 + 2;
        }

        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        let content_w = area.width.saturating_sub(4);
        let visible_h = area.height.saturating_sub(5);
        let virtual_h = 500u16;
        let rt_area = Rect::new(0, 0, content_w, virtual_h);
        let rt_plane = self.rich_text.render(rt_area);

        let mut last_row = 0u16;
        for row in (0..virtual_h as usize).rev() {
            let row_start = row * content_w as usize;
            let row_end = row_start + content_w as usize;
            if row_end > rt_plane.cells.len() { continue; }
            for col in 0..content_w as usize {
                let cell = &rt_plane.cells[row_start + col];
                if cell.char != '\0' && !cell.transparent {
                    last_row = (row + 1) as u16;
                    break;
                }
            }
            if last_row > 0 { break; }
        }
        self.content_height.set(last_row);

        let content_y = 2u16;
        for row in 0..visible_h {
            let src_row = self.scroll_offset as usize + row as usize;
            if src_row >= virtual_h as usize { break; }
            for col in 0..content_w as usize {
                let src_idx = src_row * content_w as usize + col;
                if src_idx >= rt_plane.cells.len() { break; }
                let cell = &rt_plane.cells[src_idx];
                if cell.transparent { continue; }
                let abs_x = 2 + col as u16;
                let abs_y = content_y + row;
                let dest_idx = (abs_y * area.width + abs_x) as usize;
                if dest_idx < plane.cells.len() {
                    plane.cells[dest_idx] = *cell;
                }
            }
        }

        let content_h = self.content_height.get();
        if content_h > visible_h {
            let sb_x = area.width.saturating_sub(2);
            let thumb_h = (visible_h as f32 / content_h as f32 * visible_h as f32).max(1.0) as u16;
            let max_off = content_h.saturating_sub(visible_h).max(1);
            let thumb_y = (self.scroll_offset as f32 / max_off as f32
                * (visible_h.saturating_sub(thumb_h)) as f32) as u16 + content_y;
            for i in 0..thumb_h {
                let y = thumb_y + i;
                if y >= content_y && y < content_y + visible_h {
                    let idx = (y * area.width + sb_x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '▐';
                        plane.cells[idx].fg = t.primary;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }

        let kb_quit = self.keybindings.display(actions::QUIT).unwrap_or("Ctrl+Q");
        let kb_help = self.keybindings.display(actions::HELP).unwrap_or("F1");
        let kb_theme = self.keybindings.display(actions::THEME).unwrap_or("Ctrl+T");
        let status = format!("Tab/1-3: switch | {kb_theme}: theme | {kb_help}: help | {kb_quit}: quit");
        for (i, c) in status.chars().enumerate() {
            let idx = ((area.height - 1) * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
                plane.cells[idx].bg = self.theme.surface;
                plane.cells[idx].transparent = false;
            }
        }

        if self.show_help {
            let hw = 44u16.min(area.width.saturating_sub(4));
            let hh = 13u16.min(area.height.saturating_sub(4));
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
            let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = *ch; plane.cells[idx].fg = t.outline; }
            }
            for x in hx + 1..hx + hw - 1 {
                let top = (hy * area.width + x) as usize;
                let bot = ((hy + hh - 1) * area.width + x) as usize;
                if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
                if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
            }
            for y in hy + 1..hy + hh - 1 {
                let left = (y * area.width + hx) as usize;
                let right = (y * area.width + hx + hw - 1) as usize;
                if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
                if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
            }
            let title = "RichText Help";
            let tx = hx + (hw - title.len() as u16) / 2;
            for (i, c) in title.chars().enumerate() {
                let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; plane.cells[idx].style = Styles::BOLD; }
            }
            let kb_back = self.keybindings.display(actions::BACK).unwrap_or("Esc");
            let shortcuts: [(&str, &str); 7] = [
                ("^/v/PgUp/PgDn", "Scroll content"),
                ("Scroll wheel", "Scroll content"),
                ("Tab", "Cycle documents"),
                ("1/2/3", "Jump to document"),
                (kb_theme, "Cycle theme"),
                (kb_help, "Toggle help"),
                (kb_back, "Dismiss help"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * area.width + hx + 2 + j as u16) as usize;
                    if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * area.width + hx + 18 + j as u16) as usize;
                    if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.fg; }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        if self.keybindings.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return false;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            self.cycle_theme();
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return true;
        }
        match key.code {
            KeyCode::Tab => {
                self.switch_doc((self.doc_index + 1) % DOCS.len());
                true
            }
            KeyCode::Char('1') if key.modifiers.is_empty() => { self.switch_doc(0); true }
            KeyCode::Char('2') if key.modifiers.is_empty() => { self.switch_doc(1); true }
            KeyCode::Char('3') if key.modifiers.is_empty() => { self.switch_doc(2); true }
            KeyCode::Up => { self.scroll_up(1); true }
            KeyCode::Down => { self.scroll_down(1); true }
            KeyCode::PageUp => { self.scroll_up(self.visible_height().saturating_sub(1).max(1)); true }
            KeyCode::PageDown => { self.scroll_down(self.visible_height().saturating_sub(1).max(1)); true }
            KeyCode::Home => { self.scroll_offset = 0; self.dirty = true; true }
            KeyCode::End => { self.scroll_offset = self.max_scroll(); self.dirty = true; true }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        match kind {
            MouseEventKind::ScrollUp => { self.scroll_up(3); true }
            MouseEventKind::ScrollDown => { self.scroll_down(3); true }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.rich_text.on_theme_change(theme);
        self.dirty = true;
    }

    fn current_theme(&self) -> Option<Theme> {
        Some(self.theme.clone())
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Rc::new(AtomicBool::new(false));
    let quit_check = Rc::clone(&should_quit);
    let theme = Theme::from_env_or(Theme::nord());
    let app = RichTextApp::new(theme.clone(), should_quit);

    let mut a = App::new()?.title("Rich Text Demo").fps(30).theme(theme);
    a.add_widget(Box::new(app), Rect::new(0, 0, 80, 24));
    a.run(move |ctx| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
        }
    })
}

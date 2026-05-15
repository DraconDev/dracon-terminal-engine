use dracon_terminal_engine::compositor::{Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind};
use ratatui::layout::Rect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::rc::Rc;

struct RichTextApp {
    id: WidgetId,
    area: Rect,
    theme: Theme,
    rich_text: RichText,
    show_help: bool,
    dirty: bool,
    keybindings: KeybindingSet,
    should_quit: Rc<AtomicBool>,
}

impl RichTextApp {
    fn new(theme: Theme, should_quit: Rc<AtomicBool>) -> Self {
        let markdown = r#"# Rich Text / Markdown Demo

This widget renders **styled text** with *minimal* Markdown support.

## Features

- **Headers** — like the one above
- **Bold text** — using double asterisks
- *Italic text* — using single asterisks or underscores
- `Inline code` — using backticks
- [Links](https://github.com) — click-supported (OSC 8)
- List items — using dashes

## Example Paragraph

The quick brown fox jumps over the lazy dog. This is a longer paragraph that demonstrates word wrapping within the widget boundaries. The renderer uses `unicode-width` for accurate character measurement and breaks at word boundaries when possible.

## Code Example

You can write `let x = 42;` inline or refer to `functions` and `types`.

*Try cycling themes with Ctrl+T to see how colors adapt.*
"#;
        let rich_text = RichText::with_id(WidgetId::new(1), markdown).with_theme(theme.clone());
        Self {
            id: WidgetId::new(0),
            area: Rect::default(),
            theme,
            rich_text,
            show_help: false,
            dirty: true,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            should_quit,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.rich_text.on_theme_change(&self.theme);
        self.dirty = true;
    }
}

impl Widget for RichTextApp {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; }
    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
    fn focusable(&self) -> bool { true }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        let rt_area = Rect::new(2, 1, area.width.saturating_sub(4), area.height.saturating_sub(4));
        let rt_plane = self.rich_text.render(rt_area);
        for (i, cell) in rt_plane.cells.iter().enumerate() {
            if cell.transparent { continue; }
            let local_y = i / rt_plane.width as usize;
            let local_x = i % rt_plane.width as usize;
            let abs_x = rt_area.x + local_x as u16;
            let abs_y = rt_area.y + local_y as u16;
            let dest_idx = (abs_y * area.width + abs_x) as usize;
            if dest_idx < plane.cells.len() {
                plane.cells[dest_idx] = *cell;
            }
        }

        let kb_quit = self.keybindings.display(actions::QUIT).unwrap_or("Ctrl+Q");
        let kb_help = self.keybindings.display(actions::HELP).unwrap_or("F1");
        let kb_theme = self.keybindings.display(actions::THEME).unwrap_or("Ctrl+T");
        let status = format!("{kb_theme}: theme | {kb_help}: help | {kb_quit}: quit");
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
            let t = &self.theme;
            let hw = 40u16.min(area.width.saturating_sub(4));
            let hh = 10u16.min(area.height.saturating_sub(4));
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
            let shortcuts: [(&str, &str); 5] = [
                ("↑/↓/PgUp/PgDn", "Scroll content"),
                (kb_theme, "Cycle theme"),
                (kb_help, "Toggle help"),
                (kb_back, "Dismiss help"),
                (kb_quit, "Quit app"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * area.width + hx + 2 + j as u16) as usize;
                    if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * area.width + hx + 16 + j as u16) as usize;
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
        self.rich_text.handle_key(key)
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.rich_text.on_theme_change(theme);
        self.dirty = true;
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

#![allow(missing_docs)]
//! Scrollable Content — Demonstrates ScrollContainer/ScrollState with keyboard+mouse scroll.
//!
//! A simulated server log viewer with 200 pre-populated entries,
//! proportional scrollbar, search/filter, and keyboard/mouse scroll.

use std::cell::RefCell;
use std::io::Result;
use std::os::fd::AsFd;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scroll::ScrollContainer;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use ratatui::layout::Rect;

const LOG_LEVELS: &[&str] = &["INFO", "WARN", "ERROR", "DEBUG", "TRACE"];
const LOG_MESSAGES: &[&str] = &[
    "Connection established to database",
    "Cache miss for session token",
    "Request processed in 42ms",
    "Memory usage at 78%",
    "Worker pool expanded to 8 threads",
    "File upload completed: report.pdf",
    "DNS resolution timeout for api.example.com",
    "SSL certificate renewed successfully",
    "Rate limit exceeded for client 10.0.1.5",
    "Background job completed: email-batch-47",
    "Configuration reloaded from /etc/app.toml",
    "Health check passed: all services green",
    "Disk usage warning: /var/log at 92%",
    "WebSocket connection closed by client",
    "Migration v42 applied successfully",
    "Query plan cache invalidated",
    "Backup completed: 2.3GB in 45s",
    "Circuit breaker opened for payments-api",
    "Graceful shutdown initiated",
    "Thread panic recovered in handler",
];

struct ScrollableContent {
    id: WidgetId,
    area: Rect,
    theme: Theme,
    dirty: bool,
    scroll: ScrollContainer,
    lines: Vec<(String, Color)>,
    show_help: bool,
    keybindings: KeybindingSet,
    hovered_line: Option<usize>,
    search_query: String,
    search_active: bool,
}

impl ScrollableContent {
    fn new(theme: Theme) -> Self {
        let mut lines = Vec::new();
        let mut rng_state: u64 = 42;
        for i in 0..200u32 {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let lvl_idx = (rng_state >> 32) as usize % LOG_LEVELS.len();
            let msg_idx = (rng_state >> 16) as usize % LOG_MESSAGES.len();
            let lvl = LOG_LEVELS[lvl_idx];
            let msg = LOG_MESSAGES[msg_idx];
            let ts = format_time(i);
            let line = format!("[{}] {:>5} ┃ {}", ts, lvl, msg);
            let color = level_color(lvl, &theme);
            lines.push((line, color));
        }
        let scroll = ScrollContainer::new()
            .with_content_height(lines.len())
            .with_viewport_height(20);
        Self {
            id: WidgetId::new(1),
            area: Rect::new(0, 0, 80, 24),
            theme,
            dirty: true,
            scroll,
            lines,
            show_help: false,
            keybindings: KeybindingSet::default(),
            hovered_line: None,
            search_query: String::new(),
            search_active: false,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        for (line, color) in self.lines.iter_mut() {
            let lvl = extract_level(line);
            *color = level_color(lvl, &self.theme);
        }
        self.dirty = true;
    }

    fn render_help_overlay(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let w = 44u16.min(area.width.saturating_sub(4));
        let h = 14u16.min(area.height.saturating_sub(4));
        let hx = (area.width - w) / 2;
        let hy = (area.height - h) / 2;

        for y in hy..hy + h {
            for x in hx..hx + w {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        let corners = [
            ('╭', hx, hy),
            ('╮', hx + w - 1, hy),
            ('╰', hx, hy + h - 1),
            ('╯', hx + w - 1, hy + h - 1),
        ];
        for (ch, cx, cy) in &corners {
            let idx = (*cy * area.width + *cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = *ch;
                plane.cells[idx].fg = t.outline;
            }
        }
        for x in hx + 1..hx + w - 1 {
            let top = (hy * area.width + x) as usize;
            let bot = ((hy + h - 1) * area.width + x) as usize;
            if top < plane.cells.len() {
                plane.cells[top].char = '─';
                plane.cells[top].fg = t.outline;
            }
            if bot < plane.cells.len() {
                plane.cells[bot].char = '─';
                plane.cells[bot].fg = t.outline;
            }
        }
        for y in hy + 1..hy + h - 1 {
            let left = (y * area.width + hx) as usize;
            let right = (y * area.width + hx + w - 1) as usize;
            if left < plane.cells.len() {
                plane.cells[left].char = '│';
                plane.cells[left].fg = t.outline;
            }
            if right < plane.cells.len() {
                plane.cells[right].char = '│';
                plane.cells[right].fg = t.outline;
            }
        }

        let title = "Scrollable Content Help";
        let tx = hx + (w - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let shortcuts = [
            ("↑/↓", "Scroll line"),
            ("PgUp/PgDn", "Scroll page"),
            ("Home/End", "Scroll to top/bottom"),
            ("Ctrl+F", "Search mode"),
            ("Ctrl+T", "Cycle theme"),
            ("F1", "Toggle help"),
            ("Esc", "Dismiss / exit search"),
            ("Ctrl+Q", "Quit"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            for (j, c) in key.chars().enumerate() {
                let idx = (row * area.width + hx + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                }
            }
            for (j, c) in desc.chars().enumerate() {
                let idx = (row * area.width + hx + 16 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                }
            }
        }
    }
}

fn format_time(idx: u32) -> String {
    let base_s = idx as u64 * 3;
    let h = (base_s / 3600) % 24;
    let m = (base_s / 60) % 60;
    let s = base_s % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn level_color(lvl: &str, theme: &Theme) -> Color {
    match lvl {
        "ERROR" => theme.error,
        "WARN" => theme.warning,
        "INFO" => theme.info,
        "DEBUG" => theme.fg_muted,
        "TRACE" => theme.fg_subtle,
        _ => theme.fg,
    }
}

fn extract_level(line: &str) -> &str {
    if let Some(start) = line.find(']') {
        let after = &line[start + 2..];
        let end = after.find(' ').unwrap_or(after.len());
        &after[..end]
    } else {
        ""
    }
}

impl Widget for ScrollableContent {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn area(&self) -> Rect {
        self.area
    }
    fn set_area(&mut self, area: Rect) {
        self.area = area;
        let content_h = area.height.saturating_sub(4) as usize;
        self.scroll.state_mut().viewport_height = content_h;
        self.scroll.state_mut().content_height = self.lines.len();
        self.dirty = true;
    }
    fn needs_render(&self) -> bool {
        self.dirty
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        for (line, color) in self.lines.iter_mut() {
            let lvl = extract_level(line);
            *color = level_color(lvl, &self.theme);
        }
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(t.bg);

        let w = area.width as usize;
        let h = area.height as usize;

        plane.cells[0].char = '╭';
        plane.cells[0].fg = t.outline;
        plane.cells[w - 1].char = '╮';
        plane.cells[w - 1].fg = t.outline;
        plane.cells[(h - 1) * w].char = '╰';
        plane.cells[(h - 1) * w].fg = t.outline;
        plane.cells[(h - 1) * w + w - 1].char = '╯';
        plane.cells[(h - 1) * w + w - 1].fg = t.outline;
        for x in 1..w - 1 {
            plane.cells[x].char = '─';
            plane.cells[x].fg = t.outline;
            plane.cells[(h - 1) * w + x].char = '─';
            plane.cells[(h - 1) * w + x].fg = t.outline;
        }
        for y in 1..h - 1 {
            plane.cells[y * w].char = '│';
            plane.cells[y * w].fg = t.outline;
            plane.cells[y * w + w - 1].char = '│';
            plane.cells[y * w + w - 1].fg = t.outline;
        }

        let header = " 📜 Scrollable Content ";
        for (i, c) in header.chars().enumerate().take(w - 4) {
            let idx = 1 + i;
            if idx < w - 1 {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: t.fg_on_accent,
                    bg: t.primary,
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }
        for x in (1 + header.len()).min(w - 2)..w - 1 {
            plane.cells[x] = Cell {
                char: '─',
                fg: t.primary,
                bg: t.primary,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }

        let content_top = 2;
        let content_h = h.saturating_sub(4);
        let sb_x = w.saturating_sub(3);

        for y in content_top..content_top + content_h {
            for x in 1..w - 1 {
                let idx = y * w + x;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface;
                }
            }
        }

        let offset = self.scroll.state().offset;
        let view_h = content_h;
        for row in 0..view_h {
            let line_idx = offset + row;
            if line_idx >= self.lines.len() {
                break;
            }
            let (line, color) = &self.lines[line_idx];
            let is_hovered = self.hovered_line == Some(line_idx);
            let is_search_match = if self.search_query.is_empty() {
                false
            } else {
                line.to_lowercase().contains(&self.search_query.to_lowercase())
            };
            let y = content_top + row;
            let max_chars = sb_x.saturating_sub(2);
            for (j, c) in line.chars().enumerate().take(max_chars) {
                let idx = y * w + 1 + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if is_search_match {
                        t.warning
                    } else {
                        *color
                    };
                    if is_hovered {
                        plane.cells[idx].bg = t.hover_bg;
                    }
                }
            }
            if is_hovered {
                for x in 1..sb_x {
                    let idx = y * w + x;
                    if idx < plane.cells.len() && plane.cells[idx].char == '\0' {
                        plane.cells[idx].bg = t.hover_bg;
                    }
                }
            }
        }

        let sb_area = Rect::new(sb_x as u16, content_top as u16, 1, content_h as u16);
        let sb_plane = self.scroll.render_scrollbar(sb_area);
        for (i, c) in sb_plane.cells.iter().enumerate() {
            if c.transparent {
                continue;
            }
            let row = i / sb_plane.width as usize;
            let col = i % sb_plane.width as usize;
            let target_y = content_top + row;
            let target_x = sb_x + col;
            if target_y < h - 1 && target_x < w - 1 {
                let idx = target_y * w + target_x;
                if idx < plane.cells.len() {
                    plane.cells[idx] = *c;
                    plane.cells[idx].bg = t.surface;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        let offset_display = self.scroll.state().offset;
        let total = self.lines.len();
        let status = format!(
            " Lines {}-{}/{}  |  {}: search | {}: theme | {}: help | {}: quit",
            offset_display + 1,
            (offset_display + view_h).min(total),
            total,
            self.keybindings.display(actions::SEARCH).unwrap_or("ctrl+f"),
            self.keybindings.display(actions::THEME).unwrap_or("ctrl+t"),
            self.keybindings.display(actions::HELP).unwrap_or("f1"),
            self.keybindings.display(actions::QUIT).unwrap_or("ctrl+q"),
        );
        for (i, c) in status.chars().enumerate().take(w - 2) {
            let idx = (h - 1) * w + 1 + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: t.fg_muted,
                    bg: t.surface_elevated,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
        for x in 1..w - 1 {
            plane.cells[(h - 1) * w + x].bg = t.surface_elevated;
        }

        if self.search_active {
            let search_bar = format!(" Search: {}█", self.search_query);
            let sb_y = h - 2;
            for (i, c) in search_bar.chars().enumerate().take(w - 4) {
                let idx = sb_y * w + 2 + i;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: t.fg,
                        bg: t.surface_elevated,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        if self.show_help {
            self.render_help_overlay(&mut plane, area);
        }

        plane
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.search_active {
            match key.code {
                KeyCode::Esc => {
                    self.search_active = false;
                    self.search_query.clear();
                    self.dirty = true;
                    return true;
                }
                KeyCode::Enter => {
                    self.search_active = false;
                    self.dirty = true;
                    return true;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.dirty = true;
                    return true;
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.dirty = true;
                    return true;
                }
                _ => return false,
            }
        }

        if self.show_help {
            return false;
        }

        self.scroll.handle_key(key)
    }

    fn handle_mouse(&mut self, kind: dracon_terminal_engine::input::event::MouseEventKind, col: u16, row: u16) -> bool {
        use dracon_terminal_engine::input::event::MouseEventKind;

        let content_top = 2u16;
        let content_h = self.area.height.saturating_sub(4);

        if row >= content_top && row < content_top + content_h && col >= 1 && col < self.area.width.saturating_sub(2) {
            match kind {
                MouseEventKind::Moved => {
                    let rel_row = row.saturating_sub(content_top) as usize;
                    let line_idx = self.scroll.state().offset + rel_row;
                    if line_idx < self.lines.len() && self.hovered_line != Some(line_idx) {
                        self.hovered_line = Some(line_idx);
                        self.dirty = true;
                    }
                    true
                }
                MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                    self.scroll.handle_mouse(kind, col, row)
                }
                _ => true,
            }
        } else {
            if self.hovered_line.is_some() {
                self.hovered_line = None;
                self.dirty = true;
            }
            self.scroll.handle_mouse(kind, col, row)
        }
    }
}

struct InputRouter {
    target: Rc<RefCell<ScrollableContent>>,
    id: WidgetId,
    area: Rect,
}

impl Widget for InputRouter {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn area(&self) -> Rect {
        self.area
    }
    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        false
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }
    fn render(&self, _area: Rect) -> Plane {
        Plane::new(0, 0, 0)
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        self.target.borrow_mut().handle_key(key)
    }

    fn handle_mouse(&mut self, kind: dracon_terminal_engine::input::event::MouseEventKind, col: u16, row: u16) -> bool {
        self.target.borrow_mut().handle_mouse(kind, col, row)
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.target.borrow_mut().on_theme_change(theme);
    }
    fn current_theme(&self) -> Option<Theme> {
        Some(self.target.borrow().theme.clone())
    }
}

fn main() -> Result<()> {
    std::thread::sleep(std::time::Duration::from_millis(300));

    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let keybindings = KeybindingSet::from_config(&resolve_keybindings());
    let kb_input = keybindings.clone();
    let env_theme = Theme::from_env_or(Theme::nord());

    let content = Rc::new(RefCell::new(ScrollableContent::new(env_theme.clone())));
    content.borrow_mut().keybindings = keybindings;
    let content_tick = Rc::clone(&content);
    let content_router = Rc::clone(&content);
    let content_input = Rc::clone(&content);

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app = App::new()?
        .title("Scrollable Content")
        .fps(30)
        .theme(env_theme.clone());

    let router = InputRouter {
        target: content_router,
        id: WidgetId::new(100),
        area: Rect::new(0, 0, w, h),
    };
    app.add_widget(Box::new(router), Rect::new(0, 0, w, h));

    app.on_input(move |key| {
            use dracon_terminal_engine::input::event::KeyEventKind;
            if key.kind != KeyEventKind::Press {
                return false;
            }
            let mut c = content_input.borrow_mut();
            if kb_input.matches(actions::QUIT, &key) {
                should_quit.store(true, Ordering::SeqCst);
                true
            } else if kb_input.matches(actions::THEME, &key) {
                c.cycle_theme();
                true
            } else if kb_input.matches(actions::HELP, &key) {
                c.show_help = !c.show_help;
                c.dirty = true;
                true
            } else if c.show_help && kb_input.matches(actions::BACK, &key) {
                c.show_help = false;
                c.dirty = true;
                true
            } else if kb_input.matches(actions::SEARCH, &key) && !c.search_active {
                c.search_active = true;
                c.search_query.clear();
                c.dirty = true;
                true
            } else if c.search_active {
                c.handle_key(key)
            } else {
                false
            }
        })
        .on_tick(move |ctx, _tick| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
                return;
            }
            let mut c = content_tick.borrow_mut();
            let (w, h) = ctx.compositor().size();
            if c.area.width != w || c.area.height != h {
                c.set_area(Rect::new(0, 0, w, h));
            }
            if c.needs_render() {
                ctx.add_plane(c.render(c.area));
                c.clear_dirty();
            }
        })
        .run(|_| {})
}

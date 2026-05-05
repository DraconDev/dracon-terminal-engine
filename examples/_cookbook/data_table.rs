#![allow(missing_docs)]
//! Data Table Demo — sortable table with SearchInput filter.
//!
//! **Layout:**
//! ```text
//! ┌────────────────────────────────────────────────────┐
//! │ Filter: [__]   Sort: [Name▼]                       │
//! ├────────────────────────────────────────────────────┤
//! │ Name        │ Age │ City      │ Profession          │
//! ├─────────────┼─────┼───────────┼────────────────────┤
//! │ > Alice     │  28 │ New York  │ Software Engineer  │ ← selected
//! │   Bob       │  34 │ London    │ Data Scientist     │
//! ├────────────────────────────────────────────────────┤
//! │ Selected: Alice | Age: 28 | City: New York | 5 rows │
//! └────────────────────────────────────────────────────┘
//! ```
//!
//! **Behavior:** Type to filter • Click Sort to cycle none→asc→desc • Up/Down navigate.

use dracon_terminal_engine::compositor::{Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::SearchInput;
use ratatui::layout::Rect;
use std::os::fd::AsFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const DATA: &[(&str, u32, &str, &str)] = &[
    ("Alice", 28, "New York", "Software Engineer"),
    ("Bob", 34, "London", "Data Scientist"),
    ("Carol", 22, "Tokyo", "Product Manager"),
    ("David", 31, "Berlin", "DevOps Engineer"),
    ("Eve", 29, "Sydney", "UX Designer"),
    ("Frank", 45, "Toronto", "Engineering Manager"),
    ("Grace", 27, "Singapore", "Frontend Developer"),
    ("Heidi", 33, "Paris", "Backend Developer"),
    ("Ivan", 41, "Amsterdam", "CTO"),
    ("Judy", 26, "Seoul", "Mobile Developer"),
];

#[derive(Clone)]
struct Person(String, u32, String, String);

impl Person {
    fn matches(&self, q: &str) -> bool {
        let q = q.to_lowercase();
        self.0.to_lowercase().contains(&q)
            || self.2.to_lowercase().contains(&q)
            || self.3.to_lowercase().contains(&q)
            || self.1.to_string().contains(&q)
    }
}

enum Sort {
    None,
    Asc,
    Desc,
}
impl Sort {
    fn next(&self) -> Self {
        match self {
            Sort::None => Sort::Asc,
            Sort::Asc => Sort::Desc,
            Sort::Desc => Sort::None,
        }
    }
    fn sym(&self) -> &'static str {
        match self {
            Sort::None => "",
            Sort::Asc => "▲",
            Sort::Desc => "▼",
        }
    }
}

struct Table {
    id: WidgetId,
    all: Vec<Person>,
    rows: Vec<Person>,
    sel: usize,
    off: usize,
    vis: usize,
    sort: Sort,
    search: SearchInput,
    theme: Theme,
    area: Rect,
    dirty: bool,
    show_help: bool,
}

impl Table {
    fn new() -> Self {
        let all: Vec<Person> = DATA
            .iter()
            .map(|(n, a, c, p)| Person(n.to_string(), *a, c.to_string(), p.to_string()))
            .collect();
        Self {
            id: WidgetId::new(0),
            all: all.clone(),
            rows: all,
            sel: 0,
            off: 0,
            vis: 8,
            sort: Sort::None,
            search: SearchInput::new(WidgetId::new(1)),
            theme: Theme::cyberpunk(),
            area: Rect::new(0, 0, 80, 20),
            dirty: true,
            show_help: false,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::nord(),
            Theme::cyberpunk(),
            Theme::dracula(),
            Theme::gruvbox_dark(),
            Theme::tokyo_night(),
        ];
        let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];
        self.search.on_theme_change(&self.theme);
        self.dirty = true;
    }

    fn filter(&mut self, q: &str) {
        self.rows = if q.is_empty() {
            self.all.clone()
        } else {
            self.all.iter().filter(|p| p.matches(q)).cloned().collect()
        };
        self.sort_rows();
        self.sel = 0;
        self.off = 0;
        self.dirty = true;
    }

    fn sort_rows(&mut self) {
        match self.sort {
            Sort::None => {}
            Sort::Asc => self.rows.sort_by(|a, b| a.0.cmp(&b.0)),
            Sort::Desc => self.rows.sort_by(|a, b| b.0.cmp(&a.0)),
        }
    }
}

impl Widget for Table {
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
        self.dirty = true;
    }
    fn z_index(&self) -> u16 {
        10
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

    fn render(&self, area: Rect) -> Plane {
        let mut p = Plane::new(0, area.width, area.height);
        p.z_index = 10;
        let (heads, widths, hh, _sh) = (
            ["󰣉 Name", "󰢮 Age", "󰉋 City", "󰠨 Profession"],
            [12u16, 5, 11, 16],
            3u16,
            1u16,
        );
        let inner_y = 1u16; // content starts after top border
        let _inner_h = area.height.saturating_sub(2); // exclude top/bottom borders

        // Background
        for y in 0..area.height {
            for x in 0..area.width {
                let idx = (y * area.width + x) as usize;
                if idx < p.cells.len() {
                    p.cells[idx].bg = self.theme.bg;
                    p.cells[idx].fg = self.theme.fg;
                }
            }
        }

        // Rounded border
        let bw = area.width;
        let bh = area.height;
        if bw > 0 && bh > 0 {
            let corners = [('╭', 0, 0), ('╮', bw - 1, 0), ('╰', 0, bh - 1), ('╯', bw - 1, bh - 1)];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < p.cells.len() {
                    p.cells[idx].char = *ch;
                    p.cells[idx].fg = self.theme.outline;
                }
            }
            for x in 1..bw.saturating_sub(1) {
                let top_idx = x as usize;
                let bot_idx = ((bh - 1) * area.width + x) as usize;
                if top_idx < p.cells.len() { p.cells[top_idx].char = '─'; p.cells[top_idx].fg = self.theme.outline; }
                if bot_idx < p.cells.len() { p.cells[bot_idx].char = '─'; p.cells[bot_idx].fg = self.theme.outline; }
            }
            for y in 1..bh.saturating_sub(1) {
                let left_idx = (y * area.width) as usize;
                let right_idx = (y * area.width + bw - 1) as usize;
                if left_idx < p.cells.len() { p.cells[left_idx].char = '│'; p.cells[left_idx].fg = self.theme.outline; }
                if right_idx < p.cells.len() { p.cells[right_idx].char = '│'; p.cells[right_idx].fg = self.theme.outline; }
            }
        }

        // Render search input (offset by 1 for top border)
        let search_area = Rect::new(1, inner_y, 20, 1);
        let search_plane = self.search.render(search_area);
        for y in 0..search_plane.height {
            for x in 0..search_plane.width {
                let src_idx = (y * search_plane.width + x) as usize;
                if search_plane.cells[src_idx].transparent {
                    continue;
                }
                let dst_idx = ((y as u16 + inner_y) * area.width + (x as u16 + 1)) as usize;
                if src_idx < search_plane.cells.len() && dst_idx < p.cells.len() {
                    p.cells[dst_idx] = search_plane.cells[src_idx].clone();
                }
            }
        }

        let lbl = format!("Sort: Name{}", self.sort.sym());
        let sx = area.width.saturating_sub(lbl.len() as u16 + 3);
        for (i, c) in lbl.chars().enumerate() {
            let idx = (inner_y * area.width + sx + i as u16) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = c;
                p.cells[idx].fg = self.theme.fg;
            }
        }

        // Separator line (y = inner_y + 1 = 2)
        let sep_y = inner_y + 1;
        for x in 1..area.width.saturating_sub(1) {
            let idx = (sep_y * area.width + x) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = '─';
                p.cells[idx].fg = self.theme.outline;
                p.cells[idx].transparent = false;
            }
        }

        let mut x = 1u16; // offset by 1 for left border
        for (h, w) in heads.iter().zip(widths.iter()) {
            let w = *w.min(&area.width.saturating_sub(x + 1));
            for (j, c) in h.chars().take(w as usize).enumerate() {
                let idx = (hh * area.width + x + j as u16) as usize;
                if idx < p.cells.len() {
                    p.cells[idx].char = c;
                    p.cells[idx].fg = self.theme.primary;
                    p.cells[idx].style = Styles::BOLD;
                }
            }
            x += w + 1;
        }

        let data_start_y = hh + 1;
        for (i, row) in self.rows.iter().skip(self.off).take(self.vis).enumerate() {
            let y = data_start_y + i as u16;
            if y >= area.height.saturating_sub(1) { break; }
            let sel = self.off + i == self.sel;
            let bg = if sel {
                self.theme.selection_bg
            } else {
                self.theme.bg
            };
            let fg = if sel {
                self.theme.selection_fg
            } else {
                self.theme.fg
            };
            let sty = if sel { Styles::BOLD } else { Styles::empty() };
            for x in 1..area.width.saturating_sub(1) {
                let idx = (y * area.width + x) as usize;
                if idx < p.cells.len() {
                    p.cells[idx].bg = bg;
                    p.cells[idx].fg = fg;
                    p.cells[idx].transparent = false;
                }
            }

            let vals = [&row.0, &row.1.to_string(), &row.2, &row.3];
            let mut x = 1u16;
            for (j, (v, w)) in vals.iter().zip(widths.iter()).enumerate() {
                let w = *w.min(&area.width.saturating_sub(x + 1));
                let txt = if j == 1 {
                    format!("{:>3}", v)
                } else {
                    v.chars().take(w as usize - 1).collect()
                };
                let pre = if sel && j == 0 { "> " } else { " " };
                for (k, c) in pre.chars().chain(txt.chars()).take(w as usize).enumerate() {
                    let idx = (y * area.width + x + k as u16) as usize;
                    if idx < p.cells.len() {
                        p.cells[idx].char = c;
                        p.cells[idx].fg = fg;
                        p.cells[idx].style = sty;
                    }
                }
                x += w + 1;
            }
        }

        let sy = area.height.saturating_sub(2);
        let txt = if let Some(r) = self.rows.get(self.sel) {
            format!(
                " Selected: {} | Age: {} | City: {} | {} rows ",
                r.0,
                r.1,
                r.2,
                self.rows.len()
            )
        } else {
            format!(" No results | {} rows ", self.rows.len())
        };
        let txt_x = (area.width.saturating_sub(txt.len() as u16)) / 2;
        for (i, c) in txt.chars().take(area.width.saturating_sub(2) as usize).enumerate() {
            let idx = (sy * area.width + txt_x + i as u16) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = c;
                p.cells[idx].fg = self.theme.primary;
                p.cells[idx].transparent = false;
            }
        }
        // Help overlay
        if self.show_help {
            let hw = 40u16.min(area.width.saturating_sub(4));
            let hh = 10u16.min(area.height.saturating_sub(4));
            let hx = (area.width - hw) / 2;
            let hy = (area.height - hh) / 2;
            for y in hy..hy + hh {
                for x in hx..hx + hw {
                    let idx = (y * area.width + x) as usize;
                    if idx < p.cells.len() {
                        p.cells[idx].bg = self.theme.surface_elevated;
                        p.cells[idx].transparent = false;
                    }
                }
            }
            // Border
            for x in hx..hx + hw {
                let top_idx = (hy * area.width + x) as usize;
                let bot_idx = ((hy + hh - 1) * area.width + x) as usize;
                if top_idx < p.cells.len() { p.cells[top_idx].char = '─'; p.cells[top_idx].fg = self.theme.outline; }
                if bot_idx < p.cells.len() { p.cells[bot_idx].char = '─'; p.cells[bot_idx].fg = self.theme.outline; }
            }
            for y in hy..hy + hh {
                let left_idx = (y * area.width + hx) as usize;
                let right_idx = (y * area.width + hx + hw - 1) as usize;
                if left_idx < p.cells.len() { p.cells[left_idx].char = '│'; p.cells[left_idx].fg = self.theme.outline; }
                if right_idx < p.cells.len() { p.cells[right_idx].char = '│'; p.cells[right_idx].fg = self.theme.outline; }
            }
            // Title
            let title = "Data Table Help";
            let tx = hx + (hw - title.len() as u16) / 2;
            for (i, c) in title.chars().enumerate() {
                let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
                if idx < p.cells.len() {
                    p.cells[idx].char = c;
                    p.cells[idx].fg = self.theme.primary;
                    p.cells[idx].style = Styles::BOLD;
                }
            }
            // Shortcuts
            let shortcuts = [
                ("↑/↓", "Navigate"),
                ("Enter", "Sort column"),
                ("Type", "Filter"),
                ("t", "Cycle theme"),
                ("?", "Toggle help"),
                ("q", "Quit"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * area.width + hx + 2 + j as u16) as usize;
                    if idx < p.cells.len() { p.cells[idx].char = c; p.cells[idx].fg = self.theme.primary; }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * area.width + hx + 12 + j as u16) as usize;
                    if idx < p.cells.len() { p.cells[idx].char = c; p.cells[idx].fg = self.theme.fg; }
                }
            }
        }

        p
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        if self.search.handle_key(key) {
            let q = self.search.query().to_string();
            self.filter(&q);
            return true;
        }
        match key.code {
            KeyCode::Char('t') if key.modifiers.is_empty() => {
                self.cycle_theme();
                true
            }
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                self.dirty = true;
                true
            }
            KeyCode::Down => {
                if self.sel + 1 < self.rows.len() {
                    self.sel += 1;
                    if self.sel >= self.off + self.vis {
                        self.off = self.sel - self.vis + 1;
                    }
                    self.dirty = true;
                }
                true
            }
            KeyCode::Up => {
                if self.sel > 0 {
                    self.sel -= 1;
                    if self.sel < self.off {
                        self.off = self.sel;
                    }
                    self.dirty = true;
                }
                true
            }
            KeyCode::Home => {
                self.sel = 0;
                self.off = 0;
                self.dirty = true;
                true
            }
            KeyCode::End => {
                self.sel = self.rows.len().saturating_sub(1);
                self.off = self.off.max(self.sel + 1 - self.vis);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let (hh, sh, inner_y) = (3u16, 1u16, 1u16);
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if row == inner_y && col >= self.area.width.saturating_sub(14) {
                    self.sort = self.sort.next();
                    self.sort_rows();
                    self.dirty = true;
                    true
                } else if row >= hh && row < self.area.height.saturating_sub(sh + 1) {
                    let idx = self.off + (row - hh) as usize;
                    if idx < self.rows.len() {
                        self.sel = idx;
                        self.dirty = true;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            MouseEventKind::ScrollDown => {
                self.off = (self.off + 1).min(self.rows.len().saturating_sub(self.vis));
                self.dirty = true;
                true
            }
            MouseEventKind::ScrollUp => {
                self.off = self.off.saturating_sub(1);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }
}

fn main() -> std::io::Result<()> {
    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let mut t = Table::new();
    t.set_area(Rect::new(0, 0, w, h));
    t.vis = (h as usize).saturating_sub(5).max(1); // account for borders + header + status

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app = App::new()?
        .title("Data Table Demo")
        .fps(30)
        .theme(Theme::cyberpunk());
    app.add_widget(Box::new(t), Rect::new(0, 0, w, h));
    app = app
        .on_input(move |key| {
            if key.code == KeyCode::Char('q') && key.kind == KeyEventKind::Press {
                should_quit.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        })
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
            }
        });
    app.run(|_ctx| {})
}

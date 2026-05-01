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

use dracon_terminal_engine::compositor::{Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::SearchInput;
use ratatui::layout::Rect;

const DATA: &[(&str, u32, &str, &str)] = &[
    ("Alice", 28, "New York", "Software Engineer"), ("Bob", 34, "London", "Data Scientist"),
    ("Carol", 22, "Tokyo", "Product Manager"), ("David", 31, "Berlin", "DevOps Engineer"),
    ("Eve", 29, "Sydney", "UX Designer"), ("Frank", 45, "Toronto", "Engineering Manager"),
    ("Grace", 27, "Singapore", "Frontend Developer"), ("Heidi", 33, "Paris", "Backend Developer"),
    ("Ivan", 41, "Amsterdam", "CTO"), ("Judy", 26, "Seoul", "Mobile Developer"),
];

#[derive(Clone)] struct Person(String, u32, String, String);

impl Person {
    fn matches(&self, q: &str) -> bool {
        let q = q.to_lowercase();
        self.0.to_lowercase().contains(&q) || self.2.to_lowercase().contains(&q)
            || self.3.to_lowercase().contains(&q) || self.1.to_string().contains(&q)
    }
}

enum Sort { None, Asc, Desc }
impl Sort {
    fn next(&self) -> Self { match self { Sort::None => Sort::Asc, Sort::Asc => Sort::Desc, Sort::Desc => Sort::None } }
    fn sym(&self) -> &'static str { match self { Sort::None => "", Sort::Asc => "▲", Sort::Desc => "▼" } }
}

struct Table {
    id: WidgetId, all: Vec<Person>, rows: Vec<Person>, sel: usize, off: usize,
    vis: usize, sort: Sort, search: SearchInput, theme: Theme, area: Rect, dirty: bool,
}

impl Table {
    fn new() -> Self {
        let all: Vec<Person> = DATA.iter().map(|(n, a, c, p)| Person(n.to_string(), *a, c.to_string(), p.to_string())).collect();
        Self { id: WidgetId::default_id(), all: all.clone(), rows: all, sel: 0, off: 0, vis: 8,
            sort: Sort::None, search: SearchInput::new(WidgetId::new(1)), theme: Theme::cyberpunk(),
            area: Rect::new(0, 0, 80, 20), dirty: true }
    }
    fn filter(&mut self, q: &str) {
        self.rows = if q.is_empty() { self.all.clone() } else { self.all.iter().filter(|p| p.matches(q)).cloned().collect() };
        self.sort_rows(); self.sel = 0; self.off = 0; self.dirty = true;
    }
    fn sort_rows(&mut self) {
        match self.sort { Sort::None => {}, Sort::Asc => self.rows.sort_by(|a, b| a.0.cmp(&b.0)), Sort::Desc => self.rows.sort_by(|a, b| b.0.cmp(&a.0)) }
    }
}

impl Widget for Table {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; self.dirty = true; }
    fn z_index(&self) -> u16 { 10 }
    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }

    fn render(&self, area: Rect) -> Plane {
        let mut p = Plane::new(0, area.width, area.height); p.z_index = 10;
        let (heads, widths, hh, sh) = (["Name", "Age", "City", "Profession"], [12u16, 5, 11, 16], 2u16, 1u16);

        for y in 0..area.height { for x in 0..area.width { let idx = (y * area.width + x) as usize; if idx < p.cells.len() { p.cells[idx].bg = self.theme.bg; p.cells[idx].fg = self.theme.fg; } } }

        p.merge_plane(self.search.render(Rect::new(0, 0, 20, 1)), 0, 0);
        let lbl = format!("Sort: Name{}", self.sort.sym());
        let sx = area.width.saturating_sub(lbl.len() as u16 + 2);
        for (i, c) in lbl.chars().enumerate() { let idx = (sx + i as u16) as usize; if idx < p.cells.len() { p.cells[idx].char = c; p.cells[idx].fg = self.theme.fg; } }
        for x in 0..area.width { let idx = (area.width + x) as usize; if idx < p.cells.len() { p.cells[idx].char = '─'; p.cells[idx].fg = Color::Ansi(240); } }

        let mut x = 0u16;
        for (h, w) in heads.iter().zip(widths.iter()) {
            let w = *w.min(&area.width.saturating_sub(x));
            for (j, c) in h.chars().take(w as usize).enumerate() { let idx = (hh * area.width + x + j as u16) as usize; if idx < p.cells.len() { p.cells[idx].char = c; p.cells[idx].fg = self.theme.active_fg; p.cells[idx].style = Styles::BOLD; } }
            x += w + 1;
        }

        for (i, row) in self.rows.iter().skip(self.off).take(self.vis).enumerate() {
            let y = hh + i as u16;
            let sel = self.off + i == self.sel;
            let bg = if sel { self.theme.selection_bg } else { self.theme.bg };
            let fg = if sel { self.theme.selection_fg } else { self.theme.fg };
            let sty = if sel { Styles::BOLD } else { Styles::empty() };
            for x in 0..area.width { let idx = (y * area.width + x) as usize; if idx < p.cells.len() { p.cells[idx].bg = bg; p.cells[idx].fg = fg; } }

            let vals = [&row.0, &row.1.to_string(), &row.2, &row.3];
            let mut x = 0u16;
            for (j, (v, w)) in vals.iter().zip(widths.iter()).enumerate() {
                let w = *w.min(&area.width.saturating_sub(x));
                let txt = if j == 1 { format!("{:>3}", v) } else { v.chars().take(w as usize - 1).collect() };
                let pre = if sel && j == 0 { "> " } else { " " };
                for (k, c) in pre.chars().chain(txt.chars()).take(w as usize).enumerate() { let idx = (y * area.width + x + k as u16) as usize; if idx < p.cells.len() { p.cells[idx].char = c; p.cells[idx].fg = fg; p.cells[idx].style = sty; } }
                x += w + 1;
            }
        }

        let sy = area.height - sh;
        let txt = if let Some(r) = self.rows.get(self.sel) { format!("Selected: {} | Age: {} | City: {} | {} rows", r.0, r.1, r.2, self.rows.len()) } else { format!("No results | {} rows", self.rows.len()) };
        for (i, c) in txt.chars().take(area.width as usize).enumerate() { let idx = (sy * area.width + i as u16) as usize; if idx < p.cells.len() { p.cells[idx].char = c; p.cells[idx].fg = Color::Rgb(0, 255, 136); } }
        p
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press { return false; }
        if self.search.handle_key(key.clone()) { self.filter(&self.search.query().to_string()); return true; }
        match key.code {
            KeyCode::Down => { if self.sel + 1 < self.rows.len() { self.sel += 1; if self.sel >= self.off + self.vis { self.off = self.sel - self.vis + 1; } self.dirty = true; } true }
            KeyCode::Up => { if self.sel > 0 { self.sel -= 1; if self.sel < self.off { self.off = self.sel; } self.dirty = true; } true }
            KeyCode::Home => { self.sel = 0; self.off = 0; self.dirty = true; true }
            KeyCode::End => { self.sel = self.rows.len().saturating_sub(1); self.off = self.off.max(self.sel + 1 - self.vis); self.dirty = true; true }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, col: u16, row: u16) -> bool {
        let (hh, sh) = (2u16, 1u16);
        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                if row == hh - 1 && col >= self.area.width.saturating_sub(14) { self.sort = self.sort.next(); self.sort_rows(); self.dirty = true; true }
                else if row >= hh && row < self.area.height - sh { let idx = self.off + (row - hh) as usize; if idx < self.rows.len() { self.sel = idx; self.dirty = true; true } else { false } }
                else { false }
            }
            crate::input::event::MouseEventKind::ScrollDown => { self.off = (self.off + 1).min(self.rows.len().saturating_sub(self.vis)); self.dirty = true; true }
            crate::input::event::MouseEventKind::ScrollUp => { self.off = self.off.saturating_sub(1); self.dirty = true; true }
            _ => false,
        }
    }
}

fn main() -> std::io::Result<()> {
    App::new()?.title("Data Table Demo").fps(30).theme(Theme::cyberpunk()).run(|ctx| {
        let (w, h) = ctx.compositor().size();
        let mut t = Table::new();
        t.set_area(Rect::new(0, 0, w, h));
        t.vis = (h as usize).saturating_sub(3).max(1);
        ctx.add_plane(t.render(t.area));
    })
}
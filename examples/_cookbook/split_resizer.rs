#![allow(missing_docs)]
//! Split Resizer — Nested SplitPane with mouse drag resize.
//! Controls: click divider=select, drag=resize, ←/→=A split, ↑/↓:B split, r=reset

use std::cell::RefCell;
use std::io::Result;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use rand::Rng;
use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::os::fd::AsFd;

const DA: f32 = 0.30;
const DB: f32 = 0.50;

struct SplitResizerApp {
    id: WidgetId,
    ra: f32,
    rb: f32,
    sel: Option<u8>,
    area: Rect,
    dirty: bool,
    pct: f32,
    txt: String,
}

impl SplitResizerApp {
    fn new(id: WidgetId) -> Self {
        Self {
            id,
            ra: DA,
            rb: DB,
            sel: None,
            area: Rect::new(0, 0, 80, 24),
            dirty: true,
            pct: 50.0,
            txt: "Ready".into(),
        }
    }
}

impl Widget for SplitResizerApp {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; self.dirty = true; }
    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }

    fn render(&self, area: Rect) -> Plane {
        let mut p = Plane::new(0, area.width, area.height);
        p.z_index = 0;
        for c in p.cells.iter_mut() { c.bg = Color::Ansi(33); }
        let h = 1u16;
        let ch = area.height - 2 * h;
        let c = Rect::new(0, h, area.width, ch);
        self.render_header(Rect::new(0, 0, area.width, h), &mut p);
        self.render_content(c, &mut p);
        self.render_status(Rect::new(0, area.height - h, area.width, h), &mut p);
        p
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        match key.code {
            KeyCode::Char('r') => { self.ra = DA; self.rb = DB; self.dirty = true; true }
            KeyCode::Left => { self.ra = (self.ra - 0.05).max(0.1); self.dirty = true; true }
            KeyCode::Right => { self.ra = (self.ra + 0.05).min(0.9); self.dirty = true; true }
            KeyCode::Up | KeyCode::Down => { if self.sel.is_none() { self.sel = Some(1); } self.rb = (self.rb + if key.code == KeyCode::Up { -0.05 } else { 0.05 }).clamp(0.1, 0.9); self.dirty = true; true }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let c = Rect::new(0, 1, self.area.width, self.area.height - 2);
        let da = Rect::new((c.width as f32 * self.ra).round() as u16, c.y, 2, c.height);
        let db = Rect::new((c.width as f32 * self.ra).round() as u16, (c.height as f32 * self.rb).round() as u16 + 1, c.width - (c.width as f32 * self.ra).round() as u16, 2);
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if col >= da.x && row >= da.y && row < da.y + da.height { self.sel = Some(0); }
                else if col >= db.x && col < db.x + db.width && row >= db.y { self.sel = Some(1); }
                self.dirty = true; true
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                match self.sel {
                    Some(0) => { self.ra = (col as f32 / c.width as f32).clamp(0.1, 0.9); self.dirty = true; }
                    Some(1) => { self.rb = ((row as f32 - 1.0) / c.height as f32).clamp(0.1, 0.9); self.dirty = true; }
                    _ => {}
                } true
            }
            _ => false,
        }
    }
}

impl SplitResizerApp {
    fn render_header(&self, r: Rect, p: &mut Plane) {
        let t = " Nested Split Panes — Drag dividers to resize ";
        for (i, c) in t.chars().enumerate().take(r.width as usize) { let idx = i; if idx < p.cells.len() { p.cells[idx] = Cell { char: c, fg: Color::Rgb(0, 255, 136), bg: Color::Ansi(33), style: Styles::BOLD, transparent: false, skip: false }; } }
        for x in 0..r.width { let idx = (r.width + x) as usize; if idx < p.cells.len() { p.cells[idx] = Cell { char: '─', fg: Color::Rgb(80, 80, 100), bg: Color::Ansi(33), style: Styles::empty(), transparent: false, skip: false }; } }
    }

    fn render_content(&self, c: Rect, p: &mut Plane) {
        let w1 = ((c.width as f32 * self.ra).round() as u16).max(10);
        let pa = Rect::new(c.x, c.y, w1, c.height);
        let pb = Rect::new(c.x + w1, c.y, c.width - w1, c.height);
        self.render_a(pa, p);
        let h1 = ((pb.height as f32 * self.rb).round() as u16).max(5);
        let pb1 = Rect::new(pb.x, pb.y, pb.width, h1);
        let pb2 = Rect::new(pb.x, pb.y + h1, pb.width, pb.height - h1);
        self.render_b1(pb1, p);
        self.render_b2(pb2, p);
        let fg = if self.sel == Some(0) { Color::Rgb(255, 255, 0) } else { Color::Rgb(80, 80, 100) };
        for y in 0..c.height { let idx = ((c.y + y) * p.width + c.x + w1) as usize; if idx < p.cells.len() { p.cells[idx] = Cell { char: if y == c.height / 2 { '╮' } else { '│' }, fg, bg: Color::Ansi(33), style: Styles::empty(), transparent: false, skip: false }; } }
        let fg = if self.sel == Some(1) { Color::Rgb(255, 255, 0) } else { Color::Rgb(80, 80, 100) };
        for x in 0..pb.width { let idx = ((pb.y + h1) * p.width + pb.x + x) as usize; if idx < p.cells.len() { p.cells[idx] = Cell { char: if x == pb.width / 2 { '┴' } else { '─' }, fg, bg: Color::Ansi(33), style: Styles::empty(), transparent: false, skip: false }; } }
    }

    fn render_a(&self, r: Rect, p: &mut Plane) {
        for y in 0..r.height { for x in 0..r.width { let idx = ((r.y + y) * p.width + r.x + x) as usize; if idx < p.cells.len() { p.cells[idx].bg = Color::Ansi(17); } } }
        for (i, c) in " Panel A ".chars().enumerate().take(r.width as usize) { let idx = (r.y * p.width + r.x + i as u16) as usize; if idx < p.cells.len() { p.cells[idx] = Cell { char: c, fg: Color::Rgb(100, 200, 255), bg: Color::Ansi(17), style: Styles::BOLD, transparent: false, skip: false }; } }
        let items = ["Item 1: Config", "Item 2: Settings", "Item 3: Prefs", "Item 4: Options", "Item 5: Controls"];
        let sel = rand::thread_rng().gen_range(0..items.len());
        for (i, item) in items.iter().enumerate() { for (j, c) in format!("{} {}", if i == sel { ">" } else { " " }, item).chars().enumerate().take((r.width as usize - 2).max(0)) { let idx = ((r.y + 2 + i as u16) * p.width + r.x + 1 + j as u16) as usize; if idx < p.cells.len() { p.cells[idx] = Cell { char: c, fg: if i == sel { Color::Rgb(255, 255, 0) } else { Color::Rgb(180, 180, 180) }, bg: Color::Ansi(17), style: Styles::empty(), transparent: false, skip: false }; } } }
    }

    fn render_b1(&self, r: Rect, p: &mut Plane) {
        for y in 0..r.height { for x in 0..r.width { let idx = ((r.y + y) * p.width + r.x + x) as usize; if idx < p.cells.len() { p.cells[idx].bg = Color::Ansi(17); } } }
        for (i, c) in " Panel B1 ".chars().enumerate().take(r.width as usize) { let idx = (r.y * p.width + r.x + i as u16) as usize; if idx < p.cells.len() { p.cells[idx] = Cell { char: c, fg: Color::Rgb(255, 100, 100), bg: Color::Ansi(17), style: Styles::BOLD, transparent: false, skip: false }; } }
        let label = format!("{:.0}%", self.pct); let lx = r.x + r.width.saturating_sub(label.len() as u16) / 2; let ly = r.y + r.height / 2;
        for (i, c) in label.chars().enumerate() { let idx = (ly * p.width + lx + i as u16) as usize; if idx < p.cells.len() { p.cells[idx] = Cell { char: c, fg: Color::Rgb(255, 255, 255), bg: Color::Ansi(17), style: Styles::BOLD, transparent: false, skip: false }; } }
        let bw = (r.width - 4).min(40); let filled = ((bw as f32 * self.pct / 100.0).round() as u16).max(1); let sx = r.x + (r.width - bw) / 2; let sy = r.y + r.height / 2 + 1;
        for i in 0..bw { let idx = (sy * p.width + sx + i) as usize; if idx < p.cells.len() { p.cells[idx] = Cell { char: if i < filled { '█' } else { '░' }, fg: if i < filled { Color::Rgb(0, 255, 136) } else { Color::Rgb(80, 80, 80) }, bg: Color::Ansi(17), style: Styles::empty(), transparent: false, skip: false }; } }
    }

    fn render_b2(&self, r: Rect, p: &mut Plane) {
        for y in 0..r.height { for x in 0..r.width { let idx = ((r.y + y) * p.width + r.x + x) as usize; if idx < p.cells.len() { p.cells[idx].bg = Color::Ansi(17); } } }
        for (i, c) in " Panel B2 ".chars().enumerate().take(r.width as usize) { let idx = (r.y * p.width + r.x + i as u16) as usize; if idx < p.cells.len() { p.cells[idx] = Cell { char: c, fg: Color::Rgb(255, 200, 100), bg: Color::Ansi(17), style: Styles::BOLD, transparent: false, skip: false }; } }
        let lx = r.x + (r.width.saturating_sub(self.txt.len() as u16)) / 2; let ly = r.y + r.height / 2;
        for (i, c) in self.txt.chars().enumerate() { let idx = (ly * p.width + lx + i as u16) as usize; if idx < p.cells.len() { p.cells[idx] = Cell { char: c, fg: Color::Rgb(200, 200, 200), bg: Color::Ansi(17), style: Styles::empty(), transparent: false, skip: false }; } }
    }

    fn render_status(&self, r: Rect, p: &mut Plane) {
        for c in p.cells.iter_mut() { c.bg = Color::Ansi(17); }
        let sa = format!("A:{}%", (self.ra * 100.0).round() as i32);
        let sb = format!("B:{}%", ((1.0 - self.ra) * 100.0).round() as i32);
        let sb1 = format!("B1:{}%", (self.rb * 100.0).round() as i32);
        let txt = format!("{} | {} | {} | ←/→:A ↑/↓:B r:reset", sa, sb, sb1);
        for (i, c) in txt.chars().enumerate().take(r.width as usize) { let idx = (r.y * p.width + i as u16) as usize; if idx < p.cells.len() { p.cells[idx] = Cell { char: c, fg: if i < sa.len() + sb.len() + 4 { Color::Rgb(0, 255, 136) } else { Color::Rgb(150, 150, 150) }, bg: Color::Ansi(17), style: Styles::BOLD, transparent: false, skip: false }; } }
    }

    fn tick(&mut self) {
        if rand::thread_rng().gen_range(0..8) == 0 { self.pct = rand::thread_rng().gen_range(10.0..90.0); }
        if rand::thread_rng().gen_range(0..12) == 0 {
            let t = ["Active", "Processing", "Idle", "Waiting"];
            self.txt = t[rand::thread_rng().gen_range(0..t.len())].to_string();
        }
        self.dirty = true;
    }
}

/// Thin wrapper that routes keyboard/mouse events to a Rc<RefCell<SplitResizerApp>>.
/// Registered in the widget system so input dispatch works, but does not render
/// (rendering is handled by the on_tick callback calling ctx.add_plane()).
struct InputRouter {
    target: Rc<RefCell<SplitResizerApp>>,
    id: WidgetId,
    area: Rect,
}

impl Widget for InputRouter {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { false }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }
    fn render(&self, _area: Rect) -> Plane { Plane::new(0, 0, 0) }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        self.target.borrow_mut().handle_key(key)
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.target.borrow_mut().handle_mouse(kind, col, row)
    }
}

fn main() -> Result<()> {
    println!("Split Resizer — drag dividers | ←/→:A | ↑/↓:B | r:reset");
    std::thread::sleep(std::time::Duration::from_millis(300));

    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let app = Rc::new(RefCell::new(SplitResizerApp::new(WidgetId::new(1))));
    let app_for_tick = Rc::clone(&app);
    let app_for_input = Rc::clone(&app);

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app_ctx = App::new()?
        .title("Split Resizer")
        .fps(30)
        .tick_interval(200);

    let router = InputRouter {
        target: app_for_input,
        id: WidgetId::new(100),
        area: Rect::new(0, 0, w, h),
    };
    app_ctx.add_widget(Box::new(router), Rect::new(0, 0, w, h));

    app_ctx
        .on_input(move |key| {
            if key.code == KeyCode::Char('q') && key.kind == KeyEventKind::Press {
                should_quit.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        })
        .on_tick(move |ctx, tick| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
                return;
            }
            if tick % 2 == 0 { app_for_tick.borrow_mut().tick(); }
            let (w, h) = ctx.compositor().size();
            let mut a = app_for_tick.borrow_mut();
            if a.area.width != w || a.area.height != h { a.set_area(Rect::new(0, 0, w, h)); }
            if a.needs_render() {
                let area = a.area();
                let plane = a.render(area);
                a.clear_dirty();
                drop(a);
                ctx.add_plane(plane);
            }
        })
        .run(|_| {})
}
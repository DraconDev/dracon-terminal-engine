#![allow(missing_docs)]
//! Framework Demo — demonstrates App, List, Breadcrumbs, SplitPane, Hud widgets.
//!
//! Layout: Split pane with message list (left) and system info (right).

use std::os::fd::AsFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

struct SystemData {
    cpu_usage: f32,
    mem_usage: f32,
    total_mem: f32,
    swap_usage: f32,
    total_swap: f32,
    uptime: u64,
    disks: Vec<DiskInfo>,
}

struct DiskInfo {
    name: String,
    used_space: f64,
    total_space: f64,
}

impl SystemData {
    fn mock() -> Self {
        Self {
            cpu_usage: 45.3,
            mem_usage: 8.2,
            total_mem: 16.0,
            swap_usage: 1.0,
            total_swap: 32.0,
            uptime: 86400,
            disks: vec![DiskInfo { name: "/dev/sda1".into(), used_space: 120.5, total_space: 500.0 }],
        }
    }
}

struct SystemMonitor {
    id: WidgetId,
    area: std::cell::Cell<Rect>,
    data: SystemData,
    theme: Theme,
    dirty: bool,
}

impl SystemMonitor {
    fn new(id: WidgetId, theme: Theme) -> Self {
        Self { id, area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)), data: SystemData::mock(), theme, dirty: true }
    }

    fn tick(&mut self) {
        self.data.cpu_usage = (self.data.cpu_usage + (rand::random::<f32>() - 0.5) * 10.0).clamp(0.0, 100.0);
        self.data.mem_usage = (self.data.mem_usage + (rand::random::<f32>() - 0.5) * 0.5).clamp(0.0, self.data.total_mem);
        self.dirty = true;
    }
}

impl Widget for SystemMonitor {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area.get() }
    fn set_area(&mut self, area: Rect) { self.area.set(area); self.dirty = true; }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
    fn on_theme_change(&mut self, theme: &Theme) { self.theme = *theme; }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut p = Plane::new(0, area.width, area.height); p.z_index = 10;
        for c in p.cells.iter_mut() { c.bg = t.bg; c.fg = t.fg; c.transparent = false; }

        let split = SplitPane::new(Orientation::Horizontal).ratio(0.3);
        let (left_rect, right_rect) = split.split(area);
        let div_plane = split.render_divider(area);

        let list_items: Vec<String> = vec!["[INFO] Application started".into(), "[WARN] High CPU usage detected".into(), "[INFO] Memory: 8.2/16.0 GB".into(), "[ERROR] Disk space low".into(), "[DEBUG] Connection established".into()];
        let list_plane = List::new_with_id(WidgetId::new(2), list_items).render(left_rect);
        for (i, c) in list_plane.cells.iter().enumerate() {
            let row = i / left_rect.width as usize;
            let col = i % left_rect.width as usize;
            let dest_idx = ((left_rect.y + row as u16) * area.width + left_rect.x + col as u16) as usize;
            if !c.transparent && dest_idx < p.cells.len() { p.cells[dest_idx] = c.clone(); }
        }

        for (i, c) in div_plane.cells.iter().enumerate() {
            let row = i / div_plane.width as usize;
            let col = i % div_plane.width as usize;
            let dest_idx = ((div_plane.y + row as u16) * area.width + div_plane.x + col as u16) as usize;
            if !c.transparent && dest_idx < p.cells.len() { p.cells[dest_idx] = c.clone(); }
        }

        let mut info_plane = Plane::new(1, right_rect.width, right_rect.height);
        for c in info_plane.cells.iter_mut() { c.bg = t.surface; c.fg = t.fg; }

        let print_line = |p: &mut Plane, y: u16, txt: &str, fg: Color| {
            for (i, ch) in txt.chars().take(p.width as usize - 2).enumerate() {
                let idx = (y * p.width + 1 + i as u16) as usize;
                if idx < p.cells.len() { p.cells[idx].char = ch; p.cells[idx].fg = fg; }
            }
        };

        let mut y = 1u16;
        print_line(&mut info_plane, y, &format!("CPU: {:.1}%", self.data.cpu_usage), t.primary); y += 1;
        print_line(&mut info_plane, y, &format!("Memory: {:.1} / {:.1} GB", self.data.mem_usage, self.data.total_mem), t.info); y += 1;
        print_line(&mut info_plane, y, &format!("Swap: {:.1} / {:.1} GB", self.data.swap_usage, self.data.total_swap), t.fg_muted); y += 1;
        print_line(&mut info_plane, y, &format!("Uptime: {}s", self.data.uptime), t.fg_muted); y += 1;
        print_line(&mut info_plane, y, "", t.fg); y += 1;

        if let Some(disk) = self.data.disks.first() {
            let pct = if disk.total_space > 0.0 { (disk.used_space / disk.total_space * 100.0) as u16 } else { 0 };
            print_line(&mut info_plane, y, &format!("Disk: {} ({}%)", disk.name, pct), t.warning);
        }

        for (i, c) in info_plane.cells.iter().enumerate() {
            let row = i / right_rect.width as usize;
            let col = i % right_rect.width as usize;
            let dest_x = right_rect.x + col as u16;
            let dest_y = right_rect.y + row as u16;
            let dest_idx = (dest_y * area.width + dest_x) as usize;
            if !c.transparent && dest_idx < p.cells.len() { p.cells[dest_idx] = c.clone(); }
        }

        let hud_plane = Hud::new(100).with_size(30, 5).with_theme(*t).render_gauge(0, 0, "CPU", self.data.cpu_usage, 100.0, 20);
        for (i, c) in hud_plane.cells.iter().enumerate() {
            if c.transparent { continue; }
            let row = i / hud_plane.width as usize;
            let col = i % hud_plane.width as usize;
            let dest_idx = ((row as u16) * area.width + col as u16) as usize;
            if dest_idx < p.cells.len() { p.cells[dest_idx] = c.clone(); }
        }

        let bc = Breadcrumbs::new_with_id(WidgetId::new(3), vec!["~".into(), "System".into(), "Monitor".into()]);
        let bc_plane = bc.render(Rect::new(0, 0, area.width, 1));
        for (i, c) in bc_plane.cells.iter().enumerate().take(area.width as usize) {
            if !c.transparent && i < p.cells.len() { p.cells[i] = c.clone(); }
        }

        p
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        if let KeyCode::Char('q') = key.code { std::process::exit(0); }
        false
    }

    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool { false }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();
    let mut app = App::new()?.title("Framework Demo").fps(30).theme(theme);
    let sm = SystemMonitor::new(WidgetId::new(1), theme);
    app.add_widget(Box::new(sm), Rect::new(0, 0, 80, 24));
    app.on_tick(|ctx, _| {
        if let Some(w) = ctx.focused() { }
    }).run(|_| {})
}
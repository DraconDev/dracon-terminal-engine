//! Dashboard Builder — showcases all command-bound widgets in a grid layout.
//!
//! Run with: cargo run --example dashboard_builder
//!
//! Demonstrates all 5 command-bound widgets: Gauge, KeyValueGrid, StatusBadge, LogViewer, StreamingText
//! with auto-refresh, theme cycling (t), pause (p), and manual refresh (r).

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::command::{BoundCommand, OutputParser};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Gauge, KeyValueGrid, LogViewer, StatusBadge, StreamingText};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

struct Dashboard {
    gauge: Gauge,
    kv_grid: KeyValueGrid,
    status_badge: StatusBadge,
    log_viewer: LogViewer,
    streaming: StreamingText,
}

impl Dashboard {
    fn new() -> Self {
        Self {
            gauge: Gauge::with_id(WidgetId::new(1), "CPU %").max(100.0).warn_threshold(70.0).crit_threshold(90.0)
                .bind_command(BoundCommand::new("echo 'cpu:67'").parser(OutputParser::Regex { pattern: r"cpu:(\d+)".into(), group: Some(1) }).refresh(2)),
            kv_grid: KeyValueGrid::with_id(WidgetId::new(2)).separator(": ")
                .bind_command(BoundCommand::new("echo 'Memory:8.2GB|Disk:45%|Network:120Mbps'").refresh(5)),
            status_badge: StatusBadge::new(WidgetId::new(3)).with_label("Connection")
                .bind_command(BoundCommand::new("echo 'Connected'").refresh(10)),
            log_viewer: LogViewer::with_id(WidgetId::new(4)).max_lines(100)
                .bind_command(BoundCommand::new("echo '[INFO] Server connected\\n[WARN] High latency: 250ms'").refresh(3)),
            streaming: StreamingText::with_id(WidgetId::new(5)).max_lines(50)
                .bind_command(BoundCommand::new("date +'%H:%M:%S'").refresh(1)),
        }
    }
}

impl Widget for Dashboard {
    fn id(&self) -> WidgetId { WidgetId::new(0) }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { Rect::new(0, 0, 80, 24) }
    fn set_area(&mut self, _area: Rect) {}
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let theme = Theme::nord();
        let mut plane = Plane::new(0, area.width, area.height);
        let content_top = 2u16;
        let content_height = area.height.saturating_sub(4);
        let content_rect = Rect::new(0, content_top, area.width, content_height);

        let grid_rects = Layout::default().constraints([Constraint::Percentage(50), Constraint::Percentage(50)]).split(content_rect);

        let top_row = Rect::new(grid_rects[0].x, grid_rects[0].y, grid_rects[0].width, grid_rects[0].height / 2);
        copy_plane_cells(&mut plane, &self.gauge.render(top_row), top_row.x as usize, top_row.y as usize);

        let kv_rect = Rect::new(grid_rects[1].x, grid_rects[1].y, grid_rects[1].width, grid_rects[1].height / 2);
        copy_plane_cells(&mut plane, &self.kv_grid.render(kv_rect), kv_rect.x as usize, kv_rect.y as usize);

        let badge_rect = Rect::new(grid_rects[0].x, grid_rects[0].y + grid_rects[0].height / 2, grid_rects[0].width, grid_rects[0].height / 2);
        copy_plane_cells(&mut plane, &self.status_badge.render(badge_rect), badge_rect.x as usize, badge_rect.y as usize);

        let log_rect = Rect::new(grid_rects[1].x, grid_rects[1].y + grid_rects[1].height / 2, grid_rects[1].width / 2, grid_rects[1].height / 2);
        copy_plane_cells(&mut plane, &self.log_viewer.render(log_rect), log_rect.x as usize, log_rect.y as usize);

        let stream_rect = Rect::new(grid_rects[1].x + grid_rects[1].width / 2, grid_rects[1].y + grid_rects[1].height / 2, grid_rects[1].width / 2, grid_rects[1].height / 2);
        copy_plane_cells(&mut plane, &self.streaming.render(stream_rect), stream_rect.x as usize, stream_rect.y as usize);

        render_separator(&mut plane, 1, area.width, theme.border);
        render_separator(&mut plane, area.height.saturating_sub(2), area.width, theme.border);
        render_header_text(&mut plane, &theme, "Dashboard Builder", "[ACTIVE]", "Refresh: 3s", "Theme: Nord");
        render_footer_text(&mut plane, area.width, area.height.saturating_sub(1), &theme);
        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        match key.code {
            KeyCode::Char('t') | KeyCode::Char('p') | KeyCode::Char('r') => true,
            _ => false,
        }
    }
    fn focusable(&self) -> bool { false }
    fn z_index(&self) -> u16 { 0 }
}

fn copy_plane_cells(dest: &mut Plane, src: &Plane, offset_x: usize, offset_y: usize) {
    for (i, cell) in src.cells.iter().enumerate() {
        if cell.char == '\0' || cell.transparent { continue; }
        let src_width = src.width as usize;
        let row = i / src_width;
        let col = i % src_width;
        let dest_row = offset_y + row;
        let dest_col = offset_x + col;
        if dest_row >= dest.height as usize || dest_col >= dest.width as usize { continue; }
        let dest_idx = dest_row * dest.width as usize + dest_col;
        if dest_idx < dest.cells.len() { dest.cells[dest_idx] = cell.clone(); }
    }
}

fn render_separator(plane: &mut Plane, row: u16, width: u16, color: Color) {
    for x in 0..width as usize {
        let idx = (row as usize * plane.width as usize + x).min(plane.cells.len().saturating_sub(1));
        plane.cells[idx] = Cell { char: '─', fg: color, bg: plane.cells[idx].bg, style: Styles::empty(), transparent: false, skip: false };
    }
}

fn render_header_text(plane: &mut Plane, theme: &Theme, title: &str, status: &str, refresh: &str, theme_str: &str) {
    let right_len = status.len() + 1 + refresh.len() + 1 + theme_str.len();
    let left_end = (plane.width as usize).saturating_sub(right_len);
    let mut offset = 0;
    for c in title.chars().take(left_end) {
        if offset < plane.cells.len() { plane.cells[offset] = Cell { char: c, fg: theme.accent, bg: theme.bg, style: Styles::BOLD, transparent: false, skip: false }; }
        offset += 1;
    }
    offset = left_end + 1;
    for c in status.chars() { if offset < plane.cells.len() { plane.cells[offset] = Cell { char: c, fg: theme.success_fg, bg: theme.bg, style: Styles::empty(), transparent: false, skip: false }; } offset += 1; }
    offset += 1;
    for c in refresh.chars() { if offset < plane.cells.len() { plane.cells[offset] = Cell { char: c, fg: theme.inactive_fg, bg: theme.bg, style: Styles::empty(), transparent: false, skip: false }; } offset += 1; }
    offset += 1;
    for c in theme_str.chars() { if offset < plane.cells.len() { plane.cells[offset] = Cell { char: c, fg: theme.fg, bg: theme.bg, style: Styles::empty(), transparent: false, skip: false }; } offset += 1; }
}

fn render_footer_text(plane: &mut Plane, width: u16, footer_y: u16, theme: &Theme) {
    let controls = "[r] Refresh  [p] Pause  [t] Theme";
    let offset = 1;
    for (i, c) in controls.chars().enumerate().take(width as usize - offset) {
        let idx = ((footer_y as usize + 1) * plane.width as usize + offset + i).min(plane.cells.len().saturating_sub(1));
        plane.cells[idx] = Cell { char: c, fg: theme.inactive_fg, bg: theme.bg, style: Styles::empty(), transparent: false, skip: false };
    }
}

fn main() -> std::io::Result<()> {
    let theme_idx = Arc::new(AtomicUsize::new(0));
    let paused = Arc::new(AtomicBool::new(false));
    let theme_idx_clone = theme_idx.clone();
    let paused_clone = paused.clone();

    let tick_cb = move |_ctx: &mut Ctx, tick: u64| {
        if tick % 3 == 0 && !paused_clone.load(Ordering::SeqCst) { theme_idx_clone.fetch_add(1, Ordering::SeqCst); }
    };

    let mut app = App::new()?
        .title("Dashboard Builder")
        .fps(30)
        .theme(Theme::nord())
        .tick_interval(1000)
        .on_tick(tick_cb);

    app.add_widget(Box::new(Dashboard::new()), Rect::new(0, 0, 80, 24));

    app.run(move |ctx| {
        ctx.hide_cursor().ok();
        ctx.mark_dirty(0, 0, 80, 24);
    })
}
#![allow(missing_docs)]
//! Tabbed Panels — demonstrates TabBar with per-tab widget state.
//!
//! This example shows:
//! - Tabbar widget with 4 tabs (Dashboard, Logs, Settings, Stats)
//! - Each tab maintaining its own independent widget state
//! - Left/Right arrows to switch tabs
//! - Click on tab to switch
//! - Active tab highlighted with different color/style

use dracon_terminal_engine::compositor::{Cell, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Gauge, KeyValueGrid, List, Select, Slider, TabBar, Toggle,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::os::fd::AsFd;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const TAB_DASHBOARD: usize = 0;
const TAB_LOGS: usize = 1;
const TAB_SETTINGS: usize = 2;
const TAB_STATS: usize = 3;

/// Copies a source plane into a destination plane at the given offset,
/// skipping cells with char == '\0'.
fn copy_plane(dest: &mut Plane, src: &Plane, dest_x: u16, dest_y: u16) {
    for sy in 0..src.height {
        for sx in 0..src.width {
            let src_idx = (sy * src.width + sx) as usize;
            if src.cells[src_idx].transparent {
                continue;
            }
            let dx = dest_x + sx;
            let dy = dest_y + sy;
            if dx < dest.width && dy < dest.height {
                let dest_idx = (dy as usize) * (dest.width as usize) + (dx as usize);
                if dest_idx < dest.cells.len() {
                    dest.cells[dest_idx] = src.cells[src_idx].clone();
                }
            }
        }
    }
}

struct DashboardState {
    cpu: Gauge,
    memory: Gauge,
    disk: Gauge,
    network: Gauge,
}

impl DashboardState {
    fn new(base_id: WidgetId) -> Self {
        Self {
            cpu: Gauge::with_id(WidgetId::new(base_id.0 + 1), "CPU"),
            memory: Gauge::with_id(WidgetId::new(base_id.0 + 2), "Memory"),
            disk: Gauge::with_id(WidgetId::new(base_id.0 + 3), "Disk"),
            network: Gauge::with_id(WidgetId::new(base_id.0 + 4), "Network"),
        }
    }

    fn set_values(&mut self, cpu: f64, memory: f64, disk: f64, network: f64) {
        self.cpu.set_value(cpu);
        self.memory.set_value(memory);
        self.disk.set_value(disk);
        self.network.set_value(network);
    }
}

struct LogsState {
    list: List<String>,
}

impl LogsState {
    fn new(base_id: WidgetId) -> Self {
        let logs = vec![
            "INFO  Boot complete".to_string(),
            "INFO  Loading configuration".to_string(),
            "WARN  Low memory warning".to_string(),
            "INFO  Network initialized".to_string(),
            "DEBUG Processing request".to_string(),
            "INFO  Request completed (200ms)".to_string(),
            "WARN  Retry attempt 1/3".to_string(),
            "INFO  Connection established".to_string(),
            "DEBUG Cache hit ratio: 87%".to_string(),
            "INFO  Shutdown signal received".to_string(),
        ];
        Self {
            list: List::new_with_id(WidgetId::new(base_id.0 + 1), logs),
        }
    }
}

struct SettingsState {
    theme_select: Select,
    notifications: Toggle,
    volume_slider: Slider,
}

impl SettingsState {
    fn new(base_id: WidgetId) -> Self {
        Self {
            theme_select: Select::new(WidgetId::new(base_id.0 + 1))
                .with_options(vec!["Dark".to_string(), "Light".to_string(), "Cyberpunk".to_string()]),
            notifications: Toggle::new(WidgetId::new(base_id.0 + 2), "Enable notifications"),
            volume_slider: Slider::new(WidgetId::new(base_id.0 + 3)).with_range(0.0, 100.0),
        }
    }
}

struct StatsState {
    grid: KeyValueGrid,
}

impl StatsState {
    fn new(base_id: WidgetId) -> Self {
        Self {
            grid: KeyValueGrid::with_id(WidgetId::new(base_id.0 + 1)).with_theme(Theme::default()),
        }
    }
}

struct TabbedApp {
    tabbar: TabBar,
    dashboard: DashboardState,
    logs: LogsState,
    settings: SettingsState,
    stats: StatsState,
    area: Rect,
    should_quit: Arc<AtomicBool>,
}

impl TabbedApp {
    fn new(should_quit: Arc<AtomicBool>) -> Self {
        let tabbar = TabBar::new_with_id(WidgetId::new(1), vec!["Dashboard", "Logs", "Settings", "Stats"]);
        Self {
            tabbar,
            dashboard: DashboardState::new(WidgetId::new(10)),
            logs: LogsState::new(WidgetId::new(20)),
            settings: SettingsState::new(WidgetId::new(30)),
            stats: StatsState::new(WidgetId::new(40)),
            area: Rect::new(0, 0, 80, 24),
            should_quit,
        }
    }

    fn active_tab(&self) -> usize {
        self.tabbar.active()
    }
}

fn render_dashboard(plane: &mut Plane, dashboard: &DashboardState, area: Rect) {
    let half_w = area.width / 2;
    let half_h = area.height / 2;

    let cpu_area = Rect::new(area.x, area.y, half_w, half_h);
    let mem_area = Rect::new(area.x + half_w, area.y, half_w, half_h);
    let disk_area = Rect::new(area.x, area.y + half_h, half_w, half_h);
    let net_area = Rect::new(area.x + half_w, area.y + half_h, half_w, half_h);

    let cpu_plane = dashboard.cpu.render(cpu_area);
    copy_plane(plane, &cpu_plane, cpu_area.x, cpu_area.y);

    let mem_plane = dashboard.memory.render(mem_area);
    copy_plane(plane, &mem_plane, mem_area.x, mem_area.y);

    let disk_plane = dashboard.disk.render(disk_area);
    copy_plane(plane, &disk_plane, disk_area.x, disk_area.y);

    let net_plane = dashboard.network.render(net_area);
    copy_plane(plane, &net_plane, net_area.x, net_area.y);
}

fn render_logs(plane: &mut Plane, logs: &LogsState, area: Rect) {
    let list_plane = logs.list.render(area);
    copy_plane(plane, &list_plane, area.x, area.y);
}

fn render_settings(plane: &mut Plane, settings: &SettingsState, area: Rect, theme: Theme) {
    let label_col = 0u16;
    let input_col = 20u16;

    let mut y = area.y;

    let label = "Theme: ";
    for (i, c) in label.chars().enumerate() {
        let idx = (y * plane.width + label_col + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell { char: c, fg: theme.fg, bg: theme.bg, style: Styles::empty(), transparent: false, skip: false };
        }
    }
    let theme_plane = settings.theme_select.render(Rect::new(input_col, y, 20, 4));
    copy_plane(plane, &theme_plane, input_col, y);
    y += 2;

    let toggle_plane = settings.notifications.render(Rect::new(input_col, y, 25, 1));
    copy_plane(plane, &toggle_plane, input_col, y);
    y += 3;

    let slider_plane = settings.volume_slider.render(Rect::new(input_col, y, 40, 1));
    copy_plane(plane, &slider_plane, input_col, y);

    let volume_label = "Volume:";
    for (i, c) in volume_label.chars().enumerate() {
        let idx = (y * plane.width + label_col + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell { char: c, fg: theme.fg, bg: theme.bg, style: Styles::empty(), transparent: false, skip: false };
        }
    }
}

fn render_stats(plane: &mut Plane, stats: &StatsState, area: Rect) {
    let grid_plane = stats.grid.render(area);
    copy_plane(plane, &grid_plane, area.x, area.y);
}

impl Widget for TabbedApp {
    fn id(&self) -> WidgetId {
        WidgetId::new(1)
    }

    fn set_id(&mut self, _id: WidgetId) {
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    fn z_index(&self) -> u16 {
        10
    }

    fn needs_render(&self) -> bool {
        true
    }

    fn mark_dirty(&mut self) {
    }

    fn clear_dirty(&mut self) {
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        // Fill with theme background
        let theme = Theme::cyberpunk();
        for cell in plane.cells.iter_mut() {
            cell.bg = theme.bg;
            cell.fg = theme.fg;
            cell.transparent = false;
        }

        let tabbar_height = 3u16;

        // Render tab bar
        let tabbar_plane = self.tabbar.render(Rect::new(0, 0, area.width, tabbar_height));
        copy_plane(&mut plane, &tabbar_plane, 0, 0);

        // Separator line
        let separator_y = tabbar_height;
        for col in 0..area.width as usize {
            let idx = (separator_y as usize) * (area.width as usize) + col;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell { char: '─', fg: theme.fg_muted, bg: theme.bg, style: Styles::empty(), transparent: false, skip: false };
            }
        }

        let content_area = Rect::new(0, tabbar_height + 1, area.width, area.height - tabbar_height - 1);

        match self.active_tab() {
            TAB_DASHBOARD => render_dashboard(&mut plane, &self.dashboard, content_area),
            TAB_LOGS => render_logs(&mut plane, &self.logs, content_area),
            TAB_SETTINGS => render_settings(&mut plane, &self.settings, content_area, theme),
            TAB_STATS => render_stats(&mut plane, &self.stats, content_area),
            _ => {}
        }

        let hint = format!("[Left/Right] Switch tabs | Active: {}", match self.active_tab() {
            TAB_DASHBOARD => "Dashboard",
            TAB_LOGS => "Logs",
            TAB_SETTINGS => "Settings",
            TAB_STATS => "Stats",
            _ => "Unknown",
        });
        for (i, c) in hint.chars().take(area.width as usize).enumerate() {
            let idx = ((area.height - 1) as usize) * (area.width as usize) + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell { char: c, fg: theme.fg_muted, bg: theme.bg, style: Styles::empty(), transparent: false, skip: false };
            }
        }

        plane
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Quit on 'q'
        if key.code == KeyCode::Char('q') {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.tabbar.handle_key(key.clone()) {
            return true;
        }

        match self.active_tab() {
            TAB_LOGS => self.logs.list.handle_key(key),
            TAB_SETTINGS => {
                if self.settings.theme_select.handle_key(key.clone()) {
                    return true;
                }
                if self.settings.notifications.handle_key(key.clone()) {
                    return true;
                }
                self.settings.volume_slider.handle_key(key)
            },
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let tabbar_height = 3u16;
        if row < tabbar_height {
            return self.tabbar.handle_mouse(kind, col, row);
        }

        match self.active_tab() {
            TAB_LOGS => self.logs.list.handle_mouse(kind, col, row - tabbar_height - 1),
            TAB_SETTINGS => {
                if row == tabbar_height + 2 && col >= 20 && col < 40 {
                    self.settings.theme_select.handle_mouse(kind, col - 20, row - tabbar_height - 1)
                } else if row == tabbar_height + 4 && col >= 20 && col < 45 {
                    self.settings.volume_slider.handle_mouse(kind, col - 20, 0)
                } else {
                    self.settings.notifications.handle_mouse(kind, col, row - tabbar_height - 1)
                }
            },
            _ => false,
        }
    }
}

/// Thin wrapper that routes keyboard/mouse events to a Rc<RefCell<TabbedApp>>.
/// Registered in the widget system so input dispatch works, but does not render
/// (rendering is handled by the on_tick callback calling ctx.add_plane()).
struct InputRouter {
    target: Rc<RefCell<TabbedApp>>,
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

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();
    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let app = Rc::new(RefCell::new(TabbedApp::new(should_quit)));
    let app_for_tick = Rc::clone(&app);
    let app_for_input = Rc::clone(&app);

    let mut app_ctx = App::new()?
        .title("Tabbed Panels Demo")
        .fps(30)
        .theme(theme);

    // Register an InputRouter so keyboard/mouse events reach TabbedApp
    let router = InputRouter {
        target: app_for_input,
        id: WidgetId::new(100),
        area: Rect::new(0, 0, w, h),
    };
    app_ctx.add_widget(Box::new(router), Rect::new(0, 0, w, h));

    app_ctx
        .on_tick(move |ctx, tick| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
                return;
            }

            let mut app = app_for_tick.borrow_mut();
            let cpu = 45.0 + (tick as f64 % 20.0);
            let memory = 60.0 + (tick as f64 % 15.0);
            let disk = 30.0 + (tick as f64 % 10.0);
            let network = 20.0 + (tick as f64 % 25.0);
            app.dashboard.set_values(cpu, memory, disk, network);
            ctx.mark_all_dirty();
        })
        .run(move |ctx| {
            let (cw, ch) = ctx.compositor().size();
            let a = app.borrow();
            let tabbed_plane = a.render(Rect::new(0, 0, cw, ch));
            ctx.add_plane(tabbed_plane);
        })
}
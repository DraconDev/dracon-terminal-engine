#![allow(missing_docs)]
//! Framework demo  -  shows App, List, Breadcrumbs, SplitPane, Hud, SystemMonitor.

use std::cell::RefCell;
use std::os::fd::AsFd;
use dracon_terminal_engine::compositor::{Color, Plane};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Breadcrumbs, Hud, List, Orientation, SplitPane};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind};
use dracon_terminal_engine::SystemMonitor;
use ratatui::layout::Rect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct FrameworkDemo {
    id: WidgetId,
    breadcrumbs: Breadcrumbs,
    sys: RefCell<SystemMonitor>,
    area: Rect,
    theme: Theme,
    show_help: bool,
    dirty: bool,
    keybindings: KeybindingSet,
}

impl FrameworkDemo {
    fn new(id: WidgetId, theme: Theme) -> Self {
        let breadcrumbs = Breadcrumbs::new(vec![
            "home".to_string(),
            "user".to_string(),
            "projects".to_string(),
            "app".to_string(),
        ]);
        let kb_config = resolve_keybindings();
        let keybindings = KeybindingSet::from_config(&kb_config);
        Self {
            id,
            breadcrumbs,
            sys: RefCell::new(SystemMonitor::new()),
            area: Rect::new(0, 0, 80, 24),
            theme,
            show_help: false,
            dirty: true,
            keybindings,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = vec![Theme::nord(), Theme::cyberpunk(), Theme::dracula()];
        let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.dirty = true;
    }
}

impl Widget for FrameworkDemo {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; }
    fn z_index(&self) -> u16 { 10 }
    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
    fn focusable(&self) -> bool { true }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            self.cycle_theme();
            return true;
        }
        if key.kind != KeyEventKind::Press { return false; }
        match key.code {
            KeyCode::Esc => {
                if self.show_help {
                    self.show_help = false;
                    self.dirty = true;
                    true
                } else { false }
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.dirty = true;
    }

    fn current_theme(&self) -> Option<Theme> {
        Some(self.theme.clone())
    }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let split = SplitPane::new(Orientation::Horizontal).ratio(0.3);
        let (left_rect, right_rect) = split.split(area);

        let mut list = List::new(vec![
            "System Monitor",
            "File Browser",
            "Network Stats",
            "Process List",
            "Disk Usage",
            "Memory Info",
            "CPU Graph",
            "Settings",
        ]);
        list.set_visible_count((left_rect.height as usize).saturating_sub(2).max(1));
        let list_plane = list.render(left_rect);

        let _ = self.breadcrumbs.render(right_rect);

        let data = self.sys.borrow_mut().get_data();

        let mut info_plane = Plane::new(0, right_rect.width, right_rect.height.saturating_sub(2));
        info_plane.z_index = 5;

        let mut y = 2u16;
        let mut print_line = |plane: &mut Plane, text: &str, fg: Color| {
            for (i, c) in text.chars().take(right_rect.width as usize - 2).enumerate() {
                let idx = (y * right_rect.width + 1 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].transparent = false;
                }
            }
            y += 1;
        };

        print_line(&mut info_plane, &format!("CPU: {:.1}%", data.cpu_usage), Color::Rgb(0, 200, 120));
        print_line(&mut info_plane, &format!("Memory: {:.1} / {:.1} GB", data.mem_usage, data.total_mem), Color::Rgb(100, 180, 255));
        print_line(&mut info_plane, &format!("Swap: {:.1} / {:.1} GB", data.swap_usage, data.total_swap), Color::Rgb(180, 180, 200));
        print_line(&mut info_plane, &format!("Uptime: {}s", data.uptime), Color::Rgb(150, 150, 150));
        print_line(&mut info_plane, "", Color::Reset);

        if let Some(disk) = data.disks.first() {
            let pct = if disk.total_space > 0.0 {
                (disk.used_space / disk.total_space * 100.0) as u16
            } else {
                0
            };
            print_line(&mut info_plane, &format!("Disk: {} ({}%)", disk.name, pct), Color::Rgb(255, 180, 100));
        }

        let hud = Hud::new(100).with_size(30, 5);
        let _ = hud.render_gauge(0, 0, "CPU", data.cpu_usage, 100.0, 20);

        let mut p = Plane::new(0, area.width, area.height);
        p.z_index = 10;
        p.fill_bg(t.bg);

        for y in 0..area.height {
            for x in 0..area.width {
                let src_idx = (y * area.width + x) as usize;
                if src_idx < list_plane.cells.len() {
                    let dest_idx = (y * area.width + x) as usize;
                    if dest_idx < p.cells.len() {
                        p.cells[dest_idx] = list_plane.cells[src_idx];
                    }
                }
            }
        }

        if self.show_help {
            let hw = 40u16.min(area.width.saturating_sub(4));
            let hh = 10u16.min(area.height.saturating_sub(4));
            let hx = (area.width - hw) / 2;
            let hy = (area.height - hh) / 2;

            for y in hy..hy + hh {
                for x in hx..hx + hw {
                    let idx = (y * area.width + x) as usize;
                    if idx < p.cells.len() {
                        p.cells[idx].bg = t.surface_elevated;
                        p.cells[idx].transparent = false;
                    }
                }
            }

            for x in hx + 1..hx + hw - 1 {
                let top = (hy * area.width + x) as usize;
                let bot = ((hy + hh - 1) * area.width + x) as usize;
                if top < p.cells.len() { p.cells[top].char = '─'; p.cells[top].fg = t.outline; }
                if bot < p.cells.len() { p.cells[bot].char = '─'; p.cells[bot].fg = t.outline; }
            }
            for y in hy + 1..hy + hh - 1 {
                let left = (y * area.width + hx) as usize;
                let right = (y * area.width + hx + hw - 1) as usize;
                if left < p.cells.len() { p.cells[left].char = '│'; p.cells[left].fg = t.outline; }
                if right < p.cells.len() { p.cells[right].char = '│'; p.cells[right].fg = t.outline; }
            }
            let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < p.cells.len() { p.cells[idx].char = *ch; p.cells[idx].fg = t.outline; }
            }

            let help_title = "Framework Demo Help";
            let tx = hx + (hw - help_title.len() as u16) / 2;
            for (i, c) in help_title.chars().enumerate() {
                let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
                if idx < p.cells.len() {
                    p.cells[idx].char = c;
                    p.cells[idx].fg = t.primary;
                    p.cells[idx].style = Styles::BOLD;
                }
            }

            let shortcuts = [
                ("Ctrl+T", "Cycle theme"),
                ("F1 / ?", "Toggle help"),
                ("Esc", "Dismiss help"),
                ("Ctrl+Q", "Quit"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * area.width + hx + 2 + j as u16) as usize;
                    if idx < p.cells.len() { p.cells[idx].char = c; p.cells[idx].fg = t.primary; }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * area.width + hx + 14 + j as u16) as usize;
                    if idx < p.cells.len() { p.cells[idx].char = c; p.cells[idx].fg = t.fg; }
                }
            }
        }

        p
    }
}

fn main() -> std::io::Result<()> {
    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app = App::new()?.title("Framework Demo").fps(30).theme(Theme::from_env_or(Theme::cyberpunk()));
    app.add_widget(Box::new(FrameworkDemo::new(WidgetId::new(0), Theme::from_env_or(Theme::cyberpunk()))), Rect::new(0, 0, w, h));
    app.on_input(move |key| {
            if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) && key.kind == KeyEventKind::Press {
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
        })
        .run(|_ctx| {})
}

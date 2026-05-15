use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use ratatui::layout::Rect;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

struct NotifierApp {
    id: WidgetId,
    area: Rect,
    should_quit: Rc<AtomicBool>,
    theme: Theme,
    notification_center: NotificationCenter,
    dirty: bool,
    keybindings: KeybindingSet,
    show_help: bool,
}

impl NotifierApp {
    fn new(should_quit: Rc<AtomicBool>, theme: Theme) -> Self {
        let mut nc = NotificationCenter::new(theme.clone());
        nc.info("Welcome", "Press buttons below to trigger notifications.");
        nc.success("Ready", "Notification center is active.");
        Self {
            id: WidgetId::new(1),
            area: Rect::default(),
            should_quit,
            theme,
            notification_center: nc,
            dirty: true,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            show_help: false,
        }
    }

    fn trigger(&mut self, kind: NotificationKind) {
        match kind {
            NotificationKind::Info => self.notification_center.info("Info", "This is an informational message."),
            NotificationKind::Success => self.notification_center.success("Success", "Operation completed successfully!"),
            NotificationKind::Warning => self.notification_center.warn("Warning", "Something might need your attention."),
            NotificationKind::Error => self.notification_center.error("Error", "Something went wrong!"),
        }
        self.dirty = true;
    }

    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.notification_center.on_theme_change(&self.theme);
        self.dirty = true;
    }
}

impl Widget for NotifierApp {
    fn id(&self) -> WidgetId { self.id }

    fn area(&self) -> Rect { self.area }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
        self.notification_center.set_area(Rect::new(0, 4, area.width, area.height.saturating_sub(4)));
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.notification_center.needs_render()
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        let t = &self.theme;

        // Title
        let title = "Notification Center Demo";
        let tx = (area.width - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (2 * area.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Render the notification center widget
        let nc_area = Rect::new(0, 4, area.width, area.height.saturating_sub(4));
        let nc_plane = self.notification_center.render(nc_area);
        for y in 0..nc_plane.height {
            for x in 0..nc_plane.width {
                let src_idx = (y * nc_plane.width + x) as usize;
                let dst_idx = ((nc_area.y + y) * area.width + (nc_area.x + x)) as usize;
                if src_idx < nc_plane.cells.len() && dst_idx < plane.cells.len() {
                    let src = &nc_plane.cells[src_idx];
                    if !src.transparent {
                        plane.cells[dst_idx] = nc_plane.cells[src_idx];
                    }
                }
            }
        }

        // Buttons
        let labels = [
            ("Info (i)", NotificationKind::Info, t.info),
            ("Success (s)", NotificationKind::Success, t.success),
            ("Warning (w)", NotificationKind::Warning, t.warning),
            ("Error (e)", NotificationKind::Error, t.error),
        ];

        let btn_w = 18u16;
        let btn_h = 3u16;
        let start_x = (area.width - (btn_w + 2) * labels.len() as u16) / 2;
        let btn_y = area.height / 2;

        for (i, (label, _kind, color)) in labels.iter().enumerate() {
            let bx = start_x + i as u16 * (btn_w + 2);
            let by = btn_y;

            for cy in by..by + btn_h {
                for cx in bx..bx + btn_w {
                    let idx = (cy * area.width + cx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            for cx in bx..bx + btn_w {
                let top = (by * area.width + cx) as usize;
                let bot = ((by + btn_h - 1) * area.width + cx) as usize;
                if top < plane.cells.len() {
                    plane.cells[top].char = '─';
                    plane.cells[top].fg = color;
                }
                if bot < plane.cells.len() {
                    plane.cells[bot].char = '─';
                    plane.cells[bot].fg = color;
                }
            }
            for cy in by..by + btn_h {
                let left = (cy * area.width + bx) as usize;
                let right = (cy * area.width + bx + btn_w - 1) as usize;
                if left < plane.cells.len() {
                    plane.cells[left].char = '│';
                    plane.cells[left].fg = color;
                }
                if right < plane.cells.len() {
                    plane.cells[right].char = '│';
                    plane.cells[right].fg = color;
                }
            }

            let corners = [('╭', bx, by), ('╮', bx + btn_w - 1, by), ('╰', bx, by + btn_h - 1), ('╯', bx + btn_w - 1, by + btn_h - 1)];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = *ch;
                    plane.cells[idx].fg = color;
                }
            }

            let lx = bx + (btn_w - label.len() as u16) / 2;
            let ly = by + 1;
            for (j, c) in label.chars().enumerate() {
                let idx = (ly * area.width + lx + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                }
            }
        }

        // Help text
        let help = "Click a button or press the key to trigger a notification. Click a notification to dismiss it.";
        let hx = (area.width - help.len() as u16) / 2;
        let hy = btn_y + btn_h + 2;
        for (i, c) in help.chars().enumerate() {
            let idx = (hy * area.width + hx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
            }
        }

        // Status bar
        let kb_quit = self.keybindings.display(actions::QUIT).unwrap_or("Ctrl+Q");
        let kb_help = self.keybindings.display(actions::HELP).unwrap_or("F1");
        let kb_theme = self.keybindings.display(actions::THEME).unwrap_or("Ctrl+T");
        let kb_back = self.keybindings.display(actions::BACK).unwrap_or("Esc");
        let status = format!("{}: quit | {}: theme | {}: help | {}: back", kb_quit, kb_theme, kb_help, kb_back);
        let sy = area.height.saturating_sub(1);
        for (i, c) in status.chars().enumerate() {
            let idx = (sy * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        // Help overlay
        if self.show_help {
            let hw = 44u16.min(area.width.saturating_sub(4));
            let hh = 12u16.min(area.height.saturating_sub(4));
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
                if idx < plane.cells.len() {
                    plane.cells[idx].char = *ch;
                    plane.cells[idx].fg = t.outline;
                }
            }
            for x in hx + 1..hx + hw - 1 {
                let top = (hy * area.width + x) as usize;
                let bot = ((hy + hh - 1) * area.width + x) as usize;
                if top < plane.cells.len() {
                    plane.cells[top].char = '─';
                    plane.cells[top].fg = t.outline;
                }
                if bot < plane.cells.len() {
                    plane.cells[bot].char = '─';
                    plane.cells[bot].fg = t.outline;
                }
            }
            for y in hy + 1..hy + hh - 1 {
                let left = (y * area.width + hx) as usize;
                let right = (y * area.width + hx + hw - 1) as usize;
                if left < plane.cells.len() {
                    plane.cells[left].char = '│';
                    plane.cells[left].fg = t.outline;
                }
                if right < plane.cells.len() {
                    plane.cells[right].char = '│';
                    plane.cells[right].fg = t.outline;
                }
            }

            let htitle = "Notification Center Help";
            let htx = hx + (hw - htitle.len() as u16) / 2;
            for (i, c) in htitle.chars().enumerate() {
                let idx = ((hy + 1) * area.width + htx + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }

            let shortcuts = [
                ("i", "Info notification"),
                ("s", "Success notification"),
                ("w", "Warning notification"),
                ("e", "Error notification"),
                (self.keybindings.display(actions::THEME).unwrap_or("Ctrl+T"), "Cycle theme"),
                (self.keybindings.display(actions::HELP).unwrap_or("F1"), "Toggle help"),
                (self.keybindings.display(actions::BACK).unwrap_or("Esc"), "Dismiss help"),
                (self.keybindings.display(actions::QUIT).unwrap_or("Ctrl+Q"), "Quit"),
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
                    let idx = (row * area.width + hx + 14 + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.fg;
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }

        if self.keybindings.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
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
            KeyCode::Char('i') if key.modifiers.is_empty() => {
                self.trigger(NotificationKind::Info);
                true
            }
            KeyCode::Char('s') if key.modifiers.is_empty() => {
                self.trigger(NotificationKind::Success);
                true
            }
            KeyCode::Char('w') if key.modifiers.is_empty() => {
                self.trigger(NotificationKind::Warning);
                true
            }
            KeyCode::Char('e') if key.modifiers.is_empty() => {
                self.trigger(NotificationKind::Error);
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.notification_center.handle_mouse(kind, col, row) {
            self.dirty = true;
            return true;
        }

        let btn_w = 18u16;
        let btn_h = 3u16;
        let labels = [
            NotificationKind::Info,
            NotificationKind::Success,
            NotificationKind::Warning,
            NotificationKind::Error,
        ];
        let start_x = (self.area().width - (btn_w + 2) * labels.len() as u16) / 2;
        let btn_y = self.area().height / 2;

        if let MouseEventKind::Down(_) = kind {
            for (i, kind) in labels.iter().enumerate() {
                let bx = start_x + i as u16 * (btn_w + 2);
                let by = btn_y;
                if col >= bx && col < bx + btn_w && row >= by && row < by + btn_h {
                    self.trigger(*kind);
                    return true;
                }
            }
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.notification_center.on_theme_change(theme);
        self.dirty = true;
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Rc::new(AtomicBool::new(false));
    let theme = Theme::from_env_or(Theme::nord());
    let app_widget = NotifierApp::new(Rc::clone(&should_quit), theme.clone());

    let mut app = App::new()?
        .title("Notification Center")
        .fps(30)
        .theme(theme);
    app.add_widget(Box::new(app_widget), Rect::new(0, 0, 80, 24));
    app.run(|ctx| {
        if should_quit.load(Ordering::SeqCst) {
            ctx.stop();
        }
    })
}

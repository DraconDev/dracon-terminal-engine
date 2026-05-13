use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use ratatui::layout::Rect;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

struct NotifierApp {
    id: WidgetId,
    area: Rect,
    should_quit: Rc<AtomicBool>,
    theme: Theme,
    notification_center: NotificationCenter,
    buttons: Vec<(String, Rect, NotificationKind)>,
    dirty: bool,
}

impl NotifierApp {
    fn new(should_quit: Rc<AtomicBool>, theme: Theme) -> Self {
        let mut nc = NotificationCenter::new(theme);
        nc.info("Welcome", "Press buttons below to trigger notifications.");
        nc.success("Ready", "Notification center is active.");
        Self {
            id: WidgetId::new(1),
            area: Rect::default(),
            should_quit,
            theme,
            notification_center: nc,
            buttons: Vec::new(),
            dirty: true,
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

        // Title
        let title = "Notification Center Demo";
        let tx = (area.width - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (2 * area.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.primary;
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
                        plane.cells[dst_idx] = nc_plane.cells[src_idx].clone();
                    }
                }
            }
        }

        // Buttons
        let labels = [
            ("Info (i)", NotificationKind::Info, self.theme.info),
            ("Success (s)", NotificationKind::Success, self.theme.success),
            ("Warning (w)", NotificationKind::Warning, self.theme.warning),
            ("Error (e)", NotificationKind::Error, self.theme.error),
        ];

        let btn_w = 18u16;
        let btn_h = 3u16;
        let start_x = (area.width - (btn_w + 2) * labels.len() as u16) / 2;
        let btn_y = area.height / 2;

        for (i, (label, _kind, color)) in labels.iter().enumerate() {
            let bx = start_x + i as u16 * (btn_w + 2);
            let by = btn_y;

            // Button background
            for cy in by..by + btn_h {
                for cx in bx..bx + btn_w {
                    let idx = (cy * area.width + cx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = self.theme.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            // Button border
            for cx in bx..bx + btn_w {
                let top = (by * area.width + cx) as usize;
                let bot = ((by + btn_h - 1) * area.width + cx) as usize;
                if top < plane.cells.len() {
                    plane.cells[top].char = '─';
                    plane.cells[top].fg = *color;
                }
                if bot < plane.cells.len() {
                    plane.cells[bot].char = '─';
                    plane.cells[bot].fg = *color;
                }
            }
            for cy in by..by + btn_h {
                let left = (cy * area.width + bx) as usize;
                let right = (cy * area.width + bx + btn_w - 1) as usize;
                if left < plane.cells.len() {
                    plane.cells[left].char = '│';
                    plane.cells[left].fg = *color;
                }
                if right < plane.cells.len() {
                    plane.cells[right].char = '│';
                    plane.cells[right].fg = *color;
                }
            }

            // Corners
            let corners = [('╭', bx, by), ('╮', bx + btn_w - 1, by), ('╰', bx, by + btn_h - 1), ('╯', bx + btn_w - 1, by + btn_h - 1)];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = *ch;
                    plane.cells[idx].fg = *color;
                }
            }

            // Label text
            let lx = bx + (btn_w - label.len() as u16) / 2;
            let ly = by + 1;
            for (j, c) in label.chars().enumerate() {
                let idx = (ly * area.width + lx + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = self.theme.fg;
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
                plane.cells[idx].fg = self.theme.fg_muted;
            }
        }

        // Status bar
        let status = "Ctrl+Q: quit | F1: help";
        let sy = area.height.saturating_sub(1);
        for (i, c) in status.chars().enumerate() {
            let idx = (sy * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
                plane.cells[idx].bg = self.theme.surface;
                plane.cells[idx].transparent = false;
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
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

        // Check button clicks
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
        self.theme = *theme;
        self.notification_center.on_theme_change(theme);
        self.dirty = true;
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Rc::new(AtomicBool::new(false));
    let theme = Theme::from_env_or(Theme::nord());
    let app_widget = NotifierApp::new(Rc::clone(&should_quit), theme);

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

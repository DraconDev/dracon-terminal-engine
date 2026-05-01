//! Split Resizer — Nested SplitPane with mouse drag resize.
//!
//! Demonstrates: Nested SplitPane (horizontal + vertical), mouse drag on dividers,
//! keyboard adjustments, and reset to defaults.
//!
//! ## Layout
//! ```
//! ┌─────────────────────────────────────────────────────────┐
//! │ Nested Split Panes — Drag dividers to resize           │
//! ├────────────────────┬────────────────────────────────────┤
//! │   Panel A          │   Panel B                          │
//! │                    │   ┌────────────────────────────┐   │
//! │                    │   │ Panel B1 (Gauge)            │   │
//! │                    │   ├────────────────────────────┤   │
//! │                    │   │ Panel B2 (Label)            │   │
//! │                    │   └────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Controls
//! | Key | Action |
//! |-----|--------|
//! | Click divider | Select (highlighted) |
//! | Drag divider | Resize |
//! | ←/→ | Adjust A/B split (5%) |
//! | ↑/↓ | Adjust B1/B2 split (5%) |
//! | r | Reset all to defaults |

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::app::App;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use rand::Rng;
use ratatui::layout::Rect;
use std::io::Result;

const DEFAULT_A: f32 = 0.30;
const DEFAULT_B: f32 = 0.50;

struct SplitResizerApp {
    id: WidgetId,
    ratio_a: f32,
    ratio_b: f32,
    selected: Option<u8>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    b1_pct: f32,
    b2_text: String,
}

impl SplitResizerApp {
    fn new(id: WidgetId) -> Self {
        Self {
            id,
            ratio_a: DEFAULT_A,
            ratio_b: DEFAULT_B,
            selected: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
            b1_pct: 50.0,
            b2_text: "Status: Ready".to_string(),
        }
    }
}

impl Widget for SplitResizerApp {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn area(&self) -> Rect {
        self.area.get()
    }
    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
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

    fn render(&self, area: Rect) -> Plane {
        let mut p = Plane::new(0, area.width, area.height);
        p.z_index = 0;
        for c in p.cells.iter_mut() {
            c.bg = Color::Ansi(33);
        }

        let h = 1u16;
        let content_h = area.height - 2 * h;
        let content = Rect::new(0, h, area.width, content_h);

        self.render_header(Rect::new(0, 0, area.width, h), &mut p);
        self.render_content(content, &mut p);
        self.render_status(Rect::new(0, area.height - h, area.width, h), &mut p);
        p
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Char('r') => {
                self.ratio_a = DEFAULT_A;
                self.ratio_b = DEFAULT_B;
                self.dirty = true;
                true
            }
            KeyCode::Left => {
                self.ratio_a = (self.ratio_a - 0.05).max(0.1);
                self.dirty = true;
                true
            }
            KeyCode::Right => {
                self.ratio_a = (self.ratio_a + 0.05).min(0.9);
                self.dirty = true;
                true
            }
            KeyCode::Up | KeyCode::Down => {
                if self.selected.is_none() {
                    self.selected = Some(1);
                }
                let d = if key.code == KeyCode::Up { -0.05 } else { 0.05 };
                self.ratio_b = (self.ratio_b + d).clamp(0.1, 0.9);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let content_h = self.area.get().height - 2;
        let content = Rect::new(0, 1, self.area.get().width, content_h);

        let div_a = self.divider_a_rect(content);
        let div_b = self.divider_b_rect(content);

        match kind {
            MouseEventKind::Down(ratatui::mouse::MouseButton::Left) => {
                if self.in_rect(col, row, div_a) {
                    self.selected = Some(0);
                    self.dirty = true;
                    true
                } else if self.in_rect(col, row, div_b) {
                    self.selected = Some(1);
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            MouseEventKind::Drag(_) => match self.selected {
                Some(0) => {
                    self.ratio_a = (col as f32 / content.width.max(1) as f32).clamp(0.1, 0.9);
                    self.dirty = true;
                    true
                }
                Some(1) => {
                    self.ratio_b =
                        ((row as f32 - 1.0) / content.height.max(1) as f32).clamp(0.1, 0.9);
                    self.dirty = true;
                    true
                }
                None => false,
            },
            _ => false,
        }
    }
}

impl SplitResizerApp {
    fn render_header(&self, rect: Rect, p: &mut Plane) {
        let title = " Nested Split Panes — Drag dividers to resize ";
        for (i, c) in title.chars().enumerate().take(rect.width as usize) {
            let idx = i;
            if idx < p.cells.len() {
                p.cells[idx] = Cell {
                    char: c,
                    fg: Color::Rgb(0, 255, 136),
                    bg: Color::Ansi(33),
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }
        for x in 0..rect.width {
            let idx = (rect.width + x) as usize;
            if idx < p.cells.len() {
                p.cells[idx] = Cell {
                    char: '─',
                    fg: Color::Rgb(80, 80, 100),
                    bg: Color::Ansi(33),
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
    }

    fn render_content(&self, rect: Rect, p: &mut Plane) {
        let w1 = ((rect.width as f32 * self.ratio_a).round() as u16).max(10);
        let panel_a = Rect::new(rect.x, rect.y, w1, rect.height);
        let panel_b = Rect::new(rect.x + w1, rect.y, rect.width - w1, rect.height);

        self.render_panel_a(panel_a, p);

        let h1 = ((panel_b.height as f32 * self.ratio_b).round() as u16).max(5);
        let panel_b1 = Rect::new(panel_b.x, panel_b.y, panel_b.width, h1);
        let panel_b2 = Rect::new(
            panel_b.x,
            panel_b.y + h1,
            panel_b.width,
            panel_b.height - h1,
        );

        self.render_panel_b1(panel_b1, p);
        self.render_panel_b2(panel_b2, p);

        self.render_divider_h(rect, w1, p);
        self.render_divider_v(panel_b, h1, p);
    }

    fn render_panel_a(&self, rect: Rect, p: &mut Plane) {
        self.fill_bg(rect, p, Color::Ansi(17));
        self.write_text(
            rect,
            p,
            0,
            " Panel A ",
            Color::Rgb(100, 200, 255),
            Styles::BOLD,
        );

        let items = [
            "Item 1: Config",
            "Item 2: Settings",
            "Item 3: Prefs",
            "Item 4: Options",
            "Item 5: Controls",
        ];
        let sel = rand::thread_rng().gen_range(0..items.len());
        for (i, item) in items.iter().enumerate() {
            let prefix = if i == sel { ">" } else { " " };
            let fg = if i == sel {
                Color::Rgb(255, 255, 0)
            } else {
                Color::Rgb(180, 180, 180)
            };
            self.write_text(
                rect,
                p,
                (2 + i) as u16,
                &format!("{}{}", prefix, item),
                fg,
                Styles::empty(),
            );
        }
    }

    fn render_panel_b1(&self, rect: Rect, p: &mut Plane) {
        self.fill_bg(rect, p, Color::Ansi(17));
        self.write_text(
            rect,
            p,
            0,
            " Panel B1 ",
            Color::Rgb(255, 100, 100),
            Styles::BOLD,
        );

        let label = format!("{:.0}%", self.b1_pct);
        let lx = rect.x + rect.width.saturating_sub(label.len() as u16) / 2;
        let ly = rect.y + rect.height / 2;
        for (i, c) in label.chars().enumerate() {
            let idx = (ly * p.width + lx + i as u16) as usize;
            if idx < p.cells.len() {
                p.cells[idx] = Cell {
                    char: c,
                    fg: Color::Rgb(255, 255, 255),
                    bg: Color::Ansi(17),
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }

        let bar_w = (rect.width - 4).min(40);
        let filled = ((bar_w as f32 * self.b1_pct / 100.0).round() as u16).max(1);
        let sx = rect.x + (rect.width - bar_w) / 2;
        let sy = rect.y + rect.height / 2 + 1;
        for i in 0..bar_w {
            let ch = if i < filled { '█' } else { '░' };
            let fg = if i < filled {
                Color::Rgb(0, 255, 136)
            } else {
                Color::Rgb(80, 80, 80)
            };
            let idx = (sy * p.width + sx + i) as usize;
            if idx < p.cells.len() {
                p.cells[idx] = Cell {
                    char: ch,
                    fg,
                    bg: Color::Ansi(17),
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
    }

    fn render_panel_b2(&self, rect: Rect, p: &mut Plane) {
        self.fill_bg(rect, p, Color::Ansi(17));
        self.write_text(
            rect,
            p,
            0,
            " Panel B2 ",
            Color::Rgb(255, 200, 100),
            Styles::BOLD,
        );

        let lx = rect.x + (rect.width.saturating_sub(self.b2_text.len() as u16)) / 2;
        let ly = rect.y + rect.height / 2;
        for (i, c) in self.b2_text.chars().enumerate() {
            let idx = (ly * p.width + lx + i as u16) as usize;
            if idx < p.cells.len() {
                p.cells[idx] = Cell {
                    char: c,
                    fg: Color::Rgb(200, 200, 200),
                    bg: Color::Ansi(17),
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
    }

    fn render_divider_h(&self, rect: Rect, w1: u16, p: &mut Plane) {
        let fg = if self.selected == Some(0) {
            Color::Rgb(255, 255, 0)
        } else {
            Color::Rgb(80, 80, 100)
        };
        for y in 0..rect.height {
            let idx = ((rect.y + y) * p.width + rect.x + w1) as usize;
            if idx < p.cells.len() {
                let ch = if y == rect.height / 2 { '╮' } else { '│' };
                p.cells[idx] = Cell {
                    char: ch,
                    fg,
                    bg: Color::Ansi(33),
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
    }

    fn render_divider_v(&self, rect: Rect, h1: u16, p: &mut Plane) {
        let fg = if self.selected == Some(1) {
            Color::Rgb(255, 255, 0)
        } else {
            Color::Rgb(80, 80, 100)
        };
        for x in 0..rect.width {
            let idx = ((rect.y + h1) * p.width + rect.x + x) as usize;
            if idx < p.cells.len() {
                let ch = if x == rect.width / 2 { '┴' } else { '─' };
                p.cells[idx] = Cell {
                    char: ch,
                    fg,
                    bg: Color::Ansi(33),
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
    }

    fn render_status(&self, rect: Rect, p: &mut Plane) {
        for c in p.cells.iter_mut() {
            c.bg = Color::Ansi(17);
        }
        let sa = format!("Panel A: {}%", (self.ratio_a * 100.0).round() as i32);
        let sb = format!(
            "Panel B: {}%",
            ((1.0 - self.ratio_a) * 100.0).round() as i32
        );
        let sb1 = format!("Panel B1: {}%", (self.ratio_b * 100.0).round() as i32);
        let ctrl = " | ←/→:A  ↑/↓:B  r:reset";
        let text = format!("{} | {} | {}{}", sa, sb, sb1, ctrl);
        for (i, c) in text.chars().enumerate().take(rect.width as usize) {
            let idx = (rect.y * p.width + i as u16) as usize;
            if idx < p.cells.len() {
                let fg = if i < sa.len() + sb.len() + 4 {
                    Color::Rgb(0, 255, 136)
                } else {
                    Color::Rgb(150, 150, 150)
                };
                p.cells[idx] = Cell {
                    char: c,
                    fg,
                    bg: Color::Ansi(17),
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }
    }

    fn write_text(&self, rect: Rect, p: &mut Plane, y: u16, text: &str, fg: Color, style: Styles) {
        for (i, c) in text.chars().take(rect.width as usize).enumerate() {
            let idx = ((rect.y + y) * p.width + rect.x + i as u16) as usize;
            if idx < p.cells.len() {
                p.cells[idx] = Cell {
                    char: c,
                    fg,
                    bg: Color::Ansi(17),
                    style,
                    transparent: false,
                    skip: false,
                };
            }
        }
    }

    fn fill_bg(&self, rect: Rect, p: &mut Plane, bg: Color) {
        for y in 0..rect.height {
            for x in 0..rect.width {
                let idx = ((rect.y + y) * p.width + rect.x + x) as usize;
                if idx < p.cells.len() {
                    p.cells[idx].bg = bg;
                }
            }
        }
    }

    fn divider_a_rect(&self, content: Rect) -> Rect {
        let w1 = (content.width as f32 * self.ratio_a).round() as u16;
        Rect::new(content.x + w1, content.y, 2, content.height)
    }

    fn divider_b_rect(&self, content: Rect) -> Rect {
        let w1 = (content.width as f32 * self.ratio_a).round() as u16;
        let h1 = (content.height as f32 * self.ratio_b).round() as u16;
        Rect::new(content.x + w1, content.y + h1, content.width - w1, 2)
    }

    fn in_rect(&self, col: u16, row: u16, rect: Rect) -> bool {
        col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
    }
}

fn main() -> Result<()> {
    println!("Split Resizer — Drag dividers | ←/→:A split | ↑/↓:B split | r:reset");
    std::thread::sleep(std::time::Duration::from_millis(300));

    let mut app = SplitResizerApp::new(WidgetId::new(1));

    app.on_tick(move |ctx, tick| {
        if tick % 8 == 0 {
            app.b1_pct = rand::thread_rng().gen_range(10.0..90.0);
        }
        if tick % 12 == 0 {
            let texts = [
                "Status: Active",
                "Status: Processing",
                "Status: Idle",
                "Status: Waiting",
            ];
            app.b2_text = texts[rand::thread_rng().gen_range(0..texts.len())].to_string();
        }
        app.mark_dirty();
    })
    .run(move |ctx| {
        let (w, h) = ctx.compositor().size();
        if app.area.get().width != w || app.area.get().height != h {
            app.set_area(Rect::new(0, 0, w, h));
        }
        if app.needs_render() {
            ctx.add_plane(app.render(app.area()));
            app.clear_dirty();
        }
    })
}

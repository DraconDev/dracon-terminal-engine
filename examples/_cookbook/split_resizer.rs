//! Split Resizer — Nested SplitPane with mouse drag resize.
//!
//! Demonstrates:
//! - Nested SplitPane (horizontal + vertical splits combined)
//! - Mouse drag on dividers to interactively resize
//! - Divider selection with visual highlight
//! - Keyboard adjustments (arrows change selected divider ratio)
//! - Reset to default ratios with `r`
//!
//! ## Layout
//! ```
//! ┌─────────────────────────────────────────────────────────┐
//! │ Nested Split Panes — Drag dividers to resize           │
//! ├────────────────────┬────────────────────────────────────┤
//! │                    │                                     │
//! │   Panel A          │   Panel B                           │
//! │   (List items)     │   ┌───────────────────────────────┐ │
//! │                    │   │ Panel B1 (top, Gauge)          │ │
//! │                    │   ├───────────────────────────────┤ │
//! │                    │   │ Panel B2 (bottom, Label)       │ │
//! │                    │   └───────────────────────────────┘ │
//! ├────────────────────┴────────────────────────────────────┤
//! │ Status: Panel A: 30% | Panel B: 70% | Panel B1: 50%      │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Controls
//!
//! | Key | Action |
//! |-----|--------|
//! | Click divider | Select it (highlighted) |
//! | Drag divider | Resize split ratio |
//! | Left/Right | Adjust horizontal split (5%) |
//! | Up/Down | Adjust vertical split inside B (5%) |
//! | r | Reset all splits to defaults |

use std::io::Result;

use rand::Rng;

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::app::App;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::split::Orientation;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

const DEFAULT_A_RATIO: f32 = 0.30;
const DEFAULT_B_RATIO: f32 = 0.50;

struct PanelA {
    items: Vec<String>,
    selected: usize,
}

impl PanelA {
    fn new() -> Self {
        let items = vec![
            "Item 1: Configuration".to_string(),
            "Item 2: Settings".to_string(),
            "Item 3: Preferences".to_string(),
            "Item 4: Options".to_string(),
            "Item 5: Controls".to_string(),
        ];
        Self { items, selected: 0 }
    }
}

struct PanelB1 {
    percentage: f32,
}

impl PanelB1 {
    fn new() -> Self {
        Self { percentage: 50.0 }
    }
    fn update(&mut self) {
        let mut rng = rand::thread_rng();
        self.percentage = rng.gen_range(10.0..90.0);
    }
}

struct PanelB2 {
    text: String,
}

impl PanelB2 {
    fn new() -> Self {
        Self { text: "Status: Ready".to_string() }
    }
    fn update(&mut self) {
        let texts = [
            "Status: Active",
            "Status: Processing",
            "Status: Idle",
            "Status: Waiting",
        ];
        let mut rng = rand::thread_rng();
        self.text = texts[rng.gen_range(0..texts.len())].to_string();
    }
}

struct SplitResizerApp {
    id: WidgetId,
    ratio_a: f32,
    ratio_b: f32,
    selected_divider: Option<u8>,
    dragging: bool,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    panel_a: PanelA,
    panel_b1: PanelB1,
    panel_b2: PanelB2,
}

impl SplitResizerApp {
    fn new(id: WidgetId) -> Self {
        Self {
            id,
            ratio_a: DEFAULT_A_RATIO,
            ratio_b: DEFAULT_B_RATIO,
            selected_divider: None,
            dragging: false,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
            panel_a: PanelA::new(),
            panel_b1: PanelB1::new(),
            panel_b2: PanelB2::new(),
        }
    }

    fn reset_ratios(&mut self) {
        self.ratio_a = DEFAULT_A_RATIO;
        self.ratio_b = DEFAULT_B_RATIO;
        self.selected_divider = None;
        self.dirty = true;
    }

    fn adjust_divider(&mut self, delta: f32) {
        match self.selected_divider {
            Some(0) => {
                self.ratio_a = (self.ratio_a + delta).clamp(0.1, 0.9);
            }
            Some(1) => {
                self.ratio_b = (self.ratio_b + delta).clamp(0.1, 0.9);
            }
            _ => {}
        }
        self.dirty = true;
    }
}

impl Widget for SplitResizerApp {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area.get() }
    fn set_area(&mut self, area: Rect) { self.area.set(area); self.dirty = true; }
    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 0;

        for cell in plane.cells.iter_mut() {
            cell.bg = Color::Ansi(33);
        }

        let header_height = 1u16;
        let status_height = 1u16;
        let content_height = area.height.saturating_sub(header_height + status_height);

        let header_rect = Rect::new(0, 0, area.width, header_height);
        let content_rect = Rect::new(0, header_height, area.width, content_height);
        let status_rect = Rect::new(0, area.height - status_height, area.width, status_height);

        self.render_header(header_rect, &mut plane);
        self.render_content(content_rect, &mut plane);
        self.render_status(status_rect, &mut plane);

        plane
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        match key.code {
            KeyCode::Char('r') => {
                self.reset_ratios();
                return true;
            }
            KeyCode::Left => {
                self.adjust_divider(-0.05);
                return true;
            }
            KeyCode::Right => {
                self.adjust_divider(0.05);
                return true;
            }
            KeyCode::Up | KeyCode::Down => {
                if self.selected_divider.is_none() {
                    self.selected_divider = Some(1);
                }
                self.adjust_divider(if key.code == KeyCode::Up { -0.05 } else { 0.05 });
                return true;
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let header_height = 1u16;
        let status_height = 1u16;
        let content_height = self.area.get().height.saturating_sub(header_height + status_height);
        let content_rect = Rect::new(0, header_height, self.area.get().width, content_height);

        let divider_a_rect = self.horizontal_divider_rect(content_rect);
        let divider_b_rect = self.vertical_divider_rect(content_rect);

        match kind {
            MouseEventKind::Down(ratatui::mouse::MouseButton::Left) => {
                if self.point_in_rect(col, row, divider_a_rect) {
                    self.selected_divider = Some(0);
                    self.dirty = true;
                    return true;
                } else if self.point_in_rect(col, row, divider_b_rect) {
                    self.selected_divider = Some(1);
                    self.dirty = true;
                    return true;
                }
            }
            MouseEventKind::Drag(_) => {
                let normalized_col = col as f32 / content_rect.width.max(1) as f32;
                let normalized_row = (row - header_height) as f32 / content_rect.height.max(1) as f32;

                if self.selected_divider == Some(0) {
                    self.ratio_a = normalized_col.clamp(0.1, 0.9);
                    self.dirty = true;
                } else if self.selected_divider == Some(1) {
                    self.ratio_b = normalized_row.clamp(0.1, 0.9);
                    self.dirty = true;
                }
                return true;
            }
            MouseEventKind::Up(_) => {
                self.dragging = false;
            }
            _ => {}
        }
        false
    }
}

impl SplitResizerApp {
    fn render_header(&self, rect: Rect, plane: &mut Plane) {
        let text = " Nested Split Panes — Drag dividers to resize ";
        for (i, c) in text.chars().enumerate().take(rect.width as usize) {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
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
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
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

    fn render_content(&self, rect: Rect, plane: &mut Plane) {
        let w1 = ((rect.width as f32 * self.ratio_a).round() as u16).max(10);
        let w2 = rect.width.saturating_sub(w1);

        let panel_a_rect = Rect::new(rect.x, rect.y, w1, rect.height);
        let panel_b_rect = Rect::new(rect.x + w1, rect.y, w2, rect.height);

        self.render_panel_a(panel_a_rect, plane);

        let h1 = ((panel_b_rect.height as f32 * self.ratio_b).round() as u16).max(5);
        let h2 = panel_b_rect.height.saturating_sub(h1);

        let panel_b1_rect = Rect::new(panel_b_rect.x, panel_b_rect.y, panel_b_rect.width, h1);
        let panel_b2_rect = Rect::new(panel_b_rect.x, panel_b_rect.y + h1, panel_b_rect.width, h2);

        self.render_panel_b1(panel_b1_rect, plane);
        self.render_panel_b2(panel_b2_rect, plane);

        self.render_divider_horizontal(rect, w1, plane);
        self.render_divider_vertical(panel_b_rect, h1, plane);
    }

    fn render_panel_a(&self, rect: Rect, plane: &mut Plane) {
        let title = " Panel A ";
        for (i, c) in title.chars().enumerate().take(rect.width as usize) {
            let idx = (rect.y * plane.width + rect.x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: Color::Rgb(100, 200, 255),
                    bg: Color::Ansi(17),
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }

        for y in 0..self.panel_a.items.len() as u16 {
            if rect.y + 2 + y >= rect.y + rect.height {
                break;
            }
            let text = &self.panel_a.items[y as usize];
            let is_selected = y as usize == self.panel_a.selected;
            let fg = if is_selected { Color::Rgb(255, 255, 0) } else { Color::Rgb(180, 180, 180) };
            let bg = if is_selected { Color::Ansi(59) } else { Color::Ansi(17) };

            let prefix = if is_selected { ">" } else { " " };
            let line = format!("{}{}", prefix, text);

            for (i, c) in line.chars().take(rect.width as usize - 2).enumerate() {
                let idx = ((rect.y + 2 + y) * plane.width + rect.x + 1 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell { char: c, fg, bg, style: Styles::empty(), transparent: false, skip: false };
                }
            }
        }
    }

    fn render_panel_b1(&self, rect: Rect, plane: &mut Plane) {
        for cell in plane.cells.iter_mut() {
            let idx = cell as *mut Cell as usize;
            let base = plane.cells.as_ptr() as usize;
            let row = (idx - base) / std::mem::size_of::<Cell>() / plane.width as usize;
            let col = (idx - base) / std::mem::size_of::<Cell>() % plane.width as usize;
            if row >= rect.y as usize && row < rect.y as usize + rect.height as usize &&
               col >= rect.x as usize && col < rect.x as usize + rect.width as usize {
                cell.bg = Color::Ansi(17);
            }
        }

        let title = " Panel B1 ";
        for (i, c) in title.chars().enumerate().take(rect.width as usize) {
            let idx = (rect.y * plane.width + rect.x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: Color::Rgb(255, 100, 100),
                    bg: Color::Ansi(17),
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }

        let percent = self.panel_b1.percentage as u16;
        let bar_width = (rect.width - 4).min(50);
        let filled = ((bar_width as f32 * self.panel_b1.percentage / 100.0).round() as u16).max(1);

        let start_x = rect.x + (rect.width - bar_width) / 2;
        let start_y = rect.y + rect.height / 2;

        let label = format!("{:.1}%", self.panel_b1.percentage);
        let label_x = rect.x + (rect.width.saturating_sub(label.len() as u16)) / 2;
        for (i, c) in label.chars().enumerate() {
            let idx = (start_y * plane.width + label_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell { char: c, fg: Color::Rgb(255, 255, 255), bg: Color::Ansi(17), style: Styles::BOLD, transparent: false, skip: false };
            }
        }

        for i in 0..bar_width {
            let char = if i < filled { '█' } else { '░' };
            let fg = if i < filled { Color::Rgb(0, 255, 136) } else { Color::Rgb(80, 80, 80) };
            let idx = ((start_y + 1) * plane.width + start_x + i) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell { char, fg, bg: Color::Ansi(17), style: Styles::empty(), transparent: false, skip: false };
            }
        }
    }

    fn render_panel_b2(&self, rect: Rect, plane: &mut Plane) {
        for cell in plane.cells.iter_mut() {
            let idx = cell as *mut Cell as usize;
            let base = plane.cells.as_ptr() as usize;
            let row = (idx - base) / std::mem::size_of::<Cell>() / plane.width as usize;
            let col = (idx - base) / std::mem::size_of::<Cell>() % plane.width as usize;
            if row >= rect.y as usize && row < rect.y as usize + rect.height as usize &&
               col >= rect.x as usize && col < rect.x as usize + rect.width as usize {
                cell.bg = Color::Ansi(17);
            }
        }

        let title = " Panel B2 ";
        for (i, c) in title.chars().enumerate().take(rect.width as usize) {
            let idx = (rect.y * plane.width + rect.x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: Color::Rgb(255, 200, 100),
                    bg: Color::Ansi(17),
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }

        let text = &self.panel_b2.text;
        let x = rect.x + (rect.width.saturating_sub(text.len() as u16)) / 2;
        let y = rect.y + rect.height / 2;
        for (i, c) in text.chars().enumerate() {
            let idx = (y * plane.width + x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell { char: c, fg: Color::Rgb(200, 200, 200), bg: Color::Ansi(17), style: Styles::empty(), transparent: false, skip: false };
            }
        }
    }

    fn render_divider_horizontal(&self, content_rect: Rect, w1: u16, plane: &mut Plane) {
        let is_selected = self.selected_divider == Some(0);
        let fg = if is_selected { Color::Rgb(255, 255, 0) } else { Color::Rgb(80, 80, 100) };

        for y in 0..content_rect.height {
            let idx = ((content_rect.y + y) * plane.width + content_rect.x + w1) as usize;
            if idx < plane.cells.len() {
                let drag_handle = if y == content_rect.height / 2 { '╮' } else { '│' };
                plane.cells[idx] = Cell { char: drag_handle, fg, bg: Color::Ansi(33), style: Styles::empty(), transparent: false, skip: false };
            }
        }
    }

    fn render_divider_vertical(&self, panel_b_rect: Rect, h1: u16, plane: &mut Plane) {
        let is_selected = self.selected_divider == Some(1);
        let fg = if is_selected { Color::Rgb(255, 255, 0) } else { Color::Rgb(80, 80, 100) };

        for x in 0..panel_b_rect.width {
            let idx = ((panel_b_rect.y + h1) * plane.width + panel_b_rect.x + x) as usize;
            if idx < plane.cells.len() {
                let drag_handle = if x == panel_b_rect.width / 2 { '┴' } else { '─' };
                plane.cells[idx] = Cell { char: drag_handle, fg, bg: Color::Ansi(33), style: Styles::empty(), transparent: false, skip: false };
            }
        }
    }

    fn render_status(&self, rect: Rect, plane: &mut Plane) {
        let status_a = format!("Panel A: {}%", (self.ratio_a * 100.0).round() as i32);
        let status_b = format!("Panel B: {}%", ((1.0 - self.ratio_a) * 100.0).round() as i32);
        let status_b1 = format!("Panel B1: {}%", (self.ratio_b * 100.0).round() as i32);
        let separator = " | ";
        let controls = " | ←/→:A split  ↑/↓:B split  r:reset";

        let parts = [&status_a, separator, &status_b, separator, &status_b1, controls];
        let mut pos = 0u16;

        for (i, part) in parts.iter().enumerate() {
            for c in part.chars() {
                let idx = (rect.y * plane.width + pos) as usize;
                if idx < plane.cells.len() {
                    let fg = if i < 5 { Color::Rgb(0, 255, 136) } else { Color::Rgb(150, 150, 150) };
                    plane.cells[idx] = Cell { char: c, fg, bg: Color::Ansi(17), style: if i < 5 { Styles::BOLD } else { Styles::empty() }, transparent: false, skip: false };
                }
                pos += 1;
                if pos >= rect.width {
                    break;
                }
            }
            if pos >= rect.width {
                break;
            }
        }
    }

    fn horizontal_divider_rect(&self, content_rect: Rect) -> Rect {
        let w1 = (content_rect.width as f32 * self.ratio_a).round() as u16;
        Rect::new(content_rect.x + w1, content_rect.y, 1, content_rect.height)
    }

    fn vertical_divider_rect(&self, content_rect: Rect) -> Rect {
        let w1 = (content_rect.width as f32 * self.ratio_a).round() as u16;
        let panel_b_x = content_rect.x + w1;
        let h1 = (content_rect.height as f32 * self.ratio_b).round() as u16;
        Rect::new(panel_b_x, content_rect.y + h1, content_rect.width - w1, 1)
    }

    fn point_in_rect(&self, col: u16, row: u16, rect: Rect) -> bool {
        col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
    }
}

fn main() -> Result<()> {
    println!("Split Resizer — Nested SplitPane demo");
    println!("Controls: ←/→ adjust A split | ↑/↓ adjust B split | r reset | drag dividers");
    std::thread::sleep(std::time::Duration::from_millis(300));

    let mut app = SplitResizerApp::new(WidgetId::new(1));

    app.on_tick(move |ctx, tick| {
        if tick % 10 == 0 {
            app.panel_b1.update();
            app.panel_b2.update();
        }
        if tick % 3 == 0 {
            let mut rng = rand::thread_rng();
            app.panel_a.selected = rng.gen_range(0..app.panel_a.items.len());
        }
        app.mark_dirty();
    }).run(move |ctx| {
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
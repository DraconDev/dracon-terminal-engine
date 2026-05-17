//! Embedded Terminal Paint scene for the showcase.
//!
//! Mouse-driven pixel art canvas with brush tools, color palette,
//! and fill/erase/clear operations. Demonstrates the mouse-first philosophy.

use crate::scenes::shared_helpers::draw_text;
use dracon_terminal_engine::compositor::plane::{Color, Plane};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind, MouseButton};
use ratatui::layout::Rect;
use std::cell::{Cell, RefCell};


#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Brush,
    Eraser,
    Fill,

}

pub struct PaintScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    // Canvas
    canvas: RefCell<Vec<Vec<Color>>>,
    canvas_w: usize,
    canvas_h: usize,
    // Cursor/brush
    brush_color: Cell<Color>,
    tool: Cell<Tool>,
    brush_size: Cell<u8>,  // 1 or 2
    // Drawing state
    is_drawing: Cell<bool>,
    last_col: Cell<Option<u16>>,
    last_row: Cell<Option<u16>>,
    // Scroll offset for large canvases
    dirty: bool,
    area: Cell<Rect>,
}

impl PaintScene {
    pub fn new(theme: Theme) -> Self {
        let canvas_w = 64;
        let canvas_h = 20;
        let canvas = RefCell::new(vec![vec![Color::Reset; canvas_w]; canvas_h]);

        Self {
            theme,
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            canvas,
            canvas_w,
            canvas_h,
            brush_color: Cell::new(Color::Rgb(255, 100, 100)),
            tool: Cell::new(Tool::Brush),
            brush_size: Cell::new(1),
            is_drawing: Cell::new(false),
            last_col: Cell::new(None),
            last_row: Cell::new(None),
            dirty: true,
            area: Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn palette_colors() -> Vec<(&'static str, Color)> {
        vec![
            ("Red",    Color::Rgb(255, 100, 100)),
            ("Orange", Color::Rgb(255, 165, 0)),
            ("Yellow", Color::Rgb(255, 255, 100)),
            ("Green",  Color::Rgb(100, 255, 100)),
            ("Cyan",   Color::Rgb(100, 255, 255)),
            ("Blue",   Color::Rgb(100, 100, 255)),
            ("Purple", Color::Rgb(200, 100, 255)),
            ("White",  Color::Rgb(255, 255, 255)),
            ("Gray",   Color::Rgb(128, 128, 128)),
            ("DkGray", Color::Rgb(64, 64, 64)),
        ]
    }

    fn paint_pixel(&self, cx: usize, cy: usize, color: Color) {
        if cx < self.canvas_w && cy < self.canvas_h {
            self.canvas.borrow_mut()[cy][cx] = color;
        }
    }

    fn paint_brush(&self, cx: usize, cy: usize) {
        let color = if self.tool.get() == Tool::Eraser {
            Color::Reset
        } else {
            self.brush_color.get()
        };

        let size = self.brush_size.get() as usize;
        for dy in 0..size {
            for dx in 0..size {
                self.paint_pixel(cx + dx, cy + dy, color);
            }
        }
    }

    fn flood_fill(&mut self, start_x: usize, start_y: usize, fill_color: Color) {
        if start_x >= self.canvas_w || start_y >= self.canvas_h {
            return;
        }
        let target = self.canvas.borrow()[start_y][start_x];
        if target == fill_color {
            return;
        }

        // BFS flood fill
        let mut stack = vec![(start_x, start_y)];
        let mut visited = vec![vec![false; self.canvas_w]; self.canvas_h];

        while let Some((x, y)) = stack.pop() {
            if x >= self.canvas_w || y >= self.canvas_h {
                continue;
            }
            if visited[y][x] {
                continue;
            }
            if self.canvas.borrow()[y][x] != target {
                continue;
            }
            visited[y][x] = true;
            self.canvas.borrow_mut()[y][x] = fill_color;

            if x > 0 { stack.push((x - 1, y)); }
            if x + 1 < self.canvas_w { stack.push((x + 1, y)); }
            if y > 0 { stack.push((x, y - 1)); }
            if y + 1 < self.canvas_h { stack.push((x, y + 1)); }
        }
    }

    fn clear_canvas(&mut self) {
        for row in self.canvas.borrow_mut().iter_mut() {
            for cell in row.iter_mut() {
                *cell = Color::Reset;
            }
        }
        self.dirty = true;
    }

    fn canvas_to_screen(&self, col: u16, row: u16) -> Option<(usize, usize)> {
        // Canvas area: x=20 (after toolbar), y=2 (below header)
        let canvas_x = 20u16;
        let canvas_y = 2u16;
        if col < canvas_x || row < canvas_y {
            return None;
        }
        let cx = (col - canvas_x) as usize;
        let cy = (row - canvas_y) as usize;
        if cx < self.canvas_w && cy < self.canvas_h {
            Some((cx, cy))
        } else {
            None
        }
    }

    fn render_toolbar(&self, plane: &mut Plane, _area: Rect) {
        let t = &self.theme;
        let tx = 0u16;
        let ty = 2u16;

        // Tool section
        draw_text(plane, tx, ty, "Tools", t.primary, t.bg, true);

        let tools = [
            ("B Brush", Tool::Brush),
            ("E Erase", Tool::Eraser),
            ("F Fill", Tool::Fill),
        ];
        for (i, (label, tool)) in tools.iter().enumerate() {
            let ty_i = ty + 2 + i as u16;
            let is_active = self.tool.get() == *tool;
            let fg = if is_active { t.primary } else { t.fg_muted };
            let bg = if is_active { t.hover_bg } else { t.bg };
            let prefix = if is_active { "► " } else { "  " };
            draw_text(plane, tx, ty_i, &format!("{}{}", prefix, label), fg, bg, is_active);
        }

        // Brush size
        draw_text(plane, tx, ty + 6, "Size", t.primary, t.bg, true);
        let sizes = ["1x1", "2x2"];
        for (i, label) in sizes.iter().enumerate() {
            let sy = ty + 8 + i as u16;
            let is_active = self.brush_size.get() as usize == i + 1;
            let fg = if is_active { t.primary } else { t.fg_muted };
            let prefix = if is_active { "► " } else { "  " };
            draw_text(plane, tx, sy, &format!("{}{}", prefix, label), fg, t.bg, is_active);
        }

        // Color palette
        draw_text(plane, tx, ty + 11, "Colors", t.primary, t.bg, true);
        let colors = Self::palette_colors();
        for (i, (name, color)) in colors.iter().enumerate() {
            let cy = ty + 13 + i as u16;
            let is_active = self.brush_color.get() == *color;
            let prefix = if is_active { "► " } else { "  " };

            // Color swatch
            let swatch_x = tx;
            if (cy as usize) < plane.height as usize {
                let idx = cy as usize * plane.width as usize + swatch_x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '█';
                    plane.cells[idx].fg = *color;
                    plane.cells[idx].transparent = false;
                }
                let label_start = swatch_x + 2;
                for (j, ch) in format!("{}{}", prefix, name).chars().enumerate() {
                    let lidx = cy as usize * plane.width as usize + label_start as usize + j;
                    if lidx < plane.cells.len() {
                        plane.cells[lidx].char = ch;
                        plane.cells[lidx].fg = if is_active { t.primary } else { t.fg_muted };
                        plane.cells[lidx].transparent = false;
                    }
                }
            }
        }

        // Actions
        draw_text(plane, tx, ty + 24, "Actions", t.primary, t.bg, true);
        draw_text(plane, tx, ty + 26, "C Clear", t.fg_muted, t.bg, false);
    }

    fn render_canvas(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let canvas_x = 20u16;
        let canvas_y = 2u16;

        // Canvas border
        let cw = self.canvas_w.min((area.width - canvas_x) as usize);
        let ch = self.canvas_h.min((area.height - canvas_y - 1) as usize);

        // Top border
        for x in 0..=cw as u16 {
            let idx = ((canvas_y - 1) * plane.width + canvas_x + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }
        // Left/right borders and content
        for y in 0..ch as u16 {
            let left_idx = ((canvas_y + y) * plane.width + canvas_x - 1) as usize;
            if left_idx < plane.cells.len() {
                plane.cells[left_idx].char = '│';
                plane.cells[left_idx].fg = t.outline;
                plane.cells[left_idx].transparent = false;
            }

            for x in 0..cw {
                let color = self.canvas.borrow()[y as usize][x];
                let cell_idx = ((canvas_y + y) * plane.width + canvas_x + x as u16) as usize;
                if cell_idx < plane.cells.len() {
                    if color == Color::Reset {
                        // Empty cell — show grid dot
                        plane.cells[cell_idx].char = '·';
                        plane.cells[cell_idx].fg = Color::Rgb(40, 40, 50);
                        plane.cells[cell_idx].bg = t.bg;
                    } else {
                        plane.cells[cell_idx].char = '█';
                        plane.cells[cell_idx].fg = color;
                        plane.cells[cell_idx].bg = color;
                    }
                    plane.cells[cell_idx].transparent = false;
                }
            }

            let right_idx = ((canvas_y + y) * plane.width + canvas_x + cw as u16) as usize;
            if right_idx < plane.cells.len() {
                plane.cells[right_idx].char = '│';
                plane.cells[right_idx].fg = t.outline;
                plane.cells[right_idx].transparent = false;
            }
        }
        // Bottom border
        for x in 0..=cw as u16 {
            let idx = ((canvas_y + ch as u16) * plane.width + canvas_x + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Canvas dimensions label
        let dim = format!("{}×{}", self.canvas_w, self.canvas_h);
        draw_text(plane, canvas_x, canvas_y + ch as u16 + 1, &dim, t.fg_muted, t.bg, false);
    }
}

impl Scene for PaintScene {
    fn scene_id(&self) -> &str { "paint" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        draw_text(&mut plane, 2, 0, " Terminal Paint ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 2), 0,
                  &theme_label, t.secondary, t.bg, false);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Toolbar (left side)
        self.render_toolbar(&mut plane, area);

        // Canvas (right of toolbar)
        self.render_canvas(&mut plane, area);

        // Footer
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let tool_name = match self.tool.get() {
            Tool::Brush => "Brush",
            Tool::Eraser => "Eraser",
            Tool::Fill => "Fill",
            
        };
        let footer = format!(
            " Tool:{} | B:brush E:erase F:fill | C:clear | {}:help | {}:back ",
            tool_name, help_key, back_key,
        );
        let fy = area.height.saturating_sub(1);
        for (i, c) in footer.chars().enumerate() {
            let idx = (fy * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        if self.show_help {
            self.render_help(&mut plane, area);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        match key.code {
            KeyCode::Char('b') if key.modifiers.is_empty() => {
                self.tool.set(Tool::Brush);
                self.dirty = true;
                true
            }
            KeyCode::Char('e') if key.modifiers.is_empty() => {
                self.tool.set(Tool::Eraser);
                self.dirty = true;
                true
            }
            KeyCode::Char('f') if key.modifiers.is_empty() => {
                self.tool.set(Tool::Fill);
                self.dirty = true;
                true
            }
            KeyCode::Char('c') if key.modifiers.is_empty() => {
                self.clear_canvas();
                true
            }
            // Color shortcuts (1-9, 0)
            KeyCode::Char(d) if key.modifiers.is_empty() && d.is_ascii_digit() => {
                let idx = if d == '0' { 9 } else { (d as usize) - ('1' as usize) };
                let colors = Self::palette_colors();
                if idx < colors.len() {
                    self.brush_color.set(colors[idx].1);
                    self.dirty = true;
                }
                true
            }
            // Brush size
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.brush_size.set(if self.brush_size.get() < 2 { 2 } else { 1 });
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Check palette clicks
                let ty = 2u16;
                let colors = Self::palette_colors();
                for (i, (_, color)) in colors.iter().enumerate() {
                    let cy = ty + 13 + i as u16;
                    if row == cy && col < 18 {
                        self.brush_color.set(*color);
                        self.dirty = true;
                        return true;
                    }
                }

                // Check tool clicks
                let tools = [Tool::Brush, Tool::Eraser, Tool::Fill];
                for (i, tool) in tools.iter().enumerate() {
                    let ty_i = ty + 2 + i as u16;
                    if row == ty_i && col < 12 {
                        self.tool.set(*tool);
                        self.dirty = true;
                        return true;
                    }
                }

                // Check size clicks
                for i in 0..2u16 {
                    let sy = ty + 8 + i;
                    if row == sy && col < 8 {
                        self.brush_size.set((i + 1) as u8);
                        self.dirty = true;
                        return true;
                    }
                }

                // Canvas click
                if let Some((cx, cy)) = self.canvas_to_screen(col, row) {
                    match self.tool.get() {
                        Tool::Fill => {
                            self.flood_fill(cx, cy, self.brush_color.get());
                            self.dirty = true;
                        }
                        _ => {
                            self.is_drawing.set(true);
                            self.paint_brush(cx, cy);
                            self.last_col.set(Some(col));
                            self.last_row.set(Some(row));
                            self.dirty = true;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            MouseEventKind::Drag(MouseButton::Left) if self.is_drawing.get() => {
                if let Some((cx, cy)) = self.canvas_to_screen(col, row) {
                    // Interpolate between last position and current for smooth drawing
                    if let (Some(lc), Some(lr)) = (self.last_col.get(), self.last_row.get()) {
                        if let Some((lx, ly)) = self.canvas_to_screen(lc, lr) {
                            // Bresenham line interpolation
                            let dx = (cx as i32 - lx as i32).abs();
                            let dy = (cy as i32 - ly as i32).abs();
                            let sx = if lx < cx { 1 } else { -1 };
                            let sy = if ly < cy { 1 } else { -1 };
                            let mut err = dx - dy;
                            let mut x = lx as i32;
                            let mut y = ly as i32;
                            loop {
                                self.paint_brush(x as usize, y as usize);
                                if x as usize == cx && y as usize == cy { break; }
                                let e2 = 2 * err;
                                if e2 > -dy { err -= dy; x += sx; }
                                if e2 < dx { err += dx; y += sy; }
                            }
                        }
                    }
                    self.last_col.set(Some(col));
                    self.last_row.set(Some(row));
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.is_drawing.set(false);
                self.last_col.set(None);
                self.last_row.set(None);
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl PaintScene {
    fn render_help(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 48u16.min(area.width.saturating_sub(4));
        let hh = 16u16.min(area.height.saturating_sub(4));
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

        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let lines = [
            ("╭────────────────────────────────────────────────╮", true),
            ("│         Terminal Paint Help                    │", true),
            ("├────────────────────────────────────────────────┤", true),
            ("│  B         Brush tool                          │", false),
            ("│  E         Eraser tool                          │", false),
            ("│  F         Flood fill tool                      │", false),
            ("│  1-0       Select color from palette             │", false),
            ("│  +/-       Toggle brush size                    │", false),
            ("│  C         Clear canvas                         │", false),
            ("│  Click     Paint / pick tool or color           │", false),
            ("│  Drag      Continuous brush strokes             │", false),
            (&format!("│  {:<10} Dismiss / go back                  │", back_key), false),
            ("╰────────────────────────────────────────────────╯", true),
        ];
        for (i, (line, is_border)) in lines.iter().enumerate() {
            let ly = hy + i as u16;
            let lx = (area.width - line.len() as u16) / 2;
            for (j, ch) in line.chars().enumerate() {
                let px = lx + j as u16;
                if px < area.width && ly < area.height {
                    let idx = (ly * area.width + px) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = if *is_border || "│╭╮├┤╰╯─".contains(ch) { t.outline } else { t.fg };
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }
    }
}

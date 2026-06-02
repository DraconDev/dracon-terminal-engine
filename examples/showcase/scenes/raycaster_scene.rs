//! Embedded 3D Raycaster scene for the showcase.
//!
//! A Wolfenstein-style 3D raycaster rendered entirely in terminal characters.
//! Uses brute-force raycasting against a grid map, then maps wall distances
//! to ASCII shading characters.

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{StatusBar, StatusSegment};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::Cell;

const MAP_W: usize = 16;
const MAP_H: usize = 16;

const MAP: [[u8; MAP_W]; MAP_H] = [
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 2, 2, 2, 0, 0, 0, 3, 3, 3, 0, 0, 0, 1],
    [1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1],
    [1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 4, 4, 4, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 4, 0, 4, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 4, 0, 4, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 5, 5, 0, 0, 0, 0, 0, 6, 6, 6, 0, 0, 1],
    [1, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
];

// ASCII shading characters from far to near
const SHADE_CHARS: &[char] = &[' ', '·', '░', '▒', '▓', '█'];

// Wall colors per wall type (1-6)
fn wall_color(wall_type: u8, theme: &Theme) -> (Color, Color) {
    match wall_type {
        1 => (theme.fg, theme.fg_muted),         // boundary walls
        2 => (theme.primary, theme.primary),     // blue room
        3 => (theme.secondary, theme.secondary), // purple room
        4 => (theme.success, theme.success),     // green room
        5 => (theme.warning, theme.warning),     // yellow room
        6 => (theme.error, theme.error),         // red room
        _ => (theme.fg, theme.fg_muted),
    }
}

pub struct RaycasterScene {
    theme: Theme,
    show_help: bool,
    show_minimap: Cell<bool>,
    keybindings: KeybindingSet,
    // Player state
    px: Cell<f64>, // position x
    py: Cell<f64>, // position y
    pa: Cell<f64>, // angle (radians)
    // Rendering
    dirty: bool,
    area: Cell<Rect>,
    status_bar: std::cell::RefCell<StatusBar>,
}

impl RaycasterScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme: theme.clone(),
            show_help: false,
            show_minimap: Cell::new(true),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            px: Cell::new(8.0), // center of map
            py: Cell::new(8.0),
            pa: Cell::new(0.0), // facing right
            dirty: true,
            area: Cell::new(Rect::new(0, 0, 80, 24)),
            status_bar: std::cell::RefCell::new(
                StatusBar::new(WidgetId::new(200))
                    .add_segment(StatusSegment::new(
                        "WASD:move | ←→:turn | M:minimap | F1:help | Esc:back",
                    ))
                    .with_theme(theme),
            ),
        }
    }

    fn move_player(&self, forward: f64, strafe: f64) {
        let a = self.pa.get();
        let dx = a.cos() * forward + (a + std::f64::consts::FRAC_PI_2).cos() * strafe;
        let dy = a.sin() * forward + (a + std::f64::consts::FRAC_PI_2).sin() * strafe;

        let speed = 0.15;
        let nx = self.px.get() + dx * speed;
        let ny = self.py.get() + dy * speed;

        // Collision check with small margin
        let margin = 0.2;
        let check_x = (nx + dx.signum() * margin) as usize;
        let check_y = (ny + dy.signum() * margin) as usize;

        if check_x < MAP_W {
            let py = self.py.get().clamp(0.0, (MAP_H - 1) as f64) as usize;
            if MAP[py][check_x] == 0 {
                self.px.set(nx.clamp(1.0, (MAP_W - 1) as f64));
            }
        }
        if check_y < MAP_H {
            let px = self.px.get().clamp(0.0, (MAP_W - 1) as f64) as usize;
            if MAP[check_y][px] == 0 {
                self.py.set(ny.clamp(1.0, (MAP_H - 1) as f64));
            }
        }
    }

    fn rotate(&self, delta: f64) {
        let new_a = self.pa.get() + delta;
        self.pa.set(new_a % (2.0 * std::f64::consts::PI));
    }

    fn cast_ray(&self, angle: f64) -> (f64, u8, bool) {
        let px = self.px.get();
        let py = self.py.get();

        let ray_dx = angle.cos();
        let ray_dy = angle.sin();

        // DDA algorithm
        let map_x = px as i32;
        let map_y = py as i32;

        let delta_dist_x = if ray_dx == 0.0 {
            f64::MAX
        } else {
            (1.0 / ray_dx).abs()
        };
        let delta_dist_y = if ray_dy == 0.0 {
            f64::MAX
        } else {
            (1.0 / ray_dy).abs()
        };

        let (step_x, side_dist_x) = if ray_dx < 0.0 {
            (-1, (px - map_x as f64) * delta_dist_x)
        } else {
            (1, (map_x as f64 + 1.0 - px) * delta_dist_x)
        };

        let (step_y, side_dist_y) = if ray_dy < 0.0 {
            (-1, (py - map_y as f64) * delta_dist_y)
        } else {
            (1, (map_y as f64 + 1.0 - py) * delta_dist_y)
        };

        let mut side_dist_x = side_dist_x;
        let mut side_dist_y = side_dist_y;
        let mut map_x = map_x;
        let mut map_y = map_y;
        let mut side = false; // false=x-side, true=y-side

        // Step through grid
        let mut hit = false;
        for _ in 0..64 {
            if side_dist_x < side_dist_y {
                side_dist_x += delta_dist_x;
                map_x += step_x;
                side = false;
            } else {
                side_dist_y += delta_dist_y;
                map_y += step_y;
                side = true;
            }

            if map_x < 0 || map_x >= MAP_W as i32 || map_y < 0 || map_y >= MAP_H as i32 {
                break;
            }

            if MAP[map_y as usize][map_x as usize] > 0 {
                hit = true;
                break;
            }
        }

        if hit {
            let dist = if side {
                side_dist_y - delta_dist_y
            } else {
                side_dist_x - delta_dist_x
            };

            let wall_type = MAP[map_y as usize][map_x as usize];
            (dist.max(0.01), wall_type, side)
        } else {
            (20.0, 0, false)
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn render_minimap(&self, plane: &mut Plane, ox: u16, oy: u16, size: u16) {
        let t = &self.theme;
        let scale = size as f64 / MAP_W as f64;

        // Draw map cells
        for my_idx in 0..MAP_H {
            for mx_idx in 0..MAP_W {
                #[allow(clippy::needless_range_loop)]
                let wall = MAP[my_idx][mx_idx];
                if wall > 0 {
                    let sx = ox + (mx_idx as f64 * scale) as u16;
                    let sy = oy + (my_idx as f64 * scale) as u16;
                    let ex = ox + ((mx_idx + 1) as f64 * scale) as u16;
                    let ey = oy + ((my_idx + 1) as f64 * scale) as u16;

                    let color = wall_color(wall, t).0;
                    for y in sy..ey.min(plane.height) {
                        for x in sx..ex.min(plane.width) {
                            let idx = (y * plane.width + x) as usize;
                            if idx < plane.cells.len() {
                                plane.cells[idx].char = '█';
                                plane.cells[idx].fg = color;
                                plane.cells[idx].transparent = false;
                            }
                        }
                    }
                }
            }
        }

        // Draw player position
        let ppx = ox + (self.px.get() * scale) as u16;
        let ppy = oy + (self.py.get() * scale) as u16;
        if ppx < plane.width && ppy < plane.height {
            let idx = (ppy * plane.width + ppx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '@';
                plane.cells[idx].fg = t.error;
                plane.cells[idx].transparent = false;
            }
        }

        // Draw direction indicator
        let dir_len = 3.0;
        let dx = self.pa.get().cos() * dir_len * scale;
        let dy = self.pa.get().sin() * dir_len * scale;
        let ex = ox + (self.px.get() * scale + dx) as u16;
        let ey = oy + (self.py.get() * scale + dy) as u16;
        if ex < plane.width && ey < plane.height {
            let idx = (ey * plane.width + ex) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '+';
                plane.cells[idx].fg = t.error;
                plane.cells[idx].transparent = false;
            }
        }

        // Border
        for x in ox..ox + size {
            let top = (oy * plane.width + x) as usize;
            let bot = ((oy + size - 1) * plane.width + x) as usize;
            if top < plane.cells.len() {
                plane.cells[top].char = '─';
                plane.cells[top].fg = t.outline;
            }
            if bot < plane.cells.len() {
                plane.cells[bot].char = '─';
                plane.cells[bot].fg = t.outline;
            }
        }
        for y in oy..oy + size {
            let left = (y * plane.width + ox) as usize;
            let right = (y * plane.width + ox + size - 1) as usize;
            if left < plane.cells.len() {
                plane.cells[left].char = '│';
                plane.cells[left].fg = t.outline;
            }
            if right < plane.cells.len() {
                plane.cells[right].char = '│';
                plane.cells[right].fg = t.outline;
            }
        }
    }
}

impl Scene for RaycasterScene {
    fn on_enter(&mut self) {
        self.show_help = false;
        self.dirty = true;
    }

    fn on_exit(&mut self) {
        self.show_help = false;
    }

    fn scene_id(&self) -> &str {
        "raycaster"
    }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header bar
        draw_text(&mut plane, 2, 0, " 3D Raycaster ", t.primary, t.bg, true);

        // 3D viewport
        let view_x = 0u16;
        let view_y = 1u16;
        let view_w = area.width;
        let view_h = area.height.saturating_sub(1);

        let fov = std::f64::consts::FRAC_PI_3; // 60 degree FOV
        let player_a = self.pa.get();
        let half_fov = fov / 2.0;

        // Cast rays across the viewport width
        for col in 0..view_w {
            let ray_angle = player_a - half_fov + (col as f64 / view_w as f64) * fov;

            let (dist, wall_type, side) = self.cast_ray(ray_angle);

            // Fix fisheye
            let corrected_dist = dist * (ray_angle - player_a).cos();

            // Wall height based on distance
            let wall_height = (view_h as f64 / corrected_dist.max(0.1)) as u16;
            let wall_top = if wall_height > view_h {
                0
            } else {
                (view_h - wall_height) / 2
            };
            let wall_bot = (wall_top + wall_height).min(view_h);

            // Determine shade based on distance
            let max_dist = 16.0;
            let shade_idx = if corrected_dist >= max_dist {
                0
            } else {
                ((1.0 - corrected_dist / max_dist) * (SHADE_CHARS.len() - 1) as f64).round()
                    as usize
            }
            .min(SHADE_CHARS.len() - 1);

            let shade_char = SHADE_CHARS[shade_idx];

            // Wall color
            let (wall_fg, wall_dim) = if wall_type > 0 {
                wall_color(wall_type, t)
            } else {
                (t.fg, t.fg_muted)
            };

            // Y-side walls are slightly darker
            let wall_fg = if side { wall_dim } else { wall_fg };

            // Draw ceiling
            for row in 0..wall_top {
                let idx = ((view_y + row) * plane.width + view_x + col) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.bg;
                    plane.cells[idx].transparent = false;
                }
            }

            // Draw wall strip
            for row in wall_top..wall_bot {
                let idx = ((view_y + row) * plane.width + view_x + col) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = shade_char;
                    plane.cells[idx].fg = wall_fg;
                    plane.cells[idx].bg = t.bg;
                    plane.cells[idx].transparent = false;
                }
            }

            // Draw floor
            for row in wall_bot..view_h {
                let floor_dist =
                    view_h as f64 / (2.0 * (row as f64 - view_h as f64 / 2.0)).abs().max(0.5);
                let floor_shade = if floor_dist >= max_dist {
                    0
                } else {
                    ((1.0 - floor_dist / max_dist) * 2.0).round() as usize
                }
                .min(2);
                let floor_char = match floor_shade {
                    0 => ' ',
                    1 => '·',
                    _ => '░',
                };
                let idx = ((view_y + row) * plane.width + view_x + col) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = floor_char;
                    plane.cells[idx].fg = t.fg_muted;
                    plane.cells[idx].bg = t.bg;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Minimap overlay (top-right corner)
        if self.show_minimap.get() {
            let mm_size = 16u16;
            let mm_x = area.width.saturating_sub(mm_size + 2);
            let mm_y = 2;

            // Semi-transparent background for minimap
            for y in mm_y..mm_y + mm_size {
                for x in mm_x..mm_x + mm_size {
                    let idx = (y * plane.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            self.render_minimap(&mut plane, mm_x, mm_y, mm_size);
        }

        // Crosshair
        let cx = view_w / 2;
        let cy = view_y + view_h / 2;
        for (dx, _dy) in [(0u16, 0u16), (1, 0), (2, 0)] {
            let idx = (cy * plane.width + cx + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = if dx == 0 { '┼' } else { '─' };
                plane.cells[idx].fg = t.error;
            }
        }

        // Help overlay
        if self.show_help {
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(
                &mut plane,
                area,
                &self.theme,
                "3D Raycaster — Help",
                &[
                    ("W/↑", "Move forward"),
                    ("S/↓", "Move backward"),
                    ("A/D", "Strafe left/right"),
                    ("←/→", "Turn left/right"),
                    ("M", "Toggle minimap"),
                    ("Scroll", "Turn with mouse wheel"),
                    (back_key, "Back"),
                ],
            );
        }

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_area = Rect::new(0, sb_y, area.width, 1);
        self.status_bar.borrow_mut().set_area(sb_area);
        let sb_plane = self.status_bar.borrow().render(sb_area);
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
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

        let rot_speed = 0.1;
        match key.code {
            KeyCode::Up | KeyCode::Char('w') if key.modifiers.is_empty() => {
                self.move_player(1.0, 0.0);
                self.dirty = true;
                true
            }
            KeyCode::Down | KeyCode::Char('s') if key.modifiers.is_empty() => {
                self.move_player(-1.0, 0.0);
                self.dirty = true;
                true
            }
            KeyCode::Left => {
                self.rotate(-rot_speed);
                self.dirty = true;
                true
            }
            KeyCode::Right => {
                self.rotate(rot_speed);
                self.dirty = true;
                true
            }
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                self.move_player(0.0, -1.0);
                self.dirty = true;
                true
            }
            KeyCode::Char('d') if key.modifiers.is_empty() => {
                self.move_player(0.0, 1.0);
                self.dirty = true;
                true
            }
            KeyCode::Char('m') if key.modifiers.is_empty() => {
                self.show_minimap.set(!self.show_minimap.get());
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        match kind {
            MouseEventKind::ScrollUp => {
                self.rotate(-0.05);
                self.dirty = true;
                true
            }
            MouseEventKind::ScrollDown => {
                self.rotate(0.05);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.status_bar.borrow_mut().on_theme_change(theme);
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
}

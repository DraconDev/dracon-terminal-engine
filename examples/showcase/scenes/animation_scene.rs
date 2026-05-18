//! Embedded Animation scene for the showcase.
//!
//! Demonstrates the AnimationManager + Easing system with:
//!   - Bouncing balls using different easing curves
//!   - Sliding panel with ease-in/out transitions
//!   - Easing curve visualizations
//!   - Color pulse animation

use crate::scenes::shared_helpers::draw_text;
use dracon_terminal_engine::compositor::plane::{Color, Plane, Styles};
use dracon_terminal_engine::framework::animation::{AnimationManager, Easing};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════════════════════
// Data
// ═══════════════════════════════════════════════════════════════════════════════

struct BouncingBall {
    x_anim_id: usize,
    y_anim_id: usize,
    color: Color,
    easing_name: &'static str,
}

fn apply_easing(easing: &Easing, t: f64) -> f64 {
    match easing {
        Easing::Linear => t,
        Easing::EaseIn => t * t,
        Easing::EaseOut => t * (2.0 - t),
        Easing::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
    }
}

fn easing_from_name(name: &str) -> Easing {
    match name {
        "EaseIn" => Easing::EaseIn,
        "EaseOut" => Easing::EaseOut,
        "EaseInOut" => Easing::EaseInOut,
        _ => Easing::Linear,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Scene
// ═══════════════════════════════════════════════════════════════════════════════

pub struct AnimationScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    anim_mgr: RefCell<AnimationManager>,
    balls: RefCell<Vec<BouncingBall>>,
    panel_anim_id: RefCell<usize>,
    panel_visible: RefCell<bool>,
    pulse_anim_id: RefCell<usize>,
    dirty: bool,
    area: std::cell::Cell<Rect>,
}

impl AnimationScene {
    pub fn new(theme: Theme) -> Self {
        let mut anim_mgr = AnimationManager::new();

        let easing_configs = [
            ("Linear", Color::Rgb(255, 100, 100)),
            ("EaseIn", Color::Rgb(100, 255, 100)),
            ("EaseOut", Color::Rgb(100, 100, 255)),
            ("EaseInOut", Color::Rgb(255, 255, 100)),
        ];

        let mut balls = Vec::new();
        for (name, color) in easing_configs {
            let x_id = anim_mgr.start(2.0, 30.0, Duration::from_secs(2));
            let y_id = anim_mgr.start(5.0, 18.0, Duration::from_millis(800));
            balls.push(BouncingBall { x_anim_id: x_id, y_anim_id: y_id, color, easing_name: name });
        }

        let panel_anim_id = anim_mgr.start(0.0, 20.0, Duration::from_millis(500));
        let pulse_anim_id = anim_mgr.start(0.0, 1.0, Duration::from_secs(2));

        Self {
            theme: theme.clone(),
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            anim_mgr: RefCell::new(anim_mgr),
            balls: RefCell::new(balls),
            panel_anim_id: RefCell::new(panel_anim_id),
            panel_visible: RefCell::new(true),
            pulse_anim_id: RefCell::new(pulse_anim_id),
            dirty: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn restart_animations(&self) {
        let mut mgr = self.anim_mgr.borrow_mut();
        mgr.clear();
        let mut balls = self.balls.borrow_mut();
        for ball in balls.iter_mut() {
            ball.x_anim_id = mgr.start(2.0, 30.0, Duration::from_secs(2));
            ball.y_anim_id = mgr.start(5.0, 18.0, Duration::from_millis(800));
        }
        *self.panel_anim_id.borrow_mut() = mgr.start(0.0, 20.0, Duration::from_millis(500));
        *self.pulse_anim_id.borrow_mut() = mgr.start(0.0, 1.0, Duration::from_secs(2));
        *self.panel_visible.borrow_mut() = true;
    }

    fn toggle_panel(&self) {
        let mut mgr = self.anim_mgr.borrow_mut();
        let visible = *self.panel_visible.borrow();
        if visible {
            *self.panel_anim_id.borrow_mut() = mgr.start(20.0, 0.0, Duration::from_millis(500));
        } else {
            *self.panel_anim_id.borrow_mut() = mgr.start(0.0, 20.0, Duration::from_millis(500));
        }
        *self.panel_visible.borrow_mut() = !visible;
    }

    fn tick_animations(&self) {
        let mut mgr = self.anim_mgr.borrow_mut();
        mgr.tick();

        // Restart completed ball animations (bounce loop)
        let mut balls = self.balls.borrow_mut();
        for ball in balls.iter_mut() {
            if mgr.is_done(ball.x_anim_id) {
                let current = mgr.value(ball.x_anim_id).unwrap_or(30.0);
                let target = if current > 15.0 { 2.0 } else { 30.0 };
                ball.x_anim_id = mgr.start(current, target, Duration::from_secs(2));
            }
            if mgr.is_done(ball.y_anim_id) {
                let current = mgr.value(ball.y_anim_id).unwrap_or(18.0);
                let target = if current > 10.0 { 5.0 } else { 18.0 };
                ball.y_anim_id = mgr.start(current, target, Duration::from_millis(800));
            }
        }

        // Restart pulse animation
        if mgr.is_done(*self.pulse_anim_id.borrow()) {
            let current = mgr.value(*self.pulse_anim_id.borrow()).unwrap_or(1.0);
            let target = if current > 0.5 { 0.0 } else { 1.0 };
            *self.pulse_anim_id.borrow_mut() = mgr.start(current, target, Duration::from_secs(2));
        }
    }
}

impl Scene for AnimationScene {
    fn scene_id(&self) -> &str { "animation" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);

        // Tick animations at the start of each render
        self.tick_animations();

        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        let mgr = self.anim_mgr.borrow();
        let balls = self.balls.borrow();

        // Header
        let title = " Animation & Easing ";
        draw_text(&mut plane, 2, 0, title, t.primary, t.bg, true);
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

        // ── Bouncing Balls (left half) ────────────────────────────────────
        let ball_area_w = area.width / 2;
        let ball_area_h = area.height.saturating_sub(8);
        draw_text(&mut plane, 2, 2, "Bouncing Balls (different easing)", t.fg, t.bg, false);

        for ball in balls.iter() {
            let x_val = mgr.value(ball.x_anim_id).unwrap_or(2.0);
            let y_val = mgr.value(ball.y_anim_id).unwrap_or(5.0);
            let bx = (x_val / 32.0 * ball_area_w as f64) as u16 + 2;
            let by = (y_val / 20.0 * ball_area_h as f64) as u16 + 4;

            if bx < area.width && by < area.height {
                let idx = (by * area.width + bx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '●';
                    plane.cells[idx].fg = ball.color;
                    plane.cells[idx].style = Styles::BOLD;
                    plane.cells[idx].transparent = false;
                }
                // Glow
                for (dx, dy) in [(-1i16, 0i16), (1, 0), (0, -1), (0, 1)] {
                    let gx = (bx as i16 + dx).max(0) as u16;
                    let gy = (by as i16 + dy).max(0) as u16;
                    if gx < area.width && gy < area.height {
                        let idx = (gy * area.width + gx) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].char = '·';
                            if let Color::Rgb(r, g, b) = ball.color {
                                plane.cells[idx].fg = Color::Rgb(r / 3, g / 3, b / 3);
                            }
                            plane.cells[idx].transparent = false;
                        }
                    }
                }
            }
        }

        // ── Sliding Panel (right half) ────────────────────────────────────
        let right_x = area.width / 2 + 2;
        let panel_w = area.width / 2 - 4;
        let panel_offset = mgr.value(*self.panel_anim_id.borrow()).unwrap_or(0.0) as u16;

        draw_text(&mut plane, right_x, 2, "Sliding Panel", t.fg, t.bg, true);
        draw_text(&mut plane, right_x, 3, "Press P to toggle", t.fg_muted, t.bg, false);

        if panel_offset > 0 {
            let py = 5u16;
            let ph = 6u16.min(area.height.saturating_sub(8));
            let visible_w = panel_offset.min(panel_w);
            for y in py..py + ph {
                for x in 0..visible_w {
                    let px = right_x + x;
                    if px < area.width && y < area.height {
                        let idx = (y * area.width + px) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].bg = t.surface_elevated;
                            plane.cells[idx].transparent = false;
                        }
                    }
                }
            }
            let panel_lines = [
                "┌──────────────────┐",
                "│  Animated Panel  │",
                "│  Slides in/out   │",
                "│  with easing!    │",
                "└──────────────────┘",
            ];
            for (i, line) in panel_lines.iter().enumerate() {
                let ly = py + i as u16;
                for (j, ch) in line.chars().enumerate() {
                    let lx = right_x + 2 + j as u16;
                    if lx < right_x + visible_w && ly < area.height && lx < area.width {
                        let idx = (ly * area.width + lx) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].char = ch;
                            plane.cells[idx].fg = if "│┌┐└┘─".contains(ch) { t.outline } else { t.fg };
                            plane.cells[idx].bg = t.surface_elevated;
                            plane.cells[idx].transparent = false;
                        }
                    }
                }
            }
        }

        // ── Easing Curves (bottom) ────────────────────────────────────────
        let curve_y = area.height.saturating_sub(8);
        draw_text(&mut plane, 2, curve_y, "Easing Curves:", t.fg, t.bg, true);

        let curve_w = 12u16;
        let curve_h = 4u16;
        for (i, ball) in balls.iter().enumerate() {
            let cx = 2 + i as u16 * (curve_w + 4);
            let cy = curve_y + 2;
            let easing = easing_from_name(ball.easing_name);

            // Draw curve
            for ti in 0..curve_w.saturating_sub(1) {
                let t_val = ti as f64 / (curve_w.saturating_sub(1)) as f64;
                let eased = apply_easing(&easing, t_val);
                let py = cy as f64 + (1.0 - eased) * (curve_h.saturating_sub(1)) as f64;
                let py = py as u16;
                if py < cy + curve_h && cx + ti < area.width {
                    let idx = (py * area.width + cx + ti) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '·';
                        plane.cells[idx].fg = ball.color;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            // Label
            draw_text(&mut plane, cx, cy + curve_h + 1, ball.easing_name, ball.color, t.bg, true);
        }

        // ── Color Pulse (bottom right) ────────────────────────────────────
        let pulse_val = mgr.value(*self.pulse_anim_id.borrow()).unwrap_or(0.0);
        let pulse_r = (80.0 + pulse_val * 175.0) as u8;
        let pulse_g = (40.0 + pulse_val * 60.0) as u8;
        let pulse_b = (120.0 + pulse_val * 135.0) as u8;
        let pulse_color = Color::Rgb(pulse_r, pulse_g, pulse_b);

        let pulse_x = area.width.saturating_sub(18);
        let pulse_y = curve_y + 1;
        draw_text(&mut plane, pulse_x, pulse_y, "Color Pulse", t.fg, t.bg, true);
        for y in (pulse_y + 1)..(pulse_y + 3).min(area.height) {
            for x in 0..14.min(area.width.saturating_sub(pulse_x) as usize) as u16 {
                let idx = (y * area.width + pulse_x + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = pulse_color;
                    plane.cells[idx].fg = Color::Rgb(255, 255, 255);
                    plane.cells[idx].char = ' ';
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // ── Footer ────────────────────────────────────────────────────────
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " {}:help | {}:back | P:panel | R:restart | Anims: {} ",
            help_key, back_key, mgr.len(),
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

        // ── Help Overlay ──────────────────────────────────────────────────
        if self.show_help {
            self.render_help(&mut plane, area);
        }

        drop(mgr);
        drop(balls);

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.keybindings.matches(actions::BACK, &key) {
            if self.show_help {
                self.show_help = false;
                return true;
            }
            return false;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            return true;
        }
        if self.show_help {
            return false;
        }

        match key.code {
            KeyCode::Char('p') if key.modifiers.is_empty() => {
                self.toggle_panel();
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.restart_animations();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();
        match kind {
            MouseEventKind::Down(_) => {
                if col < area.width / 2 {
                    self.restart_animations();
                    self.dirty = true;
                    return true;
                }
                // Click on sliding panel area (right half, rows 3-8) → toggle panel
                if col >= area.width / 2 && (3..10).contains(&row) {
                    self.toggle_panel();
                    self.dirty = true;
                    return true;
                }
                // Click on easing curve labels (bottom rows) → restart
                if row >= area.height.saturating_sub(8) {
                    self.restart_animations();
                    self.dirty = true;
                    return true;
                }
                false
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

impl AnimationScene {
    fn render_help(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 40u16.min(area.width.saturating_sub(4));
        let hh = 10u16.min(area.height.saturating_sub(4));
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

        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");

        let shortcuts = [
            ("╭────────────────────────────────────╮", true),
            ("│       Animation Scene Help         │", true),
            ("├────────────────────────────────────┤", true),
            ("│  P          Toggle sliding panel   │", false),
            ("│  R          Restart animations     │", false),
            (&format!("│  {:<10} Toggle this help          │", help_key), false),
            (&format!("│  {:<10} Dismiss / go back        │", back_key), false),
            ("╰────────────────────────────────────╯", true),
        ];

        for (i, (line, is_border)) in shortcuts.iter().enumerate() {
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

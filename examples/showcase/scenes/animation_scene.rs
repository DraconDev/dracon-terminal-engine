//! Embedded Animation scene for the showcase.
//!
//! Demonstrates the AnimationManager + Easing system with:
//!   - Bouncing balls using different easing curves
//!   - Sliding panel with ease-in/out transitions
//!   - Easing curve visualizations
//!   - Live animated widgets (ProgressBar, Spinner, PulseBox)
//!   - Sidebar with animation controls + hover info

use crate::scenes::shared_helpers::{blit_to, draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::{Color, Plane, Styles};
use dracon_terminal_engine::framework::animation::AnimationManager;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::progress_bar::ProgressBar;
use dracon_terminal_engine::framework::widgets::progress_ring::ProgressRing;
use dracon_terminal_engine::framework::widgets::spinner::Spinner;
use dracon_terminal_engine::framework::widgets::{StatusBar, StatusSegment};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::{Cell, RefCell};
use std::time::Duration;

const SIDEBAR_W: u16 = 22;
const DIV_X: u16 = SIDEBAR_W + 2;

type ZoneRect = (u16, u16, u16, u16, usize);

struct BouncingBall {
    x_anim_id: usize,
    y_anim_id: usize,
    color: Color,
    easing_name: &'static str,
}

pub struct AnimationScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    anim_mgr: RefCell<AnimationManager>,
    balls: RefCell<Vec<BouncingBall>>,
    panel_anim_id: RefCell<usize>,
    panel_visible: RefCell<bool>,
    pulse_anim_id: RefCell<usize>,
    // Animated widgets
    progress_bar: RefCell<ProgressBar>,
    progress_ring: RefCell<ProgressRing>,
    spinner: RefCell<Spinner>,
    bar_anim_id: RefCell<usize>,
    ring_anim_id: RefCell<usize>,
    // Demo control
    auto_running: Cell<bool>,
    demo_mode: Cell<usize>, // 0=balls, 1=widgets, 2=all
    dirty: bool,
    area: Cell<Rect>,
    zones: RefCell<Vec<ZoneRect>>,
    status_bar: RefCell<StatusBar>,
}

impl AnimationScene {
    pub fn new(theme: Theme) -> Self {
        let mut anim_mgr = AnimationManager::new();

        let easing_configs = [
            ("Linear", Color::Rgb(255, 107, 107)),
            ("EaseIn", Color::Rgb(107, 203, 119)),
            ("EaseOut", Color::Rgb(77, 150, 255)),
            ("EaseInOut", Color::Rgb(255, 217, 61)),
        ];

        let mut balls = Vec::new();
        for (name, color) in easing_configs {
            let x_id = anim_mgr.start(2.0, 28.0, Duration::from_secs(2));
            let y_id = anim_mgr.start(4.0, 16.0, Duration::from_millis(700));
            balls.push(BouncingBall {
                x_anim_id: x_id,
                y_anim_id: y_id,
                color,
                easing_name: name,
            });
        }

        let panel_anim_id = anim_mgr.start(0.0, 18.0, Duration::from_millis(400));
        let pulse_anim_id = anim_mgr.start(0.0, 1.0, Duration::from_secs(2));
        let bar_anim_id = anim_mgr.start(0.0, 100.0, Duration::from_secs(4));
        let ring_anim_id = anim_mgr.start(0.0, 1.0, Duration::from_secs(6));

        let progress_bar = ProgressBar::new(WidgetId::new(1)).with_theme(theme.clone());

        let progress_ring = ProgressRing::new(0.0)
            .with_theme(theme.clone())
            .with_size(6)
            .show_percentage(true)
            .with_label("Ring");

        let spinner = Spinner::new(WidgetId::new(2)).with_theme(theme.clone());

        let status_bar = StatusBar::new(WidgetId::new(2002))
            .add_segment(StatusSegment::new(
                "1-3:demo | SPACE:auto | r:reset | F1:help | Esc:back",
            ))
            .with_theme(theme.clone());

        Self {
            theme,
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            anim_mgr: RefCell::new(anim_mgr),
            balls: RefCell::new(balls),
            panel_anim_id: RefCell::new(panel_anim_id),
            panel_visible: RefCell::new(true),
            pulse_anim_id: RefCell::new(pulse_anim_id),
            progress_bar: RefCell::new(progress_bar),
            progress_ring: RefCell::new(progress_ring),
            spinner: RefCell::new(spinner),
            bar_anim_id: RefCell::new(bar_anim_id),
            ring_anim_id: RefCell::new(ring_anim_id),
            auto_running: Cell::new(true),
            demo_mode: Cell::new(2),
            dirty: true,
            area: Cell::new(Rect::new(0, 0, 80, 24)),
            zones: RefCell::new(Vec::new()),
            status_bar: RefCell::new(status_bar),
        }
    }

    fn tick_animations(&self) {
        let mut mgr = self.anim_mgr.borrow_mut();
        mgr.tick();

        let auto = self.auto_running.get();
        let demo = self.demo_mode.get();

        // Bouncing balls
        if demo == 0 || demo == 2 {
            let mut balls = self.balls.borrow_mut();
            for ball in balls.iter_mut() {
                if mgr.is_done(ball.x_anim_id) {
                    let current = mgr.value(ball.x_anim_id).unwrap_or(14.0);
                    let target = if current > 14.0 { 2.0 } else { 28.0 };
                    ball.x_anim_id = mgr.start(current, target, Duration::from_secs(2));
                }
                if mgr.is_done(ball.y_anim_id) {
                    let current = mgr.value(ball.y_anim_id).unwrap_or(10.0);
                    let target = if current > 10.0 { 4.0 } else { 16.0 };
                    ball.y_anim_id = mgr.start(current, target, Duration::from_millis(700));
                }
            }
        }

        // Sliding panel
        if (demo == 0 || demo == 2) && mgr.is_done(*self.panel_anim_id.borrow()) {
            // Restart panel animation
            let visible = *self.panel_visible.borrow();
            let (from, to) = if visible { (0.0, 18.0) } else { (18.0, 0.0) };
            *self.panel_anim_id.borrow_mut() = mgr.start(from, to, Duration::from_millis(400));
            *self.panel_visible.borrow_mut() = !visible;
        }

        // Pulse
        if mgr.is_done(*self.pulse_anim_id.borrow()) {
            let current = mgr.value(*self.pulse_anim_id.borrow()).unwrap_or(1.0);
            let target = if current > 0.5 { 0.0 } else { 1.0 };
            *self.pulse_anim_id.borrow_mut() = mgr.start(current, target, Duration::from_secs(2));
        }

        // Progress bar
        if auto && (demo == 1 || demo == 2) {
            if mgr.is_done(*self.bar_anim_id.borrow()) {
                *self.bar_anim_id.borrow_mut() = mgr.start(0.0, 100.0, Duration::from_secs(4));
            }
            let val = mgr.value(*self.bar_anim_id.borrow()).unwrap_or(0.0) as f32;
            self.progress_bar.borrow_mut().set_progress(val / 100.0);
        }

        // Progress ring
        if auto && (demo == 1 || demo == 2) {
            if mgr.is_done(*self.ring_anim_id.borrow()) {
                *self.ring_anim_id.borrow_mut() = mgr.start(0.0, 1.0, Duration::from_secs(6));
            }
            let val = mgr.value(*self.ring_anim_id.borrow()).unwrap_or(0.0);
            self.progress_ring.borrow_mut().set_progress(val);
        }

        // Spinner tick
        self.spinner.borrow_mut().tick();
    }
}

impl Scene for AnimationScene {
    fn scene_id(&self) -> &str {
        "animation"
    }

    fn render(&self, area: Rect) -> self::Plane {
        self.area.set(area);
        self.tick_animations();

        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Clear zone registry
        self.zones.borrow_mut().clear();

        // ── Header ──────────────────────────────────────────────────────────
        draw_text(
            &mut plane,
            2,
            0,
            " Animation Playground ",
            t.primary,
            t.bg,
            true,
        );
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(
            &mut plane,
            area.width.saturating_sub(theme_label.len() as u16 + 2),
            0,
            &theme_label,
            t.secondary,
            t.bg,
            false,
        );
        if self.auto_running.get() {
            draw_text(
                &mut plane,
                DIV_X.saturating_sub(6),
                0,
                "▶ AUTO",
                t.primary,
                t.bg,
                true,
            );
        }

        // Divider
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── Left sidebar: controls ─────────────────────────────────────────
        self.render_sidebar(&mut plane, t, area.height);

        // Vertical divider
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * area.width + DIV_X) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // ── Main area: bouncing balls + panel ──────────────────────────────
        let main_x = DIV_X + 2;
        let main_w = area.width.saturating_sub(main_x + 2);

        let mgr = self.anim_mgr.borrow();
        let balls = self.balls.borrow();

        // Section: Bouncing Balls
        let ball_y = 2;
        draw_text(
            &mut plane,
            main_x,
            ball_y,
            "Bouncing Balls",
            t.primary,
            t.bg,
            true,
        );

        let ball_arena_h = 8u16;
        let ball_arena_w = main_w;

        // Draw arena border
        let arena_x = main_x;
        for bx in 0..ball_arena_w {
            let top = ((ball_y + 1) * area.width + arena_x + bx) as usize;
            if top < plane.cells.len() {
                plane.cells[top].char = '─';
                plane.cells[top].fg = t.outline;
            }
            let bot = ((ball_y + ball_arena_h) * area.width + arena_x + bx) as usize;
            if bot < plane.cells.len() {
                plane.cells[bot].char = '─';
                plane.cells[bot].fg = t.outline;
            }
        }
        for by in 0..ball_arena_h {
            let left = ((ball_y + 1 + by) * area.width + arena_x) as usize;
            if left < plane.cells.len() {
                plane.cells[left].char = '│';
                plane.cells[left].fg = t.outline;
            }
            let right = ((ball_y + 1 + by) * area.width + arena_x + ball_arena_w - 1) as usize;
            if right < plane.cells.len() {
                plane.cells[right].char = '│';
                plane.cells[right].fg = t.outline;
            }
        }

        for ball in balls.iter() {
            let x_val = mgr.value(ball.x_anim_id).unwrap_or(2.0);
            let y_val = mgr.value(ball.y_anim_id).unwrap_or(4.0);
            let bx = (x_val / 30.0 * ball_arena_w as f64) as u16 + arena_x + 1;
            let by = (y_val / 20.0 * ball_arena_h as f64) as u16 + ball_y + 2;

            if bx > arena_x
                && bx < arena_x + ball_arena_w - 1
                && by > ball_y
                && by < ball_y + ball_arena_h
            {
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

        // Easing labels
        for (i, ball) in balls.iter().enumerate() {
            let label_x = main_x + 2 + i as u16 * (ball_arena_w / 4);
            let label_y = ball_y + ball_arena_h + 1;
            draw_text(
                &mut plane,
                label_x,
                label_y,
                ball.easing_name,
                ball.color,
                t.bg,
                false,
            );
        }

        // Section: Sliding Panel
        let panel_y = ball_y + ball_arena_h + 3;
        if panel_y + 7 < area.height.saturating_sub(2) {
            draw_text(
                &mut plane,
                main_x,
                panel_y,
                "Sliding Panel",
                t.primary,
                t.bg,
                true,
            );

            let panel_offset = mgr.value(*self.panel_anim_id.borrow()).unwrap_or(0.0) as u16;
            let panel_w = 20;
            let visible_w = panel_offset.min(panel_w);

            if visible_w > 0 {
                let py = panel_y + 2;
                let ph = 5u16.min(area.height.saturating_sub(panel_y + 3));

                // Panel background
                for dy in 0..ph {
                    for dx in 0..visible_w {
                        let px = main_x + dx;
                        let py2 = py + dy;
                        if px < area.width && py2 < area.height {
                            let idx = (py2 * area.width + px) as usize;
                            if idx < plane.cells.len() {
                                plane.cells[idx].bg = t.surface_elevated;
                                plane.cells[idx].transparent = false;
                            }
                        }
                    }
                }

                // Panel content
                if visible_w > 4 {
                    let lines = [
                        "┌──────────────┐",
                        "│ Animated     │",
                        "│ Panel slides │",
                        "│ in/out       │",
                        "└──────────────┘",
                    ];
                    for (li, line) in lines.iter().enumerate() {
                        let ly = py + li as u16;
                        if ly < area.height.saturating_sub(2) {
                            for (ci, ch) in line.chars().enumerate() {
                                let lx = main_x + 1 + ci as u16;
                                if lx < main_x + visible_w && lx < area.width {
                                    let idx = (ly * area.width + lx) as usize;
                                    if idx < plane.cells.len() {
                                        plane.cells[idx].char = ch;
                                        plane.cells[idx].fg = if "│┌┐└┘─".contains(ch) {
                                            t.outline
                                        } else {
                                            t.fg
                                        };
                                        plane.cells[idx].bg = t.surface_elevated;
                                        plane.cells[idx].transparent = false;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Section: Animated Widgets
        let widget_y = panel_y + 8;
        if widget_y + 6 < area.height.saturating_sub(3) {
            draw_text(
                &mut plane,
                main_x,
                widget_y,
                "Animated Widgets",
                t.primary,
                t.bg,
                true,
            );

            // Progress Bar
            let bar_area_y = widget_y + 2;
            let bar_w = main_w / 2 - 2;
            let bar_area = Rect::new(main_x, bar_area_y, bar_w, 3);
            let bar_plane = self.progress_bar.borrow().render(bar_area);
            blit_to(&mut plane, &bar_plane, main_x as usize, bar_area_y as usize);

            // Progress Ring
            let ring_area = Rect::new(main_x + bar_w + 2, bar_area_y, 10, 5);
            let ring_plane = self.progress_ring.borrow().render(ring_area);
            blit_to(
                &mut plane,
                &ring_plane,
                (main_x + bar_w + 2) as usize,
                bar_area_y as usize,
            );

            // Spinner
            let spin_y = bar_area_y + 5;
            if spin_y < area.height.saturating_sub(3) {
                draw_text(
                    &mut plane, main_x, spin_y, "Spinner:", t.fg_muted, t.bg, false,
                );
                let spin_area = Rect::new(main_x + 10, spin_y, 8, 1);
                let spin_plane = self.spinner.borrow().render(spin_area);
                blit_to(&mut plane, &spin_plane, 10, spin_y as usize);
            }

            // Pulse indicator
            let pulse_val = mgr.value(*self.pulse_anim_id.borrow()).unwrap_or(0.0);
            let pulse_y = bar_area_y + 5;
            let pulse_x = main_x + bar_w / 2;
            let pulse_idx = (pulse_y * area.width + pulse_x) as usize;
            if pulse_idx < plane.cells.len() {
                plane.cells[pulse_idx].char = '◆';
                plane.cells[pulse_idx].fg = Color::Rgb(
                    (255.0 * pulse_val) as u8,
                    (100.0 * (1.0 - pulse_val)) as u8,
                    255,
                );
                plane.cells[pulse_idx].style = Styles::BOLD;
                plane.cells[pulse_idx].transparent = false;
            }
        }

        // ── Footer ────────────────────────────────────────────────────────
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " 1-3:demo | SPACE:auto | r:reset | {}:help | {}:back ",
            help_key, back_key,
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
            let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(
                &mut plane,
                area,
                &self.theme,
                "Animation Playground — Help",
                &[
                    ("1", "Bouncing balls only"),
                    ("2", "Animated widgets only"),
                    ("3", "All animations"),
                    ("SPACE", "Toggle auto-play"),
                    ("r", "Reset all animations"),
                    (help_key, "Toggle this help"),
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

        match key.code {
            KeyCode::Char('1') => {
                self.demo_mode.set(0);
                self.dirty = true;
                true
            }
            KeyCode::Char('2') => {
                self.demo_mode.set(1);
                self.dirty = true;
                true
            }
            KeyCode::Char('3') => {
                self.demo_mode.set(2);
                self.dirty = true;
                true
            }
            KeyCode::Char(' ') if key.modifiers.is_empty() => {
                self.auto_running.set(!self.auto_running.get());
                self.dirty = true;
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                let mut mgr = self.anim_mgr.borrow_mut();
                mgr.clear();
                drop(mgr);

                let mut balls = self.balls.borrow_mut();
                let mut mgr = self.anim_mgr.borrow_mut();
                for ball in balls.iter_mut() {
                    ball.x_anim_id = mgr.start(2.0, 28.0, Duration::from_secs(2));
                    ball.y_anim_id = mgr.start(4.0, 16.0, Duration::from_millis(700));
                }
                *self.panel_anim_id.borrow_mut() = mgr.start(0.0, 18.0, Duration::from_millis(400));
                *self.panel_visible.borrow_mut() = true;
                *self.pulse_anim_id.borrow_mut() = mgr.start(0.0, 1.0, Duration::from_secs(2));
                *self.bar_anim_id.borrow_mut() = mgr.start(0.0, 100.0, Duration::from_secs(4));
                *self.ring_anim_id.borrow_mut() = mgr.start(0.0, 1.0, Duration::from_secs(6));
                self.auto_running.set(true);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        match kind {
            MouseEventKind::Down(_) => {
                // Demo mode buttons (sidebar rows 2-4)
                if col < DIV_X && (2..=4).contains(&row) {
                    let demo = (row - 2) as usize;
                    self.demo_mode.set(demo.min(2));
                    self.dirty = true;
                    return true;
                }
                // Toggle auto (row 6)
                if col < DIV_X && row == 6 {
                    self.auto_running.set(!self.auto_running.get());
                    self.dirty = true;
                    return true;
                }
                // Reset (row 8)
                if col < DIV_X && row == 8 {
                    let mut mgr = self.anim_mgr.borrow_mut();
                    mgr.clear();
                    drop(mgr);

                    let mut balls = self.balls.borrow_mut();
                    let mut mgr = self.anim_mgr.borrow_mut();
                    for ball in balls.iter_mut() {
                        ball.x_anim_id = mgr.start(2.0, 28.0, Duration::from_secs(2));
                        ball.y_anim_id = mgr.start(4.0, 16.0, Duration::from_millis(700));
                    }
                    *self.panel_anim_id.borrow_mut() =
                        mgr.start(0.0, 18.0, Duration::from_millis(400));
                    *self.panel_visible.borrow_mut() = true;
                    *self.pulse_anim_id.borrow_mut() = mgr.start(0.0, 1.0, Duration::from_secs(2));
                    *self.bar_anim_id.borrow_mut() = mgr.start(0.0, 100.0, Duration::from_secs(4));
                    *self.ring_anim_id.borrow_mut() = mgr.start(0.0, 1.0, Duration::from_secs(6));
                    self.auto_running.set(true);
                    self.dirty = true;
                    return true;
                }

                // Click in main area: toggle auto
                if col >= DIV_X {
                    self.auto_running.set(!self.auto_running.get());
                    self.dirty = true;
                    return true;
                }
            }
            MouseEventKind::Moved => {
                self.dirty = true;
                return true;
            }
            _ => {}
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.progress_bar.borrow_mut().on_theme_change(theme);
        self.progress_ring.borrow_mut().on_theme_change(theme);
        self.spinner.borrow_mut().on_theme_change(theme);
        self.status_bar.borrow_mut().on_theme_change(theme);
    }

    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

impl AnimationScene {
    fn render_sidebar(&self, plane: &mut Plane, t: &Theme, area_h: u16) {
        let sx = 2u16;

        // Demo mode selector
        let demo_labels = ["1: Balls", "2: Widgets", "3: All"];
        let current_demo = self.demo_mode.get();

        draw_text(plane, sx, 2, "Demo Mode", t.primary, t.bg, true);
        for (i, label) in demo_labels.iter().enumerate() {
            let is_active = i == current_demo;
            let y = 3 + i as u16;
            let bg = if is_active { t.primary } else { t.bg };
            let fg = if is_active { t.bg } else { t.fg_muted };
            for cx in 0..SIDEBAR_W {
                let idx = (y * plane.width + sx + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].transparent = false;
                }
            }
            draw_text_clipped(plane, sx + 1, y, label, sx + SIDEBAR_W, fg, bg, is_active);
        }

        // Auto toggle
        let auto_y = 7;
        draw_text(plane, sx, auto_y, "Playback", t.secondary, t.bg, true);
        let auto_running = self.auto_running.get();
        let auto_bg = if auto_running { t.success } else { t.fg_muted };
        let auto_text = if auto_running {
            "● Running"
        } else {
            "○ Paused "
        };
        for cx in 0..SIDEBAR_W {
            let idx = ((auto_y + 1) * plane.width + sx + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = auto_bg;
                plane.cells[idx].transparent = false;
            }
        }
        let auto_fg = if auto_running { t.bg } else { t.fg };
        draw_text_clipped(
            plane,
            sx + 1,
            auto_y + 1,
            auto_text,
            sx + SIDEBAR_W,
            auto_fg,
            auto_bg,
            true,
        );

        // Reset button
        let reset_y = 10;
        let reset_text = "  r  Reset All";
        for cx in 0..SIDEBAR_W.min(reset_text.len() as u16) {
            let idx = (reset_y * plane.width + sx + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        draw_text(plane, sx, reset_y, reset_text, t.warning, t.surface, false);

        // Easing reference
        let ref_y = 13;
        if ref_y < area_h.saturating_sub(4) {
            draw_text(plane, sx, ref_y, "Easing", t.secondary, t.bg, true);
            let easings = [
                ("Linear", Color::Rgb(255, 107, 107)),
                ("EaseIn", Color::Rgb(107, 203, 119)),
                ("EaseOut", Color::Rgb(77, 150, 255)),
                ("EaseInOut", Color::Rgb(255, 217, 61)),
            ];
            for (i, (name, color)) in easings.iter().enumerate() {
                let ey = ref_y + 1 + i as u16;
                let swatch_idx = (ey * plane.width + sx) as usize;
                if swatch_idx < plane.cells.len() {
                    plane.cells[swatch_idx].char = '■';
                    plane.cells[swatch_idx].fg = *color;
                }
                draw_text_clipped(plane, sx + 2, ey, name, sx + SIDEBAR_W, t.fg, t.bg, false);
            }
        }

        // Click hint
        let hint_y = area_h.saturating_sub(4);
        if hint_y > ref_y + 6 {
            draw_text(
                plane,
                sx,
                hint_y,
                "Click main area",
                t.fg_muted,
                t.bg,
                false,
            );
            draw_text(
                plane,
                sx,
                hint_y + 1,
                "to toggle auto",
                t.fg_muted,
                t.bg,
                false,
            );
        }
    }
}

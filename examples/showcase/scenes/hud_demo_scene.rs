//! HUD Demo scene — HUD overlay + StatusBar + Gauge + Spinner.
//!
//! A game-style HUD overlay showing health, ammo, score, and minimap.
//! Demonstrates the HUD widget's render_text and render_gauge methods
//! overlaid on a simulated game view.

#![allow(dead_code)]

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    Divider, Gauge, Hud, Label, Spinner, StatusBar, StatusSegment,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

pub struct HudDemoScene {
    theme: Theme,
    keybindings: KeybindingSet,
    hud: RefCell<Hud>,
    // Game state
    health: f32,
    ammo: f32,
    shield: f32,
    score: u32,
    level: u32,
    wave: u32,
    // Widgets
    health_gauge: RefCell<Gauge>,
    shield_gauge: RefCell<Gauge>,
    spinner: RefCell<Spinner>,
    status_bar: RefCell<StatusBar>,
    show_help: bool,
    dirty: bool,
}

impl HudDemoScene {
    pub fn new(theme: Theme) -> Self {
        let hud = Hud::new_with_id(WidgetId::new(1100), 50)
            .with_size(40, 12)
            .with_theme(theme.clone());

        let health_gauge = Gauge::with_id(WidgetId::new(1110), "HP")
            .max(100.0)
            .crit_threshold(25.0)
            .with_theme(theme.clone());
        let shield_gauge = Gauge::with_id(WidgetId::new(1111), "SH")
            .max(100.0)
            .warn_threshold(50.0)
            .with_theme(theme.clone());

        let spinner = Spinner::new(WidgetId::new(1120)).with_theme(theme.clone());

        let status_bar = StatusBar::new(WidgetId::new(1130))
            .add_segment(StatusSegment::new("H/D/S/A: modify | Space: tick | F1: help | Esc: back"))
            .add_segment(StatusSegment::new("HUD Demo"))
            .with_theme(theme.clone());

        Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            hud: RefCell::new(hud),
            health: 100.0,
            ammo: 30.0,
            shield: 75.0,
            score: 0,
            level: 1,
            wave: 1,
            health_gauge: RefCell::new(health_gauge),
            shield_gauge: RefCell::new(shield_gauge),
            spinner: RefCell::new(spinner),
            status_bar: RefCell::new(status_bar),
            show_help: false,
            dirty: true,
        }
    }
}

impl Scene for HudDemoScene {
    fn on_enter(&mut self) {}
    fn on_exit(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // Title
        let title = Label::new("HUD Demo").with_style(Styles::BOLD).with_theme(t.clone());
        let title_plane = title.render(Rect::new(0, 0, 10, 1));
        blit_to(&mut plane, &title_plane, 1, 0);
        draw_text(&mut plane, 12, 0, "— HUD · Gauge · Spinner (game overlay)", t.fg_muted, t.bg, false);

        // ── Simulated game area (center) ──────────────────────────
        let game_y = 2;
        let game_h = area.height.saturating_sub(6);
        let game_w = area.width;

        // Dark game background
        for y in game_y..game_y + game_h {
            for x in 0..game_w {
                let idx = (y as usize) * area.width as usize + x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = Color::Rgb(10, 10, 20);
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Game content text
        let center_x = game_w / 2;
        let center_y = game_y + game_h / 2;
        draw_text(&mut plane, center_x.saturating_sub(8), center_y, "⚡ ARENA ZONE ⚡", t.primary, Color::Rgb(10, 10, 20), true);
        draw_text(&mut plane, center_x.saturating_sub(7), center_y + 1, "Survive the waves!", t.fg_muted, Color::Rgb(10, 10, 20), false);

        // Enemy indicators (simple dots)
        let enemies = ["👾", "👾", "👾", "👾", "👾"];
        for (i, e) in enemies.iter().enumerate() {
            let ex = 5 + i as u16 * 8;
            let ey = game_y + 3;
            draw_text(&mut plane, ex, ey, e, t.fg, Color::Rgb(10, 10, 20), false);
        }

        // ── HUD overlay (top-left of game area) ───────────────────
        let hud = self.hud.borrow();
        let hud_bg = Color::Rgb(0, 0, 0);
        let hud_fg = Color::Rgb(220, 220, 220);

        // Score line
        let score_plane = hud.render_text(
            1, game_y + 1,
            &format!("SCORE: {:06}  LVL:{}  WAVE:{}", self.score, self.level, self.wave),
            hud_fg, hud_bg,
        );
        blit_to(&mut plane, &score_plane, 1, (game_y + 1) as usize);

        // Health gauge via HUD
        let hp_plane = hud.render_gauge(1, game_y + 3, "HP", self.health, 100.0, 20);
        blit_to(&mut plane, &hp_plane, 1, (game_y + 3) as usize);

        // Shield gauge via HUD
        let sh_plane = hud.render_gauge(1, game_y + 4, "SH", self.shield, 100.0, 20);
        blit_to(&mut plane, &sh_plane, 1, (game_y + 4) as usize);

        // Ammo via HUD
        let ammo_plane = hud.render_text(
            1, game_y + 6,
            &format!("AMMO: {:.0}/30", self.ammo),
            hud_fg, hud_bg,
        );
        blit_to(&mut plane, &ammo_plane, 1, (game_y + 6) as usize);

        // ── Bottom panel: Gauge widgets + Spinner ──────────────────
        let panel_y = game_y + game_h;
        let div = Divider::new().with_label("Dashboard").with_theme(t.clone());
        let div_plane = div.render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &div_plane, 0, panel_y as usize);

        // Gauges
        self.health_gauge.borrow_mut().set_value(self.health as f64);
        self.shield_gauge.borrow_mut().set_value(self.shield as f64);

        let gauge_y = panel_y + 1;
        let half_w = area.width / 2;
        let hp_area = Rect::new(1, gauge_y, half_w.saturating_sub(2), 1);
        self.health_gauge.borrow_mut().set_area(hp_area);
        let hp_plane = self.health_gauge.borrow().render(hp_area);
        blit_to(&mut plane, &hp_plane, 1, gauge_y as usize);

        let sh_area = Rect::new(half_w + 1, gauge_y, half_w.saturating_sub(2), 1);
        self.shield_gauge.borrow_mut().set_area(sh_area);
        let sh_plane = self.shield_gauge.borrow().render(sh_area);
        blit_to(&mut plane, &sh_plane, (half_w + 1) as usize, gauge_y as usize);

        // Spinner + status
        let sp_y = gauge_y + 1;
        let sp_area = Rect::new(1, sp_y, 3, 1);
        self.spinner.borrow_mut().set_area(sp_area);
        let sp_plane = self.spinner.borrow().render(sp_area);
        blit_to(&mut plane, &sp_plane, 1, sp_y as usize);
        draw_text(&mut plane, 5, sp_y, "Game running…", t.fg_muted, t.bg, false);

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self.status_bar.borrow().render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        if self.show_help {
            render_help_overlay(&mut plane, area, t, "HUD Demo — Help", &[("H", "Take damage (-10 HP)"), ("D", "Heal (+10 HP)"), ("S", "Deplete shield (-15)"), ("A", "Fire weapon (-1 ammo)"), ("Space", "Tick + score"), ("R", "Reset all stats"), ("F1", "Toggle this help"), ("Esc", "Back")]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::HELP, &key) || self.keybindings.matches(actions::BACK, &key) {
                self.show_help = false; self.dirty = true; return true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) { self.show_help = !self.show_help; self.dirty = true; return true; }
        if self.keybindings.matches(actions::BACK, &key) { return false; }

        match key.code {
            KeyCode::Char('h') if key.modifiers.is_empty() => { self.health = (self.health - 10.0).max(0.0); self.dirty = true; true }
            KeyCode::Char('d') if key.modifiers.is_empty() => { self.health = (self.health + 10.0).min(100.0); self.dirty = true; true }
            KeyCode::Char('s') if key.modifiers.is_empty() => { self.shield = (self.shield - 15.0).max(0.0); self.dirty = true; true }
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                if self.ammo > 0.0 { self.ammo -= 1.0; self.score += 100; }
                if self.ammo <= 0.0 { self.wave += 1; self.ammo = 30.0; }
                self.dirty = true; true
            }
            KeyCode::Char(' ') => {
                self.spinner.borrow_mut().tick();
                self.score += 10;
                if self.score > 0 && self.score.is_multiple_of(1000) { self.level += 1; }
                self.dirty = true; true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.health = 100.0; self.shield = 75.0; self.ammo = 30.0;
                self.score = 0; self.level = 1; self.wave = 1;
                self.dirty = true; true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let _ = (col, row, kind);
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.hud.borrow_mut().on_theme_change(theme);
        self.health_gauge.borrow_mut().on_theme_change(theme);
        self.shield_gauge.borrow_mut().on_theme_change(theme);
        self.spinner.borrow_mut().on_theme_change(theme);
        self.status_bar.borrow_mut().on_theme_change(theme);
        self.dirty = true;
    }

    fn scene_id(&self) -> &str { "hud_demo" }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}


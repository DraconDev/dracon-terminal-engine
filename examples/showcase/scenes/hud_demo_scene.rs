//! HUD Demo scene — HUD overlay + StatusBar + Gauge + Spinner.
//!
//! A game-style HUD overlay showing health, ammo, score, and minimap.
//! Demonstrates the HUD widget's render_text and render_gauge methods
//! overlaid on a simulated game view.

#![allow(dead_code)]

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
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
    // Enemies: (x, y, hp, max_hp, kind)
    enemies: Vec<(u16, u16, f32, f32, u8)>,
    // Active damage flash (column, row, frames_remaining, intensity)
    damage_flashes: RefCell<Vec<(u16, u16, u8)>>,
    // Damage numbers (x, y, text, frames_remaining)
    damage_numbers: RefCell<Vec<(u16, u16, String, u8)>>,
    // Combat log
    combat_log: Vec<String>,
    // Hit counter for visual feedback
    hits_taken: u32,
    hits_dealt: u32,
    // Widgets
    health_gauge: RefCell<Gauge>,
    shield_gauge: RefCell<Gauge>,
    spinner: RefCell<Spinner>,
    status_bar: RefCell<StatusBar>,
    show_help: bool,
    dirty: bool,
    // mutable-from-render flag for time-based updates (e.g. flash decay)
    render_dirty: std::cell::Cell<bool>,
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
            .add_segment(StatusSegment::new(
                "H/D/S/A: modify | Space: tick | F1: help | Esc: back",
            ))
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
            // Five enemies, varied HP and positions
            enemies: vec![
                (8, 5, 30.0, 30.0, 0),
                (24, 5, 45.0, 45.0, 1),
                (40, 6, 25.0, 25.0, 2),
                (56, 5, 60.0, 60.0, 3),
                (72, 5, 35.0, 35.0, 4),
            ],
            damage_flashes: RefCell::new(Vec::new()),
            damage_numbers: RefCell::new(Vec::new()),
            combat_log: vec!["[SYS] Arena initialized.".to_string()],
            hits_taken: 0,
            hits_dealt: 0,
            health_gauge: RefCell::new(health_gauge),
            shield_gauge: RefCell::new(shield_gauge),
            spinner: RefCell::new(spinner),
            status_bar: RefCell::new(status_bar),
            show_help: false,
            render_dirty: std::cell::Cell::new(false),
            dirty: true,
        }
    }
}

impl Scene for HudDemoScene {
    fn on_enter(&mut self) {
        // Reset game state when entering the scene
        self.health = 100.0;
        self.ammo = 30.0;
        self.shield = 75.0;
        self.score = 0;
        self.level = 1;
        self.wave = 1;
        self.dirty = true;
    }
    fn on_exit(&mut self) {
        self.show_help = false;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // Title
        let title = Label::new("HUD Demo")
            .with_style(Styles::BOLD)
            .with_theme(t.clone());
        let title_plane = title.render(Rect::new(0, 0, 10, 1));
        blit_to(&mut plane, &title_plane, 1, 0);
        draw_text(
            &mut plane,
            12,
            0,
            "— HUD · Gauge · Spinner (game overlay)",
            t.fg_muted,
            t.bg,
            false,
        );

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
        draw_text(
            &mut plane,
            center_x.saturating_sub(8),
            center_y,
            "⚡ ARENA ZONE ⚡",
            t.primary,
            Color::Rgb(10, 10, 20),
            true,
        );
        draw_text(
            &mut plane,
            center_x.saturating_sub(7),
            center_y + 1,
            "Survive the waves!",
            t.fg_muted,
            Color::Rgb(10, 10, 20),
            false,
        );

        // Enemy indicators with HP bars
        let enemy_chars = ['Z', 'X', 'B', 'Q', 'M'];
        for (i, (ex, ey, hp, max_hp, kind)) in self.enemies.iter().enumerate() {
            let ch = enemy_chars.get(*kind as usize).copied().unwrap_or('?');
            // Enemy glyph
            let color = if *hp > *max_hp * 0.5 {
                t.fg
            } else if *hp > *max_hp * 0.25 {
                t.warning
            } else {
                t.error
            };
            draw_text(
                &mut plane,
                *ex,
                *ey,
                &ch.to_string(),
                color,
                Color::Rgb(10, 10, 20),
                true,
            );
            // HP bar underneath
            let bar_w = 6u16;
            let filled = ((*hp / *max_hp) * bar_w as f32) as u16;
            let mut bar = String::new();
            for j in 0..bar_w {
                if j < filled {
                    bar.push('█');
                } else {
                    bar.push('░');
                }
            }
            draw_text(
                &mut plane,
                ex.saturating_sub(1),
                ey + 1,
                &bar,
                color,
                Color::Rgb(10, 10, 20),
                false,
            );
            // HP number
            let _ = i; // suppress unused warning
        }

        // Damage flashes (overlay)
        for (fx, fy, intensity) in self.damage_flashes.borrow().iter() {
            let color = match intensity {
                0..=2 => t.error,
                3..=5 => t.warning,
                _ => t.fg,
            };
            let idx = (*fy as usize) * area.width as usize + *fx as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '!';
                plane.cells[idx].fg = color;
                plane.cells[idx].bg = Color::Rgb(10, 10, 20);
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Decay flash intensities
        if !self.damage_flashes.borrow().is_empty() {
            self.damage_flashes
                .borrow_mut()
                .retain_mut(|(_, _, intensity)| {
                    *intensity = intensity.saturating_sub(1);
                    *intensity > 0
                });
            self.render_dirty.set(true);
        }

        // Damage numbers
        for (dx, dy, text, _frames) in self.damage_numbers.borrow().iter() {
            let color = if text.starts_with('+') {
                t.success
            } else {
                t.error
            };
            draw_text(
                &mut plane,
                *dx,
                *dy,
                text,
                color,
                Color::Rgb(10, 10, 20),
                true,
            );
        }
        // Decay damage numbers
        if !self.damage_numbers.borrow().is_empty() {
            self.damage_numbers
                .borrow_mut()
                .retain_mut(|(_, _, _, frames)| {
                    *frames = frames.saturating_sub(1);
                    *frames > 0
                });
            self.render_dirty.set(true);
        }

        // Combat log (bottom of game area)
        let log_y = game_y + game_h.saturating_sub(2);
        let visible_log: Vec<&String> = self.combat_log.iter().rev().take(2).collect();
        for (i, entry) in visible_log.iter().enumerate() {
            let color = if entry.starts_with("[HIT]") {
                t.success
            } else if entry.starts_with("[DMG]") {
                t.error
            } else {
                t.fg_muted
            };
            draw_text(
                &mut plane,
                1,
                log_y + i as u16,
                entry,
                color,
                Color::Rgb(10, 10, 20),
                false,
            );
        }

        // ── HUD overlay (top-left of game area) ───────────────────
        let hud = self.hud.borrow();
        let hud_bg = Color::Rgb(0, 0, 0);
        let hud_fg = Color::Rgb(220, 220, 220);

        // Score line
        let score_plane = hud.render_text(
            1,
            game_y + 1,
            &format!(
                "SCORE: {:06}  LVL:{}  WAVE:{}",
                self.score, self.level, self.wave
            ),
            hud_fg,
            hud_bg,
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
            1,
            game_y + 6,
            &format!("AMMO: {:.0}/30", self.ammo),
            hud_fg,
            hud_bg,
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
        blit_to(
            &mut plane,
            &sh_plane,
            (half_w + 1) as usize,
            gauge_y as usize,
        );

        // Spinner + status
        let sp_y = gauge_y + 1;
        let sp_area = Rect::new(1, sp_y, 3, 1);
        self.spinner.borrow_mut().set_area(sp_area);
        let sp_plane = self.spinner.borrow().render(sp_area);
        blit_to(&mut plane, &sp_plane, 1, sp_y as usize);
        draw_text(
            &mut plane,
            5,
            sp_y,
            "Game running…",
            t.fg_muted,
            t.bg,
            false,
        );

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self
            .status_bar
            .borrow()
            .render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        if self.show_help {
            render_help_overlay(
                &mut plane,
                area,
                t,
                "HUD Demo — Help",
                &[
                    ("H", "Take damage (-10 HP)"),
                    ("D", "Heal (+10 HP)"),
                    ("S", "Deplete shield (-15)"),
                    ("A", "Fire weapon (-1 ammo)"),
                    ("Space", "Tick + score"),
                    ("R", "Reset all stats"),
                    ("F1", "Toggle this help"),
                    ("Esc", "Back"),
                ],
            );
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::HELP, &key)
                || self.keybindings.matches(actions::BACK, &key)
            {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        match key.code {
            KeyCode::Char('h') if key.modifiers.is_empty() => {
                self.health = (self.health - 10.0).max(0.0);
                self.hits_taken += 1;
                self.damage_flashes.borrow_mut().push((2, 3, 8));
                self.combat_log
                    .push(format!("[DMG] Took 10 damage (HP: {:.0})", self.health));
                self.dirty = true;
                true
            }
            KeyCode::Char('d') if key.modifiers.is_empty() => {
                self.health = (self.health + 10.0).min(100.0);
                self.combat_log
                    .push(format!("[HEAL] +10 HP (now {:.0})", self.health));
                self.dirty = true;
                true
            }
            KeyCode::Char('s') if key.modifiers.is_empty() => {
                self.shield = (self.shield - 15.0).max(0.0);
                self.damage_flashes.borrow_mut().push((2, 4, 6));
                self.combat_log
                    .push(format!("[DMG] Shield -15 (now {:.0})", self.shield));
                self.dirty = true;
                true
            }
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                if self.ammo > 0.0 {
                    self.ammo -= 1.0;
                    // Damage the leftmost alive enemy
                    if let Some(enemy) = self.enemies.iter_mut().find(|e| e.2 > 0.0) {
                        let dmg = 8.0 + (self.level as f32 * 2.0);
                        enemy.2 = (enemy.2 - dmg).max(0.0);
                        self.score += 100;
                        self.hits_dealt += 1;
                        self.damage_flashes.borrow_mut().push((enemy.0, enemy.1, 5));
                        let kind = enemy.4;
                        let enemy_chars = ['Z', 'X', 'B', 'Q', 'M'];
                        let ch = enemy_chars.get(kind as usize).copied().unwrap_or('?');
                        if enemy.2 <= 0.0 {
                            self.score += 500;
                            self.combat_log
                                .push(format!("[HIT] Killed {} +500 score", ch));
                            self.damage_numbers.borrow_mut().push((
                                enemy.0,
                                enemy.1,
                                "+500".to_string(),
                                6,
                            ));
                        } else {
                            self.combat_log.push(format!("[HIT] {} -{:.0} HP", ch, dmg));
                            self.damage_numbers.borrow_mut().push((
                                enemy.0,
                                enemy.1,
                                format!("-{:.0}", dmg),
                                5,
                            ));
                        }
                    }
                }
                if self.ammo <= 0.0 {
                    self.wave += 1;
                    self.ammo = 30.0;
                    self.combat_log
                        .push(format!("[SYS] Wave {} starting", self.wave));
                }
                // Add particle effect at enemy position when hit
                if let Some(enemy) = self.enemies.iter().find(|e| e.2 > 0.0) {
                    for dx in -1..=1 {
                        for dy in -1..=1 {
                            let px = (enemy.0 as i16 + dx).max(0) as u16;
                            let py = (enemy.1 as i16 + dy).max(0) as u16;
                            self.damage_flashes.borrow_mut().push((px, py, 4));
                        }
                    }
                }
                self.dirty = true;
                true
            }
            KeyCode::Char(' ') => {
                self.spinner.borrow_mut().tick();
                self.score += 10;
                if self.score > 0 && self.score.is_multiple_of(1000) {
                    self.level += 1;
                    self.combat_log
                        .push(format!("[SYS] Level up! Now level {}", self.level));
                }
                // Auto-regen shield slowly on idle tick
                if self.shield < 100.0 {
                    self.shield = (self.shield + 2.0).min(100.0);
                    self.damage_flashes.borrow_mut().push((1, 4, 3));
                }
                self.dirty = true;
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.health = 100.0;
                self.shield = 75.0;
                self.ammo = 30.0;
                self.score = 0;
                self.level = 1;
                self.wave = 1;
                self.hits_taken = 0;
                self.hits_dealt = 0;
                self.damage_flashes.borrow_mut().clear();
                self.combat_log = vec!["[SYS] Arena reset.".to_string()];
                for enemy in self.enemies.iter_mut() {
                    enemy.2 = enemy.3;
                }
                self.dirty = true;
                true
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

    fn scene_id(&self) -> &str {
        "hud_demo"
    }
    fn needs_render(&self) -> bool {
        if self.render_dirty.get() {
            self.render_dirty.set(false);
            return true;
        }
        true
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

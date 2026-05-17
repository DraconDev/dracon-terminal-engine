#![allow(missing_docs)]
//! Terminal Arena — Real-time arena survival game.
//!
//! A fully playable game demonstrating the engine's real-time rendering,
//! mouse interaction, particle effects, and compositor performance.
//!
//! Controls:
//!   WASD / Arrows — Move
//!   Click         — Shoot toward cursor
//!   Space         — Pause / Resume
//!   R             — Restart (after game over)
//!   ? / F1        — Toggle help
//!   Esc           — Dismiss help / menu
//!   Ctrl+Q        — Quit

use dracon_terminal_engine::backend::tty::poll_input;
use dracon_terminal_engine::compositor::engine::Compositor;
use dracon_terminal_engine::compositor::plane::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::core::terminal::Terminal;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::input::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEventKind};
use dracon_terminal_engine::input::parser::Parser;
use signal_hook::consts::signal::SIGINT;
use std::io::{self, Read, Write};
use std::os::fd::AsFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

const TARGET_FPS: f32 = 60.0;
const PLAYER_SPEED: f32 = 22.0;      // cells per second
const PLAYER_MAX_HP: i32 = 100;
const PROJECTILE_SPEED: f32 = 45.0;
const PROJECTILE_DAMAGE: i32 = 25;
const PROJECTILE_LIFE: f32 = 1.5;
const SPAWN_BASE_INTERVAL: f32 = 2.0;
const SPAWN_MIN_INTERVAL: f32 = 0.4;
const INVINCIBILITY_TIME: f32 = 1.0;
const KNOCKBACK_FORCE: f32 = 8.0;
const XP_PER_KILL: u32 = 10;
const XP_TO_LEVEL: u32 = 50;

// ═══════════════════════════════════════════════════════════════════════════════
// Math
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, Default)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self { Self { x, y } }
    fn len_sq(self) -> f32 { self.x * self.x + self.y * self.y }
    fn len(self) -> f32 { self.len_sq().sqrt() }
    fn normalized(self) -> Self {
        let len = self.len();
        if len > 0.001 {
            Self { x: self.x / len, y: self.y / len }
        } else {
            Self::new(1.0, 0.0)
        }
    }
    fn dist_sq(self, other: Vec2) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y) }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y) }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self { Self::new(self.x * rhs, self.y * rhs) }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Game Entities
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq)]
enum EnemyKind {
    Grunt,   // fast, weak
    Tank,    // slow, high HP
    Swarmer, // tiny, very fast, low HP
}

impl EnemyKind {
    fn stats(self) -> (i32, f32, char, Color, u32) {
        // (hp, speed, glyph, color, score)
        match self {
            EnemyKind::Grunt => (30, 3.5, 'g', Color::Rgb(100, 200, 100), 10),
            EnemyKind::Tank => (80, 1.8, 'T', Color::Rgb(200, 80, 80), 25),
            EnemyKind::Swarmer => (10, 6.0, 's', Color::Rgb(200, 200, 80), 5),
        }
    }
    fn size_cells(self) -> u16 {
        match self {
            EnemyKind::Grunt => 1,
            EnemyKind::Tank => 2,
            EnemyKind::Swarmer => 1,
        }
    }
}

struct Enemy {
    pos: Vec2,
    vel: Vec2,
    hp: i32,
    max_hp: i32,
    speed: f32,
    kind: EnemyKind,
    score_value: u32,
    glyph: char,
    color: Color,
    size: u16,
    flash_timer: f32,
}

struct Projectile {
    pos: Vec2,
    vel: Vec2,
    damage: i32,
    life: f32,
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    color: Color,
    char: char,
}

struct DamageNumber {
    pos: Vec2,
    text: String,
    life: f32,
    max_life: f32,
    color: Color,
}

struct Player {
    pos: Vec2,
    hp: i32,
    max_hp: i32,
    level: u32,
    xp: u32,
    xp_to_next: u32,
    invincible_timer: f32,
    kills: u32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Game State
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq)]
enum GamePhase {
    Playing,
    Paused,
    GameOver,
}

struct GameState {
    player: Player,
    enemies: Vec<Enemy>,
    projectiles: Vec<Projectile>,
    particles: Vec<Particle>,
    damage_numbers: Vec<DamageNumber>,
    score: u32,
    wave: u32,
    game_time: f32,
    spawn_timer: f32,
    spawn_interval: f32,
    phase: GamePhase,
    screen_shake: f32,
    mouse_pos: Vec2,
}

impl GameState {
    fn new(w: u16, h: u16) -> Self {
        Self {
            player: Player {
                pos: Vec2::new(w as f32 / 2.0, h as f32 / 2.0),
                hp: PLAYER_MAX_HP,
                max_hp: PLAYER_MAX_HP,
                level: 1,
                xp: 0,
                xp_to_next: XP_TO_LEVEL,
                invincible_timer: 0.0,
                kills: 0,
            },
            enemies: Vec::new(),
            projectiles: Vec::new(),
            particles: Vec::new(),
            damage_numbers: Vec::new(),
            score: 0,
            wave: 1,
            game_time: 0.0,
            spawn_timer: 1.0,
            spawn_interval: SPAWN_BASE_INTERVAL,
            phase: GamePhase::Playing,
            screen_shake: 0.0,
            mouse_pos: Vec2::new(0.0, 0.0),
        }
    }

    fn reset(&mut self, w: u16, h: u16) {
        *self = Self::new(w, h);
    }

    fn spawn_enemy(&mut self, w: u16, h: u16) {
        // Pick spawn position at a random edge
        let edge = rand::random::<u8>() % 4;
        let margin = 2.0;
        let pos = match edge {
            0 => Vec2::new(margin, rand::random::<f32>() * h as f32),           // left
            1 => Vec2::new(w as f32 - margin, rand::random::<f32>() * h as f32), // right
            2 => Vec2::new(rand::random::<f32>() * w as f32, margin),           // top
            _ => Vec2::new(rand::random::<f32>() * w as f32, h as f32 - margin), // bottom
        };

        // Pick enemy kind based on wave
        let roll = rand::random::<f32>();
        let kind = if self.wave >= 5 && roll < 0.15 {
            EnemyKind::Tank
        } else if self.wave >= 3 && roll < 0.35 {
            EnemyKind::Swarmer
        } else {
            EnemyKind::Grunt
        };

        let (base_hp, speed, glyph, color, score) = kind.stats();
        let size = kind.size_cells();
        let hp = (base_hp as f32 * (1.0 + self.wave as f32 * 0.15)) as i32;

        self.enemies.push(Enemy {
            pos,
            vel: Vec2::new(0.0, 0.0),
            hp,
            max_hp: hp,
            speed: speed * (1.0 + self.wave as f32 * 0.05),
            kind,
            score_value: score + self.wave * 2,
            glyph,
            color,
            size,
            flash_timer: 0.0,
        });
    }

    fn shoot(&mut self, target_x: f32, target_y: f32) {
        let target = Vec2::new(target_x, target_y);
        let dir = (target - self.player.pos).normalized();
        self.projectiles.push(Projectile {
            pos: self.player.pos,
            vel: dir * PROJECTILE_SPEED,
            damage: PROJECTILE_DAMAGE + (self.player.level * 5) as i32,
            life: PROJECTILE_LIFE,
        });
        // Muzzle flash particles
        for _ in 0..3 {
            let angle = rand::random::<f32>() * std::f32::consts::TAU;
            let speed = 5.0 + rand::random::<f32>() * 10.0;
            self.particles.push(Particle {
                pos: self.player.pos,
                vel: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                life: 0.2 + rand::random::<f32>() * 0.3,
                max_life: 0.5,
                color: Color::Rgb(255, 220, 100),
                char: ['*', '+', '.'][rand::random::<usize>() % 3],
            });
        }
    }

    fn spawn_hit_particles(&mut self, pos: Vec2, color: Color, count: usize) {
        for _ in 0..count {
            let angle = rand::random::<f32>() * std::f32::consts::TAU;
            let speed = 8.0 + rand::random::<f32>() * 20.0;
            self.particles.push(Particle {
                pos,
                vel: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                life: 0.3 + rand::random::<f32>() * 0.5,
                max_life: 0.8,
                color,
                char: ['*', '+', '·', '•'][rand::random::<usize>() % 4],
            });
        }
    }

    fn spawn_death_particles(&mut self, pos: Vec2, color: Color) {
        for _ in 0..12 {
            let angle = rand::random::<f32>() * std::f32::consts::TAU;
            let speed = 10.0 + rand::random::<f32>() * 25.0;
            self.particles.push(Particle {
                pos,
                vel: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                life: 0.4 + rand::random::<f32>() * 0.6,
                max_life: 1.0,
                color,
                char: ['*', '+', '·', '•', '░'][rand::random::<usize>() % 5],
            });
        }
    }

    fn add_damage_number(&mut self, pos: Vec2, amount: i32, color: Color) {
        self.damage_numbers.push(DamageNumber {
            pos,
            text: format!("{}", amount),
            life: 0.8,
            max_life: 0.8,
            color,
        });
    }

    fn update(&mut self, dt: f32, w: u16, h: u16) {
        if self.phase != GamePhase::Playing {
            return;
        }

        self.game_time += dt;
        self.screen_shake = (self.screen_shake - dt * 5.0).max(0.0);

        // Update player invincibility
        if self.player.invincible_timer > 0.0 {
            self.player.invincible_timer -= dt;
        }

        // Spawn enemies
        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            self.spawn_enemy(w, h);
            // Spawn multiple at higher waves
            let extra_spawns = (self.wave as f32 / 4.0) as usize;
            for _ in 0..extra_spawns {
                self.spawn_enemy(w, h);
            }
            self.spawn_interval = (SPAWN_BASE_INTERVAL - self.game_time * 0.02)
                .max(SPAWN_MIN_INTERVAL);
            self.spawn_timer = self.spawn_interval;
        }

        // Update wave
        let new_wave = (self.game_time / 20.0) as u32 + 1;
        if new_wave > self.wave {
            self.wave = new_wave;
        }

        // Update projectiles
        for proj in self.projectiles.iter_mut() {
            proj.pos = proj.pos + proj.vel * dt;
            proj.life -= dt;
        }
        self.projectiles.retain(|p| p.life > 0.0);

        // Update enemies (chase player)
        for enemy in self.enemies.iter_mut() {
            let to_player = (self.player.pos - enemy.pos).normalized();
            enemy.vel = to_player * enemy.speed;
            enemy.pos = enemy.pos + enemy.vel * dt;
            enemy.flash_timer = (enemy.flash_timer - dt).max(0.0);

            // Keep in bounds
            enemy.pos.x = enemy.pos.x.clamp(1.0, w as f32 - 2.0);
            enemy.pos.y = enemy.pos.y.clamp(2.0, h as f32 - 2.0);
        }

        // Projectile-enemy collision
        let mut proj_to_remove = Vec::new();
        let mut enemies_to_remove = Vec::new();
        let mut hit_effects: Vec<(Vec2, Color, i32)> = Vec::new();
        for (pi, proj) in self.projectiles.iter().enumerate() {
            for (ei, enemy) in self.enemies.iter_mut().enumerate() {
                let threshold = if enemy.size > 1 { 1.5 } else { 0.8 };
                if proj.pos.dist_sq(enemy.pos) < threshold * threshold {
                    enemy.hp -= proj.damage;
                    enemy.flash_timer = 0.1;
                    proj_to_remove.push(pi);
                    hit_effects.push((enemy.pos, enemy.color, proj.damage));

                    // Knockback
                    let knock_dir = (enemy.pos - self.player.pos).normalized();
                    enemy.pos = enemy.pos + knock_dir * KNOCKBACK_FORCE;

                    if enemy.hp <= 0 && !enemies_to_remove.contains(&ei) {
                        enemies_to_remove.push(ei);
                    }
                    break;
                }
            }
        }

        // Apply hit effects after borrow is released
        for (pos, color, damage) in hit_effects {
            self.spawn_hit_particles(pos, color, 4);
            self.add_damage_number(pos, damage, Color::Rgb(255, 255, 255));
        }

        // Remove dead enemies (in reverse order to preserve indices)
        enemies_to_remove.sort_unstable_by(|a, b| b.cmp(a));
        for &ei in &enemies_to_remove {
            if ei < self.enemies.len() {
                let enemy = self.enemies.remove(ei);
                self.score += enemy.score_value;
                self.player.kills += 1;
                self.player.xp += XP_PER_KILL;

                // Level up
                if self.player.xp >= self.player.xp_to_next {
                    self.player.level += 1;
                    self.player.xp -= self.player.xp_to_next;
                    self.player.xp_to_next = (self.player.xp_to_next as f32 * 1.3) as u32;
                    self.player.max_hp += 10;
                    self.player.hp = self.player.max_hp;
                }

                self.spawn_death_particles(enemy.pos, enemy.color);
                self.screen_shake = 0.3;
            }
        }

        // Remove spent projectiles
        proj_to_remove.sort_unstable_by(|a, b| b.cmp(a));
        proj_to_remove.dedup();
        for &pi in &proj_to_remove {
            if pi < self.projectiles.len() {
                self.projectiles.remove(pi);
            }
        }

        // Enemy-player collision
        let mut player_damage: Option<(i32, Vec2)> = None;
        if self.player.invincible_timer <= 0.0 {
            for enemy in &self.enemies {
                let threshold = if enemy.size > 1 { 1.8 } else { 1.0 };
                if self.player.pos.dist_sq(enemy.pos) < threshold * threshold {
                    let damage = match enemy.kind {
                        EnemyKind::Grunt => 10,
                        EnemyKind::Tank => 20,
                        EnemyKind::Swarmer => 5,
                    };
                    player_damage = Some((damage, enemy.pos));
                    break;
                }
            }
        }
        if let Some((damage, enemy_pos)) = player_damage {
            self.player.hp -= damage;
            self.player.invincible_timer = INVINCIBILITY_TIME;
            self.screen_shake = 0.5;
            self.spawn_hit_particles(self.player.pos, Color::Rgb(255, 50, 50), 8);
            self.add_damage_number(self.player.pos, damage, Color::Rgb(255, 80, 80));

            // Knock player back
            let knock_dir = (self.player.pos - enemy_pos).normalized();
            self.player.pos = self.player.pos + knock_dir * KNOCKBACK_FORCE;

            if self.player.hp <= 0 {
                self.phase = GamePhase::GameOver;
            }
        }

        // Keep player in bounds
        self.player.pos.x = self.player.pos.x.clamp(1.0, w as f32 - 2.0);
        self.player.pos.y = self.player.pos.y.clamp(2.0, h as f32 - 2.0);

        // Update particles
        for p in self.particles.iter_mut() {
            p.pos = p.pos + p.vel * dt;
            p.vel = p.vel * 0.98; // drag
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);

        // Update damage numbers
        for dn in self.damage_numbers.iter_mut() {
            dn.pos.y -= 8.0 * dt; // float up
            dn.life -= dt;
        }
        self.damage_numbers.retain(|dn| dn.life > 0.0);
    }

    fn handle_input(&mut self, key: &KeyEvent, w: u16, h: u16) {
        if self.phase == GamePhase::GameOver {
            if let KeyCode::Char('r') = key.code {
                self.reset(w, h);
            }
            return;
        }

        if self.phase == GamePhase::Paused {
            if let KeyCode::Char(' ') = key.code {
                self.phase = GamePhase::Playing;
            }
            return;
        }

        // Movement
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => dy -= 1.0,
            KeyCode::Char('s') | KeyCode::Down => dy += 1.0,
            KeyCode::Char('a') | KeyCode::Left => dx -= 1.0,
            KeyCode::Char('d') | KeyCode::Right => dx += 1.0,
            KeyCode::Char(' ') => self.phase = GamePhase::Paused,
            _ => {}
        }

        // Normalize diagonal movement
        let move_vec = Vec2::new(dx, dy);
        if move_vec.len_sq() > 0.001 {
            let normalized = move_vec.normalized();
            self.player.pos = self.player.pos + normalized * PLAYER_SPEED * (1.0 / TARGET_FPS);
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) {
        self.mouse_pos = Vec2::new(col as f32, row as f32);
        if self.phase == GamePhase::Playing
            && matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
                self.shoot(col as f32, row as f32);
            }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rendering
// ═══════════════════════════════════════════════════════════════════════════════

fn render_game(p: &mut Plane, state: &GameState, w: u16, h: u16, _fps: u32, kb: &KeybindingSet) {
    let t = Theme::from_env_or(Theme::dark());
    let bg = Color::Rgb(10, 10, 15);
    let floor_dim = Color::Rgb(18, 18, 24);

    // Screen shake offset
    let shake_x = if state.screen_shake > 0.0 {
        ((rand::random::<f32>() - 0.5) * state.screen_shake * 3.0) as i16
    } else {
        0
    };
    let shake_y = if state.screen_shake > 0.0 {
        ((rand::random::<f32>() - 0.5) * state.screen_shake * 3.0) as i16
    } else {
        0
    };

    // Background with subtle floor pattern
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if idx < p.cells.len() {
                let is_checker = ((x / 2) + (y / 2)) % 2 == 0;
                p.cells[idx] = Cell {
                    char: ' ',
                    fg: t.fg,
                    bg: if is_checker { bg } else { floor_dim },
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
    }

    let sx = shake_x;
    let sy = shake_y;

    // Helper to draw a cell with shake
    let mut draw_cell = |x: i16, y: i16, ch: char, fg: Color, bg: Color, style: Styles| {
        let dx = x + sx;
        let dy = y + sy;
        if dx >= 0 && dx < w as i16 && dy >= 0 && dy < h as i16 {
            let idx = (dy as u16 * w + dx as u16) as usize;
            if idx < p.cells.len() {
                p.cells[idx] = Cell { char: ch, fg, bg, style, transparent: false, skip: false };
            }
        }
    };

    // Draw enemies
    for enemy in &state.enemies {
        let ex = enemy.pos.x as i16;
        let ey = enemy.pos.y as i16;
        let flash = enemy.flash_timer > 0.0;
        let color = if flash { Color::Rgb(255, 255, 255) } else { enemy.color };

        if enemy.size > 1 {
            // Tank: 2x2 block
            for dy in 0..2i16 {
                for dx in 0..2i16 {
                    let ch = if dx == 0 && dy == 0 { '▛' }
                        else if dx == 1 && dy == 0 { '▜' }
                        else if dx == 0 && dy == 1 { '▙' }
                        else { '▟' };
                    draw_cell(ex + dx, ey + dy, ch, color, bg, Styles::BOLD);
                }
            }
        } else {
            let ch = if flash { '✦' } else { enemy.glyph };
            draw_cell(ex, ey, ch, color, bg, Styles::BOLD);
        }

        // HP bar above enemy (for tanks and damaged enemies)
        if enemy.size > 1 || enemy.hp < enemy.max_hp {
            let bar_w = 3;
            let hp_pct = enemy.hp as f32 / enemy.max_hp as f32;
            let filled = (hp_pct * bar_w as f32).ceil() as usize;
            for i in 0..bar_w {
                let ch = if i < filled { '█' } else { '░' };
                let bar_color = if hp_pct > 0.5 { Color::Rgb(100, 200, 100) }
                    else if hp_pct > 0.25 { Color::Rgb(200, 200, 100) }
                    else { Color::Rgb(200, 80, 80) };
                draw_cell(ex + i as i16 - 1, ey - 1, ch, bar_color, bg, Styles::empty());
            }
        }
    }

    // Draw projectiles
    for proj in &state.projectiles {
        let px = proj.pos.x as i16;
        let py = proj.pos.y as i16;
        let trail_dir = proj.vel.normalized();
        draw_cell(px, py, '●', Color::Rgb(255, 220, 50), bg, Styles::BOLD);
        // Trail
        let tx = (proj.pos.x - trail_dir.x * 1.5) as i16;
        let ty = (proj.pos.y - trail_dir.y * 1.5) as i16;
        draw_cell(tx, ty, '·', Color::Rgb(200, 180, 40), bg, Styles::empty());
    }

    // Draw player
    let px = state.player.pos.x as i16;
    let py = state.player.pos.y as i16;
    let player_flash = state.player.invincible_timer > 0.0
        && (state.player.invincible_timer * 10.0) as i32 % 2 == 0;
    if !player_flash {
        let player_color = Color::Rgb(80, 160, 255);
        draw_cell(px, py, '●', player_color, bg, Styles::BOLD);
        // Player glow
        draw_cell(px - 1, py, '·', Color::Rgb(60, 120, 200), bg, Styles::empty());
        draw_cell(px + 1, py, '·', Color::Rgb(60, 120, 200), bg, Styles::empty());
        draw_cell(px, py - 1, '·', Color::Rgb(60, 120, 200), bg, Styles::empty());
        draw_cell(px, py + 1, '·', Color::Rgb(60, 120, 200), bg, Styles::empty());
    }

    // Draw particles
    for particle in &state.particles {
        let x = (particle.pos.x as i16) + sx;
        let y = (particle.pos.y as i16) + sy;
        if x >= 0 && x < w as i16 && y >= 0 && y < h as i16 {
            let idx = (y as u16 * w + x as u16) as usize;
            if idx < p.cells.len() {
                let alpha = particle.life / particle.max_life;
                let a = (alpha * 255.0) as u8;
                p.cells[idx].char = particle.char;
                if let Color::Rgb(r, g, b) = particle.color {
                    p.cells[idx].fg = Color::Rgb(
                        ((r as f32 * alpha) as u8).max(a),
                        ((g as f32 * alpha) as u8).max(a),
                        ((b as f32 * alpha) as u8).max(a),
                    );
                }
                p.cells[idx].transparent = false;
            }
        }
    }

    // Draw damage numbers
    for dn in &state.damage_numbers {
        let x = dn.pos.x as i16 + sx;
        let y = dn.pos.y as i16 + sy;
        for (i, ch) in dn.text.chars().enumerate() {
            let dx = x + i as i16;
            if dx >= 0 && dx < w as i16 && y >= 0 && y < h as i16 {
                let idx = (y as u16 * w + dx as u16) as usize;
                if idx < p.cells.len() {
                    let alpha = dn.life / dn.max_life;
                    let a = (alpha * 255.0) as u8;
                    if let Color::Rgb(r, g, b) = dn.color {
                        p.cells[idx].char = ch;
                        p.cells[idx].fg = Color::Rgb(
                            ((r as f32 * alpha) as u8).max(a),
                            ((g as f32 * alpha) as u8).max(a),
                            ((b as f32 * alpha) as u8).max(a),
                        );
                        p.cells[idx].style = Styles::BOLD;
                        p.cells[idx].transparent = false;
                    }
                }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HUD
    // ═══════════════════════════════════════════════════════════════════════════

    // Top bar background
    let bar_bg = Color::Rgb(20, 20, 30);
    for x in 0..w {
        let idx = x as usize;
        if idx < p.cells.len() {
            p.cells[idx].bg = bar_bg;
            p.cells[idx].transparent = false;
        }
    }

    // HP bar (left)
    let hp_pct = state.player.hp as f32 / state.player.max_hp as f32;
    let hp_filled = (hp_pct * 20.0).ceil() as usize;
    let hp_color = if hp_pct > 0.6 { Color::Rgb(80, 200, 80) }
        else if hp_pct > 0.3 { Color::Rgb(200, 200, 80) }
        else { Color::Rgb(200, 60, 60) };
    let hp_text = format!("HP {}/{} ", state.player.hp, state.player.max_hp);
    p.put_str(1, 0, &hp_text);
    for i in 0..hp_text.len() {
        let idx = i + 1;
        if idx < p.cells.len() {
            p.cells[idx].fg = Color::Rgb(220, 220, 230);
            p.cells[idx].bg = bar_bg;
            p.cells[idx].style = Styles::BOLD;
        }
    }
    let bar_x = hp_text.len() + 2;
    for i in 0..20 {
        let idx = bar_x + i;
        if idx < p.cells.len() {
            p.cells[idx].char = if i < hp_filled { '█' } else { '░' };
            p.cells[idx].fg = hp_color;
            p.cells[idx].bg = bar_bg;
        }
    }

    // Score (center)
    let score_text = format!("Score: {:5}", state.score);
    let score_x = (w as usize - score_text.len()) / 2;
    p.put_str(score_x as u16, 0, &score_text);
    for i in 0..score_text.len() {
        let idx = score_x + i;
        if idx < p.cells.len() {
            p.cells[idx].fg = Color::Rgb(255, 220, 100);
            p.cells[idx].bg = bar_bg;
            p.cells[idx].style = Styles::BOLD;
        }
    }

    // Level / Wave / Kills (right)
    let right_info = format!(
        "Lv:{} Wave:{} Kills:{} ",
        state.player.level, state.wave, state.player.kills
    );
    let right_x = w as usize - right_info.len();
    p.put_str(right_x as u16, 0, &right_info);
    for i in 0..right_info.len() {
        let idx = right_x + i;
        if idx < p.cells.len() {
            p.cells[idx].fg = Color::Rgb(180, 180, 200);
            p.cells[idx].bg = bar_bg;
        }
    }

    // XP bar (second row)
    if h > 1 {
        let xp_pct = state.player.xp as f32 / state.player.xp_to_next as f32;
        let xp_filled = (xp_pct * w as f32) as usize;
        let xp_color = Color::Rgb(100, 180, 255);
        for x in 0..w {
            let idx = (w + x) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = if (x as usize) < xp_filled { '▬' } else { ' ' };
                p.cells[idx].fg = xp_color;
                p.cells[idx].bg = Color::Rgb(15, 15, 22);
                p.cells[idx].transparent = false;
            }
        }
    }

    // Bottom bar
    if h > 2 {
        let bottom_y = h - 1;
        let help_key = kb.display(actions::HELP).unwrap_or("?");
        let quit_key = kb.display(actions::QUIT).unwrap_or("q");
        let bottom_text = format!(
            " {}:help | {}:quit | WASD:move | Click:shoot | Space:pause ",
            help_key, quit_key
        );
        p.put_str(0, bottom_y, &bottom_text);
        for x in 0..bottom_text.len().min(w as usize) {
            let idx = (bottom_y * w + x as u16) as usize;
            if idx < p.cells.len() {
                p.cells[idx].fg = Color::Rgb(140, 140, 150);
                p.cells[idx].bg = Color::Rgb(15, 15, 22);
                p.cells[idx].transparent = false;
            }
        }
    }

    // Crosshair at mouse position
    let mx = state.mouse_pos.x as i16;
    let my = state.mouse_pos.y as i16;
    if mx > 0 && mx < w as i16 - 1 && my > 1 && my < h as i16 - 1 {
        let idx = (my as u16 * w + mx as u16) as usize;
        if idx < p.cells.len() {
            p.cells[idx].fg = Color::Rgb(255, 255, 255);
            p.cells[idx].style = Styles::BOLD;
        }
    }
}

fn render_pause(p: &mut Plane, w: u16, h: u16, kb: &KeybindingSet) {
    let overlay_bg = Color::Rgb(10, 10, 15);
    // Semi-transparent overlay
    for y in 3..h.saturating_sub(3) {
        for x in 3..w.saturating_sub(3) {
            let idx = (y * w + x) as usize;
            if idx < p.cells.len() {
                p.cells[idx].bg = overlay_bg;
                p.cells[idx].transparent = false;
            }
        }
    }

    let title = "⏸ PAUSED";
    let tx = (w as usize - title.len()) / 2;
    let ty = h as usize / 2 - 2;
    p.put_str(tx as u16, ty as u16, title);
    for i in 0..title.len() {
        let idx = ty * w as usize + tx + i;
        if idx < p.cells.len() {
            p.cells[idx].fg = Color::Rgb(255, 220, 100);
            p.cells[idx].style = Styles::BOLD;
            p.cells[idx].bg = overlay_bg;
        }
    }

    let help_key = kb.display(actions::HELP).unwrap_or("?");
    let back_key = kb.display(actions::BACK).unwrap_or("esc");
    let lines = [
        "Space — Resume".to_string(),
        format!("{} — Help", help_key),
        format!("{} — Dismiss", back_key),
    ];
    for (i, line) in lines.iter().enumerate() {
        let lx = (w as usize - line.len()) / 2;
        let ly = ty + 2 + i;
        p.put_str(lx as u16, ly as u16, line);
        for j in 0..line.len() {
            let idx = ly * w as usize + lx + j;
            if idx < p.cells.len() {
                p.cells[idx].fg = Color::Rgb(180, 180, 190);
                p.cells[idx].bg = overlay_bg;
            }
        }
    }
}

fn render_game_over(p: &mut Plane, state: &GameState, w: u16, h: u16) {
    let overlay_bg = Color::Rgb(10, 10, 15);
    for y in 2..h.saturating_sub(2) {
        for x in 2..w.saturating_sub(2) {
            let idx = (y * w + x) as usize;
            if idx < p.cells.len() {
                p.cells[idx].bg = overlay_bg;
                p.cells[idx].transparent = false;
            }
        }
    }

    let title = "💀 GAME OVER";
    let tx = (w as usize - title.len()) / 2;
    let ty = h as usize / 2 - 4;
    p.put_str(tx as u16, ty as u16, title);
    for i in 0..title.len() {
        let idx = ty * w as usize + tx + i;
        if idx < p.cells.len() {
            p.cells[idx].fg = Color::Rgb(255, 80, 80);
            p.cells[idx].style = Styles::BOLD;
            p.cells[idx].bg = overlay_bg;
        }
    }

    let stats = [
        format!("Final Score: {}", state.score),
        format!("Waves Survived: {}", state.wave),
        format!("Enemies Defeated: {}", state.player.kills),
        format!("Time: {:.1}s", state.game_time),
        format!("Level Reached: {}", state.player.level),
    ];
    for (i, line) in stats.iter().enumerate() {
        let lx = (w as usize - line.len()) / 2;
        let ly = ty + 1 + i;
        p.put_str(lx as u16, ly as u16, line);
        for j in 0..line.len() {
            let idx = ly * w as usize + lx + j;
            if idx < p.cells.len() {
                p.cells[idx].fg = Color::Rgb(200, 200, 210);
                p.cells[idx].bg = overlay_bg;
            }
        }
    }

    let restart = "Press R to restart";
    let rx = (w as usize - restart.len()) / 2;
    let ry = ty + stats.len() + 2;
    p.put_str(rx as u16, ry as u16, restart);
    for i in 0..restart.len() {
        let idx = (ry * w as usize + rx + i) as usize;
        if idx < p.cells.len() {
            p.cells[idx].fg = Color::Rgb(100, 200, 100);
            p.cells[idx].style = Styles::BOLD;
            p.cells[idx].bg = overlay_bg;
        }
    }
}

fn render_help(p: &mut Plane, w: u16, h: u16, kb: &KeybindingSet) {
    let box_bg = Color::Rgb(18, 18, 28);
    let fg = Color::Rgb(200, 200, 210);
    let _accent = Color::Rgb(255, 200, 100);
    let dim = Color::Rgb(120, 120, 130);

    let quit_key = kb.display(actions::QUIT).unwrap_or("ctrl+q");
    let help_key = kb.display(actions::HELP).unwrap_or("f1");
    let back_key = kb.display(actions::BACK).unwrap_or("esc");

    let lines = [
        ("╭──────────────────────────────────────────────────────────╮", dim),
        ("│              🎮 Terminal Arena — Help                    │", fg),
        ("├──────────────────────────────────────────────────────────┤", dim),
        (&format!("│  {:<10}  —  Move                                       │", "WASD / ↑↓←→"), fg),
        ("│  Click       —  Shoot toward cursor                      │", fg),
        ("│  Space       —  Pause / Resume                           │", fg),
        (&format!("│  {:<10}  —  Toggle this help                           │", help_key), fg),
        (&format!("│  {:<10}  —  Dismiss help / menu                        │", back_key), fg),
        ("│  R           —  Restart (after game over)                │", fg),
        (&format!("│  {:<10}  —  Quit                                       │", quit_key), fg),
        ("├──────────────────────────────────────────────────────────┤", dim),
        ("│  Tips:                                                   │", fg),
        ("│    • Keep moving — enemies chase you!                    │", fg),
        ("│    • Click rapidly to shoot in different directions      │", fg),
        ("│    • Tanks are slow but tough — focus fire!              │", fg),
        ("│    • Swarmers are fast but fragile — one shot kills      │", fg),
        ("│    • Leveling up restores full HP                        │", fg),
        ("╰──────────────────────────────────────────────────────────╯", dim),
    ];

    let start_y = (h as usize - lines.len()) / 2;
    for (i, (line, line_fg)) in lines.iter().enumerate() {
        let y = start_y + i;
        let x = (w as usize - line.len()) / 2;
        if y >= h as usize { continue; }
        for (ci, ch) in line.chars().enumerate() {
            let px = x + ci;
            if px >= w as usize { continue; }
            let idx = y * w as usize + px;
            if idx < p.cells.len() {
                let ch_fg = if "│╭╮├┤╰╯─".contains(ch) { dim } else { *line_fg };
                p.cells[idx] = Cell {
                    char: ch,
                    fg: ch_fg,
                    bg: box_bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = Theme::from_env_or(Theme::dark());

    let mut term = Terminal::new(io::stdout())?;
    write!(term, "\x1b[?1000h\x1b[?1003h\x1b[?1006h\x1b[?25l")?;
    term.flush()?;

    let (mut w, mut h) = dracon_terminal_engine::backend::tty::get_window_size(term.as_fd())?;
    let mut compositor = Compositor::new(w, h);
    compositor.set_clear_color(Color::Rgb(10, 10, 15));
    let mut parser = Parser::new();
    let mut stdin = io::stdin();

    let mut state = GameState::new(w, h);
    let mut last_tick = Instant::now();
    let mut show_help = false;

    let should_quit = Arc::new(AtomicBool::new(false));
    let sig_flag = Arc::clone(&should_quit);
    unsafe {
        signal_hook::low_level::register(SIGINT, move || {
            sig_flag.store(true, Ordering::SeqCst);
        })
    }
    .ok();

    let keybindings = KeybindingSet::from_config(&resolve_keybindings());

    let write_theme_file = || {
        if let Ok(path) = std::env::var("DTRON_THEME_FILE") {
            let _ = std::fs::write(&path, theme.name.as_bytes());
        }
    };

    let mut frames = 0u32;
    let mut fps = 0u32;
    let mut fps_timer = Instant::now();

    loop {
        if should_quit.load(Ordering::SeqCst) {
            write!(term, "\x1b[?1000l\x1b[?1003l\x1b[?1006l\x1b[?25h")?;
            term.flush()?;
            write_theme_file();
            return Ok(());
        }

        // Poll input
        if poll_input(term.as_fd(), 0)? {
            let mut buf = [0u8; 128];
            if let Ok(n) = stdin.read(&mut buf) {
                for &byte in &buf[..n] {
                    match parser.advance(byte) {
                        Some(Event::Key(ref key_event))
                            if keybindings.matches(actions::QUIT, key_event) =>
                        {
                            write!(term, "\x1b[?1000l\x1b[?1003l\x1b[?1006l\x1b[?25h")?;
                            term.flush()?;
                            write_theme_file();
                            return Ok(());
                        }
                        Some(Event::Key(ref key_event))
                            if keybindings.matches(actions::HELP, key_event) =>
                        {
                            show_help = !show_help;
                        }
                        Some(Event::Key(ref key_event))
                            if keybindings.matches(actions::BACK, key_event) =>
                        {
                            show_help = false;
                        }
                        Some(Event::Key(ref key_event)) => {
                            state.handle_input(key_event, w, h);
                        }
                        Some(Event::Mouse(mouse)) => {
                            state.handle_mouse(mouse.kind, mouse.column, mouse.row);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Resize check
        if let Ok((new_w, new_h)) =
            dracon_terminal_engine::backend::tty::get_window_size(term.as_fd())
        {
            if new_w != w || new_h != h {
                w = new_w;
                h = new_h;
                compositor.resize(w, h);
            }
        }

        // Update & Render
        let now = Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f32();
        if dt >= 1.0 / TARGET_FPS {
            last_tick = now;
            state.update(dt, w, h);

            compositor.planes.clear();
            let mut p = Plane::new(1, w, h);

            if show_help {
                render_game(&mut p, &state, w, h, fps, &keybindings);
                render_help(&mut p, w, h, &keybindings);
            } else if state.phase == GamePhase::GameOver {
                render_game(&mut p, &state, w, h, fps, &keybindings);
                render_game_over(&mut p, &state, w, h);
            } else if state.phase == GamePhase::Paused {
                render_game(&mut p, &state, w, h, fps, &keybindings);
                render_pause(&mut p, w, h, &keybindings);
            } else {
                render_game(&mut p, &state, w, h, fps, &keybindings);
            }

            compositor.add_plane(p);
            compositor.render(term.inner())?;
            frames += 1;
        } else {
            std::thread::sleep(Duration::from_millis(1));
        }

        if fps_timer.elapsed().as_secs() >= 1 {
            fps = frames;
            frames = 0;
            fps_timer = Instant::now();
        }
    }
}

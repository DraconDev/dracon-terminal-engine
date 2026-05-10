#![allow(missing_docs)]
#![allow(clippy::manual_is_multiple_of)]
//! Game Loop — 60fps animation demo with particles and mouse interaction.
//!
//! A visual showcase of the compositor's animation capabilities.
//!
//! Controls:
//!   q        — quit
//!   ?        — toggle help
//!   Click    — spawn particle burst
//!   Space    — toggle turbo mode

use dracon_terminal_engine::backend::tty::poll_input;
use dracon_terminal_engine::compositor::engine::Compositor;
use dracon_terminal_engine::compositor::plane::{Color, Plane, Styles};
use dracon_terminal_engine::core::terminal::Terminal;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::input::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use dracon_terminal_engine::input::parser::Parser;
use signal_hook::consts::signal::SIGINT;
use std::io::{self, Read, Write};
use std::os::fd::AsFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    color: Color,
    char: char,
}

impl Particle {
    fn update(&mut self, dt: f32) -> bool {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.vy += 30.0 * dt; // gravity
        self.life -= dt;
        self.life > 0.0
    }
}

struct Star {
    x: f32,
    y: f32,
    brightness: f32,
    twinkle_speed: f32,
    phase: f32,
}

struct GameState {
    rocket_x: f32,
    rocket_y: f32,
    particles: Vec<Particle>,
    stars: Vec<Star>,
    turbo: bool,
    click_count: u32,
    frame_count: u64,
}

impl GameState {
    fn new(w: u16, h: u16) -> Self {
        let mut stars = Vec::new();
        for _ in 0..50 {
            stars.push(Star {
                x: rand::random::<f32>() * w as f32,
                y: rand::random::<f32>() * h as f32,
                brightness: 0.3 + rand::random::<f32>() * 0.7,
                twinkle_speed: 2.0 + rand::random::<f32>() * 4.0,
                phase: rand::random::<f32>() * std::f32::consts::TAU,
            });
        }
        Self {
            rocket_x: 0.0,
            rocket_y: h as f32 / 2.0,
            particles: Vec::new(),
            stars,
            turbo: false,
            click_count: 0,
            frame_count: 0,
        }
    }

    fn spawn_burst(&mut self, x: f32, y: f32, count: usize) {
        let colors = [
            Color::Rgb(255, 100, 100),
            Color::Rgb(100, 255, 100),
            Color::Rgb(100, 100, 255),
            Color::Rgb(255, 255, 100),
            Color::Rgb(255, 100, 255),
            Color::Rgb(100, 255, 255),
        ];
        let chars = ['●', '◆', '★', '•', '·'];
        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU + rand::random::<f32>() * 0.5;
            let speed = 20.0 + rand::random::<f32>() * 40.0;
            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: 1.0 + rand::random::<f32>() * 1.5,
                max_life: 2.5,
                color: colors[i % colors.len()],
                char: chars[i % chars.len()],
            });
        }
    }

    fn update(&mut self, dt: f32, w: u16, h: u16) {
        let speed = if self.turbo { 80.0 } else { 40.0 };
        self.rocket_x += speed * dt;
        self.rocket_y += (self.rocket_x * 0.1).sin() * 0.5;

        if self.rocket_x >= w as f32 {
            self.rocket_x = 0.0;
            self.rocket_y = h as f32 / 2.0 + (rand::random::<f32>() - 0.5) * (h as f32 * 0.5);
        }

        // Spawn trail particles
        if self.frame_count % 3 == 0 {
            self.particles.push(Particle {
                x: self.rocket_x - 2.0,
                y: self.rocket_y + 0.5,
                vx: -5.0 + rand::random::<f32>() * -10.0,
                vy: (rand::random::<f32>() - 0.5) * 5.0,
                life: 0.5 + rand::random::<f32>() * 0.5,
                max_life: 1.0,
                color: Color::Rgb(200, 150, 50),
                char: '·',
            });
        }

        self.particles.retain_mut(|p| p.update(dt));
        self.frame_count += 1;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut term = Terminal::new(io::stdout())?;

    write!(term, "\x1b[?1000h\x1b[?1003h\x1b[?1006h\x1b[?25l")?;
    term.flush()?;

    let (mut w, mut h) = dracon_terminal_engine::backend::tty::get_window_size(term.as_fd())?;
    let mut compositor = Compositor::new(w, h);
    compositor.set_clear_color(Color::Rgb(8, 8, 16));
    let mut parser = Parser::new();
    let mut stdin = io::stdin();

    let mut state = GameState::new(w, h);
    let mut last_tick = Instant::now();
    let mut show_help = false;

    // Signal handler for clean quit on Ctrl+C
    let should_quit = Arc::new(AtomicBool::new(false));
    let sig_flag = Arc::clone(&should_quit);
    unsafe { signal_hook::low_level::register(SIGINT, move || { sig_flag.store(true, Ordering::SeqCst); }) }
        .ok();

    let keybindings = KeybindingSet::from_config(&resolve_keybindings());

    let mut frames = 0;
    let mut fps = 0;
    let mut fps_timer = Instant::now();
    let target_fps = 60.0;

    loop {
        if should_quit.load(Ordering::SeqCst) {
            write!(term, "\x1b[?1000l\x1b[?1003l\x1b[?1006l\x1b[?25h")?;
            term.flush()?;
            return Ok(());
        }
        // Poll Input
        if poll_input(term.as_fd(), 0)? {
            let mut buf = [0u8; 128];
            if let Ok(n) = stdin.read(&mut buf) {
                for &byte in &buf[..n] {
                    match parser.advance(byte) {
                        Some(Event::Key(KeyEvent { code: KeyCode::Char('q'), .. })) => {
                            write!(
                                term,
                                "\x1b[?1000l\x1b[?1003l\x1b[?1006l\x1b[?25h"
                            )?;
                            term.flush()?;
                            return Ok(());
                        }
                        Some(Event::Key(KeyEvent { code: KeyCode::Char('c'), ref modifiers, .. }))
                            if modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            write!(term, "\x1b[?1000l\x1b[?1003l\x1b[?1006l\x1b[?25h")?;
                            term.flush()?;
                            return Ok(());
                        }
                        Some(Event::Key(KeyEvent { code: KeyCode::Char('?'), .. })) => {
                            show_help = !show_help;
                        }
                        Some(Event::Key(KeyEvent { code: KeyCode::Esc, .. })) => {
                            show_help = false;
                        }
                        Some(Event::Key(KeyEvent { code: KeyCode::Char(' '), .. })) => {
                            state.turbo = !state.turbo;
                        }
                        Some(Event::Mouse(mouse)) => {
                            if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                                state.spawn_burst(mouse.column as f32, mouse.row as f32, 20);
                                state.click_count += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Resize Check
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
        if dt >= 1.0 / target_fps {
            last_tick = now;
            state.update(dt, w, h);

            compositor.planes.clear();
            let mut p = Plane::new(1, w, h);

            if show_help {
                render_help(&mut p, w, h);
            } else {
                render_game(&mut p, &state, w, h, fps);
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

fn render_game(p: &mut Plane, state: &GameState, w: u16, h: u16, fps: u32) {
    // Stars
    let t = state.frame_count as f32 * 0.05;
    for star in &state.stars {
        let brightness = star.brightness
            * ((t * star.twinkle_speed + star.phase).sin() * 0.5 + 0.5);
        let c = (brightness * 255.0) as u8;
        let sx = star.x as u16;
        let sy = star.y as u16;
        if sx < w && sy < h {
            let idx = (sy * w + sx) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = if brightness > 0.7 { '★' } else { '·' };
                p.cells[idx].fg = Color::Rgb(c, c, c.max(200));
                p.cells[idx].transparent = false;
            }
        }
    }

    // Particles
    for particle in &state.particles {
        let px = particle.x as u16;
        let py = particle.y as u16;
        if px < w && py < h {
            let idx = (py * w + px) as usize;
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

    // Rocket
    let rx = state.rocket_x as u16;
    let ry = state.rocket_y as u16;
    if rx < w && ry < h {
        let idx = (ry * w + rx) as usize;
        if idx < p.cells.len() {
            p.cells[idx].char = '🚀';
            p.cells[idx].fg = Color::Rgb(255, 200, 50);
            p.cells[idx].style = Styles::BOLD;
            p.cells[idx].transparent = false;
        }
    }
    // Rocket trail
    if rx > 2 && ry < h {
        for i in 1..=3 {
            let tx = rx.saturating_sub(i);
            let idx = (ry * w + tx) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = ['░', '▒', '▓'][i as usize - 1];
                p.cells[idx].fg = Color::Rgb((200 - i * 40) as u8, (150 - i * 30) as u8, 30);
                p.cells[idx].transparent = false;
            }
        }
    }

    // HUD
    let hud = format!(
        " FPS: {} | Particles: {} | Clicks: {} | Turbo: {} | ?:help | Esc:dismiss | q:quit ",
        fps,
        state.particles.len(),
        state.click_count,
        if state.turbo { "ON" } else { "off" }
    );
    p.put_str(0, 0, &hud);
    // HUD background
    for i in 0..hud.len().min(w as usize) {
        let idx = i;
        if idx < p.cells.len() {
            p.cells[idx].bg = Color::Rgb(30, 30, 40);
            p.cells[idx].transparent = false;
        }
    }
}

fn render_help(p: &mut Plane, w: u16, h: u16) {
    let help_lines = [
        "╭────────────────────────────────────────────────────╮",
        "│              Game Loop Help                        │",
        "├────────────────────────────────────────────────────┤",
        "│  q       — Quit                                    │",
        "│  ?       — Toggle this help                        │",
        "│  Esc     — Dismiss help                            │",
        "│  Space   — Toggle turbo mode                       │",
        "│  Click   — Spawn particle burst                    │",
        "├────────────────────────────────────────────────────┤",
        "│  Features:                                         │",
        "│    • 60fps animation loop                          │",
        "│    • Particle physics with gravity                 │",
        "│    • Starfield with twinkling                      │",
        "│    • Mouse interaction                             │",
        "│    • Rocket with trail effect                      │",
        "╰────────────────────────────────────────────────────╯",
    ];

    let start_y = (h as usize - help_lines.len()) / 2;
    for (i, line) in help_lines.iter().enumerate() {
        let y = start_y + i;
        let x = (w as usize - line.len()) / 2;
        if y < h as usize {
            p.put_str(x as u16, y as u16, line);
        }
    }
}

#![allow(missing_docs)]
use rand::Rng;
use std::io::{self, stdout, Read, Write};

use dracon_terminal_engine::{
    compositor::engine::Compositor,
    compositor::plane::{Cell, Plane},
    input::event::{Event, KeyCode, KeyEvent},
    input::parser::Parser,
    Terminal,
};

struct Window {
    _id: usize,
    title: String,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    color: u8,
    minimized: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut term = Terminal::new(stdout())?;

    // Enter Alt Screen for "Desktop" feel
    write!(term, "\x1b[?1049h")?;
    // Enable SGR Mouse (1006) + Any Event (1003) for fluid motion
    write!(term, "\x1b[?1000h\x1b[?1003h\x1b[?1006h")?;
    // Hide Cursor
    write!(term, "\x1b[?25l")?;
    // CRITICAL: Flush to ensure terminal receives commands
    term.flush()?;

    let size = (80, 24); // Assume standard or get from ioctl/crossterm if added
    let mut compositor = Compositor::new(size.0, size.1);
    let mut parser = Parser::new();
    let mut stdin = io::stdin();

    // Matrix Rain State
    let mut drops: Vec<f32> = vec![0.0; size.0 as usize];
    let mut rng = rand::thread_rng();

    // Windows
    let mut windows = vec![
        Window {
            _id: 1,
            title: " TERMINAL ".to_string(),
            x: 5,
            y: 3,
            width: 20,
            height: 10,
            color: 0,
            minimized: false,
        },
        Window {
            _id: 2,
            title: " SYSTEM ".to_string(),
            x: 30,
            y: 6,
            width: 25,
            height: 8,
            color: 20,
            minimized: false,
        },
        Window {
            _id: 3,
            title: " ALERT ".to_string(),
            x: 10,
            y: 14,
            width: 20,
            height: 5,
            color: 1,
            minimized: false,
        },
    ];

    // Interaction State
    let mut dragging_window: Option<usize> = None;
    let mut drag_offset: (u16, u16) = (0, 0);

    // Loop
    loop {
        // Input Handling
        let mut buf = [0u8; 128];
        // Note: This is blocking. Animation runs only on input/mouse move.
        // For a real game loop, use a separate thread or non-blocking I/O.
        if let Ok(n) = stdin.read(&mut buf) {
            for &byte in &buf[..n] {
                if let Some(event) = parser.advance(byte) {
                    match event {
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('q'),
                            ..
                        }) => {
                            // Cleanup
                            write!(term, "\x1b[?1049l\x1b[?25h")?;
                            return Ok(());
                        }
                        Event::Mouse(dracon_terminal_engine::input::event::MouseEvent {
                            row: y,
                            column: x,
                            kind,
                            ..
                        }) => {
                            let is_press = matches!(
                                kind,
                                dracon_terminal_engine::input::event::MouseEventKind::Down(_)
                            );
                            let is_drag = matches!(
                                kind,
                                dracon_terminal_engine::input::event::MouseEventKind::Drag(_)
                            );
                            let cx = x.saturating_sub(1);
                            let cy = y.saturating_sub(1);

                            if is_press && !is_drag {
                                // Check for minimize button click on title bar (right side)
                                let mut minimized_click = None;
                                for (idx, win) in windows.iter().enumerate() {
                                    if cy == win.y && cx >= win.x + win.width.saturating_sub(4) && cx < win.x + win.width - 1 {
                                        minimized_click = Some(idx);
                                        break;
                                    }
                                }
                                if let Some(idx) = minimized_click {
                                    windows[idx].minimized = !windows[idx].minimized;
                                    dragging_window = None;
                                    continue;
                                }

                                // Check taskbar click to restore minimized windows
                                if cy == size.1 - 1 {
                                    for (idx, win) in windows.iter().enumerate() {
                                        if win.minimized {
                                            let label_x = 2 + idx as u16 * 12;
                                            if cx >= label_x && cx < label_x + 10 {
                                                windows[idx].minimized = false;
                                                break;
                                            }
                                        }
                                    }
                                    dragging_window = None;
                                    continue;
                                }

                                // Hit Test for normal windows
                                let mut focused = None;
                                for (idx, win) in windows.iter_mut().enumerate().rev() {
                                    if win.minimized { continue; }
                                    if cx >= win.x
                                        && cx < win.x + win.width
                                        && cy >= win.y
                                        && cy < win.y + win.height
                                    {
                                        focused = Some(idx);
                                        drag_offset = (cx - win.x, cy - win.y);
                                        break;
                                    }
                                }

                                if let Some(idx) = focused {
                                    let win = windows.remove(idx);
                                    windows.push(win);
                                    dragging_window = Some(windows.len() - 1);
                                } else {
                                    dragging_window = None;
                                }
                            } else if !is_press && !is_drag {
                                dragging_window = None;
                            } else if is_drag {
                                if let Some(idx) = dragging_window {
                                    if let Some(win) = windows.get_mut(idx) {
                                        // Bounds check
                                        win.x = cx
                                            .saturating_sub(drag_offset.0)
                                            .min(size.0 - win.width);
                                        win.y = cy
                                            .saturating_sub(drag_offset.1)
                                            .min(size.1 - win.height);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Update Animation (Rain)
        drops.iter_mut().for_each(|drop| {
            if rng.gen_bool(0.1) {
                *drop = 0.0;
            }
            *drop += 0.5;
            if *drop > size.1 as f32 {
                *drop = size.1 as f32 + 10.0;
            }
        });

        // Render Frame
        compositor.planes.clear();

        // 1. Background: Matrix Rain
        let mut bg = Plane::new(0, size.0, size.1);
        for x in 0..size.0 {
            for y in 0..size.1 {
                let drop_y = drops[x as usize] as i32;
                let dist = drop_y - y as i32;
                if (0..5).contains(&dist) {
                    let char = if dist == 0 { '0' } else { '1' };
                    let fg = if dist == 0 { 46 } else { 22 };
                    let cell = Cell {
                        char,
                        fg: dracon_terminal_engine::compositor::plane::Color::Ansi(fg),
                        bg: dracon_terminal_engine::compositor::plane::Color::Reset,
                        transparent: false,
                        skip: false,
                        style: Default::default(),
                    };
                    bg.put_cell(x, y, cell);
                }
            }
        }
        compositor.add_plane(bg);

        // 2. Windows
        for (i, win) in windows.iter().enumerate() {
            if win.minimized { continue; }
            let z_label = format!("[z:{}]", i + 1);
            let mut p = Plane::new(i + 1, win.width, win.height);
            p.set_absolute_position(win.x, win.y);
            p.set_z_index((i + 10) as i32);

            for wy in 0..win.height {
                for wx in 0..win.width {
                    let border = wx == 0 || wx == win.width - 1 || wy == 0 || wy == win.height - 1;
                    let header = wy == 0;

                    let mut c = Cell::default();
                    if border {
                        c.char = if header { '=' } else { '|' };
                        c.fg = dracon_terminal_engine::compositor::plane::Color::Ansi(15);
                        c.bg = dracon_terminal_engine::compositor::plane::Color::Ansi(win.color);
                    } else {
                        c.char = ' ';
                        c.bg = dracon_terminal_engine::compositor::plane::Color::Reset;
                    }

                    if header && wx > 1 && wx < win.title.len() as u16 + 2 {
                        c.char = win.title.chars().nth((wx - 2) as usize).unwrap_or(' ');
                        c.fg = dracon_terminal_engine::compositor::plane::Color::Ansi(15);
                        c.bg = dracon_terminal_engine::compositor::plane::Color::Ansi(win.color);
                    }
                    // Z-order label on right side of title bar
                    if header && wx > win.width.saturating_sub(z_label.len() as u16 + 1) {
                        let label_idx = wx - (win.width - z_label.len() as u16);
                        if let Some(ch) = z_label.chars().nth(label_idx as usize) {
                            c.char = ch;
                            c.fg = dracon_terminal_engine::compositor::plane::Color::Ansi(15);
                            c.bg = dracon_terminal_engine::compositor::plane::Color::Ansi(win.color);
                        }
                    }
                    p.put_cell(wx, wy, c);
                }
            }
            compositor.add_plane(p);
        }

        // 3. Taskbar (Bottom) with clock
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let secs = now.as_secs() % 86400;
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let clock = format!("{:02}:{:02}", hours, mins);
        let status = format!(" [Start]  Dracon Desktop  |  {}  | q: quit  |  Minimized:", clock);
        let mut minimized_labels = String::new();
        for (idx, win) in windows.iter().enumerate() {
            if win.minimized {
                minimized_labels.push_str(&format!(" [{}]", &win.title.trim()));
            }
        }
        let full_status = if minimized_labels.is_empty() {
            status
        } else {
            format!("{} {}", &status[..status.len().min(size.0 as usize - minimized_labels.len() as usize - 2)], minimized_labels)
        };
        // Taskbar rendering
        let mut taskbar = Plane::new(999, size.0, 1);
        taskbar.set_absolute_position(0, size.1 - 1);
        taskbar.set_z_index(2000);
        for (i, c) in full_status.chars().enumerate() {
            let cell = Cell {
                char: c,
                fg: dracon_terminal_engine::compositor::plane::Color::Ansi(0),
                bg: dracon_terminal_engine::compositor::plane::Color::Ansi(15),
                transparent: false,
                skip: false,
                style: Default::default(),
            };
            taskbar.put_cell(i as u16, 0, cell);
        }
        compositor.add_plane(taskbar);

        compositor.render(&mut term)?;
    }
}

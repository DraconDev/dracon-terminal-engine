#![allow(missing_docs)]
//! Dracon Terminal Engine — Example Showcase Launcher
//!
//! Interactive grid-based launcher for all framework examples.
//! Features: category filtering, real-time search, animated selection,
//! card-based layout with mini previews, live data previews, keyboard shortcuts,
//! and an interactive primitives bar demonstrating engine building blocks.
//!
//! Controls:
//!   arrows  — navigate cards
//!   Enter   — launch selected example
//!   /       — focus search bar
//!   Tab     — cycle categories
//!   t       — cycle theme
//!   d       — toggle debug overlay
//!   ?       — toggle help
//!   Space   — preview card (modal)
//!   1-5     — interact with primitives bar
//!   right-click — context menu (Launch / Copy name / Filter by category)
//!   q       — quit
//!
//! When an example is launched, press `q` or `Esc` in the example to return to the showcase.
//! A "Returned from [example]" toast will appear when you come back.

use std::io::Read;
use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use dracon_terminal_engine::backend::tty::poll_input;
use dracon_terminal_engine::framework::prelude::*;
use ratatui::layout::Rect;

mod data;
mod render;
mod scenes;
mod state;
mod widget;
use state::Showcase;

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("Dracon Terminal Engine — Example Showcase");
    println!("Grid launcher with search, categories, and live previews");
    std::thread::sleep(Duration::from_millis(500));

    let pending = Arc::new(Mutex::new(None));
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);
    let fps_counter = Arc::new(AtomicU64::new(0));
    let fps_for_tick = Arc::clone(&fps_counter);
    let returned_from = Arc::new(Mutex::new(None));
    let returned_for_tick = Arc::clone(&returned_from);
    let last_launched: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let last_launched_for_tick = Arc::clone(&last_launched);
    let pending_app_theme: Arc<Mutex<Option<Theme>>> = Arc::new(Mutex::new(None));
    let pending_app_theme_tick = Arc::clone(&pending_app_theme);

    let showcase = Showcase::new(should_quit, pending.clone(), fps_counter, returned_from, pending_app_theme);

    let mut app = App::new()?
        .title("Dracon Showcase")
        .fps(30)
        .theme(Theme::from_env_or(Theme::nord()));
    let _showcase_id = app.add_widget(Box::new(showcase), Rect::new(0, 0, 80, 24));

    app.on_tick(move |ctx, _tick| {
        if quit_check.load(std::sync::atomic::Ordering::SeqCst) {
            ctx.stop();
            return;
        }

        // Sync pending theme from showcase widget to App framework
        if let Some(theme) = pending_app_theme_tick.lock().unwrap().take() {
            ctx.set_theme(theme);
        }

        // Compute and store FPS
        fps_for_tick.store(ctx.fps(), std::sync::atomic::Ordering::Relaxed);

        // Handle pending binary launch
        if let Some(binary_name) = pending.lock().unwrap().take() {
            let exe_dir = match std::env::current_exe() {
                Ok(p) => p.parent().map(|parent| parent.to_path_buf()).unwrap_or_default(),
                Err(_) => return,
            };
            let binary_path = exe_dir.join(&binary_name);

            // Remember what we launched for the return message
            *last_launched_for_tick.lock().unwrap() = Some(binary_name.clone());

            // Set env vars so the launched binary inherits our theme
            // and can report its final theme back via DTRON_THEME_FILE
            std::env::set_var("DTRON_THEME", ctx.theme().name);
            let theme_return_path = std::env::temp_dir().join("dron_theme_return");
            std::env::set_var("DTRON_THEME_FILE", theme_return_path.to_str().unwrap());

            let _ = ctx.suspend_terminal();

            // Auto-build if missing
            if !binary_path.exists() {
                let find_crate_root = || -> Option<std::path::PathBuf> {
                    let mut dir = exe_dir.clone();
                    loop {
                        if dir.join("Cargo.toml").exists() {
                            return Some(dir);
                        }
                        if !dir.pop() {
                            return None;
                        }
                    }
                };

                if let Some(crate_root) = find_crate_root() {
                    let _ = std::process::Command::new("cargo")
                        .args(["build", "--example", &binary_name])
                        .current_dir(&crate_root)
                        .status();
                }
            }

            let _ = std::process::Command::new(&binary_path)
                .current_dir(&exe_dir)
                .status();

            // Check if the child reported its final theme back
            if theme_return_path.exists() {
                if let Ok(theme_name) = std::fs::read_to_string(&theme_return_path) {
                    let theme_name = theme_name.trim().to_string();
                    if !theme_name.is_empty() {
                        if let Some(theme) = Theme::from_name(&theme_name) {
                            ctx.set_theme(theme);
                        }
                    }
                }
                let _ = std::fs::remove_file(&theme_return_path);
            }
            std::env::remove_var("DTRON_THEME_FILE");

            let _ = ctx.resume_terminal();

            // Non-blocking drain of any stray input bytes left by the child.
            // In raw mode, blocking read() would hang forever if the buffer is empty.
            let mut drain_buf = [0u8; 512];
            let mut stdin = std::io::stdin();
            let raw_fd = stdin.as_raw_fd();
            for _ in 0..10 {
                // SAFETY: poll_input only reads the fd, no mutation
                let fd = unsafe { std::os::fd::BorrowedFd::borrow_raw(raw_fd) };
                match poll_input(fd, 50) {
                    Ok(true) => {
                        let n = stdin.read(&mut drain_buf).unwrap_or(0);
                        if n == 0 {
                            break;
                        }
                    }
                    _ => break,
                }
            }

            // Set the "returned from" message
            if let Some(name) = last_launched_for_tick.lock().unwrap().take() {
                *returned_for_tick.lock().unwrap() = Some((name, std::time::Instant::now()));
            }

            ctx.mark_all_dirty();
        }
    })
    .run(|_ctx| {
        // Render loop handled by framework
    })
}

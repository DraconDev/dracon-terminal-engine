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

use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use dracon_terminal_engine::framework::prelude::*;
use ratatui::layout::Rect;

mod data;
mod render;
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

    let showcase = Showcase::new(should_quit, pending.clone(), fps_counter);

    let mut app = App::new()?
        .title("Dracon Showcase")
        .fps(30)
        .theme(Theme::nord());
    let _showcase_id = app.add_widget(Box::new(showcase), Rect::new(0, 0, 80, 24));

    app.on_tick(move |ctx, _tick| {
        if quit_check.load(std::sync::atomic::Ordering::SeqCst) {
            ctx.stop();
            return;
        }

        // Compute and store FPS
        fps_for_tick.store(ctx.fps(), std::sync::atomic::Ordering::Relaxed);

        // Handle pending binary launch
        if let Some(binary_name) = pending.lock().unwrap().take() {
            let exe_dir = match std::env::current_exe() {
                Ok(p) => p.parent().unwrap().to_path_buf(),
                Err(_) => return,
            };
            let binary_path = exe_dir.join(&binary_name);

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

            let mut drain_buf = [0u8; 256];
            let _ = std::io::stdin().read(&mut drain_buf);

            let _ = ctx.resume_terminal();
            ctx.mark_all_dirty();
        }
    })
    .run(|_ctx| {
        // Render loop handled by framework
    })
}

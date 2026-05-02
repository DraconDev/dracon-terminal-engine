# Fix Showcase Child Process Terminal Corruption

## Problem
Showcase launches child TUI examples with `Command::spawn()`. Children inherit the parent's **raw terminal state** (alternate screen, raw mode, mouse capture, hidden cursor), causing instant corruption/breakage.

## Solution
Add `suspend()`/`resume()` methods to `Terminal` that restore/re-enter raw mode around child process execution.

## Files to Change

### 1. `src/core/terminal.rs` — Add suspend/resume
- Add `pub fn suspend(&mut self)` — restores original termios, exits alt screen, shows cursor
- Add `pub fn resume(&mut self)` — re-enters raw mode, enters alt screen, hides cursor

### 2. `examples/showcase.rs` — Use suspend/resume
- Add `pending_cmd: Arc<Mutex<Option<String>>>` to `Showcase` struct
- In `launch_selected()`, store command string instead of spawning
- In `.run()` callback, check for pending command:
  - `ctx.terminal.suspend()`
  - `Command::new("sh").arg("-c").arg(cmd).status()` (waits for completion)
  - `ctx.terminal.resume()`

## Verification
- `cargo build --example showcase` — compiles
- `cargo run --example showcase` → Enter on widget_gallery → child runs, exits, showcase resumes

## Test Steps
1. Launch showcase
2. Press Enter on "widget_gallery"
3. Child TUI should display correctly
4. Press q in child to exit
5. Showcase should redraw correctly
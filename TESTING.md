# Testing Guide

## Build

```bash
cargo build --example showcase
```

## Runtime Testing

### Manual Test Procedure

1. **Run the showcase:**
   ```bash
   cargo run --example showcase
   ```

2. **Test the blit_to fix:**
   - Navigate to **Login Screen** or **Theme Studio**
   - Click on the username/search input field
   - Type some text ("test" or "abc")
   - **Expected:** No white horizontal lines appear
   - **Before fix:** White horizontal lines visible under input

3. **Test all 29 scenes:**
   - Use arrow keys to navigate the scene list
   - Press Enter to launch each scene
   - Press Esc or B to return
   - Verify no visual glitches or crashes

4. **Test theme cycling:**
   - Press `Ctrl+T` to cycle themes
   - Verify theme changes propagate to all scenes
   - Check that text remains readable

5. **Test help overlay:**
   - Press `F1` or `?` to toggle help
   - Press `Esc` to dismiss
   - Verify help renders correctly in all scenes

### Expected Results

| Check | Pass Criteria |
|-------|---------------|
| No white lines after typing | Input fields render cleanly |
| All 29 scenes launch | No crashes or silent failures |
| Theme cycling works | Colors change consistently |
| Help overlay works | Renders centered with shortcuts |
| Mouse interaction | Clicks register correctly |

### Quick Smoke Test (Headless)

```bash
cargo test --test showcase_smoke_test -- --nocapture
```

This spawns the showcase binary and verifies it initializes without crashing.
# Post-Review Fix Plan

## Objective
Fix all identified issues from the 2026-05-13 comprehensive review.

## Implementation Plan

- [ ] Register 15 missing examples in Cargo.toml (`calendar`, `autocomplete`, `notification_center`, `rich_text`, `cell_pool`, `accessibility`, `form_validation`, `debug_overlay`, `menu_system`, `tutorial_app`, `event_bus_demo`, `scene_router_demo`, `todo_app`, `network_client`, `_cookbook/plugin_demo`)
- [ ] Replace `unwrap()` in `src/framework/widgets/calendar.rs:407` with `unwrap_or(fallback_date)`
- [ ] Update `src/lib.rs:42` — "37 framework widgets" → "41"
- [ ] Update `src/lib.rs:44` — "20+ built-in themes" → "21"
- [ ] Add `DraconError` to prelude in `src/framework/mod.rs`
- [ ] Update CHANGELOG.md with entries for 0.1.1, 0.1.2, 0.1.3
- [ ] Replace `unwrap()` in `src/widgets/editor.rs:2323,2487` with `unwrap_or('\u{FFFD}')`
- [ ] Add `#[allow(dead_code)]` to `CellBlock` in `src/compositor/pool.rs`
- [ ] Verify all fixes compile, clippy clean, tests pass

## Verification Criteria
- `cargo check --all-targets` passes with 0 errors
- `cargo clippy --all-targets` shows 0 warnings
- `cargo test` shows all tests passing
- All 15 newly registered examples compile via `cargo check --example <name>`

## Potential Risks and Mitigations
1. **Calendar fallback date** — Use `NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()` as fallback (guaranteed valid)
2. **Example registration** — Some examples may need path corrections if their filenames don't match expected patterns

# Dracon Terminal Engine — TODO

Revised from full review (2026-05-17). Previous review overrated several issues; ratings corrected.

---

## Showcase & Examples (NEW — High User Impact)

### 🔴 Critical (User-Facing)

- [ ] **Rewrite `chat_client` example** — Currently the flagship app example but hand-rolls everything (manual cell-by-cell rendering, manual scroll, manual input). Doesn't use framework widgets at all. Should be rewritten as Pattern-1 `Widget` composing:
  - `List<Message>` for message list (with custom bubble rendering via builder)
  - `SearchInput` for text input
  - `StatusBar` for bottom bar
  - `Modal` for emoji picker + settings
  - `NotificationCenter` / `Toast` for feedback
  - Must work offline (local messages, no async feature requirement)
  - Should demonstrate *proper* widget composition, not manual cell manipulation

- [ ] **Make `game_loop` an actual game** — Currently a passive screensaver (rocket flies, particles trail, click spawns more). No gameplay, no scoring, no framework widget usage. Should be a real game that demonstrates:
  - `HitZone` for click targets
  - `AnimationManager` for smooth animations (if usable in raw terminal mode)
  - Score tracking
  - Increasing difficulty
  - Ideas: Duck Hunt (click targets, score), Whack-a-Mole (grid of holes), or Space Invaders (moving targets)

### 🟡 High (Fill Major Gaps)

- [ ] **Add Animation showcase scene** — `AnimationManager` + `Easing` have zero demos. Scene showing panels sliding in/out, elements bouncing with different easing curves, or size/position transitions. A unique framework capability with no visibility.

- [ ] **Add Color Picker showcase scene** — `ColorPicker` widget exists but only appears in `widget_gallery` rotation. Should have a standalone scene where picking a color live-updates a preview panel. Shows reactive state + the picker widget.

- [ ] **Add Tags Input showcase scene** — `TagsInput` only in gallery rotation. Scene showing email-style tag composition with autocomplete suggestions. Demonstrates `TagsInput` + `Autocomplete` together.

- [ ] **Upgrade `cell_pool` scene** — Currently just text stats (allocation count, reuse count). Should show visual bars/gauges for pool utilization, animated allocation waves, reuse rate visualization. The concept is good (demonstrates compositor internals) but the presentation is dry.

- [ ] **Upgrade `notification_center` scene** — Add clear-all button, auto-generation timer, priority level filtering, unread badge counter. Currently just "press SPACE for random toast."

- [ ] **Upgrade `rich_text` scene** — Add tabbed markdown documents (switch between multiple samples), or a writable markdown editor with live preview split. Currently static rendering of one sample.

- [ ] **Upgrade `kanban` scene** — Add card creation (press `n` for new card), card deletion, card editing inline. Currently has drag-drop but no content manipulation.

### 🟢 Medium (Nice to Have)

- [ ] **Add Tooltip showcase scene** — `Tooltip` widget has zero visibility. Scene with buttons/icons that show tooltips on hover.

- [ ] **Add Progress/Loading scene** — `ProgressRing`, `ProgressBar`, `Spinner` together in a loading simulation. Currently only `Spinner` appears in chat_client loading state.

- [ ] **Add Widget Inspector scene** — `WidgetInspector` exists but has no demo. Live inspector showing render state, area, dirty flag, focus state of interactive widgets.

- [ ] **Add Event Logger scene** — `EventLogger` widget has no example. Live event stream with type/color filtering (keyboard, mouse, theme changes).

- [ ] **Add Password Input standalone scene** — `PasswordInput` only in `form_demo`. A login form scene (username + password + submit) would showcase it properly.

- [ ] **Add Radio Button scene** — `Radio` only in gallery rotation. Settings panel with radio groups and live preview.

- [ ] **Add Debug Overlay scene** — `debug_overlay` binary exists but isn't in showcase. FPS counter, frame time graph, cell count, render stats.

### Widgets With Zero Showcase (for reference)

`button`, `color_picker`, `debug_overlay`, `divider`, `event_logger`, `hud`, `key_value_grid`, `label`, `log_viewer`, `password_input`, `profiler`, `progress_ring`, `radio`, `status_badge`, `streaming_text`, `tags_input`, `tooltip`, `widget_inspector`

---

## Core Framework

- ~~Invest in test coverage~~ — Misperceived. Actual count is ~1,920 tests (257 unit + 1,663 integration across 25+ suites), not 26.
- ~~Add SAFETY comments to `plane.rs`~~ — Already present. All 3 unsafe call sites and the function definition have `SAFETY:` comments. Previous review was wrong.

## High

- [ ] **Start Widget trait decomposition** — ✅ **Phase 1 done** — Defined 5 sub-traits (`Renderable`, `Focusable`, `Themable`, `Commandable`, `InputHandler`) in `widget.rs`. Exported from prelude. Widget retains all methods for backward compatibility. Sub-traits available for generic bounds (`T: Renderable`). Phase 2 (making sub-traits the primary definitions) deferred to 0.2.0 as it requires changing all `impl Widget` blocks.

- [ ] **Audit `RefCell<Vec<Box<dyn Widget>>>` in App** — ✅ **Done** — Documented borrow-safety rules on `App` struct (18-line doc). No redesign needed — the event loop serializes access and the wrapper types prevent scope leaks.

## Medium

- [ ] **Convert remaining 23 ignored doc-tests** — Converted `plane.rs::crop` to `no_run` (compiles). Restored `logging.rs` (5) and `mod.rs` (1) to `no_run`. Now 5 compile-verified + 23 ignored (down from 25). Remaining 23 need Widget trait decomposition or are async/feature-gated.

- [ ] ~~Profile `on_tick` theme cloning in Pattern-2 apps~~ — Reviewed: only 4 theme clone sites in `app.rs`. Theme's `Color` fields are `Copy` (stack copy). `Arc<str>` names are ref-counted (just pointer copy). Not a performance concern.

- [ ] **Audit 34 interior mutability points** — Reviewed: 34 `RefCell` usages across 14 files. Categories:
  - **Legitimate** (can't remove): `App.widgets`, `EventBus.*`, `ScopedZoneRegistry` in widgets (render needs `&self` mutation)
  - **Pattern-driven** (could redesign): `drag_manager`/`context_menu` in `table`/`list`/`tree` — `RefCell` because `render(&self)` reads drag state but `handle_mouse(&mut self)` writes it. Moving drag rendering to `draw_to(&mut self)` would eliminate these.
  - **App-internal**: `on_tick`, `z_order_cache`, `commands` — tied to App's `&self` render loop
  - **Notification center**: `notifications`/`next_id` — modified in handlers, read in render
  Verdict: No quick wins. The Widget trait decomposition (making `render` take `&mut self` or using `draw_to`) would eliminate most of these.

- [ ] **Consider `Cow<'static, str>` for theme names** — Deferred (breaking API change). `Arc<str>` works for both static and dynamic names. For 0.2.0, consider switching to `Cow<'static, str>` so builtins use `&'static str` (zero-cost) and `Theme::custom` still supports runtime names.

## Low

- [ ] **Add `KeyCode::Unsupported` variant** — ✅ **Done** — Added `KeyCode::Unsupported(u32)` in `event.rs`, updated `kitty_key.rs` mapping, added format display in `Debug::fmt`.

- [ ] **Add proper media key `KeyCode` variants** — ✅ **Done** — `keybindings.rs:244-248` now uses `KeyCode::Media(MediaKeyCode::Play/Pause/Stop/TrackNext/TrackPrevious)` instead of Unicode substitutions.

- [ ] **Replace `panic!()` with `assert!` in test helpers** — Reviewed: 32 `panic!()` calls in `command.rs` tests are all match-arm fallthroughs (`_ => panic!("expected X")`). This is idiomatic Rust test code — not worth changing. Closing as won't-fix.

## Done ✅

- [x] ~~Fix 1 library clippy warning~~ — `form.rs:225`: `map_or(false, …)` → `is_some_and(…)`
- [x] ~~Fix example clippy warnings~~ — Cleared all 16 warnings across 13 files (cell_pool, rich_text, framework_chat, framework_demo, rich_text_demo, accessibility, chat_client, menu_system, widget_gallery, showcase scenes)
- [x] ~~Fix calendar production `unwrap()`~~ — `calendar.rs:145` inner `.unwrap()` → `.expect("hardcoded date 2024-01-01 is always valid")`
- [x] ~~Remove dead code~~ — chat_client `kb_config` field, widget_gallery `update_bg` method
- [x] ~~Fix showcase `blit_to` mutability~~ — 8 call sites + empty-plane guard in `shared_helpers.rs`
- [x] ~~Document `draw_to` design rationale~~ — Added design note in `widget.rs` explaining `&mut self` vs `&self`
- [x] ~~Document App RefCell borrow safety~~ — Added 18-line borrow-safety doc on `App` struct
- [x] ~~Add `KeyCode::Unsupported` variant~~ — Added `KeyCode::Unsupported(u32)` in `event.rs`, updated `kitty_key.rs` mapping
- [x] ~~Add proper media key `KeyCode` variants~~ — `keybindings.rs` now uses `KeyCode::Media(MediaKeyCode::Play/Pause/Stop/TrackNext/TrackPrevious)`
- [x] ~~Convert doc-tests from `ignore` to `no_run`~~ — 5 doc-tests now compile-verified (up from 4). Remaining 23 blocked by Widget trait size or async features.
- [x] ~~Audit interior mutability~~ — 34 RefCells across 14 files. No quick wins. Blocked by Widget trait decomposition.
- [x] ~~Profile theme cloning~~ — Not a performance concern (Color is Copy, Arc<str> is ref-counted).
- [x] ~~panic! in tests~~ — Won't fix. Match-arm fallthroughs are idiomatic Rust.
- [x] ~~Widget trait decomposition Phase 1~~ — Defined 5 sub-traits (Renderable, Focusable, Themable, Commandable, InputHandler), exported from prelude. Phase 2 (make sub-traits primary) deferred to 0.2.0.
- [x] ~~App RefCell audit~~ — Documented borrow-safety rules on App struct.
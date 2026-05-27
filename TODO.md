# Dracon Terminal Engine — TODO

Audit date: 2026-05-27  
Total LOC: 41,488  
Source files: 113 · Framework widgets: 47 · Themes: 21 · Examples: 98 · Test files: 110  
Test functions: ~1,500+ · Compiler: 0 warnings

---

## 🔴 High Priority

### Framework Bugs (10 found)

#### 1. `OutputParser::JsonPath` silently continues on missing segments
- **File:** `src/framework/command.rs:81-91`
- **Issue:** If a path segment is missing, falls back to parent object and continues parsing remaining segments instead of short-circuiting to `None`.
- **Fix:** Return early on missing segment; don't use `unwrap_or(cur)`.

#### 2. `i18n::load_locale()` clears translations before confirming file exists
- **File:** `src/framework/i18n.rs:131`
- **Issue:** `self.translations.clear()` runs before any path is found. If no locale file is found at any search path, existing translations are lost.
- **Fix:** Move `clear()` to after successful file read.

#### 3. `SceneTransition::SlideUp` / `SlideDown` not implemented
- **File:** `src/framework/scene_router.rs:564`
- **Issue:** Fall through to generic fade in `_` arm. Should implement proper slide transitions.
- [ ] Implement `SlideUp` / `SlideDown` transitions, or document as unsupported

#### 4. BACK key handler doesn't check scene depth changes
- **File:** `src/framework/app.rs:205-218`
- **Issue:** The general key dispatch path checks `scene_router.stack_depth()` before/after to detect scene pops and mark dirty. The BACK handler path is missing this check.
- **Fix:** Add `scene_router.stack_depth()` comparison in BACK handler.

#### 5. `CommandRunner::spawn()` uses `split_whitespace()` — no shell escaping
- **File:** `src/framework/command.rs:269`
- **Issue:** Cannot handle quoted arguments, shell escapes, or pipe characters.
- **Fix:** Use `shlex::split()` or document the limitation.

#### 6. `layout.rs` `Min` constraint consumed fully from `fixed_total`
- **File:** `src/framework/layout.rs:182`
- **Issue:** `Min(30)` + `Percentage(50)` on 100-width gives 80, not properly proportional. Min should only consume the larger of remaining and min.
- [ ] Fix mixed Min+Percentage constraint resolution

#### 7. stdin EOF silently ignored
- **File:** `src/framework/app.rs:716`
- **Issue:** `if n == 0 { }` — should trigger graceful shutdown.
- [ ] Add graceful exit on stdin EOF

#### 8. `DTRON_THEME_FILE` write error silently discarded
- **File:** `src/framework/app.rs:996`
- **Issue:** `let _ = std::fs::write(&path, ...)` — no error feedback.
- [ ] Log or propagate write failure

#### 9. `SixelImage::from_sixel()` is a stub
- **File:** `src/framework/sixel.rs:29-31`
- **Issue:** Always returns `Err("Sixel decoding not yet implemented")`.
- [ ] Either implement sixel decoding or gate behind a feature flag

#### 10. `App::theme()` vs `App::set_theme()` — duplicate APIs
- **File:** `src/framework/app.rs:460,448`
- **Issue:** Both do the exact same thing with different return types (`Self` vs `&mut Self`).
- [ ] Deprecate one; prefer `set_theme()` to avoid confusion

### Widget Bugs (5 found)

#### 11. Filter/readonly key handling returns `false` — events leak
- **File:** `src/widgets/editor.rs:710`
- **Issue:** In filter/readonly mode, navigation keys (Up/Down/Home/End) are processed but `return false` propagates the event to parent handlers, causing double-processing.
- **Fix:** Return `true` when a key was matched and consumed.

#### 12. `find_opening_bracket` off-by-one at column 0
- **File:** `src/widgets/editor.rs:1573-1577`
- **Issue:** Loop is `while c > 0` — bracket at column 0 is never checked.
- **Fix:** Change to `while c >= 0` with proper break, or handle index 0 before loop.

#### 13. `save_config()` data loss via `unwrap_or_default()`
- **File:** `src/widgets/editor.rs`
- **Issue:** `serde_json::to_string_pretty(&config).unwrap_or_default()` — if serialization fails, silently writes empty string destroying the config file.
- **Fix:** Return the error instead of `unwrap_or_default()`.

#### 14. `replace_next` cursor col after multi-byte replacement
- **File:** `src/widgets/editor.rs:601`
- **Issue:** `self.cursor_col = actual_col + replace.len()` may land in middle of a multi-byte char boundary.
- **Fix:** Call `ensure_valid_cursor_col()` after replacement, or use `next_char_boundary()` logic.

#### 15. `insert_char` with multi-cursors overwrites primary cursor row
- **File:** `src/widgets/editor.rs:1506-1508`
- **Issue:** After inserting with multiple cursors, primary cursor row is overwritten by last extra cursor's row. Possibly intentional but undocumented.
- [ ] Document multi-cursor behavior or fix row tracking

---

## 🟡 Medium Priority

### Code Quality — Framework

#### 16. `theme.rs` — 21 duplicate theme constructors (~950 lines of repetition)
- **File:** `src/framework/theme.rs` (1446 lines)
- **Issue:** Each theme constructor is ~40 lines of `Color::Rgb(r, g, b)` assignments. Structurally identical.
- [ ] Use a macro or data-driven approach to reduce to ~400 lines
- [ ] Example: `theme!(nord, "Nord", ThemeKind::Dark, bg=0x2E3440, surface=0x3B4252, ...)`

#### 17. `app.rs` — too large (1678 lines)
- **File:** `src/framework/app.rs`
- **Issue:** `run()` is ~120 lines; tests are 640 lines inline.
- [ ] Extract `render_frame()`, `handle_cursor()` from `run()`
- [ ] Move tests to `tests/app_tests.rs`

#### 18. `command.rs` — too large (1094 lines)
- **File:** `src/framework/command.rs`
- [ ] Extract parser variants into methods on `OutputParser`
- [ ] Move TOML config types (`AppConfig`, `WidgetConfig`, etc.) to `config.rs`
- [ ] Move inline tests to `tests/command_tests.rs`

#### 19. Duplicate z-order invalidation pattern
- **Files:** `src/framework/app.rs`
- **Issue:** `add_widget()`, `remove_widget()`, and `invalidate_z_order_cache()` repeated in 3 places.
- [ ] Consolidate into a single private helper

#### 20. `plugin.rs` — fully undoc'd with `#![allow(missing_docs)]`
- **File:** `src/framework/plugin.rs`
- [ ] Remove the global `#[allow]` and add doc comments to all public items

### Code Quality — Widgets

#### 21. `editor.rs` — 3025-line monolith
- **File:** `src/widgets/editor.rs`
- **Issue:** 78% of widget code in one file. Previous split attempt failed due to tightly-coupled private methods.
- **Alternative approach:** Extract `EditorState` struct (lines, cursor, selection, undo stacks) so methods can be split by concern:
  - `editor_state.rs` — document model, cursor, undo
  - `editor_render.rs` — `Widget` impl (wrap + non-wrap)
  - `editor_edit.rs` — insert, delete, selection ops
  - `editor_nav.rs` — cursor movement helpers

#### 22. `handle_event()` — 490 lines
- **File:** `src/widgets/editor.rs:635-1122`
- **Issue:** Single match with 50+ key combos + mouse events.
- [ ] Extract `handle_mode_input()`, `handle_readonly_keys()`, `handle_editing_keys()`

#### 23. `Widget::render()` — 700 lines with duplicated status bar
- **File:** `src/widgets/editor.rs:2174-2927`
- **Issue:** Status bar rendering is duplicated verbatim between wrap and non-wrap paths (~90 lines identical).
- [ ] Extract `render_status_bar()` method
- [ ] Extract wrap-line computation into single pass (currently computed twice)

#### 24. Status bar shows `Col {}` as byte index
- **File:** `src/widgets/editor.rs:2553,2827`
- **Issue:** `cursor_col + 1` is byte offset, misleading for multi-byte UTF-8.
- [ ] Rename to `Byte` or compute visual column via `get_visual_x()`

#### 25. Word deletion duplicated between `TextEditor` and `TextInput`
- **Files:** `src/widgets/editor.rs` + `src/widgets/input.rs`
- **Issue:** `delete_word_backwards/forwards` is same algorithm in both.
- [ ] Extract shared word-boundary navigation into `text.rs` or `utils.rs`

#### 26. `move_cursor()` — dead code
- **File:** `src/widgets/editor.rs:168`
- **Issue:** `#[allow(dead_code)]` on a function never called anywhere.
- [ ] Remove or replace with targeted allow if kept for API completeness

#### 27. `#![allow(unused_imports)]` on `editor.rs`
- **File:** `src/widgets/editor.rs:1`
- **Issue:** Module-level attribute suppresses all unused import warnings.
- [ ] Remove global allow; add targeted `#[allow]` on specific imports if needed

### Documentation Gaps

#### 28. Undocumented public items
- [ ] `WidgetRegistry` and all methods (`register`, `unregister`, `get`, `get_mut`, etc.)
- [ ] `SixelImage` and all methods
- [ ] `SixelRenderer` and all methods
- [ ] `EventBus::replay_last()`
- [ ] `EventBus::set_history_capacity()`
- [ ] `Ctx::stop()` — explain graceful shutdown vs immediate
- [ ] `Constraint::resolve()`
- [ ] `DirtyRegion::expand()`
- [ ] `SceneRouter::pop_force()` — how it differs from `pop()`
- [ ] `FocusManager::enter_trap()` — trap-exit-disabled behavior is non-obvious

### API Consistency

#### 29. Widget trait method duplication
- **File:** `src/framework/widget.rs`
- **Issue:** `Widget` trait methods duplicated across sub-traits (`Renderable`, `Focusable`, `Themable`, `Commandable`, `InputHandler`). Blanket impls bridge them. AGENTS.md notes cleanup planned for 0.2.0.
- [ ] Consolidate in 0.2.0; remove method-level duplication

#### 30. `TextEditor` dual error-return variants
- **File:** `src/widgets/editor.rs`
- **Issue:** Every writable operation has two variants (`save()` → `io::Result`, `save_err()` → `DraconError`). Doubles API surface.
- [ ] Pick one error type convention; remove the other variants

#### 31. Builder methods use `&mut self` instead of consuming `self`
- **Files:** `src/widgets/editor.rs`, various framework widgets
- **Issue:** `with_show_line_numbers(&mut self)` returns nothing; standard builder pattern uses `fn with_foo(mut self, ...) -> Self`.
- [ ] Evaluate switching to consuming builders for consistency

#### 32. `BoundCommand` naming inconsistency
- **File:** `src/framework/command.rs`
- **Issue:** `.parser()` / `.confirm()` / `.refresh()` / `.label()` / `.description()` don't use `with_` prefix unlike `ScrollContainer::with_content_height()`, `Animation::with_easing()`.
- [ ] Standardize on `with_` prefix for all builder methods

#### 33. `HotkeyHint` is a needless unit struct
- **File:** `src/widgets/hotkey.rs`
- **Issue:** `pub struct HotkeyHint;` with only a static method. Adds no value over a free function.
- [ ] Replace with free function or make it hold configuration (theme, spacing)

#### 34. `Component` trait unimplemented by any widget in `src/widgets/`
- **Files:** `src/widgets/component.rs` + `src/widgets/`
- **Issue:** Trait defined but never used by its sibling widgets. They use `ratatui::widgets::Widget` instead.
- [ ] Either wire up implementations or sunset the trait

---

## 🟢 Low Priority

### Code Duplication Across Examples

#### 35. `draw_text` duplicated in 10 standalone binaries
- `git_tui.rs`, `file_manager.rs`, `ide.rs`, `widget_gallery.rs`, `sqlite_browser.rs`,
  `dashboard_builder.rs`, `text_editor_demo.rs`, `table_widget.rs`, `form_demo.rs`, `system_monitor.rs`
- **Fix:** Extract to `framework::helpers::draw_text()` or `examples/shared.rs`

#### 36. `draw_rounded_border` duplicated in 6 standalone binaries
- `git_tui.rs`, `ide.rs`, `sqlite_browser.rs`, `text_editor_demo.rs`, `table_widget.rs`, `form_demo.rs`
- [ ] Extract to shared helper module

#### 37. `blit` duplicated in 3 standalone binaries
- `text_editor_demo.rs`, `table_widget.rs`, `form_demo.rs`
- [ ] Extract to shared helper module

### Theme / Keybinding Consistency

#### 38. `Theme::from_env_or(Theme::default())` — unnamed fallback
- **File:** `examples/_cookbook/tabbed_panels.rs:127`
- [ ] Replace with explicit fallback like `Theme::from_env_or(Theme::nord())`

#### 39. Redundant `KeyCode::Char('?')` in 2 showcase scene help toggles
- **Files:** `scenes/tags_input_scene.rs:320`, `scenes/modal_demo.rs:513,519`
- [ ] Remove `?` override or document why it's needed (probably for UX discoverability)

### Dead Code / Minor Cleanup

#### 40. Suspicious `.clone()` after inner attribute
- **File:** `examples/_plugins/lib.rs:10`
- **Issue:** `#![allow(dead_code)].clone()` — `.clone()` on inner attribute return value (`()`) is a no-op. Looks like a copy-paste error.
- [ ] Remove `.clone()`

#### 41. `#[allow(unused)] use std::cell::RefCell` — redundant import
- **File:** `src/framework/layout.rs:8-9`
- [ ] Clean up

#### 42. `#[allow(unused)] theme: Color` on `SixelRenderer`
- **File:** `src/framework/sixel.rs:68`
- [ ] Wire it up or remove dead field

#### 43. `on_focus_change_internal()` — never called
- **File:** `src/framework/focus.rs:196`
- [ ] Remove dead function

#### 44. `Ctx::frame_count` and `Ctx::last_frame` — never read
- **File:** `src/framework/ctx.rs:49-52`
- [ ] Remove or wire up to actual frame count tracking

### Unsafe Block Documentation

#### 45. `unsafe` blocks missing `// SAFETY:` comments
- [ ] `src/compositor/plane.rs` (5 blocks)
- [ ] `src/backend/tty.rs` (4 blocks)
- [ ] `examples/showcase/main.rs` (1 block)
- [ ] `examples/game_loop.rs`, `input_debug.rs`, `desktop.rs`, `arena.rs` (1 each)

### Testing Gaps

#### 46. `text_input_base` — no integration tests in `tests/`
- [ ] Test Tab between fields, focus styling, scroll behavior
- [ ] Test mask/unmask toggle for `PasswordInput`

#### 47. `lsp-server` extension — 22 `unwrap()` calls
- **File:** `extensions/lsp-server/src/main.rs`
- [ ] Replace with proper `Result` propagation
- [ ] Add error messages for each fallible operation

#### 48. `cargo-dracon` CLI — zero tests
- **Directory:** `crates/cargo-dracon/src/`
- [ ] Add test: template generation produces compilable output
- [ ] Add snapshot tests for generated file contents

#### 49. Event bus — no benchmarks
- **File:** `src/framework/event_bus.rs`
- [ ] Add criterion benchmark: publish/subscribe throughput at 1/10/100 subscribers
- [ ] Add benchmark: filter vs unfiltered dispatch

### Build / Infrastructure

#### 50. `CHANGELOG.md` format drift
- [ ] Enforce `keepachangelog.com` format in CI
- [ ] Add `[Unreleased]` section at top for tracking WIP changes

#### 51. `dracon.toml` — no validation
- [ ] Add TOML schema validation
- [ ] Test `KeybindingConfig::parse_keybinding()` edge cases

#### 52. `input/mapping.rs` — deprecated identity function
- **File:** `src/input/mapping.rs`
- [ ] Remove if truly unused

---

## 🧪 Ideas (Further Investigation)

- **Panic safety audit**: Search for potential panics (index arithmetic, `[..]` slicing)
- **Thread safety**: Framework is single-threaded by design; document as explicit constraint
- **Plugin architecture**: `PluginRegistry` exists but only one sample plugin; evaluate real-world use
- **Tracing feature**: Currently optional behind `tracing` feature flag; verify no perf regression when disabled
- **macOS/Windows testing**: `libc` gated to non-Windows; no macOS-specific test coverage
- **Snapshot tests**: `insta` in dev-dependencies but no snapshot test files; add for `Plane` or `Theme` serialization
- **Build optimization**: Profile debug build time (slow generics in `Plane`, `Compositor`, `Table<T>`)
- **`bitflags::serde` feature audit**: Check if `serde` feature on `bitflags` is actually needed

---

## 📋 Summary

| Category | Items | Status |
|----------|-------|--------|
| High-severity framework bugs | 10 | 🔴 Open |
| High-severity widget bugs | 5 | 🔴 Open |
| Medium code quality | 13 | 🟡 Open |
| API consistency issues | 7 | 🟡 Open |
| Missing documentation | 10 items | 🟡 Open |
| Code duplication (examples) | 19 copies (3 functions) | 🟢 Open |
| Low-priority cleanup | 8 | 🟢 Open |
| Testing gaps | 4 | 🟢 Open |
| Unsafe block docs | 12 blocks in 8 files | 🟢 Open |
| Build/infrastructure | 3 | 🟢 Open |

**Previous completed items (rolled over):**
- ✅ lru unsoundness fix (ratatui 0.30)
- ✅ CI pipeline (ci, bench, plugin-ci, release, outdated, changelog)
- ✅ Test coverage gaps for 4 framework widgets
- ✅ `App::new()` documentation
- ✅ `compositor/size_test.rs` moved to `tests/`
- ✅ Doc examples (14 compile-tested)

**Note:** The previous `editor.rs` split attempt was marked SKIPPED (tightly-coupled call graph). The new TODO suggests an alternative approach: extract `EditorState` struct first, then split methods by concern.

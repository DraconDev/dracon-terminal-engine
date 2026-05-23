# Dracon Terminal Engine — Project Audit

**Created:** 2026-05-23  
**Version:** 0.1.10  
**Total LOC:** 39,065  
**Framework modules:** 24  
**Framework widgets:** 53  
**Themes:** 21  
**Examples:** 57  
**Test files:** 76  
**Test functions:** ~1,500+

---

## 📦 Project Overview

### What It Is
`dracon-terminal-engine` is a terminal application framework for Rust. Not a TUI library — a complete runtime that owns the terminal, input, rendering, and event loop.

### Key Characteristics
- **Mouse-friendly** with z-indexed planes
- **53 built-in widgets** in the framework
- **21 themes** (Nord, Dracula, Catppuccin, etc.)
- **Dirty rendering** for efficient updates
- **Focus management** system
- **Command-driven architecture** — AI can enumerate and trigger all actions
- **Single binary deployment** — no external runtime

### License
- **AGPL-3.0-only** (open source)
- **Commercial License** available

---

## 📁 Source Structure

### Core Modules (`src/`)

| Module | LOC | Pub Fns | Description |
|--------|-----|---------|-------------|
| `lib.rs` | 232 | — | Main entry point |
| `utils.rs` | 1,217 | — | Catch-all utilities (⚠️ large) |
| `text.rs` | 387 | — | Text utilities |
| `layout.rs` | 145 | — | Layout engine |
| `system.rs` | 288 | — | System metrics |
| `error.rs` | — | — | Error types |
| **compositor/** | — | — | Plane pool, rendering |
| **input/** | — | — | Event parsing |
| **integration/** | — | — | ratatui integration |
| **widgets/** | — | — | Standalone widgets |
| **framework/** | — | — | App framework + widgets |
| **backend/** | — | — | TTY backend |
| **visuals/** | — | — | Icons, accessibility |
| **core/** | — | — | Terminal core |

### Large Files (>400 LOC)

| File | LOC | Priority |
|------|-----|----------|
| `src/widgets/editor.rs` | 3,025 | 🔴 HIGH |
| `src/utils.rs` | 1,217 | 🟡 MEDIUM |
| `src/framework/theme.rs` | 1,447 | 🟢 OK |
| `src/framework/app.rs` | 1,667 | 🟢 OK |
| `src/framework/command.rs` | 1,095 | 🟢 OK |
| `src/framework/scene_router.rs` | 625 | 🟢 OK |
| `src/visuals/accessibility.rs` | 416 | 🟢 OK |
| `src/visuals/icons.rs` | 412 | 🟢 OK |

### Framework Modules (`src/framework/`)

| Module | LOC | Pub Fns | Description |
|--------|-----|---------|-------------|
| `app.rs` | 1,667 | 21 | Main App builder |
| `theme.rs` | 1,447 | 26 | 21 built-in themes |
| `command.rs` | 1,095 | 19 | Command registry |
| `scene_router.rs` | 625 | 29 | Multi-screen navigation |
| `event_bus.rs` | 528 | 20 | Publish/subscribe |
| `i18n.rs` | 523 | 11 | Internationalization |
| `keybindings.rs` | 599 | 12 | Keybinding system |
| `animation.rs` | 462 | 15 | Tweening animations |
| `marquee.rs` | 481 | 13 | Drag selection |
| `hitzone.rs` | 401 | 23 | Click detection |
| `focus.rs` | 333 | 15 | Focus management |
| `ctx.rs` | 289 | 36 | Ctx for callbacks |
| `dirty_regions.rs` | 288 | 12 | Partial screen updates |
| `layout.rs` | 454 | 11 | Constraint-based layout |
| `scroll.rs` | 246 | 18 | Scroll management |
| `dragdrop.rs` | 225 | 14 | Drag-and-drop |
| `sixel.rs` | 147 | 10 | Sixel image support |
| `plugin.rs` | 199 | 8 | Plugin system |
| `widget.rs` | 353 | 3 | Widget trait |
| `widget_container.rs` | 149 | 15 | Widget wrapper |
| `event_dispatcher.rs` | 179 | 6 | Event routing |
| `logging.rs` | 209 | 5 | Debug logging |
| `mod.rs` | 107 | 0 | Module re-exports |

### Framework Widgets (`src/framework/widgets/`)

| Widget | LOC | Status | Priority |
|--------|-----|--------|----------|
| `color_picker.rs` | 750 | — | 🟡 MEDIUM |
| `kanban.rs` | 744 | — | 🟡 MEDIUM |
| `tags_input.rs` | 691 | — | 🟡 MEDIUM |
| `table.rs` | 676 | Tests ✅ | 🟢 OK |
| `calendar.rs` | 628 | — | 🟡 MEDIUM |
| `form.rs` | 585 | Tests ✅ | 🟢 OK |
| `tree.rs` | 567 | Tests ✅ | 🟢 OK |
| `command_palette.rs` | 558 | — | 🟡 MEDIUM |
| `list.rs` | 557 | Tests ✅ | 🟢 OK |
| `context_menu.rs` | 865 | Tests ✅ | 🟢 OK |
| `confirm_dialog.rs` | 501 | Tests ✅ | 🟢 OK |
| `sparkline.rs` | 456 | Tests ✅ | 🟢 OK |
| `autocomplete.rs` | 453 | — | 🟡 MEDIUM |
| `rich_text.rs` | 436 | — | 🟡 MEDIUM |
| `log_viewer.rs` | 423 | Tests ✅ | 🟢 OK |
| `split.rs` | 349 | Tests ✅ | 🟢 OK |
| `notification_center.rs` | 342 | — | 🟢 LOW |
| `divider.rs` | 330 | — | 🟢 LOW |
| `select.rs` | 294 | — | 🟢 LOW |
| `gauge.rs` | 263 | Tests ✅ | 🟢 OK |
| `key_value_grid.rs` | 260 | Tests ✅ | 🟢 OK |
| `streaming_text.rs` | 254 | Tests ✅ | 🟢 OK |
| `tabbar.rs` | 252 | — | 🟢 LOW |
| `menu_bar.rs` | 237 | Tests ✅ | 🟢 OK |
| `hud.rs` | 242 | — | 🟢 LOW |
| `text_input_base.rs` | 287 | Tests ✅ | 🟢 OK |
| `checkbox.rs` | 217 | — | 🟢 LOW |
| `radio.rs` | 215 | — | 🟢 LOW |
| `button.rs` | 214 | Tests ✅ | 🟢 OK |
| `toggle.rs` | 205 | — | 🟢 LOW |
| `toast.rs` | 201 | Tests ✅ | 🟢 OK |
| `list_common.rs` | 201 | Tests ✅ | 🟢 OK |
| `status_badge.rs` | 198 | Tests ✅ | 🟢 OK |
| `status_bar.rs` | 186 | Tests ✅ | 🟢 OK |
| `profiler.rs` | 176 | — | 🟢 LOW |
| `widget_inspector.rs` | 160 | — | 🟢 LOW |
| `event_logger.rs` | 156 | — | 🟢 LOW |
| `password_input.rs` | 143 | Tests ✅ | 🟢 OK |
| `progress_bar.rs` | 143 | — | 🟢 LOW |
| `spinner.rs` | 141 | — | 🟢 LOW |
| `search_input.rs` | 135 | — | 🟢 LOW |
| `debug_overlay.rs` | 129 | Tests ✅ | 🟢 OK |
| `tooltip.rs` | 116 | — | 🟢 LOW |
| `label.rs` | 133 | Tests ✅ | 🟢 OK |
| `mod.rs` | 99 | — | Re-exports |
| `text_editor_adapter.rs` | 262 | Tests ✅ | 🟢 OK |
| `progress_ring.rs` | 384 | Tests ✅ | 🟢 OK |
| `modal.rs` | 389 | Tests ✅ | 🟢 OK |
| `breadcrumbs.rs` | 352 | Tests ✅ | 🟢 OK |
| `slider.rs` | 275 | Tests ✅ | 🟢 OK |

### Standalone Widgets (`src/widgets/`)

| Widget | LOC | Description |
|--------|-----|-------------|
| `editor.rs` | 3,025 | TextEditor (view/edit) |
| `editor_search.rs` | 293 | Search state |
| `input.rs` | 286 | Text input |
| `context_menu.rs` | 83 | Context menu |
| `panel.rs` | 50 | Panel container |
| `component.rs` | 53 | Component trait |
| `button.rs` | 41 | Button |
| `hotkey.rs` | 22 | Hotkey display |
| `mod.rs` | 23 | Module re-exports |

### Compositor (`src/compositor/`)

| File | Description |
|------|-------------|
| `engine.rs` | Compositor engine |
| `plane.rs` | Plane (⚠️ unsafe blocks) |
| `pool.rs` | Plane pool |
| `filter.rs` | Plane filtering |
| `size_test.rs` | Size check |
| `mod.rs` | Module re-exports |

### Input (`src/input/`)

| File | Description |
|------|-------------|
| `parser.rs` | Input parsing |
| `event.rs` | Event types |
| `reader.rs` | Input reader |
| `mapping.rs` | ⚠️ Deprecated |
| `kitty_key.rs` | Kitty keyboard |
| `async_reader.rs` | Async reader |
| `mod.rs` | Module re-exports |

### Visuals (`src/visuals/`)

| File | LOC | Description |
|------|-----|-------------|
| `icons.rs` | 412 | Icon library |
| `accessibility.rs` | 416 | Screen reader support |
| `osc.rs` | 54 | OSC commands |
| `sync.rs` | 16 | Sync operations |
| `mod.rs` | 15 | Module re-exports |

### Backend (`src/backend/`)

| File | Description |
|------|-------------|
| `tty.rs` | TTY backend (⚠️ unsafe) |
| `mod.rs` | Module re-exports |

---

## 📊 Test Coverage

### Integration Tests (`tests/`)

| Test File | Functions | Widget | Status |
|----------|-----------|--------|--------|
| `widget_tests.rs` | 167 | Multiple | 🔴 HEAVY |
| `theme_test.rs` | 116 | Theme | 🔴 HEAVY |
| `command_output_test.rs` | 82 | — | 🟡 MEDIUM |
| `app_tick_test.rs` | 77 | App | 🟡 MEDIUM |
| `compositor_test.rs` | 60 | Compositor | 🟡 MEDIUM |
| `utils_test.rs` | 60 | Utils | 🟡 MEDIUM |
| `event_handler_test.rs` | 57 | — | 🟡 MEDIUM |
| `focus_test.rs` | 34 | — | 🟡 MEDIUM |
| `scroll_test.rs` | 43 | — | 🟡 MEDIUM |
| `untested_widgets_test.rs` | 29 | Various | 🟡 MEDIUM |
| `text_editor_test.rs` | 48 | TextEditor | 🟡 MEDIUM |
| `complex_integration_test.rs` | 28 | — | 🟡 MEDIUM |
| `async_command_runner_test.rs` | 37 | — | 🟡 MEDIUM |
| `button_test.rs` | 28 | Button | 🟢 OK |
| `multi_widget_test.rs` | 26 | Multiple | 🟢 OK |
| `text_editor_adapter_test.rs` | 19 | Adapter | 🟢 OK |
| `text_editor_adapter_edge_test.rs` | 25 | Adapter | 🟢 OK |
| `phase1_widget_test.rs` | 24 | Various | 🟢 OK |
| `widget_sparkline_test.rs` | 37 | Sparkline | ✅ GOOD |
| `widget_progress_ring_test.rs` | 34 | ProgressRing | ✅ GOOD |
| `event_bus_test.rs` | 10 | EventBus | 🟢 OK |
| `dragdrop_test.rs` | 12 | DragDrop | 🟢 OK |
| `tree_widget_test.rs` | 13 | Tree | 🟢 OK |
| `scene_router_test.rs` | 16 | SceneRouter | 🟢 OK |
| `showcase_app_compliance_test.rs` | 12 | Showcase | 🟢 OK |
| `form_validation_test.rs` | 10 | Form | 🟢 OK |
| `form_widget_test.rs` | 13 | Form | 🟢 OK |
| `widget_list_common_test.rs` | 25 | List | ✅ GOOD |
| `widget_log_viewer_test.rs` | 23 | LogViewer | 🟢 OK |
| `widget_streaming_text_test.rs` | 21 | StreamingText | 🟢 OK |
| `widget_status_badge_test.rs` | 17 | StatusBadge | 🟢 OK |
| `toast_test.rs` | 14 | Toast | 🟢 OK |
| `widget_confirm_dialog_test.rs` | 26 | ConfirmDialog | 🟢 OK |
| `resize_test.rs` | 10 | — | 🟢 OK |
| `panel_test.rs` | 7 | Panel | 🟢 OK |
| `breadcrumbs_test.rs` | 11 | Breadcrumbs | 🟢 OK |
| `widget_password_input_test.rs` | 20 | PasswordInput | 🟢 OK |
| `widget_text_input_base_test.rs` | 26 | TextInput | 🟢 OK |
| `label_test.rs` | 20 | Label | 🟢 OK |
| `widget_gauge_test.rs` | 19 | Gauge | 🟢 OK |
| `clipboard_test.rs` | 11 | — | 🟢 OK |
| `input_reader_test.rs` | 28 | Input | 🟢 OK |
| `menu_test.rs` | 11 | Menu | 🟢 OK |
| `modal_widget_test.rs` | 13 | Modal | 🟢 OK |
| `table_sort_persistence_test.rs` | 9 | Table | 🟢 OK |
| `splitpane_test.rs` | 25 | SplitPane | 🟢 OK |
| `status_bar_test.rs` | 10 | StatusBar | 🟢 OK |
| `streaming_text_test.rs` | 16 | StreamingText | 🟢 OK |
| `syntax_highlighting_test.rs` | 15 | Syntax | 🟢 OK |
| `widget_gallery_edge_test.rs` | 14 | Gallery | 🟢 OK |
| `debug_overlay_test.rs` | 11 | DebugOverlay | 🟢 OK |
| `command_palette_test.rs` | 15 | CommandPalette | 🟢 OK |
| `phase2_3_4_widget_test.rs` | 27 | Various | 🟢 OK |
| `profiler_test.rs` | 10 | Profiler | 🟢 OK |
| `context_menu_test.rs` | 9 | ContextMenu | 🟢 OK |
| `hitzone_test.rs` | 9 | HitZone | 🟢 OK |
| `widget_key_value_grid_test.rs` | 14 | KeyValueGrid | 🟢 OK |
| `widget_slider_test.rs` | 11 | Slider | 🟢 OK |
| `accessibility_test.rs` | 9 | Accessibility | 🟢 OK |
| `theme_propagation_test.rs` | 16 | Theme | 🟢 OK |
| `theme_validation_test.rs` | 13 | Theme | 🟢 OK |
| `widget_test.rs` | 16 | Various | 🟢 OK |
| `example_smoke_test.rs` | 10 | Examples | 🟢 OK |
| `example_quit_test.rs` | 14 | Examples | 🟢 OK |
| `showcase_smoke_test.rs` | 1 | Showcase | 🟢 OK |
| `editor_smoke_test.rs` | 1 | Editor | 🟢 OK |
| `compositor_stress_test.rs` | 12 | Compositor | 🟢 OK |
| `filter_test.rs` | 24 | Filter | 🟢 OK |
| `network_widget_test.rs` | 10 | Network | 🟢 OK |
| `widget_snapshot_tests.rs` | 4 | Snapshot | 🟢 LOW |
| `compositor_size_test.rs` | 0 | Size | 🟢 LOW |
| `framework_benchmarks.rs` | 0 | Benchmark | 🟢 LOW |
| `performance_benchmarks.rs` | 0 | Benchmark | 🟢 LOW |
| `property_tests.rs` | 0 | Property | 🟢 LOW |

**Total: 76 test files, ~1,500+ test functions**

---

## 🧩 Examples (57 total)

### Full Applications
| Example | LOC | Description |
|---------|-----|-------------|
| `ide.rs` | ~1,500 | IDE-style editor |
| `git_tui.rs` | ~1,100 | Git TUI |
| `scene_router_demo.rs` | ~700 | Multi-screen nav |
| `form_demo.rs` | ~900 | Form demo |
| `table_widget.rs` | ~950 | Table widget |
| `chat_client.rs` | ~650 | Chat app |
| `sqlite_browser.rs` | ~850 | SQLite browser |
| `todo_app.rs` | ~920 | Todo app |
| `network_client.rs` | ~670 | Network client |
| `theme_switcher.rs` | ~860 | Theme cycling |

### Framework Demos
| Example | Description |
|---------|-------------|
| `framework_demo.rs` | Basic usage |
| `framework_widgets.rs` | Widget gallery |
| `framework_file_manager.rs` | File manager |
| `framework_chat.rs` | Chat UI |
| `plugin_demo.rs` | Plugin system |
| `event_bus_demo.rs` | Event bus |
| `command_dashboard.rs` | Command palette |

### Raw Terminal Demos
| Example | Description |
|---------|-------------|
| `desktop.rs` | Desktop env |
| `game_loop.rs` | Game loop |
| `input_debug.rs` | Input debug |
| `arena.rs` | Game arena |

### Other Examples
| Example | Description |
|---------|-------------|
| `modal_demo.rs` | Modal dialogs |
| `text_editor_demo.rs` | Text editor |
| `rich_text_demo.rs` | Rich text |
| `tutorial_app.rs` | Tutorial |
| `widget_tutorial.rs` | Widget tutorial |
| `basic_raw.rs` | Raw basics |
| `god_mode.rs` | God mode |
| `from_toml.rs` | TOML config |
| `showcase/` | Showcase launcher |

---

## 🏗️ Extensions & Crates

### Extensions (`extensions/`)
| Extension | Description |
|----------|-------------|
| `lsp-server/` | LSP server (⚠️ 22 unwraps) |
| `_plugins/` | Sample plugins |
| `vscode/` | VSCode extension |
| `README.md` | Extension docs |

### Crates (`crates/`)
| Crate | Description |
|-------|-------------|
| `cargo-dracon/` | Cargo subcommand (⚠️ untested) |
| `dracon-macros/` | Proc macros |

---

## 🔴 HIGH PRIORITY TASKS

### 1. Security Vulnerabilities

- [ ] Monitor transitive unmaintained dependencies:
  - [ ] `bincode 1.3.3` — RUSTSEC-2025-0141 (unmaintained)
  - [ ] `yaml-rust 0.4.5` — RUSTSEC-2024-0320 (upstream: syntect)
- [ ] Schedule quarterly `cargo outdated` review
- [ ] Verify `cargo audit` runs in CI
- [ ] Add security policy to `SECURITY.md`

### 2. Production Unwraps (~64 calls)

**In `src/`: (~50 unwraps)**

- [ ] Audit all `unwrap()`/`expect()` in `src/compositor/`
  - [ ] `plane.rs` — 5+ unwraps (unsafe context)
  - [ ] `pool.rs` — ?
  - [ ] `filter.rs` — ?
- [ ] Audit all `unwrap()`/`expect()` in `src/input/`
  - [ ] `parser.rs` — ?
  - [ ] `reader.rs` — ?
- [ ] Audit all `unwrap()`/`expect()` in `src/framework/`
  - [ ] `app.rs` — 2+ (signal handlers)
  - [ ] `event_bus.rs` — ?
  - [ ] `theme.rs` — ?
  - [ ] `command.rs` — ?
- [ ] Audit all `unwrap()`/`expect()` in `src/widgets/`
  - [ ] `editor.rs` — ?
  - [ ] `editor_search.rs` — ?
- [ ] Replace with proper `Result` propagation where appropriate

**In `extensions/lsp-server/` (22 unwraps)**

- [ ] Replace all 22 unwraps in `extensions/lsp-server/src/main.rs`
- [ ] Add proper error messages
- [ ] Test error recovery paths

### 3. Unsafe Blocks (Missing SAFETY Comments)

**`src/compositor/plane.rs` — 5 blocks**

- [ ] Line 196: `next_char_unchecked` call
- [ ] Line 201: `next_char_unchecked` call
- [ ] Line 266: `next_char_unchecked` call
- [ ] Line 276: `next_char_unchecked` call
- [ ] Line 478: `unsafe fn next_char_unchecked`

**`src/backend/tty.rs` — 4 blocks**

- [ ] Line 12: `libc::ioctl`
- [ ] Line 26: `libc::tcsetattr`
- [ ] Line 38: `libc::cfmakeraw`
- [ ] Line 46: `libc::tcgetattr`
- [ ] Line 60: `libc` operations

**`src/framework/app.rs` — 2 blocks**

- [ ] Line 887: Signal handler (already has SAFETY)
- [ ] Line 893: Signal handler

**Examples — multiple blocks**

- [ ] `examples/arena.rs` — unsafe gaming loop
- [ ] `examples/game_loop.rs` — unsafe timer
- [ ] `examples/desktop.rs` — unsafe operations
- [ ] `examples/input_debug.rs` — unsafe input

---

## 🟡 MEDIUM PRIORITY TASKS

### 4. Code Organization

**`utils.rs` (1,217 LOC) — Catch-all utilities**

- [ ] Extract `visual_width`, `truncate`, `formatting` → `src/text.rs`
- [ ] Extract `clamp`, `bounding_box` → `src/layout.rs`
- [ ] Extract `parse_hex_color`, `darken`, `lighten` → `src/visuals/` or `theme.rs`
- [ ] Extract `ansi` parsing → `src/visuals/` or `text.rs`
- [ ] Remaining helpers → `src/framework/helpers.rs`
- [ ] Document remaining functions with doc comments

**`src/input/mapping.rs` — Deprecated**

- [ ] Remove deprecated identity functions
- [ ] Verify `to_ui_event()` function is still needed
- [ ] Move any remaining logic to `event.rs`

**`src/framework/prelude.rs` — Module Organization**

- [ ] Consider extracting `prelude.rs` as standalone file
- [ ] Audit all re-exports for API surface

**Large Widget Refactors**

- [ ] `color_picker.rs` (750 LOC) — consider splitting:
  - Color calculation → shared utility
  - UI rendering → smaller methods
- [ ] `kanban.rs` (744 LOC) — consider:
  - Card management → separate type
  - Column logic → extracted functions
- [ ] `tags_input.rs` (691 LOC) — consider:
  - Tag parsing → utility module
  - Autocomplete → shared with autocomplete.rs
- [ ] `calendar.rs` (628 LOC) — consider:
  - Date calculation → utility module
  - Rendering → simpler methods

### 5. Test Coverage Gaps

**Integration Tests Needed**

- [ ] `text_input_base_test.rs` — add integration tests:
  - [ ] Tab between fields
  - [ ] Focus styling verification
  - [ ] Scroll behavior
  - [ ] PasswordInput mask/unmask toggle
- [x] `TagsInput` widget — add tests (690 LOC, ✅ **52 tests**)
- [x] `Calendar` widget — add tests (628 LOC, ✅ **56 tests**)
- [x] `ColorPicker` widget — add tests (750 LOC, ✅ **54 tests**)
- [ ] `Autocomplete` widget — add tests (453 LOC, 0 tests)
- [ ] `RichText` widget — add tests (436 LOC, 0 tests)

**Benchmark Tests**

- [ ] `event_bus.rs` micro-benchmarks:
  - [ ] Publish/subscribe throughput at 1/10/100 subscribers
  - [ ] Filter vs unfiltered dispatch
  - [ ] Add to criterion suite
- [ ] `command.rs` benchmarks:
  - [ ] Command enumeration performance
  - [ ] Command execution overhead
- [ ] Compositor benchmarks (already exists, verify coverage)

**Snapshot Tests**

- [ ] `insta` is in dev-deps but unused:
  - [ ] Add first snapshot for `Plane` serialization
  - [ ] Add snapshot for `Theme` JSON/YAML
  - [ ] Add widget rendering snapshots

### 6. Documentation

**Doc Test Conversion (19 remaining)**

- [ ] `SceneRouter` — add compile-tested example
- [ ] `ContextMenu` — add compile-tested example
- [ ] `Accessibility` — add compile-tested example
- [ ] `Theme::custom` — add example
- [ ] `Theme::from_env_or` — add example
- [ ] `KeybindingSet::format_hint` — add example
- [ ] `App::from_toml` — add example
- [ ] `App::shield_input` — add example
- [ ] `I18n` module — add examples
- [ ] `i18n::tr`, `i18n::trf` — add examples

**API Documentation**

- [ ] `ctx.rs` — 36 pub fns, audit for missing docs
- [ ] `command.rs` — 19 pub fns, audit for missing docs
- [ ] `scene_router.rs` — 29 pub fns, audit for missing docs
- [ ] `widget_container.rs` — 15 pub fns, audit for missing docs
- [ ] All framework widgets — audit public methods

### 7. Build Optimization

- [ ] Profile `debug` build time:
  - [ ] Identify slow generics in `Plane<T>`, `Compositor`, `Table<T>`
  - [ ] Check compile times with `cargo bloat --time`
- [ ] Add `lto = "thin"` for release builds
- [ ] Evaluate `codegen-units = 1` for release
- [ ] Check if `bitflags::serde` feature is actually used
- [ ] Consider incremental compilation caching

### 8. Configuration & Validation

**`dracon.toml`**

- [ ] Add TOML schema validation
- [ ] Add unit tests for `KeybindingConfig::parse_keybinding()`:
  - [ ] Uppercase key handling
  - [ ] Malformed chords
  - [ ] Invalid modifiers
- [ ] Test `DraconError::InvalidKeybinding` path

**`CHANGELOG.md`**

- [ ] Enforce `keepachangelog.com` format in CI
- [ ] Add `[Unreleased]` section at top
- [ ] Fix subsection names ("Fixed" vs "Changed")
- [ ] Add changelog lint job to CI

---

## 🟢 LOW PRIORITY TASKS

### 9. Ideas (Further Investigation)

**Safety & Correctness**

- [ ] Panic safety audit:
  - [ ] Search for index arithmetic that could panic
  - [ ] Check `[..]` slicing for bounds
  - [ ] Verify all `usize` conversions are safe
- [ ] Thread safety documentation:
  - [ ] Document single-threaded design constraint
  - [ ] Add thread safety comments to shared state
- [ ] Error propagation consistency:
  - [ ] Audit error types across modules
  - [ ] Standardize error messages

**Architecture Evaluation**

- [ ] Plugin architecture (`PluginRegistry`):
  - [ ] Evaluate real-world usage
  - [ ] Add plugin loading benchmarks
  - [ ] Document plugin API stability
- [ ] Tracing feature:
  - [ ] Verify no perf regression when disabled
  - [ ] Check feature gate correctness
- [ ] Async runtime usage:
  - [ ] Audit `tokio` usage patterns
  - [ ] Verify async/await correctness
  - [ ] Check for blocking calls in async context

**Cross-Platform Testing**

- [ ] macOS testing:
  - [ ] `libc` is gated to non-Windows
  - [ ] No macOS CI coverage (only Linux/Windows)
  - [ ] Add macOS CI job
- [ ] Windows testing:
  - [ ] Verify all Windows paths
  - [ ] Test with Windows Terminal
  - [ ] Test with legacy cmd.exe

**Modernization**

- [ ] Consider `rustfmt` updates
- [ ] Evaluate `clippy` suggestions
- [ ] Check for `dead_code` warnings
- [ ] Audit `#[allow]` attributes

### 10. Nice-to-Have Features

**Dev Dependencies**

- [ ] Update `criterion 0.5.1` → `0.8.2`
- [ ] Update `itertools 0.10` → `0.13`
- [ ] Update `proptest 1.4` → latest
- [ ] Add `cargo-outdated` to CI

**CI/CD**

- [ ] Add `cargo upgrade` to maintenance workflow
- [ ] Add `cargo-diet` for binary size
- [ ] Add `cargo-udeps` for dead dependency check
- [ ] Add `cargo-fuzz` for fuzz testing

**Documentation**

- [ ] Add architecture diagram to `AGENTS.md`
- [ ] Add widget interaction diagram
- [ ] Create API stability policy
- [ ] Add migration guide for major versions

**Tooling**

- [ ] `cargo-dracon` scaffolding tool:
  - [ ] Add template generation tests
  - [ ] Add snapshot tests for generated files
  - [ ] Verify generated code compiles
- [ ] Add pre-commit hooks for formatting
- [ ] Add editor config for VS Code

### 11. Widget-Specific Tasks

**Under-tested Widgets**

- [x] `TagsInput` (691 LOC) — ✅ 52 tests
- [x] `Calendar` (628 LOC) — ✅ 56 tests
- [x] `ColorPicker` (750 LOC) — ✅ 54 tests
- [ ] `Autocomplete` (453 LOC) — 0 tests
- [ ] `RichText` (436 LOC) — 0 tests
- [ ] `NotificationCenter` (342 LOC) — 0 tests
- [ ] `Divider` (330 LOC) — 0 tests
- [ ] `Select` (294 LOC) — 0 tests
- [ ] `TabBar` (252 LOC) — 0 tests
- [ ] `Hud` (242 LOC) — 0 tests
- [ ] `Slider` (275 LOC) — 11 tests (needs more)
- [ ] `Radio` (215 LOC) — 0 tests
- [ ] `Checkbox` (217 LOC) — 0 tests
- [ ] `Toggle` (205 LOC) — 0 tests
- [ ] `ProgressBar` (143 LOC) — 0 tests
- [ ] `Spinner` (141 LOC) — 0 tests
- [ ] `SearchInput` (135 LOC) — 0 tests
- [ ] `Tooltip` (116 LOC) — 0 tests
- [ ] `DebugOverlay` (129 LOC) — 11 tests
- [ ] `Profiler` (176 LOC) — 10 tests
- [ ] `EventLogger` (156 LOC) — 0 tests
- [ ] `StatusBar` (186 LOC) — 10 tests
- [ ] `WidgetInspector` (160 LOC) — 0 tests

**Widget Feature Parity**

- [ ] Verify all 53 widgets have:
  - [ ] Theme propagation
  - [ ] Focus handling
  - [ ] Mouse handling
  - [ ] Keyboard navigation
  - [ ] Help overlay
  - [ ] Keybinding support

### 12. Example Improvements

- [ ] Add smoke tests for untested examples
- [ ] Add integration tests for complex examples:
  - [ ] `ide.rs`
  - [ ] `git_tui.rs`
  - [ ] `scene_router_demo.rs`
  - [ ] `form_demo.rs`
- [ ] Add benchmarks for example startup time
- [ ] Document example patterns in `AGENTS.md`

---

## 📈 Progress Tracking

### Completed (from 2026-05-23 session)

| Item | Status |
|------|--------|
| lru unsoundness fix (ratatui 0.30) | ✅ DONE |
| CI pipeline (outdated + changelog) | ✅ DONE |
| Security advisories updated | ✅ DONE |
| editor.rs split documented | ✅ DONE |
| App::new().unwrap() docs fixed | ✅ DONE |
| Test coverage gaps (progress_ring, sparkline, list_common) | ✅ DONE |
| size_test.rs moved to tests/ | ✅ DONE |
| set_theme doc comment added | ✅ DONE |
| 14 compile-tested doc examples added | ✅ DONE |

### Doc Test Progress

| Metric | Before | After |
|--------|--------|-------|
| Compile-tested | 0 | 14 |
| Ignored | 31 | 19 |

---

## 🎯 Recommended Next Loops

### Loop 1: Error Handling Audit (8-10 iterations)
1. Audit `utils.rs` unwraps
2. Audit `compositor/` unwraps
3. Audit `input/` unwraps
4. Audit `framework/` unwraps
5. Audit `widgets/` unwraps
6. Audit `extensions/lsp-server/`
7. Replace with proper error types
8. Add error context/messages

### Loop 2: Test Coverage (6-8 iterations)
1. ~~Add `TagsInput` tests~~ — ✅ DONE (52 tests)
2. ~~Add `Calendar` tests~~ — ✅ DONE (56 tests)
3. ~~Add `Kanban` tests~~ — ✅ DONE (64 tests)
3. ~~Add `ColorPicker` tests~~ — ✅ DONE (54 tests)
4. ~~**Restore App::theme() builder method**~~ — ✅ DONE
4. Add `Autocomplete` tests
5. Add `RichText` tests
6. Add widget integration tests
7. Add snapshot tests

### Loop 3: Documentation (4-6 iterations)
1. Convert remaining 19 doc tests
2. Audit ctx.rs docs
3. Audit command.rs docs
4. Audit scene_router.rs docs
5. Add widget API docs
6. Update AGENTS.md

### Loop 4: Code Organization (8-10 iterations)
1. Split `utils.rs` extraction plan
2. Extract text utilities
3. Extract layout utilities
4. Extract color utilities
5. Clean up deprecated code
6. Refactor large widgets
7. Add SAFETY comments to unsafe blocks
8. Audit and clean up allow attributes

---

*Last updated: 2026-05-23*
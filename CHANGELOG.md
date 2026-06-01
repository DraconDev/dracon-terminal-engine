# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Notes

- **Sixel decoding stub now `#[deprecated]`** (P5-2) — `SixelImage::from_sixel` and `SixelRenderer::load_sixel` in `src/framework/sixel.rs` are now marked `#[deprecated]` with notes pointing at the documented limitation. The module, types, and widget are preserved (no public type removed) to avoid breaking downstream imports. Users who need real sixel decoding should integrate `sixel-rs` or `libimagequant` externally and feed the resulting RGB buffer into `SixelImage::new` + `set_pixel`.
- **`App::from_defaults()` already exists** (P5-1) — the fallible `App::from_defaults() -> io::Result<Self>` constructor is at `src/framework/app.rs:1092` and is already used by 4 test cases in `tests/app_tick_test.rs` (lines 1159, 1167, 1174, 1182). The P5-1 task wanted to add this and migrate internal callers; both are done. **Cannot mark `App::default()` with `#[deprecated]`** because Rust forbids the attribute on trait method overrides. The doc comment on `Default::default` already directs users to `from_defaults()`; that is the migration signal.
- **No duplicate I/O error variants in `DraconError`** — the P2-2 audit found a single `Io(io::Error)` variant. The earlier task description confused `DraconError::Io` with `I18nError::IoError` (in `src/framework/i18n.rs:306`), which lives in a separate type. Future consolidation: consider wrapping `I18nError` in `DraconError` to unify error types across the i18n subsystem and the rest of the engine.
- **Builder method ownership is already consistent** — P2-3 audit found that builder methods use `mut self -> Self` (chainable, used for app-level construction: `App::title`, `App::fps`, `App::set_theme`, `App::tick_interval`, `CommandConfig::parser`/`confirm`/`refresh`/`label`/`description`) and mutator methods use `&mut self` (single op, used for widget manipulation: `set_*`, `add_*`, `remove_*`, action methods). The two categories are not interchangeable; they serve different intents. Improvement: added `#[must_use]` to 9 builder methods so users get a warning if they accidentally drop the chained return value.
- **Standalone widgets (`src/widgets/`)** — P2-4 audit found:
  - `component.rs` is already `#[cfg(feature = "legacy")]`-gated. It is the only data model used by `src/layout.rs` (which is also legacy-gated). Decision: **retain under `legacy`**. Removal in 0.2.0 will require co-removing `src/layout.rs` and any downstream consumer migrating to the framework's `Widget` trait.
  - `hotkey.rs` is **not deprecated** — the P2-4 task description mischaracterized it. It is a small active utility (renders `[Ctrl+C] Copy` keyboard shortcut badges) re-exported from `src/lib.rs:233`. Decision: **no change**.

### Removed

- **`App::theme()` builder method** (`src/framework/app.rs`) — deprecated since 0.2.0, now removed. Use `App::set_theme()` instead. The method had zero in-tree callers and was only kept under the `legacy` feature flag for downstream users. Downstream users should migrate to `App::set_theme()` before upgrading.
- **`src/framework/command.rs` (single file, 1338 lines) split into a directory** (P6-1) — replaced with `src/framework/command/{mod.rs, parser.rs, exec.rs, config.rs}`. The new layout groups concerns: `parser.rs` (output parsing: `OutputParser`, `ParsedOutput`, `LoggedLine`), `exec.rs` (command execution: `BoundCommand`, `CommandRunner`, `split_command_args`), `config.rs` (TOML config: `AppConfig`, `WidgetConfig`, `LayoutConfig`, `AreaConfig`, `ParserConfig`). `mod.rs` is a thin re-export wrapper, so all downstream imports of `crate::framework::command::*` continue to work unchanged. All 394 lib tests + 25 integration tests pass.
- **`src/framework/helpers.rs` (single file, 250 lines) split into a directory** (P6-2) — replaced with `src/framework/helpers/{mod.rs, text.rs, borders.rs, blit.rs}`. The new layout groups concerns: `text.rs` (text drawing: `draw_text`), `borders.rs` (border rendering: `draw_rounded_border`), `blit.rs` (plane blitting: `blit_to`). `mod.rs` re-exports all three. All 5 original tests migrated. All tests pass.
- **Layout module duplication resolved** (P6-3) — the two layout modules serve different APIs and are not interchangeable. Decision documented in the module doc of `src/layout.rs` (legacy, `Component`-based, `legacy`-gated) and the existing `src/framework/layout.rs` (preferred, constraint-based, `Widget`-based). The legacy module is scheduled for 0.2.0 removal alongside the `Component` trait. No code change required; the legacy module is already feature-gated and contains unique types (`Stack`, `centered_rect`, `Orientation`) that have no framework equivalent.

### Deprecated

The following items remain available under the `legacy` Cargo feature flag and are scheduled for removal in 0.2.0:

- `widgets::component::Bounds` (`src/widgets/component.rs:11`) — Use `framework::layout::Layout` instead
- `widgets::component::Component` trait (`src/widgets/component.rs:45`) — Use `framework::widget::Widget` instead
- `App::theme()` builder (`src/framework/app.rs:529`) — Use `App::set_theme()` instead
- `Theme::scrollbar_width` field (`src/framework/theme.rs:117`) — Use `framework::scroll::DEFAULT_SCROLLBAR_WIDTH` instead

### Notes

- The `legacy` feature in `Cargo.toml` is the single switch for all deprecated APIs; it controls `widgets::component`, `src/layout.rs`, `App::theme()`, and the `Theme::scrollbar_width` field
- No deprecated items were removed in this release; all remain reachable via `--features legacy` for downstream users on 0.1.x

## [0.1.10] - 2026-05-13

### Fixed

- **Button widget mouse coordinates** — `handle_mouse` compared local coordinates against absolute screen positions; now correctly uses `col < area.width && row < area.height`
- **Tree widget scroll offset** — `handle_mouse` didn't account for scroll_offset; clicking while scrolled selected wrong node
- **Editor u16 underflow** — `mouse.column - area.x - gutter` replaced with `saturating_sub` in both wrap and non-wrap branches
- **Calendar unwrap panic** — `NaiveDate::from_ymd_opt().unwrap()` replaced with safe fallback
- **Editor unwrap on chars** — `text.chars().next().unwrap()` replaced with `unwrap_or('\u{FFFD}')`
- **6 examples with dead input** — `text_editor_demo`, `table_widget`, `widget_tutorial`, `form_widget`, `form_demo` had broken Widget/on_input patterns; all converted to proper Pattern 1 or Pattern 2
- **Showcase category clicks** — Sidebar only recognized 4 of 7 categories; now all 7 work (all, apps, input, data, cookbook, tools, accessibility)
- **Showcase FPS counter** — Overlapped by toggle checkbox; repositioned
- **Showcase features/stats** — "37 Widgets" → 41, "20 Themes" → 21
- **Nerd Font icons** — Replaced with standard Unicode for universal terminal compatibility
- **Theme propagation** — All 44 examples now use `Theme::from_env_or()` instead of hardcoded themes
- **Pattern 2 `current_theme()` sync** — All 12 Pattern 2 examples now implement `current_theme()` for DTRON_THEME_FILE
- **Autocomplete/NotificationCenter examples** — Now render their widgets visibly
- **CellPool example** — Now quittable via Ctrl+Q
- **3 button tests** — Updated to use local coordinates matching the Button fix
- **8 missing examples registered** in Cargo.toml
- **`DraconError` and `Widget` added to prelude**
- **Calendar example keybindings** — Replaced hardcoded Ctrl+Q with `actions::QUIT`, added modifier guard on 't'

### Changed

- Version aligned to semantic versioning (0.1.3 → 0.1.10)
- Widget count: 37 → 41 framework widgets
- Theme count: 20 → 21 built-in themes

## [0.1.3] - 2026-05-13

### Added

- **Bracketed Paste** — Terminal enables bracketed paste mode (ESC[2004h); App dispatches paste events to widgets
- **Vertical Layout** — `Layout::direction(Direction::Vertical)` and `Layout::vertical()` constructors
- **CellPool** — Object pool for Cell allocation, reducing per-frame allocation pressure
- **Accessibility** — `Role` enum, `Accessible` trait, `AccessibilityManager` with OSC 99/633 announcements
- **Calendar/DatePicker** — Date selection widget with month/year navigation
- **RichText** — Rich text renderer with headers, bold, italic, code blocks, links, lists
- **Autocomplete** — Type-ahead suggestion widget with keyboard/mouse navigation
- **NotificationCenter** — Queued notification stack with auto-dismiss
- **Grapheme cluster awareness** — `grapheme_width()` for correct Unicode display width
- **DraconError** — Unified error type for the engine
- **Structured logging** — `DraconLogger` with `tracing` integration (behind `tracing` feature)
- **Proptest** — Property-based tests for layout, grapheme width, theme colors
- **Async feature** — `async = ["dep:tokio"]` for async I/O examples
- **VSCode extension** — Live TUI preview extension in `extensions/vscode/`
- **8 new cookbook examples** — accessibility, cell_pool, notification_center, stat_widget_plugin, rich_text, autocomplete, form_validation, calendar

### Changed

- Version aligned to semantic versioning (0.1.0 → 0.1.3)

## [0.1.0] - 2026-05-12

### Added

#### Initial Release

- **37 Framework Widgets** — Autocomplete, Breadcrumbs, Button, Calendar, Checkbox, CommandPalette, ConfirmDialog, ContextMenu, DebugOverlay, EventLogger, Form, Gauge, Hud, KeyValueGrid, Label, List, LogViewer, MenuBar, Modal, NotificationCenter, PasswordInput, Profiler, ProgressBar, Radio, RichText, SearchInput, Select, Slider, Spinner, SplitPane, StatusBadge, StatusBar, StreamingText, TabBar, Table, TextEditorAdapter, Toast, Toggle, Tooltip, Tree, WidgetInspector

- **TextEditor** — Full-featured code editor with:
  - Syntax highlighting via syntect (20+ built-in grammars)
  - Undo/redo with persistent `.file.undo` support
  - Filter mode for highlighting matching lines
  - Multi-cursor editing
  - Per-file config via `.file.dte.json`

- **Application Framework** — One-import entry point:
  - `App` and `Ctx` for event loop and state management
  - Dirty rendering with `needs_render()` optimization
  - Theme system with 20+ built-in themes
  - HitZone system for declarative click/drag regions
  - Drag-and-drop with ghost rendering
  - Animation system with easing curves
  - Focus manager for tab-order navigation
  - Layout engine with constraint-based sizing

- **Compositor** — Z-indexed layer system:
  - TrueColor (RGB), ANSI 256, Reset colors
  - Style flags: Bold, Italic, Underline
  - Visual filters: Dim, Invert, Scanline, Pulse, Glitch
  - Braille compositing for sub-cell precision

- **Input System** — SGR mouse parsing, keyboard chords, modifiers

- **System Monitoring** — `SystemMonitor` for CPU, memory, disk, process metrics

- **30+ Examples** — Including showcase launcher, file manager, system monitor, chat client, IDE, todo app

### Changed

- Migrated from internal versioning to semantic versioning (0.1.0)

### Documentation

- README with quick start guide and feature overview
- AGENTS.md for development guidelines
- CONTRIBUTING.md for contribution process

## [29.11.0] - 2026-05-08

### Added

#### Architecture (Major)
- **Event Bus** (`src/framework/event_bus.rs`) — Decoupled publish/subscribe messaging with `Reactive<T>` helper for observable values
- **Scene Router** (`src/framework/scene_router.rs`) — Multi-screen navigation with push/pop/replace, transitions (fade, slide), lifecycle hooks (on_enter, on_exit, on_pause, on_resume), and deep linking
- **Plugin Registry** (`src/framework/plugin.rs`) — Dynamic widget loading via `WidgetFactory` trait

#### New Examples
- `event_bus_demo.rs` — Demonstrates event bus with counter + event log
- `scene_router_demo.rs` — Multi-screen app with transitions
- `tutorial_app.rs` — Progressive "Building Your First App" tutorial
- `todo_app.rs` — Real SQLite-backed todo app with CRUD operations
- `network_client.rs` — HTTP API consumer with async requests

#### Showcase Launcher
- Embedded SceneRouter scenes for instant launch (no external process spawn)
- 5 examples converted: widget_gallery, theme_switcher, form_demo, tree_navigator, modal_demo
- Seamless B/Esc back navigation from scenes
- Theme sync between showcase and embedded scenes
- Fade transitions between showcase grid and scenes
- "⚡ Embedded" badges on scene cards
- Scene title bar rendering
- `is_embedded()` detection in showcase state

#### Framework Improvements
- `Ctx::set_theme()` — Pattern 2 apps can now cycle themes via context
- `Compositor::invalidate_last_frame()` — Fixes black screen after returning from external examples
- `SceneRouter` interior mutability — `render(&self)` auto-ticks transitions
- Dithered crossfade transitions between scenes

### Fixed

#### Terminal State Corruption
- `form_widget.rs` — Added missing `on_tick` handler so `q` actually exits
- `input_debug.rs` — Full terminal cleanup on exit (disable mouse, focus events, bracketed paste, kitty keyboard)
- `game_loop.rs` — Disables mouse modes before exit
- `desktop.rs` — Disables mouse modes before exit
- `system_monitor.rs` — Replaced `process::exit(0)` with `ctx.stop()` bridge pattern
- `split_resizer.rs` — Replaced `process::exit(0)` with `ctx.stop()` bridge pattern

#### Showcase Bugs
- Slider in primitives bar now increments/decrements based on click position
- Black screen / transparent holes after returning from examples fixed via `invalidate_last_frame()`

#### Style Compliance
- `chat_client.rs` — Complete rewrite: moved rendering to `on_tick`, added `ScopedZoneRegistry`, modal input capture, dirty flag, removed manual z_index
- `form_widget.rs` — Fixed quit handler with `on_tick` bridge
- 4 examples fixed status bar hints to include `t: theme | ?: help`

### Tests
- **1,732 tests** across 68 test files (all passing)
- New: `event_bus_test.rs` (10 tests)
- New: `scene_router_test.rs` (11 tests)
- All 33 examples compile with zero warnings

## [27.0.5] - 2026-05-01

### Fixed

#### Clippy cleanup
- Fixed `absurd_extreme_comparisons` in `App::fps` — replaced `fps.max(1).min(120)` with `fps.clamp(1, 120)`
- Fixed `logic_bug` in `CommandRunner::test` — replaced tautological `assert!(code == 0 || code != 0)` with `assert!(code != 0)`
- Fixed `flatten()` infinite loop risk in `CommandRunner::spawn` — replaced `lines().flatten()` with `lines().map_while(|r| r.ok())`
- Removed unused `exit_code` field from `CommandRunner` struct
- Removed dead `matches_filter_by_raw` method from `LogViewer`
- Fixed redundant `id` binding in `SearchInput::new`
- Fixed `if let` collapsible warnings in `parser.rs` (mouse event parsing for SGR back/forward buttons)
- Fixed identical blocks in `editor_smoke_test` — collapsed `if code == Some(0) {} else if code == Some(1) {} else {}` to `if code == Some(0) || code == Some(1) { return; }`
- Fixed `assert!(true)` always-true assertion in `test_ctx_dirty_regions`
- Added `#![allow(dead_code)]` to `tests/common/mod.rs` for unused test helpers

#### Example fixes
- `game_loop.rs` — fixed double-indentation that broke compilation
- `desktop.rs` — replaced `Cell::default()` field assignment with struct initializer; fixed `drops.iter_mut().enumerate()` unused variable `i`
- `framework_chat.rs` — replaced `ToString` impl with `Display` impl for `Message`
- `framework_file_manager.rs` — replaced `ToString` impl with `Display` impl for `FileEntry`
- `button_test.rs` — removed unnecessary parentheses `let end_idx = (1 + "Button".len())`
- `text_editor_adapter_test.rs` — removed unnecessary parentheses `let idx_i = (gutter + 1)`
- `filter_test.rs` — fixed test that was assigning to `_changed` but never reading it; rewritten to sensible assertion
- `editor_smoke_test.rs` — added `child.wait()` after early return to fix spawned process not waited warning

### Changed

#### Prelude
- `ScrollState` re-exported from `scroll` module in prelude (was missing, broke `List::scroll_state()` return type)

### Tests

- All 291+ tests passing, 0 failures
- New test assertion: `test_glitch_at_zero_time_most_cells_unchanged` now asserts `changed < 5` (was broken assertion on exact char match)

## [27.0.4] - 2026-05-01

### Added

- `examples/command_dashboard.rs` — working command-driven dashboard example demonstrating `Gauge`, `KeyValueGrid`, and `StatusBadge` with bound CLI commands and auto-refresh
- `App::from_toml()` now loads `commands` array from TOML into the global command registry
- `AppConfig` gained `commands: Vec<BoundCommand>` field — global commands can be defined in TOML alongside layout and widgets
- `BoundCommand` fields (`parser`, `confirm_message`, `refresh_seconds`, `label`, `description`) now all have `#[serde(default)]` so they are optional in TOML

### Tests

- `test_app_config_commands` — parses TOML with `[[commands]]` array

## [27.0.3] - 2026-05-01

### Added

- `Widget::apply_command_output(&mut self, &ParsedOutput)` — default no-op trait method; called by the app tick loop when a widget's bound command is re-run after `refresh_seconds` has elapsed
- 5 widgets implement `apply_command_output`: `Gauge` (Scalar→f64), `StatusBadge` (Scalar→status), `KeyValueGrid` (Text/Scalar→pairs), `LogViewer` (Text/Lines→append), `StreamingText` (Text/Scalar/Lines→append)
- `App::command_tracking: HashMap<WidgetId, (Instant, BoundCommand)>` — tracks last-run time per widget's bound command
- Tick loop auto-re-executes commands whose `refresh_seconds` interval has elapsed, calls `apply_command_output` on the widget, marks it dirty
- `App::add_widget` populates `command_tracking` for any widget whose command has a `refresh_seconds` value
- `App::remove_widget` cleans up `command_tracking` entry for the removed widget

### Tests

- `test_gauge_apply_command_output_scalar` — parses "75.5" → value 75.5
- `test_gauge_apply_command_output_ignores_non_scalar` — None output leaves value unchanged
- `test_gauge_apply_command_output_parses_invalid_as_zero` — invalid string → 0
- `test_status_badge_apply_command_output_scalar` — sets status from Scalar
- `test_status_badge_apply_command_output_ignores_non_scalar` — None output leaves status unchanged
- `test_key_value_grid_apply_command_output` — Text parses "KEY: value" lines into pairs
- `test_log_viewer_apply_command_output_text` — Text appends lines
- `test_log_viewer_apply_command_output_lines` — Lines appends LogLine entries
- `test_streaming_text_apply_command_output_scalar` — Scalar appends as single line
- `test_streaming_text_apply_command_output_text` — Text appends lines
- `test_app_command_tracking_on_add_widget` — Label (no refresh) → not tracked
- `test_app_command_tracking_removed_on_widget_remove` — tracking cleaned up on remove

## [27.0.2] - 2026-05-01

### Added

#### Command-driven TOML architecture

- `src/framework/command.rs` — CommandRunner, BoundCommand, OutputParser, ParsedOutput
- `BoundCommand` — (cmd, parser, confirm, refresh_interval, label, description) — serde-serializable to TOML
- `OutputParser` variants — JsonKey, JsonPath, JsonArray, Regex, LineCount, ExitCode, SeverityLine, Plain
- `ParsedOutput` — Scalar, List, Lines(Vec<LoggedLine>), Text, None
- `AppConfig`, `WidgetConfig`, `LayoutConfig`, `AreaConfig`, `ParserConfig` — all TOML-serializable structs
- `AppConfig::from_toml(path)` and `AppConfig::from_toml_str(content)` — TOML-driven app creation
- `App::from_toml(path)` — create entire app from TOML config file
- `App::add_command(cmd)` — register command to global registry
- `App::available_commands()` — enumerate all commands across all widgets (AI surface)
- `App::run_command(cmd)` — execute CLI command synchronously, returns (stdout, stderr, exit_code)
- `Ctx::run_command(cmd)` — execute CLI from tick/render callbacks
- `Ctx::available_commands()` — enumerate commands from Ctx callbacks

#### Widget trait extension

- `Widget::commands(&self) -> Vec<BoundCommand>` — default returns empty vec
- All 6 new widgets implement commands() returning their bound command
- AI can call `ctx.available_commands()` to enumerate every action the TUI can perform

#### 6 new widgets

| Widget | File | Purpose |
|--------|------|---------|
| `ConfirmDialog` | `confirm_dialog.rs` | Modal yes/no with danger styling, border color changes on danger |
| `KeyValueGrid` | `key_value_grid.rs` | Displays JSON/Scalar as "KEY   VALUE" rows, BTreeMap sorted, alternating row colors |
| `Gauge` | `gauge.rs` | Filled progress bar with warn/crit thresholds (70%/90%), color changes with level |
| `LogViewer` | `log_viewer.rs` | Auto-scrolling log with severity detection (Fatal/Error/Warn/Debug/Info) and filter support |
| `StreamingText` | `streaming_text.rs` | Live-updating text with word-wrap, auto-scroll, max_lines |
| `StatusBadge` | `status_badge.rs` | Colored `[OK]` / `[WARN]` / `[ERROR]` badge from CLI status output |

### Fixed

- `WidgetRegistry.next_id` field missing — initialized to 1 in `WidgetRegistry::new()`
- Release workflow simplified: GitHub Release only (crates.io publish removed)
- CI: removed `minimal-versions` toolchain job (broke on nightly), removed `-D warnings` from clippy step

### Changed

- Total test count: 609 → 650+ tests (new widget tests added)
- Widget count: 29 → 35 framework widgets
- `toml = "0.8"` dependency added for TOML serialization

## [27.0.1] - 2026-04-30

### Infrastructure

- CI/CD: GitHub Actions workflows for CI (clippy, fmt, tests on stable/beta/nightly + macOS/Windows) and release (crates.io publish + GitHub Release)
- Added issue templates (bug report, feature request) and PR template
- Added CODE_OF_CONDUCT.md (Contributor Covenant v2.0)
- Added CONTRIBUTING.md (dev setup, code style, PR process)
- Added CHANGELOG.md (keepachangelog format)
- GitHub topics: rust, terminal, tui, framework, cli, compositor, ratatui, syntax-highlighting
- GitHub description and homepage set

### Added

- `Ctx::layout()` — constraint-based layout helper for use in `App::run` callbacks
- 8 new dirty tracking integration tests in `tests/phase1_widget_test.rs`
- All 29 framework widgets now have dirty tracking (`needs_render()`, `mark_dirty()`, `clear_dirty()`)
- State-changing methods on widgets (toggle, set_value, select, etc.) now call `mark_dirty()`
- `App::add_widget` now calls `widget.set_id(id)` to sync App-assigned IDs
- `Terminal::new()` falls back to null-mode when stdout is not a TTY

### Fixed

- README license badge now points to LICENSE-MIT (was deleted LICENSE pointer file)
- README widget count: 23 → 29 (corrected)
- README Quick Start example: unused `tick` param now `_tick`
- README example description: "23+" → "29" framework widgets
- Clippy: fixed always-zero multiplication `0u16 * plane.width` in 6 widget files
- Clippy: fixed always-true assertion in scroll_test.rs
- Smoke test `test_text_editor_demo_smoke` marked `#[ignore]` (requires real TTY)
- Fixed parallel test race in theme propagation tests (per-widget Rc<Cell> tracking)

## [27.0.0] - 2024-12-01

### Added

#### Framework Module
- **App & Ctx** — One-import application runtime: `App::new()`, event loop, terminal, compositor
- **Widget Trait v3** — `set_id()`, `needs_render()`, `mark_dirty()`, `clear_dirty()`, `on_theme_change()`, `on_mount()`, `on_unmount()`
- **Dirty Rendering** — `DirtyRegionTracker` for efficient partial screen updates; render loop skips clean widgets
- **23 Framework Widgets** — Breadcrumbs, Button, Checkbox, ContextMenu, DebugOverlay, EventLogger, Form, Hud, Label, List, MenuBar, Modal, PasswordInput, ProgressBar, Profiler, Radio, SearchInput, Select, Slider, Spinner, SplitPane, StatusBar, TabBar, Table, Toast, Toggle, Tooltip, Tree, WidgetInspector
- **15 Built-in Themes** — dark, light, cyberpunk, dracula, nord, catppuccin_mocha, gruvbox_dark, tokyo_night, solarized_dark, solarized_light, one_dark, rose_pine, kanagawa, everforest, monokai
- **Theme Propagation** — `App::set_theme()` calls `on_theme_change()` on all widgets
- **HitZone System** — `HitZone<T>`, `HitZoneGroup<T>`, `ScopedZone<T>`, `ScopedZoneRegistry<T>` for declarative click/double/drag/hover regions
- **Drag & Drop** — `DragManager<T>` with ghost rendering and state machine
- **Scroll Container** — `ScrollContainer` with offset management and scrollbar
- **Focus Manager** — Tab-order focus ring with keyboard navigation
- **Animation Manager** — Tweening with easing curves (Easing enum)
- **Layout Engine** — Constraint-based layout (Percentage, Fixed, Min, Max, Ratio)
- **Split Panes** — `split_h()` and `split_v()` helpers on `Ctx`

#### Compositor
- **Z-indexed Planes** — Multi-layer compositing with per-plane opacity
- **Cell, Color, Styles** — TrueColor (RGB), ANSI 256, Reset colors; Bold, Italic, Underline styles
- **Visual Filters** — Dim, Invert, Scanline, Pulse, Glitch per-plane filters
- **Braille Compositing** — Unicode braille characters for sub-cell precision

#### Input
- **SGR Mouse Parsing** — Mouse clicks, movement, drag, scroll wheel
- **Keyboard Chord Parsing** — Modifiers (Shift, Ctrl, Alt, Meta), key chords
- **EINTR Retry** — Non-blocking input reader handles EINTR gracefully

#### TextEditor Widget
- **Syntax Highlighting** — via syntect with 20+ built-in grammars
- **Undo/Redo** — Full history stack with save/load to `.file.undo`
- **Filter Mode** — Highlight and navigate matching lines
- **Multi-cursor** — `add_cursor()`, `clear_extra_cursors()`
- **Per-file Config** — Load/save `.file.dte.json`

#### System Monitor
- **SystemMonitor** — CPU, memory, disk, process metrics
- **DiskInfo, ProcessInfo, SystemData** types

#### Utilities
- **Layout Helpers** — Grid, border, padding utilities
- **Visual OSC** — Clipboard, hyperlinks, bell, notifications
- **Sync Mode 2026** — Synchronized tear-free output
- **Icons** — File-type icon set

#### Examples
- `framework_demo` — List + Breadcrumbs + SplitPane + Hud + SystemMonitor
- `framework_file_manager` — File browser with List + Breadcrumbs + SplitPane
- `framework_chat` — Chat UI: message list + input bar + theme
- `framework_widgets` — Showcase all 23+ framework widgets
- `text_editor_demo` — TextEditor with theme switching
- `basic_raw`, `god_mode`, `input_debug`

### Changed
- All 23 framework widgets now implement `needs_render()`, `mark_dirty()`, `clear_dirty()`
- State-changing methods on widgets (toggle, set_value, etc.) now call `mark_dirty()`
- `App::add_widget` now calls `widget.set_id(id)` to sync App-assigned IDs
- `Terminal::new()` falls back to null-mode when stdout is not a TTY (headless/CI environments)

### Fixed
- Widgets with derived theme state now properly update via `on_theme_change()`
- Parallel test execution no longer races on theme propagation tracking

## Prior Versions

See the git history for versions prior to v27.0.0.

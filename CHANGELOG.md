# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

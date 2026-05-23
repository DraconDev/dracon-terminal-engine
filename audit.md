# Dracon Terminal Engine — Project Audit

**Created:** 2026-05-23  
**Version:** 0.1.10  
**Total LOC:** ~39,000  
**Framework widgets:** 41  
**Themes:** 21  
**Examples:** 57

---

## 📦 Project Overview

**Dracon Terminal Engine** is a terminal application framework for Rust. Not a TUI library — a complete runtime that owns the terminal, input, rendering, and event loop.

### Key Characteristics
- **Mouse-friendly** with z-indexed planes
- **41 built-in widgets** (List, Table, Tree, Form, etc.)
- **21 themes** (Nord, Dracula, Catppuccin, etc.)
- **Dirty rendering** for efficient updates
- **Focus management** system
- **Command-driven architecture** — AI can enumerate and trigger all actions
- **Single binary deployment** — no external runtime

### Dependencies
- `ratatui 0.30` — Terminal rendering backend
- `unicode-width` — Character width calculation
- `chrono` — Date/time (with serde)
- `signal-hook` — Signal handling
- `serde` / `serde_json` — Serialization
- `toml` — Configuration
- Optional: `sysinfo`, `syntect`, `tokio`, `reqwest`, `rusqlite`, `tracing`

---

## 🏗️ Architecture

### Core Modules

| Module | Path | Description |
|--------|------|-------------|
| **compositor** | `src/compositor/` | Plane pool, filtering, rendering engine |
| **input** | `src/input/` | Event parsing, keyboard/mouse handling |
| **integration** | `src/integration/` | ratatui integration |
| **widgets** | `src/widgets/` | Standalone widgets (TextEditor, etc.) |
| **framework** | `src/framework/` | App framework, widgets, utilities |
| **backend** | `src/backend/` | TTY backend, terminal operations |
| **visuals** | `src/visuals/` | Icons, accessibility, OSC commands |
| **core** | `src/core/` | Terminal core |
| **system** | `src/system.rs` | System metrics (CPU, memory, etc.) |
| **text** | `src/text.rs` | Text utilities (387 LOC) |
| **layout** | `src/layout.rs` | Layout engine (145 LOC) |
| **error** | `src/error.rs` | Error types |
| **utils** | `src/utils.rs` | Utilities (1,217 LOC) |

### Large Files (>500 LOC)

| File | LOC | Status |
|------|-----|--------|
| `src/widgets/editor.rs` | 3,025 | ⚠️ Large monolith |
| `src/utils.rs` | 1,217 | ⚠️ Catch-all utilities |
| `src/framework/widgets/tree.rs` | 1,817 | OK |
| `src/framework/app.rs` | 1,575 | OK |
| `src/framework/event_bus.rs` | 1,200+ | OK |
| `src/framework/widgets/kanban.rs` | ~1,100 | OK |
| `src/framework/widgets/table.rs` | ~1,000 | OK |
| `src/framework/widgets/form.rs` | ~900 | OK |
| `src/visuals/accessibility.rs` | 416 | OK |
| `src/visuals/icons.rs` | 412 | OK |
| `src/text.rs` | 387 | OK |
| `src/system.rs` | 288 | OK |
| `src/widgets/editor_search.rs` | 293 | OK |

---

## 🎛️ Framework Systems

### 41 Framework Widgets

| Category | Widgets |
|----------|---------|
| **Navigation** | Breadcrumbs, TabBar, Tree, SplitPane |
| **Input** | Button, Checkbox, Radio, Toggle, Slider, SearchInput, PasswordInput, TagsInput, Autocomplete |
| **Display** | Label, ProgressBar, ProgressRing, Sparkline, Spinner, Gauge, StatusBadge, LogViewer, StreamingText |
| **Layout** | Divider, Tooltip, Modal, ConfirmDialog |
| **Containers** | List, Table, Form, Kanban, ContextMenu, MenuBar, Toast, CommandPalette |
| **Specialized** | TextEditorAdapter, WidgetInspector, Hud, Profiler, DebugOverlay, EventLogger, ColorPicker, KeyValueGrid |
| **Status** | StatusBar |

### Framework Utilities

| Module | What |
|--------|------|
| `DirtyRegionTracker` | Efficient partial screen updates |
| `AnimationManager` | Tweening animations with easing curves |
| `Layout` | Constraint-based layout engine |
| `Theme` | 21 built-in themes |
| `SceneRouter` | Multi-screen navigation with push/pop |
| `EventBus` | Publish/subscribe event system |
| `Keybindings` | Configurable keybinding system |
| `I18n` | Internationalization |
| `MarqueeState` | Drag selection rectangle |
| `DragDrop` | Drag-and-drop system |
| `HitZone` | Click detection system |

### Widgets in `src/widgets/` (Standalone)

| Widget | Description |
|--------|-------------|
| `editor.rs` | TextEditor (3,025 LOC) — view/edit widget |
| `editor_search.rs` | Search state for TextEditor |
| `panel.rs` | Panel container |
| `input.rs` | Text input base |
| `button.rs` | Button widget |
| `component.rs` | Component trait |
| `hotkey.rs` | Hotkey display |
| `context_menu.rs` | Context menu |

---

## 📊 Examples (57 total)

### Showcase & Gallery
- `showcase/` — Interactive showcase launcher

### Full Applications
| Example | Description |
|---------|-------------|
| `ide.rs` | IDE-style editor (57KB) |
| `git_tui.rs` | Git TUI (43KB) |
| `scene_router_demo.rs` | Multi-screen navigation |
| `todo_app.rs` | Todo application |
| `form_demo.rs` | Form demo |
| `table_widget.rs` | Table widget demo |
| `chat_client.rs` | Chat application |
| `sqlite_browser.rs` | SQLite browser |
| `network_client.rs` | Network client |

### Framework Demos
| Example | Description |
|---------|-------------|
| `framework_demo.rs` | Basic framework usage |
| `framework_widgets.rs` | Widget gallery |
| `framework_file_manager.rs` | File manager |
| `framework_chat.rs` | Chat UI |
| `plugin_demo.rs` | Plugin architecture |
| `event_bus_demo.rs` | Event bus system |

### Raw Terminal Demos
| Example | Description |
|---------|-------------|
| `desktop.rs` | Desktop environment |
| `game_loop.rs` | Game loop demo |
| `input_debug.rs` | Input debugging |
| `arena.rs` | Game arena (42KB) |

### Other Examples
| Example | Description |
|---------|-------------|
| `command_dashboard.rs` | Command palette |
| `cyberpunk_dashboard.rs` | Cyberpunk theme |
| `modal_demo.rs` | Modal dialogs |
| `theme_switcher.rs` | Theme cycling |
| `text_editor_demo.rs` | Text editor |
| `rich_text_demo.rs` | Rich text |
| `tutorial_app.rs` | Tutorial |
| `widget_tutorial.rs` | Widget tutorial |
| `basic_raw.rs` | Raw terminal basics |
| `god_mode.rs` | God mode demo |
| `from_toml.rs` | TOML config |

---

## 🔴 HIGH PRIORITY TASKS

### Security

- [ ] Monitor transitive unmaintained dependencies:
  - `bincode 1.3.3` — RUSTSEC-2025-0141
  - `yaml-rust 0.4.5` — RUSTSEC-2024-0320
- [ ] Schedule quarterly `cargo outdated` review
- [ ] Add `cargo audit` to CI if not present

### Production Unwraps

- [ ] Audit ~50 `unwrap()`/`expect()` calls in `src/`
- [ ] Audit 22 unwraps in `extensions/lsp-server/`
- [ ] Replace with proper error propagation

### Unsafe Blocks

- [ ] Add `// SAFETY:` comments to all unsafe blocks:
  - `src/compositor/plane.rs` — 5 blocks
  - `src/backend/tty.rs` — 4 blocks
  - `examples/` — multiple blocks

---

## 🟡 MEDIUM PRIORITY TASKS

### Code Organization

- [ ] Split `utils.rs` (1,217 LOC):
  - Extract `visual_width`, `truncate`, `formatting` → `src/text.rs`
  - Extract `clamp`, `bounding_box` → `src/layout.rs`
  - Extract color utils → `src/visuals/` or `theme.rs`
- [ ] Consider extracting `src/framework/prelude.rs` into standalone file
- [ ] Remove `src/input/mapping.rs` (deprecated identity functions)

### Test Coverage Gaps

- [ ] `text_input_base_test.rs` — 26 unit tests, needs integration tests:
  - Tab between fields
  - Focus styling
  - Scroll behavior
  - PasswordInput mask/unmask toggle
- [ ] `cargo-dracon` scaffolding tool — add template generation tests
- [ ] Event bus micro-benchmarks (criterion)

### Documentation

- [ ] Add SceneRouter compile-tested doc example
- [ ] Convert remaining 19 ignored doc tests to compile-tested
- [ ] Add `// SAFETY:` preamble for every `unsafe` block

### Build Optimization

- [ ] Profile `debug` build time
- [ ] Add `lto = "thin"` for release builds
- [ ] Evaluate `codegen-units = 1` tradeoff
- [ ] Check if `bitflags::serde` feature is needed

### Configuration

- [ ] Add TOML schema validation for `dracon.toml`
- [ ] Add unit tests for `KeybindingConfig::parse_keybinding()` edge cases
- [ ] Enforce `keepachangelog.com` format in CI

---

## 🟢 LOW PRIORITY TASKS

### Ideas (Further Investigation)

- [ ] Panic safety audit — search for index arithmetic panics
- [ ] Thread safety documentation — framework is single-threaded by design
- [ ] Plugin architecture evaluation — `PluginRegistry` exists but underused
- [ ] Tracing feature performance check when disabled
- [ ] macOS/Windows testing — `libc` gated, no macOS test coverage
- [ ] Snapshot tests with `insta` — no snapshot files visible

### Nice-to-Have

- [ ] Add `cargo upgrade` to maintenance workflow
- [ ] Dev-deps updates: `criterion 0.5.1` → `0.8.2`, `itertools 0.10` → `0.13`
- [ ] Add `[Unreleased]` section to CHANGELOG.md
- [ ] Add widget tree inspector to showcase

---

## 📈 Progress Tracking

### Completed (from TODO.md work)

| Item | Status |
|------|--------|
| lru unsoundness fix (ratatui 0.30) | ✅ DONE |
| CI pipeline (outdated + changelog) | ✅ DONE |
| Security advisories updated | ✅ DONE |
| editor.rs split documented as impractical | ✅ DONE |
| App::new().unwrap() docs fixed | ✅ DONE |
| Test coverage gaps (progress_ring, sparkline, list_common) | ✅ DONE |
| size_test.rs moved to tests/ | ✅ DONE |
| set_theme doc comment added | ✅ DONE |
| 14 compile-tested doc examples added | ✅ DONE |

### Doc Test Progress

| Iteration | Compile-Tested | Ignored |
|----------|---------------|---------|
| Start | 0 | 31 |
| Current | 14 | 19 |

---

## 📁 Directory Structure

```
dracon-terminal-engine/
├── src/
│   ├── compositor/      # Plane pool, rendering
│   ├── input/           # Event parsing
│   ├── integration/     # ratatui integration
│   ├── widgets/         # Standalone widgets
│   ├── framework/       # App framework + widgets
│   │   └── widgets/     # 41 framework widgets
│   ├── backend/         # TTY backend
│   ├── visuals/         # Icons, accessibility
│   ├── core/            # Terminal core
│   ├── lib.rs          # Main entry
│   ├── utils.rs        # Utilities (1,217 LOC)
│   └── ...
├── examples/           # 57 examples
├── tests/             # Integration tests
├── crates/            # Cargo sub-commands
├── extensions/        # LSP server etc.
├── benches/           # Criterion benchmarks
├── .github/workflows/ # CI/CD
├── CHANGELOG.md
├── Cargo.toml
└── README.md
```

---

## 🔗 Links

- Repository: https://github.com/DraconDev/dracon-terminal-engine
- Documentation: https://docs.rs/dracon-terminal-engine
- AGENTS.md: Detailed agent instructions for working with this codebase
- AGENTS.md: Architecture principles, widget patterns, example patterns

---

*Last updated: 2026-05-23*
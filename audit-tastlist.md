# Dracon Terminal Engine — Project Audit Checklist

## 1. Project Structure

```
dracon-terminal-engine/
├── src/                          # Main library source
├── examples/                      # Example applications
├── tests/                        # Integration and unit tests
├── crates/                       # Supporting crates
│   ├── cargo-dracon/            # Cargo plugin
│   └── dracon-macros/           # Procedural macros
├── extensions/                   # LSP server & VSCode extension
├── Cargo.toml                   # Root manifest (v0.1.10)
├── dracon.toml                  # User configuration
├── AGENTS.md                    # Agent instructions
├── CHANGELOG.md                 # Version history
└── README.md                    # Project overview
```

- [ ] Verify all top-level directories exist
- [ ] Verify `Cargo.toml`, `dracon.toml`, `AGENTS.md`, `CHANGELOG.md`, `README.md` exist
- [ ] Verify `crates/cargo-dracon/` and `crates/dracon-macros/` subdirs
- [ ] Verify `extensions/lsp-server/` and `extensions/vscode/`
- [ ] Verify `.cargo/config.toml` and `rustfmt.toml`

---

## 2. Main Source Layout (`src/`)

| Directory/File | Purpose | Audit |
|---|---|---|
| `lib.rs` | Library root — exports all modules and prelude | [ ] Compiles |
| `backend/` | POSIX tty ioctls, raw mode setup | [ ] Builds |
| `compositor/` | Z-indexed layer compositor (Plane, Cell, Color, Styles, filters) | [ ] Builds |
| `core/` | Terminal wrapper (RAII raw mode + alt screen) | [ ] Builds |
| `framework/` | App, widgets, themes, HitZone, ScrollContainer — **main API** | [ ] Builds |
| `framework/widgets/` | 41 framework widgets | [ ] All build |
| `input/` | InputReader + SGR mouse/chord parser | [ ] Builds |
| `integration/` | Ratatui integration bridge | [ ] Builds |
| `layout.rs` | Layout helpers (grid, border, padding) | [ ] Builds |
| `system.rs` | System monitoring (CPU, memory, disk, processes) — feature-gated | [ ] Builds |
| `text.rs` | Unicode text handling utilities | [ ] Builds |
| `utils.rs` | General utilities (visual width, truncate, formatting) | [ ] Builds |
| `visuals/` | Icons, OSC strings (clipboard, hyperlink, bell) | [ ] Builds |
| `widgets/` | Standalone widgets (TextEditor, TextInput, Button, Panel, etc.) | [ ] Builds |
| `error.rs` | Unified error types | [ ] Builds |
| `contracts.rs` | UiRenderer, UiRuntime traits | [ ] Builds |

---

## 3. Core Framework (`src/framework/`)

| File | Purpose | Audit |
|---|---|---|
| `app.rs` (59KB) | Main App struct, event loop, widget registry | [ ] Builds |
| `command.rs` (39KB) | Command system, parsers, runners | [ ] Builds |
| `theme.rs` (61KB) | 21 built-in themes | [ ] Builds |
| `ctx.rs` | Ctx — context passed to callbacks | [ ] Builds |
| `widget.rs` | Widget trait definition | [ ] Builds |
| `widget_container.rs` | Widget container utilities | [ ] Builds |
| `event_bus.rs` | Pub/sub event system | [ ] Builds |
| `event_dispatcher.rs` | Event dispatching | [ ] Builds |
| `focus.rs` | Tab-order focus ring | [ ] Builds |
| `keybindings.rs` | Keybinding configuration | [ ] Builds |
| `scene_router.rs` (26KB) | Scene navigation controller | [ ] Builds |
| `layout.rs` | Constraint-based layout engine | [ ] Builds |
| `animation.rs` | Animation system | [ ] Builds |
| `dragdrop.rs` | Drag-and-drop system | [ ] Builds |
| `hitzone.rs` | Hit zone system | [ ] Builds |
| `marquee.rs` | Marquee selection | [ ] Builds |
| `scroll.rs` | Scroll container | [ ] Builds |
| `dirty_regions.rs` | Dirty region tracking | [ ] Builds |
| `i18n.rs` | Internationalization | [ ] Builds |
| `logging.rs` | Tracing/logging (feature-gated) | [ ] Builds |
| `sixel.rs` | Sixel image support (feature-gated) | [ ] Builds |
| `plugin.rs` | Plugin registry | [ ] Builds |
| `helpers.rs` | Shared rendering helpers | [ ] Builds |
| `prelude.rs` | Common re-exports | [ ] Builds |

---

## 4. Framework Widgets (`src/framework/widgets/`) — 41 widgets

| Widget | File | Audit | Widget | File | Audit |
|---|---|---|---|---|---|
| Autocomplete | `autocomplete.rs` | [ ] | MenuBar | `menu_bar.rs` | [ ] |
| Breadcrumbs | `breadcrumbs.rs` | [ ] | Modal | `modal.rs` | [ ] |
| Button | `button.rs` | [ ] | NotificationCenter | `notification_center.rs` | [ ] |
| Calendar | `calendar.rs` | [ ] | PasswordInput | `password_input.rs` | [ ] |
| Checkbox | `checkbox.rs` | [ ] | Profiler | `profiler.rs` | [ ] |
| CommandPalette | `command_palette.rs` | [ ] | ProgressBar | `progress_bar.rs` | [ ] |
| ConfirmDialog | `confirm_dialog.rs` | [ ] | ProgressRing | `progress_ring.rs` | [ ] |
| ContextMenu | `context_menu.rs` | [ ] | Radio | `radio.rs` | [ ] |
| DebugOverlay | `debug_overlay.rs` | [ ] | RichText | `rich_text.rs` | [ ] |
| EventLogger | `event_logger.rs` | [ ] | SearchInput | `search_input.rs` | [ ] |
| Form | `form.rs` | [ ] | Select | `select.rs` | [ ] |
| Gauge | `gauge.rs` | [ ] | Slider | `slider.rs` | [ ] |
| Hud | `hud.rs` | [ ] | Spinner | `spinner.rs` | [ ] |
| KeyValueGrid | `key_value_grid.rs` | [ ] | SplitPane | `split.rs` | [ ] |
| Label | `label.rs` | [ ] | StatusBadge | `status_badge.rs` | [ ] |
| List | `list.rs` | [ ] | StatusBar | `status_bar.rs` | [ ] |
| ListCommon | `list_common.rs` | [ ] | StreamingText | `streaming_text.rs` | [ ] |
| LogViewer | `log_viewer.rs` | [ ] | TabBar | `tabbar.rs` | [ ] |
| Table | `table.rs` | [ ] | TagsInput | `tags_input.rs` | [ ] |
| TextEditorAdapter | `text_editor_adapter.rs` | [ ] | TextInputBase | `text_input_base.rs` | [ ] |
| Toast | `toast.rs` | [ ] | Toggle | `toggle.rs` | [ ] |
| Tooltip | `tooltip.rs` | [ ] | Tree | `tree.rs` | [ ] |
| WidgetInspector | `widget_inspector.rs` | [ ] | | | |

---

## 5. Standalone Widgets (`src/widgets/`)

| Widget | File | Audit |
|---|---|---|
| TextEditor | `editor.rs` (119KB) | [ ] Builds |
| EditorSearch | `editor_search.rs` | [ ] Builds |
| TextInput | `input.rs` | [ ] Builds |
| Button | `button.rs` | [ ] Builds |
| Panel | `panel.rs` | [ ] Builds |
| HotkeyHint | `hotkey.rs` | [ ] Builds |
| ContextMenuAction | `context_menu.rs` | [ ] Builds |
| Component | `component.rs` | [ ] Deprecated/removed |

---

## 6. Compositor (`src/compositor/`)

| File | Purpose | Audit |
|---|---|---|
| `engine.rs` (28KB) | Main compositor engine | [ ] Builds |
| `plane.rs` (17KB) | Plane — drawable surface | [ ] Builds |
| `pool.rs` | Cell pool for memory management | [ ] Builds |
| `filter.rs` | Compositor filters | [ ] Builds |

---

## 7. Input System (`src/input/`)

| File | Purpose | Audit |
|---|---|---|
| `parser.rs` (25KB) | SGR mouse / chord parser | [ ] Builds |
| `event.rs` | Event types (KeyEvent, MouseEvent) | [ ] Builds |
| `reader.rs` | Input reader | [ ] Builds |
| `async_reader.rs` | Async input reader | [ ] Builds |
| `mapping.rs` | Key mapping | [ ] Builds |
| `kitty_key.rs` | Kitty keyboard protocol | [ ] Builds |

---

## 8. Core/Backend (`src/core/`, `src/backend/`)

| File | Purpose | Audit |
|---|---|---|
| `terminal.rs` | Terminal wrapper (Capabilities, CursorShape, Terminal) | [ ] Builds |
| `backend/tty.rs` | POSIX tty ioctls, raw mode setup | [ ] Builds |

---

## 9. Core Systems Audit

| System | Location | Audit |
|---|---|---|
| HitZone System | `src/framework/hitzone.rs` | [ ] `HitZone<T>`, `HitZoneGroup<T>`, `ScopedZone<T>`, `ScopedZoneRegistry<T>` all build and are used by examples |
| Drag-and-Drop | `src/framework/dragdrop.rs` | [ ] `DragManager<T>`, `DragItem<T>`, `DragGhost`, `DropTarget<T>` all build |
| Animation | `src/framework/animation.rs` | [ ] `Animation`, `AnimationManager`, `Easing` all build |
| Marquee Selection | `src/framework/marquee.rs` | [ ] `MarqueeState`, `MarqueeRect`, `render_marquee` all build |
| Event Bus | `src/framework/event_bus.rs` | [ ] `EventBus`, `Reactive<T>`, `SubscriptionId` all build |
| Focus Manager | `src/framework/focus.rs` | [ ] Tab-order focus ring works |
| Scene Router | `src/framework/scene_router.rs` | [ ] `Scene`, `SceneRouter` navigate correctly |
| Layout Engine | `src/framework/layout.rs` | [ ] `Constraint`, `Direction`, `Layout` work |
| Scroll Container | `src/framework/scroll.rs` | [ ] `ScrollContainer`, `ScrollState` work |
| Dirty Regions | `src/framework/dirty_regions.rs` | [ ] `DirtyRegion`, `DirtyRegionTracker` work |
| Plugin System | `src/framework/plugin.rs` | [ ] `PluginRegistry`, `WidgetFactory` work |
| Keybindings | `src/framework/keybindings.rs` | [ ] `KeybindingSet`, `KeybindingConfig`, `actions` resolve correctly |
| TextEditor | `src/widgets/editor.rs` | [ ] Full editor with search, undo, syntax highlighting |
| Theme System | `src/framework/theme.rs` | [ ] All 21 themes render correctly |

---

## 10. Feature Flags

| Flag | Purpose | Default | Audit |
|---|---|---|---|
| `debug_events` | Debug event logging | off | [ ] Works when enabled |
| `async` | Async runtime (tokio + reqwest) | off | [ ] Builds with feature |
| `tracing` | Tracing/structured logging | off | [ ] Builds with feature |
| `system` | System monitoring (sysinfo) | **on** | [ ] Builds with feature |
| `syntax-highlighting` | Syntax highlighting (syntect) | **on** | [ ] Builds with feature |
| `sixel` | Sixel image support | off | [ ] Builds with feature |
| `sqlite` | SQLite support (rusqlite) | off | [ ] Builds with feature |
| `default` | `system` + `syntax-highlighting` | — | [ ] Default build works |

---

## 11. Example Applications

### 11.1 Full Examples (`examples/`)

| Example | Path | Audit |
|---|---|---|
| `desktop` | `examples/desktop.rs` | [ ] Builds and runs |
| `git_tui` | `examples/git_tui.rs` | [ ] Builds and runs |
| `ide` | `examples/ide.rs` | [ ] Builds and runs |
| `sqlite_browser` | `examples/sqlite_browser.rs` | [ ] Builds and runs |
| `text_editor_demo` | `examples/text_editor_demo.rs` | [ ] Builds and runs |
| `form_demo` | `examples/form_demo.rs` | [ ] Builds and runs |
| `theme_switcher` | `examples/theme_switcher.rs` | [ ] Builds and runs |
| `game_loop` | `examples/game_loop.rs` | [ ] Builds and runs |
| `input_debug` | `examples/input_debug.rs` | [ ] Builds and runs |
| `modal_demo` | `examples/modal_demo.rs` | [ ] Builds and runs |
| `widget_tutorial` | `examples/widget_tutorial.rs` | [ ] Builds and runs |
| `framework_demo` | `examples/framework_demo.rs` | [ ] Builds and runs |
| `framework_file_manager` | `examples/framework_file_manager.rs` | [ ] Builds and runs |
| `event_bus_demo` | `examples/event_bus_demo.rs` | [ ] Builds and runs |
| `scene_router_demo` | `examples/scene_router_demo.rs` | [ ] Builds and runs |
| `network_client` | `examples/network_client.rs` | [ ] Builds and runs |
| `plugin_demo` | `examples/plugin_demo.rs` | [ ] Builds and runs |
| `todo_app` | `examples/todo_app.rs` | [ ] Builds and runs (requires `sqlite`) |
| `tutorial_app` | `examples/tutorial_app.rs` | [ ] Builds and runs |
| `form_widget` | `examples/form_widget.rs` | [ ] Builds and runs |
| `table_widget` | `examples/table_widget.rs` | [ ] Builds and runs |

### 11.2 Apps (`examples/_apps/`)

| Example | Purpose | Audit |
|---|---|---|
| `system_monitor` | Real `/proc` data (CPU, memory, disk, processes) | [ ] Builds and runs |
| `file_manager` | File manager with split pane, breadcrumb navigation | [ ] Builds and runs |
| `chat_client` | Chat client UI | [ ] Builds and runs |
| `dashboard_builder` | Dashboard builder | [ ] Builds and runs |

### 11.3 Cookbook (`examples/_cookbook/`)

| Example | Audit |
|---|---|
| `widget_gallery` | [ ] |
| `tree_navigator` | [ ] |
| `log_monitor` | [ ] |
| `tabbed_panels` | [ ] |
| `data_table` | [ ] |
| `split_resizer` | [ ] |
| `command_bindings` | [ ] |
| `menu_system` | [ ] |
| `debug_overlay` | [ ] |
| `rich_text` | [ ] |
| `calendar` | [ ] |
| `autocomplete` | [ ] |
| `form_validation` | [ ] |
| `scrollable_content` | [ ] |
| `accessibility` | [ ] |
| `cell_pool` | [ ] |
| `notification_center` | [ ] |
| `stat_widget_plugin` | [ ] |

### 11.4 Showcase (`examples/showcase/`)

| File | Audit |
|---|---|
| `main.rs` | [ ] Builds and runs |
| `data.rs` | [ ] All examples registered |
| `state.rs` | [ ] State management works |
| `render.rs` | [ ] Card rendering correct |
| `widget.rs` | [ ] Widget implementation works |
| `scenes/` (all embedded scenes) | [ ] All scenes build and run |

### 11.5 Plugins (`examples/_plugins/`)

| File | Audit |
|---|---|
| `lib.rs` | [ ] Plugin interface builds |
| `stat_widget.rs` | [ ] Stat widget builds |
| `welcome_widget.rs` | [ ] Welcome widget builds |

---

## 12. Testing Infrastructure (`tests/`)

| Category | Files | Audit |
|---|---|---|
| Widget Tests | `widget_*.rs` (50+ files) | [ ] All pass |
| Framework Tests | `framework_*.rs` | [ ] All pass |
| App Tests | `app_*.rs`, `app_tick_test.rs` | [ ] All pass |
| Integration Tests | `*_integration_test.rs` | [ ] All pass |
| Smoke Tests | `example_smoke_test.rs`, `showcase_smoke_test.rs` | [ ] All pass |
| Snapshot Tests | `tests/snapshots/` | [ ] All pass |
| Performance Benchmarks | `performance_benchmarks.rs`, `framework_benchmarks.rs` | [ ] All pass |
| Theme Tests | `theme_test.rs`, `theme_propagation_test.rs` | [ ] All pass |
| Focus Tests | `focus_test.rs` | [ ] All pass |
| Event Tests | `event_handler_test.rs`, `event_bus_test.rs` | [ ] All pass |
| Scene Tests | `scene_router_test.rs` | [ ] All pass |
| Editor Tests | `editor_smoke_test.rs` | [ ] All pass |
| Property Tests | `property_tests.rs` | [ ] All pass |
| Accessibility Tests | `accessibility_test.rs` | [ ] All pass |

---

## 13. Supporting Crates (`crates/`)

| Crate | Audit |
|---|---|
| `cargo-dracon/` | [ ] Builds |
| `dracon-macros/` | [ ] Builds |

---

## 14. Extensions (`extensions/`)

| Extension | Audit |
|---|---|
| `lsp-server/` | [ ] Builds |
| `vscode/` | [ ] Builds |
| `_plugins/` | [ ] Builds |

---

## 15. Configuration Files

| File | Audit |
|---|---|
| `Cargo.toml` | [ ] Valid manifest, correct version, all features/dependencies |
| `dracon.toml` | [ ] Valid user configuration |
| `.cargo/config.toml` | [ ] Valid Cargo config |
| `rustfmt.toml` | [ ] Valid Rust formatting config |
| `examples/from_toml.toml` | [ ] Valid TOML example |

---

## 16. Key Dependencies

| Dependency | Feature Gate | Audit |
|---|---|---|
| ratatui | — | [ ] Compatible version |
| unicode-width | — | [ ] Used correctly |
| chrono | — | [ ] Used for dates |
| signal-hook | — | [ ] Signal handling works |
| serde/serde_json/toml | — | [ ] Serialization works |
| sysinfo | `system` | [ ] System monitoring works |
| syntect | `syntax-highlighting` | [ ] Syntax highlighting works |
| tokio | `async` | [ ] Async runtime works |
| reqwest | `async` | [ ] HTTP client works |
| rusqlite | `sqlite` | [ ] SQLite works |
| tracing | `tracing` | [ ] Tracing works |
| regex | — | [ ] Regex used correctly |

---

## 17. Documentation

- [ ] `README.md` is up to date
- [ ] `CHANGELOG.md` reflects recent changes
- [ ] `AGENTS.md` is current and complete
- [ ] All public APIs have doc comments
- [ ] Any `unsafe` blocks are documented

---

## 18. Code Quality

- [ ] `cargo check` passes with no warnings
- [ ] `cargo clippy` passes with no warnings (or known acceptable warnings)
- [ ] `cargo fmt` produces no changes
- [ ] No hardcoded secrets or keys in source
- [ ] All errors are handled appropriately (no `.unwrap()` on user input)
- [ ] No obvious memory leaks (Rc/RefCell cycles, unbounded allocations)

---

## 19. Platform-Specific

- [ ] Builds on Linux (primary target)
- [ ] POSIX tty code (`src/backend/tty.rs`) reviewed for correctness
- [ ] Signal handling works (`signal-hook` integration)
- [ ] Raw mode / alt screen entry/exit is RAII-safe

---

## 20. Release Checklist

- [ ] Version bumped in `Cargo.toml`
- [ ] `CHANGELOG.md` updated with version and date
- [ ] All examples build with `cargo build --examples`
- [ ] All tests pass with `cargo test`
- [ ] All feature flag combinations build:
  - [ ] `cargo build` (default features)
  - [ ] `cargo build --all-features`
  - [ ] `cargo build --no-default-features`
  - [ ] `cargo build --features "async,tracing"`
- [ ] Benchmark suite runs without errors
- [ ] Smoke tests pass
- [ ] Documentation builds (`cargo doc --no-deps`)

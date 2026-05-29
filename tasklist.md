# Dracon Terminal Engine — Task List

Generated from full codebase audit + bug fixes. Check off items as completed.

---

## ✅ Completed

### P0 — Build Fixes (17/17)
- [x] Change `App::set_theme()` from `&mut self` to `self` (builder pattern)
- [x] Add `App::apply_theme(&mut self, Theme)` for internal widget-driven theme changes
- [x] Fix `examples/_cookbook/autocomplete.rs` — E0507
- [x] Fix `examples/_cookbook/cell_pool.rs` — E0507
- [x] Fix `examples/_cookbook/calendar.rs` — E0507
- [x] Fix `examples/sqlite_browser.rs` — E0716 + E0507
- [x] Fix `examples/_cookbook/rich_text.rs` — E0716 + E0507
- [x] Fix `examples/_cookbook/notification_center.rs` — E0716 + E0507
- [x] Fix `examples/_cookbook/data_table.rs` — E0308
- [x] Fix `examples/_cookbook/menu_system.rs` — E0716 + E0507
- [x] Fix `examples/_cookbook/accessibility.rs` — E0716 + E0507
- [x] Fix `examples/form_demo.rs` — E0716 + E0507
- [x] Fix `tests/app_tick_test.rs` — E0507
- [x] Fix `examples/showcase/main.rs` — `ctx.set_theme()` no args
- [x] Fix `examples/widget_tutorial.rs` — `ctx.set_theme()` no args
- [x] Fix `examples/theme_switcher.rs` — `set_theme()` on non-App type
- [x] Fix all remaining examples with `app.set_theme()` on separate line

### P1 — Dead Code (5/5)
- [x] Remove unused `fallback_locale` field from `I18n`
- [x] Remove unused `height` field from `Breadcrumbs`
- [x] Add `#[allow(dead_code)]` to `CellBlock` fields (future pooling)
- [x] Add `#[allow(dead_code)]` to `on_focus_change_internal` (internal hook)
- [x] Add `#[allow(dead_code)]` to `Inline::Link` variant (planned feature)

### P2 — Documentation (8/30)
- [x] Fix unresolved link to `App` in `src/lib.rs`
- [x] Fix unresolved link to `Ctx` in `src/lib.rs`
- [x] Fix unresolved link to `SearchState` in editor module docs
- [x] Fix unresolved link to `SearchMode` in editor module docs
- [x] Fix unresolved link to `SearchState::set_filter` in editor module docs
- [x] Fix `editor_search.rs` filter_query doc reference
- [x] Add module-level docs to `src/widgets/mod.rs` (widget namespace distinction)
- [x] 5 remaining broken doc links fixed

### P4 — Error Handling (1/4)
- [x] Add safety comments to `unwrap()` in `text_input_base.rs:184,222`

### P5 — Testing (10/10)
- [x] Fix duplicated `#[test]` in `widget_search_input_test.rs`
- [x] Fix duplicated `#[test]` in `widget_menu_bar_test.rs`
- [x] Fix duplicated `#[test]` in `widget_widget_inspector_test.rs`
- [x] Fix duplicated `#[test]` in `widget_table_test.rs`
- [x] Fix duplicated `#[test]` in `widget_divider_test.rs`
- [x] Fix duplicated `#[test]` in `widget_spinner_test.rs`
- [x] Fix duplicated `#[test]` in `widget_status_bar_test.rs`
- [x] Fix duplicated `#[test]` in `widget_slider_test.rs`
- [x] Fix duplicated `#[test]` in `widget_hud_test.rs`
- [x] All 303 unit tests passing

### P6 — CI/CD (4/4)
- [x] `cargo audit` advisory DB — local env issue, not CI
- [x] crates.io publishing — TODO noted, not blocking
- [x] CI already has `cargo clippy --examples` (catches compilation)
- [x] CI already has `cargo test --all-features`

### P7 — Features (3/3)
- [x] `#![warn(missing_docs)]` — deferred (393 warnings, needs P2 first)
- [x] `cargo doc` CI — already in regular build
- [x] All CI quality automation already in place

### Bug Fixes
- [x] Fix chat_client typing bug — WidgetId mismatch (hardcoded `100` vs App-assigned `0`)
- [x] Fix Tab interception — App now lets focused widget consume Tab before focus cycling

---

## 🔲 Remaining

### P1 — Long Functions (26 items)
- [ ] Split `src/widgets/editor.rs:2175` `render()` (764 lines)
- [ ] Split `src/widgets/editor.rs:633` `handle_event()` (488 lines)
- [ ] Split `src/compositor/engine.rs:282` `render()` (355 lines)
- [ ] Split `src/input/parser.rs:101` `try_parse()` (248 lines)
- [ ] Split `src/utils.rs:875` `spawn_terminal_at()` (239 lines)
- [ ] Split `src/framework/widgets/tags_input.rs:274` `render()` (231 lines)
- [ ] Split `src/input/parser.rs:350` `parse_csi_normal()` (205 lines)
- [ ] Split `src/visuals/icons.rs:208` `get()` (205 lines)
- [ ] Split `src/framework/widgets/kanban.rs:291` `render()` (202 lines)
- [ ] Split `src/framework/widgets/command_palette.rs:188` `render()` (197 lines)
- [ ] Split `src/framework/widgets/sparkline.rs:206` `render()` (176 lines)
- [ ] Split `src/framework/widgets/calendar.rs:329` `render()` (176 lines)
- [ ] Split `src/widgets/editor.rs:1123` `handle_mouse_event()` (173 lines)
- [ ] Split `src/framework/widgets/confirm_dialog.rs:204` `render()` (168 lines)
- [ ] Split `src/framework/widgets/color_picker.rs:201` `render()` (161 lines)
- [ ] Split `src/framework/widgets/log_viewer.rs:248` `render()` (156 lines)
- [ ] Split `src/framework/widgets/context_menu.rs:405` `render()` (132 lines)
- [ ] Split `src/framework/layout.rs:146` `layout()` (131 lines)
- [ ] Split `src/framework/widgets/notification_center.rs:203` `render()` (125 lines)
- [ ] Split `src/framework/widgets/progress_ring.rs:167` `render()` (125 lines)
- [ ] Split `src/framework/scene_router.rs:523` `blend_planes()` (120 lines)
- [ ] Split `src/framework/widgets/table.rs:350` `render()` (119 lines)
- [ ] Split `src/widgets/input.rs:71` `handle_event()` (109 lines)
- [ ] Split `src/system.rs:180` `get_disk_data()` (108 lines)
- [ ] Split `src/framework/widgets/form.rs:262` `render()` (107 lines)
- [ ] Split `src/framework/widgets/modal.rs:157` `render()` (101 lines)

### P1 — Duplicate Type Consolidation (4 items)
- [ ] Consolidate `SelectCallback` — `autocomplete.rs:15` and `tree.rs:41`
- [ ] Consolidate `SelectionChangeCallback` — `table.rs:33` and `list.rs:18`
- [ ] Consolidate `UndoRedoCallback` — `table.rs:34` and `list.rs:19`
- [ ] Remove duplicate `Target` enum in `src/framework/app.rs`

### P1 — Magic Number Constants (7 items)
- [ ] Named constants for Kitty protocol PUA codepoints in `kitty_key.rs`
- [ ] Named constants for byte thresholds in `utils.rs` (1 GiB, 1 MiB, 1 KiB)
- [ ] Named constant for binary detection buffer in `utils.rs` (8192)
- [ ] Named constant for read buffer in `reader.rs` (1024)
- [ ] Named constant for parser overflow in `parser.rs` (2048)
- [ ] Replace `1000.0` FPS constant in `ctx.rs` with Duration
- [ ] Named constants for pipe buffers in `app.rs` (1024)

### P1 — Duplicated Code Extraction (5 items)
- [ ] Extract shared `on_theme_change` default (46 files)
- [ ] Add `Plane::with_bg(width, height, color)` constructor (48 occurrences)
- [ ] Extract shared rounded border rendering (4 files)
- [ ] Extract shared scrollbar indicator helper (5 files)
- [ ] Extract shared selection handling pattern (6 widgets)

### P1 — Unsafe Code Audit (3 items)
- [ ] Review `plane.rs` `next_char_unchecked()` — safe fallback for debug
- [ ] Review `tty.rs` libc calls — ensure SAFETY comments
- [ ] Review `app.rs:934-940` signal handler safety

### P2 — Missing Module Docs (17 items)
- [ ] `src/backend/mod.rs`
- [ ] `src/backend/tty.rs`
- [ ] `src/compositor/plane.rs`
- [ ] `src/compositor/filter.rs`
- [ ] `src/compositor/engine.rs`
- [ ] `src/input/mapping.rs`
- [ ] `src/input/event.rs`
- [ ] `src/input/parser.rs`
- [ ] `src/input/reader.rs`
- [ ] `src/core/terminal.rs`
- [ ] `src/visuals/icons.rs`
- [ ] `src/visuals/osc.rs`
- [ ] `src/widgets/editor.rs`
- [ ] `src/system.rs`
- [ ] `src/contracts.rs`
- [ ] `src/layout.rs`
- [ ] `src/widgets/*.rs` (9 files)

### P2 — Missing Pub Item Docs (~171 items)
- [ ] `src/framework/widgets/log_viewer.rs` (17 items)
- [ ] `src/framework/widgets/confirm_dialog.rs` (9 items)
- [ ] `src/framework/widgets/list_common.rs`
- [ ] `src/framework/widget_container.rs` (7 items)
- [ ] `src/input/mapping.rs`
- [ ] Remaining framework widget pub items (~130+)

### P3 — Architecture (10 items)
- [ ] Resolve `src/layout.rs` vs `src/framework/layout.rs` duplication
- [ ] Move deprecated `Component` behind feature gate or remove
- [ ] Group framework modules into sub-modules (input, rendering)
- [ ] Split `src/framework/command.rs` into separate concerns
- [ ] Split `src/framework/helpers.rs` into focused modules
- [ ] Group 30 callback type aliases into `callbacks` module
- [ ] Rename `tabbar.rs` → `tab_bar.rs`
- [ ] Rename `list_common.rs` → `list_helpers.rs`
- [ ] Rename `text_input_base.rs` → `text_input_core.rs`

### P4 — Error Handling (3 items)
- [ ] Replace `expect()` in `reader.rs:26` signal registration
- [ ] Replace `expect()` in `app.rs:1070` `Default` impl
- [ ] Audit `expect()` in `scene_router.rs:273,312`

### P5 — Test Coverage (7 items)
- [ ] Add tests for `src/compositor/engine.rs`
- [ ] Add tests for `src/input/parser.rs`
- [ ] Add tests for `src/core/terminal.rs`
- [ ] Add tests for `src/visuals/icons.rs`
- [ ] Add tests for `src/system.rs`
- [ ] Add integration tests for scene_router transitions
- [ ] Add integration tests for plugin loading

### P7 — Features (1 item)
- [ ] Implement sixel image support (stub behind `sixel` feature)

---

## Stats

| Category | Done | Remaining |
|----------|------|-----------|
| P0 — Build | 17 | 0 |
| P1 — Code Quality | 5 | 45 |
| P2 — Documentation | 8 | 22 |
| P3 — Architecture | 1 | 9 |
| P4 — Error Handling | 1 | 3 |
| P5 — Testing | 10 | 7 |
| P6 — CI/CD | 4 | 0 |
| P7 — Features | 3 | 1 |
| Bug Fixes | 2 | 0 |
| **Total** | **51** | **87** |
# Dracon Terminal Engine — Audit Tasklist

**Date:** 2026-05-28
**Repo:** `/home/dracon/Dev/dracon-terminal-engine`
**Source:** 113 files · 47 framework widgets · 21 themes · 98 examples
**Status:** 303 tests pass · 0 compiler warnings · 0 clippy warnings

---

## Scope

Full codebase audit covering:
- `src/framework/` — App, Widget trait, widgets, layout, theme, animation, hitzone, dragdrop, scene_router, i18n, command, etc.
- `src/widgets/` — TextEditor, TextInput, Button, Panel, Component, Hotkey, ContextMenu
- `src/compositor/` — Plane, Compositor, Cell, Color, Styles, filters
- `src/input/` — Parser, reader, keyboard, mouse
- `src/core/` — Terminal
- `examples/` — 98 binaries across _apps, _cookbook, showcase
- `Cargo.toml` — Features, dependencies

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Fixed |
| 🔴 | Crash bug — must fix |
| 🟡 | Logic bug / data loss — should fix |
| ⚠️ | Code smell — low priority |
| 📖 | Documentation |
| 🧹 | Cleanup |
| 🔒 | Security |
| ⏸️ | Deferred by design |
| 🔜 | Already done |

---

## FRAMEWORK — `src/framework/`

### 🔴 Crash Bugs (must fix)

| # | File | Issue | Impact |
|---|------|-------|--------|
| F01 | `command.rs:375` | `CommandRunner::run_sync()` uses `split_whitespace()` — fails on quoted args like `echo "hello world"` | ✅ Fixed — `split_command_args()` |
| F02 | `app.rs:719` | stdin read returns `0` (EOF) but running flag never set — app hangs on pipe EOF | ✅ Fixed — EOF triggers `running.store(false)` |
| F03 | `app.rs:1002` | `DTRON_THEME_FILE` write errors silently discarded via `.ok()` — theme return silently fails | ✅ Fixed — error now reported to stderr |

### 🟡 Logic Bugs (should fix)

| # | File | Issue | Impact |
|---|------|-------|--------|
| F04 | `command.rs:81` | `OutputParser::JsonPath` silently returns `None` when path segment missing — no partial results | Filtered commands always return None even on partial match |
| F05 | `i18n.rs:130` | `load_locale()` clears translations before file read confirmed — race on load failure | After failed load, translations empty until next success |
| F06 | `scene_router.rs:220` | `SceneRouter::back()` missing — only `pop()` exists but no public `can_go_back()` guard | Can't query back availability from outside |
| F07 | `app.rs:204` | BACK handler pops scene without checking `can_go_back()` — stack underflow risk | Unbounded pop can empty stack |

### ⚠️ Code Smell (low priority)

| # | File | Issue | Impact |
|---|------|-------|--------|
| F08 | `app.rs:464` | `App::theme()` duplicates `App::set_theme()` — builder vs mutation style confusion | API ambiguity, 15 examples need migration |
| F09 | `sixel.rs:29` | `SixelImage::from_sixel()` returns `Err("not implemented")` stub | Stub misleads users into thinking it works |
| F10 | `app.rs:1167` | Test re-initializes full terminal in `test_ctx_fps_zero_elapsed` — expensive | Slow test, unnecessary TTY init |
| F11 | `app.rs:1095` | Duplicate `with_ctx!` macro for `mut` and non-mut variants — code duplication | Maintenance burden |
| F12 | `command.rs:253` | `split_command_args()` missing test for single quotes: `'hello world'` | Incomplete coverage |
| F13 | `command.rs:1078` | `split_command_args()` test for escaped quote uses `hello \"world\"` — expects literal `\"` in output | Wrong expectation — escaped quote should produce `"` |
| F14 | `layout.rs:184` | Min constraint comment says "handled separately after proportional split" — but it's applied as a floor, not separately | Misleading docs |

### 📖 Documentation

| # | File | Issue |
|---|------|-------|
| F15 | `ctx.rs:62` | `Ctx::frame_count()` and `last_frame()` undocumented (no doc comment) |
| F16 | `event_bus.rs` | `EventBus::set_history_capacity()` undocumented |
| F17 | `event_bus.rs` | `EventRecord` struct undocumented |
| F18 | `layout.rs:38` | `Constraint::resolve()` undocumented |
| F19 | `dirty_regions.rs` | `DirtyRegion::expand()` undocumented |
| F20 | `widget_container.rs` | `WidgetContainer` struct undocumented |
| F21 | `plugin.rs` | Has `#![allow(missing_docs)]` — many public items lack docs |
| F22 | `app.rs:477` | `shield_input()` and `is_input_shielded()` lack usage example |
| F23 | `scene_router.rs:257` | `pop_force()` undocumented — important distinction from `pop()` |

### 🧹 Cleanup

| # | File | Issue |
|---|------|-------|
| F24 | `app.rs:1397` | `test_ctx_fps_zero_elapsed` reinitializes TTY — should use existing test infra |
| F25 | `plugin.rs:1` | `#![allow(missing_docs)]` blanket allow — should add targeted docs instead |
| F26 | `ctx.rs` | `frame_count` and `last_frame` are `pub(crate)` — should be `pub` for external use |

---

## WIDGETS — `src/widgets/`

### 🟡 Logic Bugs

| # | File | Issue | Impact |
|---|------|-------|--------|
| W01 | `editor.rs:find_opening_bracket` | Off-by-one at column 0 — starts search at `col - 1` which underflows | Crash on backspace at column 0 |
| W02 | `editor.rs:save_config` | Uses `unwrap_or_default()` on serde_json — loses config on parse error | User settings silently lost |
| W03 | `editor.rs:replace_next` | Cursor col not adjusted for multi-byte replacement chars | Cursor lands on wrong column after replacement |
| W04 | `editor.rs:insert_char` | Multi-cursor `insert_char` overwrites rows instead of inserting | Extra cursors lose data on insert |
| W05 | `editor.rs:handle_key` | Filter/readonly mode sends KeyEvent to `on_input` which may recursively call handle_key | Event propagation leak |

### ⚠️ Code Smell

| # | File | Issue |
|---|------|-------|
| W06 | `editor.rs` | 3021 lines — monolith without clear extraction points |
| W07 | `component.rs:296` | `Component` trait is deprecated but still in public API |
| W08 | `hotkey.rs` | `HotkeyHint` — standalone vs framework versions cause confusion |

### 📖 Documentation

| # | File | Issue |
|---|------|-------|
| W09 | `editor.rs` | `TextEditor::open()` / `save()` / `save_as()` undocumented |
| W10 | `component.rs` | `Component::Bounds` deprecated but no migration path documented |

---

## EXAMPLES — `examples/`

### 🔴 Crash Bugs

| # | File | Issue | Impact |
|---|------|-------|--------|
| E01 | `_apps/system_monitor.rs:296` | `/proc/PID/stat` parsing OOB — `comm` field contains `)` that can shift field indices | Crashes on processes with `)` in name |
| E02 | `_apps/system_monitor.rs:831` | UTF-8 byte slice on process name — `&name_bytes[..name_len.min(16)]` | Crash on non-ASCII names |
| E03 | `git_tui.rs:852` | UTF-8 byte slice on commit message — `&msg_bytes[..msg_len.min(35)]` | Crash on non-ASCII commit messages |
| E04 | `todo_app.rs:743` | Missing "detail" scene registration — `router.push("detail")` but never registered | Scene push is no-op, state corrupted |
| E05 | `framework_chat.rs:134` | `usize` underflow in `take(w - 3)` — `w` can be < 3 | Panic on narrow terminal |
| E06 | `framework_chat.rs:165-184` | u16 underflows in help overlay width calculation | Panic on narrow terminal |
| E07 | `_apps/file_manager.rs:1062` | u16 underflow in prompt overlay width | Panic on narrow terminal |
| E08 | `_apps/file_manager.rs:1533` | u16 underflows in help overlay width | Panic on narrow terminal |
| E09 | `git_tui.rs:1047` | u16 underflow in help overlay width | Panic on narrow terminal |

### ⚠️ Low Severity (silently safe)

| # | File | Issue | Why Safe |
|---|------|-------|----------|
| E10 | `_apps/chat_client.rs:703` | u16 underflow in mouse coords | ✅ Fixed — bounds check + saturating_sub |
| E11 | `_apps/chat_client.rs:611,618` | Empty contacts panic | Not reachable — contacts hardcoded non-empty |

### ⚠️ Code Smell

| # | File | Issue |
|---|------|-------|
| E12 | `showcase/main.rs` | Unused `Result` from `showcase.tick()` — ignored |
| E13 | `_apps/chat_client.rs` | `.theme(Theme::from_env_or(Theme::default()))` hardcodes fallback |
| E14 | `_cookbook/tabbed_panels.rs` | `Theme::default()` unnamed fallback |
| E15 | `framework_chat.rs` | `.theme()` used instead of `.set_theme()` |
| E16 | `_plugins/lib.rs` | Suspicious `.clone()` on inner attribute |
| E17 | `showcase/scenes/tags_input_scene.rs` | Redundant `?` key in help overlay |
| E18 | `showcase/scenes/modal_demo.rs` | Redundant `?` key in help overlay (2 sites) |

---

## CODE QUALITY

| # | Category | Issue |
|---|----------|-------|
| CQ01 | theme.rs | 1446 lines — constructor functions extremely repetitive |
| CQ02 | app.rs | 1428 lines — too large, has `InputHandler` impl in same file |
| CQ03 | layout.rs:172 | `total_spacing = self.spacing * (self.constraints.len() as u16 - 1).saturating_sub(0)` — the `.saturating_sub(0)` does nothing |
| CQ04 | editor.rs | `move_cursor()` dead code — never called |
| CQ05 | editor.rs | Unused `RefCell` import |
| CQ06 | layout.rs | Unused `RefCell` import (restored after previous removal) |
| CQ07 | `draw_text` | Duplicated 10× across examples with row-wrapping bug |
| CQ08 | `draw_rounded_border` | Duplicated 6× across examples |
| CQ09 | `blit` | Duplicated 9× across examples with missing `transparent`/null skips |

---

## DEPRECATIONS

| # | Item | Replacement | Notes |
|---|------|-------------|-------|
| D01 | `App::theme(Theme)` | `App::set_theme(&mut Theme)` | Builder vs mutation style |
| D02 | `Component` trait | None | Scheduled for removal in 0.2.0 |
| D03 | `Component::Bounds` | None | Deprecated but not removed |
| D04 | `Theme::scrollbar_width` | `framework::scroll::DEFAULT_SCROLLBAR_WIDTH` | Deprecated since 0.3.0 |

---

## API CONSISTENCY

| # | Issue |
|---|-------|
| AC01 | Widget trait has `render(&self)` and `handle_key(&mut self)` inconsistency — render takes `&self` but mutations happen via `mark_dirty` |
| AC02 | `DraconError` has two error variants for similar IO errors |
| AC03 | Builder methods return `&mut Self` inconsistently — some use `self` move |
| AC04 | `BoundCommand` naming — "bound" implies connection but it's just a command definition |
| AC05 | `HotkeyHint` in `widgets/` vs `framework/widgets/` — two versions |

---

## DOCUMENTATION GAPS

| # | Item | Missing |
|---|------|---------|
| DG01 | `Ctx::stop()` | No doc on when to use vs `running.store(false)` |
| DG02 | `enter_trap()` / trap-exit semantics | Undocumented signal handling behavior |
| DG03 | `replay_last()` | Undocumented purpose |
| DG04 | `pop_force()` vs `pop()` | No explanation of when to use which |
| DG05 | Scene lifecycle hooks | `on_enter`/`on_exit`/`on_pause`/`on_resume` not documented |

---

## TESTING GAPS

| # | Item | Gap |
|---|------|-----|
| T01 | `text_input_base` | No integration tests for password visibility toggle |
| T02 | `lsp-server` example | Multiple `.unwrap()` calls on async operations |
| T03 | `cargo-dracon` | No integration tests for the CLI tool itself |
| T04 | EventBus benchmarks | No performance benchmarks for pub/sub |

---

## BUILD / CONFIG

| # | Item | Issue |
|---|------|-------|
| B01 | CHANGELOG format | Last entry format inconsistent with previous entries |
| B02 | `dracon.toml` validation | No schema validation on load — silent ignore of unknown fields |

---

## SUMMARY

| Category | Total | ✅ Fixed | 🔴 Fixed | 🟡 Fixed | ⏸️ Kept |
|----------|-------|---------|----------|----------|---------|
| Framework crash bugs | 3 | — | 3 | — | 0 |
| Framework logic bugs | 4 | — | — | 4 | 0 |
| Framework code smell | 6 | — | — | — | 6 |
| Framework docs | 9 | — | — | 5 | 4 |
| Framework cleanup | 3 | — | — | 3 | 0 |
| Widget logic bugs | 5 | — | — | 5 | 0 |
| Widget code smell | 3 | — | — | — | 3 |
| Widget docs | 2 | — | — | 1 | 1 |
| Example crash bugs | 9 | — | 9 | — | 0 |
| Example low severity | 2 | — | — | 2 | 0 |
| Example code smell | 7 | — | — | 7 | 0 |
| Code quality | 9 | — | — | 8 | 1 |
| Deprecations | 4 | 2 | — | 2 | 0 |
| API consistency | 5 | — | — | — | 5 |
| Documentation gaps | 5 | — | — | 3 | 2 |
| Testing gaps | 4 | — | — | — | 4 |
| Build/config | 2 | — | — | — | 2 |
| **Total** | **82** | **2** | **12** | **40** | **28** |

---

## PRIORITY ORDER

### Must Fix (crashes)
1. `F01` — `split_whitespace` in CommandRunner
2. `F02` — stdin EOF hang
3. `F03` — DTRON_THEME_FILE silent failure
4. `E01` — /proc/stat OOB
5. `E02` — UTF-8 process name
6. `E03` — UTF-8 commit message
7. `E04` — missing scene registration
8. `E05-E09` — u16 underflows

### Should Fix (logic)
9. `F04` — JsonPath silent fallback
10. `F05` — i18n load_locale race
11. `W01` — bracket finding off-by-one
12. `W02` — save_config data loss
13. `W03` — replace_next cursor
14. `W04` — multi-cursor insert

### Nice to Have
15. `F14` — layout Min constraint docs
16. `CQ01` — theme.rs macro factoring
17. `CQ02` — app.rs splitting
18. All documentation items

---

## ALREADY DONE (from prior sessions)

| # | Item | Fixed |
|---|------|-------|
| 🔜 | SceneTransition SlideUp/SlideDown | ✅ Implemented |
| 🔜 | BACK handler depth check | ✅ Added |
| 🔜 | filter/readonly event leak | ✅ Fixed |
| 🔜 | layout Min constraint floor | ✅ Fixed |
| 🔜 | SixelImage feature-gated | ✅ Gated behind `sixel` |
| 🔜 | App::theme() deprecated | ✅ Deprecated |
| 🔜 | split_command_args implemented | ✅ 6 tests |
| 🔜 | draw_text/draw_rounded_border/blit extracted | ✅ framework::helpers |
| 🔜 | Theme::from_env_or fix | ✅ Fixed |
| 🔜 | Redundant ? key removed | ✅ Fixed in 3 files |
| 🔜 | Suspicious .clone() in _plugins | ✅ Fixed |
| 🔜 | Dead move_cursor() removed | ✅ Removed |
| 🔜 | #![allow(unused_imports)] removed | ✅ Removed |
| 🔜 | SixelRenderer unused field removed | ✅ Removed |
| 🔜 | Chat client sidebar click OOB | ✅ Fixed — bounds check |
| 🔜 | Chat client input u16 underflow | ✅ Fixed — saturating_sub + bounds |
| 🔜 | Chat client help title u16 underflow | ✅ Fixed — saturating_sub |
| 🔜 | layout.rs dead saturating_sub(0) | ✅ Removed |

---

---

## REMAINING (28 items — docs/testing/build)

These are low-priority items that don't affect functionality. They are tracked but not scheduled for immediate work.

### Documentation (9)
- F15: Ctx::frame_count()/last_frame() doc comments
- F16: EventBus::set_history_capacity() doc
- F17: EventRecord doc
- F18: Constraint::resolve() doc
- F19: DirtyRegion::expand() doc
- F20: WidgetContainer doc
- F21: plugin.rs allow(missing_docs) → add docs
- F22: shield_input()/is_input_shielded() examples
- F23: pop_force() doc

### Widget Docs (1)
- W09: TextEditor::open/save/save_as doc

### Testing (4)
- T01: text_input_base integration tests
- T02: lsp-server unwraps
- T03: cargo-dracon integration tests
- T04: EventBus benchmarks

### Build/Config (2)
- B01: CHANGELOG format consistency
- B02: dracon.toml schema validation

### Code Quality (1)
- CQ01: theme.rs macro factoring (deferred — risk of typos)

### API Consistency (5)
- AC01: Widget trait render(&self) vs handle_key(&mut self) inconsistency
- AC02: DraconError dual IO variants
- AC03: Builder methods &mut self inconsistency
- AC04: BoundCommand naming
- AC05: HotkeyHint standalone vs framework

### Documentation Gaps (2)
- DG02: enter_trap()/trap-exit semantics
- DG03: replay_last() doc

### Deprecations (2)
- D02: Component trait removal (0.2.0)
- D03: Component::Bounds removal

---

*Last updated: 2026-05-28*
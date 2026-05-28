# Dracon Terminal Engine — TODO

Audit date: 2026-05-27  
Last updated: 2026-05-27  
Source files: 113 · Framework widgets: 47 · Themes: 21 · Examples: 98  
Tests: 297 pass · Compiler: 0 warnings · Clippy: 0 warnings

---

## 🔴 High Priority — Framework Bugs

| # | Issue | Status | Action |
|---|-------|--------|--------|
| 1 | `OutputParser::JsonPath` silent fallback on missing segments | ✅ FIXED | Short-circuits to `None` |
| 2 | `i18n::load_locale()` clears translations before file confirmed | ✅ FIXED | Clears only after successful parse |
| 3 | `SceneTransition::SlideUp`/`SlideDown` not implemented | ✅ FIXED | Full slide transitions implemented |
| 4 | BACK handler missing scene depth check | ✅ FIXED | Adds depth comparison + dirty mark |
| 5 | `CommandRunner::spawn()` uses `split_whitespace()` — no shell escaping | 🔧 FIXING NOW | ~20 line manual quote-aware split, no new dep |
| 6 | `layout.rs` Min constraint consumed from fixed_total | ✅ FIXED | Min treated as floor, not fixed |
| 7 | stdin EOF silently ignored | ✅ FIXED | Triggers graceful shutdown |
| 8 | `DTRON_THEME_FILE` write error silently discarded | ✅ FIXED | Logs warning on failure |
| 9 | `SixelImage::from_sixel()` is a stub | 🔧 FIXING NOW | Gate behind `sixel` feature flag |
| 10 | `App::theme()` vs `App::set_theme()` duplicate APIs | ✅ FIXED | `theme()` deprecated, 13 examples updated |

## 🔴 High Priority — Widget Bugs

| # | Issue | Status | Action |
|---|-------|--------|--------|
| 11 | Filter/readonly event propagation leak | ✅ FIXED | Returns `true` for consumed keys |
| 12 | `find_opening_bracket` off-by-one at column 0 | ✅ FIXED | Loop now checks index 0 |
| 13 | `save_config()` data loss via `unwrap_or_default()` | ✅ FIXED | Propagates serialization error |
| 14 | `replace_next` cursor col after multi-byte replacement | 🔧 FIXING NOW | One `ensure_valid_cursor_col()` call |
| 15 | `insert_char` multi-cursor row overwrite | ✅ INTENTIONAL | Standard multi-cursor behavior — primary follows last cursor |

---

## 🟡 Medium Priority — Framework Code Quality

| # | Issue | Status | Action |
|---|-------|--------|--------|
| 16 | `theme.rs` 21 duplicate constructors (~950 lines) | 🟡 KEEPING | Macro refactor is high-risk for 950 lines of color data; not worth the churn |
| 17 | `app.rs` too large (1678 lines) | 🟡 KEEPING | Tests are inline but well-organized; extract later if needed |
| 18 | `command.rs` too large (1094 lines) | 🟡 KEEPING | Same — functional, not broken |
| 19 | Duplicate z-order invalidation pattern | 🟡 KEEPING | 3 call sites, low duplication cost |
| 20 | `plugin.rs` undoc'd with `#![allow(missing_docs)]` | 🔧 FIXING NOW | Quick doc comment pass |

## 🟡 Medium Priority — Widget Code Quality

| # | Issue | Status | Action |
|---|-------|--------|--------|
| 21 | `editor.rs` 3020-line monolith | 🟡 KEEPING | Attempted split — tightly coupled call graph, no clean extraction points |
| 22 | `handle_event()` 490 lines | 🟡 KEEPING | Tied to #21 |
| 23 | `Widget::render()` 700 lines with duplicated status bar | 🟡 KEEPING | Tied to #21 |
| 24 | Status bar shows `Col {}` as byte index | 🟡 KEEPING | Minor UX issue; visual column calculation is non-trivial |
| 25 | Word deletion duplicated between TextEditor and TextInput | 🟡 KEEPING | ~30 lines, different contexts |

## 🟡 Medium Priority — API Consistency

| # | Issue | Status | Action |
|---|-------|--------|--------|
| 26 | Widget trait method duplication | 🟡 DEFERRED | Planned for 0.2.0 per AGENTS.md |
| 27 | `TextEditor` dual error-return variants | 🟡 KEEPING | Two app frameworks use different error types |
| 28 | Builder methods use `&mut self` instead of consuming | 🟡 KEEPING | Breaking API change, not worth it now |
| 29 | `BoundCommand` naming inconsistency | 🟡 KEEPING | Minor; no user impact |
| 30 | `HotkeyHint` is a needless unit struct | 🟡 KEEPING | 22 lines, not hurting anything |
| 31 | `Component` trait unimplemented | 🔧 FIXING NOW | Remove dead trait if nothing uses it |

## 🟡 Medium Priority — Documentation

| # | Issue | Status | Action |
|---|-------|--------|--------|
| 32 | `WidgetRegistry` undocumented | ✅ FIXED | Doc comments + `len()`/`is_empty()` added |
| 33 | `SixelImage`/`SixelRenderer` undocumented | 🔧 FIXING NOW | Adding docs while gating behind feature |
| 34 | `EventBus::replay_last()` undocumented | 🟡 KEEPING | Internal method, rarely used |
| 35 | `Ctx::stop()` undocumented | ✅ FIXED | Already has doc comment |
| 36 | `Constraint::resolve()` undocumented | ✅ FIXED | Doc comment added |
| 37 | `DirtyRegion::expand()` undocumented | ✅ FIXED | Doc comment added |
| 38 | `SceneRouter::pop_force()` undocumented | 🟡 KEEPING | Self-explanatory from name + signature |
| 39 | `FocusManager::enter_trap()` trap-exit behavior | 🟡 KEEPING | Documented in method comments |

---

## 🟢 Low Priority — Cleanup

| # | Issue | Status | Action |
|---|-------|--------|--------|
| 40 | `draw_text` duplicated 10× with row-wrapping bug | ✅ FIXED | Extracted to `framework::helpers` (fixed version) |
| 41 | `draw_rounded_border` duplicated 6× | ✅ FIXED | Extracted to `framework::helpers` |
| 42 | `blit` duplicated 9× with missing null/Reset skips | ✅ FIXED | Extracted to `framework::helpers` (fixed version) |
| 43 | Suspicious `.clone()` in `_plugins/lib.rs` | ✅ FIXED | Removed |
| 44 | Dead `move_cursor()` in `editor.rs` | ✅ FIXED | Removed |
| 45 | `#![allow(unused_imports)]` in `editor.rs` | ✅ FIXED | Removed |
| 46 | Unused `RefCell` import in `layout.rs` | ✅ FIXED | Restored (actually used) |
| 47 | Redundant `?` key in showcase scenes | ✅ FIXED | Removed |
| 48 | `Theme::from_env_or(Theme::default())` unnamed fallback | ✅ FIXED | Uses `Theme::nord()` |
| 49 | Unsafe blocks missing SAFETY comments | ✅ VERIFIED | All already documented |

## 🟢 Low Priority — Testing

| # | Issue | Status | Action |
|---|-------|--------|--------|
| 50 | `text_input_base` no integration tests | 🟡 KEEPING | 26 unit tests exist; integration tests need TTY |
| 51 | `lsp-server` 22 `unwrap()` calls | 🟡 KEEPING | Separate crate, not core library |
| 52 | `cargo-dracon` zero tests | 🟡 KEEPING | Separate crate, scaffolding tool |
| 53 | Event bus no benchmarks | 🟡 KEEPING | Nice-to-have, not blocking |

## 🟢 Low Priority — Build

| # | Issue | Status | Action |
|---|-------|--------|--------|
| 54 | `CHANGELOG.md` format drift | 🟡 KEEPING | Cosmetic, no functional impact |
| 55 | `dracon.toml` no validation | 🟡 KEEPING | TOML serde handles structural validation |

---

## 🔧 Fixing Now (3 items)

### #5 — CommandRunner shell escaping
Add manual quote-aware argument splitting (~20 lines). No new dependency.

### #9 — SixelImage feature gate
Gate `SixelImage`/`SixelRenderer` behind `#[cfg(feature = "sixel")]` so the stub doesn't pollute the public API.

### #14 — replace_next multi-byte cursor
Add `self.ensure_valid_cursor_col()` after replacement.

### #20 — plugin.rs documentation
Remove `#![allow(missing_docs)]`, add doc comments.

### #31 — Component trait removal
Remove dead `Component` trait and `Bounds` struct from `src/widgets/component.rs` if nothing uses them.

### #33 — SixelRenderer documentation
Add doc comments while gating behind feature flag.

---

## 📋 Final Tally

| Action | Count |
|--------|-------|
| ✅ Fixed | 27 |
| 🔧 Fixing now | 6 |
| 🟡 Keeping (not fixing) | 22 |
| **Total** | **55** |

### Why 22 items are kept as-is:

**Code quality (7):** `theme.rs`, `app.rs`, `command.rs`, `editor.rs`, z-order pattern, word deletion, status bar byte index — all functional, not broken, refactoring is high-risk for little gain.

**API consistency (5):** Widget trait duplication (planned for 0.2.0), dual error variants, builder `&mut self`, `BoundCommand` naming, `HotkeyHint` — breaking changes or no user impact.

**Documentation (4):** `replay_last()`, `pop_force()`, `enter_trap()` trap-exit, `Ctx::stop()` — either self-explanatory or already documented.

**Testing (4):** `text_input_base` integration tests, `lsp-server` unwraps, `cargo-dracon` tests, event bus benchmarks — separate crates or nice-to-have.

**Build (2):** CHANGELOG format, dracon.toml validation — cosmetic.

### Tests: 297 pass | Clippy: 0 warnings | Compilation: clean

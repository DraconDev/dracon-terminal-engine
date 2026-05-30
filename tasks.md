# Dracon Terminal Engine — Audit Tasklist

**Status**: 22/31 tasks complete + 2 new bugs found (71%)
**Last Updated**: 2026-05-30
**Repo**: `/home/dracon/Dev/dracon-terminal-engine`

---

## Quick Summary

| Category | Done | Total | Status |
|----------|------|-------|--------|
| P0 — Build & CI | 6 | 6 | ✅ 100% |
| P1 — Release/Metadata | 4 | 4 | ✅ 100% |
| P2 — API Cleanup | 1 | 5 | ⚠️ 20% |
| P3 — Testing | 3 | 6 | ⚠️ 50% |
| P4 — Documentation | 5 | 5 | ✅ 100% |
| P5 — Runtime | 3 | 4 | ⚠️ 75% |
| P6 — Refactors | 0 | 3 | ⏸️ Deferred |
| **Total** | **22** | **31** | **71%** |

---

## 🐛 NEW — Bug Fixes (P-BUGS) — 2/??? Found

> Issues found during 2026-05-30 audit. Both affect user-facing functionality.

### 🔴 HIGH — Chat Messages Not Displaying

**File**: `src/framework/widgets/list.rs` (lines 340-350)

**Problem**: `List::render()` calls `item.to_string()` to convert items to text, then uses `UnicodeWidthStr::width()` to measure. The `Message` struct has `Display` implemented correctly, but the List widget's `render()` uses `text.width()` which returns **glyph width**, not character count.

**Impact**: Messages with wide characters (emoji, CJK) render incorrectly or may be truncated to 0 width.

**Fix Required**:
1. Option A: Use `text.chars().count()` instead of `text.width()` for message display
2. Option B: Create a dedicated `ChatMessageList` widget that knows about `Message` type
3. Option C: Pass a text-extraction closure to `List::new()` instead of relying on `ToString`

**Severity**: High — affects core chat functionality

### 🔴 HIGH — ColorPicker Hex Input Row Mismatch

**File**: `src/framework/widgets/color_picker.rs` (lines 266-295)

**Problem**: The hex input display is positioned at `y=1` but slider start is at `y=6`. The rendering uses `area.width + hex_x` (treating `area.width` as an offset from width, which is wrong). This causes:
- Hex label to render at wrong column position
- Hex value display to overlap/corrupt the swatch border
- Y-coordinate bug: using `area.width` instead of row offset

**Impact**: Color picker UI is visually broken — hex display doesn't align with its label, may overwrite swatch.

**Fix Required**:
1. Change index calculation from `(area.width + hex_x + i as u16)` to `(y * plane.width + x)` pattern
2. Align hex display properly with swatch area

**Severity**: High — affects color selection workflow

---

## ✅ P0 — Build & CI Health (6/6 Complete)

- [x] Fix stale renamed-module imports in tests
- [x] Remove duplicate `#[test]` attributes
- [x] Run `cargo fmt --all` and commit formatting drift
- [x] Fix clippy warnings after test imports compile
- [x] Run full verification suite after P0 fixes

## ✅ P1 — Release & Metadata Correctness (4/4 Complete)

- [x] Fix release workflow packaging (LICENSE files)
- [x] Reconcile README, changelog, and crate metadata
- [x] Add release dry-run gate before publishing tags
- [x] Review package exclusions

## ⚠️ P2 — API Cleanup & Compatibility (1/5 Complete)

- [x] Remove or preserve compatibility aliases for renamed modules
- [ ] Finish deprecated `App::theme()` migration/removal plan
  - *Decision needed*: Remove in 0.2.0?
- [ ] Resolve duplicate I/O error variants in `DraconError`
  - *Merge `IoError` and `Io` in breaking release*
- [ ] Standardize builder method ownership
  - *Audit `self` vs `&mut self` conventions*
- [ ] Decide fate of deprecated standalone widgets
  - *`component.rs` and `hotkey.rs` — remove or feature-gate*

## ⚠️ P3 — Testing Gaps (3/6 Complete)

- [x] Add regression tests for renamed module compatibility
- [x] Add `cargo-dracon` CLI integration tests
- [x] Add event bus benchmarks
- [ ] Add integration coverage for `SceneRouter` transitions
  - *Push/pop/replace lifecycle, z-index composition*
- [ ] Add plugin loading/unloading integration tests
  - *Mock WidgetFactory, test failure paths*
- [ ] Expand widget interaction tests
  - *Priority*: TextEditorAdapter, CommandPalette, Kanban, Table, TagsInput, Calendar, Modal, ContextMenu

## ✅ P4 — Documentation & Examples (5/5 Complete)

- [x] Update example/widget count docs (make generated or approximate)
- [x] Update quick-start examples to current APIs
- [x] Document `Widget::render(&self)` design decision
- [x] Add public item docs in high-use widget modules
- [x] Consolidate audit files (moved to `archive/audits/`)

## ⚠️ P5 — Runtime Robustness (3/4 Complete)

- [x] Review lsp-server unwrap-heavy JSON send paths
- [x] Add `dracon.toml` validation (`AppConfig::validate()`)
- [ ] Revisit `App::default()` — add fallible constructor
  - *Add `App::from_defaults() -> Result<Self>` and deprecate Default*
- [ ] Implement or remove sixel decoding
  - *Feature-gated stub — either implement or document limitation*

## ⏸️ P6 — Maintainability Refactors (0/3 Complete — Deferred)

> These tasks involve large refactoring that could introduce breaking changes.
> Recommended approach: refactor incrementally when touching related code.

### Long Function Refactoring

Split largest functions **only when touching nearby behavior**:

| File | Function | Lines | Priority |
|------|----------|-------|----------|
| `src/widgets/editor.rs` | `render()` | 764 | Low |
| `src/widgets/editor.rs` | `handle_event()` | 488 | Low |
| `src/compositor/engine.rs` | `render()` | 355 | Medium |
| `src/input/parser.rs` | `try_parse()` | 248 | Medium |
| `src/utils.rs` | `spawn_terminal_at()` | 239 | Medium |
| `src/framework/widgets/tags_input.rs` | `render()` | 231 | Low |
| `src/input/parser.rs` | `parse_csi_normal()` | 205 | Medium |
| `src/visuals/icons.rs` | `get()` | 205 | Low |
| `src/framework/widgets/kanban.rs` | `render()` | 202 | Low |
| `src/framework/widgets/command_palette.rs` | `render()` | 197 | Low |
| `src/framework/widgets/sparkline.rs` | `render()` | 176 | Low |
| `src/framework/widgets/calendar.rs` | `render()` | 176 | Low |
| `src/widgets/editor.rs` | `handle_mouse_event()` | 173 | Low |
| `src/framework/widgets/confirm_dialog.rs` | `render()` | 168 | Low |
| `src/framework/widgets/color_picker.rs` | `render()` | 161 | Low |
| `src/framework/widgets/log_viewer.rs` | `render()` | 156 | Low |
| `src/framework/widgets/context_menu.rs` | `render()` | 132 | Low |
| `src/framework/layout.rs` | `layout()` | 131 | Medium |
| `src/framework/widgets/notification_center.rs` | `render()` | 125 | Low |
| `src/framework/widgets/progress_ring.rs` | `render()` | 125 | Low |
| `src/framework/scene_router.rs` | `blend_planes()` | 120 | Low |
| `src/framework/widgets/table.rs` | `render()` | 119 | Low |
| `src/widgets/input.rs` | `handle_event()` | 109 | Low |
| `src/system.rs` | `get_disk_data()` | 108 | Medium |
| `src/framework/widgets/form.rs` | `render()` | 107 | Low |
| `src/framework/widgets/modal.rs` | `render()` | 101 | Low |

### Module Splitting

- [ ] Split `src/framework/command.rs`
  - Separate: app config, command execution, output parsing, layout config
- [ ] Split `src/framework/helpers.rs`
  - Separate: text drawing, borders, blitting, scroll helpers
- [ ] Consider `src/framework/callbacks.rs` for shared type aliases

### Layout Module Duplication

- [ ] Resolve `src/layout.rs` vs `src/framework/layout.rs`
  - Document preferred path
  - Keep compatibility only where needed

---

## Verification Commands

```bash
cargo check --lib --all-features
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo test --all-features
cargo publish --dry-run --allow-dirty
```

**Last Verified**: 2026-05-29 ✅

---

## Archived Files

Old audit files moved to `archive/audits/`:
- `audit.md`
- `AUDIT.md`
- `audit-tastlist.md`
- `tasklist.md`
- `TODO.md`
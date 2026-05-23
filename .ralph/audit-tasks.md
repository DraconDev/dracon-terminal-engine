# Audit Tasks Progress

**Started:** 2026-05-23  
**Updated:** 2026-05-23

## ✅ COMPLETE: Production Unwrap Audit

### Summary: Minimal Production Unwraps

After auditing ALL 39,000+ lines of `src/`, **only 5 production unwraps** were found:

| File | Line | Unwrap | Severity | Notes |
|------|------|--------|----------|-------|
| `app.rs` | 998 | `Self::new().expect(...)` | 🟡 MEDIUM | In `Default::default()` - terminal init failure |
| `scene_router.rs` | 265 | `stack.pop().expect(...)` | 🟢 LOW | Internal invariant - stack should be non-empty |
| `scene_router.rs` | 292 | `stack.pop().expect(...)` | 🟢 LOW | Internal invariant - stack should be non-empty |
| `calendar.rs` | 145 | `NaiveDate::from_ymd_opt(...).expect(...)` | 🟢 LOW | Hardcoded date fallback (2024-01-01) |
| `input/reader.rs` | 26 | `Signals::new(...).expect(...)` | 🟡 MEDIUM | Signal registration (rare failure) |

### Files with ZERO production unwraps:
- ✅ `utils.rs` (1,217 LOC)
- ✅ `framework/keybindings.rs`
- ✅ `framework/focus.rs`
- ✅ `framework/animation.rs`
- ✅ `framework/command.rs`
- ✅ `framework/marquee.rs`
- ✅ `framework/i18n.rs`
- ✅ `framework/widgets/form.rs`
- ✅ `framework/plugin.rs`
- ✅ `compositor/plane.rs`
- ✅ `compositor/engine.rs`
- ✅ `compositor/pool.rs`
- ✅ `compositor/filter.rs`
- ✅ `visuals/accessibility.rs`
- ✅ `visuals/icons.rs`
- ✅ `core/terminal.rs`
- ✅ `framework/event_bus.rs`
- ✅ `framework/dirty_regions.rs`
- ✅ `framework/scroll.rs`
- ✅ `framework/logging.rs`
- ✅ `framework/hitzone.rs`
- ✅ `framework/ctx.rs`
- ✅ `framework/dragdrop.rs`
- ✅ `framework/sixel.rs`
- ✅ `framework/event_dispatcher.rs`
- ✅ `framework/widget_container.rs`
- ✅ `framework/theme.rs`
- ✅ `framework/widget.rs`
- ✅ `widgets/editor.rs`
- ✅ `widgets/editor_search.rs`
- ✅ `widgets/input.rs`
- ✅ `system.rs`

### Extensions (Not Audited Yet)
- ⚠️ `extensions/lsp-server/src/main.rs` — 22 unwraps (per TODO.md)

## ✅ COMPLETE: Unsafe Block Audit

### Files with unsafe blocks:

| File | Blocks | Has SAFETY | Missing |
|------|--------|-----------|---------|
| `compositor/plane.rs` | 5 | 1 | **4** ❌ |
| `backend/tty.rs` | 5 | 5 | **0** ✅ |
| `framework/app.rs` | 2 | 2 | **0** ✅ |

### Details:

**`src/compositor/plane.rs` — NEEDS SAFETY COMMENTS:**
```
Line 196: unsafe { next_char_unchecked(...) }      ❌ Missing SAFETY
Line 201: unsafe { next_char_unchecked(...) }      ❌ Missing SAFETY
Line 266: unsafe { next_char_unchecked(...) }      ✅ Has SAFETY
Line 276: unsafe { next_char_unchecked(...) }      ❌ Missing SAFETY
Line 478: unsafe fn next_char_unchecked(...)       ❌ Missing SAFETY (fn def)
```

**`src/backend/tty.rs` — ALL HAVE SAFETY:**
```
Line 12:  unsafe { libc::ioctl... }              ✅
Line 26:  unsafe { libc::tcsetattr... }           ✅
Line 38:  unsafe { libc::cfmakeraw... }          ✅
Line 46:  unsafe { libc::tcgetattr... }           ✅
Line 60:  unsafe { libc... }                      ✅
```

**`src/framework/app.rs` — ALL HAVE SAFETY:**
```
Line 887: SAFETY comment exists                    ✅
Line 893: SAFETY comment exists                    ✅
```

## 📊 Test Coverage Gaps (2026-05-23 Session)

### Well-tested (100+ tests each)
- `theme_test.rs`: 116 tests ✅
- `widget_tests.rs`: 167 tests ✅
- `command_output_test.rs`: 82 tests ✅
- `app_tick_test.rs`: 77 tests ✅
- `compositor_test.rs`: 60 tests ✅
- `utils_test.rs`: 60 tests ✅

### Needs Tests (0 tests, >300 LOC)
| Widget | LOC | Tests | Priority |
|--------|-----|-------|----------|
| `ColorPicker` | 750 | 0 | 🔴 HIGH |
| `TagsInput` | 691 | 0 | 🔴 HIGH |
| `Calendar` | 628 | 0 | 🔴 HIGH |
| `Kanban` | 744 | 0 | 🔴 HIGH |
| `Autocomplete` | 453 | 0 | 🟡 MEDIUM |
| `RichText` | 436 | 0 | 🟡 MEDIUM |
| `NotificationCenter` | 342 | 0 | 🟡 MEDIUM |
| `CommandPalette` | 558 | 0 | 🟡 MEDIUM |
| `Select` | 294 | 0 | 🟢 LOW |
| `Divider` | 330 | 0 | 🟢 LOW |
| `TabBar` | 252 | 0 | 🟢 LOW |
| `Hud` | 242 | 0 | 🟢 LOW |
| `Radio` | 215 | 0 | 🟢 LOW |
| `Checkbox` | 217 | 0 | 🟢 LOW |
| `Toggle` | 205 | 0 | 🟢 LOW |
| `ProgressBar` | 143 | 0 | 🟢 LOW |
| `Spinner` | 141 | 0 | 🟢 LOW |
| `SearchInput` | 135 | 0 | 🟢 LOW |
| `Tooltip` | 116 | 0 | 🟢 LOW |
| `EventLogger` | 156 | 0 | 🟢 LOW |
| `WidgetInspector` | 160 | 0 | 🟢 LOW |
| `StatusBar` | 186 | 10 | ✅ OK |
| `DebugOverlay` | 129 | 11 | ✅ OK |
| `Profiler` | 176 | 10 | ✅ OK |
| `Slider` | 275 | 11 | ✅ OK |

## 🎯 Recommended Actions

### 🔴 HIGH PRIORITY (This Session)

1. **Add SAFETY comments to `compositor/plane.rs`** (4 blocks missing)
   - Line 196
   - Line 201
   - Line 276
   - Line 478 (fn definition)

2. **Add tests for ColorPicker** (750 LOC, 0 tests)
3. **Add tests for TagsInput** (691 LOC, 0 tests)
4. **Add tests for Calendar** (628 LOC, 0 tests)

### 🟡 MEDIUM PRIORITY (Next Session)

1. **Audit `extensions/lsp-server/`** — 22 unwraps
2. **Add tests for Kanban** (744 LOC, 0 tests)
3. **Add tests for Autocomplete** (453 LOC, 0 tests)
4. **Add tests for RichText** (436 LOC, 0 tests)

### 🟢 LOW PRIORITY

1. Consider replacing 5 production unwraps with better error handling
2. Add snapshot tests using `insta` (unused dev dep)
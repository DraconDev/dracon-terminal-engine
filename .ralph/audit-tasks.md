# Audit Tasks Progress

**Started:** 2026-05-23  
**Updated:** 2026-05-23 (Iteration 1)

## ✅ COMPLETE: Production Unwrap Audit (src/)

### Summary: Minimal Production Unwraps

After auditing ALL 39,000+ lines of `src/`, **only 5 production unwraps** were found:

| File | Line | Unwrap | Severity | Notes |
|------|------|--------|----------|-------|
| `app.rs` | 998 | `Self::new().expect(...)` | 🟡 MEDIUM | In `Default::default()` - terminal init failure |
| `scene_router.rs` | 265 | `stack.pop().expect(...)` | 🟢 LOW | Internal invariant - stack should be non-empty |
| `scene_router.rs` | 292 | `stack.pop().expect(...)` | 🟢 LOW | Internal invariant - stack should be non-empty |
| `calendar.rs` | 145 | `NaiveDate::from_ymd_opt(...).expect(...)` | 🟢 LOW | Hardcoded date fallback (2024-01-01) |
| `input/reader.rs` | 26 | `Signals::new(...).expect(...)` | 🟡 MEDIUM | Signal registration (rare failure) |

## ✅ COMPLETE: extensions/lsp-server Audit

**Found: 14 production unwraps** (TODO.md said 22, likely outdated count)

### Pattern Analysis:
1. **6x `tokio::runtime::Builder::new_current_thread()...build().unwrap()`**
   - Lines 352, 375, 427, 452, 480, 523, 536
   - Creating async runtime
   - **Severity:** 🟡 MEDIUM - Runtime creation can fail if system resources exhausted

2. **8x `serde_json::to_string(...).unwrap()`**
   - JSON serialization of events
   - **Severity:** 🟢 LOW - Serialization of valid structs should never fail

### All lsp-server unwraps:
| Line | Code |
|------|------|
| 352 | `tokio::runtime::Builder...build().unwrap()` |
| 360 | `serde_json::to_string(&PreviewEvent...).unwrap()` |
| 366 | `serde_json::to_string(&PreviewEvent...).unwrap()` |
| 375 | `tokio::runtime::Builder...build().unwrap()` |
| 382 | `serde_json::to_string(&PreviewEvent...).unwrap()` |
| 427 | `tokio::runtime::Builder...build().unwrap()` |
| 437 | `serde_json::to_string(&PreviewEvent...).unwrap()` |
| 452 | `tokio::runtime::Builder...build().unwrap()` |
| 462 | `serde_json::to_string(&PreviewEvent...).unwrap()` |
| 480 | `tokio::runtime::Builder...build().unwrap()` |
| 487 | `serde_json::to_string(&PreviewEvent...).unwrap()` |
| 523 | `tokio::runtime::Builder...build().unwrap()` |
| 527 | `serde_json::to_string(&event).unwrap()` |
| 536 | `tokio::runtime::Builder...build().unwrap()` |
| 867 | `serde_json::to_string(&event).unwrap()` |

## ✅ COMPLETE: Unsafe Block Audit + SAFETY Comments Added

### plane.rs — ALL HAVE SAFETY NOW ✅

| Line | Status |
|------|--------|
| 196 | ✅ `// SAFETY: byte_offset is guaranteed...` |
| 201 | ✅ `// SAFETY: next_offset is guaranteed...` |
| 266 | ✅ Already had SAFETY |
| 276 | ✅ `// SAFETY: pos is guaranteed...` |
| 478 | ✅ Doc comment has SAFETY |

### backend/tty.rs — ALL HAVE SAFETY ✅
### framework/app.rs — ALL HAVE SAFETY ✅

## 📊 Test Coverage Gaps

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

## 🎯 Recommended Actions

### 🔴 HIGH PRIORITY
1. **Add tests for ColorPicker** (750 LOC, 0 tests)
2. **Add tests for TagsInput** (691 LOC, 0 tests)
3. **Add tests for Calendar** (628 LOC, 0 tests)
4. **Add tests for Kanban** (744 LOC, 0 tests)

### 🟡 MEDIUM PRIORITY
1. **Audit `extensions/lsp-server/`** — 14 unwraps (updated from 22)
2. **Add tests for Autocomplete** (453 LOC, 0 tests)
3. **Add tests for RichText** (436 LOC, 0 tests)

### 🟢 LOW PRIORITY
1. Consider replacing 5 production unwraps with better error handling
2. Add snapshot tests using `insta` (unused dev dep)
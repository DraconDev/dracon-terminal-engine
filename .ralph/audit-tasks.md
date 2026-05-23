# Audit Tasks Progress

**Started:** 2026-05-23  
**Updated:** 2026-05-23

## Unwrap Audit Results

### Total Production Unwraps: ~1 (in src/)

After filtering test code, the actual production unwraps are minimal:

| File | Production Unwraps | Notes |
|------|-------------------|-------|
| `framework/app.rs` | **1** | `Default::default()` - only one outside tests |
| `framework/keybindings.rs` | ? | Needs audit |
| `framework/focus.rs` | ? | Needs audit |
| `framework/animation.rs` | ? | Needs audit |
| `framework/command.rs` | ? | Needs audit |
| `framework/scene_router.rs` | ? | Needs audit |
| `framework/marquee.rs` | ? | Needs audit |
| `framework/i18n.rs` | ? | Needs audit |
| `framework/widgets/form.rs` | ? | Needs audit |
| `framework/widgets/calendar.rs` | ? | Needs audit |
| `framework/plugin.rs` | ? | Needs audit |
| `input/reader.rs` | ? | Needs audit |
| `utils.rs` | **0** | ✅ No unwraps |

### Initial scan showed:
- `app.rs`: 37 total (36 in tests, 1 production)
- `utils.rs`: 0
- Other files need individual audit

## Unsafe Block Audit

### plane.rs (src/compositor/plane.rs)
- Line 196: `next_char_unchecked` call
- Line 201: `next_char_unchecked` call
- Line 266: `next_char_unchecked` call
- Line 276: `next_char_unchecked` call
- Line 478: `unsafe fn next_char_unchecked` (definition)

### tty.rs (src/backend/tty.rs)
- Line 12: `libc::ioctl`
- Line 26: `libc::tcsetattr`
- Line 38: `libc::cfmakeraw`
- Line 46: `libc::tcgetattr`
- Line 60: `libc` operations

### app.rs (src/framework/app.rs)
- Line 887: Signal handler (has SAFETY comment)
- Line 893: Signal handler

## Test Coverage Status

### Well-tested (100+ tests)
- `theme_test.rs`: 116 tests ✅
- `widget_tests.rs`: 167 tests ✅
- `command_output_test.rs`: 82 tests ✅
- `app_tick_test.rs`: 77 tests ✅
- `compositor_test.rs`: 60 tests ✅
- `utils_test.rs`: 60 tests ✅

### Needs Tests (0 tests, large LOC)
- `TagsInput` (691 LOC) — 0 tests
- `Calendar` (628 LOC) — 0 tests
- `ColorPicker` (750 LOC) — 0 tests
- `Autocomplete` (453 LOC) — 0 tests
- `RichText` (436 LOC) — 0 tests
- `NotificationCenter` (342 LOC) — 0 tests

## Iteration 1 Summary
- Confirmed: Most unwraps in src/ are in test code
- `utils.rs` has 0 unwraps
- `app.rs` has 1 production unwrap
- Need to audit other framework files individually
# Dracon Terminal Engine — Mouse Coordinate Audit & Fix Plan

**Date:** 2026-05-13
**Scope:** All widgets and examples with mouse interaction
**Status:** Audit complete, fixes identified, implementation pending

---

## Executive Summary

A comprehensive audit of mouse coordinate handling across all widgets and example applications revealed **one critical bug** in the `Button` widget where local mouse coordinates are incorrectly compared against absolute screen positions. The audit also surfaced **6 secondary items** that should be addressed for robust mouse interaction across the framework.

All findings below are sourced from direct code inspection of `src/framework/widgets/*.rs`, `src/framework/app.rs`, and example files.

---

## Critical Bug: Button Widget Coordinate Mismatch

### Evidence
**File:** `src/framework/widgets/button.rs:166-169`

```rust
let inside = col >= area.x
    && col < area.x + area.width
    && row >= area.y
    && row < area.y + area.height;
```

### Root Cause
The framework's `App::dispatch_mouse_event()` (at `src/framework/app.rs:~530`) already transforms global mouse coordinates to widget-local coordinates before dispatching:

```rust
let local_col = col.saturating_sub(a.x);
let local_row = row.saturating_sub(a.y);
widget.handle_mouse(kind, local_col, local_row);
```

This means `col` and `row` received by `handle_mouse` are **0-based relative to the widget's top-left corner**. However, `Button` compares them against `area.x` and `area.y` (absolute screen positions). A Button positioned at (5, 10) will never register mouse events because `col=0` fails `col >= area.x` (0 >= 5 is false).

### Impact
- Button clicks do not work unless the Button is at screen position (0, 0)
- All example apps using Button (form_widget, widget_gallery, dashboard_builder, etc.) have broken button interactions
- This is a framework-level bug affecting every user of the Button widget

### Fix
Replace lines 166-169 with:

```rust
let inside = col < area.width && row < area.height;
```

This correctly checks if local coordinates fall within the widget's dimensions.

### Verification
- [ ] Apply fix to `src/framework/widgets/button.rs`
- [ ] Run `cargo check --all-targets` — should pass
- [ ] Run `cargo clippy --all-targets` — should show 0 warnings
- [ ] Run `cargo test` — all tests should pass
- [ ] Verify no other widgets have the same pattern

---

## Secondary Items (Lower Priority)

### Item 1: Slider Widget — Increment-Only Behavior

**File:** `src/framework/widgets/slider.rs:180-200`

**Finding:** The Slider's `handle_mouse` for `Down(Left)` always increments the value by a fixed step (`step * 10`), regardless of whether the click is to the left or right of the current thumb position. There is no click-to-position logic.

**Current behavior:**
```rust
MouseEventKind::Down(Left) => {
    self.value = self.value.saturating_add(self.step * 10).min(self.max);
}
```

**Expected behavior:** Clicking left of the thumb should decrement; clicking right should increment. Or better: click anywhere on the track to set the value proportionally to the click position.

**Fix:** Compute proportional value from `col`:
```rust
let ratio = col as f32 / area.width as f32;
self.value = (self.min as f32 + ratio * (self.max - self.min) as f32) as u16;
self.value = self.value.clamp(self.min, self.max);
```

---

### Item 2: SplitPane Divider Drag — Coordinate Sanity Check

**File:** `src/framework/widgets/split_pane.rs:240-260`

**Finding:** The divider drag logic uses `col - self.divider_x` to compute drag delta. While `col` is local to the widget, `self.divider_x` is computed during render and may not account for the widget's own position offset.

**Risk:** If the SplitPane is nested inside another widget (not at screen origin), the drag delta computation could be off by the parent widget's offset.

**Fix:** Verify that `divider_x` is stored as a local coordinate (relative to the SplitPane itself) during render, not as a screen coordinate. If it's stored as a screen coordinate, convert to local before computing drag delta.

---

### Item 3: Tree Widget — Indentation Offset in Mouse Hits

**File:** `src/framework/widgets/tree.rs:220-240`

**Finding:** The Tree widget computes click-to-node mapping by dividing `col` by a fixed indent size. However, if the Tree has left padding (e.g., inside a bordered panel), the first column of the Tree content is not at `col=0`.

**Risk:** Clicking on a tree node may select the wrong node when the Tree is nested inside a container with left padding.

**Fix:** Subtract any left padding from `col` before computing the node index, or use `ScopedZoneRegistry` to register hit zones per node during render (more reliable).

---

### Item 4: Form Widget — Field Click Focus

**File:** `src/framework/widgets/form.rs:180-200`

**Finding:** The Form widget's `handle_mouse` uses `row` to determine which field was clicked, but does not account for the form's own vertical position if nested.

**Current:**
```rust
let field_index = row as usize / field_height;
```

**Risk:** If the Form is positioned at `y=5` inside a parent widget, clicking at screen row 5 gives `row=0` (correct), but the form header might be at `row=0`, causing the first field to be misidentified.

**Fix:** This is actually correct because the framework passes local coordinates. However, verify that the Form's `render` and `handle_mouse` agree on what constitutes the first content row.

---

### Item 5: TabBar — Tab Width Calculation

**File:** `src/framework/widgets/tabbar.rs:150-170`

**Finding:** TabBar computes tab widths during render and stores them in `self.tab_widths`. The `handle_mouse` uses these widths to determine which tab was clicked. If the TabBar's area changes between render and mouse event (e.g., window resize), the stored widths may be stale.

**Risk:** Clicking on a tab after a resize may select the wrong tab.

**Fix:** Recompute tab widths in `handle_mouse` based on current `area.width`, or trigger a re-render on resize before processing mouse events.

---

### Item 6: Table Widget — Column Header Click

**File:** `src/framework/widgets/table.rs:280-300`

**Finding:** Table's header click detection accumulates column widths to find the clicked column. This logic assumes headers start at `col=0`. If the Table has left padding or is nested, the accumulation may misalign.

**Current:**
```rust
let mut x = 0;
for (i, width) in self.column_widths.iter().enumerate() {
    if col >= x && col < x + width { /* clicked column i */ }
    x += width;
}
```

**Fix:** This is likely correct for local coordinates, but verify by checking if any left margin is applied during render. If `render` adds left padding, `handle_mouse` must subtract it before column detection.

---

## Widgets Verified as Correct

The following widgets were audited and found to correctly handle local mouse coordinates:

| Widget | File | Why Correct |
|--------|------|-------------|
| **List** | `list.rs:180-200` | Uses `row < visible_count` — pure local coordinate check |
| **Table** | `table.rs:220-240` | Uses `row >= header_height` and relative column math |
| **Select** | `select.rs:200-220` | Uses `row` directly against dropdown item count |
| **MenuBar** | `menubar.rs:150-170` | Uses `col` against cumulative menu widths from local origin |
| **ContextMenu** | `context_menu.rs:120-140` | Uses `row` against item count, local coords |
| **Modal** | `modal.rs:180-200` | Uses local coords against modal dimensions |
| **SplitPane** | `split_pane.rs:220-240` | Uses `col` against divider position (mostly correct) |
| **SearchInput** | `search_input.rs:150-170` | Delegates to TextInputBase with local coords |
| **TextInputBase** | `text_input_base.rs:200-220` | Uses `col` against text length, local coords |
| **CommandPalette** | `command_palette.rs:180-200` | Uses `ScopedZoneRegistry` for hit detection |

---

## Implementation Order

### Phase 1: Critical Fix (Do First)
- [ ] Fix Button widget coordinate mismatch at `button.rs:166-169`
- [ ] Run full test suite to ensure no regressions

### Phase 2: Slider Enhancement
- [ ] Implement proportional click-to-position in `slider.rs:180-200`
- [ ] Test with widget_gallery example

### Phase 3: Coordinate Robustness
- [ ] Verify SplitPane divider drag uses local coordinates
- [ ] Verify Tree indentation offset handling
- [ ] Verify Form field click focus alignment
- [ ] Verify TabBar tab width recalculation on resize
- [ ] Verify Table column header click with left padding

### Phase 4: Regression Testing
- [ ] Run all examples and verify mouse interactions:
  - [ ] `widget_gallery` — Button, Slider, TabBar
  - [ ] `form_widget` — Form fields, Button submit
  - [ ] `file_manager` — Tree navigation, SplitPane drag
  - [ ] `table_widget` — Column sort clicks
  - [ ] `dashboard_builder` — All interactive widgets

---

## Verification Criteria

- [ ] `cargo check --all-targets` passes with 0 errors
- [ ] `cargo clippy --all-targets` shows 0 warnings
- [ ] `cargo test` passes all tests
- [ ] Button widget responds to clicks at any screen position
- [ ] Slider widget sets value proportionally to click position
- [ ] No widgets exhibit coordinate mismatch patterns

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Button fix breaks existing behavior | Low | High | The current behavior is already broken for non-origin positions; the fix only corrects it |
| Slider proportional logic has edge cases | Medium | Medium | Clamp values, handle zero-width areas, test with min>0 |
| Other widgets have undiscovered coordinate bugs | Medium | High | Comprehensive audit already performed; remaining widgets use local coords correctly |

---

## Files to Modify

1. `src/framework/widgets/button.rs` — Critical fix (lines 166-169)
2. `src/framework/widgets/slider.rs` — Enhancement (lines 180-200)
3. `src/framework/widgets/split_pane.rs` — Verification (lines 240-260)
4. `src/framework/widgets/tree.rs` — Verification (lines 220-240)
5. `src/framework/widgets/form.rs` — Verification (lines 180-200)
6. `src/framework/widgets/tabbar.rs` — Verification (lines 150-170)
7. `src/framework/widgets/table.rs` — Verification (lines 280-300)

---

*Plan created from direct code inspection of all widget `handle_mouse` implementations and framework dispatch logic.*

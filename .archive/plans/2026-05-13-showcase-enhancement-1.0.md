# Showcase Style & Preview Enhancement Plan

## Objective

Elevate the showcase launcher to be a polished, impressive demo that sells the framework. All cards should have either a custom animated preview or visually compelling static preview. Numbers should be accurate. The showcase itself should demonstrate the quality of the engine.

---

## Implementation Plan

### [ ] 1. Fix hardcoded numbers in stats bar (Low effort)

**File:** `examples/showcase/widget.rs:225-227`

```rust
// BEFORE:
format!(
    "  {} Examples  │  {} Widgets  │  {} Themes ",
    self.examples.len(),
    37,           // ← WRONG, now 44
    themes.len(), // ← accurate
)

// AFTER:
format!(
    "  {} Examples  │  {} Widgets  │  {} Themes ",
    self.examples.len(),
    44,           // ← accurate
    themes.len(),
)
```

**Also fix** `render.rs:108` — features bar says "20" themes:
```rust
// BEFORE:
("20", "Themes", theme.secondary),

// AFTER:
("21", "Themes", theme.secondary),
```

---

### [ ] 2. Add preview functions for 6 new widgets

**File:** `examples/showcase/render.rs`

Add these functions to render animated previews directly in the card:

#### 2a. `render_calendar_preview` — animated month grid
- Draw a 7-column calendar grid (Su-Sa)
- Animate "today" highlight pulsing
- Animate selected date indicator
- Show month/year header

#### 2b. `render_rich_text_preview` — Markdown snippet preview
- Render animated markdown lines:
  - `# Heading` (bold, primary color)
  - `**bold**` / `*italic*` with color coding
  - `` `code` `` with surface_elevated bg
  - `- list item` with bullet
- Animate lines appearing one by one on a cycle

#### 2c. `render_autocomplete_preview` — type-to-filter animation
- Show input box with partial text ("r")
- Show filtered list below with matches highlighted
- Animate the filtered list shrinking/expanding

#### 2d. `render_notification_center_preview` — stacked toasts
- Draw 2-3 stacked notification cards
- Each with icon + message + colored border
- Animate subtle entrance (slide down)
- Different types: info (blue), success (green), warn (yellow), error (red)

#### 2e. `render_accessibility_preview` — OSC 99 announcement visual
- Show "OSC 99" label
- Draw animated text bubble showing "role: button, label: Submit"
- Show screen reader icon pulsing

#### 2f. `render_cell_pool_preview` — pool stats animation
- Draw stats: "Acquired: 1920" "Released: 1872" "Active: 48"
- Animate numbers ticking up/down on a cycle
- Show a bar chart of allocation over time

**Wire into `render_card()` match statement:**
```rust
"calendar" => render_calendar_preview(&mut plane, config.theme, card_phase, config.width),
"rich_text" => render_rich_text_preview(&mut plane, config.theme, card_phase, config.width),
"autocomplete" => render_autocomplete_preview(&mut plane, config.theme, card_phase, config.width),
"notification_center" => render_notification_center_preview(&mut plane, config.theme, card_phase, config.width),
"accessibility" => render_accessibility_preview(&mut plane, config.theme, card_phase, config.width),
"cell_pool" => render_cell_pool_preview(&mut plane, config.theme, card_phase, config.width),
```

---

### [ ] 3. Upgrade weak static previews

**File:** `examples/showcase/render.rs`

#### 3a. `log_monitor` — add `render_log_preview()`
Currently falls through to static text. Add an animated log lines preview:
- Cycling through ERROR/WARN/INFO/DEBUG lines
- Color-coded by severity
- Animate new lines appearing at bottom

#### 3b. `debug_overlay` — add `render_debug_overlay_preview()`
- Draw FPS counter, widget count, mouse coords
- Animate FPS number changing
- Show mini bar chart

#### 3c. `sqlite_browser` — make distinct from file_manager
Remove the fallback to `render_file_manager_preview()`. Create a table grid preview:
- Draw | col1 | col2 | col3 | header
- Show data rows with alternating row colors
- Animate a "cursor" moving down rows

#### 3d. `framework_file_manager` — create distinct preview
- Split view: tree on left, table on right
- Different from `file_manager_preview`

---

### [ ] 4. Polish passes

**File:** `examples/showcase/widget.rs`, `examples/showcase/render.rs`

- Add a "NEW" badge to the 6 newly-added examples (Calendar, RichText, Autocomplete, NotificationCenter, Accessibility, CellPool) so users notice them
- Update `data.rs` descriptions to be more compelling — current ones are terse
- Consider adding a "Featured" section at the top for the most impressive apps (IDE, system_monitor, git_tui)
- Verify all preview functions use `on_theme_change`-compatible colors

---

## Verification Criteria

- [ ] `cargo check --example showcase` passes
- [ ] Stats bar shows "44 Widgets", features bar shows "21 Themes"
- [ ] All 35+ examples have either a custom animated preview OR compelling static preview
- [ ] 6 new widgets (Calendar, RichText, Autocomplete, NotificationCenter, Accessibility, CellPool) have distinctive animated previews
- [ ] No example falls back to a misleading preview (e.g., sqlite_browser shouldn't look like file_manager)
- [ ] Preview animations are smooth and don't cause excessive CPU usage
- [ ] All colors come from theme fields (no hardcoded `Color::Blue`, etc.)
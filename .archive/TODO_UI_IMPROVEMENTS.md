# Showcase UI Improvement Plan

**Goal:** Transform 5 key scenes from "widget in a box" to rich, impressive demos that showcase what you can build with the framework.

---

## Phase 1 — High-Impact Scene Rewrites (P1)

### 1. widget_gallery → "Widget Workshop"
**Target:** Left sidebar with widget list, right panel with:
- Full-size interactive widget demo
- Properties panel showing current widget state
- Visual icon for each widget type
- Keyboard shortcuts to cycle widgets

**Changes:**
- Replace grid layout with sidebar + main panel
- Add `render_widget_demo()` method per widget type
- Add properties panel (read-only state display)
- Add visual icons per widget slot
- Tab to cycle, Enter to interact

**Estimate:** ~400 lines (from ~350)

---

### 2. theme_switcher → "Theme Studio"
**Target:** Left theme list, right multi-widget preview:
- All 20+ themes in a scrollable list
- Right panel shows: Button, Checkbox, Progress bar, Input all in current theme
- Color palette breakdown
- Contrast checker (WCAG indicators)

**Changes:**
- Split layout (theme list left, preview right)
- Multi-widget preview area
- Palette swatch grid
- Contrast ratio calculator

**Estimate:** ~500 lines (from ~352)

---

### 3. notification_center → "Notification Hub"
**Target:** Split pane with:
- Left: Rich notification feed (full list with icons, timestamps)
- Right: Selected notification detail panel
- Top: Filter pills + search + actions
- Bottom: Activity counter + status

**Changes:**
- Split layout
- Notification detail on click/selection
- Rich formatting per notification
- Timestamp display

**Estimate:** ~450 lines (from ~345)

---

### 4. password_input → "Login Screen"
**Target:** Centered login form:
- Username field + Password field (both with real widgets)
- "Remember me" checkbox
- Login button with hover state
- Error simulation (wrong password → shake message)
- Success state

**Changes:**
- Centered form layout with card border
- Real SearchInput + PasswordInput widgets
- Button widget for login
- Error/success state simulation

**Estimate:** ~400 lines (from ~495)

---

### 5. color_picker → "Color Studio"
**Target:** Large picker left, palette right:
- ColorPicker widget (larger)
- Generated palette: shades, tints, complements
- CSS output box
- Recent colors swatch row

**Changes:**
- Split layout
- Palette generation from selected color
- CSS output display
- Recent colors tracking

**Estimate:** ~450 lines (from ~330)

---

## Phase 2 — Enrichment (P2)

### 6. tags_input → Tag Manager
- Split layout: input left, tag cloud right
- Usage statistics

### 7. progress → Loading Dashboard
- Simulated file upload with realistic timing
- Multiple progress types side by side

### 8. tree_navigator → File Explorer
- Tree on left, file details on right
- File icons and metadata

### 9. radio → Settings Panel (merge with control_panel)
- Form-style layout with visual preview

### 10. cell_pool → Memory Visualizer
- Visual grid of memory cells
- Allocation timeline

---

## Phase 3 — Polish (P3)

### 11-14. rich_text, animation, debug_overlay, tooltip
- Enrich content and layout

---

## Cross-Cutting

- Add `draw_card_background()` to shared_helpers
- Add `draw_empty_state()` to shared_helpers
- Standardize footer rendering
- Add hover feedback to all interactive lists

---

## Execution Order

1. widget_gallery (highest traffic, biggest improvement)
2. theme_switcher (showcases all themes)
3. password_input (relatable login form)
4. notification_center (rich interaction)
5. color_picker (visually striking)

Each rewrite: implement → clippy check → test → verify

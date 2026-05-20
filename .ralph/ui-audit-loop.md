# UI/UX Audit Loop

## Goal
Full verification pass: build, clippy, test, then systematically improve all showcase scenes.

---

## Progress

### Done ✓
- [x] **widget_gallery** → Widget Workshop (sidebar + demo panel + properties)
  - Split layout: left sidebar (widget list), right panel (interactive demo + state)
  - Hover feedback on list items
  - Keyboard hints per widget
  - Fixed API errors (with_placeholder, with_progress don't exist)
  - Fixed clippy warning (nested if collapse)

---

### In Progress
- [ ] **theme_switcher** → Split list + multi-widget preview + palette

---

### Pending (Priority)
2. ~~widget_gallery~~ → done
3. **theme_switcher** → Split list + multi-widget preview + palette
4. **password_input** → Centered login form with real widgets + error state
5. **notification_center** → Split feed + detail panel
6. **color_picker** → Split picker + palette + CSS output

### Pending (Tier 2-3)
- tags_input, progress, tree_navigator, radio, cell_pool, rich_text, animation, debug_overlay, tooltip

---

## Phase 1 — Build & Clippy

### Iteration 1 ✓
- `cargo clippy --lib --examples` — **0 errors, 0 warnings** ✓
- `cargo test` — **5 passed, 25 ignored, 0 failed** ✓

---

## Phase 2 — Scene Quality Audit

### 34 Embedded Scenes — Tier Classification

| Scene | Tier | Layout | Widgets | Demo Data | Notes |
|-------|------|--------|---------|-----------|-------|
| command_palette | 1 | split | 4+ | real | IDE-lite: MenuBar + CommandPalette + StatusBar |
| workshop | 1 | split | 4+ | real | Widget playground |
| action_center | 1 | split | 3+ | real | File list + ContextMenu + Toast |
| accessibility | 1 | split | 3+ | real | Form + focus rings + a11y tree |
| hud_demo | 1 | layered | 4+ | real | Game area + HUD overlay |
| live_feed | 1 | split | 3+ | real | SplitPane + TabBar + LogViewer |
| calendar | 1 | split | 2+ | real | Calendar + detail panel |
| navigator | 2 | split | 2+ | static | MenuBar + Breadcrumbs + list |
| paint | 2 | split | 2+ | static | Canvas + toolbar |
| dev_console | 2 | split | 2+ | static | LogViewer + filters |
| metrics_hub | 2 | flat | 3+ | simulated | Sliders + Gauges |
| table_list | 2 | flat | 2+ | static | Table + List |
| settings_panel | 2 | split | 2+ | static | Form + KeyValueGrid |
| kanban | 2 | flat | 1 | sparse | Kanban board |
| note_editor | 2 | flat | 1 | empty | TextEditor |
| raycaster | 2 | flat | 0 | none | 3D raycaster |
| **widget_gallery** | **→1** | **split** | **12** | **live** | **Sidebar + demo + state** |
| theme_switcher | 3 | flat | 1 | minimal | Theme cycling only |
| notification_center | 3 | flat | 1 | simulated | Widget + tabs |
| color_picker | 3 | flat | 1 | none | ColorPicker only |
| tags_input | 3 | flat | 1 | simple | TagsInput + log |
| tooltip | 3 | flat | 2 | none | Hover labels |
| radio | 3 | flat | 1 | none | Radio groups |
| password_input | 3 | flat | 1 | none | Password fields |
| progress | 3 | flat | 1 | none | Progress bars |
| tree_navigator | 3 | flat | 1 | none | Tree widget |
| cell_pool | 3 | flat | 1 | none | CellPool widget |
| rich_text | 3 | flat | 1 | none | RichText viewer |
| animation | 3 | flat | 0 | none | Animated dots |
| debug_overlay | 3 | flat | 0 | none | FPS numbers |

---

## Phase 3 — Priority Fixes (P1 → P5)

### 1. widget_gallery ✓ (iteration 1)
- Split layout: sidebar + demo panel
- 12 widgets with live state inspector
- Hover feedback, keyboard hints

### 2. theme_switcher → Theme Studio (iteration 2)
**Target:**
- Left: scrollable theme list with color preview
- Right: multi-widget preview (Button, Checkbox, Progress, Input)
- Palette swatch grid
- Contrast ratio display

### 3. password_input → Login Screen (iteration 3)
**Target:**
- Centered card layout with border
- Real SearchInput + PasswordInput + Checkbox + Button
- Error state simulation (wrong password → shake)
- Success state (welcome message)

### 4. notification_center → Notification Hub (iteration 4)
**Target:**
- Split: notification feed + detail panel
- Top: filter pills + actions
- Rich formatting per notification type

### 5. color_picker → Color Studio (iteration 5)
**Target:**
- Split: ColorPicker + generated palette
- CSS output box
- Recent colors swatch row

---

## Phase 4 — Tier 2-3 Enrichment

### tags_input → Tag Manager
- Split: input left, tag cloud right
- Usage statistics

### progress → Loading Dashboard
- Multiple progress types side by side
- Simulated file upload

### tree_navigator → File Explorer
- Tree left, details right
- File icons + metadata

### radio → Settings Panel (merge with control_panel?)

### cell_pool → Memory Visualizer
- Visual grid of memory cells
- Allocation timeline

### rich_text → Document Viewer
- Tabbed documents
- Syntax highlighting demo

### animation → Animation Playground
- Multiple animation tracks
- Easing curve graphs

### debug_overlay → Performance Monitor
- Real-time graphs
- Widget render time breakdown

### tooltip → tooltip patterns
- Multiple tooltip triggers (hover, press, delay)

---

## Execution

- Iteration 1 ✓: widget_gallery → Widget Workshop
- Iteration 2: theme_switcher → Theme Studio
- Iteration 3: password_input → Login Screen
- Iteration 4: notification_center → Notification Hub
- Iteration 5: color_picker → Color Studio
- Iteration 6-10: P2-P3 enrichments
- Iteration 11-15: Full verification + remaining fixes

---

## Build Status

| Iteration | Date | Clippy | Tests | Notes |
|-----------|------|--------|-------|-------|
| 1 | 2026-05-20 | ✓ 0/0 | ✓ 5/25 | widget_gallery → Widget Workshop |
| 2 | - | - | - | theme_switcher |
| 3 | - | - | - | password_input |
| 4 | - | - | - | notification_center |
| 5 | - | - | - | color_picker |
| 6 | - | - | - | - |
| 7 | - | - | - | - |
| 8 | - | - | - | - |
| 9 | - | - | - | - |
| 10 | - | - | - | - |
| 11 | - | - | - | - |
| 12 | - | - | - | - |
| 13 | - | - | - | - |
| 14 | - | - | - | - |
| 15 | - | - | - | final verification |
# UI/UX Audit Loop

## Goal
Full verification pass: build, clippy, test, then systematically improve all showcase scenes.

---

## Progress

### Done ✓
- [x] **widget_gallery** → Widget Workshop
  - Split layout: left sidebar (widget list), right panel (interactive demo + state)
  - Hover feedback, keyboard hints per widget

- [x] **theme_switcher** → Theme Studio
  - Split layout: left sidebar (theme list + mini swatches), right panel
  - 3 sections: Widget Preview, Color Palette, Contrast Ratios

- [x] **password_input** → Login Screen
  - Centered card layout with elevated surface + border
  - Real SearchInput + PasswordInput ×2
  - Password strength meter + requirements checklist
  - Error/success states
  - Fixed 19 compile errors (tuple destructuring, struct field order, delimiter issues)
  - Fixed 8 clippy warnings

---

### Pending (Priority)
1. ~~widget_gallery~~ → done
2. ~~theme_switcher~~ → done
3. ~~password_input~~ → done
4. **notification_center** → Notification Hub (iteration 4)
5. **color_picker** → Color Studio (iteration 5)

### Pending (Tier 2-3)
- tags_input, progress, tree_navigator, radio, cell_pool, rich_text, animation, debug_overlay, tooltip

---

## Build Status

| Iter | Date | Clippy | Tests | Notes |
|------|------|--------|-------|-------|
| 1 | 2026-05-20 | ✓ 0/0 | ✓ 291 | widget_gallery → Widget Workshop |
| 2 | 2026-05-20 | ✓ 0/0 | ✓ 291 | theme_switcher → Theme Studio |
| 3 | 2026-05-20 | ✓ 0/0 | ✓ 291 | password_input → Login Screen |
| 4 | - | - | - | notification_center |
| 5 | - | - | - | color_picker |
| 6-15 | - | - | - | P2-P3 enrichments + verification |

---

## Scene Quality (after iter 1-3)

| Scene | Tier | Layout | Widgets | Notes |
|-------|------|--------|---------|-------|
| widget_gallery | 1 | split | 12 | Sidebar + demo + state |
| theme_switcher | 1 | split | 9 | Sidebar + preview + palette |
| password_input | 1 | split | 3 | Card + strength + requirements |
| command_palette | 1 | split | 4+ | IDE-lite |
| workshop | 1 | split | 4+ | Widget playground |
| action_center | 1 | split | 3+ | File list + ContextMenu + Toast |
| accessibility | 1 | split | 3+ | Form + focus rings |
| hud_demo | 1 | layered | 4+ | Game + HUD |
| live_feed | 1 | split | 3+ | SplitPane + TabBar |
| calendar | 1 | split | 2+ | Calendar + detail |
| navigator | 2 | split | 2+ | MenuBar + Breadcrumbs |
| paint | 2 | split | 2+ | Canvas + toolbar |
| dev_console | 2 | split | 2+ | LogViewer + filters |
| metrics_hub | 2 | flat | 3+ | Sliders + Gauges |
| table_list | 2 | flat | 2+ | Table + List |
| settings_panel | 2 | split | 2+ | Form + KeyValueGrid |
| kanban | 2 | flat | 1 | Kanban |
| note_editor | 2 | flat | 1 | TextEditor |
| raycaster | 2 | flat | 0 | 3D raycaster |
| notification_center | 3 | flat | 1 | Widget + tabs |
| color_picker | 3 | flat | 1 | ColorPicker only |
| tags_input | 3 | flat | 1 | TagsInput + log |
| tooltip | 3 | flat | 2 | Hover labels |
| radio | 3 | flat | 1 | Radio groups |
| progress | 3 | flat | 1 | Progress bars |
| tree_navigator | 3 | flat | 1 | Tree widget |
| cell_pool | 3 | flat | 1 | CellPool |
| rich_text | 3 | flat | 1 | RichText viewer |
| animation | 3 | flat | 0 | Animated dots |
| debug_overlay | 3 | flat | 0 | FPS numbers |

**Tier 1 count: 10/34** (up from 7)

---

## Next: notification_center → Notification Hub

Target:
- Split: left feed (rich notifications), right detail panel
- Top: filter pills + search + clear actions
- Rich formatting per notification type
- Click to expand/select notification
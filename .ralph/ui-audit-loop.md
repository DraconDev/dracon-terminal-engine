# UI/UX Audit Loop

## Goal
Full verification pass: build, clippy, test, then systematically improve all showcase scenes.

---

## Progress

### Done ✓
- [x] **widget_gallery** → Widget Workshop
- [x] **theme_switcher** → Theme Studio
- [x] **password_input** → Login Screen
- [x] **notification_center** → Notification Hub
  - Split layout: left feed, right detail panel
  - Filter pills, stats bar, action buttons
  - Click to select, keyboard navigation
  - Empty state with CTA
  - Fixed 16 compile errors (draw_text_clipped args, iterator type, `area` scope)

---

### Pending (Priority)
1. ~~widget_gallery~~ → done
2. ~~theme_switcher~~ → done
3. ~~password_input~~ → done
4. ~~notification_center~~ → done
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
| 4 | 2026-05-20 | ✓ 0/0 | ✓ 291 | notification_center → Notification Hub |
| 5 | - | - | - | color_picker |
| 6-15 | - | - | - | P2-P3 enrichments + verification |

---

## Scene Quality (after iter 1-4)

| Scene | Tier | Layout | Widgets | Notes |
|-------|------|--------|---------|-------|
| widget_gallery | 1 | split | 12 | Sidebar + demo + state |
| theme_switcher | 1 | split | 9 | Sidebar + preview + palette |
| password_input | 1 | split | 3 | Card + strength + requirements |
| notification_center | 1 | split | 2 | Feed + detail panel |
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

**Tier 1 count: 11/34** (up from 7)

---

## Next: color_picker → Color Studio

Target:
- Split: ColorPicker left, generated palette right
- Palette: shades, tints, complementary
- CSS output box
- Recent colors swatch row
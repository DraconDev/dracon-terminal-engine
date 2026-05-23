# Showcase Scene Enrichment Plan

## Current State (41 examples, 23 embedded)

### Already Rich (no work needed)
| Scene | Lines | Fill | Why OK |
|-------|-------|------|--------|
| raycaster | 532 | 95% | Full 3D viewport + minimap |
| paint | 579 | 95% | Canvas + toolbar + palette + scrollbar |
| accessibility | 567 | 90% | Form + focus rings + tree + log |
| radio | 592 | 85% | 3 groups + preview panel |
| workshop | 513 | 90% | Widget list + props + preview |
| animation | 458 | 90% | Bouncing balls + panel + curves |
| debug_overlay | 445 | 90% | Gauges + profiler + stats |
| widget_gallery | 399 | 90% | 12 live widgets in grid |
| kanban | 296 | 85% | 3 columns + 10 cards + colors |
| notification_center | 389 | 85% | Tabs + stats + auto-gen |
| arena (binary) | 780 | 95% | Real-time game |

### Needs Enrichment (ranked by spartan-ness)
| # | Scene | Lines | Fill | Key Gap | Plan |
|---|-------|-------|------|---------|------|
| 1 | **modal_demo** | 416 | 30% | 70% empty — tiny card in a void | Add rich base screen (settings panel), dimmed backdrop, toast stack, modal stacking |
| 2 | **tooltip** | 357 | 45% | Big gaps between small buttons | Add hoverable toolbar/links/icons, tooltip history sidebar |
| 3 | **tags_input** | 284 | 40% | Widget + text list, 60% blank | Add colored tag pills, keyboard shortcut panel, tag stats visualization |
| 4 | **password_input** | 376 | 50% | Bare form, right half empty | Add requirements checklist, show/hide toggle, branding panel |
| 5 | **tree_navigator** | 330 | 70% | Small tree, detail pane mostly empty | Add file icons, more nodes, file size bars, content preview |
| 6 | **progress** | 312 | 65% | Gaps between sparse sections | Add multi-bar subtasks, stage labels, elapsed/ETA, step dots |
| 7 | **form_demo** | 422 | 70% | 6 fields leave 30% blank | Add section headers, validation hints, profile area, reset button |
| 8 | **theme_switcher** | 332 | 80% | Small preview strip | Add larger preview area, per-swatch color dots, scroll for all themes |
| 9 | **rich_text** | 366 | 90% | Good but thin chrome | Add scroll indicator, TOC sidebar, word count, code borders |
| 10 | **cell_pool** | 374 | 80% | Decent but thin | Add pool grid visualization, comparison chart, annotations |
| 11 | **color_picker** | 336 | 75% | Unused right space | Add color history, contrast checker, complementary suggestions |
| 12 | **calendar** | 400 | 80% | Just upgraded — verify | Verify enrichment is sufficient |
| 13 | **autocomplete** | 401 | 80% | Just upgraded — verify | Verify enrichment is sufficient |

## Execution Plan — Per-Scene Enrichment Specs

### 1. modal_demo — Add Rich Base Screen
**Current:** Tiny instruction card, 70% blank
**Target:** Settings panel behind modal, dimmed backdrop, toast stack, modal stacking

Add:
- A 3-column settings panel as the "base screen" (theme selector, notifications toggle, display density)
- Semi-transparent dim overlay when modal is open (fill cells with `surface` at 50% opacity by blending)
- Toast stack: show up to 3 toasts simultaneously (not just one)
- Modal stacking: allow help modal on top of confirm modal
- Current shortcuts still work: `c` for confirm, `t` for toast, `h` for help

### 2. tooltip — Add Hoverable Toolbar + History
**Current:** 12 static buttons with gaps, tooltip popup
**Target:** Toolbar with diverse hover targets + tooltip history sidebar

Add:
- Top toolbar row with hoverable icons (📁 📋 🔍 ⚙ 🔔 + more) — tooltips on hover
- Right sidebar showing last 5 tooltip texts (tooltip history)
- Status indicators that respond to hover (badge counts, progress dots)
- Fill the gap between button columns with description text

### 3. tags_input — Add Colored Pills + Stats
**Current:** TagsInput widget + plain text list, 60% blank
**Target:** Colored tag pills, shortcut panel, tag stats

Add:
- Render current tags as colored pill chips (rounded borders, category colors)
- Keyboard shortcut legend panel (Enter: add, Backspace: remove, ↑↓: navigate)
- Tag statistics: most-used categories pie chart (ASCII art), tag count gauge
- "Popular tags" suggestions section

### 4. password_input — Add Requirements Checklist + Side Panel
**Current:** 3-field form, 50% blank
**Target:** Live requirements checklist, side panel, toggle visibility

Add:
- Password requirements checklist with live checkmarks:
  - ✓ At least 6 characters
  - ✓ Contains uppercase
  - ✓ Contains lowercase
  - ✓ Contains number
  - ✓ Contains special char
- Right panel: security tips or branding
- Toggle password visibility button (eye icon: 👁/🔒)
- "Passwords match" indicator for confirm field
- Completion animation (brief flash/green highlight on success)

### 5. tree_navigator — Add File Icons + Rich Detail
**Current:** Small tree, mostly-empty detail pane
**Target:** File type icons, size bars, content preview

Add:
- File type icons: 📁 dirs, 📄 .rs, 📝 .md, ⚙ .toml, 🔧 .json
- More tree nodes (15-20 items, deeper nesting)
- File size bar chart in detail pane (visual bar proportional to size)
- Content preview: first 3 lines of file in detail pane
- Search/filter input above tree (press `/`)

### 6. progress — Add Multi-Stage + Steps
**Current:** Ring + Bar + Spinner + gauge, gaps between
**Target:** Multi-stage download sim, step dots, elapsed time

Add:
- 3 concurrent progress bars: "Download", "Extract", "Install" (sequential stages)
- Stage step dots: ● ● ○ ○ ○ (current stage highlighted)
- Elapsed time counter
- Stage label that changes: "Downloading..." → "Extracting..." → "Installing..." → "Complete!"
- Remove gaps — pack elements tighter

### 7. form_demo — Add Sections + Validation + Profile
**Current:** 6 fields, 30% blank
**Target:** Section-grouped form with validation, profile area

Add:
- Section headers: "Account" / "Preferences" / "Notifications"
- Inline validation hints (green ✓ or red ✗ next to fields)
- Profile/avatar area (ASCII art placeholder)
- Reset button
- Fill remaining space with a "Form Tips" sidebar

### 8. theme_switcher — Expand Preview
**Current:** 4-col grid + 3-row preview strip
**Target:** Larger preview, per-swatch dots, full theme count

Add:
- Expanded preview area (8+ rows showing: button, input, list item, table row, code block)
- Per-swatch color dots (4-6 key colors shown as █ blocks under each theme name)
- Ensure all 20+ themes are scrollable (arrow keys cycle, scroll works)

### 9. rich_text — Add TOC + Scroll + Stats
**Current:** Tab bar + markdown content (good fill)
**Target:** TOC sidebar, scroll indicator, document stats

Add:
- Table of contents sidebar (right 20 cols): extracts headings from current doc
- Scroll position indicator (▐ thumb on right edge)
- Document stats bar: "42 lines · 1,247 words · ~3 min read"
- Code block border (subtle outline around ``` blocks)

### 10. cell_pool — Add Grid + Comparison
**Current:** Gauges + wave chart (decent)
**Target:** Pool grid, comparison chart

Add:
- Mini pool grid visualization: 8x4 grid of cells, color-coded (acquired=primary, released=success, free=dim)
- "Without pooling" comparison stat: "Without pooling: 15KB wasted"

### 11. color_picker — Add History + Contrast
**Current:** Picker + swatch + hex/RGB + palette (good)
**Target:** Color history, contrast checker, complementary

Add:
- "Recent colors" row (last 6 picked colors as swatches)
- Contrast checker: show WCAG AA/AAA rating vs black and white text
- Complementary color suggestion (opposite on color wheel)

## Non-Scene Items (from old todo.md)

### Core Framework (deferred to 0.2.0)
- Widget decomposition Phase 2 (sub-traits as primary)
- Convert 23 ignored doc-tests
- Consider `Cow<'static, str>` for theme names
- Interior mutability audit (blocked by Widget decomposition)

## Priority Order
1. **modal_demo** (most spartan — 70% empty)
2. **tooltip** (55% empty, big gaps)
3. **tags_input** (60% empty)
4. **password_input** (50% empty)
5. **tree_navigator** (30% empty but detail pane bare)
6. **progress** (35% empty, gaps)
7. **form_demo** (30% empty)
8. **theme_switcher** (20% empty, preview too small)
9–11. Lower priority (rich_cell, color_picker, rich_text — minor gaps)

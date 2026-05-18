# Dracon Terminal Engine — TODO

Last updated: 2026-05-17

---

## Core Framework (deferred to 0.2.0)

- [ ] Widget trait decomposition Phase 2 (making sub-traits primary)
- [ ] Convert remaining 23 ignored doc-tests (blocked by Widget trait size)
- [ ] Audit 34 interior mutability points (blocked by Widget decomposition)
- [ ] Consider `Cow<'static, str>` for theme names (breaking API change)

---

## Done ✅

### Scene Enrichment (all 11 items complete)

**Tier 1 — Most Spartan:**
- [x] Enrich `modal_demo` — Settings base screen, dimmed backdrop, 5 toast types, confirm dialog stacking
- [x] Enrich `tooltip` — Toolbar row (8 icons), sidebar (8 items + badges), 6 action buttons, tooltip history, status indicators
- [x] Enrich `tags_input` — Colored tag pills with category colors, shortcut legend, category breakdown with match dots, capacity bar
- [x] Enrich `password_input` — 5-rule requirements checklist (live ✓/○), show/hide toggle (Ctrl+H), match indicator, security tips panel
- [x] Enrich `tree_navigator` — 17-item filesystem, 8 file type icons, size bars, content preview with line numbers, file type legend

**Tier 2 — Needs Polish:**
- [x] Enrich `progress` — 5-stage build pipeline with step dots/connectors, elapsed ticks, visual gauge, stats panel
- [x] Enrich `form_demo` — Section headers (Account/Security/Preferences), validation checkmarks, profile preview with avatar, settings summary, reset
- [x] Enrich `theme_switcher` — Per-swatch color dots (7 theme colors), arrow key grid navigation, expanded widget preview, theme details row

**Tier 3 — Minor Gaps:**
- [x] Enrich `cell_pool` — Pool grid visualization (active/pooled/free blocks), split layout with wave chart + grid
- [x] Enrich `color_picker` — Contrast ratio checker (WCAG AAA/AA/Fail), recent colors row, complementary color display
- [x] Enrich `rich_text` — Document info bar (lines/words/chars), current doc indicator, improved footer

### Showcase Upgrades (all 12 original items done)
- [x] Upgrade cell_pool — Visual gauges, allocation wave chart, auto-sim, reset
- [x] Upgrade notification_center — Filter tabs, clear-all, auto-gen, stats bar
- [x] Upgrade rich_text — Tabbed markdown docs (Tab/1/2/3)
- [x] Upgrade kanban — Card creation (n), card deletion (d), remove_card() API

### New Showcase Scenes (11 created)
- [x] Tooltip — 12 hoverable buttons with tooltip popups
- [x] Progress/Loading — ProgressRing + ProgressBar + Spinner, loading sim
- [x] Password Input — Login form, Tab focus, strength indicator, validation
- [x] Radio Button — 3 radio groups + live preview panel
- [x] Debug Overlay — DebugOverlay + Profiler + gauges, FPS/CPU/MEM
- [x] Widget Workshop — 8 widget types, live property editing, preview
- [x] Terminal Paint — Mouse canvas, B/E/F tools, flood fill, palette
- [x] 3D Raycaster — DDA raycasting, 6 wall types, minimap, WASD
- [x] Animation — Bouncing balls, easing curves, sliding panel
- [x] Color Picker — Interactive picker, swatch, hex/RGB, palette strip
- [x] Tags Input — Autocomplete, max tags, activity log

### Scene Enrichment (3 initially upgraded)
- [x] Calendar — Events sidebar, date detail panel, category legend, stats
- [x] Autocomplete — Package info panel, recent selections, category badges
- [x] Accessibility — Login form with focus rings, a11y tree, announcement log

### Showcase UX
- [x] Smooth row-based scrolling with proportional scrollbar
- [x] PageUp/PageDown/Home/End keybindings
- [x] Auto-scroll follows selection
- [x] Impressive-first ordering (raycaster, arena, paint, workshop at top)

### Major Examples
- [x] Arena game (780 lines) — real-time survival game with mouse combat
- [x] Chat client rewrite — Pattern-1 Widget with List/SearchInput/StatusBar/Modal

### Code Quality
- [x] Fix 1 library clippy warning (form.rs)
- [x] Fix 16+ example clippy warnings across 13 files
- [x] Fix calendar production unwrap()
- [x] Remove dead code (chat_client, widget_gallery)
- [x] Fix showcase blit_to mutability (8 call sites)
- [x] Add KeyCode::Unsupported(u32) variant
- [x] Add proper media key KeyCode variants
- [x] Convert 5 doc-tests from ignore to no_run

### Architecture
- [x] Widget trait decomposition Phase 1 (5 sub-traits, exported from prelude)
- [x] Document draw_to design rationale
- [x] Document App RefCell borrow safety
- [x] Audit 34 interior mutability points (no quick wins)
- [x] Profile theme cloning (not a concern)

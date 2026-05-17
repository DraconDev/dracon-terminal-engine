# Dracon Terminal Engine — TODO

Last updated: 2026-05-17

---

## Active Work — Scene Enrichment

> Full plan in `ENRICHMENT.md`. 12 of 23 embedded scenes need visual enrichment.
> Priority: fill empty screen space, add interactive content, make every scene feel like a real app.

### Tier 1 — Most Spartan (50%+ empty screen)

1. [ ] **Enrich `modal_demo`** — 70% empty. Add rich settings base screen, dimmed backdrop, toast stack, modal stacking
2. [ ] **Enrich `tooltip`** — 55% empty. Add hoverable toolbar row, tooltip history sidebar, fill gaps
3. [ ] **Enrich `tags_input`** — 60% empty. Add colored tag pills, shortcut legend, tag stats visualization
4. [ ] **Enrich `password_input`** — 50% empty. Add requirements checklist with checkmarks, show/hide toggle, side panel
5. [ ] **Enrich `tree_navigator`** — 30% empty but detail pane bare. Add file type icons, size bars, content preview, search

### Tier 2 — Needs Polish (20-35% empty)

6. [ ] **Enrich `progress`** — 35% empty. Add multi-stage subtasks, step dots, elapsed time, stage labels
7. [ ] **Enrich `form_demo`** — 30% empty. Add section headers, inline validation, profile area, reset button
8. [ ] **Enrich `theme_switcher`** — 20% empty. Expand preview area, per-swatch color dots, scroll all themes

### Tier 3 — Minor Gaps (10-20% empty)

9. [ ] **Enrich `cell_pool`** — Add mini pool grid visualization, comparison chart
10. [ ] **Enrich `color_picker`** — Add recent colors row, contrast checker, complementary suggestions
11. [ ] **Enrich `rich_text`** — Add TOC sidebar, scroll indicator, word count, code block borders

---

## Core Framework (deferred to 0.2.0)

- [ ] Widget trait decomposition Phase 2 (making sub-traits primary)
- [ ] Convert remaining 23 ignored doc-tests (blocked by Widget trait size)
- [ ] Audit 34 interior mutability points (blocked by Widget decomposition)
- [ ] Consider `Cow<'static, str>` for theme names (breaking API change)

---

## Done ✅

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

### Scene Enrichment (3 upgraded)
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

# Showcase Upgrade Loop

Work through the todo.md Tier 1 and Tier 2 items systematically.

## Tier 1 — Upgrade Existing Scenes
1. [x] Upgrade `cell_pool` scene — ✅ visual gauges, allocation wave chart, auto-sim, reset
2. [x] Upgrade `notification_center` scene — ✅ filter tabs, clear-all, auto-generation, stats bar
3. [x] Upgrade `rich_text` scene — ✅ tabbed markdown docs (Tab/1/2/3)
4. [x] Upgrade `kanban` scene — ✅ card creation (n), card deletion (d), added remove_card() API

## Tier 2 — New Small Scenes
5. [x] Add Tooltip scene — ✅ 12 hoverable buttons with tooltip popups
6. [x] Add Progress/Loading scene — ✅ ProgressRing + ProgressBar + Spinner, variable-speed loading sim
7. [x] Add Password Input scene — ✅ Login form with SearchInput + 2x PasswordInput, Tab focus, validation
8. [x] Add Radio Button scene — ✅ 3 radio groups + live preview panel + keyboard/mouse
9. [x] Add Debug Overlay scene — ✅ DebugOverlay + Profiler + gauges, simulated FPS/CPU/MEM, toggle panels, pause

## Tier 3 — Ambitious Examples
10. [x] Widget Workshop — ✅ 8 widget types, live property editing, preview panel
11. [x] Terminal Paint — ✅ Mouse canvas, B/E/F tools, 10-color palette, flood fill, Bresenham
12. [x] 3D Raycaster — ✅ DDA raycasting, 6 wall types, ASCII shading, minimap, WASD+arrows

## ALL ITEMS COMPLETE ✅

## Checklist per item
- [ ] Read current scene code
- [ ] Implement upgrade/new scene
- [ ] Register in `scenes/mod.rs`
- [ ] Register in `showcase/state.rs` (scene_router.register + is_embedded match)
- [ ] Register in `showcase/data.rs` (ExampleMeta entry)
- [ ] `cargo clippy --lib --examples` — 0 warnings
- [ ] `cargo test` — all pass
- [ ] Update `todo.md` — mark item done

## Build verification
- `cargo clippy --lib --examples` must produce 0 warnings
- `cargo test` must pass
- Every new scene must have: help overlay (F1/?), back key (Esc), theme propagation
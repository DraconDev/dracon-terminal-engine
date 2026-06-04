# Showcase Audit — 2026-06-04

## Summary
- **49 registered** examples in data.rs
- **33 embedded** scene files
- **4 external** app binaries
- **13 registered but no implementation** (data.rs entries with no scene file or app binary)
- **Target: ~34 scenes** after cuts

---

## Classification

### ⭐ STARS (Keep + Polish) — The "wow" demos

| # | Name | Cat | Type | Why it's a star |
|---|------|-----|------|-----------------|
| 1 | raycaster | apps | EMB | Wolfenstein 3D raycaster — jaw-dropping for a terminal app |
| 2 | paint | apps | EMB | Pixel art canvas with brushes — shows mouse-driven creative apps |
| 3 | workshop | apps | EMB | Interactive widget playground — the "Storybook" of the framework |
| 4 | command_palette | apps | EMB | IDE Lite — shows CommandPalette + MenuBar, professional UX |
| 5 | live_feed | apps | EMB | SplitPane + TabBar + StreamingText + Sparkline — rich composition |
| 6 | metrics_hub | tools | EMB | Slider + Gauge + ProgressRing + Spinner + StatusBadge — data viz |
| 7 | dev_console | tools | EMB | LogViewer + EventLogger + Inspector — developer tooling |
| 8 | system_monitor | apps | EXT | Live system gauges with auto-refresh — real /proc data |
| 9 | file_manager | apps | EXT | File browser with Tree + Table — real filesystem |
| 10 | chat_client | apps | EXT | Rich chat UI with contacts & panels |
| 11 | ide | apps | — | Full IDE with menus, tabs, tree, editor |
| 12 | git_tui | apps | — | Real Git status/log/diff/branches |
| 13 | dashboard_builder | apps | EXT | Build dashboards with drag & drop |

### ✅ SOLID (Keep as-is) — Good demos, no changes needed

| # | Name | Cat | Type | Notes |
|---|------|-----|------|-------|
| 14 | form_demo | input | EMB | Form layout with validation |
| 15 | autocomplete | input | EMB | Search input with suggestions |
| 16 | tags_input | input | EMB | Tag composition with autocomplete |
| 17 | tooltip | input | EMB | Hover tooltips on buttons |
| 18 | password_input | input | EMB | Login form with masked password |
| 19 | color_picker | input | EMB | Interactive color picker with preview |
| 20 | rich_text | data | EMB | Markdown rendering with tabbed docs |
| 21 | notification_center | data | EMB | Toast notification queue with filters |
| 22 | kanban | data | EMB | Drag-drop kanban board |
| 23 | animation | data | EMB | Animation & easing curves |
| 24 | debug_overlay | data | EMB | Performance metrics, FPS, frame time, profiler |
| 25 | widget_gallery | cookbook | EMB | All interactive widgets demo |
| 26 | theme_switcher | cookbook | EMB | Live theme cycling (21 themes) |
| 27 | accessibility | accessibility | EMB | Accessibility features demo |
| 28 | navigator | apps | EMB | Breadcrumbs + MenuBar + Divider + Label |
| 29 | action_center | apps | EMB | ContextMenu + ConfirmDialog + Toast |

### ⚠️ WEAK (Cut) — Too simple, redundant, or no implementation

| # | Name | Cat | Type | Reason to cut |
|---|------|-----|------|---------------|
| 30 | table_list | apps | EMB | Redundant with data_table concept — just a Table + List |
| 31 | note_editor | apps | EMB | Redundant with text_editor_demo — just TextEditorAdapter |
| 32 | settings_scene | — | EMB | 308 lines, simplest scene, minimal value |
| 33 | radio_scene | input | EMB | Just radio buttons — too simple for standalone scene |
| 34 | calendar | input | EMB | Simple date picker — not impressive enough |
| 35 | progress | data | EMB | Progress indicators — too simple, no wow factor |
| 36 | cell_pool | data | EMB | Cell allocation recycling — too technical/niche |
| 37 | tree_navigator | cookbook | EMB | Expandable tree — simple, covered by file_manager |
| 38 | modal_demo | cookbook | EMB | Modal dialogs — simple, covered by action_center |
| 39 | hud_demo | cookbook | EMB | HUD overlay — simple, covered by workshop |

### ❌ NO IMPLEMENTATION (Remove from data.rs only)

| # | Name | Cat | Reason |
|---|------|-----|--------|
| 40 | arena | apps | No scene file or app binary |
| 41 | command_bindings | cookbook | No scene file — "Live CLI-bound widgets" |
| 42 | split_resizer | cookbook | No scene file — "Drag-to-resize SplitPane" |
| 43 | menu_system | cookbook | No scene file — "MenuBar + ContextMenu" |
| 44 | tabbed_panels | cookbook | No scene file — "Tab bar with panel switching" |
| 45 | data_table | cookbook | No scene file — "Sortable table with selection" |
| 46 | log_monitor | tools | No scene file — "Live log viewer with filters" |
| 47 | desktop | tools | No scene file — "Draggable windows + taskbar" |
| 48 | input_debug | tools | No scene file — "Key/mouse event visualizer" |
| 49 | text_editor_demo | tools | No scene file — "Syntax-highlighted editor" |
| — | sqlite_browser | tools | No scene file — "Browse SQLite databases" |

---

## Result: 49 → 34 (cut 15)

**Kept: 29 embedded + 4 external + 1 unimplemented = 34**

Actually: 29 embedded (after cutting 4 weak scenes) + 4 external + 1 ide (keep despite no file) = 34

Wait — let me recount. We keep 30 embedded (33 - 3 weak) + 4 external = 34. But we also keep ide, git_tui, arena despite no files? No — arena has no implementation at all, so cut it. ide and git_tui are also no-implementation, but they're impressive concepts. Since they have no implementation, they must be cut from data.rs.

**Final count: 29 embedded scenes + 4 external apps = 33 scenes**

Cut 15 entries from data.rs (10 no-implementation + 5 weak scenes).
Cut 3 scene files (table_list_scene.rs, note_editor_scene.rs, settings_scene.rs).
Cut 2 scene files (radio_scene.rs, calendar_scene.rs).
Cut 2 scene files (progress_scene.rs, cell_pool_scene.rs).
Cut 3 scene files (tree_navigator.rs, modal_demo.rs, hud_demo_scene.rs).

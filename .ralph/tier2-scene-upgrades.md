# Tier 2 Scene Upgrades

## Goal
Upgrade remaining Tier 2/3 scenes to Tier 1 level.

## Done (21/34 = 62%)
widget_gallery, theme_switcher, password_input, notification_center, color_picker, animation, tags_input, progress, cell_pool, rich_text, debug_overlay, metrics_hub, table_list, navigator

## Remaining (13 scenes)

### Tier 2 Priority
1. kanban → Kanban Board
2. tree_navigator → Tree Navigator (already decent — add polish)
3. modal_demo → Modal Showcase (already decent)
4. live_feed → Live Feed
5. hud_demo → HUD Overlay

### Tier 3 (lower priority)
6. action_center → Action Center
7. autocomplete → Autocomplete Demo
8. calendar → Calendar Widget
9. command_palette → Command Palette
10. control_panel → Control Panel
11. dev_console → Dev Console
12. form_demo → Form Demo
13. note_editor → Note Editor

## Approach
Each iteration: read existing scene, upgrade to split layout with sidebar, widgets, help overlay, footer.

## Constraints
- 0 clippy warnings
- 0 production .unwrap()
- cargo test --lib passes
- Follow existing patterns from completed scenes

## Max Iterations: 15
# Showcase Full Audit — TODO

## Audit Summary
- **23 embedded scenes**, 11,489 lines total
- **20 framework widgets** with NO showcase demo
- **3 scenes** manually render widgets that exist in the framework (modal_demo, tooltip_scene, accessibility_scene)
- **All 23 scenes** pass compliance (help overlay, Esc/BACK, mouse handling)
- **Key issue**: many scenes hardcode char keys instead of using KeybindingSet

## Priority 1: New Scenes for Undemonstrated Widgets (HIGH impact)

### 1. Command Palette + Menu Bar Scene (~500 lines)
**Widgets:** CommandPalette, MenuBar
**Concept:** "IDE Lite" — command palette overlay (Ctrl+P) + menu bar at top, with a simple text area showing action log
**Why:** CommandPalette and MenuBar are critical interactive widgets with zero showcase

### 2. Table + List + Log Viewer Scene (~450 lines)
**Widgets:** Table (sortable, scrollable), List (selectable), LogViewer
**Concept:** "Server Dashboard" — process table (sortable columns) + event log viewer + stats list
**Why:** Table and List are the two most fundamental data widgets, both missing from showcase

### 3. Form Widget + KeyValueGrid Scene (~400 lines)
**Widgets:** Form (with validation), KeyValueGrid
**Concept:** "Settings Panel" — real Form widget with validation rules + key-value grid showing config values
**Why:** form_demo uses individual widgets, not the actual Form widget. KeyValueGrid has no demo.

### 4. Split Pane + TabBar Scene (~400 lines)
**Widgets:** Split (resizable), TabBar
**Concept:** "Multi-Panel Layout" — split pane with draggable divider + tab bar switching content
**Why:** Split and TabBar are fundamental layout/navigation widgets with zero showcase

### 5. Streaming Text + Sparkline Scene (~350 lines)
**Widgets:** StreamingText, Sparkline
**Concept:** "Live Feed" — streaming log output + sparkline charts showing metrics over time
**Why:** Both are real-time/data widgets with no showcase

### 6. Context Menu + Confirm Dialog + Toast Scene (~400 lines)
**Widgets:** ContextMenu (upgraded), ConfirmDialog, Toast
**Concept:** "Action Center" — right-click for context menu, confirm before delete, toast notifications
**Why:** These are the interaction pattern widgets; modal_demo manually renders instead of using them

## Priority 2: Rewrite Scenes Using Framework Widgets (MEDIUM impact)

### 7. Rewrite modal_demo to use Modal/ConfirmDialog/Toast widgets
**Current:** Manually renders confirm dialogs, toasts, dimmed backdrops (626 lines of manual cells)
**Target:** Compose Modal, ConfirmDialog, Toast widgets properly

### 8. Rewrite tooltip_scene to use Tooltip widget
**Current:** Manually renders tooltip popups (662 lines)
**Target:** Use the actual Tooltip framework widget

### 9. Fix rich_text_scene — add scroll key forwarding
**Current:** Up/Down/PageUp/PageDown not forwarded to RichText widget
**Target:** Forward scroll keys so users can read full content

## Priority 3: Polish Existing Scenes (LOW impact)

### 10. Help overlay completeness
- progress_scene: add click interactions to help
- debug_overlay_scene: add help shortcut to help
- autocomplete_scene: use ↑↓ symbols instead of ^v

### 11. Theme propagation gaps
- animation_scene: ball colors should use theme colors, not hardcoded
- cell_pool_scene: pool grid colors should respond to theme
- paint_scene: toolbar chrome should propagate theme

### 12. Visual density improvements
- color_picker_scene: fill empty space below "recent" with usage tips or color theory
- rich_text_scene: add more content or companion panel to fill screen

## Priority 4: Remaining Undemonstrated Widgets (LOW — niche widgets)

These are less critical but would complete coverage:
- Divider (simple, could add to any multi-panel scene)
- EventLogger (specialized, maybe add to debug_overlay_scene)
- HUD (specialized, could add to raycaster or arena)
- Label (trivial, could add to workshop)
- TextEditorAdapter (specialized, maybe own scene)
- WidgetInspector (debug tool, could add to workshop)

## Implementation Order

Phase 1 (this loop): Items 1-3 (3 new scenes)
Phase 2: Items 4-6 (3 more new scenes)
Phase 3: Items 7-9 (3 rewrites)
Phase 4: Items 10-12 (polish)
Phase 5: Items from Priority 4 as needed

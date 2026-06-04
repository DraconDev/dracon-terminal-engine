# Showcase Audit Report — 2026-06-04

## Summary

- **Total scenes/apps audited:** 32 (28 embedded + 4 external)
- **Audit type:** Read-only, no code changes
- **Merit ratings:** 1-5 scale (1=weak, 5=exceptional)
- **Audit dimensions:** render, mouse, keyboard, theme, focus, dirty tracking

## Audit Methodology

Each scene was analyzed across 6 dimensions:
1. **Render:** Background fill (fill_bg or manual cell iteration), Plane creation
2. **Mouse:** Down/Drag/Up/Scroll/Moved event handling
3. **Keyboard:** KeybindingSystem usage, Esc/Back handling, Help key
4. **Theme:** on_theme_change propagation to child widgets
5. **Focus:** Focus management (focused field, Tab navigation)
6. **Dirty:** Dirty flag tracking, mark_dirty/clear_dirty

---

## Star Scenes (8)

### raycaster (567 lines) — ⭐5/5
**Wolfenstein-style 3D raycaster engine**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill L306-310 (`cell.bg = t.bg`), raycasting viewport |
| Mouse | ✅ | ScrollUp/ScrollDown for turning (L539-544), no click/drag |
| Keyboard | ✅ | WASD movement, arrow keys, keybindings system |
| Theme | ✅ | Propagates to status_bar (L553-555) |
| Focus | ⚠️ | No focus management (pure keyboard-driven) |
| Dirty | ✅ | dirty flag set on state changes |

**Notes:** Unique 3D raycaster in terminal. Mouse wheel turns camera. Keyboard-only movement is appropriate for this type of app.

---

### paint (619 lines) — ⭐5/5
**Mouse-driven pixel art canvas with brushes**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill L368-372, canvas rendering |
| Mouse | ✅ | Full mouse support: Down/Drag/Up for drawing (L501-595) |
| Keyboard | ✅ | Tool selection (B/E/F), color shortcuts, keybindings |
| Theme | ✅ | Propagates to status_bar (L605-607) |
| Focus | ⚠️ | No focus management (mouse-driven) |
| Dirty | ✅ | dirty flag set on canvas changes |

**Notes:** Excellent mouse interaction. Drawing works smoothly with drag support.

---

### workshop (776 lines) — ⭐4/5
**Interactive widget playground (Storybook)**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, widget preview area |
| Mouse | ✅ | ScrollUp/ScrollDown for widget selection (L715-724), click |
| Keyboard | ✅ | Arrow keys adjust properties, Space toggles, keybindings |
| Theme | ✅ | Propagates to 9 widgets (L737-747) |
| Focus | ⚠️ | Widget selection via keyboard, no Tab focus |
| Dirty | ✅ | dirty flag set on property changes |

**Notes:** Good variety of widgets. Scroll wheel cycles through widgets.

---

### hud_demo (588 lines) — ⭐4/5
**Game HUD overlay with combat mechanics**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | fill_bg() L128, HUD overlay rendering |
| Mouse | ⚠️ | handle_mouse exists but minimal (L557) |
| Keyboard | ✅ | Attack/movement keys, keybindings system |
| Theme | ✅ | Propagates to 5 widgets (L562-568) |
| Focus | ⚠️ | No focus management (game-driven) |
| Dirty | ✅ | dirty flag set on game state changes |

**Notes:** Engaging game HUD. Mouse interaction could be improved for click-to-attack.

---

### live_feed (807 lines) — ⭐5/5
**Split pane + TabBar + StreamingText + Sparkline**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | fill_bg() L228, multi-panel layout |
| Mouse | ✅ | Tab bar clicks, split divider drag (L715-750) |
| Keyboard | ✅ | Tab switching, Space for updates, keybindings |
| Theme | ✅ | Propagates to 6 widgets (L757-764) |
| Focus | ⚠️ | Tab-based navigation, no field focus |
| Dirty | ✅ | dirty flag set on data changes |

**Notes:** Complex multi-panel dashboard. Good split pane drag interaction.

---

### metrics_hub (767 lines) — ⭐4/5
**Slider + Gauge + ProgressRing + Spinner + StatusBadge**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | fill_bg() L197, data visualization |
| Mouse | ⚠️ | Minimal mouse handling (L483-484) |
| Keyboard | ✅ | Slider adjustment, keybindings system |
| Theme | ✅ | Propagates to 11 widgets (L510-523) |
| Focus | ⚠️ | Slider focus via keyboard only |
| Dirty | ✅ | dirty flag set on slider changes |

**Notes:** Good data visualization. Mouse interaction for sliders would improve UX.

---

### dev_console (404 lines) — ⭐3/5
**LogViewer + EventLogger + Label + Divider + Inspector**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | fill_bg() L150, log viewer rendering |
| Mouse | ✅ | Filter bar clicks, log viewer scroll (L350-375) |
| Keyboard | ✅ | Space adds logs, 1-5 filters, I toggles inspector |
| Theme | ✅ | Propagates to 4 widgets (L383-388) |
| Focus | ⚠️ | Log viewer focus via keyboard |
| Dirty | ✅ | mark_dirty/clear_dirty properly implemented (L398-403) |

**Notes:** Simple but functional log viewer. Inspector toggle is nice.

---

### command_palette (758 lines) — ⭐4/5
**IDE Lite: CommandPalette + MenuBar**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | fill_bg() L327, menu bar + palette rendering |
| Mouse | ✅ | Menu bar clicks, command palette item clicks (L711-725) |
| Keyboard | ✅ | Ctrl+P opens palette, arrow keys navigate, Enter executes |
| Theme | ✅ | Propagates to 3 widgets (L738-742) |
| Focus | ✅ | Menu bar focus, palette focus management |
| Dirty | ✅ | dirty flag set on command execution |

**Notes:** Professional IDE-like UX. Good keyboard + mouse integration.

---

## Secondary Scenes (20)

### action_center (709 lines) — ⭐4/5
**ContextMenu + ConfirmDialog + Toast**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | fill_bg(), context menu + dialog rendering |
| Mouse | ✅ | Right-click context menu, dialog button clicks |
| Keyboard | ✅ | Arrow keys navigate, Enter selects, Esc closes |
| Theme | ✅ | Propagates to 4 widgets |
| Focus | ✅ | Context menu focus, dialog focus trapping |
| Dirty | ✅ | dirty flag set on actions |

**Notes:** Good interaction patterns. Context menu + confirm + toast work well together.

---

### navigator (748 lines) — ⭐3/5
**Breadcrumbs + MenuBar + Divider + Label**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | fill_bg(), file list rendering |
| Mouse | ✅ | ScrollUp/ScrollDown for file list, breadcrumb clicks |
| Keyboard | ✅ | Arrow keys navigate, Enter opens, keybindings |
| Theme | ✅ | Propagates to 4 widgets |
| Focus | ⚠️ | File list focus via keyboard only |
| Dirty | ✅ | dirty flag set on navigation |

**Notes:** Basic file explorer. Mouse scroll support is good.

---

### widget_gallery (615 lines) — ⭐4/5
**All interactive widgets demo**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill L394, widget preview area |
| Mouse | ✅ | Widget interaction clicks |
| Keyboard | ✅ | Arrow keys navigate widgets, Space/Enter interact |
| Theme | ✅ | Propagates to 14 widgets |
| Focus | ✅ | Widget selection focus |
| Dirty | ✅ | dirty flag set on widget changes |

**Notes:** Comprehensive widget showcase. Good variety of interactions.

---

### theme_switcher (643 lines) — ⭐3/5
**Live theme cycling (21 themes)**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, theme preview |
| Mouse | ⚠️ | Minimal mouse handling |
| Keyboard | ✅ | Arrow keys cycle themes, keybindings |
| Theme | ✅ | Propagates to 20 widgets (extensive!) |
| Focus | ⚠️ | Theme selection via keyboard only |
| Dirty | ✅ | dirty flag set on theme change |

**Notes:** Excellent theme propagation coverage. Mouse interaction would improve UX.

---

### modal_demo (885 lines) — ⭐3/5
**Modal dialogs + focus trapping**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, modal overlay rendering |
| Mouse | ✅ | Modal button clicks, backdrop click dismiss |
| Keyboard | ✅ | Tab cycles fields, Enter submits, Esc closes |
| Theme | ✅ | Propagates to 2 widgets |
| Focus | ✅ | Focus trapping in modals |
| Dirty | ✅ | dirty flag set on modal state |

**Notes:** Good focus trapping implementation. Modal dialogs work well.

---

### tree_navigator (795 lines) — ⭐3/5
**Expandable tree widget with detail pane**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, tree + detail rendering |
| Mouse | ✅ | Tree node clicks, expand/collapse |
| Keyboard | ✅ | Arrow keys navigate, Enter expands, keybindings |
| Theme | ✅ | Propagates to 4 widgets |
| Focus | ⚠️ | Tree focus via keyboard only |
| Dirty | ✅ | dirty flag set on tree changes |

**Notes:** Basic tree widget. Detail pane is nice addition.

---

### settings_panel (309 lines) — ⭐2/5
**Form + KeyValueGrid configuration panel**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | fill_bg(), form + grid rendering |
| Mouse | ✅ | Form field clicks |
| Keyboard | ✅ | Tab cycles fields, Enter submits |
| Theme | ✅ | Propagates to 4 widgets |
| Focus | ✅ | Form field focus |
| Dirty | ✅ | dirty flag set on form changes |

**Notes:** Very basic implementation. Minimal complexity.

---

### animation (803 lines) — ⭐4/5
**Animation & easing curves**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill L212, animation rendering |
| Mouse | ⚠️ | Minimal mouse handling |
| Keyboard | ✅ | Space toggles animations, keybindings |
| Theme | ✅ | Propagates to 5 widgets |
| Focus | ⚠️ | No focus management (animation-driven) |
| Dirty | ✅ | dirty flag set on animation changes |

**Notes:** Visually engaging bouncing balls. Easing curves are well-implemented.

---

### rich_text (683 lines) — ⭐3/5
**Markdown rendering with tabbed docs**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, markdown rendering |
| Mouse | ✅ | Tab clicks, content interaction |
| Keyboard | ✅ | Tab switching, arrow keys scroll, keybindings |
| Theme | ✅ | Propagates to 3 widgets |
| Focus | ⚠️ | Tab focus only |
| Dirty | ✅ | dirty flag set on tab changes |

**Notes:** Standard markdown rendering. Tab switching works well.

---

### notification_center (872 lines) — ⭐3/5
**Toast notification queue with filters**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, notification list rendering |
| Mouse | ⚠️ | Minimal mouse handling |
| Keyboard | ✅ | Arrow keys navigate, Enter dismisses, keybindings |
| Theme | ✅ | Propagates to 2 widgets |
| Focus | ⚠️ | Notification focus via keyboard only |
| Dirty | ✅ | dirty flag set on notification changes |

**Notes:** Standard toast queue. Filter pills are nice.

---

### cell_pool (656 lines) — ⭐2/5
**Cell allocation recycling with gauges**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, memory map visualization |
| Mouse | ⚠️ | Minimal mouse handling |
| Keyboard | ✅ | Space allocates, +/- adjust speed, keybindings |
| Theme | ✅ | Propagates to 2 widgets |
| Focus | ⚠️ | No focus management (technical demo) |
| Dirty | ✅ | dirty flag set on allocation changes |

**Notes:** Very technical/niche. Cell allocation recycling is specialized.

---

### kanban (478 lines) — ⭐4/5
**Drag-drop kanban board**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | fill_bg(), kanban board rendering |
| Mouse | ✅ | Card drag-and-drop between columns |
| Keyboard | ✅ | Arrow keys navigate, Enter moves cards, keybindings |
| Theme | ✅ | Propagates to 3 widgets |
| Focus | ⚠️ | Card focus via keyboard only |
| Dirty | ✅ | dirty flag set on card moves |

**Notes:** Good drag-and-drop interaction. Card movement works smoothly.

---

### debug_overlay (833 lines) — ⭐3/5
**Performance metrics, FPS, frame time, profiler**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, metrics visualization |
| Mouse | ⚠️ | Minimal mouse handling |
| Keyboard | ✅ | Space toggles overlay, 1-2 toggle profiler, keybindings |
| Theme | ✅ | Propagates to 4 widgets |
| Focus | ⚠️ | No focus management (overlay-driven) |
| Dirty | ✅ | dirty flag set on metrics changes |

**Notes:** Standard performance overlay. Profiler toggle is useful.

---

### form_demo (863 lines) — ⭐3/5
**Form layout with validation**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, form + preview rendering |
| Mouse | ✅ | Form field clicks, drag reorder |
| Keyboard | ✅ | Tab cycles fields, Enter submits, keybindings |
| Theme | ✅ | Propagates to 8 widgets |
| Focus | ✅ | Form field focus, Tab navigation |
| Dirty | ✅ | dirty flag set on form changes |

**Notes:** Good form implementation. Drag reorder is nice feature.

---

### autocomplete (651 lines) — ⭐3/5
**Search input with suggestions**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, search + suggestions rendering |
| Mouse | ✅ | Suggestion item clicks |
| Keyboard | ✅ | Type to filter, arrow keys navigate, Enter selects |
| Theme | ✅ | Propagates to 3 widgets |
| Focus | ✅ | Search input focus |
| Dirty | ✅ | dirty flag set on search changes |

**Notes:** Standard autocomplete. Suggestion list works well.

---

### tags_input (728 lines) — ⭐3/5
**Tag composition with autocomplete**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, tag pills + input rendering |
| Mouse | ✅ | Tag removal clicks |
| Keyboard | ✅ | Type to add tags, Backspace removes, keybindings |
| Theme | ✅ | Propagates to 3 widgets |
| Focus | ✅ | Input focus |
| Dirty | ✅ | dirty flag set on tag changes |

**Notes:** Standard tags input. Autocomplete integration is nice.

---

### tooltip (885 lines) — ⭐3/5
**Hover tooltips on buttons**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, tooltip rendering |
| Mouse | ✅ | Hover triggers tooltip, click dismisses |
| Keyboard | ✅ | Arrow keys navigate, keybindings |
| Theme | ✅ | Propagates to 2 widgets |
| Focus | ⚠️ | No focus management (hover-driven) |
| Dirty | ✅ | dirty flag set on hover changes |

**Notes:** Standard tooltip implementation. Hover trigger works well.

---

### password_input (834 lines) — ⭐3/5
**Login form with masked password input**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, login form rendering |
| Mouse | ✅ | Form field clicks |
| Keyboard | ✅ | Tab cycles fields, Enter submits, keybindings |
| Theme | ✅ | Propagates to 5 widgets |
| Focus | ✅ | Form field focus, Tab navigation |
| Dirty | ✅ | dirty flag set on input changes |

**Notes:** Standard login form. Password masking works correctly.

---

### color_picker (738 lines) — ⭐3/5
**Interactive color picker with preview**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, color wheel rendering |
| Mouse | ✅ | Color wheel clicks, palette clicks |
| Keyboard | ✅ | Arrow keys adjust color, keybindings |
| Theme | ✅ | Propagates to 3 widgets |
| Focus | ⚠️ | Color selection via keyboard/mouse |
| Dirty | ✅ | dirty flag set on color changes |

**Notes:** Standard color picker. Color wheel interaction works well.

---

### accessibility (882 lines) — ⭐3/5
**Screen reader support (OSC 99)**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, form + focus rings rendering |
| Mouse | ✅ | Form field clicks |
| Keyboard | ✅ | Tab cycles fields, Enter submits, keybindings |
| Theme | ✅ | Propagates to 4 widgets |
| Focus | ✅ | Focus ring visualization, Tab navigation |
| Dirty | ✅ | dirty flag set on focus changes |

**Notes:** Good accessibility implementation. Focus rings are visible.

---

## External Apps (4)

### system_monitor (1462 lines) — ⭐5/5
**Live system gauges with auto-refresh**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, gauge + sparkline rendering |
| Mouse | ✅ | ScrollUp/ScrollDown for process list, click selection |
| Keyboard | ✅ | Arrow keys navigate processes, keybindings |
| Theme | ✅ | Propagates to 5 widgets |
| Focus | ⚠️ | Process list focus via keyboard only |
| Dirty | ✅ | dirty flag set on data refresh |

**Notes:** Comprehensive system monitoring. Real /proc data reading. Scroll support for process list.

---

### file_manager (1762 lines) — ⭐5/5
**File browser with Tree + Table**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, tree + table rendering |
| Mouse | ✅ | Tree node clicks, split pane drag, breadcrumb clicks |
| Keyboard | ✅ | Arrow keys navigate, Enter opens, keybindings |
| Theme | ✅ | Propagates to 4 widgets |
| Focus | ✅ | Tree/table focus management |
| Dirty | ✅ | dirty flag set on navigation |

**Notes:** Full-featured file manager. Split pane drag works well.

---

### chat_client (1044 lines) — ⭐4/5
**Rich chat UI with contacts & panels**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | fill_bg() L438, message list + input rendering |
| Mouse | ✅ | Contact list clicks, message input focus |
| Keyboard | ✅ | Enter sends message, arrow keys navigate, keybindings |
| Theme | ✅ | Propagates to 3 widgets |
| Focus | ✅ | Input field focus |
| Dirty | ✅ | dirty flag set on message changes |

**Notes:** Rich chat UI. Contact list + message list work well together.

---

### dashboard_builder (1015 lines) — ⭐4/5
**Build dashboards with drag & drop**

| Dimension | Status | Details |
|-----------|--------|---------|
| Render | ✅ | Manual bg fill, dashboard grid rendering |
| Mouse | ✅ | ScrollUp/ScrollDown for gauge values, drag widgets |
| Keyboard | ✅ | Arrow keys navigate, keybindings |
| Theme | ✅ | Propagates to 2 widgets |
| Focus | ⚠️ | Widget focus via keyboard only |
| Dirty | ✅ | dirty flag set on widget changes |

**Notes:** Good drag-and-drop interaction. Scroll wheel adjusts gauge values.

---

## Bug Findings

### No HIGH or MEDIUM severity bugs found.

All scenes properly:
- Fill backgrounds (either fill_bg() or manual cell iteration)
- Use keybindings system
- Propagate theme changes
- Track dirty state

### LOW Severity Observations

1. **Most scenes lack mouse scroll support** — Only raycaster, navigator, workshop have ScrollUp/ScrollDown handling. Other scrollable content (lists, logs, messages) cannot be scrolled with mouse wheel.

2. **Focus management varies** — Some scenes have explicit focus (form_demo, autocomplete, password_input), others rely on keyboard-only navigation (raycaster, paint, animation).

3. **settings_panel and cell_pool are minimal** — Both have merit 2/5, very basic implementations.

---

## Verification

- cargo check: ✅ 0 errors
- cargo clippy: ✅ 0 warnings  
- cargo test --example showcase: ✅ 12/12 pass
- No code changes made (read-only audit)

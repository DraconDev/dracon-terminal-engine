# Port from Tiles — What Actually Makes Sense

## DONE ✅

### 1. Context Menu Widget Upgrade
**What:** Upgraded existing `ContextMenu` widget with Tiles-pattern features:
- `ContextMenuItem` struct (id, label, icon, is_separator) — replaces old `(String, ContextAction)` pairs
- Separator rendering (─ line, skipped during keyboard nav)
- Hover highlighting (`hover_bg`)
- Click-outside dismiss (returns `false` → caller hides)
- Rounded border (╭╮╰╯) with `theme.outline` + `surface_elevated` bg
- `on_select` callback
- Auto-width calculation from item labels
- Screen bounds clamping
- `from_actions()` / `from_actions_with_id()` for backward compat
- 17 internal unit tests

### 2. Middle-Click Paste in BaseInput
**What:** `MouseButton::Middle` Down in `BaseInput::handle_mouse()` calls
`get_primary_selection_text()` and inserts at cursor. SearchInput and PasswordInput
inherit it via `self.base.handle_mouse()` delegation.
**Caveat:** X11 works via xclip, Wayland spotty (wl-paste --primary).

### 3. Input Shield on App (from previous iteration)
**What:** `app.shield_input(Duration)` and `app.is_input_shielded()`.
Already added — scene router integration is just calling it after push/pop.

## NOT PORTING (app-level, not framework)
- ❌ Stale file list clearing — Tiles-specific async pattern
- ❌ Undo close tab — app-level tab management
- ❌ Sub-struct decomposition — app architecture
- ❌ AppMode enum — documented as pattern in AGENTS.md
- ❌ DragState pending_click_idx — already in MarqueeState
- ❌ BulkRename/SignalSelect/OpenWith — Tiles features
- ❌ History navigation (push_history) — app-level

## MAYBE LATER
- `previous_mode` for stacked overlays
- Persistent render-bounds vs per-frame ScopedZoneRegistry
- Double-click detection utility

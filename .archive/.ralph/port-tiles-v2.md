Port 3 items from Tiles to dracon-terminal-engine framework.

## Item 1: Context Menu Widget
- New `src/framework/widgets/context_menu.rs`
- `ContextMenuItem { id: String, label: String, icon: Option<char>, is_separator: bool }`
- `ContextMenu` struct with items, position (x,y), selected index, theme, on_select callback
- Builder: `.new(items)`, `.at(x, y)`, `.with_theme(t)`, `.on_select(cb)`
- Auto-clamp to screen bounds
- Keyboard: ↑/↓ nav, Enter select, Esc dismiss
- Mouse: hover highlight, click select, click outside dismiss
- Render: Clear + styled list with border
- ScopedZoneRegistry for mouse dispatch
- Unit tests

## Item 2: Middle-Click Paste in BaseInput
- `src/framework/widgets/text_input_base.rs` — handle MouseButton::Middle Down
- Call `get_primary_selection_text()`, insert at cursor position
- Same for `src/framework/widgets/search_input.rs`
- Document X11/Wayland caveats

## Item 3: Input Shield Integration in Scene Router
- After scene push/pop, signal that input should be shielded
- Either: scene_router returns a flag, or showcase calls shield_input after transitions

## Acceptance:
- Context menu compiles, 0 clippy warnings
- Context menu has unit tests
- Middle-click paste works in BaseInput
- `cargo clippy --lib --examples` 0 warnings
- `cargo test` all pass

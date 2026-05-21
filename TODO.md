# TODO

## 🐛 White Lines When Typing

**Status**: Needs reproduction + root cause identification

**Suspects** (from compositor pipeline audit):

- [ ] **Terminal emulator compatibility**: The render outputs `\x1b[?2026h` (synchronized update). Some terminals mishandle this. Try disabling it or testing in a different terminal.
- [ ] **`Color::Reset` + light terminal theme**: Cells with `bg: Color::Reset` show the terminal's default background. If the user's terminal uses white default, `\x1b[49m` cells appear as white lines.
- [ ] **`line_cursor_moved` logic**: When `cell.skip` or `cell == last_cell`, `line_cursor_moved` is forced to `false`, causing extra absolute cursor moves. This is correct but wasteful — verify it doesn't create visual artifacts in any terminal.
- [ ] **`draw_text` overflow**: `draw_text()` (in `examples/showcase/render.rs`) iterates by character index, not display width, and does NOT check plane width before setting cells. If text exceeds `plane.width`, cells wrap to the next row.
- [ ] **`Plane::put_str` grapheme handling**: Zero-width characters are skipped without advancing `x`. Verify this doesn't leave uninitialized non-transparent cells.

**Help needed from reporter:**
- [ ] Which terminal emulator? (kitty, Alacritty, Windows Terminal, etc.)
- [ ] `$TERM` and `$COLORTERM` values
- [ ] Is the search bar active (`/` pressed) when typing causes white lines, or does it happen with any keypress?
- [ ] Screenshot or description: does the whole row go white, or just specific columns?

---

## 🔧 Clippy Cleanup

The `else if` condition `c >= 128 || c < 0x20` in `src/compositor/engine.rs` was rewritten as a bare `else` to fix `clippy::manual_range_contains`. (Done in the latest round.)

---

## 🧹 General Code Quality

- [ ] **`examples/showcase/render.rs`**: `draw_text` and `draw_text_bounded` use character index, not display width, and lack plane-width bounds checks. Refactor to use `put_str` with clipping, or add width checks before setting cells.
- [ ] **`examples/showcase/widget.rs`**: `dispatch_key` sets `self.dirty = true` for EVERY keypress, even unhandled ones. This causes full re-renders on keys that do nothing — wasteful for performance.

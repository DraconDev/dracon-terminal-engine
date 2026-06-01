//! Plane blitting helpers.

use crate::compositor::{Color, Plane};

/// Blits `src` onto `dest` at the given offset.
///
/// Skips transparent cells, null characters, and cells with `Color::Reset` background.
/// Bounds-checked to prevent panics.
pub fn blit_to(dest: &mut Plane, src: &Plane, offset_x: usize, offset_y: usize) {
    if dest.cells.is_empty() || src.cells.is_empty() {
        return;
    }
    for i in 0..src.cells.len() {
        let cell = &src.cells[i];
        if cell.char == '\0' || cell.transparent || cell.bg == Color::Reset {
            continue;
        }
        let x = (i % src.width as usize) + offset_x;
        let y = (i / src.width as usize) + offset_y;
        if x < dest.width as usize && y < dest.height as usize {
            let idx = y * dest.width as usize + x;
            if idx < dest.cells.len() {
                dest.cells[idx] = *cell;
            }
        }
    }
}

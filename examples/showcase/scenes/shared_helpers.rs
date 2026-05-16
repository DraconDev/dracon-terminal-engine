//! Shared helper functions for embedded scenes.
//!
//! These are extracted from duplicated local definitions across all scene files
//! to reduce code duplication and ensure consistent behavior.

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};

/// Draws a line of text at (x, y) with foreground, background, and optional bold styling.
/// Writes directly into `plane.cells` using flat indexing, which matches the
/// internal representation used by all embedded scenes.
pub fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = (y * plane.width + x + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch,
                fg,
                bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false,
                skip: false,
            };
        }
    }
}

/// Copies cells from `src` plane into `dest` at the given offset.
/// Skips transparent cells and cells with '\0' character.
/// Bounds-checks to prevent out-of-range writes.
pub fn blit_to(dest: &mut Plane, src: &Plane, offset_x: usize, offset_y: usize) {
    for i in 0..src.cells.len() {
        let cell = &src.cells[i];
        if cell.char == '\0' || cell.transparent {
            continue;
        }
        let row = i / src.width as usize;
        let col = i % src.width as usize;
        let dy = offset_y + row;
        let dx = offset_x + col;
        if dy >= dest.height as usize || dx >= dest.width as usize {
            continue;
        }
        let idx = dy * dest.width as usize + dx;
        if idx < dest.cells.len() {
            dest.cells[idx] = *cell;
        }
    }
}
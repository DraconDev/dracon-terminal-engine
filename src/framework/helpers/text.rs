//! Text drawing helpers.

use crate::compositor::{Cell, Color, Plane, Styles};

/// Draws text at position `(x, y)` on the plane with the given colors.
///
/// Clips at the right boundary of `plane.width` to prevent row-wrapping.
/// Text that exceeds the plane width is truncated.
pub fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    if y >= plane.height {
        return;
    }
    let style = if bold { Styles::BOLD } else { Styles::empty() };
    for (i, ch) in text.chars().enumerate() {
        let cx = x + i as u16;
        if cx >= plane.width {
            break;
        }
        let idx = (y * plane.width + cx) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch,
                fg,
                bg,
                style,
                transparent: false,
                skip: false,
            };
        }
    }
}

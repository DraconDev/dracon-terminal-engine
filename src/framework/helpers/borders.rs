//! Border drawing helpers.

use crate::compositor::{Cell, Plane, Styles};

/// Draws a rounded border (╭╮╰╯─│) at position `(x, y)` with dimensions `(w, h)`.
///
/// Border cells are drawn with the theme's `outline` color.
/// Corner cells use the theme's `primary` color.
pub fn draw_rounded_border(
    plane: &mut Plane,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    t: &crate::framework::theme::Theme,
) {
    if w < 2 || h < 2 || x + w > plane.width || y + h > plane.height {
        return;
    }
    // Corners
    let corners = [
        ('╭', x, y),
        ('╮', x + w - 1, y),
        ('╰', x, y + h - 1),
        ('╯', x + w - 1, y + h - 1),
    ];
    for (ch, cx, cy) in corners {
        let idx = (cy * plane.width + cx) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch,
                fg: t.primary,
                bg: t.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
    }
    // Horizontal edges
    for cx in x + 1..x + w - 1 {
        let idx = (y * plane.width + cx) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: '─',
                fg: t.outline,
                bg: t.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
        let idx = ((y + h - 1) * plane.width + cx) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: '─',
                fg: t.outline,
                bg: t.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
    }
    // Vertical edges
    for cy in y + 1..y + h - 1 {
        let idx = (cy * plane.width + x) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: '│',
                fg: t.outline,
                bg: t.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
        let idx = (cy * plane.width + x + w - 1) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: '│',
                fg: t.outline,
                bg: t.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
    }
}

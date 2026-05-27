//! Shared rendering helpers for standalone examples and widgets.
//!
//! Provides common drawing primitives that are duplicated across many
//! standalone binary examples. Import via `use dracon_terminal_engine::framework::helpers::*;`.

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

/// Draws a rounded border (╭╮╰╯─│) at position `(x, y)` with dimensions `(w, h)`.
///
/// Border cells are drawn with the theme's `outline` color.
/// Corner cells use the theme's `primary` color.
pub fn draw_rounded_border(plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, t: &crate::framework::theme::Theme) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_text_basic() {
        let mut plane = Plane::new(0, 10, 3);
        draw_text(&mut plane, 0, 0, "Hi", Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), false);
        assert_eq!(plane.cells[0].char, 'H');
        assert_eq!(plane.cells[1].char, 'i');
    }

    #[test]
    fn test_draw_text_clips_at_boundary() {
        let mut plane = Plane::new(0, 5, 1);
        draw_text(&mut plane, 3, 0, "Hello", Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), false);
        assert_eq!(plane.cells[3].char, 'H');
        assert_eq!(plane.cells[4].char, 'e');
        // "llo" is clipped — would have wrapped to next row in the buggy version
    }

    #[test]
    fn test_draw_text_out_of_bounds_y() {
        let mut plane = Plane::new(0, 10, 2);
        draw_text(&mut plane, 0, 5, "Hi", Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), false);
        // Should not panic
    }

    #[test]
    fn test_blit_to_skips_transparent() {
        let mut dest = Plane::new(0, 4, 2);
        let mut src = Plane::new(0, 2, 1);
        src.cells[0] = Cell { char: 'A', fg: Color::Rgb(255, 255, 255), bg: Color::Rgb(0, 0, 0), style: Styles::empty(), transparent: false, skip: false };
        src.cells[1] = Cell { char: 'B', fg: Color::Rgb(255, 255, 255), bg: Color::Rgb(0, 0, 0), style: Styles::empty(), transparent: true, skip: false };
        blit_to(&mut dest, &src, 0, 0);
        assert_eq!(dest.cells[0].char, 'A');
        // transparent cell was not copied — dest retains its initial value
        assert_ne!(dest.cells[1].char, 'B');
    }

    #[test]
    fn test_blit_to_skips_null() {
        let mut dest = Plane::new(0, 4, 2);
        let mut src = Plane::new(0, 2, 1);
        src.cells[0] = Cell { char: 'A', fg: Color::Rgb(255, 255, 255), bg: Color::Rgb(0, 0, 0), style: Styles::empty(), transparent: false, skip: false };
        src.cells[1] = Cell { char: '\0', fg: Color::Rgb(255, 255, 255), bg: Color::Rgb(0, 0, 0), style: Styles::empty(), transparent: false, skip: false };
        blit_to(&mut dest, &src, 0, 0);
        assert_eq!(dest.cells[0].char, 'A');
        // null cell was not copied — dest retains its initial value
        assert_ne!(dest.cells[1].char, 'A');
    }
}

//! Shared rendering helpers for standalone examples and widgets.
//!
//! Provides common drawing primitives that are duplicated across many
//! standalone binary examples. Import via `use dracon_terminal_engine::framework::helpers::*;`.

mod blit;
mod borders;
mod text;

// Public re-exports — preserves the original `crate::framework::helpers::*` API
// so downstream code does not need to change.
pub use blit::blit_to;
pub use borders::draw_rounded_border;
pub use text::draw_text;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::{Cell, Color, Plane, Styles};

    #[test]
    fn test_draw_text_basic() {
        let mut plane = Plane::new(0, 10, 3);
        draw_text(
            &mut plane,
            0,
            0,
            "Hi",
            Color::Rgb(255, 255, 255),
            Color::Rgb(0, 0, 0),
            false,
        );
        assert_eq!(plane.cells[0].char, 'H');
        assert_eq!(plane.cells[1].char, 'i');
    }

    #[test]
    fn test_draw_text_clips_at_boundary() {
        let mut plane = Plane::new(0, 5, 1);
        draw_text(
            &mut plane,
            3,
            0,
            "Hello",
            Color::Rgb(255, 255, 255),
            Color::Rgb(0, 0, 0),
            false,
        );
        assert_eq!(plane.cells[3].char, 'H');
        assert_eq!(plane.cells[4].char, 'e');
        // "llo" is clipped — would have wrapped to next row in the buggy version
    }

    #[test]
    fn test_draw_text_out_of_bounds_y() {
        let mut plane = Plane::new(0, 10, 2);
        draw_text(
            &mut plane,
            0,
            5,
            "Hi",
            Color::Rgb(255, 255, 255),
            Color::Rgb(0, 0, 0),
            false,
        );
        // Should not panic
    }

    #[test]
    fn test_blit_to_skips_transparent() {
        let mut dest = Plane::new(0, 4, 2);
        let mut src = Plane::new(0, 2, 1);
        src.cells[0] = Cell {
            char: 'A',
            fg: Color::Rgb(255, 255, 255),
            bg: Color::Rgb(0, 0, 0),
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        src.cells[1] = Cell {
            char: 'B',
            fg: Color::Rgb(255, 255, 255),
            bg: Color::Rgb(0, 0, 0),
            style: Styles::empty(),
            transparent: true,
            skip: false,
        };
        blit_to(&mut dest, &src, 0, 0);
        assert_eq!(dest.cells[0].char, 'A');
        // transparent cell was not copied — dest retains its initial value
        assert_ne!(dest.cells[1].char, 'B');
    }

    #[test]
    fn test_blit_to_skips_null() {
        let mut dest = Plane::new(0, 4, 2);
        let mut src = Plane::new(0, 2, 1);
        src.cells[0] = Cell {
            char: 'A',
            fg: Color::Rgb(255, 255, 255),
            bg: Color::Rgb(0, 0, 0),
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        src.cells[1] = Cell {
            char: '\0',
            fg: Color::Rgb(255, 255, 255),
            bg: Color::Rgb(0, 0, 0),
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        blit_to(&mut dest, &src, 0, 0);
        assert_eq!(dest.cells[0].char, 'A');
        // null cell was not copied — dest retains its initial value
        assert_ne!(dest.cells[1].char, 'A');
    }
}

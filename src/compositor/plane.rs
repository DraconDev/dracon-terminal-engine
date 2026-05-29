//! Plane-based rendering types for the terminal compositor.
//!
//! This module provides the core types for compositing-based rendering:
//! - [`Color`] for terminal-aware color representation (Reset, ANSI 256, RGB)
//! - [`Styles`] for text styling flags (bold, italic, underline, etc.)
//! - [`Cell`] for a single terminal cell (character + color + style)
//! - [`Plane`] for a 2D buffer of cells that can be composited

use bitflags::bitflags;
use ratatui::layout::Rect;

/// Terminal color representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    /// Default terminal foreground/background color.
    Reset,
    /// ANSI 256-color palette entry (0–255).
    Ansi(u8),
    /// 24-bit RGB color.
    Rgb(u8, u8, u8),
}

bitflags! {
    #[allow(missing_docs)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct Styles: u8 {
        /// Bold text.
        const BOLD       = 1 << 0;
        /// Dimmed text.
        const DIM        = 1 << 1;
        /// Italic text.
        const ITALIC     = 1 << 2;
        /// Underlined text.
        const UNDERLINE  = 1 << 3;
        /// Blinking text.
        const BLINK      = 1 << 4;
        /// Reversed foreground/background colors.
        const REVERSE    = 1 << 5;
        /// Hidden text (invisible).
        const HIDDEN     = 1 << 6;
        /// Strikethrough text.
        const STRIKETHROUGH = 1 << 7;
    }
}

/// A single cell in the terminal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    /// The character displayed in this cell.
    pub char: char,
    /// Foreground color.
    pub fg: Color,
    /// Background color.
    pub bg: Color,
    /// Text styling flags.
    pub style: Styles,
    /// Whether this cell is transparent (shows content beneath).
    pub transparent: bool,
    /// Whether this cell should be skipped by the renderer (e.g., for wide character padding).
    pub skip: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            char: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: true,
            skip: false,
        }
    }
}

use crate::compositor::filter::Filter;

/// A 2D plane of cells representing a layer in the terminal compositor.
pub struct Plane {
    /// Unique identifier for this plane.
    pub id: usize,
    /// Z-index determining render order (higher = on top).
    pub z_index: i32,
    /// X position of the plane origin.
    pub x: u16,
    /// Y position of the plane origin.
    pub y: u16,
    /// Width of the plane in cells.
    pub width: u16,
    /// Height of the plane in cells.
    pub height: u16,
    /// Grid of cells representing the plane content.
    pub cells: Vec<Cell>,
    /// Whether the plane is visible.
    pub visible: bool,
    /// Opacity multiplier (0.0 to 1.0).
    pub opacity: f32,
    /// Optional filter applied to this plane.
    pub filter: Option<Box<dyn Filter>>,
}

impl Plane {
    /// Creates a new plane with the given id and dimensions.
    /// Minimum dimensions are 1×1 to prevent division-by-zero in downstream code.
    pub fn new(id: usize, width: u16, height: u16) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        Self {
            id,
            z_index: 0,
            x: 0,
            y: 0,
            width,
            height,
            cells: vec![Cell::default(); width as usize * height as usize],
            visible: true,
            opacity: 1.0,
            filter: None,
        }
    }

    /// Sets the absolute position of this plane in the compositor.
    pub fn set_absolute_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    /// Sets the z-index for render ordering.
    pub fn set_z_index(&mut self, z: i32) {
        self.z_index = z;
    }

    /// Safe write char to local plane coordinates
    pub fn put_char(&mut self, x: u16, y: u16, c: char) {
        use unicode_width::UnicodeWidthChar;
        if x >= self.width || y >= self.height {
            return;
        }
        let width = c.width().unwrap_or(0);
        let idx = (y * self.width + x) as usize;
        self.cells[idx].char = c;
        self.cells[idx].transparent = false;
        self.cells[idx].skip = false;
        if width == 2 && x + 1 < self.width {
            let next_idx = idx + 1;
            self.cells[next_idx].char = ' ';
            self.cells[next_idx].transparent = false;
            self.cells[next_idx].skip = true;
        }
    }

    /// Writes a cell to the specified position.
    pub fn put_cell(&mut self, x: u16, y: u16, mut cell: Cell) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        cell.transparent = false;
        self.cells[idx] = cell;
    }

    // Helper to set style
    /// Sets the style (colors and text style) for a cell at the given position.
    pub fn set_style(&mut self, x: u16, y: u16, fg: Color, bg: Color, style: Styles) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        self.cells[idx].fg = fg;
        self.cells[idx].bg = bg;
        self.cells[idx].style = style;
        self.cells[idx].transparent = false;
        self.cells[idx].skip = false;
    }

    /// Sets the skip flag for a cell at the given position.
    pub fn set_skip(&mut self, x: u16, y: u16, skip: bool) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        self.cells[idx].skip = skip;
        if skip {
            self.cells[idx].transparent = false;
        }
    }

    /// Writes a string starting at the given position. Returns the x position after writing.
    pub fn put_str(&mut self, mut x: u16, y: u16, text: &str) -> u16 {
        use crate::text::grapheme_width;

        // SAFETY: `text` is a valid &str, so all byte slices derived from
        // it at valid char boundaries are guaranteed valid UTF-8.
        // We track byte offsets carefully and only advance by `len_utf8()`
        // which always lands on a char boundary.
        let bytes = text.as_bytes();
        let mut byte_offset = 0;

        while byte_offset < bytes.len() {
            if x >= self.width {
                break;
            }

            // SAFETY: `byte_offset` is guaranteed to be on a valid UTF-8 char boundary
            // because it starts at 0 and is advanced only by `char_len` from previous
            // `next_char_unchecked` calls.
            let (c, char_len) = unsafe { next_char_unchecked(bytes, byte_offset) };

            if matches!(c, '\u{1F1E6}'..='\u{1F1FF}') {
                let next_offset = byte_offset + char_len;
                if next_offset < bytes.len() {
                    // SAFETY: `next_offset` is guaranteed to be on a valid UTF-8 char boundary
                    // because it equals `byte_offset + char_len` where `char_len` is the byte
                    // length of the preceding character.
                    let (next_c, next_len) = unsafe { next_char_unchecked(bytes, next_offset) };
                    if matches!(next_c, '\u{1F1E6}'..='\u{1F1FF}') {
                        if x + 1 >= self.width {
                            break;
                        }
                        let idx = (y * self.width + x) as usize;
                        self.cells[idx].char = c;
                        self.cells[idx].transparent = false;
                        self.cells[idx].skip = false;

                        let next_idx = idx + 1;
                        self.cells[next_idx].char = next_c;
                        self.cells[next_idx].transparent = false;
                        self.cells[next_idx].skip = true;

                        x += 2;
                        byte_offset += char_len + next_len;
                        continue;
                    }
                }
                byte_offset += char_len;
                continue;
            }

            let width = grapheme_width(c);

            if width == 0 {
                let consumed = Self::consume_grapheme_cluster(text, byte_offset);
                byte_offset += consumed;
                continue;
            }

            if width == 2 && x + 1 >= self.width {
                break;
            }

            let idx = (y * self.width + x) as usize;
            self.cells[idx].char = c;
            self.cells[idx].transparent = false;
            self.cells[idx].skip = false;

            if width == 2 {
                let next_idx = idx + 1;
                self.cells[next_idx].char = ' ';
                self.cells[next_idx].transparent = false;
                self.cells[next_idx].skip = true;
            }

            let consumed = Self::consume_grapheme_cluster(text, byte_offset);
            byte_offset += consumed;
            x += width as u16;
        }

        x
    }

    /// Consumes the grapheme cluster starting at byte_offset and returns the number of bytes consumed.
    fn consume_grapheme_cluster(text: &str, byte_offset: usize) -> usize {
        use crate::text::grapheme_width;

        // SAFETY: Same rationale as put_str — `text` is valid UTF-8 and
        // byte_offset is always advanced by char_len which lands on a
        // valid char boundary.
        let bytes = text.as_bytes();

        let (c, char_len) = unsafe { next_char_unchecked(bytes, byte_offset) };

        let mut pos = byte_offset;
        if matches!(c, '\u{1F1E6}'..='\u{1F1FF}') {
            return char_len;
        }

        pos += char_len;

        while pos < bytes.len() {
            // SAFETY: `pos` is guaranteed to be on a valid UTF-8 char boundary because
            // it starts at `byte_offset + char_len` and is advanced only by `next_len`
            // from previous `next_char_unchecked` calls.
            let (next_c, next_len) = unsafe { next_char_unchecked(bytes, pos) };

            if next_c == '\u{200D}' {
                pos += next_len;
                continue;
            }

            if matches!(next_c, '\u{1F3FB}'..='\u{1F3FF}') {
                pos += next_len;
                continue;
            }

            if matches!(next_c, '\u{1F1E6}'..='\u{1F1FF}') {
                break;
            }

            if grapheme_width(next_c) == 0 {
                pos += next_len;
                continue;
            }

            break;
        }

        pos - byte_offset
    }

    /// Sets the filter for this plane.
    pub fn set_filter(&mut self, filter: Box<dyn Filter>) {
        self.filter = Some(filter);
    }

    /// Sets all cells to the given transparency state.
    pub fn set_transparent(&mut self, transparent: bool) {
        for cell in &mut self.cells {
            cell.transparent = transparent;
        }
    }

    /// Fills the background color of all cells.
    #[inline]
    pub fn fill_bg(&mut self, bg: Color) {
        for cell in &mut self.cells {
            cell.bg = bg;
            cell.transparent = false;
        }
    }

    /// Resets all cells to their default state.
    #[inline]
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
    }

    /// Blits (copies) non-transparent, non-skip cells from `source` into this plane
    /// at the given destination offset. This is significantly faster than creating
    /// intermediate planes and manually cloning cells, as it avoids allocation entirely.
    ///
    /// Only cells where `source.cells[src].transparent == false && skip == false` are copied.
    /// Cells outside the destination plane's bounds are silently clipped.
    #[inline]
    pub fn blit_from(&mut self, source: &Plane, dest_x: u16, dest_y: u16) {
        let src_w = source.width as usize;
        let src_h = source.height as usize;
        let dst_w = self.width as usize;
        let dst_h = self.height as usize;
        let dx = dest_x as usize;
        let dy = dest_y as usize;

        // Skip entirely if offset is out of bounds
        if dx >= dst_w || dy >= dst_h {
            return;
        }

        let max_rows = (src_h).min(dst_h - dy);
        let max_cols = (src_w).min(dst_w - dx);

        for row in 0..max_rows {
            let src_base = row * src_w;
            let dst_base = (dy + row) * dst_w + dx;
            for col in 0..max_cols {
                let src_idx = src_base + col;
                let src_cell = &source.cells[src_idx];
                if !src_cell.transparent && !src_cell.skip {
                    self.cells[dst_base + col] = *src_cell;
                }
            }
        }
    }

    /// Fast bulk blit that uses memcpy when the source plane is fully opaque.
    ///
    /// This is significantly faster than `blit_from` because it uses
    /// `copy_from_slice` when the source has no transparent cells.
    /// When the source contains transparent cells, it falls back to
    /// the per-cell blit approach.
    ///
    /// # Performance
    ///
    /// - **Fully opaque source, exact dimensions**: Single `copy_from_slice` for O(n) bulk copy
    /// - **Fully opaque source, smaller than dest**: Per-row `copy_from_slice` on the overlapping region
    /// - **Contains transparent cells**: Falls back to `blit_from` behavior
    #[inline]
    pub fn blit_from_fast(&mut self, source: &Plane) {
        if source.cells.iter().all(|c| !c.transparent) {
            if source.width == self.width && source.height == self.height {
                self.cells.copy_from_slice(&source.cells);
            } else if source.width == self.width {
                let copy_rows = (source.height as usize).min(self.height as usize);
                let src_stride = source.width as usize;
                let dst_stride = self.width as usize;
                for row in 0..copy_rows {
                    let src_start = row * src_stride;
                    let src_end = src_start + src_stride;
                    let dst_start = row * dst_stride;
                    let dst_end = dst_start + src_stride;
                    if dst_end <= self.cells.len() && src_end <= source.cells.len() {
                        self.cells[dst_start..dst_end]
                            .copy_from_slice(&source.cells[src_start..src_end]);
                    }
                }
            } else {
                let copy_rows = (source.height as usize).min(self.height as usize);
                let copy_cols = (source.width as usize).min(self.width as usize);
                let src_stride = source.width as usize;
                let dst_stride = self.width as usize;
                for row in 0..copy_rows {
                    let src_start = row * src_stride;
                    let dst_start = row * dst_stride;
                    if dst_start + copy_cols <= self.cells.len()
                        && src_start + copy_cols <= source.cells.len()
                    {
                        self.cells[dst_start..dst_start + copy_cols]
                            .copy_from_slice(&source.cells[src_start..src_start + copy_cols]);
                    }
                }
            }
        } else {
            self.blit_from(source, 0, 0);
        }
    }

    /// Resets all cells to transparent defaults without reallocating the underlying Vec.
    /// Use this to reuse a Plane across frames instead of creating a new one each time.
    pub fn reset_cells(&mut self) {
        for cell in &mut self.cells {
            cell.char = ' ';
            cell.fg = Color::Reset;
            cell.bg = Color::Reset;
            cell.style = Styles::empty();
            cell.transparent = true;
            cell.skip = false;
        }
    }

    /// Extracts a sub-plane from this plane at the specified rectangle.
    ///
    /// Returns a new plane containing only the cells within the given rect.
    /// The returned plane has its position set to the rectangle's position
    /// and shares the same z-index as the source.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use dracon_terminal_engine::compositor::Plane;
    /// use ratatui::layout::Rect;
    /// let full_plane = Plane::new(0, 80, 24);
    /// // ... fill full_plane with content ...
    /// let sub = full_plane.crop(Rect::new(10, 5, 20, 10));
    /// // sub is a 20x10 plane positioned at (10, 5)
    /// ```
    pub fn crop(&self, rect: Rect) -> Plane {
        let rect = rect.intersection(ratatui::layout::Rect::new(0, 0, self.width, self.height));
        let mut plane = Plane::new(self.id, rect.width, rect.height);
        plane.x = rect.x;
        plane.y = rect.y;
        plane.z_index = self.z_index;

        for py in 0..rect.height {
            for px in 0..rect.width {
                let src_idx = ((rect.y + py) * self.width + (rect.x + px)) as usize;
                let dst_idx = (py * rect.width + px) as usize;
                if src_idx < self.cells.len() && dst_idx < plane.cells.len() {
                    plane.cells[dst_idx] = self.cells[src_idx];
                }
            }
        }

        plane
    }
}

/// Reads the next char and its byte length from `bytes` starting at `offset`.
///
/// # Safety
///
/// `offset` must be a valid char boundary within `bytes`, and `bytes` must
/// be valid UTF-8. Both invariants hold when `bytes` comes from a `&str` and
/// `offset` is advanced only by previous `len_utf8()` values.
#[inline]
unsafe fn next_char_unchecked(bytes: &[u8], offset: usize) -> (char, usize) {
    let s = std::str::from_utf8_unchecked(&bytes[offset..]);
    let c = s.chars().next().unwrap_or('\0');
    (c, c.len_utf8())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_new() {
        let plane = Plane::new(1, 80, 24);
        assert_eq!(plane.id, 1);
        assert_eq!(plane.width, 80);
        assert_eq!(plane.height, 24);
        assert_eq!(plane.cells.len(), 80 * 24);
        assert!(plane.visible);
        assert_eq!(plane.opacity, 1.0);
    }

    #[test]
    fn test_plane_new_minimum_dimensions() {
        // Minimum dimensions should be enforced
        let plane = Plane::new(1, 0, 0);
        assert_eq!(plane.width, 1);
        assert_eq!(plane.height, 1);
        assert_eq!(plane.cells.len(), 1);
    }

    #[test]
    fn test_plane_clear() {
        let mut plane = Plane::new(1, 10, 5);
        plane.put_char(2, 2, 'X');
        assert_eq!(plane.cells[2 * 10 + 2].char, 'X');

        plane.clear();
        assert_eq!(plane.cells[2 * 10 + 2].char, ' ');
    }

    #[test]
    fn test_plane_put_char() {
        let mut plane = Plane::new(1, 10, 5);
        plane.put_char(5, 2, 'A');
        assert_eq!(plane.cells[2 * 10 + 5].char, 'A');
    }

    #[test]
    fn test_plane_put_char_out_of_bounds() {
        let mut plane = Plane::new(1, 10, 5);
        // Should not panic
        plane.put_char(100, 100, 'X');
    }

    #[test]
    fn test_plane_get_char() {
        let mut plane = Plane::new(1, 10, 5);
        plane.put_char(3, 1, 'B');
        assert_eq!(plane.get_char(3, 1), Some('B'));
    }

    #[test]
    fn test_plane_get_char_out_of_bounds() {
        let plane = Plane::new(1, 10, 5);
        assert_eq!(plane.get_char(100, 100), None);
    }

    #[test]
    fn test_plane_resize() {
        let mut plane = Plane::new(1, 10, 5);
        plane.put_char(5, 2, 'X');

        plane.resize(20, 10);
        assert_eq!(plane.width, 20);
        assert_eq!(plane.height, 10);
        assert_eq!(plane.cells.len(), 200);
    }

    #[test]
    fn test_plane_set_cell_bg() {
        let mut plane = Plane::new(1, 10, 5);
        plane.set_cell_bg(2, 1, Color::Rgb(255, 0, 0));
        assert_eq!(plane.cells[1 * 10 + 2].bg, Color::Rgb(255, 0, 0));
    }

    #[test]
    fn test_plane_fill_bg() {
        let mut plane = Plane::new(1, 10, 5);
        plane.fill_bg(Color::Ansi(21));
        for cell in &plane.cells {
            assert_eq!(cell.bg, Color::Ansi(21));
        }
    }

    #[test]
    fn test_plane_blit_from() {
        let mut dest = Plane::new(1, 20, 10);
        let mut src = Plane::new(2, 5, 3);
        src.put_char(1, 1, 'T');

        dest.blit_from(&src, 2, 2);
        assert_eq!(dest.get_char(3, 3), Some('T'));
    }

    #[test]
    fn test_plane_blit_from_fast_opaque() {
        let mut dest = Plane::new(1, 10, 5);
        let src = Plane::new(2, 10, 5);

        // Fast blit should work for fully opaque planes
        dest.blit_from_fast(&src);
    }

    #[test]
    fn test_plane_crop() {
        let mut plane = Plane::new(1, 20, 10);
        plane.put_char(15, 5, 'C');

        let cropped = plane.crop(Rect::new(10, 3, 10, 5));
        assert_eq!(cropped.width, 10);
        assert_eq!(cropped.height, 5);
        assert_eq!(cropped.x, 10);
        assert_eq!(cropped.y, 3);
        assert_eq!(cropped.get_char(5, 2), Some('C'));
    }

    #[test]
    fn test_plane_crop_out_of_bounds() {
        let plane = Plane::new(1, 10, 5);
        // Should clamp to plane bounds
        let cropped = plane.crop(Rect::new(100, 100, 50, 50));
        assert_eq!(cropped.width, 10);
        assert_eq!(cropped.height, 5);
    }

    #[test]
    fn test_plane_set_position() {
        let mut plane = Plane::new(1, 20, 10);
        plane.set_position(50, 30);
        assert_eq!(plane.x, 50);
        assert_eq!(plane.y, 30);
    }

    #[test]
    fn test_plane_z_index() {
        let mut plane = Plane::new(1, 20, 10);
        assert_eq!(plane.z_index, 0);

        plane.z_index = 5;
        assert_eq!(plane.z_index, 5);
    }

    #[test]
    fn test_plane_visibility() {
        let mut plane = Plane::new(1, 20, 10);
        assert!(plane.visible);

        plane.visible = false;
        assert!(!plane.visible);
    }

    #[test]
    fn test_plane_opacity() {
        let mut plane = Plane::new(1, 20, 10);
        assert_eq!(plane.opacity, 1.0);

        plane.opacity = 0.5;
        assert_eq!(plane.opacity, 0.5);
    }

    #[test]
    fn test_plane_id() {
        let plane = Plane::new(42, 20, 10);
        assert_eq!(plane.id, 42);
    }

    #[test]
    fn test_cell_default() {
        let cell = Cell::default();
        assert_eq!(cell.char, ' ');
        assert_eq!(cell.fg, Color::Reset);
        assert_eq!(cell.bg, Color::Reset);
        assert!(cell.transparent);
        assert!(!cell.skip);
        assert_eq!(cell.style, Styles::empty());
    }

    #[test]
    fn test_cell_clone() {
        let cell = Cell {
            char: 'X',
            fg: Color::Rgb(255, 0, 0),
            bg: Color::Rgb(0, 0, 255),
            transparent: false,
            skip: false,
            style: Styles::BOLD,
        };
        let cloned = cell;
        assert_eq!(cloned.char, 'X');
        assert_eq!(cloned.fg, Color::Rgb(255, 0, 0));
    }
}

use bitflags::bitflags;

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

        let bytes = text.as_bytes();
        let mut byte_offset = 0;

        while byte_offset < bytes.len() {
            if x >= self.width {
                break;
            }

            // Get the grapheme cluster info
            let remaining = &bytes[byte_offset..];
            let Some((c, char_len)) = std::str::from_utf8(remaining)
                .ok()
                .and_then(|s| s.chars().next())
                .map(|c| (c, c.len_utf8()))
            else {
                // Invalid UTF-8, skip byte
                byte_offset += 1;
                x += 1;
                continue;
            };

            // Check if this is a regional indicator pair (flag emoji)
            if matches!(c, '\u{1F1E6}'..='\u{1F1FF}') {
                let next_offset = byte_offset + char_len;
                if next_offset < bytes.len() {
                    if let Some(next_c) = std::str::from_utf8(&bytes[next_offset..])
                        .ok()
                        .and_then(|s| s.chars().next())
                    {
                        let next_len = next_c.len_utf8();
                        if matches!(next_c, '\u{1F1E6}'..='\u{1F1FF}') {
                            // Flag emoji: write both regional indicators
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
                }
                // Single regional indicator: skip (zero-width)
                byte_offset += char_len;
                continue;
            }

            // Get width of this grapheme cluster
            let width = grapheme_width(c);

            // Skip zero-width characters
            if width == 0 {
                // Skip combining marks, ZWJ, etc. by consuming them
                let consumed = Self::consume_grapheme_cluster(text, byte_offset);
                byte_offset += consumed;
                continue;
            }

            // If it's a wide char (width 2), we need to ensure space
            if width == 2 && x + 1 >= self.width {
                break;
            }

            // Write the character
            let idx = (y * self.width + x) as usize;
            self.cells[idx].char = c;
            self.cells[idx].transparent = false;
            self.cells[idx].skip = false;

            // Mark next cell as padding for wide characters
            if width == 2 {
                let next_idx = idx + 1;
                self.cells[next_idx].char = ' ';
                self.cells[next_idx].transparent = false;
                self.cells[next_idx].skip = true;
            }

            // Consume the full grapheme cluster (including combining marks)
            let consumed = Self::consume_grapheme_cluster(text, byte_offset);
            byte_offset += consumed;
            x += width as u16;
        }

        x
    }

    /// Consumes the grapheme cluster starting at byte_offset and returns the number of bytes consumed.
    fn consume_grapheme_cluster(text: &str, byte_offset: usize) -> usize {
        use crate::text::grapheme_width;

        let bytes = text.as_bytes();
        let remaining = &bytes[byte_offset..];

        let Some((_c, char_len)) = std::str::from_utf8(remaining)
            .ok()
            .and_then(|s| s.chars().next())
            .map(|c| (c, c.len_utf8()))
        else {
            return 1; // Invalid UTF-8, consume 1 byte
        };

        // Skip regional indicators (flags) - they only start clusters, don't extend them
        let mut pos = byte_offset;
        if matches!(_c, '\u{1F1E6}'..='\u{1F1FF}') {
            // Skip the single RI, let the main loop handle pairs
            return char_len;
        }

        pos += char_len;

        while pos < bytes.len() {
            let rem = &bytes[pos..];
            let Some((next_c, next_len)) = std::str::from_utf8(rem)
                .ok()
                .and_then(|s| s.chars().next())
                .map(|c| (c, c.len_utf8()))
            else {
                break;
            };

            // ZWJ continues the cluster
            if next_c == '\u{200D}' {
                pos += next_len;
                continue;
            }

            // Skin tone modifiers extend the cluster
            if matches!(next_c, '\u{1F3FB}'..='\u{1F3FF}') {
                pos += next_len;
                continue;
            }

            // Regional indicator starts a new grapheme
            if matches!(next_c, '\u{1F1E6}'..='\u{1F1FF}') {
                break;
            }

            // Zero-width characters are part of this cluster
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
    pub fn fill_bg(&mut self, bg: Color) {
        for cell in &mut self.cells {
            cell.bg = bg;
            cell.transparent = false;
        }
    }

    /// Resets all cells to their default state.
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
                    self.cells[dst_base + col] = src_cell.clone();
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
    /// - **Fully opaque source**: Uses `copy_from_slice` for O(n) bulk copy
    /// - **Contains transparent cells**: Falls back to `blit_from` behavior
    ///
    /// # Panics
    ///
    /// Panics if source dimensions differ from destination blit area dimensions.
    pub fn blit_from_fast(&mut self, source: &Plane) {
        // Fast path: if all cells are opaque, use bulk copy
        if source.cells.iter().all(|c| !c.transparent) {
            // Validate dimensions match for bulk copy
            if source.width == self.width && source.height == self.height {
                self.cells.copy_from_slice(&source.cells);
            } else {
                // Dimensions differ, need to blit with offset (0, 0)
                self.blit_from(source, 0, 0);
            }
        } else {
            // Fallback: use per-cell blit for transparent cell handling
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
}
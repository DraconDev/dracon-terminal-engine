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
#[derive(Clone, Debug, PartialEq)]
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
        let bytes = text.as_bytes();
        let remaining = &bytes[byte_offset..];

        let Some((_c, char_len)) = std::str::from_utf8(remaining)
            .ok()
            .and_then(|s| s.chars().next())
            .map(|c| (c, c.len_utf8()))
        else {
            return 1; // Invalid UTF-8, consume 1 byte
        };

        let mut pos = byte_offset + char_len;

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

            // Regional indicator after base? Don't consume (start of new grapheme)
            if matches!(next_c, '\u{1F1E6}'..='\u{1F1FF}') {
                break;
            }

            // Zero-width characters are part of this cluster
            if crate::text::grapheme_width(next_c) == 0 {
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
}
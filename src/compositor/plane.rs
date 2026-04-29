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
    pub fn new(id: usize, width: u16, height: u16) -> Self {
        Self {
            id,
            z_index: 0,
            x: 0,
            y: 0,
            width,
            height,
            cells: vec![Cell::default(); (width * height) as usize],
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
        use unicode_width::UnicodeWidthChar;

        for c in text.chars() {
            if x >= self.width {
                break;
            }

            let width = c.width().unwrap_or(0);
            if width == 0 {
                continue;
            } // Skip zero-width chars

            // If it's a wide char (width 2), we need to ensure space
            if width == 2 && x + 1 >= self.width {
                break;
            }

            self.put_char(x, y, c);

            if width == 2 {
                let idx = (y * self.width + x + 1) as usize;
                if idx < self.cells.len() {
                    self.cells[idx].char = ' ';
                    self.cells[idx].transparent = false;
                    self.cells[idx].skip = true;
                }
            }

            x += width as u16;
        }
        x
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

    /// Resets all cells to their default state.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
    }
}
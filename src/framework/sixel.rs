//! Sixel image support for sixel-encoded graphics.
//!
//! Provides types and utilities for rendering sixel-encoded images
//! in the terminal. Sixel is a bitmap graphics format using
//! six-element encoded patterns.
//!
//! # Current Limitations
//!
//! **Sixel decoding is NOT implemented.** The [`SixelImage::from_sixel`] method
//! returns `Err("Sixel decoding not yet implemented")`. You can:
//!
//! - Create images programmatically via [`SixelImage::new`] + [`set_pixel`](SixelImage::set_pixel)
//! - Use the [`SixelRenderer`] widget to display pixel data
//!
//! To decode actual sixel data, integrate an external crate such as
//! `sixel-rs` or `libimagequant` and feed the resulting RGB buffer
//! into `SixelImage::new` + `set_pixel`.
//!
//! # Feature Gate
//!
//! This module is behind the `sixel` feature flag. Enable it in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! dracon-terminal-engine = { version = "0.1", features = ["sixel"] }
//! ```

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A sixel-encoded image.
pub struct SixelImage {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

impl SixelImage {
    /// Creates a new sixel image with the given dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0; width * height * 3],
            width,
            height,
        }
    }

    /// Creates a sixel image from encoded data.
    ///
    /// **Not yet implemented.** This method always returns
    /// `Err("Sixel decoding not yet implemented")`. To load actual sixel
    /// data, decode it externally (e.g. with `sixel-rs` or `libimagequant`)
    /// and feed the resulting RGB buffer into [`SixelImage::new`] and
    /// [`SixelImage::set_pixel`].
    #[deprecated(
        since = "0.2.0",
        note = "Sixel decoding is not implemented. Decode externally and use SixelImage::new + set_pixel to populate the buffer."
    )]
    pub fn from_sixel(_data: &[u8]) -> Result<Self, &'static str> {
        Err("Sixel decoding not yet implemented")
    }

    /// Returns the width of the image in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the image in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns the RGB pixel value at the given coordinates.
    pub fn pixel(&self, x: usize, y: usize) -> Option<(u8, u8, u8)> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) * 3;
        Some((self.data[idx], self.data[idx + 1], self.data[idx + 2]))
    }

    /// Sets the RGB pixel value at the given coordinates.
    pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) * 3;
        self.data[idx] = r;
        self.data[idx + 1] = g;
        self.data[idx + 2] = b;
    }
}

/// A widget that renders sixel images.
pub struct SixelRenderer {
    id: WidgetId,
    image: Option<SixelImage>,
    area: std::cell::Cell<Rect>,
}

impl SixelRenderer {
    /// Creates a new sixel renderer with the given widget ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            image: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 20)),
        }
    }

    /// Sets the image to render.
    pub fn with_image(mut self, image: SixelImage) -> Self {
        self.image = Some(image);
        self
    }

    /// Sets the image to render.
    pub fn set_image(&mut self, image: SixelImage) {
        self.image = Some(image);
    }

    /// Loads a sixel image from encoded data.
    ///
    /// **Not yet implemented.** See [`SixelImage::from_sixel`].
    #[allow(deprecated)]
    #[deprecated(
        since = "0.2.0",
        note = "Sixel decoding is not implemented. Decode externally and use with_image/set_image with a programmatically-populated SixelImage."
    )]
    pub fn load_sixel(&mut self, data: &[u8]) -> Result<(), &'static str> {
        self.image = Some(SixelImage::from_sixel(data)?);
        Ok(())
    }
}

impl crate::framework::widget::Widget for SixelRenderer {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 5;

        if let Some(ref image) = self.image {
            let scale_x = image.width() as f32 / area.width as f32;
            let scale_y = image.height() as f32 / area.height as f32;

            for y in 0..area.height as usize {
                for x in 0..area.width as usize {
                    let img_x = (x as f32 * scale_x) as usize;
                    let img_y = (y as f32 * scale_y) as usize;
                    if let Some((r, g, b)) = image.pixel(img_x, img_y) {
                        let idx = (y as u16 * plane.width + x as u16) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx] = Cell {
                                char: ' ',
                                fg: Color::Rgb(r, g, b),
                                bg: Color::Rgb(r, g, b),
                                style: Styles::empty(),
                                transparent: false,
                                skip: false,
                            };
                        }
                    }
                }
            }
        }

        plane
    }
}

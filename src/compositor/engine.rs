use crate::compositor::plane::{Cell, Color, Plane, Styles};
use crate::framework::dirty_regions::DirtyRegionTracker;
use std::io::{self, Write};

/// Composites multiple planes into a single render target.
pub struct Compositor {
    /// The planes to composite, ordered by z-index.
    pub planes: Vec<Plane>,
    width: u16,
    height: u16,
    last_frame: Vec<Cell>,
    /// Reusable final composition buffer (avoids per-frame allocation).
    final_buffer: Vec<Cell>,
    /// Background color for cells not covered by any plane.
    /// Set this to the theme background to avoid black gaps.
    clear_color: Color,
    /// Last frame duration in milliseconds.
    last_frame_duration_ms: f64,
    /// Number of registered widgets (for metrics).
    widget_count: usize,
    /// Dirty region tracker for partial screen updates.
    dirty_regions: DirtyRegionTracker,
}

impl Compositor {
    /// Creates a new Compositor with the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as u32 * height as u32) as usize;
        let default_cell = Cell {
            bg: Color::Reset,
            transparent: false,
            ..Cell::default()
        };
        Self {
            planes: Vec::new(),
            width,
            height,
            last_frame: vec![Cell::default(); size],
            final_buffer: vec![default_cell; size],
            clear_color: Color::Reset,
            last_frame_duration_ms: 0.0,
            widget_count: 0,
            dirty_regions: DirtyRegionTracker::new(),
        }
    }

    /// Returns the number of registered widgets.
    pub fn widget_count(&self) -> usize {
        self.widget_count
    }

    /// Sets the number of registered widgets (called by App).
    pub fn set_widget_count(&mut self, count: usize) {
        self.widget_count = count;
    }

    /// Returns the last frame duration in milliseconds.
    pub fn last_frame_duration_ms(&self) -> f64 {
        self.last_frame_duration_ms
    }

    /// Sets the last frame duration (called by App after each frame).
    pub fn set_last_frame_duration(&mut self, ms: f64) {
        self.last_frame_duration_ms = ms;
    }

    /// Returns the current clear color used for uncovered cells.
    pub fn clear_color(&self) -> Color {
        self.clear_color
    }

    /// Sets the background color for cells not covered by any plane.
    /// Set this to the theme background to avoid black gaps.
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Clears the last frame buffer, forcing a full redraw on next render.
    /// Call this after externally clearing the terminal (e.g., resume from suspend).
    pub fn invalidate_last_frame(&mut self) {
        self.last_frame = vec![Cell::default(); (self.width as u32 * self.height as u32) as usize];
    }

    /// Advances the compositor state by one frame.
    pub fn tick(&mut self, _delta: f32) {}

    /// Returns the topmost visible plane at the given coordinates, if any.
    /// Scans all planes and returns the one with the highest z-index that
    /// contains a non-transparent cell at (x, y).
    pub fn hit_test(&self, x: u16, y: u16) -> Option<&Plane> {
        let mut best: Option<&Plane> = None;
        for plane in self.planes.iter() {
            if !plane.visible {
                continue;
            }
            if x >= plane.x
                && x < plane.x.saturating_add(plane.width)
                && y >= plane.y
                && y < plane.y.saturating_add(plane.height)
            {
                let lx = x - plane.x;
                let ly = y - plane.y;
                let idx = (ly * plane.width + lx) as usize;
                if !plane.cells[idx].transparent {
                    match best {
                        None => best = Some(plane),
                        Some(prev) if plane.z_index > prev.z_index => best = Some(plane),
                        _ => {}
                    }
                }
            }
        }
        best
    }

    /// Returns the width and height of the compositor.
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Adds a plane to the compositor, inserting it at the correct z-index position.
    pub fn add_plane(&mut self, plane: Plane) {
        self.planes.push(plane);
    }

    /// Draws text at the specified position with the given colors and style.
    pub fn draw_text(&mut self, text: &str, x: u16, y: u16, fg: Color, bg: Color, style: Styles) {
        use unicode_width::UnicodeWidthStr;
        let visual_width = text.width() as u16;
        if visual_width == 0 {
            return;
        }
        let mut plane = Plane::new(0, visual_width, 1);
        plane.x = x;
        plane.y = y;
        plane.z_index = 10;

        plane.put_str(0, 0, text);
        for cell in plane.cells.iter_mut() {
            if !cell.transparent {
                cell.fg = fg;
                cell.bg = bg;
                cell.style = style;
            }
        }
        self.add_plane(plane);
    }

    /// Draws a filled rectangle at the specified position with the given character, colors, and style.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_rect(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        char: char,
        fg: Color,
        bg: Color,
        style: Styles,
    ) {
        let mut plane = Plane::new(0, width, height);
        plane.x = x;
        plane.y = y;
        plane.z_index = 5;

        let cell = Cell {
            char,
            fg,
            bg,
            style,
            transparent: false,
            skip: false,
        };

        for i in 0..plane.cells.len() {
            plane.cells[i] = cell;
        }
        self.add_plane(plane);
    }

    /// Clears the terminal and resets the internal frame buffer.
    pub fn force_clear(&mut self) {
        if let Some(base) = self.planes.first_mut() {
            base.clear();
        }
        for cell in &mut self.last_frame {
            cell.char = '\x01';
        }
    }

    /// Draws a ratatui Line at the specified position.
    pub fn draw_ratatui_line(&mut self, line: &ratatui::text::Line, x: u16, y: u16) {
        let total_len: usize = line.spans.iter().map(|s| s.content.len()).sum();
        if total_len == 0 {
            return;
        }

        let mut plane = Plane::new(0, total_len as u16, 1);
        plane.x = x;
        plane.y = y;
        plane.z_index = 10;

        let mut idx = 0;
        for span in &line.spans {
            let fg = map_color(span.style.fg.unwrap_or(ratatui::style::Color::Reset));
            let bg = map_color(span.style.bg.unwrap_or(ratatui::style::Color::Reset));
            let mut style = crate::compositor::plane::Styles::empty();
            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::BOLD)
            {
                style.insert(crate::compositor::plane::Styles::BOLD);
            }
            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::ITALIC)
            {
                style.insert(crate::compositor::plane::Styles::ITALIC);
            }
            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::UNDERLINED)
            {
                style.insert(crate::compositor::plane::Styles::UNDERLINE);
            }

            for c in span.content.chars() {
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg,
                        bg,
                        style,
                        transparent: false,
                        skip: false,
                    };
                    idx += 1;
                }
            }
        }
        self.add_plane(plane);
    }

    /// Resizes the compositor to the given dimensions, resetting the frame buffer.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        let size = (width as u32 * height as u32) as usize;
        self.last_frame = vec![Cell::default(); size];
        let default_cell = Cell {
            bg: self.clear_color,
            transparent: false,
            ..Cell::default()
        };
        self.final_buffer = vec![default_cell; size];
    }

    /// Sets dirty region info from the given tracker for partial rendering.
    /// The compositor copies the relevant state; the caller retains ownership.
    pub fn set_dirty_regions(&mut self, regions: &DirtyRegionTracker) {
        self.dirty_regions.clear();
        if regions.needs_full_refresh() {
            self.dirty_regions.mark_all_dirty();
        } else {
            for r in regions.dirty_regions() {
                self.dirty_regions.mark_dirty(r.x, r.y, r.width, r.height);
            }
        }
    }

    #[inline]
    fn sort_planes(&mut self) {
        self.planes.sort_by_key(|a| a.z_index);
    }

    /// Renders the compositor state to the given writer, outputting terminal escape codes.
    #[inline]
    pub fn render<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        let render_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        let clear_cell = Cell {
            bg: self.clear_color,
            transparent: false,
            ..Cell::default()
        };

        let full_refresh = self.dirty_regions.needs_full_refresh();
        let regions = self.dirty_regions.dirty_regions().to_vec();

        if full_refresh || regions.is_empty() {
            for cell in self.final_buffer.iter_mut() {
                *cell = clear_cell;
            }

            self.sort_planes();

            for plane in &self.planes {
                if !plane.visible {
                    continue;
                }
                
                // Pre-compute bounds and strides for this plane
                let px_end = plane.width.min(self.width.saturating_sub(plane.x)) as usize;
                let py_end = plane.height.min(self.height.saturating_sub(plane.y)) as usize;
                let plane_stride = plane.width as usize;
                let dest_stride = self.width as usize;
                let base_y = plane.y as usize;
                let base_x = plane.x as usize;
                let plane_cells = &plane.cells;
                let opacity = plane.opacity;
                
                // Ultra-fast path: fully opaque plane, no filter, no blending needed
                if opacity >= 1.0 && plane.filter.is_none() {
                    for py in 0..py_end {
                        let src_row_base = py * plane_stride;
                        let dest_row_base = (base_y + py) * dest_stride;
                        for px in 0..px_end {
                            let src_idx = src_row_base + px;
                            let dest_idx = dest_row_base + base_x + px;
                            let src_cell = &plane_cells[src_idx];
                            let dest_cell = &mut self.final_buffer[dest_idx];
                            
                            // Direct copy for fully opaque, non-transparent cells
                            if !src_cell.transparent {
                                if src_cell.skip {
                                    dest_cell.skip = true;
                                    dest_cell.char = ' ';
                                } else {
                                    dest_cell.char = src_cell.char;
                                    dest_cell.fg = src_cell.fg;
                                    dest_cell.style = src_cell.style;
                                    dest_cell.skip = false;
                                }
                                if src_cell.bg != Color::Reset {
                                    dest_cell.bg = src_cell.bg;
                                }
                                dest_cell.transparent = false;
                            }
                        }
                    }
                } else {
                    // Full blend path
                    if plane.filter.is_none() {
                        for py in 0..py_end {
                            let src_row_base = py * plane_stride;
                            let dest_row_base = (base_y + py) * dest_stride;
                            for px in 0..px_end {
                                let src_idx = src_row_base + px;
                                let dest_idx = dest_row_base + base_x + px;
                                let src_cell = &plane_cells[src_idx];
                                blend_cells(&mut self.final_buffer[dest_idx], src_cell, opacity);
                            }
                        }
                    } else {
                        let plane_filter = plane.filter.as_ref();
                        for py in 0..py_end {
                            let src_row_base = py * plane_stride;
                            let dest_row_base = (base_y + py) * dest_stride;
                            for px in 0..px_end {
                                let src_idx = src_row_base + px;
                                let dest_idx = dest_row_base + base_x + px;
                                let mut src_cell = plane_cells[src_idx];
                                if let Some(filter) = plane_filter {
                                    filter.apply(&mut src_cell, (base_x + px) as u16, (base_y + py) as u16, render_time as f32);
                                }
                                blend_cells(&mut self.final_buffer[dest_idx], &src_cell, opacity);
                            }
                        }
                    }
                }
            }
        } else {
            for region in &regions {
                let y_end = (region.y + region.height).min(self.height);
                let x_end = (region.x + region.width).min(self.width);
                for y in region.y..y_end {
                    for x in region.x..x_end {
                        let idx = (y * self.width + x) as usize;
                        self.final_buffer[idx] = clear_cell;
                    }
                }
            }

            self.sort_planes();

            for plane in &self.planes {
                if !plane.visible {
                    continue;
                }
                for py in 0..plane.height {
                    let abs_y = plane.y.saturating_add(py);
                    if abs_y >= self.height {
                        continue;
                    }
                    for px in 0..plane.width {
                        let abs_x = plane.x.saturating_add(px);
                        if abs_x >= self.width {
                            continue;
                        }

                        let mut in_dirty = false;
                        for region in &regions {
                            if abs_x >= region.x
                                && abs_x < region.x + region.width
                                && abs_y >= region.y
                                && abs_y < region.y + region.height
                            {
                                in_dirty = true;
                                break;
                            }
                        }
                        if !in_dirty {
                            continue;
                        }

                        let src_idx = (py * plane.width + px) as usize;
                        let dest_idx = (abs_y * self.width + abs_x) as usize;
                        let mut src_cell = plane.cells[src_idx];

                        if let Some(filter) = &plane.filter {
                            filter.apply(&mut src_cell, abs_x, abs_y, render_time as f32);
                        }

                        blend_cells(&mut self.final_buffer[dest_idx], &src_cell, plane.opacity);
                    }
                }
            }
        }

        // Buffer all output into a Vec<u8> and issue a single write_all() call.
        let mut buf: Vec<u8> = Vec::with_capacity(self.width as usize * self.height as usize * 20);

        write!(buf, "\x1b[?2026h")?;

        let mut current_fg = Color::Reset;
        let mut current_bg = Color::Reset;
        let mut current_style = Styles::empty();

        write!(buf, "\x1b[?7l")?;

        let check_cell = |x: u16, y: u16, regions: &[crate::framework::dirty_regions::DirtyRegion]| -> bool {
            if full_refresh || regions.is_empty() {
                return true;
            }
            for region in regions {
                if x >= region.x
                    && x < region.x + region.width
                    && y >= region.y
                    && y < region.y + region.height
                {
                    return true;
                }
            }
            false
        };

        for y in 0..self.height {
            let mut line_cursor_moved = false;
            let row_base = y as usize * self.width as usize;
            let last_row_base = y as usize * self.width as usize;
            
            for x in 0..self.width {
                let idx = row_base + x as usize;
                let cell = &self.final_buffer[idx];
                let last_cell = &self.last_frame[last_row_base + x as usize];

                if cell.skip {
                    continue;
                }

                if full_refresh || regions.is_empty() {
                    if cell == last_cell {
                        line_cursor_moved = false;
                        continue;
                    }
                } else {
                    // Inline dirty check
                    let mut in_dirty = false;
                    for region in &regions {
                        if x >= region.x && x < region.x + region.width &&
                           y >= region.y && y < region.y + region.height {
                            in_dirty = true;
                            break;
                        }
                    }
                    if !in_dirty && cell == last_cell {
                        line_cursor_moved = false;
                        continue;
                    }
                }

                if !line_cursor_moved {
                    write!(buf, "\x1b[{};{}H", y + 1, x + 1)?;
                    line_cursor_moved = true;
                }

                if cell.style != current_style {
                    let diff = cell.style ^ current_style;
                    if diff.contains(Styles::BOLD) {
                        if cell.style.contains(Styles::BOLD) {
                            buf.extend_from_slice(b"\x1b[1m");
                        } else {
                            buf.extend_from_slice(b"\x1b[22m");
                        }
                    }
                    if diff.contains(Styles::ITALIC) {
                        if cell.style.contains(Styles::ITALIC) {
                            buf.extend_from_slice(b"\x1b[3m");
                        } else {
                            buf.extend_from_slice(b"\x1b[23m");
                        }
                    }
                    if diff.contains(Styles::UNDERLINE) {
                        if cell.style.contains(Styles::UNDERLINE) {
                            buf.extend_from_slice(b"\x1b[4m");
                        } else {
                            buf.extend_from_slice(b"\x1b[24m");
                        }
                    }
                    current_style = cell.style;
                }

                if cell.fg != current_fg {
                    match cell.fg {
                        Color::Reset => buf.extend_from_slice(b"\x1b[39m"),
                        Color::Ansi(c) => write!(buf, "\x1b[38;5;{}m", c)?,
                        Color::Rgb(r, g, b) => write!(buf, "\x1b[38;2;{};{};{}m", r, g, b)?,
                    }
                    current_fg = cell.fg;
                }
                if cell.bg != current_bg {
                    match cell.bg {
                        Color::Reset => buf.extend_from_slice(b"\x1b[49m"),
                        Color::Ansi(c) => write!(buf, "\x1b[48;5;{}m", c)?,
                        Color::Rgb(r, g, b) => write!(buf, "\x1b[48;2;{};{};{}m", r, g, b)?,
                    }
                    current_bg = cell.bg;
                }
                write!(buf, "{}", cell.char)?;
            }
        }

        write!(buf, "\x1b[?7h")?;
        write!(buf, "\x1b[?2026l")?;

        writer.write_all(&buf)?;

        self.last_frame.clone_from_slice(&self.final_buffer);
        self.planes.clear();
        self.dirty_regions.clear();
        writer.flush()?;
        Ok(())
    }
}

#[inline]
fn blend_cells(dest: &mut Cell, src: &Cell, alpha: f32) {
    if src.transparent || alpha <= 0.0 {
        return;
    }

    if alpha >= 1.0 {
        if src.bg != Color::Reset {
            dest.bg = src.bg;
        }

        if src.skip {
            dest.skip = true;
            dest.char = ' ';
        } else if src.char != '\0' {
            if is_braille(dest.char) && is_braille(src.char) {
                dest.char = merge_braille(dest.char, src.char);
            } else {
                dest.char = src.char;
            }
            dest.fg = src.fg;
            dest.style = src.style;
            dest.skip = false;
        }
    } else {
        let blend = |c1: Color, c2: Color, a: f32| -> Color {
            match (c1, c2) {
                (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => Color::Rgb(
                    ((r1 as f32 * (1.0 - a)) + (r2 as f32 * a)).clamp(0.0, 255.0) as u8,
                    ((g1 as f32 * (1.0 - a)) + (g2 as f32 * a)).clamp(0.0, 255.0) as u8,
                    ((b1 as f32 * (1.0 - a)) + (b2 as f32 * a)).clamp(0.0, 255.0) as u8,
                ),
                (_, c) => {
                    if a > 0.5 {
                        c
                    } else {
                        c1
                    }
                }
            }
        };

        if src.bg != Color::Reset {
            dest.bg = blend(dest.bg, src.bg, alpha);
        }

        if src.skip {
            if alpha > 0.5 {
                dest.skip = true;
                dest.char = ' ';
            }
        } else if src.char != '\0' {
            dest.fg = blend(dest.fg, src.fg, alpha);

            if alpha > 0.5 {
                dest.char = src.char;
                dest.style = src.style;
                dest.skip = false;
            }
        }
    }

    dest.transparent = false;
}

#[inline]
fn is_braille(c: char) -> bool {
    let u = c as u32;
    (0x2800..=0x28FF).contains(&u)
}

fn merge_braille(c1: char, c2: char) -> char {
    let b1 = (c1 as u32) & 0xFF;
    let b2 = (c2 as u32) & 0xFF;
    std::char::from_u32(0x2800 | (b1 | b2)).unwrap_or(c1)
}

/// Converts a ratatui Color into a compositor Color.
pub fn map_color(c: ratatui::style::Color) -> Color {
    use ratatui::style::Color as RColor;
    match c {
        RColor::Reset => Color::Reset,
        RColor::Black => Color::Rgb(0, 0, 0),
        RColor::Red => Color::Rgb(255, 0, 85),
        RColor::Green => Color::Rgb(0, 255, 150),
        RColor::Yellow => Color::Rgb(255, 255, 0),
        RColor::Blue => Color::Rgb(0, 150, 255),
        RColor::Magenta => Color::Rgb(255, 0, 255),
        RColor::Cyan => Color::Rgb(0, 255, 200),
        RColor::Gray => Color::Rgb(180, 180, 180),
        RColor::DarkGray => Color::Rgb(60, 60, 70),
        RColor::LightRed => Color::Rgb(255, 100, 100),
        RColor::LightGreen => Color::Rgb(100, 255, 100),
        RColor::LightYellow => Color::Rgb(255, 255, 150),
        RColor::LightBlue => Color::Rgb(150, 150, 255),
        RColor::LightMagenta => Color::Rgb(255, 150, 255),
        RColor::LightCyan => Color::Rgb(150, 255, 255),
        RColor::White => Color::Rgb(255, 255, 255),
        RColor::Indexed(i) => Color::Ansi(i),
        RColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

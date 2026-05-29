//! Shared helper functions for embedded scenes.
//!
//! These are extracted from duplicated local definitions across all scene files
//! to reduce code duplication and ensure consistent behavior.

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};

/// Draws a line of text at (x, y) with foreground, background, and optional bold styling.
/// Clips at the plane's right boundary — text that would overflow the row is truncated.
/// This prevents text from wrapping into the next row, which was a common bug with
/// the old flat-indexing approach.
pub fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    if y >= plane.height {
        return;
    }
    let max_x = plane.width;
    let style = if bold { Styles::BOLD } else { Styles::empty() };
    for (i, ch) in text.chars().enumerate() {
        let cx = x + i as u16;
        if cx >= max_x {
            break; // clip at right boundary
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

/// Draws text clipped to a column boundary (max_x). Text beyond max_x is replaced with "…".
/// Use this when rendering inside a panel/column to prevent text from bleeding into
/// adjacent panels.
#[allow(clippy::too_many_arguments)]
pub fn draw_text_clipped(
    plane: &mut Plane,
    x: u16,
    y: u16,
    text: &str,
    max_x: u16,
    fg: Color,
    bg: Color,
    bold: bool,
) {
    if y >= plane.height || x >= max_x {
        return;
    }
    let available = (max_x - x) as usize;
    if available == 0 {
        return;
    }
    let char_count = text.chars().count();
    if char_count <= available {
        draw_text(plane, x, y, text, fg, bg, bold);
    } else {
        // Truncate and add ellipsis
        let truncated: String = text.chars().take(available.saturating_sub(1)).collect();
        draw_text(plane, x, y, &truncated, fg, bg, bold);
        let ellipsis_x = x + truncated.len() as u16;
        if ellipsis_x < max_x {
            draw_text(plane, ellipsis_x, y, "…", fg, bg, bold);
        }
    }
}

/// Copies cells from `src` plane into `dest` at the given offset.
/// Skips transparent cells and cells with '\0' character.
/// Bounds-checks to prevent out-of-range writes.
pub fn blit_to(dest: &mut Plane, src: &Plane, offset_x: usize, offset_y: usize) {
    if dest.cells.is_empty() || src.cells.is_empty() {
        return;
    }
    for i in 0..src.cells.len() {
        let cell = &src.cells[i];
        if cell.char == '\0' || cell.transparent {
            continue;
        }
        // Skip cells with Color::Reset bg — these are unfilled/default cells
        // that would override the destination's themed background with the
        // terminal's default (usually white), causing visible white lines.
        if cell.bg == Color::Reset {
            continue;
        }
        let row = i / src.width as usize;
        let col = i % src.width as usize;
        let dy = offset_y + row;
        let dx = offset_x + col;
        if dy >= dest.height as usize || dx >= dest.width as usize {
            continue;
        }
        let idx = dy * dest.width as usize + dx;
        if idx < dest.cells.len() {
            dest.cells[idx] = *cell;
        }
    }
}

/// Renders a centered help overlay with rounded border, title, and shortcut list.
///
/// - `title`: displayed centered at top with primary color + BOLD
/// - `shortcuts`: slice of `(key, description)` pairs in two-column layout
/// - Keys use `theme.primary`, descriptions use `theme.fg`
/// - Background: `theme.surface_elevated`, border: `theme.outline`
/// - Corners: ╭╮╰╯, sides: ─│
/// - Auto-sizes to fit content, clamped to `area`
pub fn render_help_overlay(
    plane: &mut Plane,
    area: Rect,
    t: &Theme,
    title: &str,
    shortcuts: &[(&str, &str)],
) {
    let min_w = 40u16;
    let hw = min_w
        .max(title.len() as u16 + 6)
        .min(area.width.saturating_sub(4));
    let hh = (3 + shortcuts.len() as u16 + 2).min(area.height.saturating_sub(4));
    let hx = (area.width.saturating_sub(hw)) / 2;
    let hy = (area.height.saturating_sub(hh)) / 2;

    // Background fill
    for y in hy..hy + hh {
        for x in hx..hx + hw {
            let idx = (y as usize) * area.width as usize + x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface_elevated;
                plane.cells[idx].transparent = false;
            }
        }
    }

    // Rounded border
    let corners = [
        ('╭', hx, hy),
        ('╮', hx + hw - 1, hy),
        ('╰', hx, hy + hh - 1),
        ('╯', hx + hw - 1, hy + hh - 1),
    ];
    for (ch, cx, cy) in corners {
        let idx = (cy as usize) * area.width as usize + cx as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = ch;
            plane.cells[idx].fg = t.outline;
        }
    }
    for x in hx + 1..hx + hw - 1 {
        let ti = (hy as usize) * area.width as usize + x as usize;
        let bi = ((hy + hh - 1) as usize) * area.width as usize + x as usize;
        if ti < plane.cells.len() {
            plane.cells[ti].char = '─';
            plane.cells[ti].fg = t.outline;
        }
        if bi < plane.cells.len() {
            plane.cells[bi].char = '─';
            plane.cells[bi].fg = t.outline;
        }
    }
    for y in hy + 1..hy + hh - 1 {
        let li = (y as usize) * area.width as usize + hx as usize;
        let ri = (y as usize) * area.width as usize + (hx + hw - 1) as usize;
        if li < plane.cells.len() {
            plane.cells[li].char = '│';
            plane.cells[li].fg = t.outline;
        }
        if ri < plane.cells.len() {
            plane.cells[ri].char = '│';
            plane.cells[ri].fg = t.outline;
        }
    }

    // Title (centered, primary + BOLD)
    let tx = hx + (hw - title.len() as u16) / 2;
    for (i, c) in title.chars().enumerate() {
        let idx = ((hy + 1) as usize) * area.width as usize + (tx + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = t.primary;
            plane.cells[idx].style = Styles::BOLD;
        }
    }

    // Shortcuts (two-column: keys in primary, descriptions in fg)
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let y = hy + 3 + i as u16;
        if y >= hy + hh - 1 {
            break;
        }
        for (j, c) in key.chars().enumerate() {
            let idx = (y as usize) * area.width as usize + (hx + 2 + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
            }
        }
        for (j, c) in desc.chars().enumerate() {
            let idx = (y as usize) * area.width as usize + (hx + 14 + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg;
            }
        }
    }
}

/// Draws a rounded focus ring (border) around a rectangular area.
/// Used to visually indicate the accessible focus target.
/// Draws ╭╮╰╯ corners, ─ top/bottom, │ left/right in the given color.
/// All coordinates are bounds-checked against the plane dimensions.
pub fn draw_focus_ring(plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, color: Color) {
    if w == 0 || h == 0 {
        return;
    }
    // Corners
    let corners = [
        ('╭', x, y),
        ('╮', x + w.saturating_sub(1), y),
        ('╰', x, y + h.saturating_sub(1)),
        ('╯', x + w.saturating_sub(1), y + h.saturating_sub(1)),
    ];
    for (ch, cx, cy) in corners {
        if cy < plane.height && cx < plane.width {
            let idx = (cy * plane.width + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = color;
                plane.cells[idx].transparent = false;
            }
        }
    }
    // Top and bottom borders
    for dx in 1..w.saturating_sub(1) {
        let cx = x + dx;
        if cx >= plane.width {
            break;
        }
        if y < plane.height {
            let top = (y * plane.width + cx) as usize;
            if top < plane.cells.len() {
                plane.cells[top].char = '─';
                plane.cells[top].fg = color;
                plane.cells[top].transparent = false;
            }
        }
        let by = y + h.saturating_sub(1);
        if by < plane.height {
            let bot = (by * plane.width + cx) as usize;
            if bot < plane.cells.len() {
                plane.cells[bot].char = '─';
                plane.cells[bot].fg = color;
                plane.cells[bot].transparent = false;
            }
        }
    }
    // Left and right borders
    for dy in 1..h.saturating_sub(1) {
        let cy = y + dy;
        if cy >= plane.height {
            break;
        }
        if x < plane.width {
            let left = (cy * plane.width + x) as usize;
            if left < plane.cells.len() {
                plane.cells[left].char = '│';
                plane.cells[left].fg = color;
                plane.cells[left].transparent = false;
            }
        }
        let rx = x + w.saturating_sub(1);
        if rx < plane.width {
            let right = (cy * plane.width + rx) as usize;
            if right < plane.cells.len() {
                plane.cells[right].char = '│';
                plane.cells[right].fg = color;
                plane.cells[right].transparent = false;
            }
        }
    }
}

use dracon_terminal_engine::framework::prelude::Theme;
use ratatui::layout::Rect;

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use ratatui::layout::Rect;
use unicode_width::UnicodeWidthChar;

use crate::data::ExampleMeta;

// ═══════════════════════════════════════════════════════════════════════════════
// RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

pub fn draw_rounded_border(plane: &mut Plane, area: Rect, fg: Color, bg: Color, selected: bool) {
    let ox = area.x as usize;
    let oy = area.y as usize;
    let w = area.width as usize;
    let h = area.height as usize;
    if w < 2 || h < 2 {
        return;
    }

    let chars = if selected {
        ('╭', '╮', '╰', '╯', '─', '│', '▓')
    } else {
        ('┌', '┐', '└', '┘', '─', '│', '░')
    };

    // Corners
    set_cell(plane, ox, oy, chars.0, fg, bg);
    set_cell(plane, ox + w - 1, oy, chars.1, fg, bg);
    set_cell(plane, ox, oy + h - 1, chars.2, fg, bg);
    set_cell(plane, ox + w - 1, oy + h - 1, chars.3, fg, bg);

    // Top/bottom edges
    for x in 1..w - 1 {
        set_cell(plane, ox + x, oy, chars.4, fg, bg);
        set_cell(plane, ox + x, oy + h - 1, chars.4, fg, bg);
    }

    // Left/right edges
    for y in 1..h - 1 {
        set_cell(plane, ox, oy + y, chars.5, fg, bg);
        set_cell(plane, ox + w - 1, oy + y, chars.5, fg, bg);
    }

    // Fill background
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            set_cell(plane, ox + x, oy + y, ' ', fg, bg);
        }
    }
}

pub fn set_cell(plane: &mut Plane, x: usize, y: usize, ch: char, fg: Color, bg: Color) {
    let idx = y * plane.width as usize + x;
    if idx < plane.cells.len() {
        plane.cells[idx] = Cell {
            char: ch,
            fg,
            bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
    }
}

#[allow(clippy::too_many_arguments)]
pub fn set_cell_bounded(
    plane: &mut Plane,
    x: usize,
    y: usize,
    ch: char,
    fg: Color,
    bg: Color,
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
) {
    if x >= min_x && x <= max_x && y >= min_y && y <= max_y {
        set_cell(plane, x, y, ch, fg, bg);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_text_bounded(
    plane: &mut Plane,
    x: usize,
    y: usize,
    text: &str,
    fg: Color,
    bg: Color,
    style: Styles,
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
) {
    if y < min_y || y > max_y {
        return;
    }
    let mut col = x;
    for ch in text.chars() {
        if col > max_x {
            break;
        }
        if col >= min_x {
            let idx = y * plane.width as usize + col;
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
        if let Some(w) = ch.width() {
            col += w;
        }
    }
}

pub fn draw_text(
    plane: &mut Plane,
    x: usize,
    y: usize,
    text: &str,
    fg: Color,
    bg: Color,
    style: Styles,
) {
    let mut col = x;
    for ch in text.chars() {
        let idx = y * plane.width as usize + col;
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
        if let Some(w) = ch.width() {
            col += w;
        }
    }
}

pub fn category_color(t: &Theme, cat: &str) -> Color {
    match cat {
        "apps" => t.warning,
        "cookbook" => t.info,
        "tools" => t.secondary,
        "input" => t.primary,
        "data" => t.success,
        "accessibility" => t.info,
        _ => t.fg_muted,
    }
}

/// Render a features highlight bar showing framework capabilities.
/// Returns the width consumed.
pub fn render_features_bar(plane: &mut Plane, theme: &Theme, y: usize, phase: f64) -> usize {
    let features = [
        ("43", "Widgets", theme.info),
        ("21", "Themes", theme.secondary),
        ("~", "Animations", theme.warning),
        ("DnD", "Drag & Drop", theme.success),
        ("KEY", "Keyboard", theme.primary),
        ("SIX", "Sixel", theme.info),
    ];

    let mut x = 2usize;
    for (i, (icon, label, color)) in features.iter().enumerate() {
        let item_phase = phase + i as f64 * 1.2;
        let pulse = (item_phase * 2.5).sin() * 0.3 + 0.7;
        let is_pulse_high = pulse > 0.85;

        // Separator
        if i > 0 {
            let sep = " | ";
            draw_text(plane, x, y, sep, theme.outline, theme.bg, Styles::empty());
            x += sep.len();
        }

        // Icon with pulse effect
        let icon_fg = if is_pulse_high {
            *color
        } else {
            theme.fg_muted
        };
        draw_text(
            plane,
            x,
            y,
            icon,
            icon_fg,
            theme.bg,
            if is_pulse_high {
                Styles::BOLD
            } else {
                Styles::empty()
            },
        );
        x += icon.chars().count();

        // Label
        let label_fg = if is_pulse_high {
            theme.fg
        } else {
            theme.fg_muted
        };
        draw_text(plane, x, y, label, label_fg, theme.bg, Styles::empty());
        x += label.len();
    }

    x + 2
}

/// Configuration for rendering a single example card.
pub struct CardConfig<'a> {
    pub ex: &'a ExampleMeta,
    pub idx: usize,
    pub selected_idx: usize,
    pub hovered_idx: Option<usize>,
    pub theme: &'a Theme,
    pub phase: f64,
    pub width: u16,
    pub height: u16,
    pub is_embedded: bool,
    pub search_query: &'a str,
    pub run_count: u32,
}

/// Draw text with optional search highlighting at absolute coordinates.
#[allow(clippy::too_many_arguments)]
fn draw_text_at(
    plane: &mut Plane,
    x: usize,
    y: usize,
    text: &str,
    normal_fg: Color,
    bg: Color,
    bold: bool,
    search: &str,
    highlight_color: Color,
) {
    let style = if bold { Styles::BOLD } else { Styles::empty() };
    if search.is_empty() || !text.to_lowercase().contains(&search.to_lowercase()) {
        draw_text(plane, x, y, text, normal_fg, bg, style);
        return;
    }
    let lower = text.to_lowercase();
    let q = search.to_lowercase();
    let mut pos = x;
    let mut remaining = 0usize;
    while remaining < text.len() {
        let rest = &text[remaining..];
        let rest_lower = &lower[remaining..];
        match rest_lower.find(&q) {
            Some(start) => {
                if start > 0 {
                    let before = &rest[..start];
                    draw_text(plane, pos, y, before, normal_fg, bg, style);
                    pos += before.chars().count();
                }
                let match_str = &rest[start..start + q.len()];
                draw_text(plane, pos, y, match_str, highlight_color, bg, Styles::BOLD);
                pos += match_str.chars().count();
                remaining += start + q.len();
            }
            None => {
                draw_text(plane, pos, y, rest, normal_fg, bg, style);
                break;
            }
        }
    }
}

/// Render card directly into the provided plane at the given offset.
/// This eliminates intermediate Plane allocation per card.
pub fn render_card(config: &CardConfig, plane: &mut Plane, offset_x: usize, offset_y: usize) {
    let t = &config.theme;
    let card_w_usize = config.width as usize;
    let card_h_usize = config.height as usize;

    let is_selected = config.idx == config.selected_idx;
    let is_hovered = Some(config.idx) == config.hovered_idx;
    let cat_color = category_color(config.theme, config.ex.category);

    let card_phase = config.phase + (config.idx as f64 * 0.73);

    let border_fg = if is_selected {
        let pulse = (card_phase * 2.0).sin() * 0.5 + 0.5;
        if pulse > 0.5 {
            t.primary
        } else {
            t.primary_hover
        }
    } else if is_hovered {
        let hover_pulse = (card_phase * 3.0).sin() * 0.15 + 0.85;
        if hover_pulse > 0.92 {
            t.primary
        } else {
            t.primary_hover
        }
    } else {
        t.outline
    };
    let bg = if is_selected {
        t.surface_elevated
    } else {
        t.surface
    };
    draw_rounded_border(
        plane,
        Rect::new(
            offset_x as u16,
            offset_y as u16,
            config.width,
            config.height,
        ),
        border_fg,
        bg,
        is_selected || is_hovered,
    );

    // Badge
    let badge = format!(" {} ", config.ex.category.to_uppercase());
    let badge_x = offset_x + 2;
    let badge_y = offset_y + 1;
    for (i, ch) in badge.chars().enumerate() {
        let px = badge_x + i;
        if px < offset_x + card_w_usize - 2 {
            set_cell(plane, px, badge_y, ch, t.fg_on_accent, cat_color);
        }
    }

    // Embedded badge
    let next_badge_x = badge_x + badge.len() + 1;
    if config.is_embedded {
        let embed_badge = " * ";
        for (i, ch) in embed_badge.chars().enumerate() {
            let px = next_badge_x + i;
            if px < offset_x + card_w_usize - 2 {
                set_cell(plane, px, badge_y, ch, t.bg, t.success);
            }
        }
    }

    let name_y = offset_y + 3;
    let max_name_len = (card_w_usize - 4).min(24);
    let name_truncated: String = config.ex.name.chars().take(max_name_len).collect();
    draw_text_at(
        plane,
        offset_x + 2,
        name_y,
        &name_truncated,
        t.fg,
        bg,
        true,
        config.search_query,
        t.primary,
    );

    let desc_y = offset_y + 4;
    let max_desc_len = (card_w_usize - 4).min(24);
    let desc: String = config.ex.description.chars().take(max_desc_len).collect();
    draw_text_at(
        plane,
        offset_x + 2,
        desc_y,
        &desc,
        t.fg_muted,
        bg,
        false,
        config.search_query,
        t.primary,
    );

    // Run count badge
    if config.run_count > 0 {
        let run_text = format!(" {}x ", config.run_count);
        let rx = offset_x + card_w_usize.saturating_sub(run_text.len() + 2);
        for (i, ch) in run_text.chars().enumerate() {
            let px = rx + i;
            if px < offset_x + card_w_usize - 2 {
                set_cell(plane, px, name_y + 1, ch, t.fg_on_accent, t.info);
            }
        }
    }

    let preview_start_y = offset_y + 6;
    let inner_min_x = offset_x + 1;
    let inner_max_x = offset_x + card_w_usize - 2;
    let inner_min_y = offset_y + 1;
    let inner_max_y = offset_y + card_h_usize - 2;

    match config.ex.name {
        "system_monitor" => render_live_gauge_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "split_resizer" => render_split_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "command_bindings" => render_command_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "theme_switcher" => render_theme_preview(
            plane,
            config.theme,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "widget_gallery" => render_widget_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "ide" => render_ide_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "desktop" => render_desktop_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "git_tui" => render_git_tui_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "file_manager" => render_file_manager_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "menu_system" => render_menu_system_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "modal_demo" => render_modal_demo_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "dashboard_builder" => render_dashboard_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "tabbed_panels" => render_tabbed_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "tree_navigator" => render_tree_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "data_table" => render_table_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "input_debug" => render_input_debug_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "text_editor_demo" => render_text_editor_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "game_loop" => render_game_loop_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "form_demo" | "form_widget" => render_form_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "framework_file_manager" => render_framework_fm_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "calendar" => render_calendar_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "rich_text" => render_rich_text_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "autocomplete" => render_autocomplete_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "notification_center" => render_notification_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "accessibility" => render_accessibility_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        "cell_pool" => render_cell_pool_preview(
            plane,
            config.theme,
            card_phase,
            offset_x,
            offset_y,
            card_w_usize,
            card_h_usize,
        ),
        _ => {
            for (i, line) in config.ex.preview.iter().enumerate() {
                let py = preview_start_y + i;
                if py >= inner_min_y && py <= inner_max_y {
                    draw_text_bounded(
                        plane,
                        offset_x + 2,
                        py,
                        line,
                        t.fg_subtle,
                        bg,
                        Styles::empty(),
                        inner_min_x,
                        inner_max_x,
                        inner_min_y,
                        inner_max_y,
                    );
                }
            }
        }
    }

    if is_selected {
        draw_text(
            plane,
            offset_x + 1,
            offset_y + card_h_usize / 2,
            "►",
            t.primary,
            bg,
            Styles::BOLD,
        );
    }
}

fn render_live_gauge_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let items = [
        ("CPU", (phase * 30.0).sin() * 40.0 + 50.0),
        ("MEM", (phase * 20.0).sin() * 30.0 + 60.0),
        ("DISK", (phase * 15.0).sin() * 20.0 + 40.0),
        ("NET", (phase * 25.0).sin() * 50.0 + 50.0),
    ];
    let bar_w = (card_w - 12).clamp(4, 14);
    for (i, (label, value)) in items.iter().enumerate() {
        let y = oy + 6 + i;
        if y > oy + 11 || y > max_y {
            break;
        }
        let val = value.clamp(0.0, 100.0);
        let fill = ((val / 100.0) * bar_w as f64).round() as usize;
        let color = if val > 80.0 {
            t.error
        } else if val > 60.0 {
            t.warning
        } else {
            t.success
        };
        draw_text_bounded(
            plane,
            ox + 2,
            y,
            label,
            t.fg_muted,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        set_cell_bounded(
            plane,
            ox + 6,
            y,
            '[',
            t.fg_muted,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        for j in 0..bar_w {
            let ch = if j < fill { '█' } else { '░' };
            let fg = if j < fill { color } else { t.fg_muted };
            set_cell_bounded(
                plane,
                ox + 7 + j,
                y,
                ch,
                fg,
                t.surface,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
        set_cell_bounded(
            plane,
            ox + 7 + bar_w,
            y,
            ']',
            t.fg_muted,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        let pct = format!("{:>3}%", val.round() as u32);
        draw_text_bounded(
            plane,
            ox + 7 + bar_w + 2,
            y,
            &pct,
            color,
            t.surface,
            Styles::BOLD,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_split_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let split_x = (4.0 + (phase * 0.5).sin() * 3.0).round() as usize;
    let split_x = split_x.min(card_w - 4).max(2);
    let w = (card_w - 2).min(26);
    for y in oy + 6..(oy + 12).min(max_y + 1) {
        for x in ox + 1..ox + w {
            let bg = if x - ox <= split_x {
                t.surface_elevated
            } else {
                t.surface
            };
            let fg = if x - ox <= split_x {
                t.fg_muted
            } else {
                t.fg_subtle
            };
            set_cell_bounded(plane, x, y, ' ', fg, bg, ox + 1, max_x, oy + 1, max_y);
        }
    }
    for y in oy + 6..(oy + 12).min(max_y + 1) {
        set_cell_bounded(
            plane,
            ox + split_x,
            y,
            '│',
            t.primary,
            t.surface_elevated,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    draw_text_bounded(
        plane,
        ox + 2,
        oy + 7,
        "A",
        t.fg,
        t.surface_elevated,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    draw_text_bounded(
        plane,
        ox + split_x + 2,
        oy + 7,
        "B",
        t.fg,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    let label = format!("{}:{}", split_x, w - split_x);
    draw_text_bounded(
        plane,
        ox + w / 2 - 3,
        oy + 11,
        &label,
        t.fg_muted,
        t.bg,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
}

fn render_command_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let lines = [
        format!("Load: {:.2}", 0.45 + (phase * 0.3).sin() * 0.2),
        format!(
            "CPU:  [{}{}]",
            "█".repeat(((phase * 4.0).sin() + 1.0) as usize),
            "░".repeat(6)
        ),
        format!(
            "Mem:  [{}{}]",
            "█".repeat(((phase * 3.0).sin() + 1.0) as usize),
            "░".repeat(5)
        ),
        format!(
            "Net:  [{}{}]",
            "█".repeat(((phase * 2.0).sin() + 1.0) as usize),
            "░".repeat(7)
        ),
    ];
    for (i, line) in lines.iter().enumerate() {
        let py = oy + 6 + i;
        if py > oy + 11 || py > max_y {
            break;
        }
        let max_len = max_x - (ox + 2) + 1;
        let truncated: String = line.chars().take(max_len).collect();
        draw_text_bounded(
            plane,
            ox + 2,
            py,
            &truncated,
            t.fg_subtle,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_theme_preview(
    plane: &mut Plane,
    t: &Theme,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let colors = [
        t.primary,
        t.primary_hover,
        t.success,
        t.warning,
        t.error,
        t.info,
        t.fg,
        t.bg,
    ];
    let cols = 4;
    let swatch_size = 3;
    for (i, color) in colors.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let x = ox + 2 + col * (swatch_size + 1);
        let y = oy + 6 + row * 2;
        if y > oy + 11 || y > max_y {
            break;
        }
        for dx in 0..swatch_size {
            set_cell_bounded(
                plane,
                x + dx,
                y,
                ' ',
                t.fg,
                *color,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
            set_cell_bounded(
                plane,
                x + dx,
                y + 1,
                ' ',
                t.fg,
                *color,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
    }
    let name = format!("  {}  ", t.name);
    draw_text_bounded(
        plane,
        ox + 2,
        oy + 11,
        &name,
        t.fg_muted,
        t.bg,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
}

fn render_widget_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let checks = ["[x] Alpha", "[ ] Beta", "[x] Gamma"];
    for (i, check) in checks.iter().enumerate() {
        let py = oy + 6 + i;
        if py > oy + 10 || py > max_y {
            break;
        }
        let text: String = check.chars().take(max_x - (ox + 2) + 1).collect();
        draw_text_bounded(
            plane,
            ox + 2,
            py,
            &text,
            t.fg_subtle,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    let slider_y = oy + 10;
    let slider_w = (card_w - 6).clamp(4, 18);
    let thumb = ((phase * 2.0).sin() * 0.5 + 0.5 * slider_w as f64).round() as usize;
    let thumb = thumb.min(slider_w - 1);
    draw_text_bounded(
        plane,
        ox + 2,
        slider_y,
        "[",
        t.fg_muted,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    for i in 0..slider_w {
        let ch = if i == thumb {
            '#'
        } else if i < thumb {
            '='
        } else {
            '-'
        };
        let fg = if i == thumb { t.primary } else { t.fg_muted };
        set_cell_bounded(
            plane,
            ox + 3 + i,
            slider_y,
            ch,
            fg,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    draw_text_bounded(
        plane,
        ox + 3 + slider_w,
        slider_y,
        "]",
        t.fg_muted,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
}

fn render_ide_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let tabs: Vec<(&str, bool)> = if card_w >= 28 {
        vec![
            (" main.rs ", true),
            (" lib.rs ", false),
            (" mod.rs ", false),
        ]
    } else if card_w >= 22 {
        vec![(" main.rs ", true), (" lib.rs ", false)]
    } else {
        vec![(" main.rs ", true)]
    };
    let mut tab_x = ox + 1;
    let mut active_tab_start = 0usize;
    let mut active_tab_len = 0usize;
    for (label, active) in &tabs {
        if tab_x + label.len() > max_x + 1 {
            break;
        }
        let fg = if *active { t.fg_on_accent } else { t.fg_muted };
        let bg = if *active { t.primary_active } else { t.surface };
        draw_text_bounded(
            plane,
            tab_x,
            oy + 5,
            label,
            fg,
            bg,
            if *active {
                Styles::BOLD
            } else {
                Styles::empty()
            },
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        if *active {
            active_tab_start = tab_x;
            active_tab_len = label.len().min(max_x - tab_x + 1);
        }
        tab_x += label.len() + 1;
    }
    for dx in 0..active_tab_len {
        set_cell_bounded(
            plane,
            active_tab_start + dx,
            oy + 6,
            '▔',
            t.primary_active,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    let lines = [
        ("1", "fn main() {"),
        ("2", "    let x = 42;"),
        ("3", "    println!(\"{}\", x);"),
        ("4", "}"),
    ];
    for (i, (num, code)) in lines.iter().enumerate() {
        let py = oy + 6 + i;
        if py > oy + 10 || py > max_y {
            break;
        }
        draw_text_bounded(
            plane,
            ox + 1,
            py,
            num,
            t.fg_muted,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        let max_code_len = max_x - (ox + 3) + 1;
        let code_truncated: String = code.chars().take(max_code_len).collect();
        draw_text_bounded(
            plane,
            ox + 3,
            py,
            &code_truncated,
            t.fg,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    if (phase * 3.0).fract() < 0.6 {
        set_cell_bounded(
            plane,
            ox + 4,
            oy + 6,
            '▎',
            t.primary,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_desktop_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let wins = [
        (1, 6, 8, 4, t.primary),
        (11, 7, 8, 4, t.warning),
        (6, 9, 10, 3, t.info),
    ];
    let offsets = [
        ((phase * 20.0).sin() as i16, (phase * 15.0).sin() as i16),
        ((phase * 18.0).sin() as i16, (phase * 12.0).sin() as i16),
        (0, 0),
    ];
    for (i, (x, y, w, h, color)) in wins.iter().enumerate() {
        let ox2 = offsets[i].0;
        let oy2 = offsets[i].1;
        let wx = (*x as i16 + ox2).max(1) as usize;
        let wy = (*y as i16 + oy2).max(6) as usize;
        let wx = wx.min(card_w - 4);
        let wy = wy.min(11);
        set_cell_bounded(
            plane,
            ox + wx,
            oy + wy,
            '┌',
            *color,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        for dx in 1..w - 1 {
            set_cell_bounded(
                plane,
                ox + wx + dx,
                oy + wy,
                '─',
                *color,
                t.surface,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
        set_cell_bounded(
            plane,
            ox + wx + w - 1,
            oy + wy,
            '┐',
            *color,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        for dy in 1..h - 1 {
            set_cell_bounded(
                plane,
                ox + wx,
                oy + wy + dy,
                '│',
                *color,
                t.surface,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
            for dx in 1..w - 1 {
                set_cell_bounded(
                    plane,
                    ox + wx + dx,
                    oy + wy + dy,
                    ' ',
                    *color,
                    t.surface,
                    ox + 1,
                    max_x,
                    oy + 1,
                    max_y,
                );
            }
            set_cell_bounded(
                plane,
                ox + wx + w - 1,
                oy + wy + dy,
                '│',
                *color,
                t.surface,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
        set_cell_bounded(
            plane,
            ox + wx,
            oy + wy + h - 1,
            '└',
            *color,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        for dx in 1..w - 1 {
            set_cell_bounded(
                plane,
                ox + wx + dx,
                oy + wy + h - 1,
                '─',
                *color,
                t.surface,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
        set_cell_bounded(
            plane,
            ox + wx + w - 1,
            oy + wy + h - 1,
            '┘',
            *color,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_git_tui_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    draw_text_bounded(
        plane,
        ox + 2,
        oy + 6,
        " main ",
        t.fg_on_accent,
        t.primary_active,
        Styles::BOLD,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    draw_text_bounded(
        plane,
        ox + 2,
        oy + 7,
        "Status: 3 files changed",
        t.fg,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    let phases = [
        [
            (" M src/main.rs", t.warning),
            (" A Cargo.toml", t.success),
            ("?? README.md", t.error),
        ],
        [
            (" M Cargo.toml", t.warning),
            (" D old.rs", t.error),
            (" A new.rs", t.success),
        ],
        [
            ("?? config.yml", t.error),
            (" M lib.rs", t.warning),
            (" A test.rs", t.success),
        ],
        [
            (" D removed.rs", t.error),
            (" M updated.rs", t.warning),
            ("?? unknown.py", t.error),
        ],
    ];
    let phase_idx = ((phase * 0.3).floor() as usize) % phases.len();
    for (i, (text, color)) in phases[phase_idx].iter().enumerate() {
        draw_text_bounded(
            plane,
            ox + 2,
            oy + 9 + i,
            text,
            *color,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_file_manager_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let items = [
        (0, "home/", true, 0),
        (1, "user/", true, 1),
        (2, "  src/", true, 2),
        (3, "    main.rs", false, -1),
        (3, "    lib.rs", false, -1),
        (2, "  docs/", true, 1),
        (3, "    README.md", false, -1),
    ];
    let expand_phase = ((phase * 0.5).sin() * 4.0).round().max(0.0) as usize % 4;
    let visible_depth = if expand_phase == 0 {
        1
    } else if expand_phase == 1 {
        2
    } else if expand_phase == 2 {
        3
    } else {
        4
    };
    for (i, (indent, name, is_dir, _)) in items.iter().enumerate() {
        if *indent as usize > visible_depth {
            continue;
        }
        let py = oy + 6 + i;
        if py > oy + 12 || py > max_y {
            break;
        }
        let icon = if *is_dir { "v" } else { ">" };
        let text = format!("{}{}", icon, name);
        let max_len = max_x - (ox + 2) + 1;
        let truncated: String = text.chars().take(max_len).collect();
        let fg = if *is_dir { t.warning } else { t.fg_subtle };
        draw_text_bounded(
            plane,
            ox + 2,
            py,
            &truncated,
            fg,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_menu_system_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let menus: Vec<&str> = if card_w >= 40 {
        vec!["File", "Edit", "View", "Help"]
    } else if card_w >= 28 {
        vec!["File", "Edit", "View"]
    } else {
        vec!["File", "Edit"]
    };
    let highlight_idx = ((phase * 2.0) as usize) % menus.len();
    let menu_w = (card_w / menus.len().max(1)).clamp(4, 8);
    for (i, menu) in menus.iter().enumerate() {
        let x = ox + 2 + i * (menu_w + 1);
        if x > max_x {
            break;
        }
        let is_highlighted = i == highlight_idx;
        let bg = if is_highlighted { t.primary } else { t.surface };
        let fg = if is_highlighted { t.fg_on_accent } else { t.fg };
        for dx in 0..menu_w {
            set_cell_bounded(
                plane,
                x + dx,
                oy + 6,
                ' ',
                fg,
                bg,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
        let text = format!(" {} ", menu);
        draw_text_bounded(
            plane,
            x,
            oy + 6,
            &text,
            fg,
            bg,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        if is_highlighted {
            for dy in 1..5 {
                for dx in 0..menu_w {
                    set_cell_bounded(
                        plane,
                        x + dx,
                        oy + 6 + dy,
                        if dy == 4 { '─' } else { ' ' },
                        if dy == 4 { t.primary } else { t.fg },
                        t.surface,
                        ox + 1,
                        max_x,
                        oy + 1,
                        max_y,
                    );
                }
            }
            for (j, item) in ["New", "Open", "Save", "Exit"].iter().enumerate() {
                draw_text_bounded(
                    plane,
                    x + 1,
                    oy + 7 + j,
                    item,
                    t.fg,
                    t.surface,
                    Styles::empty(),
                    ox + 1,
                    max_x,
                    oy + 1,
                    max_y,
                );
            }
        }
    }
}

fn render_modal_demo_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let mw = (card_w - 6).clamp(8, 18);
    let mh = 7;
    let mx = ox + (card_w - mw) / 2;
    let my = oy + 5;
    let bc = t.warning;
    set_cell_bounded(
        plane,
        mx,
        my,
        '┌',
        bc,
        t.surface_elevated,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    for dx in 1..mw - 1 {
        set_cell_bounded(
            plane,
            mx + dx,
            my,
            '─',
            bc,
            t.surface_elevated,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    set_cell_bounded(
        plane,
        mx + mw - 1,
        my,
        '┐',
        bc,
        t.surface_elevated,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    for dy in 1..mh - 1 {
        set_cell_bounded(
            plane,
            mx,
            my + dy,
            '│',
            bc,
            t.surface_elevated,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        for dx in 1..mw - 1 {
            set_cell_bounded(
                plane,
                mx + dx,
                my + dy,
                ' ',
                bc,
                t.surface_elevated,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
        set_cell_bounded(
            plane,
            mx + mw - 1,
            my + dy,
            '│',
            bc,
            t.surface_elevated,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    set_cell_bounded(
        plane,
        mx,
        my + mh - 1,
        '└',
        bc,
        t.surface_elevated,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    for dx in 1..mw - 1 {
        set_cell_bounded(
            plane,
            mx + dx,
            my + mh - 1,
            '─',
            bc,
            t.surface_elevated,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    set_cell_bounded(
        plane,
        mx + mw - 1,
        my + mh - 1,
        '┘',
        bc,
        t.surface_elevated,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    let text = " Confirm? ";
    draw_text_bounded(
        plane,
        mx + (mw - text.len()) / 2,
        my + 2,
        text,
        t.fg,
        t.surface_elevated,
        Styles::BOLD,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    let pulse = (phase * 3.0).sin() * 0.5 + 0.5;
    let yes_fg = if pulse > 0.5 { t.success } else { t.fg_muted };
    draw_text_bounded(
        plane,
        mx + 3,
        my + 4,
        "[ Yes ]",
        yes_fg,
        t.surface_elevated,
        Styles::BOLD,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    draw_text_bounded(
        plane,
        mx + 10,
        my + 4,
        "[ No  ]",
        t.fg_muted,
        t.surface_elevated,
        Styles::BOLD,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
}

fn render_dashboard_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let bar_len = (card_w - 12).clamp(4, 10);
    let items = [
        ("CPU", (phase * 25.0).sin() * 30.0 + 55.0),
        ("MEM", (phase * 20.0).sin() * 20.0 + 65.0),
        ("NET", (phase * 15.0).sin() * 40.0 + 50.0),
    ];
    for (i, (label, value)) in items.iter().enumerate() {
        let y = oy + 6 + i;
        if y > oy + 10 || y > max_y {
            break;
        }
        let val = value.clamp(0.0, 100.0) as u32;
        let filled = ((val as f64 / 100.0) * bar_len as f64).round() as usize;
        let bar_str = format!(
            "{} {}",
            label,
            format_args!(
                "{}{}{}",
                "[",
                "█".repeat(filled.min(bar_len)),
                "░".repeat(bar_len - filled.min(bar_len))
            )
        );
        let color = if val > 80 {
            t.error
        } else if val > 50 {
            t.warning
        } else {
            t.success
        };
        let max_text_len = max_x - (ox + 1) + 1;
        let truncated: String = bar_str.chars().take(max_text_len).collect();
        draw_text_bounded(
            plane,
            ox + 1,
            y,
            &truncated,
            color,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        set_cell_bounded(
            plane,
            ox + 6 + bar_len + 1,
            y,
            ']',
            t.fg_muted,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_tabbed_preview(
    plane: &mut Plane,
    t: &Theme,
    _phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let tabs = ["Tab1", "Tab2", "Tab3+"];
    for (i, tab) in tabs.iter().enumerate() {
        let is_active = i == 0;
        let fg = if is_active { t.primary } else { t.fg_muted };
        let bg = if is_active {
            t.surface_elevated
        } else {
            t.surface
        };
        for (j, ch) in tab.chars().enumerate() {
            set_cell_bounded(
                plane,
                ox + 1 + j + i * 5,
                oy + 5,
                ch,
                fg,
                bg,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
    }
    let line_w = (card_w - 2).min(19);
    for x in ox + 1..ox + line_w + 1 {
        set_cell_bounded(
            plane,
            x,
            oy + 6,
            '─',
            t.outline,
            t.surface_elevated,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    for y in oy + 6..(oy + 11).min(max_y + 1) {
        for x in ox + 1..ox + (card_w - 1).min(20) + 1 {
            set_cell_bounded(
                plane,
                x,
                y,
                ' ',
                t.fg,
                t.surface_elevated,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
    }
    draw_text_bounded(
        plane,
        ox + 3,
        oy + 8,
        "Tab content here",
        t.fg_muted,
        t.surface_elevated,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
}

fn render_tree_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let lines = [
        "v root/",
        "| v src/",
        "| | > main.rs",
        "| | > lib.rs",
        "| v target/",
    ];
    let scroll = ((phase * 0.5).sin() * 1.5) as i16;
    let max_text_len = max_x - (ox + 2) + 1;
    for (i, line) in lines.iter().enumerate() {
        let y = oy + 6 + i;
        if !(oy + 6..=oy + 11).contains(&y) || y > max_y {
            continue;
        }
        let offset = if i == 2 { scroll } else { 0 };
        let x = (ox as i16 + 2 + offset).clamp(ox as i16 + 1, ox as i16 + 20) as usize;
        let truncated: String = line
            .chars()
            .skip(x.saturating_sub(ox + 2))
            .take(max_text_len)
            .collect();
        let prefix = if truncated.starts_with('|') {
            "│"
        } else {
            " "
        };
        draw_text_bounded(
            plane,
            ox + 1,
            y,
            prefix,
            t.fg_muted,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        draw_text_bounded(
            plane,
            ox + 2,
            y,
            &truncated,
            t.fg_subtle,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_table_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let headers = " Name    | Age | City ";
    let sep = "---------|-----|------";
    draw_text_bounded(
        plane,
        ox + 1,
        oy + 5,
        headers,
        t.primary,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    draw_text_bounded(
        plane,
        ox + 1,
        oy + 6,
        sep,
        t.outline,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    let name_w = (card_w - 8).clamp(4, 10);
    let rows = [
        (" Alice   ", "  28 ", " NYC  "),
        (" Bob     ", "  34 ", " LA   "),
    ];
    let highlight_row = ((phase * 0.5).sin() * 0.5 + 0.5) > 0.5;
    for (i, (name, age, city)) in rows.iter().enumerate() {
        let y = oy + 7 + i;
        if y > max_y {
            break;
        }
        let is_selected = highlight_row && i == 1;
        let fg = if is_selected {
            t.selection_fg
        } else {
            t.fg_subtle
        };
        let prefix = if is_selected { ">" } else { " " };
        draw_text_bounded(
            plane,
            ox + 1,
            y,
            prefix,
            t.primary,
            t.surface,
            Styles::BOLD,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        let name_trunc: String = name.chars().take(name_w).collect();
        draw_text_bounded(
            plane,
            ox + 2,
            y,
            &name_trunc,
            fg,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        draw_text_bounded(
            plane,
            ox + 2 + name_w,
            y,
            age,
            t.fg_muted,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        draw_text_bounded(
            plane,
            ox + 2 + name_w + 6,
            y,
            city,
            t.fg_muted,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_input_debug_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let keys = ["Key: ArrowUp  0x2191", "Mod: Ctrl+Shift"];
    for (i, key) in keys.iter().enumerate() {
        let max_len = max_x - (ox + 1) + 1;
        let t_str: String = key.chars().take(max_len).collect();
        draw_text_bounded(
            plane,
            ox + 1,
            oy + 6 + i,
            &t_str,
            t.fg_subtle,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    let mx = (phase * 30.0).sin() as i16 + 40;
    let my = (phase * 20.0).sin() as i16 + 10;
    let mouse_text = format!("Mouse: {:3}, {:2} [L-down]", mx, my);
    let max_len = max_x - (ox + 1) + 1;
    let truncated: String = mouse_text.chars().take(max_len).collect();
    draw_text_bounded(
        plane,
        ox + 1,
        oy + 8,
        &truncated,
        t.primary,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    let wheel = if (phase * 2.0).sin() > 0.0 {
        "+1"
    } else {
        "-1"
    };
    let wheel_text = format!("Wheel: {}", wheel);
    let truncated2: String = wheel_text.chars().take(max_len).collect();
    draw_text_bounded(
        plane,
        ox + 1,
        oy + 9,
        &truncated2,
        t.fg_muted,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
}

fn render_text_editor_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let lines = ["1 | fn main() {", "2 |   println!();", "3 | }"];
    for (i, line) in lines.iter().enumerate() {
        let max_len = max_x - (ox + 1) + 1;
        let t_str: String = line.chars().take(max_len).collect();
        draw_text_bounded(
            plane,
            ox + 1,
            oy + 6 + i,
            &t_str,
            t.fg_subtle,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    let cursor_x = ox + 7 + ((phase * 2.0).sin() * 0.5 + 0.5) as usize * 5;
    set_cell_bounded(
        plane,
        cursor_x.min(max_x),
        oy + 7,
        '▎',
        t.primary,
        t.surface,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    let lang = "  [rust] UTF-8 ";
    let lang_x = max_x.saturating_sub(lang.len()).max(ox + 2);
    draw_text_bounded(
        plane,
        lang_x,
        oy + 10,
        lang,
        t.fg_muted,
        t.bg,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    if (phase * 2.0).sin() > 0.0 {
        set_cell_bounded(
            plane,
            ox + 3,
            oy + 6,
            '█',
            t.primary,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_game_loop_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let (snake_y, snake_x) = (
        7.0 + (phase * 3.0).sin() * 1.5,
        12.0 + (phase * 2.0).cos() * 3.0,
    );
    let (sy, sx) = (snake_y.round() as usize, snake_x.round() as usize);
    let (min_px, max_px) = (sx.saturating_sub(1).max(8), (sx + 1).min(card_w - 4));
    let (min_py, max_py) = (sy.saturating_sub(1).max(6), (sy + 1).min(max_y));
    for py in min_py..=max_py {
        for px in min_px..=max_px {
            let (dx, dy) = (px as i32 - snake_x as i32, py as i32 - snake_y as i32);
            let dist_sq = dx * dx + dy * dy;
            let (ch, color) = if dist_sq <= 2 && dist_sq > 0 {
                ('█', t.success)
            } else {
                (' ', t.surface)
            };
            set_cell_bounded(
                plane,
                px,
                py,
                ch,
                color,
                t.surface,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
    }
    let score_str = format!("  Score: {:3}  ", ((phase * 10.0).sin() * 10.0) as i32 + 42);
    draw_text_bounded(
        plane,
        ox + (card_w - 2) / 2 - score_str.len() / 2,
        oy + 11,
        &score_str,
        t.warning,
        t.surface,
        Styles::BOLD,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
}

fn render_form_preview(
    plane: &mut Plane,
    t: &Theme,
    _phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let fields = [("Name:", "[___________]"), ("Email:", "[__________]")];
    for (i, (label, field)) in fields.iter().enumerate() {
        let y = oy + 5 + i * 2;
        if y > max_y {
            break;
        }
        draw_text_bounded(
            plane,
            ox + 2,
            y,
            label,
            t.fg_muted,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        draw_text_bounded(
            plane,
            ox + 8,
            y,
            field,
            t.fg,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    let btns = ["[Submit]", "[Cancel]"];
    for (i, btn) in btns.iter().enumerate() {
        let fg = if i == 0 { t.primary } else { t.fg_muted };
        draw_text_bounded(
            plane,
            ox + 6 + i * 10,
            oy + 10,
            btn,
            fg,
            t.surface,
            Styles::BOLD,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_framework_fm_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    draw_text_bounded(
        plane,
        ox + 2,
        oy + 5,
        "/ home/ user/",
        t.primary,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    set_cell_bounded(
        plane,
        ox + 1,
        oy + 6,
        '├',
        t.outline,
        t.surface,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    let line_w = (card_w - 4).clamp(4, 22);
    for cx in ox + 2..ox + line_w + 2 {
        set_cell_bounded(
            plane,
            cx,
            oy + 6,
            '─',
            t.outline,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    set_cell_bounded(
        plane,
        ox + line_w + 2,
        oy + 6,
        '┤',
        t.outline,
        t.surface,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    let rows = [
        ("src/", "  -   "),
        ("main.rs", " 1.2KB"),
        ("lib.rs", "  842B"),
    ];
    for (i, (name, size)) in rows.iter().enumerate() {
        let y = oy + 7 + i;
        if y > max_y {
            break;
        }
        let fg = if name.ends_with('/') {
            t.primary
        } else {
            t.fg_subtle
        };
        let name_max = max_x - (ox + 2) + 1;
        let name_trunc: String = name.chars().take(name_max).collect();
        draw_text_bounded(
            plane,
            ox + 2,
            y,
            &name_trunc,
            fg,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        draw_text_bounded(
            plane,
            ox + 14,
            y,
            size,
            t.fg_muted,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    if (phase * 2.0).sin() > 0.0 {
        set_cell_bounded(
            plane,
            ox + 2,
            oy + 8,
            '█',
            t.primary,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_calendar_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let months = ["January", "February", "March", "April", "May", "June"];
    let month_idx = ((phase * 0.3).floor() as usize) % months.len();
    let title = format!("{} 2026", months[month_idx]);
    draw_text_bounded(
        plane,
        ox + 1,
        oy + 5,
        &title,
        t.fg,
        t.surface,
        Styles::BOLD,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    draw_text_bounded(
        plane,
        ox + 1,
        oy + 6,
        "Mo Tu We Th Fr Sa Su",
        t.fg_muted,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    let day_grid = [
        "    1  2  3  4  5",
        " 6  7  8  9 10 11 12",
        "13 14 15 16 17 18 19",
        "20 21 22 23 24 25 26",
        "27 28 29 30 31     ",
    ];
    let offset = ((phase * 0.5).floor() as usize) % 2;
    for (i, row) in day_grid.iter().enumerate() {
        let y = oy + 7 + i;
        if y > oy + 11 || y > max_y {
            break;
        }
        let max_len = max_x - (ox + 1) + 1;
        let truncated: String = row.chars().skip(offset).take(max_len).collect();
        let fg = if i == 1 { t.primary } else { t.fg_subtle };
        draw_text_bounded(
            plane,
            ox + 1,
            y,
            &truncated,
            fg,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    let sel = (((phase * 0.8).sin() * 0.5 + 0.5) * 30.0).round() as usize % 31 + 1;
    let sel_text = format!("Sel: {}-{:>2}", month_idx + 1, sel.min(28));
    draw_text_bounded(
        plane,
        ox + 1,
        oy + 11,
        &sel_text,
        t.fg_muted,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
}

fn render_rich_text_preview(
    plane: &mut Plane,
    t: &Theme,
    _phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let lines = [
        ("# Heading", t.primary, true),
        ("**Bold** and *italic*", t.fg, false),
        ("`inline code`", t.secondary, false),
        ("- List item", t.fg_muted, false),
        ("[link](https://)", t.info, false),
    ];
    for (i, (text, color, bold)) in lines.iter().enumerate() {
        let y = oy + 5 + i;
        if y > oy + 10 || y > max_y {
            break;
        }
        let max_len = max_x - (ox + 1) + 1;
        let truncated: String = text.chars().take(max_len).collect();
        draw_text_bounded(
            plane,
            ox + 1,
            y,
            &truncated,
            *color,
            t.surface,
            if *bold { Styles::BOLD } else { Styles::empty() },
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

fn render_autocomplete_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    draw_text_bounded(
        plane,
        ox + 1,
        oy + 5,
        "[rust           ]",
        t.fg,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    if (phase * 3.0).fract() < 0.6 {
        set_cell_bounded(
            plane,
            ox + 6,
            oy + 5,
            '█',
            t.primary,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    let suggestions = ["rust-analyzer", "rustc", "cargo", "rustfmt", "clippy"];
    let offset = ((phase * 0.5).sin() * 2.0).round() as i16;
    for (i, s) in suggestions.iter().enumerate() {
        let y = oy + 6 + i;
        if y > oy + 10 || y > max_y {
            break;
        }
        let x_offset = if i == 0 { offset } else { 0 };
        let x = if x_offset >= 0 {
            (ox + 2 + x_offset as usize).min(max_x.saturating_sub(1))
        } else {
            (ox + 2).saturating_sub(x_offset.unsigned_abs() as usize)
        };
        let fg = if i == 0 { t.primary } else { t.fg_subtle };
        let prefix = if i == 0 { "> " } else { "  " };
        draw_text_bounded(
            plane,
            x,
            y,
            prefix,
            t.fg_muted,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        draw_text_bounded(
            plane,
            x + 2,
            y,
            s,
            fg,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}
fn render_notification_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    let notifications = [
        (NotificationType::Info, "Info", "File saved", t.info),
        (
            NotificationType::Success,
            "Success",
            "Build complete",
            t.success,
        ),
        (
            NotificationType::Warning,
            "Warning",
            "Low memory",
            t.warning,
        ),
        (
            NotificationType::Error,
            "Error",
            "Connection failed",
            t.error,
        ),
    ];
    let offset = ((phase * 0.3).floor() as usize) % notifications.len();
    let notif_w = (card_w - 4).clamp(6, 10);
    let notif_x = ox + card_w - notif_w - 2;
    for i in 0..2.min(notifications.len()) {
        let idx = (offset + i) % notifications.len();
        let (kind, title, msg, color) = &notifications[idx];
        let y = oy + 5 + i * 3;
        if y > oy + 11 || y > max_y {
            break;
        }
        let icon = match kind {
            NotificationType::Info => "i",
            NotificationType::Success => "[OK]",
            NotificationType::Warning => "!",
            NotificationType::Error => "[X]",
        };
        set_cell_bounded(
            plane,
            notif_x,
            y,
            '╭',
            t.outline,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        for cx in 1..notif_w - 1 {
            set_cell_bounded(
                plane,
                notif_x + cx,
                y,
                '─',
                t.outline,
                t.surface,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
        set_cell_bounded(
            plane,
            notif_x + notif_w - 1,
            y,
            '╮',
            t.outline,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        draw_text_bounded(
            plane,
            notif_x + 1,
            y,
            &format!(" {} {}", icon, title),
            *color,
            t.surface,
            Styles::BOLD,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        set_cell_bounded(
            plane,
            notif_x,
            y + 1,
            '│',
            t.outline,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        let max_msg_len = notif_w.saturating_sub(2).max(1);
        let truncated: String = msg.chars().take(max_msg_len).collect();
        draw_text_bounded(
            plane,
            notif_x + 1,
            y + 1,
            &truncated,
            t.fg,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        for cx in 1..notif_w - 1 {
            set_cell_bounded(
                plane,
                notif_x + cx,
                y + 1,
                ' ',
                t.fg,
                t.surface,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
        set_cell_bounded(
            plane,
            notif_x + notif_w - 1,
            y + 1,
            '│',
            t.outline,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        set_cell_bounded(
            plane,
            notif_x,
            y + 2,
            '╰',
            t.outline,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        for cx in 1..notif_w - 1 {
            set_cell_bounded(
                plane,
                notif_x + cx,
                y + 2,
                '─',
                t.outline,
                t.surface,
                ox + 1,
                max_x,
                oy + 1,
                max_y,
            );
        }
        set_cell_bounded(
            plane,
            notif_x + notif_w - 1,
            y + 2,
            '╯',
            t.outline,
            t.surface,
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
}

#[derive(Clone, Copy)]
enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

fn render_accessibility_preview(
    plane: &mut Plane,
    t: &Theme,
    _phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    draw_text_bounded(
        plane,
        ox + 1,
        oy + 5,
        "OSC 99 Announcements:",
        t.primary,
        t.surface,
        Styles::BOLD,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    let items = [
        ("Role:", "button"),
        ("Label:", "Submit"),
        ("Shortcut:", "Ctrl+Enter"),
        ("Level:", "assertive"),
        ("Terminal:", "NVDA"),
    ];
    for (i, (label, value)) in items.iter().enumerate() {
        let y = oy + 6 + i;
        if y > oy + 10 || y > max_y {
            break;
        }
        draw_text_bounded(
            plane,
            ox + 1,
            y,
            label,
            t.fg_muted,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        draw_text_bounded(
            plane,
            ox + 12,
            y,
            value,
            t.fg,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    draw_text_bounded(
        plane,
        ox + 1,
        oy + 11,
        "* enabled",
        t.success,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
}

fn render_cell_pool_preview(
    plane: &mut Plane,
    t: &Theme,
    phase: f64,
    ox: usize,
    oy: usize,
    card_w: usize,
    card_h: usize,
) {
    let max_x = ox + card_w - 2;
    let max_y = oy + card_h - 2;
    draw_text_bounded(
        plane,
        ox + 1,
        oy + 5,
        "Cell Pool Stats:",
        t.primary,
        t.surface,
        Styles::BOLD,
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
    let stats = [
        (
            "Avail:",
            format!("{:>4}", 1024 - ((phase * 5.0).sin() * 200.0).round() as i32),
        ),
        (
            "Used:",
            format!("{:>4}", ((phase * 5.0).sin() * 200.0).round() as i32),
        ),
        (
            "Hit:",
            format!("{:>3}%", 95 - ((phase * 3.0).sin() * 10.0) as i32),
        ),
    ];
    for (i, (label, value)) in stats.iter().enumerate() {
        let y = oy + 6 + i;
        if y > oy + 10 || y > max_y {
            break;
        }
        draw_text_bounded(
            plane,
            ox + 1,
            y,
            label,
            t.fg_muted,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
        draw_text_bounded(
            plane,
            ox + 8,
            y,
            value,
            t.success,
            t.surface,
            Styles::empty(),
            ox + 1,
            max_x,
            oy + 1,
            max_y,
        );
    }
    let hit_rate = (95.0 - (phase * 3.0).sin() * 10.0).max(0.0) as u32;
    let bar_len = (card_w - 6).clamp(4, 10);
    let filled = (hit_rate / 10) as usize;
    let bar_str = format!(
        "[{}{}]",
        "█".repeat(filled.min(bar_len)),
        "░".repeat(bar_len - filled.min(bar_len))
    );
    draw_text_bounded(
        plane,
        ox + 1,
        oy + 10,
        &bar_str,
        t.success,
        t.surface,
        Styles::empty(),
        ox + 1,
        max_x,
        oy + 1,
        max_y,
    );
}
// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET IMPL
// ═══════════════════════════════════════════════════════════════════════════════

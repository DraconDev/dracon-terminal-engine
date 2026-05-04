use chrono::Local;

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::WidgetId;
use ratatui::layout::Rect;

use crate::data::ExampleMeta;

// ═══════════════════════════════════════════════════════════════════════════════
// RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

fn draw_rounded_border(plane: &mut Plane, area: Rect, fg: Color, bg: Color, selected: bool) {
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
    set_cell(plane, 0, 0, chars.0, fg, bg);
    set_cell(plane, w - 1, 0, chars.1, fg, bg);
    set_cell(plane, 0, h - 1, chars.2, fg, bg);
    set_cell(plane, w - 1, h - 1, chars.3, fg, bg);

    // Top/bottom edges
    for x in 1..w - 1 {
        set_cell(plane, x, 0, chars.4, fg, bg);
        set_cell(plane, x, h - 1, chars.4, fg, bg);
    }

    // Left/right edges
    for y in 1..h - 1 {
        set_cell(plane, 0, y, chars.5, fg, bg);
        set_cell(plane, w - 1, y, chars.5, fg, bg);
    }

    // Fill background
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            set_cell(plane, x, y, ' ', fg, bg);
        }
    }
}

fn set_cell(plane: &mut Plane, x: usize, y: usize, ch: char, fg: Color, bg: Color) {
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

fn draw_text(plane: &mut Plane, x: usize, y: usize, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = y * plane.width as usize + x + i;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch,
                fg,
                bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false,
                skip: false,
            };
        }
    }
}

fn category_color(t: Theme, cat: &str) -> Color {
    match cat {
        "apps" => t.warning,
        "cookbook" => t.info,
        "tools" => t.secondary,
        _ => t.fg_muted,
    }
}

fn render_card(
    ex: &ExampleMeta,
    idx: usize,
    selected_idx: usize,
    hovered_idx: Option<usize>,
    t: Theme,
    phase: f64,
    card_w: u16,
    card_h: u16,
) -> Plane {
    let mut plane = Plane::new(0, card_w, card_h);
    let card_w_usize = card_w as usize;
    let card_h_usize = card_h as usize;

    let is_selected = idx == selected_idx;
    let is_hovered = Some(idx) == hovered_idx;
    let cat_color = category_color(t, ex.category);

    // Per-card phase offset for more organic animations
    let card_phase = phase + (idx as f64 * 0.73);

    let border_fg = if is_selected {
        let pulse = (card_phase * 2.0).sin() * 0.5 + 0.5;
        if pulse > 0.5 {
            t.primary
        } else {
            t.primary_hover
        }
    } else if is_hovered {
        t.primary_hover
    } else {
        t.outline
    };
    let bg = if is_selected {
        t.surface_elevated
    } else {
        t.surface
    };
    draw_rounded_border(
        &mut plane,
        Rect::new(0, 0, card_w, card_h),
        border_fg,
        bg,
        is_selected || is_hovered,
    );

    let badge = format!(" {} ", ex.category.to_uppercase());
    let badge_x = 2usize;
    let badge_y = 1usize;
    for (i, ch) in badge.chars().enumerate() {
        let px = badge_x + i;
        if px < card_w_usize - 2 {
            set_cell(&mut plane, px, badge_y, ch, t.fg_on_accent, cat_color);
        }
    }

    let name_y = 3usize;
    let max_name_len = (card_w_usize - 4).min(24);
    let name_truncated: String = ex.name.chars().take(max_name_len).collect();
    draw_text(&mut plane, 2, name_y, &name_truncated, t.fg, bg, true);

    let desc_y = 4usize;
    let max_desc_len = (card_w_usize - 4).min(24);
    let desc: String = ex.description.chars().take(max_desc_len).collect();
    draw_text(&mut plane, 2, desc_y, &desc, t.fg_muted, bg, false);

    let preview_start_y = 6usize;
    let _preview_lines = card_h_usize.saturating_sub(preview_start_y + 1);

    match ex.name {
        "system_monitor" => render_live_gauge_preview(&mut plane, t, card_phase, card_w),
        "split_resizer" => render_split_preview(&mut plane, t, card_phase, card_w),
        "command_bindings" => render_command_preview(&mut plane, t, card_phase, card_w),
        "theme_switcher" => render_theme_preview(&mut plane, t, card_phase, card_w),
        "widget_gallery" => render_widget_preview(&mut plane, t, card_phase, card_w),
        "ide" => render_ide_preview(&mut plane, t, card_phase, card_w),
        "desktop" => render_desktop_preview(&mut plane, t, card_phase, card_w),
        "chat_client" | "log_viewer" => render_scroll_preview(&mut plane, t, card_phase, card_w),
        "git_tui" => render_git_tui_preview(&mut plane, t, card_phase, card_w),
        "file_manager" => render_file_manager_preview(&mut plane, t, card_phase, card_w),
        "menu_system" => render_menu_system_preview(&mut plane, t, card_phase, card_w),
        "modal_demo" => render_modal_demo_preview(&mut plane, t, card_phase, card_w),
        _ => {
            for (i, line) in ex.preview.iter().enumerate() {
                let py = preview_start_y + i;
                if py < card_h_usize - 1 {
                    let preview_line: String = line.chars().take(card_w_usize - 4).collect();
                    draw_text(&mut plane, 2, py, &preview_line, t.fg_subtle, bg, false);
                }
            }
        }
    }

    if is_selected {
        draw_text(&mut plane, 1, card_h_usize / 2, "►", t.primary, bg, true);
    }

    plane
}

fn render_live_gauge_preview(plane: &mut Plane, t: Theme, phase: f64, _card_w: u16) {
    let items = [
        ("CPU", (phase * 30.0).sin() * 40.0 + 50.0),
        ("MEM", (phase * 20.0).sin() * 30.0 + 60.0),
        ("DISK", (phase * 15.0).sin() * 20.0 + 40.0),
        ("NET", (phase * 25.0).sin() * 50.0 + 50.0),
    ];
    for (i, (label, value)) in items.iter().enumerate() {
        let y = 6 + i;
        if y > 11 {
            break;
        }
        let bar_w = 14;
        let val = value.clamp(0.0, 100.0);
        let fill = ((val / 100.0) * bar_w as f64).round() as usize;
        let color = if val > 80.0 {
            t.error
        } else if val > 60.0 {
            t.warning
        } else {
            t.success
        };
        draw_text(plane, 2, y, label, t.fg_muted, t.surface, false);
        set_cell(plane, 6, y, '[', t.fg_muted, t.surface);
        for j in 0..bar_w {
            let ch = if j < fill { '█' } else { '░' };
            let fg = if j < fill { color } else { t.fg_muted };
            set_cell(plane, 7 + j, y, ch, fg, t.surface);
        }
        set_cell(plane, 7 + bar_w, y, ']', t.fg_muted, t.surface);
        let pct = format!("{:>3}%", val.round() as u32);
        draw_text(plane, 7 + bar_w + 2, y, &pct, color, t.surface, true);
    }
}

fn render_split_preview(plane: &mut Plane, t: Theme, phase: f64, _card_w: u16) {
    let split_x = (4.0 + (phase * 0.5).sin() * 3.0).round() as usize;
    let split_x = split_x.min(25);
    let w = 26;

    for y in 6..12 {
        for x in 1..w {
            let bg = if x <= split_x {
                t.surface_elevated
            } else {
                t.surface
            };
            let fg = if x <= split_x {
                t.fg_muted
            } else {
                t.fg_subtle
            };
            set_cell(plane, x, y, ' ', fg, bg);
        }
    }

    for y in 6..12 {
        set_cell(plane, split_x, y, '│', t.primary, t.surface_elevated);
    }

    draw_text(plane, 2, 7, "A", t.fg, t.surface_elevated, false);
    draw_text(plane, split_x + 2, 7, "B", t.fg, t.surface, false);
    let label = format!("{}:{}", split_x, 26 - split_x);
    draw_text(plane, w / 2 - 3, 11, &label, t.fg_muted, t.bg, false);
}

fn render_command_preview(plane: &mut Plane, t: Theme, phase: f64, _card_w: u16) {
    let lines = [
        format!("Load: {:.2}", 0.45 + (phase * 0.3).sin() * 0.2),
        format!(
            "CPU:  [{}{}]",
            "█".repeat((phase * 4.0).sin() as usize * 2 + 2),
            "░".repeat(6)
        ),
        format!(
            "Mem:  [{}{}]",
            "█".repeat((phase * 3.0).sin() as usize * 2 + 3),
            "░".repeat(5)
        ),
        format!(
            "Net:  [{}{}]",
            "█".repeat((phase * 2.0).sin() as usize * 2 + 1),
            "░".repeat(7)
        ),
    ];
    for (i, line) in lines.iter().enumerate() {
        let py = 6 + i;
        if py > 11 {
            break;
        }
        let truncated: String = line.chars().take(24).collect();
        draw_text(plane, 2, py, &truncated, t.fg_subtle, t.surface, false);
    }
}

fn render_theme_preview(plane: &mut Plane, t: Theme, _phase: f64, _card_w: u16) {
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
        let x = 2 + col * (swatch_size + 1);
        let y = 6 + row * 2;
        if y > 11 {
            break;
        }
        for dx in 0..swatch_size {
            set_cell(plane, x + dx, y, ' ', t.fg, *color);
            set_cell(plane, x + dx, y + 1, ' ', t.fg, *color);
        }
    }
    let name = format!("  {}  ", t.name);
    draw_text(plane, 2, 11, &name, t.fg_muted, t.bg, false);
}

fn render_widget_preview(plane: &mut Plane, t: Theme, phase: f64, _card_w: u16) {
    let checks = ["[x] Alpha", "[ ] Beta", "[x] Gamma"];
    for (i, check) in checks.iter().enumerate() {
        let py = 6 + i;
        if py > 10 {
            break;
        }
        let text: String = check.chars().take(12).collect();
        draw_text(plane, 2, py, &text, t.fg_subtle, t.surface, false);
    }

    let slider_y = 10;
    let slider_w = 18;
    let thumb = ((phase * 2.0).sin() * 0.5 + 0.5 * slider_w as f64).round() as usize;
    let thumb = thumb.min(slider_w - 1);
    draw_text(plane, 2, slider_y, "[", t.fg_muted, t.surface, false);
    for i in 0..slider_w {
        let ch = if i == thumb {
            '#'
        } else if i < thumb {
            '='
        } else {
            '-'
        };
        let fg = if i == thumb { t.primary } else { t.fg_muted };
        set_cell(plane, 3 + i, slider_y, ch, fg, t.surface);
    }
    draw_text(
        plane,
        3 + slider_w,
        slider_y,
        "]",
        t.fg_muted,
        t.surface,
        false,
    );
}

fn render_scroll_preview(plane: &mut Plane, t: Theme, phase: f64, _card_w: u16) {
    let lines = [
        "  line 0  ▸ active",
        "  line 1",
        "  line 2",
        "  line 3",
        "  line 4",
        "  line 5",
        "  line 6",
        "  line 7",
        "  line 8",
        "  line 9",
        "  line 10",
        "  line 11",
        "  line 12",
        "  line 13",
        "  line 14",
    ];

    let view_h = 6usize;
    let offset = ((phase * 2.0).sin() * 4.0).round() as usize;
    let offset = offset.min(lines.len().saturating_sub(view_h));

    let track_x = 24usize;
    let track_h = view_h;

    for (i, line) in lines.iter().enumerate() {
        let view_idx = i.saturating_sub(offset);
        if view_idx < view_h {
            let py = 6 + view_idx;
            if py < 13 {
                let text: String = line.chars().take(20).collect();
                let fg = if line.contains("active") {
                    t.primary
                } else {
                    t.fg_subtle
                };
                draw_text(plane, 2, py, &text, fg, t.surface, false);
            }
        }
    }

    let thumb_len = ((view_h as f32 / lines.len() as f32) * track_h as f32).ceil() as usize;
    let thumb_len = thumb_len.max(1);
    let max_offset = lines.len().saturating_sub(view_h);
    let thumb_pos = if max_offset == 0 {
        0
    } else {
        (offset * (track_h.saturating_sub(thumb_len)))
            .checked_div(max_offset)
            .unwrap_or(0)
    };

    for y in 0..track_h {
        let cy = 6 + y;
        if cy >= 13 {
            break;
        }
        let ch = if y >= thumb_pos && y < thumb_pos + thumb_len {
            '█'
        } else {
            '░'
        };
        let fg = if y >= thumb_pos && y < thumb_pos + thumb_len {
            t.primary
        } else {
            t.fg_muted
        };
        set_cell(plane, track_x, cy, ch, fg, t.surface);
    }
}

fn render_ide_preview(plane: &mut Plane, t: Theme, phase: f64, _card_w: u16) {
    // Tab bar with active/inactive tabs
    let tabs = [
        (" main.rs ", true),
        (" lib.rs ", false),
        (" mod.rs ", false),
    ];
    let mut tab_x = 1usize;
    let mut active_tab_start = 0usize;
    let mut active_tab_len = 0usize;
    for (label, active) in &tabs {
        let fg = if *active { t.fg_on_accent } else { t.fg_muted };
        let bg = if *active { t.primary_active } else { t.surface };
        draw_text(plane, tab_x, 5, label, fg, bg, *active);
        if *active {
            active_tab_start = tab_x;
            active_tab_len = label.len();
        }
        tab_x += label.len() + 1;
    }
    // Underline for active tab
    for dx in 0..active_tab_len {
        set_cell(
            plane,
            active_tab_start + dx,
            5 + 1,
            '▔',
            t.primary_active,
            t.surface,
        );
    }

    // Code lines with line numbers
    let lines = [
        ("1", "fn main() {"),
        ("2", "    let x = 42;"),
        ("3", "    println!(\"{}\", x);"),
        ("4", "}"),
    ];
    for (i, (num, code)) in lines.iter().enumerate() {
        let py = 6 + i;
        if py > 10 {
            break;
        }
        draw_text(plane, 1, py, num, t.fg_muted, t.surface, false);
        draw_text(plane, 3, py, code, t.fg, t.surface, false);
    }
    // Blinking cursor on line 3 (the empty line after code)
    let cursor_visible = (phase * 3.0).fract() < 0.6;
    if cursor_visible {
        set_cell(plane, 4, 6, '▎', t.primary, t.surface);
    }
}

fn render_desktop_preview(plane: &mut Plane, t: Theme, phase: f64, _card_w: u16) {
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
        let ox = offsets[i].0;
        let oy = offsets[i].1;
        let wx = (*x as i16 + ox).max(1) as usize;
        let wy = (*y as i16 + oy).max(6) as usize;
        let wx = wx.min(20);
        let wy = wy.min(11);

        set_cell(plane, wx, wy, '┌', *color, t.surface);
        for dx in 1..w - 1 {
            set_cell(plane, wx + dx, wy, '─', *color, t.surface);
        }
        set_cell(plane, wx + w - 1, wy, '┐', *color, t.surface);
        for dy in 1..h - 1 {
            set_cell(plane, wx, wy + dy, '│', *color, t.surface);
            for dx in 1..w - 1 {
                set_cell(plane, wx + dx, wy + dy, ' ', *color, t.surface);
            }
            set_cell(plane, wx + w - 1, wy + dy, '│', *color, t.surface);
        }
        set_cell(plane, wx, wy + h - 1, '└', *color, t.surface);
        for dx in 1..w - 1 {
            set_cell(plane, wx + dx, wy + h - 1, '─', *color, t.surface);
        }
        set_cell(plane, wx + w - 1, wy + h - 1, '┘', *color, t.surface);
    }
}

#[allow(dead_code)]
fn render_zindex_preview(plane: &mut Plane, t: Theme, phase: f64) {
    let wins = [
        (2, 7, 14, 5, t.primary, "z:3"),
        (8, 6, 14, 5, t.warning, "z:2"),
        (5, 8, 14, 5, t.info, "z:1"),
    ];
    let phase = phase * 0.3;
    for (i, (x, y, w, h, color, label)) in wins.iter().enumerate() {
        let ox = ((phase * (i as f64 + 1.0) * 20.0).sin() * 1.5) as i16;
        let oy = ((phase * (i as f64 + 1.0) * 15.0).sin() * 1.0) as i16;
        let wx = (*x as i16 + ox).max(1) as usize;
        let wy = (*y as i16 + oy).max(6) as usize;
        let wx = wx.min(19);
        let wy = wy.min(11);

        set_cell(plane, wx, wy, '┌', *color, t.surface);
        for dx in 1..w - 1 {
            set_cell(plane, wx + dx, wy, '─', *color, t.surface);
        }
        set_cell(plane, wx + w - 1, wy, '┐', *color, t.surface);
        for dy in 1..h - 1 {
            set_cell(plane, wx, wy + dy, '│', *color, t.surface);
            for dx in 1..w - 1 {
                set_cell(plane, wx + dx, wy + dy, ' ', *color, t.surface);
            }
            set_cell(plane, wx + w - 1, wy + dy, '│', *color, t.surface);
        }
        set_cell(plane, wx, wy + h - 1, '└', *color, t.surface);
        for dx in 1..w - 1 {
            set_cell(plane, wx + dx, wy + h - 1, '─', *color, t.surface);
        }
        set_cell(plane, wx + w - 1, wy + h - 1, '┘', *color, t.surface);

        draw_text(plane, wx + 2, wy + 1, label, *color, t.surface, true);
    }
}

fn render_git_tui_preview(plane: &mut Plane, t: Theme, phase: f64, _card_w: u16) {
    // Branch header
    draw_text(
        plane,
        2,
        6,
        " main ",
        t.fg_on_accent,
        t.primary_active,
        true,
    );
    draw_text(
        plane,
        2,
        7,
        "Status: 3 files changed",
        t.fg,
        t.surface,
        false,
    );

    // Animated diff lines cycling through different statuses
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
    let lines = &phases[phase_idx];
    for (i, (text, color)) in lines.iter().enumerate() {
        draw_text(plane, 2, 9 + i, text, *color, t.surface, false);
    }
}

fn render_file_manager_preview(plane: &mut Plane, t: Theme, phase: f64, _card_w: u16) {
    let items = [
        (0, "home/", true, 0),
        (1, "user/", true, 1),
        (2, "  src/", true, 2),
        (3, "    main.rs", false, -1),
        (3, "    lib.rs", false, -1),
        (2, "  docs/", true, 1),
        (3, "    README.md", false, -1),
    ];
    let expand_phase = ((phase * 0.5).sin() * 4.0).round() as usize % 4;
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
        let py = 6 + i;
        if py > 12 {
            break;
        }
        let icon = if *is_dir { "v" } else { ">" };
        let text = format!("{}{}", icon, name);
        let fg = if *is_dir { t.warning } else { t.fg_subtle };
        draw_text(plane, 2, py, &text, fg, t.surface, false);
    }
}

fn render_menu_system_preview(plane: &mut Plane, t: Theme, phase: f64, _card_w: u16) {
    let menus = ["File", "Edit", "View", "Help"];
    let highlight_idx = ((phase * 2.0) as usize) % menus.len();
    let menu_w = 8;

    for (i, menu) in menus.iter().enumerate() {
        let x = 2 + i * (menu_w + 1);
        let is_highlighted = i == highlight_idx;
        let bg = if is_highlighted { t.primary } else { t.surface };
        let fg = if is_highlighted { t.fg_on_accent } else { t.fg };

        for dx in 0..menu_w {
            set_cell(plane, x + dx, 6, ' ', fg, bg);
        }
        let text = format!(" {} ", menu);
        draw_text(plane, x, 6, &text, fg, bg, false);

        if is_highlighted {
            for dy in 1..5 {
                for dx in 0..menu_w {
                    if dy == 4 {
                        set_cell(plane, x + dx, 6 + dy, '─', t.primary, t.surface);
                    } else {
                        set_cell(plane, x + dx, 6 + dy, ' ', t.fg, t.surface);
                    }
                }
            }
            let dropdown_items = ["New", "Open", "Save", "Exit"];
            for (j, item) in dropdown_items.iter().enumerate() {
                draw_text(plane, x + 1, 7 + j, item, t.fg, t.surface, false);
            }
        }
    }
}

fn render_modal_demo_preview(plane: &mut Plane, t: Theme, phase: f64, _card_w: u16) {
    let modal_w = 24usize;
    let modal_h = 8usize;
    let modal_x = 2usize;
    let modal_y = 6usize;

    let pulse = (phase * 2.0).sin() * 0.5 + 0.5;
    let border_color = if pulse > 0.5 {
        t.primary
    } else {
        t.primary_hover
    };

    set_cell(
        plane,
        modal_x,
        modal_y,
        '┌',
        border_color,
        t.surface_elevated,
    );
    for dx in 1..modal_w - 1 {
        set_cell(
            plane,
            modal_x + dx,
            modal_y,
            '─',
            border_color,
            t.surface_elevated,
        );
    }
    set_cell(
        plane,
        modal_x + modal_w - 1,
        modal_y,
        '┐',
        border_color,
        t.surface_elevated,
    );

    for dy in 1..modal_h - 1 {
        set_cell(
            plane,
            modal_x,
            modal_y + dy,
            '│',
            border_color,
            t.surface_elevated,
        );
        for dx in 1..modal_w - 1 {
            set_cell(
                plane,
                modal_x + dx,
                modal_y + dy,
                ' ',
                t.fg,
                t.surface_elevated,
            );
        }
        set_cell(
            plane,
            modal_x + modal_w - 1,
            modal_y + dy,
            '│',
            border_color,
            t.surface_elevated,
        );
    }

    set_cell(
        plane,
        modal_x,
        modal_y + modal_h - 1,
        '└',
        border_color,
        t.surface_elevated,
    );
    for dx in 1..modal_w - 1 {
        set_cell(
            plane,
            modal_x + dx,
            modal_y + modal_h - 1,
            '─',
            border_color,
            t.surface_elevated,
        );
    }
    set_cell(
        plane,
        modal_x + modal_w - 1,
        modal_y + modal_h - 1,
        '┘',
        border_color,
        t.surface_elevated,
    );

    let text = " Confirm? ";
    let text_x = modal_x + (modal_w - text.len()) / 2;
    draw_text(
        plane,
        text_x,
        modal_y + 2,
        text,
        t.fg,
        t.surface_elevated,
        true,
    );

    let yes_text = "[ Yes ]";
    let no_text = "[ No  ]";
    draw_text(
        plane,
        modal_x + 4,
        modal_y + 4,
        yes_text,
        t.primary,
        t.surface_elevated,
        true,
    );
    draw_text(
        plane,
        modal_x + 14,
        modal_y + 4,
        no_text,
        t.fg_muted,
        t.surface_elevated,
        false,
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET IMPL
// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET IMPL
// ═══════════════════════════════════════════════════════════════════════════════

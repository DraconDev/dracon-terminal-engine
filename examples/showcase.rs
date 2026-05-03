#![allow(missing_docs)]
//! Dracon Terminal Engine — Example Showcase Launcher
//!
//! Interactive grid-based launcher for all framework examples.
//! Features: category filtering, real-time search, animated selection,
//! card-based layout with mini previews, live data previews, keyboard shortcuts,
//! and an interactive primitives bar demonstrating engine building blocks.
//!
//! Controls:
//!   arrows  — navigate cards
//!   Enter   — launch selected example
//!   /       — focus search bar
//!   Tab     — cycle categories
//!   t       — cycle theme
//!   d       — toggle debug overlay
//!   ?       — toggle help
//!   Space   — preview card (modal)
//!   1-5     — interact with primitives bar
//!   right-click — context menu (Launch / Copy name / Filter by category)
//!   q       — quit

use std::cell::RefCell;
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::Local;

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

// ═══════════════════════════════════════════════════════════════════════════════
// DATA
// ═══════════════════════════════════════════════════════════════════════════════

struct ExampleMeta {
    name: &'static str,
    category: &'static str,
    description: &'static str,
    binary_name: &'static str,
    preview: &'static [&'static str],
}

impl ExampleMeta {
    fn all() -> Vec<Self> {
        vec![
            // Apps
            ExampleMeta { name: "system_monitor", category: "apps", description: "Live system gauges with auto-refresh", binary_name: "system_monitor", preview: &["CPU [████████░░] 80%", "MEM [██████░░░░] 60%", "DISK [████░░░░░░] 40%", "NET  [██████████] 100%",] },
            ExampleMeta { name: "ide", category: "apps", description: "Full IDE with menus, tabs, tree, editor", binary_name: "ide", preview: &["[File][Edit][View]", "+-src/ +--------+", "| main |fn main|", "| lib  |{      |",] },
            ExampleMeta { name: "file_manager", category: "apps", description: "File browser with Tree + Table", binary_name: "file_manager", preview: &["v home/", "  v user/", "    v src/", "      > main.rs",] },
            ExampleMeta { name: "chat_client", category: "apps", description: "Rich chat UI with panels", binary_name: "chat_client", preview: &["[10:42] Alice: Hey!", "[10:43] Bob: Hi", "[10:44] Alice: Hi!", "> _",] },
            ExampleMeta { name: "git_tui", category: "apps", description: "Real Git status/log/diff/branches", binary_name: "git_tui", preview: &["[Status][Log][Diff]", " M src/main.rs", " A Cargo.toml", "?? README.md",] },
            // Cookbook
            ExampleMeta { name: "widget_gallery", category: "cookbook", description: "All interactive widgets demo", binary_name: "widget_gallery", preview: &["[x] Checkbox", "(o) Radio", "[----] Slider", "Loading [####] ",] },
            ExampleMeta { name: "command_bindings", category: "cookbook", description: "Live CLI-bound widgets", binary_name: "command_bindings", preview: &["Load: 0.45 0.32", "CPU:  [####--]", "Mem:  [######]", "Net:  [------]",] },
            ExampleMeta { name: "split_resizer", category: "cookbook", description: "Drag-to-resize SplitPane", binary_name: "split_resizer", preview: &["+-----+-----+", "|  A  |  B  |", "+--+--+-----+", "|  C  |  D  |",] },
            ExampleMeta { name: "menu_system", category: "cookbook", description: "MenuBar + ContextMenu", binary_name: "menu_system", preview: &["[File][Edit][View]", "+-----------+", "| New        |", "| Open       |",] },
            // Tools
            ExampleMeta { name: "theme_switcher", category: "tools", description: "Live theme cycling (15 themes)", binary_name: "theme_switcher", preview: &["Theme: Nord", "+----------+", "| # # # #   |", "| # # # #   |",] },
            ExampleMeta { name: "modal_demo", category: "tools", description: "Modal dialogs + focus trapping", binary_name: "modal_demo", preview: &["+---------------+", "| Confirm?     |", "| [Yes] [No]   |", "+---------------+",] },
            ExampleMeta { name: "desktop", category: "tools", description: "Draggable windows + taskbar", binary_name: "desktop", preview: &["+------++------+", "| Win1 || Win2  |", "|      ||      |", "+------++------+",] },
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SHOWCASE STATE
// ═══════════════════════════════════════════════════════════════════════════════

struct Showcase {
    examples: Vec<ExampleMeta>,
    filtered: Vec<usize>,
    selected: usize,
    category_filter: Option<&'static str>,
    search_query: String,
    search_active: bool,
    theme: Theme,
    pending_theme: Option<usize>,
    should_quit: Arc<AtomicBool>,
    pending_binary: Arc<Mutex<Option<String>>>,
    status_message: Option<(String, Instant)>,
    area: Rect,
    cols: std::cell::Cell<usize>,
    last_click_time: Option<Instant>,
    last_click_idx: Option<usize>,
    fps: Arc<AtomicU64>,
    hovered_card: Option<usize>,
    mouse_pos: Option<(u16, u16)>,
    context_menu: Option<(usize, u16, u16)>,
    context_menu_selected: usize,
    tooltip_text: Option<String>,
    tooltip_timer: Option<Instant>,
    tooltip_pos: Option<(u16, u16)>,
    show_help: bool,
    modal_preview: bool,
    show_fps: bool,
    card_start: Instant,
    primitive_toggle: bool,
    primitive_slider: f32,
    primitive_checkbox: bool,
    primitive_radio: usize,
    primitive_button: bool,
    show_debug: bool,
    primitive_button_time: Option<Instant>,
    zones: RefCell<ScopedZoneRegistry<usize>>,
}

impl Showcase {
    fn new(should_quit: Arc<AtomicBool>, pending: Arc<Mutex<Option<String>>>, fps: Arc<AtomicU64>) -> Self {
        let examples = ExampleMeta::all();
        let filtered: Vec<usize> = (0..examples.len()).collect();
        Self {
            examples,
            filtered,
            selected: 0,
            category_filter: None,
            search_query: String::new(),
            search_active: false,
            theme: Theme::nord(),
            pending_theme: None,
            should_quit,
            pending_binary: pending,
            status_message: None,
            area: Rect::new(0, 0, 80, 24),
            cols: std::cell::Cell::new(3),
            last_click_time: None,
            last_click_idx: None,
            fps,
            hovered_card: None,
            mouse_pos: None,
            context_menu: None,
            context_menu_selected: 0,
            tooltip_text: None,
            tooltip_timer: None,
            tooltip_pos: None,
            show_help: false,
            modal_preview: false,
            show_fps: true,
            card_start: Instant::now(),
            primitive_toggle: false,
            primitive_slider: 0.5,
            primitive_checkbox: true,
            primitive_radio: 0,
            primitive_button: false,
            show_debug: false,
            primitive_button_time: None,
            zones: RefCell::new(ScopedZoneRegistry::new()),
        }
    }

    fn themes() -> Vec<(&'static str, Theme)> {
        vec![
            ("dark", Theme::dark()),
            ("light", Theme::light()),
            ("cyberpunk", Theme::cyberpunk()),
            ("dracula", Theme::dracula()),
            ("nord", Theme::nord()),
            ("catppuccin", Theme::catppuccin_mocha()),
            ("gruvbox", Theme::gruvbox_dark()),
            ("tokyo-night", Theme::tokyo_night()),
            ("solarized-dark", Theme::solarized_dark()),
            ("solarized-light", Theme::solarized_light()),
            ("one-dark", Theme::one_dark()),
            ("rose-pine", Theme::rose_pine()),
            ("kanagawa", Theme::kanagawa()),
            ("everforest", Theme::everforest()),
            ("monokai", Theme::monokai()),
        ]
    }

    fn apply_filter(&mut self) {
        if let Some(idx) = self.pending_theme.take() {
            self.theme = Self::themes()[idx % Self::themes().len()].1;
        }
        self.filtered = self.examples.iter().enumerate()
            .filter(|(_, ex)| {
                let matches_category = self.category_filter.map_or(true, |cat| ex.category == cat);
                let matches_search = if self.search_query.is_empty() {
                    true
                } else {
                    let q = self.search_query.to_lowercase();
                    ex.name.to_lowercase().contains(&q) ||
                    ex.description.to_lowercase().contains(&q) ||
                    ex.category.to_lowercase().contains(&q)
                };
                matches_category && matches_search
            })
            .map(|(i, _)| i)
            .collect();
        self.selected = self.selected.min(self.filtered.len().saturating_sub(1));
    }

    fn selected_example(&self) -> Option<&ExampleMeta> {
        self.filtered.get(self.selected).and_then(|&idx| self.examples.get(idx))
    }

    fn launch_selected(&mut self) {
        if let Some(ex) = self.selected_example() {
            *self.pending_binary.lock().unwrap() = Some(ex.binary_name.to_string());
            self.status_message = Some((format!("Launching {}...", ex.name), Instant::now()));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

fn draw_rounded_border(plane: &mut Plane, area: Rect, fg: Color, bg: Color, selected: bool) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w < 2 || h < 2 { return; }

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

fn render_card(ex: &ExampleMeta, idx: usize, selected_idx: usize, hovered_idx: Option<usize>, t: Theme, phase: f64) -> Plane {
    let card_w = 28u16;
    let card_h = 14u16;
    let mut plane = Plane::new(0, card_w, card_h);

    let is_selected = idx == selected_idx;
    let is_hovered = Some(idx) == hovered_idx;
    let cat_color = category_color(t, ex.category);

    let border_fg = if is_selected {
        let pulse = (phase * 2.0).sin() * 0.5 + 0.5;
        if pulse > 0.5 { t.primary } else { t.primary_hover }
    } else if is_hovered {
        t.primary_hover
    } else {
        t.outline
    };
    let bg = if is_selected { t.surface_elevated } else if is_hovered { t.surface } else { t.surface };
    draw_rounded_border(&mut plane, Rect::new(0, 0, card_w, card_h), border_fg, bg, is_selected || is_hovered);

    let badge = format!(" {} ", ex.category.to_uppercase());
    let badge_x = 2usize;
    let badge_y = 1usize;
    for (i, ch) in badge.chars().enumerate() {
        let px = badge_x + i;
        if px < plane.width as usize - 2 {
            set_cell(&mut plane, px, badge_y, ch, t.fg_on_accent, cat_color);
        }
    }

    let name_y = 3usize;
    let name_truncated = if ex.name.len() > 24 { &ex.name[..24] } else { ex.name };
    draw_text(&mut plane, 2, name_y, name_truncated, t.fg, bg, true);

    let desc_y = 4usize;
    let desc: String = ex.description.chars().take(24).collect();
    draw_text(&mut plane, 2, desc_y, &desc, t.fg_muted, bg, false);

    match ex.name {
        "system_monitor" => render_live_gauge_preview(&mut plane, t, phase),
        "split_resizer" => render_split_preview(&mut plane, t, phase),
        "command_bindings" => render_command_preview(&mut plane, t, phase),
        "theme_switcher" => render_theme_preview(&mut plane, t, phase),
        "widget_gallery" => render_widget_preview(&mut plane, t, phase),
        "ide" => render_ide_preview(&mut plane, t, phase),
        "desktop" => render_desktop_preview(&mut plane, t, phase),
        "chat_client" | "log_viewer" => render_scroll_preview(&mut plane, t, phase),
        "git_tui" => render_git_tui_preview(&mut plane, t, phase),
        "file_manager" => render_file_manager_preview(&mut plane, t, phase),
        "menu_system" => render_menu_system_preview(&mut plane, t, phase),
        "modal_demo" => render_modal_demo_preview(&mut plane, t, phase),
        _ => {
            for (i, line) in ex.preview.iter().enumerate() {
                let py = 6 + i;
                if py < card_h as usize - 1 {
                    let preview_line: String = line.chars().take(24).collect();
                    draw_text(&mut plane, 2, py, &preview_line, t.fg_subtle, bg, false);
                }
            }
        }
    }

    if is_selected {
        draw_text(&mut plane, 1, card_h as usize / 2, "►", t.primary, bg, true);
    }

    plane
}

fn render_live_gauge_preview(plane: &mut Plane, t: Theme, phase: f64) {
    let items = [
        ("CPU", (phase * 30.0).sin() * 40.0 + 50.0),
        ("MEM", (phase * 20.0).sin() * 30.0 + 60.0),
        ("DISK", (phase * 15.0).sin() * 20.0 + 40.0),
        ("NET", (phase * 25.0).sin() * 50.0 + 50.0),
    ];
    for (i, (label, value)) in items.iter().enumerate() {
        let y = 6 + i;
        if y > 11 { break; }
        let bar_w = 14;
        let val = value.max(0.0).min(100.0);
        let fill = ((val / 100.0) * bar_w as f64).round() as usize;
        let color = if val > 80.0 { t.error } else if val > 60.0 { t.warning } else { t.success };
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

fn render_split_preview(plane: &mut Plane, t: Theme, phase: f64) {
    let split_x = (4.0 + (phase * 0.5).sin() * 3.0).round() as usize;
    let split_x = split_x.min(25);
    let w = 26;

    for y in 6..12 {
        for x in 1..w {
            let bg = if x <= split_x { t.surface_elevated } else { t.surface };
            let fg = if x <= split_x { t.fg_muted } else { t.fg_subtle };
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

fn render_command_preview(plane: &mut Plane, t: Theme, phase: f64) {
    let lines = [
        format!("Load: {:.2}", 0.45 + (phase * 0.3).sin() * 0.2),
        format!("CPU:  [{}{}]", "█".repeat((phase * 4.0).sin() as usize * 2 + 2), "░".repeat(6)),
        format!("Mem:  [{}{}]", "█".repeat((phase * 3.0).sin() as usize * 2 + 3), "░".repeat(5)),
        format!("Net:  [{}{}]", "█".repeat((phase * 2.0).sin() as usize * 2 + 1), "░".repeat(7)),
    ];
    for (i, line) in lines.iter().enumerate() {
        let py = 6 + i;
        if py > 11 { break; }
        let truncated: String = line.chars().take(24).collect();
        draw_text(plane, 2, py, &truncated, t.fg_subtle, t.surface, false);
    }
}

fn render_theme_preview(plane: &mut Plane, t: Theme, _phase: f64) {
    let colors = [t.primary, t.primary_hover, t.success, t.warning, t.error, t.info, t.fg, t.bg];
    let cols = 4;
    let swatch_size = 3;
    for (i, color) in colors.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let x = 2 + col * (swatch_size + 1);
        let y = 6 + row * 2;
        if y > 11 { break; }
        for dx in 0..swatch_size {
            set_cell(plane, x + dx, y, ' ', t.fg, *color);
            set_cell(plane, x + dx, y + 1, ' ', t.fg, *color);
        }
    }
    let name = format!("  {}  ", t.name);
    draw_text(plane, 2, 11, &name, t.fg_muted, t.bg, false);
}

fn render_widget_preview(plane: &mut Plane, t: Theme, phase: f64) {
    let checks = ["[x] Alpha", "[ ] Beta", "[x] Gamma"];
    for (i, check) in checks.iter().enumerate() {
        let py = 6 + i;
        if py > 10 { break; }
        let text: String = check.chars().take(12).collect();
        draw_text(plane, 2, py, &text, t.fg_subtle, t.surface, false);
    }

    let slider_y = 10;
    let slider_w = 18;
    let thumb = ((phase * 2.0).sin() * 0.5 + 0.5 * slider_w as f64).round() as usize;
    let thumb = thumb.min(slider_w - 1);
    draw_text(plane, 2, slider_y, "[", t.fg_muted, t.surface, false);
    for i in 0..slider_w {
        let ch = if i == thumb { '#' } else if i < thumb { '=' } else { '-' };
        let fg = if i == thumb { t.primary } else { t.fg_muted };
        set_cell(plane, 3 + i, slider_y, ch, fg, t.surface);
    }
    draw_text(plane, 3 + slider_w, slider_y, "]", t.fg_muted, t.surface, false);
}

fn render_scroll_preview(plane: &mut Plane, t: Theme, phase: f64) {
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
                let fg = if line.contains("active") { t.primary } else { t.fg_subtle };
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
        (offset * (track_h.saturating_sub(thumb_len))).checked_div(max_offset).unwrap_or(0)
    };

    for y in 0..track_h {
        let cy = 6 + y;
        if cy >= 13 { break; }
        let ch = if y >= thumb_pos && y < thumb_pos + thumb_len { '█' } else { '░' };
        let fg = if y >= thumb_pos && y < thumb_pos + thumb_len { t.primary } else { t.fg_muted };
        set_cell(plane, track_x, cy, ch, fg, t.surface);
    }
}

fn render_ide_preview(plane: &mut Plane, t: Theme, phase: f64) {
    // Tab bar with active/inactive tabs
    let tabs = [(" main.rs ", true), (" lib.rs ", false), (" mod.rs ", false)];
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
        set_cell(plane, active_tab_start + dx, 5 + 1, '▔', t.primary_active, t.surface);
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
        if py > 10 { break; }
        draw_text(plane, 1, py, num, t.fg_muted, t.surface, false);
        draw_text(plane, 3, py, code, t.fg, t.surface, false);
    }
    // Blinking cursor on line 3 (the empty line after code)
    let cursor_visible = (phase * 3.0).fract() < 0.6;
    if cursor_visible {
        set_cell(plane, 4, 6, '▎', t.primary, t.surface);
    }
}

fn render_desktop_preview(plane: &mut Plane, t: Theme, phase: f64) {
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
        let ox = offsets[i].0 as i16;
        let oy = offsets[i].1 as i16;
        let wx = (*x as i16 + ox).max(1) as usize;
        let wy = (*y as i16 + oy).max(6) as usize;
        let wx = wx.min(20);
        let wy = wy.min(11);

        set_cell(plane, wx, wy, '┌', *color, t.surface);
        for dx in 1..w - 1 { set_cell(plane, wx + dx, wy, '─', *color, t.surface); }
        set_cell(plane, wx + w - 1, wy, '┐', *color, t.surface);
        for dy in 1..h - 1 {
            set_cell(plane, wx, wy + dy, '│', *color, t.surface);
            for dx in 1..w - 1 { set_cell(plane, wx + dx, wy + dy, ' ', *color, t.surface); }
            set_cell(plane, wx + w - 1, wy + dy, '│', *color, t.surface);
        }
        set_cell(plane, wx, wy + h - 1, '└', *color, t.surface);
        for dx in 1..w - 1 { set_cell(plane, wx + dx, wy + h - 1, '─', *color, t.surface); }
        set_cell(plane, wx + w - 1, wy + h - 1, '┘', *color, t.surface);
    }
}

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
        for dx in 1..w - 1 { set_cell(plane, wx + dx, wy, '─', *color, t.surface); }
        set_cell(plane, wx + w - 1, wy, '┐', *color, t.surface);
        for dy in 1..h - 1 {
            set_cell(plane, wx, wy + dy, '│', *color, t.surface);
            for dx in 1..w - 1 { set_cell(plane, wx + dx, wy + dy, ' ', *color, t.surface); }
            set_cell(plane, wx + w - 1, wy + dy, '│', *color, t.surface);
        }
        set_cell(plane, wx, wy + h - 1, '└', *color, t.surface);
        for dx in 1..w - 1 { set_cell(plane, wx + dx, wy + h - 1, '─', *color, t.surface); }
set_cell(plane, wx + w - 1, wy + h - 1, '┘', *color, t.surface);

        draw_text(plane, wx + 2, wy + 1, label, *color, t.surface, true);
    }
}

fn render_file_manager_preview(plane: &mut Plane, t: Theme, phase: f64) {
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
    let visible_depth = if expand_phase == 0 { 1 } else if expand_phase == 1 { 2 } else if expand_phase == 2 { 3 } else { 4 };

    for (i, (indent, name, is_dir, _)) in items.iter().enumerate() {
        if *indent as usize > visible_depth { continue; }
        let py = 6 + i;
        if py > 12 { break; }
        let icon = if *is_dir { "v" } else { ">" };
        let text = format!("{}{}", icon, name);
        let fg = if *is_dir { t.warning } else { t.fg_subtle };
        draw_text(plane, 2, py, &text, fg, t.surface, false);
    }
}

fn render_menu_system_preview(plane: &mut Plane, t: Theme, phase: f64) {
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

fn render_modal_demo_preview(plane: &mut Plane, t: Theme, phase: f64) {
    let modal_w = 24usize;
    let modal_h = 8usize;
    let modal_x = 2usize;
    let modal_y = 6usize;

    let pulse = (phase * 2.0).sin() * 0.5 + 0.5;
    let border_color = if pulse > 0.5 { t.primary } else { t.primary_hover };

    set_cell(plane, modal_x, modal_y, '┌', border_color, t.surface_elevated);
    for dx in 1..modal_w - 1 { set_cell(plane, modal_x + dx, modal_y, '─', border_color, t.surface_elevated); }
    set_cell(plane, modal_x + modal_w - 1, modal_y, '┐', border_color, t.surface_elevated);

    for dy in 1..modal_h - 1 {
        set_cell(plane, modal_x, modal_y + dy, '│', border_color, t.surface_elevated);
        for dx in 1..modal_w - 1 { set_cell(plane, modal_x + dx, modal_y + dy, ' ', t.fg, t.surface_elevated); }
        set_cell(plane, modal_x + modal_w - 1, modal_y + dy, '│', border_color, t.surface_elevated);
    }

    set_cell(plane, modal_x, modal_y + modal_h - 1, '└', border_color, t.surface_elevated);
    for dx in 1..modal_w - 1 { set_cell(plane, modal_x + dx, modal_y + modal_h - 1, '─', border_color, t.surface_elevated); }
    set_cell(plane, modal_x + modal_w - 1, modal_y + modal_h - 1, '┘', border_color, t.surface_elevated);

    let text = " Confirm? ";
    let text_x = modal_x + (modal_w - text.len()) / 2;
    draw_text(plane, text_x, modal_y + 2, text, t.fg, t.surface_elevated, true);

    let yes_text = "[ Yes ]";
    let no_text = "[ No  ]";
    draw_text(plane, modal_x + 4, modal_y + 4, yes_text, t.primary, t.surface_elevated, true);
    draw_text(plane, modal_x + 14, modal_y + 4, no_text, t.fg_muted, t.surface_elevated, false);
}

// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET IMPL
// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET IMPL
// ═══════════════════════════════════════════════════════════════════════════════

impl Widget for Showcase {
    fn id(&self) -> WidgetId { WidgetId::new(0) }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = self.theme;

        // Background fill
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Clear and rebuild zone registry for this frame
        self.zones.borrow_mut().clear();

        // Title bar with decorative border
        let title_text = " Dracon Terminal Engine ";
        let title_x = 2usize;
        let title_y = 0usize;

        for (i, ch) in title_text.chars().enumerate() {
            let px = title_x + i;
            if px < area.width as usize {
                set_cell(&mut plane, px, title_y, ch, t.primary, t.bg);
            }
        }

        // Live clock
        let now = Local::now();
        let clock_text = now.format("%H:%M:%S").to_string();
        let clock_x = title_x + title_text.len() + 2;
        if clock_x + clock_text.len() < area.width as usize - 10 {
            draw_text(&mut plane, clock_x, title_y, &clock_text, t.fg_muted, t.bg, false);
        }

        // FPS counter (right-aligned)
        if self.show_fps {
            let fps_val = self.fps.load(Ordering::Relaxed);
            let fps_text = format!("{} FPS", fps_val);
            let fps_x = area.width as usize - fps_text.len() - 2;
            if fps_x > title_x + title_text.len() {
                draw_text(&mut plane, fps_x, title_y, &fps_text, t.success, t.bg, false);
            }
        }
        
        // FPS toggle checkbox
        let fps_toggle = if self.show_fps { "[x] FPS" } else { "[ ] FPS" };
        let toggle_x = area.width as usize - fps_toggle.len() - 2;
        draw_text(&mut plane, toggle_x, title_y, fps_toggle, t.fg_muted, t.bg, false);
        // Zone: FPS toggle
        let mut zones = self.zones.borrow_mut();
        zones.register(400, toggle_x as u16, title_y as u16, fps_toggle.len() as u16, 1);
        drop(zones);

        // Theme palette bar
        let palette_y = 1usize;
        let themes = Self::themes();
        let square_w = 2usize;
        let gap = 1usize;
        let max_visible = (area.width as usize).saturating_sub(4) / (square_w + gap);
        let visible_themes = max_visible.min(themes.len());
        let palette_start_x = ((area.width as usize).saturating_sub(visible_themes * (square_w + gap))) / 2;
        const PALETTE_BASE: usize = 200;
        // Determine hovered palette swatch
        let hovered_palette = self.mouse_pos
            .filter(|(_, my)| *my as usize == palette_y)
            .and_then(|(mx, _)| {
                let x = mx as usize;
                if x >= palette_start_x && x < palette_start_x + visible_themes * (square_w + gap) {
                    let rel = x - palette_start_x;
                    let idx = rel / (square_w + gap);
                    if rel % (square_w + gap) < square_w { Some(idx) } else { None }
                } else {
                    None
                }
            })
            .filter(|idx| *idx < themes.len());
        for (i, (_name, theme)) in themes.iter().enumerate() {
            if i >= visible_themes { break; }
            let x = palette_start_x + i * (square_w + gap);
            let is_active = theme.name == self.theme.name;
            let is_hovered = hovered_palette == Some(i);
            let bg = if is_hovered { theme.primary_hover } else if is_active { theme.primary_active } else { theme.primary };
            let fg = if is_hovered || is_active { theme.fg_on_accent } else { theme.fg_muted };
            // Draw 2-char wide colored square
            for dx in 0..square_w {
                if x + dx < area.width as usize {
                    let ch = if dx == 0 && is_active && !is_hovered { '▶' } else { ' ' };
                    set_cell(&mut plane, x + dx, palette_y, ch, fg, bg);
                }
            }
            // Register zone for this palette swatch
            let mut zones = self.zones.borrow_mut();
            zones.register(PALETTE_BASE + i, x as u16, palette_y as u16, square_w as u16, 1);
            drop(zones);
        }

        // Stats bar
        let stats_y = 2usize;
        let stats_text = format!(
            " {} Examples  │  {} Widgets  │  {} Themes ",
            self.examples.len(),
            35,
            themes.len()
        );
        let stats_start = 2usize;
        draw_text(&mut plane, stats_start, stats_y, &stats_text, t.fg_muted, t.bg, false);
        for x in stats_start + stats_text.len()..area.width as usize - 2 {
            set_cell(&mut plane, x, stats_y, '─', t.outline, t.bg);
        }

        // Search bar with icon
        let search_y = 3usize;
        let search_icon = if self.search_active { ">" } else { ":" };
        let search_prompt = if self.search_active { ">" } else { " " };
        let search_text = format!("{} {} [{}]", search_icon, search_prompt, self.search_query);
        let search_fg = if self.search_active { t.primary } else { t.fg_muted };
        let search_text_chars = search_text.chars().count() + 1;
        draw_text(&mut plane, 2, search_y, &search_text, search_fg, t.surface, false);
        // Fill rest of search bar
        for x in search_text_chars + 2..area.width as usize - 2 {
            set_cell(&mut plane, x, search_y, ' ', search_fg, t.surface);
        }
        // Draw cursor if active
        if self.search_active && !self.search_query.is_empty() {
            let cursor_x = 2 + search_text_chars - 1;
            if cursor_x < area.width as usize - 2 {
                set_cell(&mut plane, cursor_x, search_y, '_', t.primary, t.surface);
            }
        }

        // Search feedback: match count or no results
        if self.search_active && !self.search_query.is_empty() {
            let feedback_text = if self.filtered.is_empty() {
                "  No results!".to_string()
            } else if self.filtered.len() == 1 {
                "  1 result".to_string()
            } else {
                format!("  {} results", self.filtered.len())
            };
            let feedback_color = if self.filtered.is_empty() { t.error } else { t.fg_muted };
            draw_text(&mut plane, 2, 5, &feedback_text, feedback_color, t.bg, false);
        }

        // Primitives bar
        let prim_y = 4usize;
        let state_0 = if self.primitive_toggle { "[*] Toggle" } else { "[ ] Toggle" };
        let state_1 = {
            let pos = ((self.primitive_slider * 10.0).round() as usize).min(10);
            let filled: String = (0..pos).map(|_| '=').collect();
            let empty: String = (pos..10).map(|_| "-").collect();
            format!("[{}{}]", filled, empty)
        };
        let state_2 = if self.primitive_checkbox { "[x] Check" } else { "[ ] Check" };
        let state_3 = {
            let sel = self.primitive_radio;
            let opts = ["(1)", "(2)", "(3)"];
            let mut s = String::new();
            for (j, _o) in opts.iter().enumerate() {
                s.push_str(if j == sel { "(*)" } else { "( )" });
            }
            s
        };
        let state_4 = if self.primitive_button { "[CLICKED!]" } else { "[ Button ]" };
        let prim_controls: [(&str, &str); 5] = [
            ("[1]", state_0),
            ("[2]", &state_1),
            ("[3]", state_2),
            ("[4]", &state_3),
            ("[5]", state_4),
        ];
        // Compute positions and register zones
        let mut prim_x = 2usize;
        let mut zones = self.zones.borrow_mut();
        const PRIM_BASE: usize = 100;
        for (i, (key, state)) in prim_controls.iter().enumerate() {
            let total_w = key.len() + 1 + state.len();
            zones.register(PRIM_BASE + i, prim_x as u16, prim_y as u16, total_w as u16, 1);
            prim_x += total_w + 3;
        }
        // Determine hover from zones
        let hovered_prim = self.mouse_pos
            .and_then(|(mx, my)| zones.dispatch(mx, my))
            .filter(|id| *id >= PRIM_BASE && *id < PRIM_BASE + 5)
            .map(|id| id - PRIM_BASE);
        drop(zones); // release borrow before using plane
        // Draw primitives with hover highlight
        prim_x = 2usize;
        for (i, (key, state)) in prim_controls.iter().enumerate() {
            let hovered = hovered_prim == Some(i);
            let key_fg = if hovered { t.primary } else { t.fg_muted };
            let state_fg = if hovered { t.primary } else { t.fg };
            draw_text(&mut plane, prim_x, prim_y, key, key_fg, t.bg, false);
            prim_x += key.len();
            draw_text(&mut plane, prim_x, prim_y, " ", t.fg_muted, t.bg, false);
            prim_x += 1;
            draw_text(&mut plane, prim_x, prim_y, state, state_fg, t.bg, false);
            prim_x += state.len() + 3;
        }

        // Category sidebar
        let sidebar_w = 14usize;
        let sidebar_start_y = 6usize;
        let categories = ["all", "apps", "cookbook", "tools"];
        const CAT_BASE: usize = 300;
        // Determine hovered sidebar category
        let hovered_cat = self.mouse_pos
            .filter(|(mx, my)| {
                let y = *my as usize;
                (*mx as usize) < sidebar_w && y >= sidebar_start_y && y < sidebar_start_y + 8
            })
            .map(|(_, my)| (my as usize - sidebar_start_y) / 2)
            .filter(|idx| *idx < categories.len());
        for (i, cat) in categories.iter().enumerate() {
            let cat_y = sidebar_start_y + i * 2;
            let is_active = self.category_filter.map_or(*cat == "all", |f| f == *cat);
            let is_hovered = hovered_cat == Some(i);
            let (fg, bg_cat) = if is_hovered {
                (t.fg, t.surface_elevated)
            } else if is_active {
                (t.fg_on_accent, t.primary_active)
            } else {
                (t.fg_muted, t.bg)
            };
            let (icon, label) = match *cat {
                "all" => ("◈", " ALL "),
                "apps" => ("▣", " APPS "),
                "cookbook" => ("◉", " COOKBOOK "),
                "tools" => ("◦", " TOOLS "),
                _ => ("•", *cat),
            };
            let icon_fg = if is_hovered { t.primary } else if is_active { t.primary } else { t.fg_muted };
            draw_text(&mut plane, 1, cat_y, icon, icon_fg, bg_cat, is_active || is_hovered);
            draw_text(&mut plane, 3, cat_y, label, fg, bg_cat, is_active || is_hovered);
            // Register zone for this category
            let mut zones = self.zones.borrow_mut();
            zones.register(CAT_BASE + i, 0, cat_y as u16, sidebar_w as u16, 1);
            drop(zones);

            // Count badge
            let count = if *cat == "all" {
                self.examples.len()
            } else {
                self.examples.iter().filter(|e| e.category == *cat).count()
            };
            let count_str = format!("{:>2}", count);
            draw_text(&mut plane, 13, cat_y, &count_str, t.fg_muted, bg_cat, false);
        }

        // Grid of cards
        let grid_start_x = sidebar_w + 2;
        let grid_start_y = sidebar_start_y + 1;
        let card_w = 28usize;
        let card_h = 14usize;
        self.cols.set(((area.width as usize - grid_start_x) / (card_w + 2)).max(1));
        let cols = self.cols.get();

        for (grid_idx, &ex_idx) in self.filtered.iter().enumerate() {
            if let Some(ex) = self.examples.get(ex_idx) {
                let col = grid_idx % cols;
                let row = grid_idx / cols;
                let x = grid_start_x + col * (card_w + 2);
                let y = grid_start_y + row * (card_h + 1);

                if x + card_w > area.width as usize || y + card_h > area.height as usize - 2 {
                    continue;
                }

                let card = render_card(ex, grid_idx, self.selected, self.hovered_card, t, self.card_start.elapsed().as_secs_f64());
                for cy in 0..card_h {
                    for cx in 0..card_w {
                        let src_idx = (cy * card_w + cx) as usize;
                        let dst_idx = ((y + cy as usize) * area.width as usize + x + cx as usize) as usize;
                        if src_idx < card.cells.len() && dst_idx < plane.cells.len() {
                            if !card.cells[src_idx].transparent {
                                plane.cells[dst_idx] = card.cells[src_idx].clone();
                            }
                        }
                    }
                }
                // Register zone for this card
                const CARD_BASE: usize = 500;
                let mut zones = self.zones.borrow_mut();
                zones.register(CARD_BASE + grid_idx, x as u16, y as u16, card_w as u16, card_h as u16);
                drop(zones);
            }
        }

        // Scroll indicator
        let total_cards = self.filtered.len();
        let visible_cards = cols * ((area.height as usize - grid_start_y - 2) / (card_h + 1)).max(1);
        if total_cards > visible_cards {
            let scroll_text = format!("{} more", total_cards - visible_cards);
            draw_text(&mut plane, area.width as usize - scroll_text.len() - 2, area.height as usize - 3, &scroll_text, t.fg_muted, t.bg, false);
        }

        // Status bar
        let status_y = area.height as usize - 1;
        for x in 0..area.width as usize {
            set_cell(&mut plane, x, status_y, ' ', t.fg, t.surface_elevated);
        }

        let hints = ["↑↓←→ nav", "Enter launch", "/ search", "Tab category", "t theme", "q quit"];
        let mut hint_x = 2usize;
        for hint in hints.iter() {
            draw_text(&mut plane, hint_x, status_y, hint, t.primary, t.surface_elevated, false);
            hint_x += hint.len() + 3;
        }

        // Mouse coordinates (right side)
        if let Some((mx, my)) = self.mouse_pos {
            let coords = format!("{}:{}", mx, my);
            let coords_x = area.width as usize - coords.len() - 2;
            if coords_x > hint_x {
                draw_text(&mut plane, coords_x, status_y, &coords, t.fg_muted, t.surface_elevated, false);
            }
        }

        // Status message (temporary) - toast style
        if let Some((ref msg, time)) = self.status_message {
            if time.elapsed() < Duration::from_secs(2) {
                let msg_y = area.height as usize / 2;
                let msg_x = ((area.width as usize).saturating_sub(msg.len() + 6)) / 2;
                let msg_w = msg.len() + 6;
                
                // Toast background
                for cx in 0..msg_w {
                    if msg_x + cx < area.width as usize {
                        set_cell(&mut plane, msg_x + cx, msg_y - 1, ' ', t.fg, t.warning);
                        set_cell(&mut plane, msg_x + cx, msg_y, ' ', t.fg, t.warning);
                        set_cell(&mut plane, msg_x + cx, msg_y + 1, ' ', t.fg, t.warning);
                    }
                }
                
                // Toast border
                for cx in 0..msg_w {
                    if msg_x + cx < area.width as usize {
                        set_cell(&mut plane, msg_x + cx, msg_y - 1, '─', t.warning, t.warning);
                        set_cell(&mut plane, msg_x + cx, msg_y + 1, '─', t.warning, t.warning);
                    }
                }
                set_cell(&mut plane, msg_x, msg_y - 1, '┌', t.warning, t.warning);
                set_cell(&mut plane, msg_x + msg_w - 1, msg_y - 1, '┐', t.warning, t.warning);
                set_cell(&mut plane, msg_x, msg_y + 1, '└', t.warning, t.warning);
                set_cell(&mut plane, msg_x + msg_w - 1, msg_y + 1, '┘', t.warning, t.warning);
                set_cell(&mut plane, msg_x, msg_y, '│', t.warning, t.warning);
                set_cell(&mut plane, msg_x + msg_w - 1, msg_y, '│', t.warning, t.warning);
                
                // Message text
                draw_text(&mut plane, msg_x + 3, msg_y, msg, t.bg, t.warning, true);
            }
        }

        // Context menu
        if let Some((card_idx, mx, my)) = self.context_menu {
            if let Some(&ex_idx) = self.filtered.get(card_idx) {
                if let Some(ex) = self.examples.get(ex_idx) {
                    let menu_x = (mx as usize).min(area.width as usize - 20);
                    let menu_y = (my as usize).min(area.height as usize - 6);
                    let menu_w = 18usize;
                    let menu_h = 6usize;
                    let menu_items: [String; 4] = [
                        "▶ Launch".to_string(),
                        format!("  Copy: {}", ex.binary_name),
                        format!("  Filter: {}", ex.category),
                        "  Cancel".to_string(),
                    ];
                    

                    // Menu background
                    for cy in 0..menu_h {
                        for cx in 0..menu_w {
                            set_cell(&mut plane, menu_x + cx, menu_y + cy, ' ', t.fg, t.surface_elevated);
                        }
                    }

                    // Border
                    for cx in 0..menu_w {
                        set_cell(&mut plane, menu_x + cx, menu_y, '─', t.outline, t.surface_elevated);
                        set_cell(&mut plane, menu_x + cx, menu_y + menu_h - 1, '─', t.outline, t.surface_elevated);
                    }
                    for cy in 0..menu_h {
                        set_cell(&mut plane, menu_x, menu_y + cy, '│', t.outline, t.surface_elevated);
                        set_cell(&mut plane, menu_x + menu_w - 1, menu_y + cy, '│', t.outline, t.surface_elevated);
                    }
                    set_cell(&mut plane, menu_x, menu_y, '┌', t.outline, t.surface_elevated);
                    set_cell(&mut plane, menu_x + menu_w - 1, menu_y, '┐', t.outline, t.surface_elevated);
                    set_cell(&mut plane, menu_x, menu_y + menu_h - 1, '└', t.outline, t.surface_elevated);
                    set_cell(&mut plane, menu_x + menu_w - 1, menu_y + menu_h - 1, '┘', t.outline, t.surface_elevated);

                    // Menu items
                    for (i, item) in menu_items.iter().enumerate() {
                        let selected = i == self.context_menu_selected;
                        let fg = if selected { t.bg } else if i == 0 { t.primary } else { t.fg };
                        let bg = if selected { t.primary } else { t.surface_elevated };
                        draw_text(&mut plane, menu_x + 2, menu_y + 1 + i, item, fg, bg, false);
                    }
                }
            }
        }

        // Tooltip on hover
        if let Some(ref text) = self.tooltip_text {
            if let Some((tx, ty)) = self.tooltip_pos {
                let tooltip_x = (tx as usize).min(area.width as usize - text.len() - 4);
                let tooltip_y = (ty as usize).saturating_sub(2);
                let tooltip_w = text.len() + 4;
                let tooltip_h = 3usize;
                
                // Background
                for cy in 0..tooltip_h {
                    for cx in 0..tooltip_w {
                        if tooltip_x + cx < area.width as usize && tooltip_y + cy < area.height as usize {
                            set_cell(&mut plane, tooltip_x + cx, tooltip_y + cy, ' ', t.fg, t.surface_elevated);
                        }
                    }
                }
                
                // Border
                for cx in 0..tooltip_w {
                    if tooltip_x + cx < area.width as usize && tooltip_y < area.height as usize {
                        set_cell(&mut plane, tooltip_x + cx, tooltip_y, '─', t.outline, t.surface_elevated);
                    }
                    if tooltip_x + cx < area.width as usize && tooltip_y + tooltip_h - 1 < area.height as usize {
                        set_cell(&mut plane, tooltip_x + cx, tooltip_y + tooltip_h - 1, '─', t.outline, t.surface_elevated);
                    }
                }
                for cy in 0..tooltip_h {
                    if tooltip_x < area.width as usize && tooltip_y + cy < area.height as usize {
                        set_cell(&mut plane, tooltip_x, tooltip_y + cy, '│', t.outline, t.surface_elevated);
                    }
                    if tooltip_x + tooltip_w - 1 < area.width as usize && tooltip_y + cy < area.height as usize {
                        set_cell(&mut plane, tooltip_x + tooltip_w - 1, tooltip_y + cy, '│', t.outline, t.surface_elevated);
                    }
                }
                
                // Text
                if tooltip_y + 1 < area.height as usize {
                    draw_text(&mut plane, tooltip_x + 2, tooltip_y + 1, text, t.fg, t.surface_elevated, false);
                }
            }
        }

        // Help overlay
        if self.show_help {
            let help_w = 50usize;
            let help_h = 16usize;
            let help_x = ((area.width as usize).saturating_sub(help_w)) / 2;
            let help_y = ((area.height as usize).saturating_sub(help_h)) / 2;
            
            // Background
            for cy in 0..help_h {
                for cx in 0..help_w {
                    if help_x + cx < area.width as usize && help_y + cy < area.height as usize {
                        set_cell(&mut plane, help_x + cx, help_y + cy, ' ', t.fg, t.surface_elevated);
                    }
                }
            }
            
            // Border
            draw_rounded_border(&mut plane, Rect::new(help_x as u16, help_y as u16, help_w as u16, help_h as u16), t.primary, t.surface_elevated, true);
            
            // Title
            let title = " Keyboard Shortcuts ";
            let title_x = help_x + (help_w - title.len()) / 2;
            draw_text(&mut plane, title_x, help_y + 1, title, t.primary, t.surface_elevated, true);
            
            // Content
            let lines = [
                ("↑↓←→", "Navigate cards"),
                ("Enter", "Launch selected"),
                ("Space", "Show details"),
                ("/", "Focus search"),
                ("Tab", "Cycle categories"),
                ("t", "Cycle theme"),
                ("d", "Debug overlay"),
                ("q / Esc", "Quit"),
                ("?", "Toggle this help"),
                ("", ""),
                ("Mouse", ""),
                ("Click", "Select card"),
                ("Double-click", "Launch example"),
                ("Right-click", "Context menu"),
                ("Scroll", "Navigate grid"),
            ];
            for (i, (key_text, desc)) in lines.iter().enumerate() {
                let y = help_y + 3 + i;
                if y < area.height as usize - 1 {
                    if !key_text.is_empty() {
                        draw_text(&mut plane, help_x + 3, y, key_text, t.primary, t.surface_elevated, false);
                        draw_text(&mut plane, help_x + 18, y, desc, t.fg, t.surface_elevated, false);
                    }
                }
            }
        }

        // Modal preview
        if self.modal_preview {
            if let Some(&ex_idx) = self.filtered.get(self.selected) {
                if let Some(ex) = self.examples.get(ex_idx) {
                    let modal_w = 50usize;
                    let modal_h = 12usize;
                    let modal_x = ((area.width as usize).saturating_sub(modal_w)) / 2;
                    let modal_y = ((area.height as usize).saturating_sub(modal_h)) / 2;
                    
                    // Background
                    for cy in 0..modal_h {
                        for cx in 0..modal_w {
                            if modal_x + cx < area.width as usize && modal_y + cy < area.height as usize {
                                set_cell(&mut plane, modal_x + cx, modal_y + cy, ' ', t.fg, t.surface_elevated);
                            }
                        }
                    }
                    
                    // Border
                    draw_rounded_border(&mut plane, Rect::new(modal_x as u16, modal_y as u16, modal_w as u16, modal_h as u16), t.primary, t.surface_elevated, true);
                    
                    // Title
                    let title = format!(" {} ", ex.name);
                    let title_x = modal_x + (modal_w - title.len()) / 2;
                    draw_text(&mut plane, title_x, modal_y + 1, &title, t.primary, t.surface_elevated, true);
                    
                    // Category badge
                    let badge = format!(" {} ", ex.category.to_uppercase());
                    draw_text(&mut plane, modal_x + 2, modal_y + 3, &badge, t.fg_on_accent, category_color(t, ex.category), false);
                    
                    // Description
                    let desc: String = ex.description.chars().take(modal_w - 4).collect();
                    draw_text(&mut plane, modal_x + 2, modal_y + 5, &desc, t.fg, t.surface_elevated, false);
                    
                    // Preview lines
                    for (i, line) in ex.preview.iter().enumerate() {
                        let py = modal_y + 7 + i;
                        if py < modal_y + modal_h - 2 && py < area.height as usize {
                            let preview_line: String = line.chars().take(modal_w - 4).collect();
                            draw_text(&mut plane, modal_x + 2, py, &preview_line, t.fg_subtle, t.surface_elevated, false);
                        }
                    }
                    
                    // Footer
                    draw_text(&mut plane, modal_x + 2, modal_y + modal_h - 2, "Press Space or Esc to close", t.fg_muted, t.surface_elevated, false);
                }
            }
        }

        // Debug overlay: show dirty region highlights
        if self.show_debug {
            let dbg_text = " DEBUG MODE [D] ";
            let dbg_w = dbg_text.len() + 4;
            let dbg_x = (area.width as usize).saturating_sub(dbg_w) / 2;
            let dbg_y = 2;

            for cx in 0..dbg_w {
                if dbg_x + cx < area.width as usize {
                    set_cell(&mut plane, dbg_x + cx, dbg_y, ' ', t.bg, t.error);
                }
            }
            for cx in 0..dbg_w {
                if dbg_x + cx < area.width as usize {
                    set_cell(&mut plane, dbg_x + cx, dbg_y, '─', t.bg, t.error);
                }
            }
            draw_text(&mut plane, dbg_x + 2, dbg_y, dbg_text, t.bg, t.error, true);

            let dbg_info = format!(
                "FPS:{:>3} | Cards:{:>2} | Selected:{:>2} | Hover:{:>2?} | Search:{:>5}",
                self.fps.load(Ordering::Relaxed),
                self.filtered.len(),
                self.selected,
                self.hovered_card,
                if self.search_active { "active" } else { "idle" }
            );
            let dbg_info_y = dbg_y + 2;
            draw_text(&mut plane, 2, dbg_info_y, &dbg_info, t.error, t.bg, false);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        // Help overlay takes priority
        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => {
                    self.show_help = false;
                    return true;
                }
                _ => return true,
            }
        }

        // Context menu takes priority
        if self.context_menu.is_some() {
            let menu_len = 4;
            match key.code {
                KeyCode::Esc => {
                    self.context_menu = None;
                    return true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.context_menu_selected = self.context_menu_selected.saturating_sub(1);
                    return true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.context_menu_selected = (self.context_menu_selected + 1).min(menu_len - 1);
                    return true;
                }
                KeyCode::Enter => {
                    let selected = self.context_menu_selected;
                    self.context_menu = None;
                    if selected == 0 {
                        self.launch_selected();
                    } else if selected == 1 {
                        if let Some(ex) = self.selected_example() {
                            println!("{}", ex.binary_name);
                            self.status_message = Some((format!("Copied: {}", ex.binary_name), Instant::now()));
                        }
                    } else if selected == 2 {
                        let category = self.selected_example().map(|ex| ex.category);
                        if let Some(cat) = category {
                            self.category_filter = Some(cat);
                            self.apply_filter();
                            self.status_message = Some((format!("Filtered: {}", cat), Instant::now()));
                        }
                    }
                    return true;
                }
                _ => return true,
            }
        }

        // Modal preview takes priority
        if self.modal_preview {
            match key.code {
                KeyCode::Esc | KeyCode::Char(' ') => {
                    self.modal_preview = false;
                    return true;
                }
                _ => return true,
            }
        }

        // Search mode
        if self.search_active {
            match key.code {
                KeyCode::Esc => {
                    self.search_active = false;
                    true
                }
                KeyCode::Enter => {
                    self.search_active = false;
                    self.launch_selected();
                    true
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.apply_filter();
                    true
                }
                KeyCode::Char(ch) => {
                    self.search_query.push(ch);
                    self.apply_filter();
                    true
                }
                _ => false,
            }
        } else {
            match key.code {
                KeyCode::Char('q') => {
                    self.should_quit.store(true, Ordering::SeqCst);
                    true
                }
                KeyCode::Char('?') => {
                    self.show_help = true;
                    true
                }
                KeyCode::Char(' ') => {
                    self.modal_preview = true;
                    true
                }
                KeyCode::Char('t') => {
                    let themes = Self::themes();
                    let current = themes.iter().position(|(_, t)| t.name == self.theme.name).unwrap_or(0);
                    self.pending_theme = Some((current + 1) % themes.len());
                    self.apply_filter();
                    true
                }
                KeyCode::Char('d') => {
                    self.show_debug = !self.show_debug;
                    true
                }
                KeyCode::Char('/') => {
                    self.search_active = true;
                    true
                }
                KeyCode::Tab => {
                    let categories = [None, Some("apps"), Some("cookbook"), Some("tools")];
                    let current = categories.iter().position(|&c| c == self.category_filter).unwrap_or(0);
                    self.category_filter = categories[(current + 1) % categories.len()];
                    self.apply_filter();
                    true
                }
                KeyCode::Char('1') => {
                    self.primitive_toggle = !self.primitive_toggle;
                    true
                }
                KeyCode::Char('2') => {
                    self.primitive_slider = (self.primitive_slider + 0.1).min(1.0);
                    true
                }
                KeyCode::Char('3') => {
                    self.primitive_checkbox = !self.primitive_checkbox;
                    true
                }
                KeyCode::Char('4') => {
                    self.primitive_radio = (self.primitive_radio + 1) % 3;
                    true
                }
                KeyCode::Char('5') => {
                    self.primitive_button = true;
                    self.primitive_button_time = Some(Instant::now());
                    true
                }
                KeyCode::Down => {
                    if self.selected + 1 < self.filtered.len() {
                        self.selected += 1;
                    } else if !self.filtered.is_empty() {
                        self.selected = 0;
                    }
                    true
                }
                KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    } else if !self.filtered.is_empty() {
                        self.selected = self.filtered.len() - 1;
                    }
                    true
                }
                KeyCode::Right => {
                    let cols = self.cols.get().max(1);
                    if !self.filtered.is_empty() {
                        self.selected = (self.selected + cols) % self.filtered.len();
                    }
                    true
                }
                KeyCode::Left => {
                    let cols = self.cols.get().max(1);
                    if !self.filtered.is_empty() {
                        self.selected = (self.selected + self.filtered.len() - cols % self.filtered.len()) % self.filtered.len();
                    }
                    true
                }
                KeyCode::Enter => {
                    self.launch_selected();
                    true
                }
                _ => false,
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.mouse_pos = Some((col, row));
        let sidebar_w = 14usize;
        let sidebar_start_y = 6usize;
        let _grid_start_x = sidebar_w + 2;
        let _grid_start_y = sidebar_start_y + 1;
        let _card_w = 28usize;
        let _card_h = 14usize;

        const PRIM_BASE: usize = 100;
        const PALETTE_BASE: usize = 200;
        const CAT_BASE: usize = 300;
        const FPS_ZONE: usize = 400;
        const CARD_BASE: usize = 500;

        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let y = row as usize;
                let x = col as usize;

                // Zone-based dispatch — query the registry populated during render
                let clicked_zone = self.zones.borrow().dispatch(col, row);
                if let Some(zone_id) = clicked_zone {
                    match zone_id {
                        // Theme palette swatches (PALETTE_BASE + i)
                        id if id >= PALETTE_BASE && id < PALETTE_BASE + 20 => {
                            let idx = id - PALETTE_BASE;
                            self.pending_theme = Some(idx);
                            self.apply_filter();
                            return true;
                        }
                        // FPS toggle
                        FPS_ZONE => {
                            self.show_fps = !self.show_fps;
                            return true;
                        }
                        // Primitives bar controls (PRIM_BASE + i)
                        id if id >= PRIM_BASE && id < PRIM_BASE + 5 => {
                            match id - PRIM_BASE {
                                0 => { self.primitive_toggle = !self.primitive_toggle; return true; }
                                1 => { self.primitive_slider = (self.primitive_slider + 0.1).min(1.0); return true; }
                                2 => { self.primitive_checkbox = !self.primitive_checkbox; return true; }
                                3 => { self.primitive_radio = (self.primitive_radio + 1) % 3; return true; }
                                4 => { self.primitive_button = true; self.primitive_button_time = Some(Instant::now()); return true; }
                                _ => {}
                            }
                        }
                        // Sidebar categories (CAT_BASE + i)
                        id if id >= CAT_BASE && id < CAT_BASE + 4 => {
                            let cats: [Option<&str>; 4] = [None, Some("apps"), Some("cookbook"), Some("tools")];
                            self.category_filter = cats[id - CAT_BASE];
                            self.apply_filter();
                            return true;
                        }
                        // Cards (CARD_BASE + grid_idx)
                        id if id >= CARD_BASE => {
                            let card_idx = id - CARD_BASE;
                            if card_idx < self.filtered.len() {
                                let now = Instant::now();
                                let is_double_click = self.last_click_time
                                    .zip(self.last_click_idx)
                                    .map(|(time, idx)| {
                                        idx == card_idx && now.duration_since(time).as_millis() < 300
                                    })
                                    .unwrap_or(false);
                                if is_double_click {
                                    self.selected = card_idx;
                                    self.launch_selected();
                                } else {
                                    self.selected = card_idx;
                                }
                                self.last_click_time = Some(now);
                                self.last_click_idx = Some(card_idx);
                                return true;
                            }
                        }
                        _ => {}
                    }
                }

                // Search bar click (no zone registered for this)
                if y == 3 && x >= 2 && x < 30 {
                    self.search_active = true;
                    return true;
                }

                false
            }
            MouseEventKind::Down(MouseButton::Right) => {
                // Use zone dispatch for right-click on cards
                let clicked_zone = self.zones.borrow().dispatch(col, row);
                if let Some(zone_id) = clicked_zone {
                    if zone_id >= CARD_BASE {
                        let card_idx = zone_id - CARD_BASE;
                        if card_idx < self.filtered.len() {
                            self.selected = card_idx;
                            self.context_menu = Some((card_idx, col, row));
                            self.context_menu_selected = 0;
                            return true;
                        }
                    }
                }
                self.context_menu = None;
                false
            }
            MouseEventKind::ScrollDown => {
                if self.selected + 1 < self.filtered.len() {
                    self.selected += 1;
                    true
                } else {
                    false
                }
            }
            MouseEventKind::ScrollUp => {
                if self.selected > 0 {
                    self.selected -= 1;
                    true
                } else {
                    false
                }
            }
            MouseEventKind::Moved => {
                if let Some(btn_time) = self.primitive_button_time {
                    if btn_time.elapsed() >= Duration::from_secs(1) {
                        self.primitive_button = false;
                        self.primitive_button_time = None;
                    }
                }
                // Use zone dispatch for hover detection
                let hovered_zone = self.zones.borrow().dispatch(col, row);
                if let Some(zone_id) = hovered_zone {
                    if zone_id >= CARD_BASE {
                        let card_idx = zone_id - CARD_BASE;
                        if card_idx < self.filtered.len() {
                            self.hovered_card = Some(card_idx);
                            // Start or update tooltip timer
                            match self.tooltip_timer {
                                None => {
                                    self.tooltip_timer = Some(Instant::now());
                                    self.tooltip_pos = Some((col, row));
                                }
                                Some(time) => {
                                    if time.elapsed().as_millis() >= 500 {
                                        if let Some(&ex_idx) = self.filtered.get(card_idx) {
                                            if let Some(ex) = self.examples.get(ex_idx) {
                                                self.tooltip_text = Some(ex.description.to_string());
                                                self.tooltip_pos = Some((col, row));
                                            }
                                        }
                                    }
                                }
                            }
                            return true;
                        }
                    }
                }
                self.hovered_card = None;
                self.tooltip_text = None;
                self.tooltip_timer = None;
                self.tooltip_pos = None;
                false
            }
            _ => false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("Dracon Terminal Engine — Example Showcase");
    println!("Grid launcher with search, categories, and live previews");
    std::thread::sleep(Duration::from_millis(500));

    let pending = Arc::new(Mutex::new(None));
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);
    let fps_counter = Arc::new(AtomicU64::new(0));
    let fps_for_tick = Arc::clone(&fps_counter);

    let showcase = Showcase::new(should_quit, pending.clone(), fps_counter);

    let mut app = App::new()?.title("Dracon Showcase").fps(30).theme(Theme::nord());
    let _showcase_id = app.add_widget(Box::new(showcase), Rect::new(0, 0, 80, 24));

    app.on_tick(move |ctx, _tick| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
            return;
        }

        // Compute and store FPS
        fps_for_tick.store(ctx.fps(), Ordering::Relaxed);

        // Handle pending binary launch
        if let Some(binary_name) = pending.lock().unwrap().take() {
            let exe_dir = match std::env::current_exe() {
                Ok(p) => p.parent().unwrap().to_path_buf(),
                Err(_) => return,
            };
            let binary_path = exe_dir.join(&binary_name);

            let _ = ctx.suspend_terminal();

            // Auto-build if missing
            if !binary_path.exists() {
                let find_crate_root = || -> Option<std::path::PathBuf> {
                    let mut dir = exe_dir.clone();
                    loop {
                        if dir.join("Cargo.toml").exists() {
                            return Some(dir);
                        }
                        if !dir.pop() {
                            return None;
                        }
                    }
                };

                if let Some(crate_root) = find_crate_root() {
                    let _ = std::process::Command::new("cargo")
                        .args(["build", "--example", &binary_name])
                        .current_dir(&crate_root)
                        .status();
                }
            }

            let _ = std::process::Command::new(&binary_path)
                .current_dir(&exe_dir)
                .status();

            let mut drain_buf = [0u8; 256];
            let _ = std::io::stdin().read(&mut drain_buf);

            let _ = ctx.resume_terminal();
            ctx.mark_all_dirty();
        }
    }).run(|_ctx| {
        // Render loop handled by framework
    })
}

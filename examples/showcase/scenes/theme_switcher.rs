//! Embedded Theme Switcher scene for the showcase.
//!
//! Displays a grid of theme swatches and a widget preview.
//! Press `t` to cycle, `B`/`Esc` to go back.

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Button, Checkbox, Gauge, StatusBadge};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

const THEMES: &[(&str, fn() -> Theme)] = &[
    ("Dark", Theme::dark),
    ("Light", Theme::light),
    ("Dracula", Theme::dracula),
    ("Nord", Theme::nord),
    ("Cyberpunk", Theme::cyberpunk),
    ("Monokai", Theme::monokai),
    ("Gruvbox", Theme::gruvbox_dark),
    ("Tokyo Night", Theme::tokyo_night),
    ("Catppuccin", Theme::catppuccin_mocha),
    ("Solarized", Theme::solarized_dark),
    ("One Dark", Theme::one_dark),
    ("Rose Pine", Theme::rose_pine),
    ("Kanagawa", Theme::kanagawa),
    ("Everforest", Theme::everforest),
    ("Forest", Theme::forest),
    ("Sunset", Theme::sunset),
    ("Warm", Theme::warm),
    ("Cool", Theme::cool),
    ("Mono", Theme::mono),
    ("High Contrast", Theme::high_contrast),
];

pub struct ThemeSwitcherScene {
    theme_index: usize,
    theme: Theme,
    show_help: bool,
    checkbox: Checkbox,
    button: Button,
    gauge: Gauge,
    badge: StatusBadge,
    area: std::cell::Cell<Rect>,
}

impl ThemeSwitcherScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme_index: 0,
            theme,
            show_help: false,
            checkbox: Checkbox::new(WidgetId::new(10), "Preview"),
            button: Button::with_id(WidgetId::new(11), "Action"),
            gauge: {
                let mut g = Gauge::with_id(WidgetId::new(12), "Load")
                    .warn_threshold(70.0)
                    .crit_threshold(90.0);
                g.set_value(65.0);
                g
            },
            badge: StatusBadge::new(WidgetId::new(13)).with_status("OK"),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % THEMES.len();
        self.theme = THEMES[self.theme_index].1();
        self.checkbox.on_theme_change(&self.theme);
        self.button.on_theme_change(&self.theme);
        self.gauge.on_theme_change(&self.theme);
        self.badge.on_theme_change(&self.theme);
    }

    fn render_swatch(&self, plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, theme: &Theme, selected: bool) {
        let border = if selected { self.theme.primary } else { self.theme.outline };
        for row in y..y + h {
            for col in x..x + w {
                let idx = (row * plane.width + col) as usize;
                if idx >= plane.cells.len() { continue; }
                plane.cells[idx].bg = theme.bg;
                let is_border = row == y || row == y + h - 1 || col == x || col == x + w - 1;
                if is_border {
                    plane.cells[idx].fg = border;
                    plane.cells[idx].char = match (row == y, row == y + h - 1, col == x, col == x + w - 1) {
                        (true, _, true, _) => '╭',
                        (true, _, _, true) => '╮',
                        (_, true, true, _) => '╰',
                        (_, true, _, true) => '╯',
                        _ => '─',
                    };
                } else if row == y + 1 && col >= x + 1 && col < x + w - 1 {
                    let name = theme.name;
                    let name_x = x + 1 + (w.saturating_sub(2).saturating_sub(name.len() as u16)) / 2;
                    if col >= name_x && (col - name_x) < name.len() as u16 {
                        let ch = name.chars().nth((col - name_x) as usize).unwrap_or(' ');
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = theme.fg;
                    }
                }
            }
        }
    }
}

impl Scene for ThemeSwitcherScene {
    fn scene_id(&self) -> &str { "theme_switcher" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Title
        let title = " Theme Switcher ";
        draw_text(&mut plane, 2, 0, title, t.primary, t.bg, true);
        let hint = format!(" {} / {} ", self.theme_index + 1, THEMES.len());
        draw_text(&mut plane, area.width.saturating_sub(hint.len() as u16 + 2), 0,
                  &hint, t.secondary, t.bg, false);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Theme grid (4 columns)
        let cols: u16 = 4;
        let _rows = (THEMES.len() as u16 + cols - 1) / cols;
        let swatch_w = (area.width.saturating_sub(2)) / cols;
        let swatch_h = 3u16;
        let start_y = 2u16;

        for (i, (_name, factory)) in THEMES.iter().enumerate() {
            let col = i as u16 % cols;
            let row = i as u16 / cols;
            let x = 1 + col * swatch_w;
            let y = start_y + row * swatch_h;
            if y + swatch_h >= area.height.saturating_sub(8) { break; }

            let theme = factory();
            let selected = i == self.theme_index;
            self.render_swatch(&mut plane, x, y, swatch_w, swatch_h, &theme, selected);

            if selected {
                // Draw indicator arrow
                let arrow_x = x + swatch_w / 2;
                let arrow_y = y + swatch_h;
                if arrow_y < area.height {
                    let idx = (arrow_y * area.width + arrow_x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '▲';
                        plane.cells[idx].fg = t.primary;
                    }
                }
            }
        }

        // Preview section
        let preview_y = area.height.saturating_sub(7);
        for x in 0..area.width {
            let idx = (preview_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
        draw_text(&mut plane, 2, preview_y + 1, "Preview:", t.primary, t.bg, true);

        let preview_w = area.width.saturating_sub(4) / 4;
        let px = 2u16;

        // Render preview widgets
        let mut cb_plane = self.checkbox.render(Rect::new(px, preview_y + 2, preview_w, 1));
        blit_to(&mut plane, &mut cb_plane, px as usize, (preview_y + 2) as usize);

        let mut btn_plane = self.button.render(Rect::new(px + preview_w + 1, preview_y + 2, preview_w, 1));
        blit_to(&mut plane, &mut btn_plane, (px + preview_w + 1) as usize, (preview_y + 2) as usize);

        let mut gauge_plane = self.gauge.render(Rect::new(px, preview_y + 4, preview_w * 2, 2));
        blit_to(&mut plane, &mut gauge_plane, px as usize, (preview_y + 4) as usize);

        let mut badge_plane = self.badge.render(Rect::new(px + preview_w * 2 + 2, preview_y + 2, preview_w, 1));
        blit_to(&mut plane, &mut badge_plane, (px + preview_w * 2 + 2) as usize, (preview_y + 2) as usize);

        // Footer
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
        let nav = " t: cycle theme | B: back | ?: help ";
        draw_text(&mut plane, 2, footer_y, nav, t.fg_muted, t.bg, false);

        if self.show_help {
            draw_help(&mut plane, area, t);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                self.show_help = false;
            }
            return true;
        }
        match key.code {
            KeyCode::Char('?') => { self.show_help = true; true }
            KeyCode::Char('t') | KeyCode::Char('T') => { self.cycle_theme(); true }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, _kind: dracon_terminal_engine::input::event::MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.checkbox.on_theme_change(theme);
        self.button.on_theme_change(theme);
        self.gauge.on_theme_change(theme);
        self.badge.on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
}

fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = (y * plane.width + x + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch, fg, bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false, skip: false,
            };
        }
    }
}

fn blit_to(dest: &mut Plane, src: &mut Plane, offset_x: usize, offset_y: usize) {
    for i in 0..src.cells.len() {
        let cell = &src.cells[i];
        if cell.char == '\0' || cell.transparent { continue; }
        let row = i / src.width as usize;
        let col = i % src.width as usize;
        let dy = offset_y + row;
        let dx = offset_x + col;
        if dy >= dest.height as usize || dx >= dest.width as usize { continue; }
        let idx = dy * dest.width as usize + dx;
        if idx < dest.cells.len() {
            dest.cells[idx] = cell.clone();
        }
    }
}

fn draw_help(plane: &mut Plane, area: Rect, t: Theme) {
    let hw = 40u16.min(area.width.saturating_sub(4));
    let hh = 10u16.min(area.height.saturating_sub(4));
    let hx = (area.width - hw) / 2;
    let hy = (area.height - hh) / 2;

    for y in hy..hy + hh {
        for x in hx..hx + hw {
            let idx = (y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface_elevated;
                plane.cells[idx].transparent = false;
            }
        }
    }

    for x in hx..hx + hw {
        let top = (hy * area.width + x) as usize;
        let bot = ((hy + hh - 1) * area.width + x) as usize;
        if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
        if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
    }
    for y in hy..hy + hh {
        let left = (y * area.width + hx) as usize;
        let right = (y * area.width + hx + hw - 1) as usize;
        if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
        if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
    }

    let title = "Theme Switcher Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

    let shortcuts = [
        ("t", "Cycle theme"),
        ("↑↓←→", "Navigate"),
        ("B/Esc", "Back to showcase"),
        ("?", "Toggle help"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = hy + 3 + i as u16;
        draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
        draw_text(plane, hx + 14, row, desc, t.fg, t.surface_elevated, false);
    }
}

//! Embedded Theme Switcher scene for the showcase.
//!
//! Displays a grid of theme swatches and a widget preview.
//! Press `t` to cycle, `B`/`Esc` to go back.

use crate::scenes::shared_helpers::{blit_to, draw_text};
use dracon_terminal_engine::compositor::{Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Button, Checkbox, Gauge, StatusBadge};
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

/// Configuration for rendering a single theme swatch.
struct SwatchConfig<'a> {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    theme: &'a Theme,
    selected: bool,
}

pub struct ThemeSwitcherScene {
    theme_index: usize,
    theme: Theme,
    show_help: bool,
    dirty: bool,
    checkbox: Checkbox,
    button: Button,
    gauge: Gauge,
    badge: StatusBadge,
    area: std::cell::Cell<Rect>,
    keybindings: KeybindingSet,
}

impl ThemeSwitcherScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme_index: 0,
            theme,
            show_help: false,
            dirty: true,
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
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn render_swatch(&self, plane: &mut Plane, cfg: SwatchConfig) {
        let SwatchConfig { x, y, w, h, theme, selected } = cfg;
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
                } else if row == y + 1 && col > x && col < x + w - 1 {
                    let name = &theme.name;
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
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Title
        let title = " Theme Switcher ";
        draw_text(&mut plane, 2, 0, title, t.primary, t.bg, true);
        let hint = format!(" {} / {} ", self.theme_index + 1, Theme::all().len());
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
        let _rows = (Theme::all().len() as u16).div_ceil(cols);
        let swatch_w = (area.width.saturating_sub(2)) / cols;
        let swatch_h = 3u16;
        let start_y = 2u16;

        for (i, theme) in Theme::all().iter().enumerate() {
            let col = i as u16 % cols;
            let row = i as u16 / cols;
            let x = 1 + col * swatch_w;
            let y = start_y + row * swatch_h;
            if y + swatch_h >= area.height.saturating_sub(8) { break; }

            // theme is already &Theme from iterator
            let selected = i == self.theme_index;
            self.render_swatch(&mut plane, SwatchConfig { x, y, w: swatch_w, h: swatch_h, theme, selected });

            if selected {
                // Draw indicator arrow
                let arrow_x = x + swatch_w / 2;
                let arrow_y = y + swatch_h;
                if arrow_y < area.height {
                    let idx = (arrow_y * area.width + arrow_x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '^';
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
        let cb_plane = self.checkbox.render(Rect::new(px, preview_y + 2, preview_w, 1));
        blit_to(&mut plane, &cb_plane, px as usize, (preview_y + 2) as usize);

        let btn_plane = self.button.render(Rect::new(px + preview_w + 1, preview_y + 2, preview_w, 1));
        blit_to(&mut plane, &btn_plane, (px + preview_w + 1) as usize, (preview_y + 2) as usize);

        let gauge_plane = self.gauge.render(Rect::new(px, preview_y + 4, preview_w * 2, 2));
        blit_to(&mut plane, &gauge_plane, px as usize, (preview_y + 4) as usize);

        let badge_plane = self.badge.render(Rect::new(px + preview_w * 2 + 2, preview_y + 2, preview_w, 1));
        blit_to(&mut plane, &badge_plane, (px + preview_w * 2 + 2) as usize, (preview_y + 2) as usize);

        // Footer
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
        let nav = " B/Esc: back | ?: help ";
        draw_text(&mut plane, 2, footer_y, nav, t.fg_muted, t.bg, false);

        if self.show_help {
            draw_help(&mut plane, area, t);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }
        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.show_help {
            return true;
        }
        if let MouseEventKind::Down(MouseButton::Left) = kind {
            let area = self.area.get();
            let cols: u16 = 4;
            let swatch_w = (area.width.saturating_sub(2)) / cols;
            let swatch_h = 3u16;
            let start_y = 2u16;
            for (i, theme) in Theme::all().iter().enumerate() {
                let col_idx = i as u16 % cols;
                let row_idx = i as u16 / cols;
                let x = 1 + col_idx * swatch_w;
                let y = start_y + row_idx * swatch_h;
                if y + swatch_h >= area.height.saturating_sub(8) {
                    break;
                }
                if col >= x && col < x + swatch_w && row >= y && row < y + swatch_h {
                    self.theme_index = i;
                    self.theme = theme.clone();
                    self.checkbox.on_theme_change(theme);
                    self.button.on_theme_change(theme);
                    self.gauge.on_theme_change(theme);
                    self.badge.on_theme_change(theme);
                    self.dirty = true;
                    return true;
                }
            }
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.checkbox.on_theme_change(theme);
        self.button.on_theme_change(theme);
        self.gauge.on_theme_change(theme);
        self.badge.on_theme_change(theme);
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

fn draw_help(plane: &mut Plane, area: Rect, t: &Theme) {
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

    for x in hx + 1..hx + hw - 1 {
        let top = (hy * area.width + x) as usize;
        let bot = ((hy + hh - 1) * area.width + x) as usize;
        if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
        if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
    }
    for y in hy + 1..hy + hh - 1 {
        let left = (y * area.width + hx) as usize;
        let right = (y * area.width + hx + hw - 1) as usize;
        if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
        if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
    }

    let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
    for (ch, cx, cy) in corners.iter() {
        let idx = (cy * area.width + cx) as usize;
        if idx < plane.cells.len() { plane.cells[idx].char = *ch; plane.cells[idx].fg = t.outline; }
    }

    let label = "Theme Switcher";
    let lx = hx + (hw - label.len() as u16) / 2;
    for (i, c) in label.chars().enumerate() {
        let idx = ((hy + 1) * area.width + lx + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = t.primary;
            plane.cells[idx].style = Styles::BOLD;
        }
    }

    let shortcuts = [
        ("↑/↓/←/→", "Navigate"),
        ("Enter", "Apply theme"),
        ("B", "Go back"),
        ("T", "Cycle showcase theme"),
        ("F1", "Toggle help"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = hy + 3 + i as u16;
        for (j, c) in key.chars().enumerate() {
            let idx = (row * area.width + hx + 2 + j as u16) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; }
        }
        for (j, c) in desc.chars().enumerate() {
            let idx = (row * area.width + hx + 14 + j as u16) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.fg; }
        }
    }
}


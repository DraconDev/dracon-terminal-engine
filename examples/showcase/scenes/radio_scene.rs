//! Embedded Radio Button scene for the showcase.
//!
//! Demonstrates the Radio widget with grouped radio options
//! and a live preview panel showing current settings.

use crate::scenes::shared_helpers::{draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::{Color, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::radio::Radio;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

struct RadioGroupConfig<'a> {
    x: u16,
    y: u16,
    label: &'a str,
    options: &'a [Radio],
    is_focused: bool,
}

pub struct RadioScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    theme_options: Vec<Radio>,
    theme_selected: usize,
    font_options: Vec<Radio>,
    font_selected: usize,
    layout_options: Vec<Radio>,
    layout_selected: usize,
    focused_group: usize, // 0=theme, 1=font, 2=layout
    dirty: bool,
    area: std::cell::Cell<Rect>,
}

impl RadioScene {
    pub fn new(theme: Theme) -> Self {
        let theme_options = vec![
            Radio::new(WidgetId::new(1), "Dark").with_theme(theme.clone()),
            Radio::new(WidgetId::new(2), "Light").with_theme(theme.clone()),
            Radio::new(WidgetId::new(3), "High Contrast").with_theme(theme.clone()),
        ];

        let font_options = vec![
            Radio::new(WidgetId::new(4), "Small (12pt)").with_theme(theme.clone()),
            Radio::new(WidgetId::new(5), "Medium (14pt)").with_theme(theme.clone()),
            Radio::new(WidgetId::new(6), "Large (18pt)").with_theme(theme.clone()),
        ];

        let layout_options = vec![
            Radio::new(WidgetId::new(7), "Compact").with_theme(theme.clone()),
            Radio::new(WidgetId::new(8), "Comfortable").with_theme(theme.clone()),
            Radio::new(WidgetId::new(9), "Spacious").with_theme(theme.clone()),
        ];

        let mut scene = Self {
            theme,
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            theme_options,
            theme_selected: 0,
            font_options,
            font_selected: 1,
            layout_options,
            layout_selected: 1,
            focused_group: 0,
            dirty: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
        };
        scene.theme_options[0].select();
        scene.font_options[1].select();
        scene.layout_options[1].select();
        scene
    }

    fn render_radio_group(&self, plane: &mut Plane, cfg: &RadioGroupConfig<'_>) {
        let t = &self.theme;
        let x = cfg.x;
        let y = cfg.y;
        let fg = if cfg.is_focused { t.primary } else { t.fg };
        let style = if cfg.is_focused {
            Styles::BOLD
        } else {
            Styles::empty()
        };

        // Group label
        for (j, ch) in cfg.label.chars().enumerate() {
            let idx = (y * plane.width + x + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = fg;
                plane.cells[idx].style = style;
                plane.cells[idx].transparent = false;
            }
        }

        // Radio options - use widget render
        for (i, radio) in cfg.options.iter().enumerate() {
            let oy = y + 1 + i as u16;
            let radio_area = Rect::new(x, oy, 20, 1);
            let radio_plane = radio.render(radio_area);
            // Blit the radio plane into the main plane
            for cell_idx in 0..radio_plane.cells.len().min(20) {
                let main_x = x + cell_idx as u16;
                let main_y = oy;
                if main_x < plane.width && main_y < plane.height {
                    let idx = (main_y * plane.width + main_x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = radio_plane.cells[cell_idx];
                    }
                }
            }
        }
    }

    fn render_preview(&self, plane: &mut Plane, x: u16, y: u16, w: u16) {
        let t = &self.theme;
        let max_x = x + w;

        // Preview box title
        draw_text_clipped(plane, x, y, "Preview", max_x, t.primary, t.bg, true);

        // Border
        for bx in x..x + w {
            if bx >= max_x {
                break;
            }
            let top = ((y + 1) * plane.width + bx) as usize;
            if top < plane.cells.len() {
                plane.cells[top].char = '─';
                plane.cells[top].fg = t.outline;
            }
        }

        // Preview content
        let theme_name = match self.theme_selected {
            0 => "Dark",
            1 => "Light",
            2 => "High Contrast",
            _ => "Unknown",
        };
        let font_name = match self.font_selected {
            0 => "Small (12pt)",
            1 => "Medium (14pt)",
            2 => "Large (18pt)",
            _ => "Unknown",
        };
        let layout_name = match self.layout_selected {
            0 => "Compact",
            1 => "Comfortable",
            2 => "Spacious",
            _ => "Unknown",
        };

        let settings = [
            ("Theme:", theme_name),
            ("Font:", font_name),
            ("Layout:", layout_name),
        ];

        for (i, (key, val)) in settings.iter().enumerate() {
            let sy = y + 2 + i as u16;
            draw_text_clipped(plane, x, sy, key, max_x, t.fg_muted, t.bg, false);
            draw_text_clipped(plane, x + 8, sy, val, max_x, t.primary, t.bg, true);
        }

        // Simulated preview panel
        let preview_y = y + 6;
        draw_text_clipped(
            plane,
            x,
            preview_y,
            "Simulated UI:",
            max_x,
            t.fg_muted,
            t.bg,
            false,
        );

        let preview_bg = match self.theme_selected {
            1 => Color::Rgb(240, 240, 240),
            2 => Color::Rgb(0, 0, 0),
            _ => t.surface,
        };
        let preview_fg = match self.theme_selected {
            1 => Color::Rgb(30, 30, 30),
            2 => Color::Rgb(255, 255, 0),
            _ => t.fg,
        };

        let pw = w.saturating_sub(2);
        let ph = 3u16;
        for dy in 0..ph {
            for dx in 0..pw {
                let idx = ((preview_y + 1 + dy) * plane.width + x + 1 + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = preview_bg;
                    plane.cells[idx].fg = preview_fg;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        let sample_text = match self.layout_selected {
            0 => "Aa Bb Cc",
            1 => "Aa  Bb  Cc",
            _ => "Aa   Bb   Cc",
        };
        draw_text(
            plane,
            x + 2,
            preview_y + 2,
            sample_text,
            preview_fg,
            preview_bg,
            self.font_selected == 2,
        );
    }

    fn select_in_group(&mut self, group: usize, idx: usize) {
        match group {
            0 => {
                self.theme_options[self.theme_selected].deselect();
                self.theme_selected = idx;
                self.theme_options[idx].select();
            }
            1 => {
                self.font_options[self.font_selected].deselect();
                self.font_selected = idx;
                self.font_options[idx].select();
            }
            2 => {
                self.layout_options[self.layout_selected].deselect();
                self.layout_selected = idx;
                self.layout_options[idx].select();
            }
            _ => {}
        }
        self.dirty = true;
    }

    fn current_group_len(&self) -> usize {
        match self.focused_group {
            0 => self.theme_options.len(),
            1 => self.font_options.len(),
            2 => self.layout_options.len(),
            _ => 0,
        }
    }

    fn current_group_selected(&self) -> usize {
        match self.focused_group {
            0 => self.theme_selected,
            1 => self.font_selected,
            2 => self.layout_selected,
            _ => 0,
        }
    }
}

impl Scene for RadioScene {
    fn scene_id(&self) -> &str {
        "radio"
    }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        draw_text(&mut plane, 2, 0, " Radio Buttons ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(
            &mut plane,
            area.width.saturating_sub(theme_label.len() as u16 + 2),
            0,
            &theme_label,
            t.secondary,
            t.bg,
            false,
        );

        // Divider
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Description
        draw_text(
            &mut plane,
            2,
            2,
            "Settings panel with radio button groups",
            t.fg,
            t.bg,
            false,
        );
        draw_text(
            &mut plane,
            2,
            3,
            "Up/Down to select, Tab to switch groups",
            t.fg_muted,
            t.bg,
            false,
        );

        // Left column: radio groups
        let col1_x = 4u16;
        self.render_radio_group(
            &mut plane,
            &RadioGroupConfig {
                x: col1_x,
                y: 5,
                label: "Color Theme",
                options: &self.theme_options,
                is_focused: self.focused_group == 0,
            },
        );
        self.render_radio_group(
            &mut plane,
            &RadioGroupConfig {
                x: col1_x,
                y: 10,
                label: "Font Size",
                options: &self.font_options,
                is_focused: self.focused_group == 1,
            },
        );
        self.render_radio_group(
            &mut plane,
            &RadioGroupConfig {
                x: col1_x,
                y: 15,
                label: "Layout Density",
                options: &self.layout_options,
                is_focused: self.focused_group == 2,
            },
        );

        // Focus indicator for active group
        let group_labels = ["Color Theme", "Font Size", "Layout Density"];
        if let Some(label) = group_labels.get(self.focused_group) {
            draw_text(
                &mut plane,
                col1_x + label.len() as u16 + 1,
                5 + self.focused_group as u16 * 5,
                "◄",
                t.primary,
                t.bg,
                true,
            );
        }

        // Vertical divider
        let div_x = area.width / 2;
        for y in 2..area.height.saturating_sub(1) {
            let idx = (y * area.width + div_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Right column: preview
        let preview_x = div_x + 2;
        let preview_w = area.width.saturating_sub(div_x + 3);
        self.render_preview(&mut plane, preview_x, 5, preview_w);

        // Footer
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " Up/Down:select | Tab:switch group | {}:help | {}:back ",
            help_key, back_key,
        );
        let fy = area.height.saturating_sub(1);
        for (i, c) in footer.chars().enumerate() {
            let idx = (fy * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        if self.show_help {
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(
                &mut plane,
                area,
                &self.theme,
                "Radio Buttons — Help",
                &[
                    ("Up/Down", "Change selection in group"),
                    ("Tab", "Switch between groups"),
                    ("1/2/3", "Quick-select option in group"),
                    ("Click", "Select option directly"),
                    (back_key, "Back"),
                ],
            );
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
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

        match key.code {
            KeyCode::Tab => {
                self.focused_group = (self.focused_group + 1) % 3;
                self.dirty = true;
                true
            }
            KeyCode::BackTab => {
                self.focused_group = if self.focused_group == 0 {
                    2
                } else {
                    self.focused_group - 1
                };
                self.dirty = true;
                true
            }
            KeyCode::Up => {
                let current = self.current_group_selected();
                if current > 0 {
                    self.select_in_group(self.focused_group, current - 1);
                }
                true
            }
            KeyCode::Down => {
                let current = self.current_group_selected();
                if current + 1 < self.current_group_len() {
                    self.select_in_group(self.focused_group, current + 1);
                }
                true
            }
            KeyCode::Char('1') if key.modifiers.is_empty() => {
                self.select_in_group(self.focused_group, 0);
                true
            }
            KeyCode::Char('2') if key.modifiers.is_empty() => {
                if self.current_group_len() > 1 {
                    self.select_in_group(self.focused_group, 1);
                }
                true
            }
            KeyCode::Char('3') if key.modifiers.is_empty() => {
                if self.current_group_len() > 2 {
                    self.select_in_group(self.focused_group, 2);
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if matches!(
            kind,
            MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left)
        ) {
            let col1_x = 4u16;
            if col >= col1_x && col < col1_x + 20 {
                // Theme group (rows 6-8)
                if (6..=8).contains(&row) {
                    let idx = (row - 6) as usize;
                    if idx < self.theme_options.len() {
                        self.focused_group = 0;
                        self.select_in_group(0, idx);
                        return true;
                    }
                }
                // Font group (rows 11-13)
                if (11..=13).contains(&row) {
                    let idx = (row - 11) as usize;
                    if idx < self.font_options.len() {
                        self.focused_group = 1;
                        self.select_in_group(1, idx);
                        return true;
                    }
                }
                // Layout group (rows 16-18)
                if (16..=18).contains(&row) {
                    let idx = (row - 16) as usize;
                    if idx < self.layout_options.len() {
                        self.focused_group = 2;
                        self.select_in_group(2, idx);
                        return true;
                    }
                }
            }
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        for radio in self.theme_options.iter_mut() {
            radio.on_theme_change(theme);
        }
        for radio in self.font_options.iter_mut() {
            radio.on_theme_change(theme);
        }
        for radio in self.layout_options.iter_mut() {
            radio.on_theme_change(theme);
        }
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

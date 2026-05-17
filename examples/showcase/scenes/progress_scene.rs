//! Embedded Progress/Loading scene for the showcase.
//!
//! Demonstrates ProgressRing, ProgressBar, and Spinner widgets
//! in a simulated loading scenario.

use crate::scenes::shared_helpers::{blit_to, draw_text};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::progress_bar::ProgressBar;
use dracon_terminal_engine::framework::widgets::progress_ring::ProgressRing;
use dracon_terminal_engine::framework::widgets::spinner::Spinner;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::{Cell, RefCell};


pub struct ProgressScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    ring: std::cell::RefCell<ProgressRing>,
    bar: std::cell::RefCell<ProgressBar>,
    spinner: std::cell::RefCell<Spinner>,
    loading: Cell<bool>,
    progress: Cell<f64>,
    dirty: bool,
    area: Cell<Rect>,
}

impl ProgressScene {
    pub fn new(theme: Theme) -> Self {
        let ring = RefCell::new(ProgressRing::new(0.0)
            .with_theme(theme.clone())
            .with_size(8)
            .show_percentage(true)
            .with_label("Loading"));

        let bar = RefCell::new(ProgressBar::new(WidgetId::new(1))
            .with_theme(theme.clone()));
        let spinner = RefCell::new(Spinner::new(WidgetId::new(2)).with_theme(theme.clone()));

        Self {
            theme,
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            ring,
            bar,
            spinner,
            loading: Cell::new(false),
            progress: Cell::new(0.0),
            dirty: true,
            area: Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn start_loading(&self) {
        self.loading.set(true);
        self.progress.set(0.0);
    }

    fn stop_loading(&self) {
        self.loading.set(false);
    }

    fn update_progress(&self) {
        if self.loading.get() {
            let current = self.progress.get();
            let speed = match current {
                p if p < 30.0 => 2.0,
                p if p < 70.0 => 1.5,
                p if p < 90.0 => 0.8,
                _ => 0.3,
            };
            let next = (current + speed).min(100.0);
            self.progress.set(next);
            if next >= 100.0 {
                self.loading.set(false);
            }
        }
    }
}

impl Scene for ProgressScene {
    fn scene_id(&self) -> &str { "progress" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        self.update_progress();

        let progress = self.progress.get();
        let is_loading = self.loading.get();

        // Update widget states
        self.ring.borrow_mut().set_progress(progress / 100.0);
        self.bar.borrow_mut().set_progress(progress as f32 / 100.0);

        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        draw_text(&mut plane, 2, 0, " Progress & Loading ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 2), 0,
                  &theme_label, t.secondary, t.bg, false);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Description
        draw_text(&mut plane, 2, 2, "Simulated loading with progress indicators", t.fg, t.bg, false);

        // ── ProgressRing ──────────────────────────────────────────────────
        draw_text(&mut plane, 2, 4, "ProgressRing", t.fg, t.bg, true);
        let ring_area = Rect::new(2, 5, 12, 4);
        let ring_plane = self.ring.borrow().render(ring_area);
        blit_to(&mut plane, &ring_plane, ring_area.x as usize, ring_area.y as usize);

        // Status text next to ring
        let (status_text, status_color) = if progress >= 100.0 {
            ("Complete!", t.success)
        } else if is_loading {
            ("Loading...", t.primary)
        } else {
            ("Ready", t.fg_muted)
        };
        draw_text(&mut plane, 16, 6, status_text, status_color, t.bg, true);

        // ── ProgressBar ───────────────────────────────────────────────────
        draw_text(&mut plane, 2, 10, "ProgressBar", t.fg, t.bg, true);
        let bar_area = Rect::new(2, 11, area.width.saturating_sub(4), 3);
        let bar_plane = self.bar.borrow().render(bar_area);
        blit_to(&mut plane, &bar_plane, bar_area.x as usize, bar_area.y as usize);

        // ── Spinner ──────────────────────────────────────────────────────
        draw_text(&mut plane, 2, 15, "Spinner", t.fg, t.bg, true);
        if is_loading {
            let spinner_area = Rect::new(12, 15, 8, 1);
            let spinner_plane = self.spinner.borrow().render(spinner_area);
            blit_to(&mut plane, &spinner_plane, spinner_area.x as usize, spinner_area.y as usize);
        } else {
            draw_text(&mut plane, 12, 15, "(idle)", t.fg_muted, t.bg, false);
        }

        // ── Progress Stats ───────────────────────────────────────────────
        let stats_y = 17;
        draw_text(&mut plane, 2, stats_y, "Stats:", t.primary, t.bg, true);
        let progress_str = format!("Progress: {:.1}%", progress);
        let status_str = format!("Status: {}", if progress >= 100.0 { "Complete" } else if is_loading { "Loading" } else { "Ready" });
        draw_text(&mut plane, 2, stats_y + 1, &progress_str, t.fg, t.bg, false);
        draw_text(&mut plane, 2, stats_y + 2, &status_str, t.fg, t.bg, false);

        // ── Visual Gauge ────────────────────────────────────────────────
        let gauge_y = stats_y + 4;
        let gauge_w = area.width.saturating_sub(4) as usize;
        let filled = (progress / 100.0 * gauge_w as f64) as usize;
        for x in 0..gauge_w {
            let idx = (gauge_y * area.width + 2 + x as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = if x < filled { '█' } else { '░' };
                plane.cells[idx].fg = if progress >= 100.0 { t.success } else { t.primary };
                plane.cells[idx].transparent = false;
            }
        }

        // Auto-run indicator
        if is_loading {
            draw_text(&mut plane, area.width.saturating_sub(8), 2, "▶ AUTO", t.primary, t.bg, true);
        }

        // Footer
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " SPACE:toggle loading | r:reset | {}:help | {}:back ",
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
            self.render_help(&mut plane, area);
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

        match key.code {
            KeyCode::Char(' ') if key.modifiers.is_empty() => {
                if self.loading.get() {
                    self.stop_loading();
                } else {
                    self.start_loading();
                }
                self.dirty = true;
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.progress.set(0.0);
                self.loading.set(false);
                self.ring.borrow_mut().set_progress(0.0);
                self.bar.borrow_mut().set_progress(0.0);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.ring.borrow_mut().on_theme_change(theme);
        self.bar.borrow_mut().on_theme_change(theme);
        self.spinner.borrow_mut().on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { true } // always re-render for auto progress
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl ProgressScene {
    fn render_help(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 42u16.min(area.width.saturating_sub(4));
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

        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let lines = [
            ("╭────────────────────────────────────╮", true),
            ("│     Progress & Loading Help        │", true),
            ("├────────────────────────────────────┤", true),
            ("│  SPACE    Toggle loading simulation│", false),
            ("│  r        Reset progress           │", false),
            (&format!("│  {:<10} Back to showcase          │", back_key), false),
            ("╰────────────────────────────────────╯", true),
        ];
        for (i, (line, is_border)) in lines.iter().enumerate() {
            let ly = hy + i as u16;
            let lx = (area.width - line.len() as u16) / 2;
            for (j, ch) in line.chars().enumerate() {
                let px = lx + j as u16;
                if px < area.width && ly < area.height {
                    let idx = (ly * area.width + px) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = if *is_border || "│╭╮├┤╰╯─".contains(ch) { t.outline } else { t.fg };
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }
    }
}

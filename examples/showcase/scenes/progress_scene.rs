//! Embedded Progress/Loading scene for the showcase.
//!
//! Demonstrates ProgressRing, ProgressBar, and Spinner widgets
//! in a simulated multi-stage loading scenario with step dots,
//! elapsed time, and stage labels.

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
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

struct Stage {
    name: &'static str,
    icon: char,
    weight: f64, // relative duration weight
}

const STAGES: &[Stage] = &[
    Stage { name: "Initializing", icon: '◈', weight: 10.0 },
    Stage { name: "Loading Config", icon: '▤', weight: 15.0 },
    Stage { name: "Compiling", icon: '⚙', weight: 40.0 },
    Stage { name: "Linking", icon: '◆', weight: 15.0 },
    Stage { name: "Optimizing", icon: '◇', weight: 20.0 },
];

pub struct ProgressScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    ring: RefCell<ProgressRing>,
    bar: RefCell<ProgressBar>,
    spinner: RefCell<Spinner>,
    loading: Cell<bool>,
    progress: Cell<f64>,
    tick_count: Cell<u64>,
    start_tick: Cell<u64>,
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
            tick_count: Cell::new(0),
            start_tick: Cell::new(0),
            dirty: true,
            area: Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn start_loading(&self) {
        self.loading.set(true);
        self.progress.set(0.0);
        self.start_tick.set(self.tick_count.get());
    }

    fn stop_loading(&self) {
        self.loading.set(false);
    }

    fn update_progress(&self) {
        self.tick_count.set(self.tick_count.get() + 1);

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

    fn current_stage_index(&self) -> usize {
        let progress = self.progress.get();
        let total_weight: f64 = STAGES.iter().map(|s| s.weight).sum();
        let mut cumulative = 0.0;
        for (i, stage) in STAGES.iter().enumerate() {
            cumulative += stage.weight;
            if progress <= cumulative / total_weight * 100.0 {
                return i;
            }
        }
        STAGES.len() - 1
    }

    fn elapsed_ticks(&self) -> u64 {
        self.tick_count.get().saturating_sub(self.start_tick.get())
    }
}

impl Scene for ProgressScene {
    fn scene_id(&self) -> &str { "progress" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        self.update_progress();

        let progress = self.progress.get();
        let is_loading = self.loading.get();
        let stage_idx = self.current_stage_index();

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

        // Auto-run indicator
        if is_loading {
            draw_text(&mut plane, area.width.saturating_sub(8), 0, "▶ AUTO", t.primary, t.bg, true);
        }

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Description
        draw_text(&mut plane, 2, 2, "Multi-stage build simulation with progress indicators", t.fg, t.bg, false);

        // ── Step dots (y=3) ──────────────────────────────────────────────
        let dots_x = 2u16;
        for (i, stage) in STAGES.iter().enumerate() {
            let dx = dots_x + i as u16 * 14;
            let is_current = i == stage_idx && is_loading;
            let is_done = i < stage_idx || (!is_loading && progress >= 100.0);
            let _is_pending = i > stage_idx || (!is_loading && progress < 100.0 && i > 0);

            // Dot
            let _dot_char = if is_done { '●' } else if is_current { '◐' } else { '○' };
            let dot_color = if is_done { t.success } else if is_current { t.primary } else { t.fg_muted };
            let dot_idx = (3 * plane.width + dx) as usize;
            if dot_idx < plane.cells.len() {
                plane.cells[dot_idx].char = stage.icon;
                plane.cells[dot_idx].fg = dot_color;
            }

            // Stage name
            draw_text(&mut plane, dx + 2, 3, stage.name, dot_color, t.bg, is_current);

            // Connector line
            if i < STAGES.len() - 1 {
                let connector_len = 14u16.saturating_sub(stage.name.len() as u16 + 3);
                for cx in 0..connector_len {
                    let cidx = (3 * plane.width + dx + stage.name.len() as u16 + 3 + cx) as usize;
                    if cidx < plane.cells.len() {
                        plane.cells[cidx].char = '─';
                        plane.cells[cidx].fg = if is_done { t.success } else { t.fg_muted };
                    }
                }
            }
        }

        // ── Left column: ProgressRing + Spinner ───────────────────────────
        let col_x = 2u16;

        draw_text(&mut plane, col_x, 5, "ProgressRing", t.fg, t.bg, true);
        let ring_area = Rect::new(col_x, 6, 12, 4);
        let ring_plane = self.ring.borrow().render(ring_area);
        blit_to(&mut plane, &ring_plane, ring_area.x as usize, ring_area.y as usize);

        let (status_text, status_color) = if progress >= 100.0 {
            ("Complete!", t.success)
        } else if is_loading {
            ("Loading...", t.primary)
        } else {
            ("Ready", t.fg_muted)
        };
        draw_text(&mut plane, 16, 7, status_text, status_color, t.bg, true);

        // Current stage label
        if is_loading || progress >= 100.0 {
            let current = &STAGES[stage_idx];
            let stage_label = if progress >= 100.0 { "All stages complete" } else { current.name };
            draw_text(&mut plane, 16, 8, stage_label, t.secondary, t.bg, false);
        }

        // Spinner
        draw_text(&mut plane, col_x, 11, "Spinner", t.fg, t.bg, true);
        if is_loading {
            let spinner_area = Rect::new(12, 11, 8, 1);
            let spinner_plane = self.spinner.borrow().render(spinner_area);
            blit_to(&mut plane, &spinner_plane, spinner_area.x as usize, spinner_area.y as usize);
        } else {
            draw_text(&mut plane, 12, 11, "(idle)", t.fg_muted, t.bg, false);
        }

        // ── Right column: ProgressBar + Stats ─────────────────────────────
        let right_x = area.width * 40 / 100;

        draw_text(&mut plane, right_x, 5, "ProgressBar", t.fg, t.bg, true);
        let bar_area = Rect::new(right_x, 6, area.width.saturating_sub(right_x + 2), 3);
        let bar_plane = self.bar.borrow().render(bar_area);
        blit_to(&mut plane, &bar_plane, bar_area.x as usize, bar_area.y as usize);

        // Visual gauge
        draw_text(&mut plane, right_x, 10, "Visual Gauge", t.fg, t.bg, true);
        let gauge_y = 11;
        let gauge_w = area.width.saturating_sub(right_x + 2) as usize;
        let filled = (progress / 100.0 * gauge_w as f64) as usize;
        for dx in 0..gauge_w {
            let idx = (gauge_y * plane.width + right_x + dx as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = if dx < filled { '█' } else { '░' };
                plane.cells[idx].fg = if progress >= 100.0 { t.success } else { t.primary };
                plane.cells[idx].transparent = false;
            }
        }

        // Stats panel
        draw_text(&mut plane, right_x, 13, "Statistics", t.secondary, t.bg, true);
        for dx in 0..area.width.saturating_sub(right_x + 2) {
            let idx = (14 * plane.width + right_x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        let stats: Vec<(&str, String)> = vec![
            ("Progress", format!("{:.1}%", progress)),
            ("Stage", format!("{}/{}", stage_idx + 1, STAGES.len())),
            ("Elapsed", format!("{} ticks", self.elapsed_ticks())),
            ("Speed", if progress < 30.0 { "Fast".into() } else if progress < 70.0 { "Normal".into() } else { "Slow".into() }),
            ("Status", if progress >= 100.0 { "Complete".into() } else if is_loading { "Running".into() } else { "Idle".into() }),
        ];
        for (i, (label, value)) in stats.iter().enumerate() {
            let sy = 15 + i as u16;
            if sy >= area.height.saturating_sub(2) { break; }
            draw_text(&mut plane, right_x, sy, label, t.fg_muted, t.bg, false);
            let val_color = match *label {
                "Progress" if progress >= 100.0 => t.success,
                "Status" if *value == "Running" => t.primary,
                _ => t.fg,
            };
            draw_text(&mut plane, right_x + 12, sy, value, val_color, t.bg, false);
        }

        // Vertical divider
        for y in 5..area.height.saturating_sub(2) {
            let idx = (y * plane.width + right_x - 1) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Footer
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " SPACE:toggle | r:reset | {}:help | {}:back ",
            help_key, back_key,
        );
        let fy = area.height.saturating_sub(1);
        for (i, c) in footer.chars().enumerate() {
            let idx = (fy * plane.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        if self.show_help {
            let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(&mut plane, area, &self.theme, "Progress & Loading — Help", &[
                ("SPACE", "Toggle loading simulation"),
                ("r", "Reset progress"),
                (help_key, "Toggle this help"),
                (back_key, "Back"),
            ]);
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
                self.tick_count.set(0);
                self.start_tick.set(0);
                self.ring.borrow_mut().set_progress(0.0);
                self.bar.borrow_mut().set_progress(0.0);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();
        match kind {
            MouseEventKind::Down(_) => {
                // Step dots area (row 3): click a dot to jump progress to that stage
                if row == 3 {
                    let _right_x = area.width * 40 / 100;
                    let step_w = 14u16;
                    for i in 0..5 {
                        let sx = 2 + i * step_w;
                        if (sx..sx + step_w).contains(&col) {
                            let stage_pcts = [0.0, 15.0, 35.0, 60.0, 85.0];
                            let pct = stage_pcts[i as usize];
                            self.progress.set(pct);
                            self.loading.set(false);
                            self.ring.borrow_mut().set_progress(pct / 100.0);
                            self.bar.borrow_mut().set_progress((pct / 100.0) as f32);
                            self.dirty = true;
                            return true;
                        }
                    }
                }

                // ProgressRing (rows 6-9, cols 2-14): toggle loading
                if (6..10).contains(&row) && (2..14).contains(&col) {
                    if self.loading.get() { self.stop_loading(); } else { self.start_loading(); }
                    self.dirty = true;
                    return true;
                }

                // ProgressBar / gauge area (rows 6-12, cols right_x+): click to set value
                let right_x = area.width * 40 / 100;
                if (6..13).contains(&row) && col >= right_x && col < area.width.saturating_sub(2) {
                    let bar_w = area.width.saturating_sub(right_x + 2);
                    if bar_w > 0 {
                        let pct = (col.saturating_sub(right_x) as f64 / bar_w as f64 * 100.0).clamp(0.0, 100.0);
                        self.progress.set(pct);
                        self.loading.set(false);
                        self.ring.borrow_mut().set_progress(pct / 100.0);
                        self.bar.borrow_mut().set_progress((pct / 100.0) as f32);
                        self.dirty = true;
                    }
                    return true;
                }

                // Any other click: toggle loading
                if self.loading.get() { self.stop_loading(); } else { self.start_loading(); }
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.ring.borrow_mut().on_theme_change(theme);
        self.bar.borrow_mut().on_theme_change(theme);
        self.spinner.borrow_mut().on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}



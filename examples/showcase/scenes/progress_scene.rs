//! Embedded Progress/Loading scene for the showcase.
//!
//! Demonstrates ProgressRing, ProgressBar, and Spinner widgets
//! in a simulated multi-stage loading scenario with a timeline,
//! real-time stats, and an operations log.

use crate::scenes::shared_helpers::{blit_to, draw_text, draw_text_clipped, render_help_overlay};
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

const SIDEBAR_W: u16 = 22;
const DIV_X: u16 = SIDEBAR_W + 2;

struct Stage {
    name: &'static str,
    icon: char,
    weight: f64,
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
    op_log: RefCell<Vec<(String, String)>>,
    dirty: bool,
    area: Cell<Rect>,
}

impl ProgressScene {
    pub fn new(theme: Theme) -> Self {
        let ring = RefCell::new(ProgressRing::new(0.0)
            .with_theme(theme.clone())
            .with_size(10)
            .show_percentage(true)
            .with_label("Overall"));

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
            op_log: RefCell::new(vec![
                ("Initializing workspace".into(), "done".into()),
                ("Loading Cargo.toml".into(), "done".into()),
            ]),
            dirty: true,
            area: Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn start_loading(&self) {
        self.loading.set(true);
        self.progress.set(0.0);
        self.start_tick.set(self.tick_count.get());
        self.op_log.borrow_mut().clear();
        self.op_log.borrow_mut().push(("Starting build".into(), "info".into()));
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

            // Log stage transitions
            let prev_stage = Self::stage_at(current);
            let new_stage = Self::stage_at(next);
            if prev_stage != new_stage {
                let stage_name = STAGES.get(new_stage).map(|s| s.name).unwrap_or("Complete");
                self.op_log.borrow_mut().push((format!("{}...", stage_name), "info".into()));
            }

            if next >= 100.0 {
                self.loading.set(false);
                self.op_log.borrow_mut().push(("Build complete".into(), "done".into()));
            }
        }

        // Spin the spinner
        self.spinner.borrow_mut().tick();
    }

    fn stage_at(progress: f64) -> usize {
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

    fn current_stage_index(&self) -> usize {
        Self::stage_at(self.progress.get())
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

        // ── Header ──────────────────────────────────────────────────────
        draw_text(&mut plane, 2, 0, " Loading Dashboard ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 2), 0,
                  &theme_label, t.secondary, t.bg, false);
        if is_loading {
            draw_text(&mut plane, DIV_X.saturating_sub(6), 0, "▶ AUTO", t.primary, t.bg, true);
        }

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── Stage Timeline (row 2-4) ─────────────────────────────────────
        let timeline_y = 2;
        draw_text(&mut plane, 2, timeline_y, "Build Pipeline", t.primary, t.bg, true);

        let total_weight: f64 = STAGES.iter().map(|s| s.weight).sum();
        let main_w = area.width.saturating_sub(DIV_X + 4);
        let main_x = DIV_X + 2;

        // Stage pipeline
        for (i, stage) in STAGES.iter().enumerate() {
            let x_offset = main_x + (i as u16) * (main_w / STAGES.len() as u16);
            let is_done = i < stage_idx || (!is_loading && progress >= 100.0);
            let is_current = i == stage_idx && is_loading;

            // Stage box
            let box_w = main_w / STAGES.len() as u16 - 2;
            let box_h = 3u16;

            for dy in 0..box_h {
                for dx in 0..box_w {
                    let bx = x_offset + dx;
                    let by = timeline_y + 1 + dy;
                    let idx = (by * area.width + bx) as usize;
                    if idx < plane.cells.len() {
                        let bg = if is_done { t.success } else if is_current { t.primary } else { t.surface };
                        plane.cells[idx].bg = bg;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            // Stage icon + name
            let icon_idx = ((timeline_y + 2) * area.width + x_offset + 1) as usize;
            if icon_idx < plane.cells.len() {
                plane.cells[icon_idx].char = if is_done { '●' } else if is_current { '◐' } else { '○' };
                plane.cells[icon_idx].fg = if is_done || is_current { t.bg } else { t.fg_muted };
            }
            let name_x = x_offset + 3;
            let name_text = if stage.name.len() as u16 > box_w - 4 {
                &stage.name[..(box_w as usize - 5).max(1)]
            } else {
                stage.name
            };
            let fg = if is_done || is_current { t.bg } else { t.fg_muted };
            draw_text_clipped(&mut plane, name_x, timeline_y + 2, name_text, x_offset + box_w, fg, t.bg, false);

            // Progress bar for current stage
            if is_current {
                let stage_pct = (progress - (STAGES[..i].iter().map(|s| s.weight).sum::<f64>() / total_weight * 100.0))
                    / (stage.weight / total_weight * 100.0);
                let bar_fill = (stage_pct.clamp(0.0, 1.0) * (box_w as f64 - 2.0)) as usize;
                for dx in 1..bar_fill + 1 {
                    let idx = ((timeline_y + box_h) * area.width + x_offset + dx as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '█';
                        plane.cells[idx].fg = t.bg;
                    }
                }
            }
        }

        // ── Left sidebar ───────────────────────────────────────────────
        self.render_sidebar(&mut plane, area, t, progress, stage_idx, is_loading);

        // Vertical divider
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * area.width + DIV_X) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // ── Main area ───────────────────────────────────────────────────
        let main_x = DIV_X + 2;
        let main_w = area.width.saturating_sub(main_x + 2);

        // Section: Progress Ring
        let ring_y = 6;
        draw_text(&mut plane, main_x, ring_y, "Overall Progress", t.primary, t.bg, true);

        let ring_size = 14u16;
        let ring_area = Rect::new(main_x + 2, ring_y + 1, ring_size, ring_size);
        let ring_plane = self.ring.borrow().render(ring_area);
        blit_to(&mut plane, &ring_plane, ring_area.x as usize, ring_area.y as usize);

        // Status text next to ring
        let status_x = main_x + ring_size + 4;
        let (status_text, status_color) = if progress >= 100.0 {
            ("Complete!", t.success)
        } else if is_loading {
            ("Running...", t.primary)
        } else {
            ("Idle", t.fg_muted)
        };
        draw_text(&mut plane, status_x, ring_y + 3, status_text, status_color, t.bg, true);

        if is_loading || progress >= 100.0 {
            let current = &STAGES[stage_idx];
            let label = if progress >= 100.0 { "All stages done" } else { current.name };
            draw_text(&mut plane, status_x, ring_y + 5, label, t.secondary, t.bg, false);
        }

        // Section: Progress Bar
        let bar_y = ring_y + ring_size + 2;
        if bar_y + 3 < area.height.saturating_sub(6) {
            draw_text(&mut plane, main_x, bar_y, "Stage Progress", t.primary, t.bg, true);

            let bar_area = Rect::new(main_x, bar_y + 1, main_w, 3);
            let bar_plane = self.bar.borrow().render(bar_area);
            blit_to(&mut plane, &bar_plane, main_x as usize, (bar_y + 1) as usize);
        }

        // Section: Operations Log
        let log_y = bar_y + 5;
        if log_y + 5 < area.height.saturating_sub(4) {
            draw_text(&mut plane, main_x, log_y, "Operations Log", t.secondary, t.bg, true);
            for dx in 0..main_w {
                let idx = ((log_y + 1) * area.width + main_x + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }

            for (i, (op, status)) in self.op_log.borrow().iter().rev().take(4).enumerate() {
                let ly = log_y + 2 + i as u16;
                let icon = match status.as_str() {
                    "done" => "●",
                    "info" => "○",
                    "warn" => "◐",
                    _ => "○",
                };
                let icon_color = match status.as_str() {
                    "done" => t.success,
                    "info" => t.primary,
                    "warn" => t.warning,
                    _ => t.fg_muted,
                };
                let text_color = match status.as_str() {
                    "done" => t.success,
                    "warn" => t.warning,
                    _ => t.fg,
                };
                draw_text_clipped(&mut plane, main_x + 1, ly, icon, main_x + 4, icon_color, t.bg, false);
                draw_text_clipped(&mut plane, main_x + 4, ly, op, main_x + main_w, text_color, t.bg, false);
            }
        }

        // ── Footer ─────────────────────────────────────────────────────
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " SPACE:toggle | r:reset | Click stage:jump | {}:help | {}:back ",
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
            let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(&mut plane, area, &self.theme, "Loading Dashboard — Help", &[
                ("SPACE", "Toggle loading simulation"),
                ("r", "Reset progress"),
                ("Click stage", "Jump to that stage"),
                ("Click ring", "Toggle loading"),
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
                self.op_log.borrow_mut().clear();
                self.op_log.borrow_mut().push(("Workspace reset".into(), "info".into()));
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
                // Timeline stage click (rows 3-5, cols main_x+)
                let main_x = DIV_X + 2;
                let main_w = area.width.saturating_sub(DIV_X + 4);
                if row >= 3 && row <= 5 && col >= main_x {
                    let stage_w = main_w / STAGES.len() as u16;
                    let stage_idx = ((col - main_x) / stage_w) as usize;
                    if stage_idx < STAGES.len() {
                        let stage_pcts = [0.0, 15.0, 35.0, 60.0, 85.0];
                        let pct = stage_pcts[stage_idx];
                        self.progress.set(pct);
                        self.loading.set(false);
                        self.ring.borrow_mut().set_progress(pct / 100.0);
                        self.bar.borrow_mut().set_progress((pct / 100.0) as f32);
                        self.dirty = true;
                        return true;
                    }
                }

                // Progress ring click (rows 7-20, cols main_x+2 to main_x+16)
                let ring_y = 7;
                let ring_size = 14u16;
                if row >= ring_y && row < ring_y + ring_size
                    && col >= main_x + 2 && col < main_x + 2 + ring_size
                {
                    if self.loading.get() { self.stop_loading(); } else { self.start_loading(); }
                    self.dirty = true;
                    return true;
                }

                // Progress bar area click (rows bar_y+1 to bar_y+4)
                let ring_y = 6;
                let bar_y = ring_y + ring_size + 2;
                if row >= bar_y + 1 && row < bar_y + 4 && col >= main_x {
                    let bar_w = area.width.saturating_sub(main_x + 2);
                    if bar_w > 0 {
                        let pct = (col.saturating_sub(main_x) as f64 / bar_w as f64 * 100.0).clamp(0.0, 100.0);
                        self.progress.set(pct);
                        self.loading.set(false);
                        self.ring.borrow_mut().set_progress(pct / 100.0);
                        self.bar.borrow_mut().set_progress((pct / 100.0) as f32);
                        self.dirty = true;
                        return true;
                    }
                }
                false
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

impl ProgressScene {
    fn render_sidebar(&self, plane: &mut Plane, area: Rect, t: &Theme, progress: f64, stage_idx: usize, is_loading: bool) {
        let sx = 2u16;

        // Controls section
        draw_text(plane, sx, 2, "Controls", t.primary, t.bg, true);

        // Play/Pause button
        let btn_y = 3;
        let is_running = is_loading;
        let btn_bg = if is_running { t.warning } else { t.success };
        let btn_text = if is_running { "■ Stop" } else { "▶ Start" };
        for cx in 0..SIDEBAR_W {
            let idx = (btn_y * plane.width + sx + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = btn_bg;
                plane.cells[idx].transparent = false;
            }
        }
        draw_text_clipped(plane, sx + 1, btn_y, btn_text, sx + SIDEBAR_W, t.bg, btn_bg, false);

        // Reset button
        let reset_y = 5;
        let reset_text = "↺ Reset";
        for cx in 0..SIDEBAR_W {
            let idx = (reset_y * plane.width + sx + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        draw_text(plane, sx, reset_y, reset_text, t.fg, t.surface, false);

        // Divider
        let div1_y = 7;
        for dx in 0..SIDEBAR_W {
            let idx = (div1_y * plane.width + sx + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Stats section
        draw_text(plane, sx, 9, "Statistics", t.secondary, t.bg, true);

        let stats = [
            ("Progress", format!("{:.1}%", progress)),
            ("Stage", format!("{}/{}", stage_idx + 1, STAGES.len())),
            ("Elapsed", format!("{} ticks", self.elapsed_ticks())),
            ("Status", if progress >= 100.0 {
                "Complete".into()
            } else if is_loading {
                "Running".into()
            } else {
                "Idle".into()
            }),
        ];

        for (i, (label, value)) in stats.iter().enumerate() {
            let sy = 10 + i as u16;
            if sy >= area.height.saturating_sub(4) { break; }

            draw_text(plane, sx, sy, label, t.fg_muted, t.bg, false);
            let val_color = match *label {
                "Progress" if progress >= 100.0 => t.success,
                "Status" if value == "Running" => t.primary,
                _ => t.fg,
            };
            draw_text_clipped(plane, sx, sy + 1, value, sx + SIDEBAR_W, val_color, t.bg, false);
        }

        // Spinner preview
        let spin_y = area.height.saturating_sub(7);
        if spin_y > 14 {
            draw_text(plane, sx, spin_y, "Spinner", t.secondary, t.bg, true);
            let spin_area = Rect::new(sx + 1, spin_y + 1, 10, 1);
            let spin_plane = self.spinner.borrow().render(spin_area);
            blit_to(plane, &spin_plane, sx as usize + 1, spin_y as usize + 1);
        }

        // Stage names reference
        let ref_y = area.height.saturating_sub(10);
        if ref_y > 15 {
            draw_text(plane, sx, ref_y, "Stages", t.secondary, t.bg, true);
            for (i, stage) in STAGES.iter().enumerate() {
                let sy = ref_y + 1 + i as u16;
                let is_done = i < stage_idx;
                let is_current = i == stage_idx && is_loading;
                let color = if is_done { t.success } else if is_current { t.primary } else { t.fg_muted };
                let icon = if is_done { '●' } else if is_current { '◐' } else { '○' };

                let icon_idx = (sy * plane.width + sx) as usize;
                if icon_idx < plane.cells.len() {
                    plane.cells[icon_idx].char = icon;
                    plane.cells[icon_idx].fg = color;
                }
                draw_text_clipped(plane, sx + 2, sy, stage.name, sx + SIDEBAR_W, color, t.bg, false);
            }
        }
    }
}
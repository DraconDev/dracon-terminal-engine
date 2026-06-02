//! Embedded Metrics Hub scene for the showcase.
//!
//! A system metrics dashboard with interactive sliders controlling gauges,
//! spinning indicators, progress rings for circular metrics, and status badges.

use crate::scenes::shared_helpers::{blit_to, draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::{
    Gauge, ProgressRing, Slider, Spinner, StatusBadge, StatusBar, StatusSegment,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

const SIDEBAR_W: u16 = 22;
const DIV_X: u16 = SIDEBAR_W + 2;

pub struct MetricsHubScene {
    theme: Theme,
    keybindings: KeybindingSet,
    // Sliders
    cpu_slider: RefCell<Slider>,
    mem_slider: RefCell<Slider>,
    disk_slider: RefCell<Slider>,
    // Gauges
    cpu_gauge: RefCell<Gauge>,
    mem_gauge: RefCell<Gauge>,
    disk_gauge: RefCell<Gauge>,
    // Progress ring
    progress_ring: RefCell<ProgressRing>,
    progress_value: f64,
    // Spinner
    spinner: RefCell<Spinner>,
    // Status badges
    badges: RefCell<Vec<StatusBadge>>,
    // Status bar
    status_bar: RefCell<StatusBar>,
    // UI state
    selected_slider: usize,
    show_help: bool,
    dirty: bool,
    // Metrics history
    cpu_history: RefCell<Vec<f32>>,
    mem_history: RefCell<Vec<f32>>,
}

impl MetricsHubScene {
    pub fn new(theme: Theme) -> Self {
        let cpu_slider = Slider::new(WidgetId::new(800))
            .with_range(0.0, 100.0)
            .with_theme(theme.clone());
        let mem_slider = Slider::new(WidgetId::new(801))
            .with_range(0.0, 100.0)
            .with_theme(theme.clone());
        let disk_slider = Slider::new(WidgetId::new(802))
            .with_range(0.0, 100.0)
            .with_theme(theme.clone());

        let cpu_gauge = Gauge::with_id(WidgetId::new(810), "CPU")
            .max(100.0)
            .warn_threshold(70.0)
            .crit_threshold(90.0)
            .with_theme(theme.clone());
        let mem_gauge = Gauge::with_id(WidgetId::new(811), "MEM")
            .max(100.0)
            .warn_threshold(80.0)
            .crit_threshold(95.0)
            .with_theme(theme.clone());
        let disk_gauge = Gauge::with_id(WidgetId::new(812), "DSK")
            .max(100.0)
            .warn_threshold(75.0)
            .crit_threshold(90.0)
            .with_theme(theme.clone());

        let progress_ring = ProgressRing::new(0.0)
            .with_theme(theme.clone())
            .show_percentage(true)
            .with_label("Uptime");

        let spinner = Spinner::new(WidgetId::new(820)).with_theme(theme.clone());

        let badges = vec![
            StatusBadge::new(WidgetId::new(830))
                .with_status("ok")
                .with_label("API")
                .with_theme(theme.clone()),
            StatusBadge::new(WidgetId::new(831))
                .with_status("warn")
                .with_label("DB")
                .with_theme(theme.clone()),
            StatusBadge::new(WidgetId::new(832))
                .with_status("error")
                .with_label("Cache")
                .with_theme(theme.clone()),
            StatusBadge::new(WidgetId::new(833))
                .with_status("ok")
                .with_label("CDN")
                .with_theme(theme.clone()),
            StatusBadge::new(WidgetId::new(834))
                .with_status("ok")
                .with_label("Queue")
                .with_theme(theme.clone()),
        ];

        let status_bar = StatusBar::new(WidgetId::new(840))
            .add_segment(StatusSegment::new(
                "Tab: switch slider | ←/→: adjust | Space: tick | r: reset | F1: help | Esc: back",
            ))
            .with_theme(theme.clone());

        Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            cpu_slider: RefCell::new(cpu_slider),
            mem_slider: RefCell::new(mem_slider),
            disk_slider: RefCell::new(disk_slider),
            cpu_gauge: RefCell::new(cpu_gauge),
            mem_gauge: RefCell::new(mem_gauge),
            disk_gauge: RefCell::new(disk_gauge),
            progress_ring: RefCell::new(progress_ring),
            progress_value: 75.0,
            spinner: RefCell::new(spinner),
            badges: RefCell::new(badges),
            status_bar: RefCell::new(status_bar),
            selected_slider: 0,
            show_help: false,
            dirty: true,
            cpu_history: RefCell::new(Vec::new()),
            mem_history: RefCell::new(Vec::new()),
        }
    }

    fn sync_gauges(&self) {
        self.cpu_gauge
            .borrow_mut()
            .set_value(self.cpu_slider.borrow().value() as f64);
        self.mem_gauge
            .borrow_mut()
            .set_value(self.mem_slider.borrow().value() as f64);
        self.disk_gauge
            .borrow_mut()
            .set_value(self.disk_slider.borrow().value() as f64);
    }

    fn update_history(&self) {
        let cpu = self.cpu_slider.borrow().value();
        let mem = self.mem_slider.borrow().value();
        self.cpu_history.borrow_mut().push(cpu);
        self.mem_history.borrow_mut().push(mem);
        if self.cpu_history.borrow().len() > 30 {
            self.cpu_history.borrow_mut().remove(0);
        }
        if self.mem_history.borrow().len() > 30 {
            self.mem_history.borrow_mut().remove(0);
        }
    }

    fn tick(&mut self) {
        self.spinner.borrow_mut().tick();
        self.progress_value = (self.progress_value + 2.0) % 100.0;
        self.progress_ring
            .borrow_mut()
            .set_progress(self.progress_value / 100.0);
        self.update_history();
        self.dirty = true;
    }

    fn reset(&mut self) {
        self.progress_value = 0.0;
        self.progress_ring.borrow_mut().set_progress(0.0);
        self.cpu_history.borrow_mut().clear();
        self.mem_history.borrow_mut().clear();
        self.dirty = true;
    }
}

impl Scene for MetricsHubScene {
    fn on_enter(&mut self) {
        self.show_help = false;
        self.dirty = true;
    }

    fn on_exit(&mut self) {
        self.show_help = false;
    }

    fn scene_id(&self) -> &str {
        "metrics_hub"
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        self.sync_gauges();

        // Header
        draw_text(
            &mut plane,
            2,
            0,
            " Metrics Dashboard ",
            t.primary,
            t.bg,
            true,
        );
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

        // Left sidebar
        self.render_sidebar(&mut plane, area, t);

        // Vertical divider
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * plane.width + DIV_X) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Main area
        let main_x = DIV_X + 2;
        let main_w = area.width.saturating_sub(main_x + 2);

        // Sparkline chart
        let chart_y = 2;
        draw_text(
            &mut plane,
            main_x,
            chart_y,
            "Metrics History",
            t.primary,
            t.bg,
            true,
        );
        let chart_h = 8u16.min(area.height.saturating_sub(14));
        self.render_sparkline_chart(&mut plane, main_x, chart_y + 2, main_w.min(50), chart_h, t);

        // Sliders + Gauges
        let slider_y = chart_y + chart_h + 3;
        if slider_y + 8 < area.height.saturating_sub(6) {
            draw_text(
                &mut plane,
                main_x,
                slider_y,
                "Interactive Controls",
                t.primary,
                t.bg,
                true,
            );

            let labels = ["CPU", "MEM", "DSK"];
            let sliders: [&RefCell<Slider>; 3] =
                [&self.cpu_slider, &self.mem_slider, &self.disk_slider];
            let gauges: [&RefCell<Gauge>; 3] = [&self.cpu_gauge, &self.mem_gauge, &self.disk_gauge];

            for (i, label) in labels.iter().enumerate() {
                let y = slider_y + 1 + i as u16 * 2;
                let is_selected = i == self.selected_slider;

                let lbl = if is_selected {
                    format!("▸ {}:", label)
                } else {
                    format!("  {}:", label)
                };
                let lbl_color = if is_selected { t.primary } else { t.fg_muted };
                draw_text(&mut plane, main_x, y, &lbl, lbl_color, t.bg, is_selected);

                let slider_w = (main_w / 2).saturating_sub(6);
                let s_area = Rect::new(main_x + 6, y, slider_w, 1);
                sliders[i].borrow_mut().set_area(s_area);
                let s_plane = sliders[i].borrow().render(s_area);
                blit_to(&mut plane, &s_plane, (main_x + 6) as usize, y as usize);

                let val = sliders[i].borrow().value();
                draw_text(
                    &mut plane,
                    main_x + 6 + slider_w + 1,
                    y,
                    &format!("{:.0}%", val),
                    t.fg,
                    t.bg,
                    false,
                );

                let gauge_x = main_x + main_w / 2 + 2;
                let gauge_w = main_w.saturating_sub(main_w / 2 + 2);
                let g_area = Rect::new(gauge_x, y, gauge_w, 1);
                gauges[i].borrow_mut().set_area(g_area);
                let g_plane = gauges[i].borrow().render(g_area);
                blit_to(&mut plane, &g_plane, gauge_x as usize, y as usize);
            }
        }

        // Indicators section
        let ind_y = area.height.saturating_sub(12);
        if ind_y > slider_y + 8 {
            draw_text(
                &mut plane,
                main_x,
                ind_y,
                "Indicators",
                t.primary,
                t.bg,
                true,
            );

            let ring_size = 8u16.min(area.height.saturating_sub(ind_y + 3));
            if ring_size >= 4 {
                let ring_area = Rect::new(main_x, ind_y + 1, ring_size + 2, ring_size + 2);
                self.progress_ring.borrow_mut().set_area(ring_area);
                let ring_plane = self.progress_ring.borrow().render(ring_area);
                blit_to(
                    &mut plane,
                    &ring_plane,
                    main_x as usize,
                    (ind_y + 1) as usize,
                );
                draw_text(
                    &mut plane,
                    main_x,
                    ind_y + ring_size + 2,
                    &format!("Uptime: {:.0}%", self.progress_value),
                    t.fg_muted,
                    t.bg,
                    false,
                );
            }

            let sp_x = main_x + 14;
            let spinner_area = Rect::new(sp_x, ind_y + 3, 3, 1);
            self.spinner.borrow_mut().set_area(spinner_area);
            let sp_plane = self.spinner.borrow().render(spinner_area);
            blit_to(&mut plane, &sp_plane, sp_x as usize, (ind_y + 3) as usize);
            draw_text(
                &mut plane,
                sp_x + 4,
                ind_y + 3,
                "Tick",
                t.fg_muted,
                t.bg,
                false,
            );

            let badge_x = main_x + main_w / 2 + 2;
            draw_text(
                &mut plane, badge_x, ind_y, "Services", t.fg_muted, t.bg, true,
            );
            let badges = self.badges.borrow();
            for (i, badge) in badges.iter().enumerate() {
                let by = ind_y + 1 + i as u16;
                let b_area = Rect::new(badge_x, by, 12, 1);
                let b_plane = badge.render(b_area);
                blit_to(&mut plane, &b_plane, badge_x as usize, by as usize);
            }
        }

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self
            .status_bar
            .borrow()
            .render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        if self.show_help {
            let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(
                &mut plane,
                area,
                t,
                "Metrics Dashboard — Help",
                &[
                    ("Tab", "Switch slider"),
                    ("←/→", "Adjust selected slider"),
                    ("Space", "Tick spinner + progress"),
                    ("r", "Reset metrics"),
                    ("Click", "Set slider value"),
                    (help_key, "Toggle this help"),
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
                self.selected_slider = (self.selected_slider + 1) % 3;
                self.dirty = true;
                true
            }
            KeyCode::BackTab => {
                self.selected_slider = (self.selected_slider + 2) % 3;
                self.dirty = true;
                true
            }
            KeyCode::Left => {
                let slider = match self.selected_slider {
                    0 => &self.cpu_slider,
                    1 => &self.mem_slider,
                    _ => &self.disk_slider,
                };
                let cur = slider.borrow().value();
                slider.borrow_mut().set_value((cur - 5.0).max(0.0));
                self.update_history();
                self.dirty = true;
                true
            }
            KeyCode::Right => {
                let slider = match self.selected_slider {
                    0 => &self.cpu_slider,
                    1 => &self.mem_slider,
                    _ => &self.disk_slider,
                };
                let cur = slider.borrow().value();
                slider.borrow_mut().set_value((cur + 5.0).min(100.0));
                self.update_history();
                self.dirty = true;
                true
            }
            KeyCode::Char(' ') => {
                self.tick();
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.reset();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if matches!(kind, MouseEventKind::Down(_)) {
            let main_x = DIV_X + 2;
            if col >= main_x {
                let slider_y = 13;
                for i in 0..3 {
                    let sy = slider_y + 1 + i as u16 * 2;
                    if row == sy {
                        self.selected_slider = i;
                        let slider = match i {
                            0 => &self.cpu_slider,
                            1 => &self.mem_slider,
                            _ => &self.disk_slider,
                        };
                        let rel_x = col.saturating_sub(main_x + 6);
                        let val = (rel_x as f32 / 30.0 * 100.0).clamp(0.0, 100.0);
                        slider.borrow_mut().set_value(val);
                        self.update_history();
                        self.dirty = true;
                        return true;
                    }
                }
            }
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.cpu_slider.borrow_mut().on_theme_change(theme);
        self.mem_slider.borrow_mut().on_theme_change(theme);
        self.disk_slider.borrow_mut().on_theme_change(theme);
        self.cpu_gauge.borrow_mut().on_theme_change(theme);
        self.mem_gauge.borrow_mut().on_theme_change(theme);
        self.disk_gauge.borrow_mut().on_theme_change(theme);
        self.progress_ring.borrow_mut().on_theme_change(theme);
        self.spinner.borrow_mut().on_theme_change(theme);
        for badge in self.badges.borrow_mut().iter_mut() {
            badge.on_theme_change(theme);
        }
        self.status_bar.borrow_mut().on_theme_change(theme);
        self.dirty = true;
    }

    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

impl MetricsHubScene {
    fn render_sidebar(&self, plane: &mut Plane, area: Rect, t: &Theme) {
        let sx = 2u16;

        draw_text(plane, sx, 2, "Controls", t.primary, t.bg, true);

        let labels = ["CPU", "MEM", "DSK"];
        for (i, label) in labels.iter().enumerate() {
            let by = 3 + i as u16;
            let is_selected = i == self.selected_slider;
            let bg = if is_selected { t.primary } else { t.surface };

            for cx in 0..SIDEBAR_W {
                let idx = (by * plane.width + sx + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].transparent = false;
                }
            }
            let btn_text = format!(" {} Slider ", label);
            draw_text_clipped(
                plane,
                sx + 1,
                by,
                &btn_text,
                sx + SIDEBAR_W,
                t.bg,
                bg,
                false,
            );
        }

        // Divider
        let div1_y = 7;
        for dx in 0..SIDEBAR_W {
            let idx = (div1_y * plane.width + sx + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Current values
        let vals_y = 9;
        draw_text(plane, sx, vals_y, "Current Values", t.secondary, t.bg, true);

        let cpu = self.cpu_slider.borrow().value();
        let mem = self.mem_slider.borrow().value();
        let disk = self.disk_slider.borrow().value();

        let values = [
            (
                "CPU",
                cpu,
                if cpu > 90.0 {
                    t.error
                } else if cpu > 70.0 {
                    t.warning
                } else {
                    t.success
                },
            ),
            (
                "MEM",
                mem,
                if mem > 95.0 {
                    t.error
                } else if mem > 80.0 {
                    t.warning
                } else {
                    t.info
                },
            ),
            (
                "DSK",
                disk,
                if disk > 90.0 {
                    t.error
                } else if disk > 75.0 {
                    t.warning
                } else {
                    t.secondary
                },
            ),
        ];

        for (i, (label, value, color)) in values.iter().enumerate() {
            let vy = vals_y + 1 + i as u16;
            if vy >= area.height.saturating_sub(4) {
                break;
            }
            draw_text(plane, sx, vy, label, t.fg_muted, t.bg, false);
            draw_text(
                plane,
                sx + 10,
                vy,
                &format!("{:.0}%", value),
                *color,
                t.bg,
                true,
            );
        }

        // Uptime
        let uptime_y = vals_y + 5;
        if uptime_y + 2 < area.height.saturating_sub(4) {
            draw_text(plane, sx, uptime_y, "Uptime", t.secondary, t.bg, true);
            draw_text(
                plane,
                sx,
                uptime_y + 1,
                &format!("{:.0}%", self.progress_value),
                t.primary,
                t.bg,
                false,
            );
        }

        // Legend
        let leg_y = area.height.saturating_sub(8);
        if leg_y > uptime_y + 3 {
            draw_text(plane, sx, leg_y, "Thresholds", t.secondary, t.bg, true);
            draw_text(plane, sx, leg_y + 1, "70% Warning", t.warning, t.bg, false);
            draw_text(plane, sx, leg_y + 2, "90%+ Critical", t.error, t.bg, false);
        }

        // Service status
        let svc_y = area.height.saturating_sub(5);
        if svc_y > leg_y + 4 {
            draw_text(plane, sx, svc_y, "Services", t.secondary, t.bg, true);
            let badges = self.badges.borrow();
            let ok = badges.iter().filter(|b| b.status == "ok").count();
            let warn = badges.iter().filter(|b| b.status == "warn").count();
            let err = badges.iter().filter(|b| b.status == "error").count();
            let summary = format!("{} OK  {} warn  {} err", ok, warn, err);
            draw_text_clipped(
                plane,
                sx,
                svc_y + 1,
                &summary,
                sx + SIDEBAR_W,
                t.fg_muted,
                t.bg,
                false,
            );
        }
    }

    fn render_sparkline_chart(&self, plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, t: &Theme) {
        if w < 4 || h < 3 {
            return;
        }

        // Border
        for i in 0..w {
            let top = (y * plane.width + x + i) as usize;
            let bot = ((y + h - 1) * plane.width + x + i) as usize;
            if top < plane.cells.len() {
                plane.cells[top].char = '─';
                plane.cells[top].fg = t.outline;
            }
            if bot < plane.cells.len() {
                plane.cells[bot].char = '─';
                plane.cells[bot].fg = t.outline;
            }
        }
        for j in 0..h {
            let left = ((y + j) * plane.width + x) as usize;
            let right = ((y + j) * plane.width + x + w - 1) as usize;
            if left < plane.cells.len() {
                plane.cells[left].char = '│';
                plane.cells[left].fg = t.outline;
            }
            if right < plane.cells.len() {
                plane.cells[right].char = '│';
                plane.cells[right].fg = t.outline;
            }
        }

        let cpu_h = self.cpu_history.borrow();
        let mem_h = self.mem_history.borrow();
        let chart_w = (w as usize).saturating_sub(2);
        let chart_h = (h as usize).saturating_sub(2);
        let start = cpu_h.len().saturating_sub(chart_w);

        for (i, cpu_val) in cpu_h.iter().skip(start).enumerate() {
            if i >= chart_w {
                break;
            }
            let bar_h = (*cpu_val as f64 / 100.0 * chart_h as f64) as usize;
            for j in 0..bar_h.min(chart_h) {
                let by = y + h - 2 - j as u16;
                let idx = (by * plane.width + x + 1 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '▓';
                    plane.cells[idx].fg = t.primary;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        for (i, mem_val) in mem_h.iter().skip(start).enumerate() {
            if i >= chart_w {
                break;
            }
            let bar_h = (*mem_val as f64 / 100.0 * chart_h as f64) as usize;
            for j in 0..bar_h.min(chart_h) {
                let by = y + h - 2 - j as u16;
                let idx = (by * plane.width + x + 1 + i as u16) as usize;
                if idx < plane.cells.len() && plane.cells[idx].char != '▓' {
                    plane.cells[idx].char = '▒';
                    plane.cells[idx].fg = t.info;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        draw_text(plane, x + 1, y + h - 1, "0", t.fg_muted, t.bg, false);
        draw_text(plane, x + w - 3, y + h - 1, "100", t.fg_muted, t.bg, false);
        draw_text(
            plane,
            x + w.saturating_sub(10),
            y + 1,
            "▓ CPU  ▒ MEM",
            t.fg_muted,
            t.bg,
            false,
        );
    }
}

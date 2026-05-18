//! Metrics Hub scene — Slider + Gauge + ProgressRing + Spinner + StatusBadge.
//!
//! A system metrics dashboard with interactive sliders controlling gauges,
//! spinning indicators, progress rings for circular metrics, and status badges.

#![allow(dead_code)]

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    Divider, Gauge, Label, ProgressRing, Slider, Spinner, StatusBar,
    StatusBadge, StatusSegment,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

pub struct MetricsHubScene {
    theme: Theme,
    keybindings: KeybindingSet,
    // Sliders (control values)
    cpu_slider: RefCell<Slider>,
    mem_slider: RefCell<Slider>,
    disk_slider: RefCell<Slider>,
    // Gauges (display values)
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
    // UI state
    status_bar: RefCell<StatusBar>,
    selected_slider: usize,
    show_help: bool,
    dirty: bool,
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

        let spinner = Spinner::new(WidgetId::new(820))
            .with_theme(theme.clone());

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
                "←/→: adjust | Tab: switch | Space: tick | R: ring+10% | F1: help | Esc: back",
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
            progress_value: 0.0,
            spinner: RefCell::new(spinner),
            badges: RefCell::new(badges),
            status_bar: RefCell::new(status_bar),
            selected_slider: 0,
            show_help: false,
            dirty: true,
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

    fn tick(&mut self) {
        self.spinner.borrow_mut().tick();
        self.progress_value = (self.progress_value + 1.0) % 100.0;
        self.progress_ring
            .borrow_mut()
            .set_progress(self.progress_value / 100.0);
        self.dirty = true;
    }

    fn bump_ring(&mut self) {
        self.progress_value = (self.progress_value + 10.0) % 100.0;
        self.progress_ring
            .borrow_mut()
            .set_progress(self.progress_value / 100.0);
        self.dirty = true;
    }
}

impl Scene for MetricsHubScene {
    fn on_enter(&mut self) {}
    fn on_exit(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // Sync gauges from sliders
        self.sync_gauges();

        // ── Title ──────────────────────────────────────────────────
        let title = Label::new("Metrics Hub")
            .with_style(Styles::BOLD)
            .with_theme(t.clone());
        let title_plane = title.render(Rect::new(0, 0, 14, 1));
        blit_to(&mut plane, &title_plane, 1, 0);
        draw_text(
            &mut plane,
            16,
            0,
            "— Slider · Gauge · ProgressRing · Spinner · StatusBadge",
            t.fg_muted,
            t.bg,
            false,
        );

        // ── Divider ────────────────────────────────────────────────
        let div = Divider::new()
            .with_label("System Metrics")
            .with_theme(t.clone());
        let div_plane = div.render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &div_plane, 0, 1);

        // ── Slider + Gauge rows (rows 2-7) ─────────────────────────
        let labels = ["CPU", "MEM", "DSK"];
        let sliders: [&RefCell<Slider>; 3] = [
            &self.cpu_slider,
            &self.mem_slider,
            &self.disk_slider,
        ];
        let gauges: [&RefCell<Gauge>; 3] = [
            &self.cpu_gauge,
            &self.mem_gauge,
            &self.disk_gauge,
        ];

        for (i, label) in labels.iter().enumerate() {
            let y = 2 + i as u16 * 2;
            let is_selected = i == self.selected_slider;

            // Label
            let lbl = if is_selected {
                format!("▸ {}", label)
            } else {
                format!("  {}", label)
            };
            let lbl_color = if is_selected { t.primary } else { t.fg };
            draw_text(&mut plane, 1, y, &lbl, lbl_color, t.bg, is_selected);

            // Slider (left half)
            let slider_w = (area.width / 2).saturating_sub(8);
            if slider_w > 4 {
                let s_area = Rect::new(6, y, slider_w, 1);
                sliders[i].borrow_mut().set_area(s_area);
                let s_plane = sliders[i].borrow().render(s_area);
                blit_to(&mut plane, &s_plane, 6, y as usize);
            }

            // Value display
            let val = sliders[i].borrow().value();
            draw_text(
                &mut plane,
                6 + slider_w + 1,
                y,
                &format!("{:.0}%", val),
                t.fg,
                t.bg,
                false,
            );

            // Gauge (right half)
            let gauge_x = area.width / 2 + 2;
            let gauge_w = area.width.saturating_sub(gauge_x + 1);
            if gauge_w > 4 {
                let g_area = Rect::new(gauge_x, y, gauge_w, 1);
                gauges[i].borrow_mut().set_area(g_area);
                let g_plane = gauges[i].borrow().render(g_area);
                blit_to(&mut plane, &g_plane, gauge_x as usize, y as usize);
            }
        }

        // ── Divider ────────────────────────────────────────────────
        let div2_y = 2 + labels.len() as u16 * 2;
        let div2 = Divider::new()
            .with_label("Indicators")
            .with_theme(t.clone());
        let div2_plane = div2.render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &div2_plane, 0, div2_y as usize);

        // ── Progress Ring + Spinner row ────────────────────────────
        let ind_y = div2_y + 1;

        // Progress ring (left area)
        let ring_size = 6u16.min(area.height.saturating_sub(ind_y + 4));
        if ring_size >= 3 {
            let ring_area = Rect::new(2, ind_y, ring_size + 2, ring_size + 2);
            self.progress_ring.borrow_mut().set_area(ring_area);
            let ring_plane = self.progress_ring.borrow().render(ring_area);
            blit_to(&mut plane, &ring_plane, 2, ind_y as usize);
            draw_text(
                &mut plane,
                2,
                ind_y + ring_size + 2,
                &format!("Uptime: {:.0}%", self.progress_value),
                t.fg_muted,
                t.bg,
                false,
            );
        }

        // Spinner (next to ring)
        let sp_x = ring_size + 6;
        let spinner_area = Rect::new(sp_x, ind_y + 1, 3, 1);
        self.spinner.borrow_mut().set_area(spinner_area);
        let sp_plane = self.spinner.borrow().render(spinner_area);
        blit_to(&mut plane, &sp_plane, sp_x as usize, (ind_y + 1) as usize);
        draw_text(&mut plane, sp_x + 4, ind_y + 1, "Loading…", t.fg_muted, t.bg, false);

        // ── Status Badges ──────────────────────────────────────────
        let badge_y = ind_y + 1;
        let badge_x = area.width / 2 + 2;
        draw_text(&mut plane, badge_x, ind_y, "Services", t.fg, t.bg, true);

        let badges = self.badges.borrow();
        for (i, badge) in badges.iter().enumerate() {
            let by = badge_y + i as u16;
            let b_area = Rect::new(badge_x, by, 10, 1);
            // Can't call set_area on borrowed ref easily, just render directly
            let b_plane = badge.render(b_area);
            blit_to(&mut plane, &b_plane, badge_x as usize, by as usize);
        }

        // ── Status bar ─────────────────────────────────────────────
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self
            .status_bar
            .borrow()
            .render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        if self.show_help {
            render_help_overlay(&mut plane, area, t, "Metrics Hub — Help", &[("←/→", "Adjust selected slider"), ("Tab", "Switch slider (CPU/MEM/DSK)"), ("Space", "Tick spinner + progress"), ("R", "Bump progress ring +10%"), ("Click slider", "Set slider value"), ("F1", "Toggle this help"), ("Esc", "Back to showcase")]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::HELP, &key)
                || self.keybindings.matches(actions::BACK, &key)
            {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
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
                self.dirty = true;
                true
            }
            KeyCode::Char(' ') => {
                self.tick();
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.bump_ring();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Slider clicks (rows 2, 4, 6)
        if matches!(kind, MouseEventKind::Down(_)) {
            for i in 0..3 {
                let slider_y = 2 + i as u16 * 2;
                if row == slider_y {
                    self.selected_slider = i;
                    let slider_w = (col.max(6) - 6) as f32;
                    let val = (slider_w / 30.0 * 100.0).clamp(0.0, 100.0);
                    let slider = match i {
                        0 => &self.cpu_slider,
                        1 => &self.mem_slider,
                        _ => &self.disk_slider,
                    };
                    slider.borrow_mut().set_value(val);
                    self.dirty = true;
                    return true;
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

    fn scene_id(&self) -> &str {
        "metrics_hub"
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


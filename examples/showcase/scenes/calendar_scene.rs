//! Embedded Calendar scene for the showcase.
//!
//! Demonstrates the Calendar widget with event markers, upcoming events,
//! date detail panel, and month navigation.

use crate::scenes::shared_helpers::{blit_to, draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{Calendar, StatusBar, StatusSegment};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

struct CalendarEvent {
    date: &'static str, // "2026-05-17"
    title: &'static str,
    category: &'static str, // "meeting", "deadline", "holiday", "reminder"
    time: &'static str,
}

const EVENTS: &[CalendarEvent] = &[
    CalendarEvent {
        date: "2026-05-17",
        title: "Release v0.4",
        category: "deadline",
        time: "09:00",
    },
    CalendarEvent {
        date: "2026-05-17",
        title: "Team standup",
        category: "meeting",
        time: "10:30",
    },
    CalendarEvent {
        date: "2026-05-18",
        title: "Design review",
        category: "meeting",
        time: "14:00",
    },
    CalendarEvent {
        date: "2026-05-19",
        title: "Sprint planning",
        category: "meeting",
        time: "09:00",
    },
    CalendarEvent {
        date: "2026-05-20",
        title: "API freeze",
        category: "deadline",
        time: "17:00",
    },
    CalendarEvent {
        date: "2026-05-21",
        title: "Demo day",
        category: "reminder",
        time: "15:00",
    },
    CalendarEvent {
        date: "2026-05-22",
        title: "Sprint retro",
        category: "meeting",
        time: "11:00",
    },
    CalendarEvent {
        date: "2026-05-25",
        title: "Memorial Day",
        category: "holiday",
        time: "All day",
    },
    CalendarEvent {
        date: "2026-05-28",
        title: "Code review",
        category: "reminder",
        time: "10:00",
    },
    CalendarEvent {
        date: "2026-05-30",
        title: "Release v0.5",
        category: "deadline",
        time: "09:00",
    },
    CalendarEvent {
        date: "2026-06-01",
        title: "Q2 kickoff",
        category: "meeting",
        time: "09:00",
    },
    CalendarEvent {
        date: "2026-06-05",
        title: "Hackathon",
        category: "reminder",
        time: "10:00",
    },
    CalendarEvent {
        date: "2026-06-10",
        title: "Board review",
        category: "deadline",
        time: "14:00",
    },
    CalendarEvent {
        date: "2026-06-19",
        title: "Juneteenth",
        category: "holiday",
        time: "All day",
    },
];

fn category_color(cat: &str, theme: &Theme) -> Color {
    match cat {
        "meeting" => theme.primary,
        "deadline" => theme.error,
        "holiday" => theme.success,
        "reminder" => theme.warning,
        _ => theme.fg_muted,
    }
}

fn category_icon(cat: &str) -> char {
    match cat {
        "meeting" => 'M',
        "deadline" => 'D',
        "holiday" => 'H',
        "reminder" => 'R',
        _ => '•',
    }
}

pub struct CalendarScene {
    theme: Theme,
    show_help: bool,
    calendar: Calendar,
    selected_date: Option<String>,
    keybindings: KeybindingSet,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    status_bar: RefCell<StatusBar>,
}

impl CalendarScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme: theme.clone(),
            show_help: false,
            calendar: Calendar::new().with_theme(theme.clone()),
            selected_date: None,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
            status_bar: RefCell::new(
                StatusBar::new(WidgetId::new(100))
                    .add_segment(StatusSegment::new(
                        "←→:month | Enter:select | c:clear | F1:help | Esc:back",
                    ))
                    .with_theme(theme),
            ),
        }
    }

    fn events_for_date(&self, date: &str) -> Vec<&CalendarEvent> {
        EVENTS.iter().filter(|e| e.date == date).collect()
    }

    fn upcoming_events(&self, after: &str) -> Vec<&CalendarEvent> {
        EVENTS.iter().filter(|e| e.date >= after).take(8).collect()
    }

    fn render_sidebar(&self, plane: &mut Plane, x: u16, y: u16, w: u16, area: Rect) {
        let t = &self.theme;
        let max_x = x + w;

        // Upcoming Events panel
        draw_text_clipped(plane, x, y, "Upcoming", max_x, t.primary, t.bg, true);
        draw_text_clipped(
            plane,
            x + 10,
            y,
            &format!("({} events)", EVENTS.len()),
            max_x,
            t.fg_muted,
            t.bg,
            false,
        );

        // Divider
        for dx in 0..w {
            let dx_pos = x + dx;
            if dx_pos >= max_x {
                break;
            }
            let idx = ((y + 1) * plane.width + dx_pos) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        let after = self.selected_date.as_deref().unwrap_or(EVENTS[0].date);
        let upcoming = self.upcoming_events(after);
        let max_events = ((area.height.saturating_sub(y + 6)) / 2) as usize;

        for (i, event) in upcoming.iter().take(max_events).enumerate() {
            let ey = y + 2 + i as u16 * 2;
            if ey + 1 >= area.height.saturating_sub(2) {
                break;
            }

            // Category dot
            let dot_idx = (ey * plane.width + x) as usize;
            if dot_idx < plane.cells.len() {
                plane.cells[dot_idx].char = category_icon(event.category);
                plane.cells[dot_idx].fg = category_color(event.category, t);
                plane.cells[dot_idx].transparent = false;
            }

            // Event title
            draw_text_clipped(
                plane,
                x + 2,
                ey,
                event.title,
                max_x,
                category_color(event.category, t),
                t.bg,
                false,
            );

            // Date + time
            let meta = format!(
                "{} {}",
                event.date.strip_prefix("2026-").unwrap_or(event.date),
                event.time
            );
            draw_text_clipped(plane, x + 2, ey + 1, &meta, max_x, t.fg_muted, t.bg, false);
        }
    }

    fn render_detail_panel(&self, plane: &mut Plane, x: u16, y: u16, w: u16) {
        let t = &self.theme;
        let max_x = x + w;

        let date_str = self.selected_date.as_deref().unwrap_or("No date selected");
        let events = self
            .selected_date
            .as_deref()
            .map(|d| self.events_for_date(d))
            .unwrap_or_default();

        // Panel border
        draw_text_clipped(plane, x, y, "Selected Date", max_x, t.primary, t.bg, true);
        draw_text_clipped(plane, x + 14, y, date_str, max_x, t.fg, t.bg, false);

        for dx in 0..w {
            let dx_pos = x + dx;
            if dx_pos >= max_x {
                break;
            }
            let idx = ((y + 1) * plane.width + dx_pos) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        if events.is_empty() {
            draw_text_clipped(plane, x, y + 2, "No events", max_x, t.fg_muted, t.bg, false);
            draw_text_clipped(plane, x, y + 3, "Free day!", max_x, t.success, t.bg, true);
        } else {
            draw_text_clipped(
                plane,
                x,
                y + 2,
                &format!("{} event(s):", events.len()),
                max_x,
                t.fg,
                t.bg,
                true,
            );

            for (i, event) in events.iter().enumerate() {
                let ey = y + 4 + i as u16 * 2;
                let dot_idx = (ey * plane.width + x) as usize;
                if dot_idx < plane.cells.len() {
                    plane.cells[dot_idx].char = category_icon(event.category);
                    plane.cells[dot_idx].fg = category_color(event.category, t);
                    plane.cells[dot_idx].transparent = false;
                }
                draw_text_clipped(
                    plane,
                    x + 2,
                    ey,
                    event.title,
                    max_x,
                    category_color(event.category, t),
                    t.bg,
                    false,
                );
                draw_text_clipped(
                    plane,
                    x + 2,
                    ey + 1,
                    event.time,
                    max_x,
                    t.fg_muted,
                    t.bg,
                    false,
                );
            }
        }
    }

    fn render_stats_bar(&self, plane: &mut Plane, x: u16, y: u16) {
        let t = &self.theme;
        let meetings = EVENTS.iter().filter(|e| e.category == "meeting").count();
        let deadlines = EVENTS.iter().filter(|e| e.category == "deadline").count();
        let holidays = EVENTS.iter().filter(|e| e.category == "holiday").count();

        // Mini stat pills
        let stats = [
            (meetings, "mtg", t.primary),
            (deadlines, "due", t.error),
            (holidays, "hol", t.success),
        ];
        let mut sx = x;
        for (count, label, color) in stats {
            let pill = format!(" {} {} ", count, label);
            draw_text(plane, sx, y, &pill, color, t.bg, true);
            sx += pill.len() as u16 + 1;
        }
    }
}

impl Scene for CalendarScene {
    fn scene_id(&self) -> &str {
        "calendar"
    }

    fn on_enter(&mut self) {
        // Reset selected date when entering the scene
        self.selected_date = None;
        self.show_help = false;
        self.dirty = true;
    }

    fn on_exit(&mut self) {
        // Clean up state when leaving the scene
        self.show_help = false;
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
        draw_text(&mut plane, 2, 0, " Calendar ", t.primary, t.bg, true);

        // Stats bar in header
        self.render_stats_bar(&mut plane, 18, 0);

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

        // Layout: Calendar (left ~38w) | Sidebar (right ~rest)
        let cal_w = 38u16.min(area.width.saturating_sub(24));
        let cal_area = Rect::new(2, 2, cal_w, area.height.saturating_sub(6));
        let cal_plane = self.calendar.render(cal_area);
        blit_to(
            &mut plane,
            &cal_plane,
            cal_area.x as usize,
            cal_area.y as usize,
        );

        // Vertical divider
        let div_x = cal_w + 3;
        for y in 2..area.height.saturating_sub(2) {
            let idx = (y * area.width + div_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Right sidebar: upcoming events + selected date detail
        let sidebar_x = div_x + 2;
        let sidebar_w = area.width.saturating_sub(sidebar_x + 2);

        // Detail panel (top of sidebar)
        self.render_detail_panel(&mut plane, sidebar_x, 2, sidebar_w);

        // Upcoming events (below detail)
        let events_y = 2
            + 4
            + (self
                .selected_date
                .as_deref()
                .map(|d| self.events_for_date(d).len())
                .unwrap_or(0)
                * 2
                + 2)
            .min(6) as u16;
        self.render_sidebar(&mut plane, sidebar_x, events_y, sidebar_w, area);

        // Event legend at bottom of calendar area (clipped to left panel)
        let legend_y = cal_area.y + cal_area.height + 1;
        if legend_y < area.height.saturating_sub(3) {
            let legends = [
                ("M Meeting", t.primary),
                ("D Deadline", t.error),
                ("H Holiday", t.success),
                ("R Reminder", t.warning),
            ];
            let mut lx = 2u16;
            for (label, color) in legends {
                let next_x = lx + label.len() as u16 + 2;
                if next_x > div_x {
                    break;
                } // clip at vertical divider
                draw_text(&mut plane, lx, legend_y, label, color, t.bg, false);
                lx = next_x;
            }
        }

        // Footer
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " <>:month | Enter:select | c:clear | {}:help | {}:back ",
            help_key, back_key
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
            render_help_overlay(
                &mut plane,
                area,
                t,
                "Calendar Help",
                &[
                    ("< >", "Navigate months"),
                    ("Enter", "Select date"),
                    ("c", "Clear selection"),
                    ("Click", "Select date on calendar"),
                    (back_key, "Back"),
                ],
            );
        }

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_area = Rect::new(0, sb_y, area.width, 1);
        self.status_bar.borrow_mut().set_area(sb_area);
        let sb_plane = self.status_bar.borrow().render(sb_area);
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

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
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        // Clear selection — delegate to Calendar so both states stay in sync
        if key.code == KeyCode::Char('c') && key.modifiers.is_empty() {
            self.calendar.handle_key(key);
            self.selected_date = None;
            return true;
        }

        if self.calendar.handle_key(key) {
            // Sync scene's selected_date with Calendar's internal selection
            self.selected_date = self.calendar.selected().map(|d| d.to_string());
            return true;
        }
        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();
        let cal_w = 38u16.min(area.width.saturating_sub(24));
        let cal_area = Rect::new(2, 2, cal_w, area.height.saturating_sub(6));
        let rel_col = col.saturating_sub(cal_area.x);
        let rel_row = row.saturating_sub(cal_area.y);
        if self.calendar.handle_mouse(kind, rel_col, rel_row) {
            if let Some(date) = self.calendar.selected() {
                self.selected_date = Some(date.to_string());
            }
            return true;
        }

        // Sidebar events: click on event list to select its date
        if let MouseEventKind::Down(_) = kind {
            let sidebar_x = cal_w + 5;
            if col >= sidebar_x && row >= 6 {
                let event_idx = ((row - 6) / 2) as usize;
                let after = self.selected_date.as_deref().unwrap_or(EVENTS[0].date);
                let upcoming = self.upcoming_events(after);
                if event_idx < upcoming.len() {
                    let event = upcoming[event_idx];
                    self.selected_date = Some(event.date.to_string());
                    self.mark_dirty();
                    return true;
                }
            }
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.calendar.on_theme_change(theme);
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

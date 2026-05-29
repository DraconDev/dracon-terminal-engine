//! Calendar widget demonstration.
//!
//! Shows the Calendar widget with both single-selection and date-range modes.
//! - Click on days to select
//! - Use < and > to navigate months
//! - Press Enter to confirm selection
//! - Press C or Backspace to clear selection
//! - Press R in range mode to reset the range
//! - Press T to toggle between single and range mode

use chrono::NaiveDate;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

struct CalendarDemo {
    id: WidgetId,
    area: Rect,
    should_quit: Rc<AtomicBool>,
    theme: Theme,
    calendar_single: Calendar,
    calendar_range: Calendar,
    range_mode: bool,
    selected_date: RefCell<Option<NaiveDate>>,
    selected_range: RefCell<Option<(NaiveDate, NaiveDate)>>,
    show_help: bool,
    dirty: bool,
    keybindings: KeybindingSet,
}

impl CalendarDemo {
    fn new(should_quit: Rc<AtomicBool>, theme: Theme) -> Self {
        let selected_date = RefCell::new(None);
        let selected_range = RefCell::new(None);

        // Single selection calendar
        let sd = selected_date.clone();
        let mut cal_single = Calendar::new();
        cal_single = cal_single.with_theme(theme.clone()).on_select(move |date| {
            *sd.borrow_mut() = Some(date);
        });

        // Range selection calendar
        let sr = selected_range.clone();
        let mut cal_range = Calendar::new();
        cal_range = cal_range
            .with_theme(theme.clone())
            .with_range_mode()
            .on_range_select(move |start, end| {
                *sr.borrow_mut() = Some((start, end));
            });

        Self {
            id: WidgetId::new(1),
            area: Rect::default(),
            should_quit,
            theme,
            calendar_single: cal_single,
            calendar_range: cal_range,
            range_mode: false,
            selected_date,
            selected_range,
            show_help: false,
            dirty: true,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn active_calendar(&self) -> &Calendar {
        if self.range_mode {
            &self.calendar_range
        } else {
            &self.calendar_single
        }
    }

    fn active_calendar_mut(&mut self) -> &mut Calendar {
        if self.range_mode {
            &mut self.calendar_range
        } else {
            &mut self.calendar_single
        }
    }

    fn toggle_mode(&mut self) {
        self.range_mode = !self.range_mode;
        // Sync theme to newly active calendar
        let theme = self.theme.clone();
        self.active_calendar_mut().on_theme_change(&theme);
        self.dirty = true;
    }
}

impl Widget for CalendarDemo {
    fn needs_render(&self) -> bool {
        self.dirty || self.calendar_single.needs_render() || self.calendar_range.needs_render()
    }

    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
        // Split area for two calendars side by side
        let half = area.width / 2;
        self.calendar_single
            .set_area(Rect::new(0, 0, half, area.height));
        self.calendar_range
            .set_area(Rect::new(half, 0, half, area.height));
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        let half = area.width / 2;

        // Render active calendar (centered in its half)
        let cal = self.active_calendar();
        let cal_height = 10u16;
        let cal_width = 25u16;

        // Center single calendar
        let single_x = (half.saturating_sub(cal_width)) / 2;
        let single_y = (area.height.saturating_sub(cal_height)) / 2;

        // Render the calendar to a temporary plane and blit
        let temp_area = Rect::new(0, 0, cal_width, cal_height);
        let temp_plane = cal.render(temp_area);

        for y in 0..cal_height {
            for x in 0..cal_width {
                let src_idx = (y * cal_width + x) as usize;
                let dst_idx = ((single_y + y) * area.width + single_x + x) as usize;
                if src_idx < temp_plane.cells.len() && dst_idx < plane.cells.len() {
                    plane.cells[dst_idx] = temp_plane.cells[src_idx];
                }
            }
        }

        // Draw title for single mode
        let title = "Single Date Selection";
        let title_len = title.len() as u16;
        let title_x = (half.saturating_sub(title_len)) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = ((single_y - 2) * area.width + title_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if !self.range_mode {
                    self.theme.primary
                } else {
                    self.theme.fg_muted
                };
                plane.cells[idx].style = if !self.range_mode {
                    Styles::BOLD
                } else {
                    Styles::empty()
                };
            }
        }

        // Render inactive calendar (dimmed) on the right
        let inactive_cal = if self.range_mode {
            &self.calendar_single
        } else {
            &self.calendar_range
        };

        let range_x = half + (half.saturating_sub(cal_width)) / 2;
        let range_y = (area.height.saturating_sub(cal_height)) / 2;

        let temp_area2 = Rect::new(0, 0, cal_width, cal_height);
        let temp_plane2 = inactive_cal.render(temp_area2);

        for y in 0..cal_height {
            for x in 0..cal_width {
                let src_idx = (y * cal_width + x) as usize;
                let dst_idx = ((range_y + y) * area.width + range_x + x) as usize;
                if src_idx < temp_plane2.cells.len() && dst_idx < plane.cells.len() {
                    let mut cell = temp_plane2.cells[src_idx];
                    // Dim inactive calendar
                    cell.fg = self.theme.fg_muted;
                    plane.cells[dst_idx] = cell;
                }
            }
        }

        // Title for range mode
        let range_title = "Date Range Selection";
        let range_title_len = range_title.len() as u16;
        let range_title_x = half + (half.saturating_sub(range_title_len)) / 2;
        for (i, c) in range_title.chars().enumerate() {
            let idx = ((range_y - 2) * area.width + range_title_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if self.range_mode {
                    self.theme.primary
                } else {
                    self.theme.fg_muted
                };
                plane.cells[idx].style = if self.range_mode {
                    Styles::BOLD
                } else {
                    Styles::empty()
                };
            }
        }

        // Selection display
        let info_y = area.height.saturating_sub(4);
        if !self.range_mode {
            if let Some(date) = *self.selected_date.borrow() {
                let info = format!("Selected: {} (week {})", date, date.format("%U"));
                for (i, c) in info.chars().enumerate() {
                    let idx = (info_y * area.width + half / 2 - info.len() as u16 / 2 + i as u16)
                        as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = self.theme.success;
                    }
                }
            }
        } else {
            if let Some((start, end)) = *self.selected_range.borrow() {
                let info = format!("Range: {} to {}", start, end);
                for (i, c) in info.chars().enumerate() {
                    let idx =
                        (info_y * area.width + half + (half - info.len() as u16) / 2 + i as u16)
                            as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = self.theme.success;
                    }
                }
            }
        }

        // Instructions panel
        let inst_y = area.height.saturating_sub(2);
        let kb_theme = self.keybindings.display(actions::THEME).unwrap_or("Ctrl+T");
        let kb_help = self.keybindings.display(actions::HELP).unwrap_or("F1");
        let kb_quit = self.keybindings.display(actions::QUIT).unwrap_or("Ctrl+Q");
        let instructions = format!("</>: navigate months | Enter: confirm | C: clear | {kb_theme}: toggle mode | {kb_help}: help | {kb_quit}: quit");
        for (i, c) in instructions.chars().enumerate() {
            let idx = (inst_y * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
                plane.cells[idx].bg = self.theme.surface;
                plane.cells[idx].transparent = false;
            }
        }

        // Divider line
        let div_x = half;
        for y in 0..area.height.saturating_sub(1) {
            let idx = (y * area.width + div_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = self.theme.outline;
            }
        }

        // Help overlay
        if self.show_help {
            let t = &self.theme;
            let hw = 44u16.min(area.width.saturating_sub(4));
            let hh = 11u16.min(area.height.saturating_sub(4));
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
            let corners = [
                ('╭', hx, hy),
                ('╮', hx + hw - 1, hy),
                ('╰', hx, hy + hh - 1),
                ('╯', hx + hw - 1, hy + hh - 1),
            ];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = *ch;
                    plane.cells[idx].fg = t.outline;
                }
            }
            for x in hx + 1..hx + hw - 1 {
                let top = (hy * area.width + x) as usize;
                let bot = ((hy + hh - 1) * area.width + x) as usize;
                if top < plane.cells.len() {
                    plane.cells[top].char = '─';
                    plane.cells[top].fg = t.outline;
                }
                if bot < plane.cells.len() {
                    plane.cells[bot].char = '─';
                    plane.cells[bot].fg = t.outline;
                }
            }
            for y in hy + 1..hy + hh - 1 {
                let left = (y * area.width + hx) as usize;
                let right = (y * area.width + hx + hw - 1) as usize;
                if left < plane.cells.len() {
                    plane.cells[left].char = '│';
                    plane.cells[left].fg = t.outline;
                }
                if right < plane.cells.len() {
                    plane.cells[right].char = '│';
                    plane.cells[right].fg = t.outline;
                }
            }
            let title = "Calendar Help";
            let tx = hx + (hw - title.len() as u16) / 2;
            for (i, c) in title.chars().enumerate() {
                let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }
            let kb_back = self.keybindings.display(actions::BACK).unwrap_or("Esc");
            let shortcuts: [(&str, &str); 6] = [
                ("</>", "Navigate months"),
                ("Enter", "Confirm selection"),
                ("C", "Clear selection"),
                (kb_theme, "Toggle mode"),
                (kb_help, "Toggle help"),
                (kb_back, "Dismiss help"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * area.width + hx + 2 + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.primary;
                    }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * area.width + hx + 16 + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.fg;
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.keybindings.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return false;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            self.toggle_mode();
            return true;
        }
        let handled = self.active_calendar_mut().handle_key(key);
        if handled {
            self.dirty = true;
        }
        handled
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let half = self.area.width / 2;

        // Determine which calendar to route to based on click position
        if col < half {
            // Left side - single selection
            if !self.range_mode {
                let rel_col = col.saturating_sub((half - 25) / 2);
                let rel_row = row.saturating_sub((self.area.height - 10) / 2);
                let handled = self.calendar_single.handle_mouse(kind, rel_col, rel_row);
                if handled {
                    self.dirty = true;
                }
                handled
            } else {
                false
            }
        } else {
            // Right side - range selection
            if self.range_mode {
                let rel_col = col.saturating_sub(half + (half - 25) / 2);
                let rel_row = row.saturating_sub((self.area.height - 10) / 2);
                let handled = self.calendar_range.handle_mouse(kind, rel_col, rel_row);
                if handled {
                    self.dirty = true;
                }
                handled
            } else {
                false
            }
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.calendar_single.on_theme_change(theme);
        self.calendar_range.on_theme_change(theme);
        self.dirty = true;
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Rc::new(AtomicBool::new(false));
    let theme = Theme::from_env_or(Theme::nord());
    let demo = CalendarDemo::new(Rc::clone(&should_quit), theme.clone());

    let mut app = App::new()?;
    app.add_widget(Box::new(demo), Rect::new(0, 0, 80, 24));

    let q = should_quit;
    app.title("Calendar Demo")
        .fps(30)
        .theme(theme)
        .run(move |ctx| {
            if q.load(Ordering::SeqCst) {
                ctx.stop();
            }
        })
}

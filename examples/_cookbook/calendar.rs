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
    /// Single-selection mode calendar
    calendar_single: Calendar,
    /// Range-selection mode calendar
    calendar_range: Calendar,
    /// Current mode: true = range, false = single
    range_mode: bool,
    /// Last selected date (single mode)
    selected_date: RefCell<Option<NaiveDate>>,
    /// Last selected range
    selected_range: RefCell<Option<(NaiveDate, NaiveDate)>>,
    dirty: bool,
}

impl CalendarDemo {
    fn new(should_quit: Rc<AtomicBool>, theme: Theme) -> Self {
        let selected_date = RefCell::new(None);
        let selected_range = RefCell::new(None);

        // Single selection calendar
        let sd = selected_date.clone();
        let mut cal_single = Calendar::new();
        cal_single = cal_single
            .with_theme(theme)
            .on_select(move |date| {
                *sd.borrow_mut() = Some(date);
            });

        // Range selection calendar
        let sr = selected_range.clone();
        let mut cal_range = Calendar::new();
        cal_range = cal_range
            .with_theme(theme)
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
            dirty: true,
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
        self.active_calendar_mut().on_theme_change(&self.theme);
        self.dirty = true;
    }
}

impl Widget for CalendarDemo {
    fn needs_render(&self) -> bool {
        self.dirty
            || self.calendar_single.needs_render()
            || self.calendar_range.needs_render()
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
        self.calendar_single.set_area(Rect::new(0, 0, half, area.height));
        self.calendar_range.set_area(Rect::new(half, 0, half, area.height));
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        let half = area.width / 2;

        // Render active calendar (centered in its half)
        let cal = self.active_calendar();
        let cal_area = cal.area();
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
                    let idx = (info_y * area.width + half + (half.len() - info.len() as u16) / 2 + i as u16)
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
        let instructions = "←/→: navigate months | Enter: confirm | C: clear | T: toggle mode | Ctrl+Q: quit";
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

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        use dracon_terminal_engine::input::event::KeyCode;

        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.toggle_mode();
                true
            }
            _ => {
                let handled = self.active_calendar_mut().handle_key(key);
                if handled {
                    self.dirty = true;
                }
                handled
            }
        }
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
        self.theme = *theme;
        self.calendar_single.on_theme_change(theme);
        self.calendar_range.on_theme_change(theme);
        self.dirty = true;
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Rc::new(AtomicBool::new(false));
    let theme = Theme::from_env_or(Theme::nord());
    let demo = CalendarDemo::new(Rc::clone(&should_quit), theme);

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

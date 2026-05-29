//! Calendar/Date picker widget with month navigation and date range selection.

use std::cell::RefCell;

use chrono::{Datelike, Local, NaiveDate};

use crate::compositor::{Plane, Styles};
use crate::framework::hitzone::ScopedZoneRegistry;
use crate::framework::theme::Theme;
use crate::framework::widget::{WidgetId, WidgetState};
use ratatui::layout::Rect;

/// Callback type for single date selection.
pub type DateSelectCallback = Box<dyn FnMut(NaiveDate)>;

/// Callback type for date range selection (start, end).
pub type DateRangeSelectCallback = Box<dyn FnMut(NaiveDate, NaiveDate)>;

/// Zone IDs for mouse interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZoneId {
    PrevMonth,
    NextMonth,
    Day(usize), // Index 0-41 for the 6-week grid
}

/// Calendar widget for date selection with month navigation.
pub struct Calendar {
    id: WidgetId,
    /// Currently displayed month (1-12).
    month: u8,
    /// Currently displayed year.
    year: i32,
    /// Selected date (single selection mode).
    selected: Option<NaiveDate>,
    /// Range selection: start date.
    range_start: Option<NaiveDate>,
    /// Range selection: end date.
    range_end: Option<NaiveDate>,
    /// Currently hovered day index (0-41).
    hovered_day: Option<usize>,
    /// Whether range mode is active.
    range_mode: bool,
    theme: Theme,
    on_select: Option<DateSelectCallback>,
    on_range_select: Option<DateRangeSelectCallback>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    /// Hit zones registered each frame (using RefCell for interior mutability).
    zones: RefCell<ScopedZoneRegistry<ZoneId>>,
}

impl Calendar {
    /// Creates a new `Calendar` widget displaying the current month.
    pub fn new() -> Self {
        let today = Local::now().date_naive();
        Self {
            id: WidgetId::next(),
            month: today.month() as u8,
            year: today.year(),
            selected: None,
            range_start: None,
            range_end: None,
            hovered_day: None,
            range_mode: false,
            theme: Theme::default(),
            on_select: None,
            on_range_select: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 25, 10)),
            dirty: true,
            zones: RefCell::new(ScopedZoneRegistry::new()),
        }
    }

    /// Creates a new `Calendar` with a specific widget ID.
    pub fn with_id(id: WidgetId) -> Self {
        let today = Local::now().date_naive();
        Self {
            id,
            month: today.month() as u8,
            year: today.year(),
            selected: None,
            range_start: None,
            range_end: None,
            hovered_day: None,
            range_mode: false,
            theme: Theme::default(),
            on_select: None,
            on_range_select: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 25, 10)),
            dirty: true,
            zones: RefCell::new(ScopedZoneRegistry::new()),
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Enables range selection mode (for check-in/check-out style selection).
    pub fn with_range_mode(mut self) -> Self {
        self.range_mode = true;
        self
    }

    /// Registers a callback invoked when a date is selected.
    pub fn on_select(mut self, f: impl FnMut(NaiveDate) + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when a date range is selected (start and end dates chosen).
    pub fn on_range_select(mut self, f: impl FnMut(NaiveDate, NaiveDate) + 'static) -> Self {
        self.on_range_select = Some(Box::new(f));
        self
    }

    /// Returns the currently selected date, if any.
    pub fn selected(&self) -> Option<NaiveDate> {
        self.selected
    }

    /// Returns the range start date, if set.
    pub fn range_start(&self) -> Option<NaiveDate> {
        self.range_start
    }

    /// Returns the range end date, if set.
    pub fn range_end(&self) -> Option<NaiveDate> {
        self.range_end
    }

    /// Sets the displayed month and year.
    pub fn set_month(&mut self, month: u8, year: i32) {
        self.month = month.clamp(1, 12);
        self.year = year;
        self.dirty = true;
    }

    /// Returns the first day of the displayed month.
    fn first_day_of_month(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month as u32, 1).unwrap_or_else(|| {
            NaiveDate::from_ymd_opt(2024, 1, 1).expect("hardcoded date 2024-01-01 is always valid")
        })
    }

    /// Returns the days in the current month.
    fn days_in_month(&self) -> u32 {
        let next_month = if self.month == 12 {
            NaiveDate::from_ymd_opt(self.year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(self.year, self.month as u32 + 1, 1)
        };
        let next = next_month.unwrap_or(NaiveDate::MAX);
        next.pred_opt().map(|d| d.day()).unwrap_or(30)
    }

    /// Calculates the offset to start the calendar grid.
    /// Monday = 0, Sunday = 6 (ISO week).
    fn start_offset(&self) -> u8 {
        let first = self.first_day_of_month();
        // Monday = 0 (chrono: Monday = 1), Sunday = 6 (chrono: Sunday = 7)
        let weekday = first.weekday().num_days_from_monday();
        weekday as u8
    }

    /// Returns the date for a given grid index (0-41), or None if invalid.
    fn date_for_index(&self, index: usize) -> Option<NaiveDate> {
        if index > 41 {
            return None;
        }
        let offset = self.start_offset() as usize;
        if index < offset {
            return None;
        }
        let day = index - offset + 1;
        let days_in_month = self.days_in_month() as usize;
        if day > days_in_month {
            return None;
        }
        NaiveDate::from_ymd_opt(self.year, self.month as u32, day as u32)
    }

    /// Navigates to the previous month.
    fn prev_month(&mut self) {
        if self.month == 1 {
            self.month = 12;
            self.year -= 1;
        } else {
            self.month -= 1;
        }
        self.dirty = true;
    }

    /// Navigates to the next month.
    fn next_month(&mut self) {
        if self.month == 12 {
            self.month = 1;
            self.year += 1;
        } else {
            self.month += 1;
        }
        self.dirty = true;
    }

    /// Selects a date based on current mode.
    fn select_date(&mut self, date: NaiveDate) {
        if self.range_mode {
            if self.range_start.is_none() {
                self.range_start = Some(date);
                self.range_end = None;
            } else if self.range_end.is_none() {
                // Ensure start <= end
                let start = self.range_start.unwrap_or(date);
                if date >= start {
                    self.range_end = Some(date);
                } else {
                    self.range_end = Some(start);
                    self.range_start = Some(date);
                }
                if let (Some(s), Some(e)) = (self.range_start, self.range_end) {
                    if let Some(ref mut cb) = self.on_range_select {
                        cb(s, e);
                    }
                }
            } else {
                // Reset and start new range
                self.range_start = Some(date);
                self.range_end = None;
            }
        } else {
            self.selected = Some(date);
            if let Some(ref mut cb) = self.on_select {
                cb(date);
            }
        }
        self.dirty = true;
    }

    /// Checks if a date is within the selected range.
    fn is_in_range(&self, date: NaiveDate) -> bool {
        if let (Some(start), Some(end)) = (self.range_start, self.range_end) {
            date >= start && date <= end
        } else {
            false
        }
    }

    /// Checks if a date is the start of the range.
    fn is_range_start(&self, date: NaiveDate) -> bool {
        self.range_start == Some(date)
    }

    /// Checks if a date is the end of the range.
    fn is_range_end(&self, date: NaiveDate) -> bool {
        self.range_end == Some(date)
    }

    /// Clears all selections.
    pub fn clear_selection(&mut self) {
        self.selected = None;
        self.range_start = None;
        self.range_end = None;
        self.dirty = true;
    }

    /// Returns the current month name.
    fn month_name(&self) -> &'static str {
        match self.month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        }
    }
}

impl Default for Calendar {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::framework::widget::Widget for Calendar {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
        self.dirty = true;
    }

    fn z_index(&self) -> u16 {
        10
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

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;
        plane.fill_bg(self.theme.bg);

        // Clear hit zones and register new ones
        self.zones.borrow_mut().clear();

        let today = Local::now().date_naive();
        let cal_width = 20u16;
        let cal_x = (area.width.saturating_sub(cal_width)) / 2;
        let cal_y = 1u16;

        // === HEADER ===
        // Previous month button (<)
        if cal_x > 0 {
            let idx = (cal_y * area.width + cal_x.saturating_sub(2)) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '<';
                plane.cells[idx].fg = self.theme.primary;
                plane.cells[idx].style = Styles::empty();
            }
            self.zones.borrow_mut().register(
                ZoneId::PrevMonth,
                cal_x.saturating_sub(2),
                cal_y,
                1,
                1,
            );
        }

        // Month/Year title
        let title = format!("{} {}", self.month_name(), self.year);
        let title_x = cal_x + (cal_width.saturating_sub(title.len() as u16)) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = ((cal_y) * area.width + title_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Next month button (>)
        let next_btn_x = cal_x + cal_width + 1;
        if next_btn_x < area.width {
            let idx = (cal_y * area.width + next_btn_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '>';
                plane.cells[idx].fg = self.theme.primary;
                plane.cells[idx].style = Styles::empty();
            }
            self.zones
                .borrow_mut()
                .register(ZoneId::NextMonth, next_btn_x, cal_y, 1, 1);
        }

        // === DAY HEADERS (Mo Tu We Th Fr Sa Su) ===
        let day_headers = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
        let header_y = cal_y + 2;
        for (i, header) in day_headers.iter().enumerate() {
            let hx = cal_x + (i as u16) * 3;
            for (j, c) in header.chars().enumerate() {
                let idx = (header_y * area.width + hx + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = self.theme.fg_muted;
                    plane.cells[idx].style = Styles::empty();
                }
            }
        }

        // === DAY GRID ===
        let grid_start_y = header_y + 1;
        let offset = self.start_offset() as usize;
        let days_in_month = self.days_in_month() as usize;

        for week in 0..6u8 {
            for day_of_week in 0..7u8 {
                let cell_index = (week as usize) * 7 + (day_of_week as usize);
                let day_num = cell_index.saturating_sub(offset) + 1;

                let cell_x = cal_x + (day_of_week as u16) * 3;
                let cell_y = grid_start_y + week as u16;

                if cell_index >= offset && day_num <= days_in_month {
                    let date =
                        NaiveDate::from_ymd_opt(self.year, self.month as u32, day_num as u32)
                            .unwrap_or_else(|| {
                                NaiveDate::from_ymd_opt(2000, 1, 1).unwrap_or(NaiveDate::MIN)
                            });
                    let day_str = format!("{:>2}", day_num);

                    // Determine cell styling
                    let is_today = date == today;
                    let is_selected = self.selected == Some(date);
                    let is_in_range = self.is_in_range(date);
                    let is_range_start = self.is_range_start(date);
                    let is_range_end = self.is_range_end(date);
                    let is_hovered = self.hovered_day == Some(cell_index);

                    let (fg, bg, style) = if is_selected || is_range_start || is_range_end {
                        (
                            self.theme.selection_fg,
                            self.theme.selection_bg,
                            Styles::BOLD,
                        )
                    } else if is_in_range {
                        (self.theme.fg, self.theme.info_bg, Styles::empty())
                    } else if is_today {
                        (self.theme.primary, self.theme.bg, Styles::BOLD)
                    } else if is_hovered {
                        (self.theme.fg, self.theme.hover_bg, Styles::empty())
                    } else {
                        (self.theme.fg, self.theme.bg, Styles::empty())
                    };

                    // Draw day number
                    for (j, c) in day_str.chars().enumerate() {
                        let idx = (cell_y * area.width + cell_x + j as u16) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].char = c;
                            plane.cells[idx].fg = fg;
                            plane.cells[idx].bg = bg;
                            plane.cells[idx].style = style;
                        }
                    }

                    // Register hit zone for this day
                    self.zones
                        .borrow_mut()
                        .register(ZoneId::Day(cell_index), cell_x, cell_y, 2, 1);
                } else {
                    // Empty cell
                    for j in 0..2u16 {
                        let idx = (cell_y * area.width + cell_x + j) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].char = ' ';
                            plane.cells[idx].fg = self.theme.fg_subtle;
                            plane.cells[idx].bg = self.theme.bg;
                        }
                    }
                }
            }
        }

        // === STATUS LINE ===
        let status_y = grid_start_y + 6;
        if status_y < area.height {
            let status = if self.range_mode {
                match (&self.range_start, &self.range_end) {
                    (Some(start), Some(end)) => {
                        format!("Range: {} to {} (click to reset)", start, end)
                    }
                    (Some(start), None) => format!("Select end: {} — ?", start),
                    _ => "Select check-in date".to_string(),
                }
            } else {
                match self.selected {
                    Some(date) => format!("Selected: {} (click to change)", date),
                    None => "Click a date to select".to_string(),
                }
            };

            let sx = cal_x;
            for (i, c) in status.chars().take(cal_width as usize).enumerate() {
                let idx = (status_y * area.width + sx + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = self.theme.fg_muted;
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }

        match key.code {
            KeyCode::Left => {
                self.prev_month();
                true
            }
            KeyCode::Right => {
                self.next_month();
                true
            }
            KeyCode::Char('r') => {
                // Reset range selection in range mode
                if self.range_mode {
                    self.range_start = None;
                    self.range_end = None;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Char('c') | KeyCode::Backspace => {
                // Clear selection
                self.clear_selection();
                true
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Confirm hovered day or today's date
                let today = Local::now().date_naive();
                if let Some(idx) = self.hovered_day {
                    if let Some(date) = self.date_for_index(idx) {
                        self.select_date(date);
                    }
                } else {
                    // Try to select today
                    if let Some(today_date) =
                        NaiveDate::from_ymd_opt(today.year(), today.month(), today.day())
                    {
                        self.select_date(today_date);
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        use crate::input::event::MouseEventKind;

        // Dispatch to hit zones - clone the zone_id to avoid borrow conflicts
        let zone_id = self.zones.borrow().dispatch(col, row);
        if let Some(zone_id) = zone_id {
            // Click on calendar
            match kind {
                MouseEventKind::Moved => {
                    if let ZoneId::Day(idx) = zone_id {
                        if self.hovered_day != Some(idx) {
                            self.hovered_day = Some(idx);
                            self.dirty = true;
                        }
                    } else if self.hovered_day.is_some() {
                        self.hovered_day = None;
                        self.dirty = true;
                    }
                    true
                }
                MouseEventKind::Down(crate::input::event::MouseButton::Left) => match zone_id {
                    ZoneId::PrevMonth => {
                        self.prev_month();
                        true
                    }
                    ZoneId::NextMonth => {
                        self.next_month();
                        true
                    }
                    ZoneId::Day(idx) => {
                        if let Some(date) = self.date_for_index(idx) {
                            self.select_date(date);
                        }
                        true
                    }
                },
                _ => true,
            }
        } else {
            // Click outside calendar
            match kind {
                MouseEventKind::Moved => {
                    if self.hovered_day.is_some() {
                        self.hovered_day = None;
                        self.dirty = true;
                    }
                    false
                }
                _ => false,
            }
        }
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = theme.clone();
        self.dirty = true;
    }
}

impl WidgetState for Calendar {
    fn state_id(&self) -> Option<&str> {
        Some("calendar")
    }

    fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        let selected_date = self.selected.map(|d| d.to_string());
        json!({
            "selected_date": selected_date,
            "month": self.month,
            "year": self.year,
        })
    }

    fn apply_json(&mut self, json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        if let Some(date_str) = json.get("selected_date").and_then(|v| v.as_str()) {
            self.selected = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok();
        }
        if let Some(month) = json.get("month").and_then(|v| v.as_u64()) {
            self.month = month as u8;
        }
        if let Some(year) = json.get("year").and_then(|v| v.as_i64()) {
            self.year = year as i32;
        }
        self.dirty = true;
        Ok(())
    }
}

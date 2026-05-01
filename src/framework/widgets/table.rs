//! Sortable, selectable table widget with header and row hit zones.

use std::cell::Cell;

use crate::compositor::{Plane, Styles};
use crate::framework::hitzone::HitZone;
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A column definition for a `Table`.
pub struct Column {
    /// Header label displayed in the column.
    pub header: String,
    /// Width in cells.
    pub width: u16,
}

/// A single row of data paired with its rendered cell strings.
#[derive(Clone)]
pub struct TableRow<T> {
    /// The underlying row data.
    pub data: T,
}

/// A sortable, selectable table with header and row hit zones.
pub struct Table<T> {
    id: WidgetId,
    columns: Vec<Column>,
    rows: Vec<TableRow<T>>,
    selected: usize,
    offset: usize,
    visible_count: usize,
    theme: Theme,
    on_select: Option<Box<dyn FnMut(&T)>>,
    area: Cell<Rect>,
    dirty: bool,
}

impl<T: Clone + ToString> Table<T> {
    /// Creates a new `Table` with the given column definitions.
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            id: WidgetId::default_id(),
            columns,
            rows: Vec::new(),
            selected: 0,
            offset: 0,
            visible_count: 10,
            theme: Theme::default(),
            on_select: None,
            area: Cell::new(Rect::new(0, 0, 80, 20)),
            dirty: true,
        }
    }

    /// Creates a new `Table` with the given widget ID and column definitions.
    pub fn new_with_id(id: WidgetId, columns: Vec<Column>) -> Self {
        Self {
            id,
            columns,
            rows: Vec::new(),
            selected: 0,
            offset: 0,
            visible_count: 10,
            theme: Theme::default(),
            on_select: None,
            area: Cell::new(Rect::new(0, 0, 80, 20)),
            dirty: true,
        }
    }

    /// Sets the theme for rendering.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Populates the table with row data. Each item is mapped to a `TableRow`.
    pub fn with_rows(mut self, rows: Vec<T>) -> Self
    where
        T: 'static,
    {
        self.rows = rows
            .into_iter()
            .map(|data| TableRow { data })
            .collect();
        self
    }

    /// Registers a callback invoked when a row is selected (Enter or click).
    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Returns the index of the currently selected row.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Returns a reference to the selected row's data, or `None`.
    pub fn get_selected(&self) -> Option<&T> {
        self.rows.get(self.selected).map(|r| &r.data)
    }

    /// Returns the number of rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Returns true if there are no rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Returns `(start, end)` indices of the currently visible rows.
    pub fn viewport(&self) -> (usize, usize) {
        let start = self.offset;
        let end = (self.offset + self.visible_count).min(self.rows.len());
        (start, end)
    }

    /// Scrolls to and selects the row at `index`.
    pub fn scroll_to(&mut self, index: usize) {
        if index >= self.rows.len() {
            return;
        }
        self.selected = index;
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + self.visible_count {
            self.offset = self.selected.saturating_sub(self.visible_count) + 1;
        }
    }

    /// Sets how many rows are visible at once.
    pub fn set_visible_count(&mut self, count: usize) {
        self.visible_count = count;
    }

    fn cell_text(&self, row: &TableRow<T>, _col: usize) -> String {
        row.data.to_string()
    }
}

impl<T: Clone + ToString> crate::framework::widget::Widget for Table<T> {
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

        let row_height: u16 = 1;
        let mut x: u16 = 0;

        for (i, col) in self.columns.iter().enumerate() {
            let w = col.width.min(area.width.saturating_sub(x));

            for col_idx in 0..w {
                let idx = col_idx as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = self.theme.active_bg;
                    plane.cells[idx].fg = self.theme.fg;
                    plane.cells[idx].char = ' ';
                }
            }

            let label_len = col.header.len().min(w as usize - 2);
            let start = (w.saturating_sub(label_len as u16)) / 2;
            for (j, ch) in col.header.chars().take(label_len).enumerate() {
                let idx = start as usize + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = self.theme.fg;
                    plane.cells[idx].style = Styles::empty();
                }
            }

            x += w;
            let _ = i;
        }

        let visible_rows: Vec<_> = self.rows.iter()
            .skip(self.offset)
            .take(self.visible_count)
            .collect();

        for (i, row) in visible_rows.iter().enumerate() {
            let y = 1 + i;
            let is_selected = self.offset + i == self.selected;
            let bg = if is_selected { self.theme.selection_bg } else { self.theme.bg };
            let fg = if is_selected { self.theme.selection_fg } else { self.theme.fg };

            let y_off = 1u16 + i as u16;
            for (j, col) in self.columns.iter().enumerate() {
                let w = col.width.min(area.width.saturating_sub(x));
                let _hit_zone = HitZone::new(self.offset + i, x, area.y + y_off, w, row_height);

                for col_idx in 0..w {
                    let idx = y * area.width as usize + x as usize + col_idx as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = bg;
                        plane.cells[idx].fg = fg;
                        plane.cells[idx].char = ' ';
                    }
                }

                let text = self.cell_text(row, j);
                let label_len = text.len().min(w as usize - 1).saturating_sub(1);
                for (k, ch) in text.chars().take(label_len).enumerate() {
                    let idx = y * area.width as usize + x as usize + 1 + k;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = fg;
                        plane.cells[idx].style = if is_selected { Styles::BOLD } else { Styles::empty() };
                    }
                }

                x += w;
                let _ = j;
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
            KeyCode::Down => {
                if self.selected + 1 < self.rows.len() {
                    self.selected += 1;
                    if self.selected >= self.offset + self.visible_count {
                        self.offset = self.selected.saturating_sub(self.visible_count) + 1;
                    }
                    self.dirty = true;
                }
                true
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    if self.selected < self.offset {
                        self.offset = self.selected;
                    }
                    self.dirty = true;
                }
                true
            }
            KeyCode::Home => {
                self.selected = 0;
                self.offset = 0;
                self.dirty = true;
                true
            }
            KeyCode::End => {
                self.selected = self.rows.len().saturating_sub(1);
                self.offset = self.rows.len().saturating_sub(self.visible_count);
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                if let Some(f) = self.on_select.as_mut() {
                    f(&self.rows[self.selected].data);
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, _col: u16, row: u16) -> bool {
        if row == 0 {
            return false;
        }

        let rel_row = row.saturating_sub(1);
        if rel_row >= self.visible_count as u16 {
            return false;
        }

        let idx = self.offset + rel_row as usize;
        if idx >= self.rows.len() {
            return false;
        }

        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                self.selected = idx;
                if let Some(f) = self.on_select.as_mut() {
                    f(&self.rows[idx].data);
                }
                self.dirty = true;
                true
            }
            crate::input::event::MouseEventKind::ScrollDown => {
                self.offset = (self.offset + 1).min(self.rows.len().saturating_sub(self.visible_count));
                self.dirty = true;
                true
            }
            crate::input::event::MouseEventKind::ScrollUp => {
                self.offset = self.offset.saturating_sub(1);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }
}
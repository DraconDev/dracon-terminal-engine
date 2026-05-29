//! Sortable, selectable table widget with header and row hit zones.

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashSet;

use crate::compositor::{Plane, Styles};
use crate::framework::dragdrop::DragManager;
use crate::framework::theme::Theme;
use crate::framework::widget::{WidgetId, WidgetState};
use crate::framework::widgets::context_menu::ContextMenu;
use crate::framework::widgets::list_common::{render_scroll_indicator, ListNavigation};
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

pub type SelectCallback<T> = Box<dyn FnMut(&T)>;
pub type CellTextFn<T> = Box<dyn Fn(&T, usize) -> String>;
pub type HeaderClickCallback = Box<dyn FnMut(usize)>;
pub type SelectionChangeCallback = Box<dyn FnMut(&HashSet<usize>)>;
pub type UndoRedoCallback = Box<dyn FnMut()>;

/// Inner state snapshot for undo/redo.
#[derive(Clone)]
pub struct TableState {
    pub selected: usize,
    pub offset: usize,
    pub sort_column: Option<usize>,
    pub sort_ascending: bool,
    pub selected_indices: HashSet<usize>,
}

/// A sortable, selectable table with header and row hit zones.
pub struct Table<T> {
    id: WidgetId,
    columns: Vec<Column>,
    rows: Vec<TableRow<T>>,
    theme: Theme,
    on_select: Option<SelectCallback<T>>,
    on_selection_change: Option<SelectionChangeCallback>,
    on_undo: Option<UndoRedoCallback>,
    on_redo: Option<UndoRedoCallback>,
    cell_text_fn: Option<CellTextFn<T>>,
    on_header_click: Option<HeaderClickCallback>,
    sort_column: Option<usize>,
    sort_ascending: bool,
    area: Cell<Rect>,
    dirty: bool,
    // Drag and drop
    drag_manager: RefCell<DragManager<usize>>,
    // Context menu
    context_menu: RefCell<Option<ContextMenu>>,
    // Shared navigation state
    nav: ListNavigation<TableState>,
}

impl<T: Clone + ToString> Table<T> {
    /// Creates a new `Table` with the given column definitions.
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            id: WidgetId::next(),
            columns,
            rows: Vec::new(),
            theme: Theme::default(),
            on_select: None,
            on_selection_change: None,
            on_undo: None,
            on_redo: None,
            cell_text_fn: None,
            on_header_click: None,
            sort_column: None,
            sort_ascending: true,
            area: Cell::new(Rect::new(0, 0, 80, 20)),
            dirty: true,
            drag_manager: RefCell::new(DragManager::new()),
            context_menu: RefCell::new(None),
            nav: ListNavigation::new(),
        }
    }

    /// Creates a new `Table` with the given widget ID and column definitions.
    pub fn new_with_id(id: WidgetId, columns: Vec<Column>) -> Self {
        Self {
            id,
            columns,
            rows: Vec::new(),
            theme: Theme::default(),
            on_select: None,
            on_selection_change: None,
            on_undo: None,
            on_redo: None,
            cell_text_fn: None,
            on_header_click: None,
            sort_column: None,
            sort_ascending: true,
            area: Cell::new(Rect::new(0, 0, 80, 20)),
            dirty: true,
            drag_manager: RefCell::new(DragManager::new()),
            context_menu: RefCell::new(None),
            nav: ListNavigation::new(),
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
        self.rows = rows.into_iter().map(|data| TableRow { data }).collect();
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

    /// Sets a per-column cell text formatter.
    /// The callback receives `(&row_data, column_index)` and returns the cell text.
    pub fn with_cell_text_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&T, usize) -> String + 'static,
    {
        self.cell_text_fn = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when a column header is clicked.
    pub fn on_header_click<F>(mut self, f: F) -> Self
    where
        F: FnMut(usize) + 'static,
    {
        self.on_header_click = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when the selection changes (including multi-select).
    pub fn on_selection_change(mut self, f: impl FnMut(&HashSet<usize>) + 'static) -> Self {
        self.on_selection_change = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when undo is triggered (Ctrl+Z).
    pub fn on_undo(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_undo = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when redo is triggered (Ctrl+Y).
    pub fn on_redo(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_redo = Some(Box::new(f));
        self
    }

    /// Enables multi-select mode.
    pub fn with_multi_select(mut self, enabled: bool) -> Self {
        self.nav.allow_multi_select = enabled;
        self
    }

    /// Enables undo/redo support.
    pub fn with_undo(mut self, enabled: bool) -> Self {
        self.nav.enable_undo = enabled;
        self
    }

    /// Sets a context menu to show on right-click.
    pub fn with_context_menu(mut self, menu: ContextMenu) -> Self {
        self.context_menu = RefCell::new(Some(menu));
        self
    }

    /// Returns the set of selected row indices in multi-select mode.
    pub fn selected_indices(&self) -> &HashSet<usize> {
        &self.nav.selected_indices
    }

    /// Clears the current selection.
    pub fn clear_selection(&mut self) {
        if self.nav.clear_selection() {
            self.dirty = true;
        }
    }

    /// Returns the drag manager for this table.
    pub fn drag_manager(&self) -> &RefCell<DragManager<usize>> {
        &self.drag_manager
    }

    /// Takes a snapshot of the current state for undo/redo.
    fn snapshot(&self) -> TableState {
        TableState {
            selected: self.nav.selected,
            offset: self.nav.offset,
            sort_column: self.sort_column,
            sort_ascending: self.sort_ascending,
            selected_indices: self.nav.selected_indices.clone(),
        }
    }

    /// Undo last operation.
    fn undo(&mut self) {
        if let Some(state) = self.nav.undo(self.snapshot()) {
            self.apply_state(&state);
            if let Some(ref mut cb) = self.on_undo {
                cb();
            }
        }
    }

    /// Redo last undone operation.
    fn redo(&mut self) {
        if let Some(state) = self.nav.redo(self.snapshot()) {
            self.apply_state(&state);
            if let Some(ref mut cb) = self.on_redo {
                cb();
            }
        }
    }

    /// Applies a state snapshot to the table.
    fn apply_state(&mut self, state: &TableState) {
        self.nav.selected = state.selected.min(self.rows.len().saturating_sub(1));
        self.nav.offset = state.offset;
        self.sort_column = state.sort_column;
        self.sort_ascending = state.sort_ascending;
        self.nav.selected_indices = state.selected_indices.clone();
        self.nav.clamp_scroll();
        self.dirty = true;
    }

    /// Sets the active sort column and direction for rendering indicators.
    pub fn set_sort(&mut self, column: Option<usize>, ascending: bool) {
        self.sort_column = column;
        self.sort_ascending = ascending;
        self.dirty = true;
    }

    /// Returns the index of the currently selected row.
    pub fn selected_index(&self) -> usize {
        self.nav.selected
    }

    /// Returns a reference to the selected row's data, or `None`.
    pub fn get_selected(&self) -> Option<&T> {
        self.rows.get(self.nav.selected).map(|r| &r.data)
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
        let start = self.nav.offset;
        let end = (self.nav.offset + self.nav.visible_count).min(self.rows.len());
        (start, end)
    }

    /// Scrolls to and selects the row at `index`.
    pub fn scroll_to(&mut self, index: usize) {
        if index >= self.rows.len() {
            return;
        }
        self.nav.selected = index;
        if self.nav.selected < self.nav.offset {
            self.nav.offset = self.nav.selected;
        } else if self.nav.selected >= self.nav.offset + self.nav.visible_count {
            self.nav.offset = self.nav.selected.saturating_sub(self.nav.visible_count) + 1;
        }
    }

    /// Sets how many rows are visible at once.
    pub fn set_visible_count(&mut self, count: usize) {
        self.nav.visible_count = count;
    }

    fn cell_text(&self, row: &TableRow<T>, col: usize) -> String {
        if let Some(ref f) = self.cell_text_fn {
            f(&row.data, col)
        } else {
            row.data.to_string()
        }
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
        plane.fill_bg(self.theme.bg);

        let mut x: u16 = 0;

        for (i, col) in self.columns.iter().enumerate() {
            let w = col.width.min(area.width.saturating_sub(x));

            for col_idx in 0..w {
                let idx = col_idx as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = self.theme.surface;
                    plane.cells[idx].fg = self.theme.fg;
                    plane.cells[idx].char = ' ';
                }
            }

            let label = if self.sort_column == Some(i) {
                if self.sort_ascending {
                    format!("{} ▲", col.header)
                } else {
                    format!("{} ▼", col.header)
                }
            } else {
                col.header.clone()
            };
            let label_len = label.len().min(w as usize - 2);
            let start = (w.saturating_sub(label_len as u16)) / 2;
            for (j, ch) in label.chars().take(label_len).enumerate() {
                let idx = start as usize + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = if self.sort_column == Some(i) {
                        self.theme.primary
                    } else {
                        self.theme.fg
                    };
                    plane.cells[idx].style = if self.sort_column == Some(i) {
                        Styles::BOLD
                    } else {
                        Styles::empty()
                    };
                }
            }

            x += w;
            let _ = i;
        }

        let total_rows = self.rows.len();
        for i in 0..self.nav.visible_count {
            let row_idx = self.nav.offset + i;
            if row_idx >= total_rows {
                break;
            }
            let row = &self.rows[row_idx];
            let y = 1 + i;
            let is_selected = row_idx == self.nav.selected;
            let is_hovered = self.nav.hovered == Some(row_idx);
            let bg = if is_selected {
                self.theme.selection_bg
            } else if is_hovered {
                self.theme.hover_bg
            } else {
                self.theme.bg
            };
            let fg = if is_selected {
                self.theme.selection_fg
            } else {
                self.theme.fg
            };

            let mut row_x: u16 = 0;
            for (j, col) in self.columns.iter().enumerate() {
                let w = col.width.min(area.width.saturating_sub(row_x));

                for col_idx in 0..w {
                    let idx = y * area.width as usize + row_x as usize + col_idx as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = bg;
                        plane.cells[idx].fg = fg;
                        plane.cells[idx].char = ' ';
                    }
                }

                let text = self.cell_text(row, j);
                let label_len = text.len().min(w as usize - 1);
                for (k, ch) in text.chars().take(label_len).enumerate() {
                    let idx = y * area.width as usize + row_x as usize + 1 + k;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = fg;
                        plane.cells[idx].style = if is_selected {
                            Styles::BOLD
                        } else {
                            Styles::empty()
                        };
                    }
                }
                row_x += w;
                let _ = j;
            }
        }

        let total = self.rows.len();
        let visible = self.nav.visible_count;
        render_scroll_indicator(
            &mut plane,
            area,
            self.nav.offset,
            total,
            visible,
            &self.theme,
        );

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Down => {
                if self.nav.move_down(self.rows.len()) {
                    self.dirty = true;
                }
                true
            }
            KeyCode::Up => {
                if self.nav.move_up() {
                    self.dirty = true;
                }
                true
            }
            KeyCode::Home => {
                self.nav.move_home();
                self.dirty = true;
                true
            }
            KeyCode::End => {
                self.nav.move_end(self.rows.len());
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                if let Some(f) = self.on_select.as_mut() {
                    f(&self.rows[self.nav.selected].data);
                }
                true
            }
            KeyCode::Char('z')
                if key
                    .modifiers
                    .contains(crate::input::event::KeyModifiers::CONTROL) =>
            {
                self.undo();
                true
            }
            KeyCode::Char('y')
                if key
                    .modifiers
                    .contains(crate::input::event::KeyModifiers::CONTROL) =>
            {
                self.redo();
                true
            }
            KeyCode::Char('a')
                if key
                    .modifiers
                    .contains(crate::input::event::KeyModifiers::CONTROL) =>
            {
                if self.nav.allow_multi_select {
                    self.nav.push_undo(self.snapshot());
                    self.nav.select_all(self.rows.len());
                    self.dirty = true;
                    if let Some(ref mut cb) = self.on_selection_change {
                        cb(&self.nav.selected_indices);
                    }
                }
                true
            }
            KeyCode::Esc => {
                if self.nav.allow_multi_select && self.nav.clear_selection() {
                    self.dirty = true;
                    if let Some(ref mut cb) = self.on_selection_change {
                        cb(&self.nav.selected_indices);
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
        if let Some(ref mut menu) = *self.context_menu.borrow_mut() {
            if menu.is_visible() && menu.handle_mouse(kind, col, row) {
                return true;
            }
        }

        match kind {
            crate::input::event::MouseEventKind::Moved => {
                if row == 0 {
                    if self.nav.hovered.is_some() {
                        self.nav.hovered = None;
                        self.dirty = true;
                    }
                    return false;
                }
                let rel_row = row.saturating_sub(1);
                if rel_row >= self.nav.visible_count as u16 {
                    if self.nav.hovered.is_some() {
                        self.nav.hovered = None;
                        self.dirty = true;
                    }
                    return false;
                }
                let idx = self.nav.offset + rel_row as usize;
                if idx >= self.rows.len() {
                    if self.nav.hovered.is_some() {
                        self.nav.hovered = None;
                        self.dirty = true;
                    }
                    return false;
                }
                if self.nav.hovered != Some(idx) {
                    self.nav.hovered = Some(idx);
                    self.dirty = true;
                }
                true
            }
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                if row == 0 {
                    let mut col_x: u16 = 0;
                    for (i, column) in self.columns.iter().enumerate() {
                        let w = column.width;
                        if col >= col_x && col < col_x + w {
                            if let Some(f) = self.on_header_click.as_mut() {
                                f(i);
                            }
                            return true;
                        }
                        col_x += w;
                    }
                    return false;
                }
                let rel_row = row.saturating_sub(1);
                if rel_row >= self.nav.visible_count as u16 {
                    return false;
                }
                let idx = self.nav.offset + rel_row as usize;
                if idx >= self.rows.len() {
                    return false;
                }

                if self.nav.allow_multi_select {
                    self.nav.last_selected = Some(idx);
                }

                self.nav.selected = idx;
                if let Some(f) = self.on_select.as_mut() {
                    f(&self.rows[idx].data);
                }
                self.dirty = true;
                true
            }
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Right) => {
                if row == 0 {
                    return false;
                }
                let rel_row = row.saturating_sub(1);
                if rel_row >= self.nav.visible_count as u16 {
                    return false;
                }
                let idx = self.nav.offset + rel_row as usize;
                if idx >= self.rows.len() {
                    return false;
                }
                if let Some(menu) = &mut *self.context_menu.borrow_mut() {
                    menu.show();
                    let area = self.area.get();
                    menu.set_anchor(area.x + col, area.y + row);
                    self.dirty = true;
                }
                true
            }
            crate::input::event::MouseEventKind::Drag(_) => {
                if self.drag_manager.borrow().is_dragging() {
                    let area = self.area.get();
                    self.drag_manager
                        .borrow_mut()
                        .move_ghost(area.x + col, area.y + row);
                }
                true
            }
            crate::input::event::MouseEventKind::ScrollDown => {
                self.nav.scroll_down(self.rows.len());
                self.dirty = true;
                true
            }
            crate::input::event::MouseEventKind::ScrollUp => {
                self.nav.scroll_up();
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = theme.clone();
    }
}

impl<T: Clone + ToString> WidgetState for Table<T> {
    fn state_id(&self) -> Option<&str> {
        None
    }

    fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "sort_column": self.sort_column,
            "sort_ascending": self.sort_ascending,
        })
    }

    fn apply_json(&mut self, json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        if let Some(sort_col) = json.get("sort_column").and_then(|v| v.as_u64()) {
            self.sort_column = Some(sort_col as usize);
        }
        if let Some(ascending) = json.get("sort_ascending").and_then(|v| v.as_bool()) {
            self.sort_ascending = ascending;
        }
        self.dirty = true;
        Ok(())
    }
}

//! Selectable list widget with keyboard and mouse navigation.

use std::cell::RefCell;
use std::collections::HashSet;

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::dragdrop::DragManager;
use crate::framework::scroll::ScrollState;
use crate::framework::theme::Theme;
use crate::framework::widget::{WidgetId, WidgetState};
use crate::framework::widgets::context_menu::ContextMenu;
use ratatui::layout::Rect;

pub type SelectCallback<T> = Box<dyn FnMut(&T)>;
pub type SelectionChangeCallback = Box<dyn FnMut(&HashSet<usize>)>;
pub type UndoRedoCallback = Box<dyn FnMut()>;

/// Inner state snapshot for undo/redo.
#[derive(Clone)]
pub struct ListState {
    pub items: Vec<String>,
    pub selected: usize,
    pub offset: usize,
    pub selected_indices: HashSet<usize>,
}

pub struct List<T> {
    id: WidgetId,
    items: Vec<T>,
    selected: usize,
    offset: usize,
    visible_count: usize,
    theme: Theme,
    on_select: Option<SelectCallback<T>>,
    on_selection_change: Option<SelectionChangeCallback>,
    on_undo: Option<UndoRedoCallback>,
    on_redo: Option<UndoRedoCallback>,
    item_height: u16,
    width: u16,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    hovered: Option<usize>,
    // Multi-select
    allow_multi_select: bool,
    selected_indices: HashSet<usize>,
    last_selected: Option<usize>,
    // Drag and drop
    drag_manager: RefCell<DragManager<usize>>,
    // Context menu
    context_menu: RefCell<Option<ContextMenu>>,
    // Undo/redo
    enable_undo: bool,
    undo_stack: Vec<ListState>,
    redo_stack: Vec<ListState>,
}

impl<T: Clone + ToString> List<T> {
    /// Creates a new `List` with the given items and default theme.
    pub fn new(items: Vec<T>) -> Self {
        Self {
            id: WidgetId::default_id(),
            items,
            selected: 0,
            offset: 0,
            visible_count: 10,
            theme: Theme::default(),
            on_select: None,
            on_selection_change: None,
            on_undo: None,
            on_redo: None,
            item_height: 1,
            width: 40,
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 10)),
            dirty: true,
            hovered: None,
            allow_multi_select: false,
            selected_indices: HashSet::new(),
            last_selected: None,
            drag_manager: RefCell::new(DragManager::new()),
            context_menu: RefCell::new(None),
            enable_undo: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Creates a new `List` with the given widget ID and items.
    pub fn new_with_id(id: WidgetId, items: Vec<T>) -> Self {
        Self {
            id,
            items,
            selected: 0,
            offset: 0,
            visible_count: 10,
            theme: Theme::default(),
            on_select: None,
            on_selection_change: None,
            on_undo: None,
            on_redo: None,
            item_height: 1,
            width: 40,
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 10)),
            dirty: true,
            hovered: None,
            allow_multi_select: false,
            selected_indices: HashSet::new(),
            last_selected: None,
            drag_manager: RefCell::new(DragManager::new()),
            context_menu: RefCell::new(None),
            enable_undo: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Sets the theme for rendering.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the height of each item row in cells.
    pub fn with_item_height(mut self, height: u16) -> Self {
        self.item_height = height;
        self
    }

    /// Sets the width of the list in cells.
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Registers a callback invoked when an item is selected (Enter or click).
    pub fn on_select(mut self, f: impl FnMut(&T) + 'static) -> Self {
        self.on_select = Some(Box::new(f));
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
        self.allow_multi_select = enabled;
        self
    }

    /// Enables undo/redo support.
    pub fn with_undo(mut self, enabled: bool) -> Self {
        self.enable_undo = enabled;
        self
    }

    /// Sets a context menu to show on right-click.
    pub fn with_context_menu(mut self, menu: ContextMenu) -> Self {
        self.context_menu = RefCell::new(Some(menu));
        self
    }

    /// Returns the set of selected indices in multi-select mode.
    pub fn selected_indices(&self) -> &HashSet<usize> {
        &self.selected_indices
    }

    /// Clears the current selection.
    pub fn clear_selection(&mut self) {
        if !self.selected_indices.is_empty() {
            self.selected_indices.clear();
            self.dirty = true;
        }
    }

    /// Returns the drag manager for this list.
    pub fn drag_manager(&self) -> &RefCell<DragManager<usize>> {
        &self.drag_manager
    }

    /// Takes a snapshot of the current state for undo/redo.
    fn snapshot(&self) -> ListState {
        ListState {
            items: self.items.iter().map(|i| i.to_string()).collect(),
            selected: self.selected,
            offset: self.offset,
            selected_indices: self.selected_indices.clone(),
        }
    }

    /// Pushes the current state onto the undo stack.
    fn push_undo(&mut self) {
        if self.enable_undo {
            self.redo_stack.clear();
            self.undo_stack.push(self.snapshot());
            // Limit stack size
            if self.undo_stack.len() > 50 {
                self.undo_stack.remove(0);
            }
        }
    }

    /// Undo last operation.
    fn undo(&mut self) {
        if self.enable_undo && !self.undo_stack.is_empty() {
            if let Some(state) = self.undo_stack.pop() {
                let snapshot = self.snapshot();
                self.redo_stack.push(snapshot);
                self.apply_state(&state);
            }
            if let Some(ref mut cb) = self.on_undo {
                cb();
            }
        }
    }

    /// Redo last undone operation.
    fn redo(&mut self) {
        if self.enable_undo && !self.redo_stack.is_empty() {
            if let Some(state) = self.redo_stack.pop() {
                let snapshot = self.snapshot();
                self.undo_stack.push(snapshot);
                self.apply_state(&state);
            }
            if let Some(ref mut cb) = self.on_redo {
                cb();
            }
        }
    }

    /// Applies a state snapshot to the list.
    fn apply_state(&mut self, state: &ListState) {
        // Reconstruct items - note this requires T: From<String> or similar
        // For now, we restore what we can: selected, offset, selection
        self.selected = state.selected.min(self.items.len().saturating_sub(1));
        self.offset = state.offset;
        self.selected_indices = state.selected_indices.clone();
        // Clamp offset
        if self.selected >= self.offset + self.visible_count {
            self.offset = self.selected.saturating_sub(self.visible_count) + 1;
        }
        if self.selected < self.offset {
            self.offset = self.selected;
        }
        self.dirty = true;
    }

    /// Returns the index of the currently selected item.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Returns a reference to the currently selected item, or `None`.
    pub fn get_selected(&self) -> Option<&T> {
        self.items.get(self.selected)
    }

    /// Returns the total number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if there are no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Replaces all items while preserving selection and scroll when possible.
    ///
    /// If the previous selection is still valid for the new item count, it is kept.
    /// Otherwise, selection is clamped to the last item.
    pub fn set_items(&mut self, items: Vec<T>) {
        let old_selected = self.selected;
        self.items = items;
        if self.items.is_empty() {
            self.selected = 0;
        } else if old_selected >= self.items.len() {
            self.selected = self.items.len() - 1;
        }
        // Clamp offset to ensure selected item remains visible
        if self.selected >= self.offset + self.visible_count {
            self.offset = self.selected.saturating_sub(self.visible_count - 1);
        }
        self.dirty = true;
    }

    /// Appends a single item to the end of the list.
    pub fn push_item(&mut self, item: T) {
        self.items.push(item);
        self.dirty = true;
    }

    /// Removes the item at the given index and adjusts selection.
    pub fn remove_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            if !self.items.is_empty() && self.selected >= self.items.len() {
                self.selected = self.items.len() - 1;
            }
            self.dirty = true;
        }
    }

    /// Returns `(start, end)` indices of the currently visible items.
    pub fn viewport(&self) -> (usize, usize) {
        let start = self.offset;
        let end = (self.offset + self.visible_count).min(self.items.len());
        (start, end)
    }

    /// Scrolls to and selects the item at `index`.
    pub fn scroll_to(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        self.selected = index;
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + self.visible_count {
            self.offset = self.selected.saturating_sub(self.visible_count) + 1;
        }
    }

    /// Returns a `ScrollState` reflecting the current scroll position.
    pub fn scroll_state(&self) -> ScrollState {
        ScrollState {
            offset: self.offset,
            content_height: self.items.len(),
            viewport_height: self.visible_count,
        }
    }

    /// Sets how many items are visible at once.
    pub fn set_visible_count(&mut self, count: usize) {
        self.visible_count = count;
    }
}

impl<T: Clone + ToString> crate::framework::widget::Widget for List<T> {
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

        let visible_items: Vec<_> = self
            .items
            .iter()
            .skip(self.offset)
            .take(self.visible_count)
            .collect();

        for (i, item) in visible_items.iter().enumerate() {
            let row = i as u16;
            let idx = self.offset + i;
            let is_selected = idx == self.selected;
            let is_hovered = self.hovered == Some(idx);
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
            let style = if is_selected {
                Styles::BOLD
            } else {
                Styles::empty()
            };

            for col in 0..area.width {
                let idx = (row * area.width + col) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: ' ',
                        fg,
                        bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }

            let text = item.to_string();
            let label_len = text.width().min((area.width as usize).saturating_sub(2));
            for (j, ch) in text.chars().take(label_len).enumerate() {
                let idx = (row * area.width + 1 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].style = style;
                }
            }
        }

        // Scroll position indicator
        let total = self.items.len();
        let visible = visible_items.len();
        if total > visible && area.height > 1 {
            let indicator = format!(
                " {}–{}/{} ",
                self.offset + 1,
                (self.offset + visible).min(total),
                total
            );
            let badge_len = indicator.len();
            let badge_x = (area.width as usize).saturating_sub(badge_len);
            let badge_y = (area.height as usize).saturating_sub(1);
            let bg = self.theme.surface_elevated;
            let fg = self.theme.fg_muted;

            for x in badge_x..(area.width as usize) {
                let idx = badge_y * area.width as usize + x;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = bg;
                }
            }

            for (i, c) in indicator.chars().enumerate() {
                let idx = badge_y * area.width as usize + badge_x + i;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
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
            KeyCode::Down => {
                if self.selected + 1 < self.items.len() {
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
                self.selected = self.items.len().saturating_sub(1);
                self.offset = self.items.len().saturating_sub(self.visible_count);
                self.dirty = true;
                true
            }
            KeyCode::PageDown => {
                self.selected =
                    (self.selected + self.visible_count).min(self.items.len().saturating_sub(1));
                if self.selected >= self.offset + self.visible_count {
                    self.offset = self.selected.saturating_sub(self.visible_count) + 1;
                }
                self.dirty = true;
                true
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(self.visible_count);
                self.offset = self.selected;
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                if let Some(f) = self.on_select.as_mut() {
                    f(&self.items[self.selected]);
                }
                true
            }
            // Ctrl+Z: Undo
            KeyCode::Char('z') if key.modifiers.contains(crate::input::event::KeyModifiers::CONTROL) => {
                self.undo();
                true
            }
            // Ctrl+Y: Redo
            KeyCode::Char('y') if key.modifiers.contains(crate::input::event::KeyModifiers::CONTROL) => {
                self.redo();
                true
            }
            // Ctrl+A: Select all (in multi-select mode)
            KeyCode::Char('a') if key.modifiers.contains(crate::input::event::KeyModifiers::CONTROL) => {
                if self.allow_multi_select {
                    self.push_undo();
                    self.selected_indices.clear();
                    for i in 0..self.items.len() {
                        self.selected_indices.insert(i);
                    }
                    self.selected = 0;
                    self.dirty = true;
                    if let Some(ref mut cb) = self.on_selection_change {
                        cb(&self.selected_indices);
                    }
                }
                true
            }
            // Escape: Clear selection
            KeyCode::Esc => {
                if self.allow_multi_select && !self.selected_indices.is_empty() {
                    self.selected_indices.clear();
                    self.dirty = true;
                    if let Some(ref mut cb) = self.on_selection_change {
                        cb(&self.selected_indices);
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
        // Check if context menu is visible
        if let Some(ref mut menu) = *self.context_menu.borrow_mut() {
            if menu.is_visible()
                && menu.handle_mouse(kind, col, row) {
                    return true;
                }
        }

        match kind {
            crate::input::event::MouseEventKind::Moved => {
                if col >= self.width || row >= self.visible_count as u16 {
                    if self.hovered.is_some() {
                        self.hovered = None;
                        self.dirty = true;
                    }
                    return false;
                }
                let idx = self.offset + row as usize;
                if idx >= self.items.len() {
                    if self.hovered.is_some() {
                        self.hovered = None;
                        self.dirty = true;
                    }
                    return false;
                }
                if self.hovered != Some(idx) {
                    self.hovered = Some(idx);
                    self.dirty = true;
                }
                true
            }
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                if col >= self.width || row >= self.visible_count as u16 {
                    return false;
                }
                let idx = self.offset + row as usize;
                if idx >= self.items.len() {
                    return false;
                }

                // Multi-select handling
                if self.allow_multi_select {
                    // Track this click for potential shift-click range selection
                    self.last_selected = Some(idx);
                }

                self.selected = idx;
                if let Some(f) = self.on_select.as_mut() {
                    f(&self.items[idx]);
                }
                self.dirty = true;
                true
            }
            // Right-click: Show context menu
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Right) => {
                if col >= self.width || row >= self.visible_count as u16 {
                    return false;
                }
                let idx = self.offset + row as usize;
                if idx >= self.items.len() {
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
                    self.drag_manager.borrow_mut().move_ghost(area.x + col, area.y + row);
                }
                true
            }
            crate::input::event::MouseEventKind::ScrollDown => {
                self.offset =
                    (self.offset + 1).min(self.items.len().saturating_sub(self.visible_count));
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

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = *theme;
    }
}

impl<T: Clone + ToString> WidgetState for List<T> {
    fn state_id(&self) -> Option<&str> {
        Some("list")
    }

    fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "selected": self.selected,
            "offset": self.offset,
            "selected_indices": self.selected_indices.iter().collect::<Vec<_>>(),
        })
    }

    fn apply_json(&mut self, json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        if let (Some(selected), Some(offset)) = (
            json.get("selected").and_then(|v| v.as_u64()),
            json.get("offset").and_then(|v| v.as_u64()),
        ) {
            self.selected = selected as usize;
            self.offset = offset as usize;
        }
        if let Some(indices) = json.get("selected_indices").and_then(|v| v.as_array()) {
            self.selected_indices.clear();
            for idx in indices {
                if let Some(i) = idx.as_u64() {
                    self.selected_indices.insert(i as usize);
                }
            }
        }
        self.dirty = true;
        Ok(())
    }
}

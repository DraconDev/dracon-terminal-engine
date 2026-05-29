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
use crate::framework::widgets::list_common::{
    render_scroll_indicator, ListNavigation, SelectionChangeCallback, UndoRedoCallback,
};
use ratatui::layout::Rect;

pub type SelectCallback<T> = Box<dyn FnMut(&T)>;

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
    theme: Theme,
    on_select: Option<SelectCallback<T>>,
    on_selection_change: Option<SelectionChangeCallback>,
    on_undo: Option<UndoRedoCallback>,
    on_redo: Option<UndoRedoCallback>,
    item_height: u16,
    width: u16,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    drag_manager: RefCell<DragManager<usize>>,
    context_menu: RefCell<Option<ContextMenu>>,
    nav: ListNavigation<ListState>,
}

impl<T: Clone + ToString> List<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            id: WidgetId::next(),
            items,
            theme: Theme::default(),
            on_select: None,
            on_selection_change: None,
            on_undo: None,
            on_redo: None,
            item_height: 1,
            width: 40,
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 10)),
            dirty: true,
            drag_manager: RefCell::new(DragManager::new()),
            context_menu: RefCell::new(None),
            nav: ListNavigation::new(),
        }
    }

    pub fn new_with_id(id: WidgetId, items: Vec<T>) -> Self {
        Self {
            id,
            items,
            theme: Theme::default(),
            on_select: None,
            on_selection_change: None,
            on_undo: None,
            on_redo: None,
            item_height: 1,
            width: 40,
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 10)),
            dirty: true,
            drag_manager: RefCell::new(DragManager::new()),
            context_menu: RefCell::new(None),
            nav: ListNavigation::new(),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_item_height(mut self, height: u16) -> Self {
        self.item_height = height;
        self
    }

    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    pub fn on_select(mut self, f: impl FnMut(&T) + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn on_selection_change(mut self, f: impl FnMut(&HashSet<usize>) + 'static) -> Self {
        self.on_selection_change = Some(Box::new(f));
        self
    }

    pub fn on_undo(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_undo = Some(Box::new(f));
        self
    }

    pub fn on_redo(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_redo = Some(Box::new(f));
        self
    }

    pub fn with_multi_select(mut self, enabled: bool) -> Self {
        self.nav.allow_multi_select = enabled;
        self
    }

    pub fn with_undo(mut self, enabled: bool) -> Self {
        self.nav.enable_undo = enabled;
        self
    }

    pub fn with_context_menu(mut self, menu: ContextMenu) -> Self {
        self.context_menu = RefCell::new(Some(menu));
        self
    }

    pub fn selected_indices(&self) -> &HashSet<usize> {
        &self.nav.selected_indices
    }

    pub fn clear_selection(&mut self) {
        if self.nav.clear_selection() {
            self.dirty = true;
        }
    }

    pub fn drag_manager(&self) -> &RefCell<DragManager<usize>> {
        &self.drag_manager
    }

    fn snapshot(&self) -> ListState {
        ListState {
            items: self.items.iter().map(|i| i.to_string()).collect(),
            selected: self.nav.selected,
            offset: self.nav.offset,
            selected_indices: self.nav.selected_indices.clone(),
        }
    }

    fn undo(&mut self) {
        if let Some(state) = self.nav.undo(self.snapshot()) {
            self.apply_state(&state);
            if let Some(ref mut cb) = self.on_undo {
                cb();
            }
        }
    }

    fn redo(&mut self) {
        if let Some(state) = self.nav.redo(self.snapshot()) {
            self.apply_state(&state);
            if let Some(ref mut cb) = self.on_redo {
                cb();
            }
        }
    }

    fn apply_state(&mut self, state: &ListState) {
        self.nav.selected = state.selected.min(self.items.len().saturating_sub(1));
        self.nav.offset = state.offset;
        self.nav.selected_indices = state.selected_indices.clone();
        self.nav.clamp_scroll();
        self.dirty = true;
    }

    pub fn selected_index(&self) -> usize {
        self.nav.selected
    }

    pub fn get_selected(&self) -> Option<&T> {
        self.items.get(self.nav.selected)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        let old_selected = self.nav.selected;
        self.items = items;
        if self.items.is_empty() {
            self.nav.selected = 0;
        } else if old_selected >= self.items.len() {
            self.nav.selected = self.items.len() - 1;
        }
        if self.nav.selected >= self.nav.offset + self.nav.visible_count {
            self.nav.offset = self.nav.selected.saturating_sub(self.nav.visible_count - 1);
        }
        self.dirty = true;
    }

    pub fn push_item(&mut self, item: T) {
        self.items.push(item);
        self.dirty = true;
    }

    pub fn remove_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            if !self.items.is_empty() && self.nav.selected >= self.items.len() {
                self.nav.selected = self.items.len() - 1;
            }
            self.dirty = true;
        }
    }

    pub fn viewport(&self) -> (usize, usize) {
        let start = self.nav.offset;
        let end = (self.nav.offset + self.nav.visible_count).min(self.items.len());
        (start, end)
    }

    pub fn scroll_to(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        self.nav.selected = index;
        if self.nav.selected < self.nav.offset {
            self.nav.offset = self.nav.selected;
        } else if self.nav.selected >= self.nav.offset + self.nav.visible_count {
            self.nav.offset = self.nav.selected.saturating_sub(self.nav.visible_count) + 1;
        }
    }

    pub fn scroll_state(&self) -> ScrollState {
        ScrollState {
            offset: self.nav.offset,
            content_height: self.items.len(),
            viewport_height: self.nav.visible_count,
        }
    }

    pub fn set_visible_count(&mut self, count: usize) {
        self.nav.visible_count = count;
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

        let total_items = self.items.len();
        for i in 0..self.nav.visible_count {
            let idx = self.nav.offset + i;
            if idx >= total_items {
                break;
            }
            let row = i as u16;
            let is_selected = idx == self.nav.selected;
            let is_hovered = self.nav.hovered == Some(idx);
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
                let cell_idx = (row * area.width + col) as usize;
                if cell_idx < plane.cells.len() {
                    plane.cells[cell_idx] = Cell {
                        char: ' ',
                        fg,
                        bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }

            let item = &self.items[idx];
            let text = item.to_string();
            let label_len = text.width().min((area.width as usize).saturating_sub(2));
            for (j, ch) in text.chars().take(label_len).enumerate() {
                let cell_idx = (row * area.width + 1 + j as u16) as usize;
                if cell_idx < plane.cells.len() {
                    plane.cells[cell_idx].char = ch;
                    plane.cells[cell_idx].fg = fg;
                    plane.cells[cell_idx].style = style;
                }
            }
        }

        let total = self.items.len();
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
                if self.nav.move_down(self.items.len()) {
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
                self.nav.move_end(self.items.len());
                self.dirty = true;
                true
            }
            KeyCode::PageDown => {
                self.nav.page_down(self.items.len());
                self.dirty = true;
                true
            }
            KeyCode::PageUp => {
                self.nav.page_up();
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                if let Some(f) = self.on_select.as_mut() {
                    f(&self.items[self.nav.selected]);
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
                    self.nav.select_all(self.items.len());
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
                if col >= self.width || row >= self.nav.visible_count as u16 {
                    if self.nav.hovered.is_some() {
                        self.nav.hovered = None;
                        self.dirty = true;
                    }
                    return false;
                }
                let idx = self.nav.offset + row as usize;
                if idx >= self.items.len() {
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
                if col >= self.width || row >= self.nav.visible_count as u16 {
                    return false;
                }
                let idx = self.nav.offset + row as usize;
                if idx >= self.items.len() {
                    return false;
                }

                if self.nav.allow_multi_select {
                    self.nav.last_selected = Some(idx);
                }

                self.nav.selected = idx;
                if let Some(f) = self.on_select.as_mut() {
                    f(&self.items[idx]);
                }
                self.dirty = true;
                true
            }
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Right) => {
                if col >= self.width || row >= self.nav.visible_count as u16 {
                    return false;
                }
                let idx = self.nav.offset + row as usize;
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
                    self.drag_manager
                        .borrow_mut()
                        .move_ghost(area.x + col, area.y + row);
                }
                true
            }
            crate::input::event::MouseEventKind::ScrollDown => {
                self.nav.scroll_down(self.items.len());
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

impl<T: Clone + ToString> WidgetState for List<T> {
    fn state_id(&self) -> Option<&str> {
        None
    }

    fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "selected": self.nav.selected,
            "offset": self.nav.offset,
        })
    }

    fn apply_json(&mut self, json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        if let (Some(selected), Some(offset)) = (
            json.get("selected").and_then(|v| v.as_u64()),
            json.get("offset").and_then(|v| v.as_u64()),
        ) {
            self.nav.selected = selected as usize;
            self.nav.offset = offset as usize;
        }
        self.dirty = true;
        Ok(())
    }
}

//! Shared navigation, selection, and undo/redo logic for list-like widgets.

/// Callback for when a selection is made (Autocomplete and Tree).
pub type SelectCallback = Box<dyn FnMut(&str)>;

/// Callback for when selection changes (Table and List).
pub type SelectionChangeCallback = Box<dyn FnMut(&HashSet<usize>)>;

/// Callback for undo/redo actions (Table and List).
pub type UndoRedoCallback = Box<dyn FnMut()>;

/// State machine for list navigation, selection, and undo/redo.
pub struct ListNavigation<S: Clone> {
    pub selected: usize,
    pub offset: usize,
    pub visible_count: usize,
    pub hovered: Option<usize>,
    pub allow_multi_select: bool,
    pub selected_indices: HashSet<usize>,
    pub last_selected: Option<usize>,
    pub enable_undo: bool,
    pub undo_stack: Vec<S>,
    pub redo_stack: Vec<S>,
}

impl<S: Clone> Default for ListNavigation<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Clone> ListNavigation<S> {
    pub fn new() -> Self {
        Self {
            selected: 0,
            offset: 0,
            visible_count: 10,
            hovered: None,
            allow_multi_select: false,
            selected_indices: HashSet::new(),
            last_selected: None,
            enable_undo: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn move_down(&mut self, item_count: usize) -> bool {
        if self.selected + 1 < item_count {
            self.selected += 1;
            if self.selected >= self.offset + self.visible_count {
                self.offset = self.selected.saturating_sub(self.visible_count) + 1;
            }
            true
        } else {
            false
        }
    }

    pub fn move_up(&mut self) -> bool {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.offset {
                self.offset = self.selected;
            }
            true
        } else {
            false
        }
    }

    pub fn move_home(&mut self) -> bool {
        self.selected = 0;
        self.offset = 0;
        true
    }

    pub fn move_end(&mut self, item_count: usize) -> bool {
        self.selected = item_count.saturating_sub(1);
        self.offset = item_count.saturating_sub(self.visible_count);
        true
    }

    pub fn page_down(&mut self, item_count: usize) -> bool {
        self.selected = (self.selected + self.visible_count).min(item_count.saturating_sub(1));
        if self.selected >= self.offset + self.visible_count {
            self.offset = self.selected.saturating_sub(self.visible_count) + 1;
        }
        true
    }

    pub fn page_up(&mut self) -> bool {
        self.selected = self.selected.saturating_sub(self.visible_count);
        self.offset = self.selected;
        true
    }

    pub fn clamp_scroll(&mut self) {
        if self.selected >= self.offset + self.visible_count {
            self.offset = self.selected.saturating_sub(self.visible_count) + 1;
        }
        if self.selected < self.offset {
            self.offset = self.selected;
        }
    }

    pub fn push_undo(&mut self, snapshot: S) {
        if self.enable_undo {
            self.redo_stack.clear();
            self.undo_stack.push(snapshot);
            if self.undo_stack.len() > MAX_UNDO_STACK {
                self.undo_stack.remove(0);
            }
        }
    }

    pub fn undo(&mut self, current_snapshot: S) -> Option<S> {
        if self.enable_undo && !self.undo_stack.is_empty() {
            let state = self.undo_stack.pop()?;
            self.redo_stack.push(current_snapshot);
            Some(state)
        } else {
            None
        }
    }

    pub fn redo(&mut self, current_snapshot: S) -> Option<S> {
        if self.enable_undo && !self.redo_stack.is_empty() {
            let state = self.redo_stack.pop()?;
            self.undo_stack.push(current_snapshot);
            Some(state)
        } else {
            None
        }
    }

    pub fn select_all(&mut self, item_count: usize) {
        if self.allow_multi_select {
            self.selected_indices.clear();
            for i in 0..item_count {
                self.selected_indices.insert(i);
            }
            self.selected = 0;
        }
    }

    pub fn clear_selection(&mut self) -> bool {
        if !self.selected_indices.is_empty() {
            self.selected_indices.clear();
            true
        } else {
            false
        }
    }

    pub fn scroll_down(&mut self, item_count: usize) {
        self.offset = (self.offset + 1).min(item_count.saturating_sub(self.visible_count));
    }

    pub fn scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }
}

pub fn render_scroll_indicator(
    plane: &mut Plane,
    area: Rect,
    offset: usize,
    total: usize,
    visible: usize,
    theme: &Theme,
) {
    if total > visible && area.height > 1 {
        let indicator = format!(
            " {}–{}/{} ",
            offset + 1,
            (offset + visible).min(total),
            total
        );
        let badge_len = indicator.len();
        let badge_x = (area.width as usize).saturating_sub(badge_len);
        let badge_y = (area.height as usize).saturating_sub(1);
        let bg = theme.surface_elevated;
        let fg = theme.fg_muted;

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
}

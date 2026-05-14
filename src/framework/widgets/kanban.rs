//! Kanban board widget with columns and cards.
//!
//! Supports horizontal scrolling for columns, vertical scrolling per column,
//! and drag-and-drop card reordering between columns.

use std::cell::RefCell;

use crate::compositor::{Color, Plane, Styles};
use crate::framework::dragdrop::DragManager;
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;
use unicode_width::UnicodeWidthStr;

/// A card in the Kanban board.
#[derive(Debug, Clone)]
pub struct KanbanCard {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<Color>,
}

impl KanbanCard {
    /// Creates a new card with the given title.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            color: None,
        }
    }

    /// Sets the description for this card.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets the accent color for this card.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// Callback type for card move events.
pub type CardMoveCallback = Box<dyn FnMut(String, usize, usize)>;

/// A Kanban board widget with columns and draggable cards.
pub struct Kanban {
    id: WidgetId,
    columns: Vec<KanbanColumn>,
    theme: Theme,
    column_width: u16,
    card_height: u16,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    hovered_col: Option<usize>,
    hovered_card: Option<(usize, usize)>,
    selected_card: Option<(usize, usize)>,
    drag_manager: RefCell<DragManager<CardDragData>>,
    on_card_move: Option<CardMoveCallback>,
    scroll_offset: u16,
    is_dragging_card: bool,
    drag_card_pos: Option<(usize, usize)>,
}

/// Internal data for card drag operations.
#[derive(Debug, Clone)]
pub struct CardDragData {
    pub card_id: String,
    pub _source_col: usize,
    pub _source_idx: usize,
}

/// A column in the Kanban board.
#[derive(Debug, Clone)]
struct KanbanColumn {
    title: String,
    cards: Vec<KanbanCard>,
}

impl Kanban {
    /// Creates a new empty Kanban board.
    pub fn new() -> Self {
        Self {
            id: WidgetId::next(),
            columns: Vec::new(),
            theme: Theme::default(),
            column_width: 20,
            card_height: 4,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 20)),
            dirty: true,
            hovered_col: None,
            hovered_card: None,
            selected_card: None,
            drag_manager: RefCell::new(DragManager::new()),
            on_card_move: None,
            scroll_offset: 0,
            is_dragging_card: false,
            drag_card_pos: None,
        }
    }

    /// Creates a Kanban board with the given initial columns.
    pub fn with_columns(columns: Vec<(&str, Vec<&str>)>) -> Self {
        let cols: Vec<KanbanColumn> = columns
            .into_iter()
            .map(|(title, card_titles)| KanbanColumn {
                title: title.to_string(),
                cards: card_titles
                    .into_iter()
                    .enumerate()
                    .map(|(i, t)| KanbanCard::new(i.to_string(), t))
                    .collect(),
            })
            .collect();

        Self {
            id: WidgetId::next(),
            columns: cols,
            theme: Theme::default(),
            column_width: 20,
            card_height: 4,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 20)),
            dirty: true,
            hovered_col: None,
            hovered_card: None,
            selected_card: None,
            drag_manager: RefCell::new(DragManager::new()),
            on_card_move: None,
            scroll_offset: 0,
            is_dragging_card: false,
            drag_card_pos: None,
        }
    }

    /// Sets the theme for rendering.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the width of each column in cells.
    pub fn with_column_width(mut self, width: u16) -> Self {
        self.column_width = width;
        self
    }

    /// Sets the height of each card in cells.
    pub fn with_card_height(mut self, height: u16) -> Self {
        self.card_height = height;
        self
    }

    /// Adds a column with the given title.
    pub fn add_column(&mut self, title: &str) {
        self.columns.push(KanbanColumn {
            title: title.to_string(),
            cards: Vec::new(),
        });
        self.dirty = true;
    }

    /// Adds a card to a specific column.
    pub fn add_card(&mut self, col: usize, card: KanbanCard) {
        if let Some(column) = self.columns.get_mut(col) {
            column.cards.push(card);
            self.dirty = true;
        }
    }

    /// Registers a callback invoked when a card is moved between columns.
    /// The callback receives (card_id, from_col, to_col).
    pub fn on_card_move(mut self, f: impl FnMut(String, usize, usize) + 'static) -> Self {
        self.on_card_move = Some(Box::new(f));
        self
    }

    /// Returns the drag manager for this kanban board.
    pub fn drag_manager(&self) -> &RefCell<DragManager<CardDragData>> {
        &self.drag_manager
    }

    /// Returns the number of columns.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Returns the number of cards in a column.
    pub fn card_count(&self, col: usize) -> Option<usize> {
        self.columns.get(col).map(|c| c.cards.len())
    }

    /// Returns the selected card position as (column, index).
    pub fn selected_card(&self) -> Option<(usize, usize)> {
        self.selected_card
    }

    /// Selects a card at the given position.
    pub fn select_card(&mut self, col: usize, idx: usize) {
        if col < self.columns.len() && idx < self.columns[col].cards.len() {
            self.selected_card = Some((col, idx));
            self.dirty = true;
        }
    }

    /// Clears the current card selection.
    pub fn clear_selection(&mut self) {
        self.selected_card = None;
        self.dirty = true;
    }

    /// Moves a card from one position to another.
    pub fn move_card(&mut self, from_col: usize, from_idx: usize, to_col: usize, to_idx: usize) {
        if from_col >= self.columns.len() || to_col >= self.columns.len() {
            return;
        }
        let from_column = &mut self.columns[from_col];
        if from_idx >= from_column.cards.len() {
            return;
        }

        let card = from_column.cards.remove(from_idx);

        let to_column = &mut self.columns[to_col];
        let insert_idx = to_idx.min(to_column.cards.len());
        to_column.cards.insert(insert_idx, card);

        self.dirty = true;
    }
}

impl Default for Kanban {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::framework::widget::Widget for Kanban {
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

        // Render columns
        let col_spacing: u16 = 1;
        let total_col_width = self.column_width + col_spacing;
        let visible_cols = (area.width / total_col_width) as usize;

        let start_col = self.scroll_offset as usize;
        let end_col = (start_col + visible_cols).min(self.columns.len());

        for (i, col_idx) in (start_col..end_col).enumerate() {
            let col_x = (i as u16) * total_col_width;
            if col_x + self.column_width > area.width {
                break;
            }

            let col = &self.columns[col_idx];

            // Column header background
            for y in 0..3u16 {
                for x in col_x..col_x + self.column_width {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        let is_hovered = self.hovered_col == Some(col_idx);
                        plane.cells[idx].bg = if is_hovered {
                            self.theme.surface_elevated
                        } else {
                            self.theme.bg
                        };
                    }
                }
            }

            // Column header border (top)
            for x in col_x..col_x + self.column_width {
                let idx = (3u16 * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = self.theme.outline;
                }
            }

            // Column title
            let title_len = col.title.width().min(self.column_width as usize - 2);
            for (j, ch) in col.title.chars().take(title_len).enumerate() {
                let idx = (area.width + col_x + 1 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = self.theme.primary;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }

            // Card count badge
            let count_text = format!("({})", col.cards.len());
            let count_x = col_x + self.column_width.saturating_sub(count_text.len() as u16 + 1);
            for (j, ch) in count_text.chars().enumerate() {
                let idx = (area.width + count_x + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = self.theme.fg_muted;
                }
            }

            // Render cards
            for (card_idx, card) in col.cards.iter().enumerate() {
                let card_y = 4 + (card_idx as u16) * self.card_height;
                if card_y >= area.height {
                    break;
                }

                let is_selected = self.selected_card == Some((col_idx, card_idx));
                let is_hovered = self.hovered_card == Some((col_idx, card_idx));

                // Card background
                let bg = if is_selected {
                    self.theme.selection_bg
                } else if is_hovered {
                    self.theme.hover_bg
                } else {
                    self.theme.surface_elevated
                };

                for y in 0..self.card_height {
                    for x in 0..self.column_width {
                        let cy = card_y + y;
                        let cx = col_x + x;
                        if cy >= area.height {
                            break;
                        }
                        let idx = (cy * area.width + cx) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].bg = bg;
                        }
                    }
                }

                // Card border
                // Left border
                for y in 0..self.card_height {
                    let cy = card_y + y;
                    if cy >= area.height {
                        break;
                    }
                    let idx = (cy * area.width + col_x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '│';
                        plane.cells[idx].fg = card.color.unwrap_or(self.theme.outline);
                    }
                }
                // Right border
                for y in 0..self.card_height {
                    let cy = card_y + y;
                    if cy >= area.height {
                        break;
                    }
                    let idx = (cy * area.width + col_x + self.column_width - 1) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '│';
                        plane.cells[idx].fg = card.color.unwrap_or(self.theme.outline);
                    }
                }

                // Card title
                let title_len = card.title.width().min((self.column_width - 2) as usize);
                for (j, ch) in card.title.chars().take(title_len).enumerate() {
                    let idx = ((card_y + 1) * area.width + col_x + 1 + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = if is_selected {
                            self.theme.selection_fg
                        } else {
                            self.theme.fg
                        };
                    }
                }

                // Card description (if present and card has enough height)
                if self.card_height > 2 {
                    if let Some(desc) = &card.description {
                        let desc_len = desc.width().min((self.column_width - 2) as usize);
                        for (j, ch) in desc.chars().take(desc_len).enumerate() {
                            let idx = ((card_y + 2) * area.width + col_x + 1 + j as u16) as usize;
                            if idx < plane.cells.len() {
                                plane.cells[idx].char = ch;
                                plane.cells[idx].fg = self.theme.fg_muted;
                            }
                        }
                    }
                }

                // Card bottom border
                let bottom_y = card_y + self.card_height - 1;
                if bottom_y < area.height {
                    for x in col_x..col_x + self.column_width {
                        let idx = (bottom_y * area.width + x) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].char = '─';
                            plane.cells[idx].fg = self.theme.outline;
                        }
                    }
                }
            }

            // Column vertical divider
            if col_x + self.column_width < area.width {
                for y in 0..area.height {
                    let idx = (y * area.width + col_x + self.column_width) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ' ';
                        plane.cells[idx].fg = self.theme.outline;
                        plane.cells[idx].bg = self.theme.bg;
                    }
                }
            }
        }

        // Horizontal scroll indicator
        if self.columns.len() > visible_cols {
            let indicator = format!(
                "< {} / {} >",
                self.scroll_offset + 1,
                self.columns.len()
            );
            let indicator_len = indicator.len() as u16;
            let indicator_x = (area.width.saturating_sub(indicator_len)) / 2;
            let indicator_y = area.height.saturating_sub(1);

            for (i, ch) in indicator.chars().enumerate() {
                let idx = (indicator_y * area.width + indicator_x + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = self.theme.fg_muted;
                    plane.cells[idx].bg = self.theme.bg;
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

        // Handle card selection navigation
        if let Some((col_idx, card_idx)) = self.selected_card {
            match key.code {
                KeyCode::Left => {
                    if col_idx > 0 {
                        let new_idx = self.columns[col_idx - 1].cards.len().saturating_sub(1);
                        self.select_card(col_idx - 1, new_idx);
                    }
                    true
                }
                KeyCode::Right => {
                    if col_idx + 1 < self.columns.len() {
                        let new_idx = self.columns[col_idx + 1].cards.len().saturating_sub(1);
                        self.select_card(col_idx + 1, new_idx);
                    }
                    true
                }
                KeyCode::Up => {
                    if card_idx > 0 {
                        self.select_card(col_idx, card_idx - 1);
                    }
                    true
                }
                KeyCode::Down => {
                    if card_idx + 1 < self.columns[col_idx].cards.len() {
                        self.select_card(col_idx, card_idx + 1);
                    }
                    true
                }
                KeyCode::Esc => {
                    self.clear_selection();
                    true
                }
                _ => false,
            }
        } else {
            // No selection, navigate columns
            match key.code {
                KeyCode::Left => {
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                        self.dirty = true;
                    }
                    true
                }
                KeyCode::Right => {
                    let visible = self.area.get().width / (self.column_width + 1);
                    if self.scroll_offset + visible < self.columns.len() as u16 {
                        self.scroll_offset += 1;
                        self.dirty = true;
                    }
                    true
                }
                KeyCode::Enter => {
                    // Select first card of hovered column or first column
                    if let Some(col_idx) = self.hovered_col {
                        if !self.columns[col_idx].cards.is_empty() {
                            self.select_card(col_idx, 0);
                        }
                    } else if !self.columns.is_empty() && !self.columns[0].cards.is_empty() {
                        self.select_card(0, 0);
                    }
                    true
                }
                _ => false,
            }
        }
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        let area = self.area.get();

        match kind {
            crate::input::event::MouseEventKind::Moved => {
                // Determine which column/region we're over
                let col_spacing: u16 = 1;
                let total_col_width = self.column_width + col_spacing;

                let rel_col = col.saturating_sub(area.x);
                let rel_row = row.saturating_sub(area.y);

                let col_idx = (rel_col / total_col_width) as usize + self.scroll_offset as usize;

                // Check if we're over a card
                if rel_row >= 4 {
                    let card_idx = ((rel_row - 4) / self.card_height) as usize;

                    if col_idx < self.columns.len() && card_idx < self.columns[col_idx].cards.len() {
                        let new_hovered = Some((col_idx, card_idx));
                        if self.hovered_card != new_hovered {
                            self.hovered_card = new_hovered;
                            self.hovered_col = Some(col_idx);
                            self.dirty = true;
                        }
                    } else {
                        if self.hovered_card.is_some() || self.hovered_col != Some(col_idx) {
                            self.hovered_card = None;
                            self.hovered_col = Some(col_idx);
                            self.dirty = true;
                        }
                    }
                } else {
                    // Header area
                    if self.hovered_col != Some(col_idx) {
                        self.hovered_col = Some(col_idx);
                        self.hovered_card = None;
                        self.dirty = true;
                    }
                }
                true
            }
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                let rel_col = col.saturating_sub(area.x);
                let rel_row = row.saturating_sub(area.y);

                let col_spacing: u16 = 1;
                let total_col_width = self.column_width + col_spacing;
                let col_idx = (rel_col / total_col_width) as usize + self.scroll_offset as usize;

                if rel_row >= 4 && col_idx < self.columns.len() {
                    let card_idx = ((rel_row - 4) / self.card_height) as usize;
                    if card_idx < self.columns[col_idx].cards.len() {
                        self.select_card(col_idx, card_idx);

                        if self.on_card_move.is_some() {
                            self.is_dragging_card = true;
                            self.drag_card_pos = Some((col_idx, card_idx));
                        }
                    }
                } else if col_idx < self.columns.len() {
                    // Clicked on column header - select first card
                    if !self.columns[col_idx].cards.is_empty() {
                        self.select_card(col_idx, 0);
                    }
                }
                true
            }
            crate::input::event::MouseEventKind::Drag(_) => {
                if self.is_dragging_card {
                    self.drag_manager.borrow_mut().move_ghost(col, row);
                }
                true
            }
            crate::input::event::MouseEventKind::Up(_) => {
                if self.is_dragging_card {
                    let from = self.drag_card_pos;
                    self.is_dragging_card = false;
                    self.drag_card_pos = None;

                    // Determine target column
                    let rel_col = col.saturating_sub(area.x);
                    let col_spacing: u16 = 1;
                    let total_col_width = self.column_width + col_spacing;
                    let to_col = (rel_col / total_col_width) as usize + self.scroll_offset as usize;

                    if let Some((from_col, from_idx)) = from {
                        if to_col < self.columns.len() && to_col != from_col {
                            // Card moved to different column
                            let card_id = self.columns[from_col].cards[from_idx].id.clone();
                            self.move_card(from_col, from_idx, to_col, self.columns[to_col].cards.len());

                            if let Some(ref mut cb) = self.on_card_move {
                                cb(card_id, from_col, to_col);
                            }
                        }
                    }
                }
                true
            }
            crate::input::event::MouseEventKind::ScrollDown => {
                let visible = area.width / (self.column_width + 1);
                if self.scroll_offset + visible < self.columns.len() as u16 {
                    self.scroll_offset += 1;
                    self.dirty = true;
                }
                true
            }
            crate::input::event::MouseEventKind::ScrollUp => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                    self.dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = theme.clone();
    }
}
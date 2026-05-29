//! Context menu widget — positioned popup with labeled actions.
//!
//! Provides a right-click style context menu that appears at a given anchor
//! position, auto-clamped to screen bounds. Supports keyboard navigation
//! (↑/↓, Enter, Esc), mouse hover highlighting, click-to-select, and
//! click-outside-to-dismiss.
//!
//! # Separator Support
//!
//! Items with `is_separator: true` render as a horizontal divider line.
//! Separators are skipped during keyboard navigation and cannot be selected.
//!
//! # Click-Outside Dismiss
//!
//! When a mouse Down event occurs outside the menu bounds, `handle_mouse`
//! returns `false` — the caller should interpret this as a dismiss signal
//! and call `hide()`.

use crate::compositor::plane::Plane;
use crate::framework::theme::Theme;
use crate::framework::widget::{WidgetId, WidgetState};
use crate::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

// ---------------------------------------------------------------------------
// ContextAction (legacy API, kept for backward compatibility)
// ---------------------------------------------------------------------------

/// An action type for context menu items.
///
/// Provided for backward compatibility with the `(label, ContextAction)` API.
/// New code should prefer `ContextMenuItem::new(id, label)` for more descriptive
/// identifiers and callback dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextAction {
    Open,
    Edit,
    Delete,
    Rename,
    Copy,
    Cut,
    Paste,
    Separator,
}

// ---------------------------------------------------------------------------
// ContextMenuItem
// ---------------------------------------------------------------------------

/// A single item in a context menu.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenuItem {
    /// Unique identifier for callback dispatch.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Optional single-char icon prefix.
    pub icon: Option<char>,
    /// If true, renders as a horizontal separator line.
    pub is_separator: bool,
    /// Optional legacy action (for backward compat with `(label, ContextAction)` API).
    pub(crate) legacy_action: Option<ContextAction>,
}

impl ContextMenuItem {
    /// Create a regular menu item.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            is_separator: false,
            legacy_action: None,
        }
    }

    /// Create a menu item with an icon prefix.
    pub fn with_icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Create a separator item.
    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            icon: None,
            is_separator: true,
            legacy_action: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Callback type
// ---------------------------------------------------------------------------

/// Callback fired when a menu item is selected (Enter or click).
pub type ContextSelectCallback = Box<dyn FnMut(&str)>;

// ---------------------------------------------------------------------------
// ContextMenu
// ---------------------------------------------------------------------------

/// A popup context menu with labeled actions.
///
/// # Usage
///
/// ```ignore
/// let menu = ContextMenu::new(vec![
///     ContextMenuItem::new("open", "Open").with_icon('📂'),
///     ContextMenuItem::new("edit", "Edit").with_icon('✏'),
///     ContextMenuItem::separator(),
///     ContextMenuItem::new("delete", "Delete").with_icon('🗑'),
/// ])
/// .with_anchor(col, row)
/// .with_theme(theme);
/// ```
pub struct ContextMenu {
    id: WidgetId,
    items: Vec<ContextMenuItem>,
    theme: Theme,
    width: u16,
    anchor_x: u16,
    anchor_y: u16,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    /// Index into `items` (skips separators during navigation).
    selected: usize,
    /// Index of the item currently hovered by mouse.
    hovered: Option<usize>,
    visible: bool,
    /// Callback fired when a non-separator item is selected.
    on_select: Option<std::cell::RefCell<ContextSelectCallback>>,
}

impl ContextMenu {
    /// Creates a new `ContextMenu` from a list of items.
    pub fn new(items: Vec<ContextMenuItem>) -> Self {
        let width = items
            .iter()
            .filter(|i| !i.is_separator)
            .map(|i| {
                let label_len = i.label.len();
                let icon_len = if i.icon.is_some() { 2 } else { 0 }; // "X " prefix
                (label_len + icon_len + 4) as u16 // padding + borders
            })
            .max()
            .unwrap_or(16)
            .max(12);

        Self {
            id: WidgetId::next(),
            items,
            theme: Theme::default(),
            width,
            anchor_x: 0,
            anchor_y: 0,
            area: std::cell::Cell::new(Rect::new(0, 0, width, 10)),
            dirty: true,
            selected: 0,
            hovered: None,
            visible: true,
            on_select: None,
        }
    }

    /// Creates a new `ContextMenu` with the given widget ID.
    pub fn new_with_id(id: WidgetId, items: Vec<ContextMenuItem>) -> Self {
        let mut menu = Self::new(items);
        menu.id = id;
        menu
    }

    /// Legacy constructor from `(label, ContextAction)` pairs.
    ///
    /// Converts `ContextAction::Separator` to `ContextMenuItem::separator()`,
    /// and other variants to `ContextMenuItem::new(action_name, label)` with
    /// the original `ContextAction` stored for `action_at()` retrieval.
    pub fn from_actions(items: Vec<(&'static str, ContextAction)>) -> Self {
        let new_items: Vec<ContextMenuItem> = items
            .into_iter()
            .map(|(label, action)| {
                if action == ContextAction::Separator {
                    ContextMenuItem::separator()
                } else {
                    let id = format!("{:?}", &action).to_lowercase();
                    ContextMenuItem {
                        id,
                        label: label.to_string(),
                        icon: None,
                        is_separator: false,
                        legacy_action: Some(action),
                    }
                }
            })
            .collect();
        Self::new(new_items)
    }

    /// Legacy constructor from `(label, ContextAction)` pairs with a given widget ID.
    pub fn from_actions_with_id(id: WidgetId, items: Vec<(&'static str, ContextAction)>) -> Self {
        let mut menu = Self::from_actions(items);
        menu.id = id;
        menu
    }

    /// Returns the action at the given index (legacy API).
    ///
    /// Only returns `Some` for items created via `from_actions()`.
    /// Returns `None` for items created via `ContextMenuItem::new()` or separators.
    pub fn action_at(&self, index: usize) -> Option<&ContextAction> {
        self.items
            .get(index)
            .and_then(|item| item.legacy_action.as_ref())
    }

    /// Sets the theme for rendering.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the menu width in cells.
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self.dirty = true;
        self
    }

    /// Sets the anchor position for the menu (where it appears).
    pub fn with_anchor(mut self, x: u16, y: u16) -> Self {
        self.anchor_x = x;
        self.anchor_y = y;
        self.dirty = true;
        self
    }

    /// Sets a callback fired when a menu item is selected.
    pub fn on_select(mut self, cb: ContextSelectCallback) -> Self {
        self.on_select = Some(std::cell::RefCell::new(cb));
        self
    }

    /// Sets the anchor position (mutable variant).
    pub fn set_anchor(&mut self, x: u16, y: u16) {
        self.anchor_x = x;
        self.anchor_y = y;
        self.dirty = true;
    }

    /// Returns whether the menu is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Shows the menu.
    pub fn show(&mut self) {
        self.visible = true;
        self.selected = self.first_selectable_index();
        self.dirty = true;
    }

    /// Hides the menu.
    pub fn hide(&mut self) {
        self.visible = false;
        self.hovered = None;
        self.dirty = true;
    }

    /// Returns the currently selected item index.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Returns the selected item's id, if any.
    pub fn selected_id(&self) -> Option<&str> {
        self.items.get(self.selected).map(|i| i.id.as_str())
    }

    /// Returns the item at the given index.
    pub fn item_at(&self, index: usize) -> Option<&ContextMenuItem> {
        self.items.get(index)
    }

    /// Returns the number of items (including separators).
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Returns the menu width.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Returns the anchor x coordinate.
    pub fn anchor_x(&self) -> u16 {
        self.anchor_x
    }

    /// Returns the anchor y coordinate.
    pub fn anchor_y(&self) -> u16 {
        self.anchor_y
    }

    // -- internal helpers --

    /// First non-separator index.
    fn first_selectable_index(&self) -> usize {
        self.items.iter().position(|i| !i.is_separator).unwrap_or(0)
    }

    /// Next selectable index after `from`, wrapping around.
    fn next_selectable(&self, from: usize) -> usize {
        let len = self.items.len();
        for step in 1..=len {
            let idx = (from + step) % len;
            if !self.items[idx].is_separator {
                return idx;
            }
        }
        from
    }

    /// Previous selectable index before `from`, wrapping around.
    fn prev_selectable(&self, from: usize) -> usize {
        let len = self.items.len();
        if len == 0 {
            return 0;
        }
        for step in 1..=len {
            let idx = (from + len - step) % len;
            if !self.items[idx].is_separator {
                return idx;
            }
        }
        from
    }

    /// Compute the screen-rect of the menu (auto-clamped).
    fn screen_rect(&self, screen: Rect) -> (u16, u16, u16, u16) {
        let height = self.items.len().max(1) as u16 + 2; // +2 for border
        let mut x = self.anchor_x;
        let mut y = self.anchor_y;
        if x + self.width > screen.width {
            x = screen.width.saturating_sub(self.width);
        }
        if y + height > screen.height {
            y = screen.height.saturating_sub(height);
        }
        (x, y, self.width, height)
    }

    /// Fire the on_select callback with the selected item's id.
    fn fire_select(&self) {
        if let Some(cb) = &self.on_select {
            if let Some(item) = self.items.get(self.selected) {
                if !item.is_separator {
                    cb.borrow_mut()(&item.id);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Widget trait
// ---------------------------------------------------------------------------

impl crate::framework::widget::Widget for ContextMenu {
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

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn z_index(&self) -> u16 {
        200
    }

    fn render(&self, screen: Rect) -> Plane {
        let (x, y, w, h) = self.screen_rect(screen);
        self.area.set(Rect::new(x, y, w, h));

        let mut plane = Plane::new(0, w, h);
        plane.x = x;
        plane.y = y;
        plane.z_index = 200;
        plane.fill_bg(self.theme.surface_elevated);

        // Border (rounded corners)
        let fg = self.theme.outline;
        let bg = self.theme.surface_elevated;
        // Top row
        for cx in 0..w {
            let idx = cx as usize;
            if idx < plane.cells.len() {
                let ch = if cx == 0 {
                    '╭'
                } else if cx == w - 1 {
                    '╮'
                } else {
                    '─'
                };
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = fg;
                plane.cells[idx].bg = bg;
                plane.cells[idx].transparent = false;
            }
        }
        // Bottom row
        let by = h - 1;
        for cx in 0..w {
            let idx = (by as usize) * w as usize + cx as usize;
            if idx < plane.cells.len() {
                let ch = if cx == 0 {
                    '╰'
                } else if cx == w - 1 {
                    '╯'
                } else {
                    '─'
                };
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = fg;
                plane.cells[idx].bg = bg;
                plane.cells[idx].transparent = false;
            }
        }
        // Side borders
        for row in 1..h.saturating_sub(1) {
            let left = (row as usize) * w as usize;
            let right = (row as usize) * w as usize + (w - 1) as usize;
            if left < plane.cells.len() {
                plane.cells[left].char = '│';
                plane.cells[left].fg = fg;
                plane.cells[left].bg = bg;
                plane.cells[left].transparent = false;
            }
            if right < plane.cells.len() {
                plane.cells[right].char = '│';
                plane.cells[right].fg = fg;
                plane.cells[right].bg = bg;
                plane.cells[right].transparent = false;
            }
        }

        // Items
        for (i, item) in self.items.iter().enumerate() {
            let row = (i as u16) + 1; // +1 for top border
            let is_selected = i == self.selected;
            let is_hovered = self.hovered == Some(i);

            if item.is_separator {
                // Render separator line
                for cx in 1..w.saturating_sub(1) {
                    let idx = (row as usize) * w as usize + cx as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '─';
                        plane.cells[idx].fg = self.theme.outline_variant;
                        plane.cells[idx].bg = bg;
                        plane.cells[idx].transparent = false;
                    }
                }
                continue;
            }

            // Item row background
            let item_bg = if is_selected {
                self.theme.selection_bg
            } else if is_hovered {
                self.theme.hover_bg
            } else {
                bg
            };
            for cx in 1..w.saturating_sub(1) {
                let idx = (row as usize) * w as usize + cx as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = item_bg;
                    plane.cells[idx].transparent = false;
                }
            }

            // Icon + label
            let mut col = 2u16; // start after border + 1 space
            if let Some(icon) = item.icon {
                let idx = (row as usize) * w as usize + col as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = icon;
                    plane.cells[idx].fg = self.theme.primary;
                    plane.cells[idx].bg = item_bg;
                }
                col += 2; // icon + space
            }

            let item_fg = self.theme.fg;

            for (j, ch) in item.label.chars().enumerate() {
                let cx = col + j as u16;
                if cx >= w - 1 {
                    break;
                }
                let idx = (row as usize) * w as usize + cx as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = item_fg;
                    plane.cells[idx].bg = item_bg;
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if !self.visible {
            return false;
        }
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Up => {
                self.selected = self.prev_selectable(self.selected);
                self.dirty = true;
                true
            }
            KeyCode::Down => {
                self.selected = self.next_selectable(self.selected);
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                self.fire_select();
                self.dirty = true;
                true
            }
            KeyCode::Esc => {
                self.hide();
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
        if !self.visible {
            return false;
        }

        let area = self.area.get();
        let inside = col >= area.x
            && col < area.x + area.width
            && row >= area.y
            && row < area.y + area.height;

        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if !inside {
                    // Click outside → dismiss. Return false so caller can handle.
                    self.hide();
                    return false;
                }
                // Click on item
                let item_row = row.saturating_sub(area.y + 1) as usize; // +1 for top border
                if item_row < self.items.len() && !self.items[item_row].is_separator {
                    self.selected = item_row;
                    self.fire_select();
                    self.dirty = true;
                }
                true
            }
            MouseEventKind::Down(_) => {
                if !inside {
                    self.hide();
                    return false;
                }
                true
            }
            MouseEventKind::Moved => {
                if !inside {
                    if self.hovered.is_some() {
                        self.hovered = None;
                        self.dirty = true;
                    }
                    return false;
                }
                let item_row = row.saturating_sub(area.y + 1) as usize;
                let new_hovered =
                    if item_row < self.items.len() && !self.items[item_row].is_separator {
                        Some(item_row)
                    } else {
                        None
                    };
                if new_hovered != self.hovered {
                    self.hovered = new_hovered;
                    self.dirty = true;
                }
                true
            }
            MouseEventKind::Up(_) => true,
            _ => inside,
        }
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = theme.clone();
    }
}

impl WidgetState for ContextMenu {
    fn state_id(&self) -> Option<&str> {
        Some("context_menu")
    }

    fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "visible": self.visible,
            "selected": self.selected,
        })
    }

    fn apply_json(&mut self, json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        if let Some(visible) = json.get("visible").and_then(|v| v.as_bool()) {
            self.visible = visible;
        }
        if let Some(selected) = json.get("selected").and_then(|v| v.as_u64()) {
            self.selected = selected as usize;
        }
        self.dirty = true;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::widget::Widget;

    fn sample_items() -> Vec<ContextMenuItem> {
        vec![
            ContextMenuItem::new("open", "Open").with_icon('📂'),
            ContextMenuItem::new("edit", "Edit").with_icon('✏'),
            ContextMenuItem::separator(),
            ContextMenuItem::new("delete", "Delete").with_icon('🗑'),
        ]
    }

    #[test]
    fn context_menu_new() {
        let menu = ContextMenu::new(sample_items());
        assert_eq!(menu.item_count(), 4);
        assert!(menu.is_visible());
    }

    #[test]
    fn context_menu_show_hide() {
        let mut menu = ContextMenu::new(sample_items());
        assert!(menu.is_visible());
        menu.hide();
        assert!(!menu.is_visible());
        menu.show();
        assert!(menu.is_visible());
    }

    #[test]
    fn context_menu_separator_skipped_nav_down() {
        let mut menu = ContextMenu::new(sample_items());
        // Start at "open" (index 0), down → skip separator → "delete" (index 3)
        assert_eq!(menu.selected_index(), 0);
        menu.handle_key(KeyEvent {
            code: KeyCode::Down,
            modifiers: Default::default(),
            kind: KeyEventKind::Press,
        });
        assert_eq!(menu.selected_index(), 1); // "edit"
        menu.handle_key(KeyEvent {
            code: KeyCode::Down,
            modifiers: Default::default(),
            kind: KeyEventKind::Press,
        });
        assert_eq!(menu.selected_index(), 3); // "delete" (skipped separator at 2)
    }

    #[test]
    fn context_menu_separator_skipped_nav_up() {
        let mut menu = ContextMenu::new(sample_items());
        // Start at "delete" (index 3), up → skip separator → "edit" (index 1)
        menu.selected = 3;
        menu.handle_key(KeyEvent {
            code: KeyCode::Up,
            modifiers: Default::default(),
            kind: KeyEventKind::Press,
        });
        assert_eq!(menu.selected_index(), 1); // "edit" (skipped separator at 2)
    }

    #[test]
    fn context_menu_esc_hides() {
        let mut menu = ContextMenu::new(sample_items());
        assert!(menu.is_visible());
        let handled = menu.handle_key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: Default::default(),
            kind: KeyEventKind::Press,
        });
        assert!(handled);
        assert!(!menu.is_visible());
    }

    #[test]
    fn context_menu_enter_fires_callback() {
        let selected = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
        let selected_clone = selected.clone();
        let mut menu = ContextMenu::new(sample_items()).on_select(Box::new(move |id: &str| {
            *selected_clone.borrow_mut() = id.to_string();
        }));
        menu.handle_key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: Default::default(),
            kind: KeyEventKind::Press,
        });
        assert_eq!(selected.borrow().as_str(), "open");
    }

    #[test]
    fn context_menu_click_outside_hides() {
        let mut menu = ContextMenu::new(sample_items())
            .with_anchor(10, 5)
            .with_width(20);
        menu.area.set(Rect::new(10, 5, 20, 6));
        let handled = menu.handle_mouse(
            MouseEventKind::Down(MouseButton::Left),
            2, // Outside menu
            2,
        );
        assert!(!handled);
        assert!(!menu.is_visible());
    }

    #[test]
    fn context_menu_click_item_selects() {
        let selected = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
        let selected_clone = selected.clone();
        let mut menu = ContextMenu::new(sample_items())
            .with_anchor(10, 5)
            .with_width(20)
            .on_select(Box::new(move |id: &str| {
                *selected_clone.borrow_mut() = id.to_string();
            }));
        menu.area.set(Rect::new(10, 5, 20, 6));
        // Click on "edit" (row 1 within menu, screen row 7)
        let handled = menu.handle_mouse(
            MouseEventKind::Down(MouseButton::Left),
            12,
            7, // area.y + 1 (border) + 1 (edit row)
        );
        assert!(handled);
        assert_eq!(menu.selected_index(), 1);
        assert_eq!(selected.borrow().as_str(), "edit");
    }

    #[test]
    fn context_menu_hover_highlights() {
        let mut menu = ContextMenu::new(sample_items())
            .with_anchor(10, 5)
            .with_width(20);
        menu.area.set(Rect::new(10, 5, 20, 6));
        assert_eq!(menu.hovered, None);
        // Hover over "open" (row 0 within items, screen row 6)
        let handled = menu.handle_mouse(MouseEventKind::Moved, 12, 6);
        assert!(handled);
        assert_eq!(menu.hovered, Some(0));
    }

    #[test]
    fn context_menu_hover_separator_none() {
        let mut menu = ContextMenu::new(sample_items())
            .with_anchor(10, 5)
            .with_width(20);
        menu.area.set(Rect::new(10, 5, 20, 6));
        // Hover over separator (index 2, screen row 8)
        let handled = menu.handle_mouse(MouseEventKind::Moved, 12, 8);
        assert!(handled);
        assert_eq!(menu.hovered, None); // Separators can't be hovered
    }

    #[test]
    fn context_menu_wrap_around() {
        let mut menu = ContextMenu::new(sample_items());
        // From "delete" (last selectable), down wraps to "open"
        menu.selected = 3;
        menu.handle_key(KeyEvent {
            code: KeyCode::Down,
            modifiers: Default::default(),
            kind: KeyEventKind::Press,
        });
        assert_eq!(menu.selected_index(), 0); // Wrapped to first
    }

    #[test]
    fn context_menu_selected_id() {
        let menu = ContextMenu::new(sample_items());
        assert_eq!(menu.selected_id(), Some("open"));
    }

    #[test]
    fn context_menu_auto_width() {
        let items = vec![
            ContextMenuItem::new("x", "Short"),
            ContextMenuItem::new("y", "A very long label that should widen the menu"),
        ];
        let menu = ContextMenu::new(items);
        assert!(menu.width() >= 20); // Should be wider than default
    }

    #[test]
    fn context_menu_screen_rect_clamp() {
        let menu = ContextMenu::new(sample_items())
            .with_anchor(200, 100)
            .with_width(25);
        let screen = Rect::new(0, 0, 80, 24);
        let (x, y, w, h) = menu.screen_rect(screen);
        assert!(x + w <= 80);
        assert!(y + h <= 24);
    }

    #[test]
    fn separator_item() {
        let sep = ContextMenuItem::separator();
        assert!(sep.is_separator);
        assert!(sep.id.is_empty());
    }

    #[test]
    fn item_with_icon() {
        let item = ContextMenuItem::new("open", "Open").with_icon('📂');
        assert_eq!(item.icon, Some('📂'));
    }
}

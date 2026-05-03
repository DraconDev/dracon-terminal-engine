//! Command palette widget — filterable list overlay for executing commands.
//!
//! Displays a searchable list of commands. The user types to filter,
//! navigates with arrow keys, and presses Enter to execute the selected command.

use crate::compositor::{Color, Plane, Styles};
use crate::framework::hitzone::ScopedZoneRegistry;
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use crate::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

/// A single command in the palette.
#[derive(Debug, Clone)]
pub struct CommandItem {
    /// Unique identifier for this command.
    pub id: &'static str,
    /// Display name shown in the palette.
    pub name: &'static str,
    /// Category or group label.
    pub category: &'static str,
}

/// A filterable command palette overlay.
///
/// Renders as a centered modal with a search input at the top and
/// a scrollable filtered list of commands below. Supports keyboard
/// navigation (↑/↓/Enter/Esc) and mouse clicks.
pub struct CommandPalette {
    id: WidgetId,
    commands: Vec<CommandItem>,
    search_query: String,
    selected_index: usize,
    visible: bool,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    width: u16,
    height: u16,
    on_execute: Option<Box<dyn FnMut(&str)>>,
    zones: RefCell<ScopedZoneRegistry<usize>>,
}

impl CommandPalette {
    /// Creates a new `CommandPalette` with the given commands.
    pub fn new(commands: Vec<CommandItem>) -> Self {
        Self {
            id: WidgetId::default_id(),
            commands,
            search_query: String::new(),
            selected_index: 0,
            visible: false,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 20)),
            dirty: true,
            width: 40,
            height: 20,
            on_execute: None,
            zones: RefCell::new(ScopedZoneRegistry::new()),
        }
    }

    /// Sets the widget theme.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self.dirty = true;
        self
    }

    /// Sets the palette dimensions.
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self.area.set(Rect::new(0, 0, width, height));
        self.dirty = true;
        self
    }

    /// Registers a callback invoked when a command is executed.
    /// The callback receives the command's `id`.
    pub fn on_execute<F>(mut self, f: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.on_execute = Some(Box::new(f));
        self
    }

    /// Shows the palette.
    pub fn show(&mut self) {
        self.visible = true;
        self.search_query.clear();
        self.selected_index = 0;
        self.dirty = true;
    }

    /// Hides the palette.
    pub fn hide(&mut self) {
        self.visible = false;
        self.dirty = true;
    }

    /// Returns `true` if the palette is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the filtered list of commands based on the current search query.
    fn filtered_commands(&self) -> Vec<&CommandItem> {
        if self.search_query.is_empty() {
            return self.commands.iter().collect();
        }
        let q = self.search_query.to_lowercase();
        self.commands
            .iter()
            .filter(|cmd| {
                cmd.name.to_lowercase().contains(&q) || cmd.category.to_lowercase().contains(&q)
            })
            .collect()
    }

    fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
        for (i, ch) in text.chars().enumerate() {
            let idx = (y * plane.width + x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = crate::compositor::Cell {
                    char: ch,
                    fg,
                    bg,
                    style: if bold { Styles::BOLD } else { Styles::empty() },
                    transparent: false,
                    skip: false,
                };
            }
        }
    }
}

impl crate::framework::widget::Widget for CommandPalette {
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
        100
    }

    fn needs_render(&self) -> bool {
        self.dirty && self.visible
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn focusable(&self) -> bool {
        true
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let t = self.theme;
        let mut plane = Plane::new(0, area.width, area.height);

        // Background
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        let w = self.width.min(area.width);
        let h = self.height.min(area.height);
        let ox = (area.width.saturating_sub(w)) / 2;
        let oy = (area.height.saturating_sub(h)) / 3;

        // Clear and rebuild zones
        let mut zones = self.zones.borrow_mut();
        zones.clear();

        // Palette background
        for y in oy..oy + h {
            for x in ox..ox + w {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                }
            }
        }

        // Border
        let border_chars = ['┌', '─', '┐', '│', '│', '└', '─', '┘'];
        for x in ox..ox + w {
            let t_idx = (oy * area.width + x) as usize;
            let b_idx = ((oy + h - 1) * area.width + x) as usize;
            if t_idx < plane.cells.len() {
                plane.cells[t_idx].char = if x == ox { border_chars[0] } else if x == ox + w - 1 { border_chars[2] } else { border_chars[1] };
                plane.cells[t_idx].fg = t.outline;
                plane.cells[t_idx].bg = t.surface_elevated;
            }
            if b_idx < plane.cells.len() {
                plane.cells[b_idx].char = if x == ox { border_chars[5] } else if x == ox + w - 1 { border_chars[7] } else { border_chars[6] };
                plane.cells[b_idx].fg = t.outline;
                plane.cells[b_idx].bg = t.surface_elevated;
            }
        }
        for y in oy..oy + h {
            for x in [ox, ox + w - 1] {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = if y == oy { border_chars[0] } else if y == oy + h - 1 { border_chars[5] } else { border_chars[3] };
                    plane.cells[idx].fg = t.outline;
                    plane.cells[idx].bg = t.surface_elevated;
                }
            }
        }

        // Title
        Self::draw_text(&mut plane, ox + 2, oy + 1, "Command Palette", t.primary, t.surface_elevated, true);

        // Search input
        let search_y = oy + 2;
        let search_prefix = "> ";
        Self::draw_text(&mut plane, ox + 2, search_y, search_prefix, t.primary, t.surface_elevated, true);
        if !self.search_query.is_empty() {
            Self::draw_text(&mut plane, ox + 2 + search_prefix.len() as u16, search_y, &self.search_query, t.fg, t.surface_elevated, false);
        } else {
            Self::draw_text(&mut plane, ox + 2 + search_prefix.len() as u16, search_y, "type to filter...", t.fg_muted, t.surface_elevated, false);
        }
        // Cursor
        let cursor_x = ox + 2 + search_prefix.len() as u16 + self.search_query.len() as u16;
        if cursor_x < ox + w - 2 {
            let idx = (search_y * area.width + cursor_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '▎';
                plane.cells[idx].fg = t.primary;
            }
        }

        // Separator
        let sep_y = search_y + 1;
        for x in ox + 1..ox + w - 1 {
            let idx = (sep_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline_variant;
            }
        }

        // Filtered command list
        let filtered = self.filtered_commands();
        let list_start_y = sep_y + 1;
        let max_visible = (h as usize).saturating_sub(4).min(filtered.len());
        let scroll_offset = if self.selected_index >= max_visible {
            self.selected_index.saturating_sub(max_visible - 1)
        } else {
            0
        };

        for i in 0..max_visible {
            let cmd_idx = scroll_offset + i;
            if cmd_idx >= filtered.len() {
                break;
            }
            let cmd = filtered[cmd_idx];
            let y = list_start_y + i as u16;
            let is_selected = cmd_idx == self.selected_index;

            let (fg, bg) = if is_selected {
                (t.fg_on_accent, t.primary_active)
            } else {
                (t.fg, t.surface_elevated)
            };

            // Category label
            let cat = format!("[{}]", cmd.category);
            Self::draw_text(&mut plane, ox + 2, y, &cat, t.fg_muted, bg, false);
            // Name
            let name_x = ox + 2 + cat.len() as u16 + 1;
            if name_x < ox + w - 2 {
                Self::draw_text(&mut plane, name_x, y, cmd.name, fg, bg, is_selected);
            }

            // Register zone for mouse click
            zones.register(cmd_idx, ox + 2, y, w - 4, 1);
        }

        // Count hint
        let hint_y = oy + h - 2;
        let hint_text = format!(" {} / {} items ", max_visible.min(filtered.len()), filtered.len());
        Self::draw_text(&mut plane, ox + w.saturating_sub(hint_text.len() as u16 + 2), hint_y, &hint_text, t.fg_muted, t.surface_elevated, false);

        drop(zones);
        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        if !self.visible {
            return false;
        }

        match key.code {
            KeyCode::Esc => {
                self.hide();
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = self.filtered_commands().len();
                if count > 0 {
                    self.selected_index = if self.selected_index > 0 {
                        self.selected_index - 1
                    } else {
                        count - 1
                    };
                    self.dirty = true;
                }
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = self.filtered_commands().len();
                if count > 0 {
                    self.selected_index = (self.selected_index + 1) % count;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Enter => {
                let filtered = self.filtered_commands();
                if !filtered.is_empty() {
                    let idx = self.selected_index.min(filtered.len() - 1);
                    let cmd = filtered[idx];
                    if let Some(ref mut f) = self.on_execute {
                        f(cmd.id);
                    }
                }
                self.hide();
                true
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.selected_index = 0;
                self.dirty = true;
                true
            }
            KeyCode::Char(c) => {
                if c == ':' {
                    return false;
                }
                self.search_query.push(c);
                self.selected_index = 0;
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if !self.visible {
            return false;
        }

        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let zones = self.zones.borrow();
                if let Some(cmd_idx) = zones.dispatch(col, row) {
                    self.selected_index = cmd_idx;
                    let filtered = self.filtered_commands();
                    if cmd_idx < filtered.len() {
                        let cmd = filtered[cmd_idx];
                        if let Some(ref mut f) = self.on_execute {
                            f(cmd.id);
                        }
                    }
                    drop(zones);
                    self.hide();
                    return true;
                }
                drop(zones);

                // Click outside the palette area hides it
                let (w, h) = (self.width.min(self.area.get().width), self.height.min(self.area.get().height));
                let ox = (self.area.get().width.saturating_sub(w)) / 2;
                let oy = (self.area.get().height.saturating_sub(h)) / 3;
                if col < ox || col >= ox + w || row < oy || row >= oy + h {
                    self.hide();
                    return true;
                }
            }
            MouseEventKind::ScrollDown => {
                let count = self.filtered_commands().len();
                if count > 0 {
                    self.selected_index = (self.selected_index + 1).min(count - 1);
                    self.dirty = true;
                }
                return true;
            }
            MouseEventKind::ScrollUp => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.dirty = true;
                }
                return true;
            }
            _ => {}
        }
        false
    }
}

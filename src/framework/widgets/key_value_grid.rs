//! KeyValueGrid widget — displays key-value pairs in a two-column grid.
//!
//! Binds to a CLI command that outputs JSON with string keys and values.
//! Renders as "KEY   VALUE" rows with alternating row colors.
//!
//! ## TOML definition
//!
//! ```toml
//! [[widget]]
//! id = 1
//! type = "KeyValueGrid"
//! bind = "dracon-sync info --json"
//! refresh = 5
//! separator = "  "
//! ```

use std::collections::BTreeMap;

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::command::{BoundCommand, ParsedOutput};
use crate::framework::theme::Theme;
use crate::framework::widget::{Widget, WidgetId};
use ratatui::layout::Rect;

pub struct KeyValueGrid {
    id: WidgetId,
    pairs: BTreeMap<String, String>,
    separator: String,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    bound_command: Option<BoundCommand>,
}

impl KeyValueGrid {
    pub fn new() -> Self {
        Self {
            id: WidgetId::default_id(),
            pairs: BTreeMap::new(),
            separator: "  ".to_string(),
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 60, 10)),
            dirty: true,
            bound_command: None,
        }
    }

    pub fn with_id(id: WidgetId) -> Self {
        Self { id, ..Self::new() }
    }

    pub fn separator(mut self, sep: &str) -> Self {
        self.separator = sep.to_string();
        self.dirty = true;
        self
    }

    pub fn bind_command(mut self, cmd: BoundCommand) -> Self {
        self.bound_command = Some(cmd);
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self.dirty = true;
        self
    }

    pub fn set_pairs(&mut self, pairs: BTreeMap<String, String>) {
        self.pairs = pairs;
        self.dirty = true;
    }

    pub fn update_from_output(&mut self, output: ParsedOutput) {
        match output {
            ParsedOutput::Scalar(v) => {
                self.pairs.insert("value".to_string(), v);
            }
            ParsedOutput::Text(s) => {
                for line in s.lines() {
                    if let Some((k, v)) = line.split_once(':') {
                        self.pairs
                            .insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
            }
            _ => {}
        }
        self.dirty = true;
    }

    fn render_row(
        &self,
        key: &str,
        value: &str,
        row: usize,
        area: Rect,
        key_fg: Color,
        val_fg: Color,
        alt_bg: Color,
    ) -> Vec<Cell> {
        let mut cells = Vec::with_capacity(area.width as usize);
        let key_str = format!("{}{}", key, self.separator);
        let max_key_len = 20usize;

        let display_key = if key_str.len() > max_key_len {
            format!("{}..", &key_str[..max_key_len - 2])
        } else {
            key_str.clone()
        };

        for c in display_key.chars() {
            if cells.len() < area.width as usize {
                cells.push(Cell {
                    char: c,
                    fg: key_fg,
                    bg: self.theme.bg,
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                });
            }
        }

        let remaining = (area.width as usize).saturating_sub(cells.len());
        let val_start = cells.len();
        for c in value.chars().take(remaining) {
            if cells.len() < area.width as usize {
                cells.push(Cell {
                    char: c,
                    fg: val_fg,
                    bg: self.theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                });
            }
        }

        while cells.len() < area.width as usize {
            cells.push(Cell {
                char: ' ',
                fg: self.theme.fg,
                bg: self.theme.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            });
        }

        if row % 2 == 1 {
            cells[val_start..].iter_mut().for_each(|cell| cell.bg = alt_bg);
        }

        cells
    }
}

impl Default for KeyValueGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for KeyValueGrid {
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

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);

        let alt_bg = Color::Ansi(236);
        let key_fg = self.theme.fg;
        let val_fg = self.theme.inactive_fg;

        for (row, (key, value)) in self.pairs.iter().enumerate() {
            if row >= area.height as usize {
                break;
            }
            let cells = self.render_row(key, value, row, area, key_fg, val_fg, alt_bg);
            for (i, cell) in cells.into_iter().enumerate() {
                let idx = row * area.width as usize + i;
                if idx < plane.cells.len() {
                    plane.cells[idx] = cell;
                }
            }
        }

        if self.pairs.is_empty() {
            let empty_msg = "(no data)";
            let col_start = (area.width as usize).saturating_sub(empty_msg.len()) / 2;
            let row = (area.height / 2) as usize;
            let char_index = row * (area.width as usize) + col_start;
            for (i, c) in empty_msg.chars().enumerate() {
                let idx = char_index + i;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: self.theme.inactive_fg,
                        bg: self.theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        plane
    }

    fn commands(&self) -> Vec<BoundCommand> {
        self.bound_command.iter().cloned().collect()
    }

    fn apply_command_output(&mut self, output: &crate::framework::command::ParsedOutput) {
        self.update_from_output(output.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_value_grid_new() {
        let grid = KeyValueGrid::new();
        assert!(grid.pairs.is_empty());
        assert_eq!(grid.separator, "  ");
    }

    #[test]
    fn test_key_value_grid_with_id() {
        let grid = KeyValueGrid::with_id(WidgetId::new(3));
        assert_eq!(grid.id, WidgetId::new(3));
    }

    #[test]
    fn test_key_value_grid_separator() {
        let grid = KeyValueGrid::new().separator(" : ");
        assert_eq!(grid.separator, " : ");
    }

    #[test]
    fn test_key_value_grid_bind_command() {
        let cmd = BoundCommand::new("sysinfo").label("info");
        let grid = KeyValueGrid::new().bind_command(cmd);
        assert_eq!(grid.commands().len(), 1);
    }

    #[test]
    fn test_key_value_grid_set_pairs() {
        let mut grid = KeyValueGrid::new();
        let mut pairs = BTreeMap::new();
        pairs.insert("CPU".to_string(), "i9".to_string());
        pairs.insert("RAM".to_string(), "64GB".to_string());
        grid.set_pairs(pairs);
        assert_eq!(grid.pairs.len(), 2);
    }

    #[test]
    fn test_key_value_grid_update_from_scalar() {
        let mut grid = KeyValueGrid::new();
        grid.update_from_output(ParsedOutput::Scalar("active".to_string()));
        assert_eq!(grid.pairs.get("value").unwrap(), "active");
    }

    #[test]
    fn test_key_value_grid_update_from_text() {
        let mut grid = KeyValueGrid::new();
        grid.update_from_output(ParsedOutput::Text("CPU: i9\nRAM: 64GB".to_string()));
        assert_eq!(grid.pairs.get("CPU").unwrap(), "i9");
        assert_eq!(grid.pairs.get("RAM").unwrap(), "64GB");
    }

    #[test]
    fn test_key_value_grid_update_from_text_with_colon_in_value() {
        let mut grid = KeyValueGrid::new();
        grid.update_from_output(ParsedOutput::Text("path: /usr/local/bin:stuff".to_string()));
        assert_eq!(grid.pairs.get("path").unwrap(), "/usr/local/bin:stuff");
    }

    #[test]
    fn test_key_value_grid_render() {
        let mut grid = KeyValueGrid::new();
        let mut pairs = BTreeMap::new();
        pairs.insert("Name".to_string(), "Test".to_string());
        grid.set_pairs(pairs);
        let plane = grid.render(Rect::new(0, 0, 40, 5));
        assert_eq!(plane.cells[0].char, 'N');
    }

    #[test]
    fn test_key_value_grid_render_empty() {
        let grid = KeyValueGrid::new();
        let plane = grid.render(Rect::new(0, 0, 30, 5));
        assert!(plane.cells.iter().any(|c| c.char == '('));
    }

    #[test]
    fn test_key_value_grid_dirty_lifecycle() {
        let mut grid = KeyValueGrid::new();
        assert!(grid.needs_render());
        grid.clear_dirty();
        assert!(!grid.needs_render());
        grid.set_pairs(BTreeMap::new());
        assert!(grid.needs_render());
    }

    #[test]
    fn test_key_value_grid_with_theme() {
        let theme = Theme::gruvbox_dark();
        let grid = KeyValueGrid::new().with_theme(theme);
        assert_eq!(grid.theme.name, "gruvbox-dark");
    }

    #[test]
    fn test_key_value_grid_sorted_keys() {
        let mut grid = KeyValueGrid::new();
        let mut pairs = BTreeMap::new();
        pairs.insert("zebra".to_string(), "last".to_string());
        pairs.insert("apple".to_string(), "first".to_string());
        grid.set_pairs(pairs);
        let keys: Vec<&String> = grid.pairs.keys().collect();
        assert_eq!(keys[0], "apple");
        assert_eq!(keys[1], "zebra");
    }

    #[test]
    fn test_key_value_grid_apply_command_output() {
        use crate::framework::command::ParsedOutput;
        let mut grid = KeyValueGrid::new();
        grid.apply_command_output(&ParsedOutput::Text("CPU: i9\nRAM: 64GB".to_string()));
        assert_eq!(grid.pairs.get("CPU").unwrap(), "i9");
        assert_eq!(grid.pairs.get("RAM").unwrap(), "64GB");
    }
}

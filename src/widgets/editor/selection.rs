//! TextEditor selection module.
//!
//! Contains the `Selection` struct and its methods for managing
//! text selection state within the editor.

use crate::widgets::TextEditor;

/// Selection state for the text editor.
#[derive(Debug, Clone, Default)]
pub struct Selection {
    /// Starting position of the current selection `(row, byte_col)`.
    pub start: Option<(usize, usize)>,
    /// Ending position of the current selection `(row, byte_col)`.
    pub end: Option<(usize, usize)>,
}

impl Selection {
    /// Returns the selection range as `Option<((start_row, start_col), (end_row, end_col))>`.
    pub fn range(&self) -> Option<((usize, usize), (usize, usize))> {
        let start = self.start?;
        let end = self.end?;
        if start <= end {
            Some((start, end))
        } else {
            Some((end, start))
        }
    }

    /// Returns `true` if the given position is inside the current selection.
    pub fn is_inside(&self, row: usize, byte_col: usize) -> bool {
        let (start, end) = match self.range() {
            Some(r) => r,
            None => return false,
        };

        if row < start.0 || row > end.0 {
            return false;
        }

        if row == start.0 && byte_col < start.1 {
            return false;
        }
        if row == end.0 && byte_col > end.1 {
            return false;
        }

        true
    }

    /// Returns the selected text from the editor's lines.
    pub fn get_selected_text(&self, lines: &[String]) -> Option<String> {
        let (start, end) = self.range()?;

        if start == end {
            return None;
        }

        if start.0 == end.0 {
            let line = lines.get(start.0)?;
            return Some(line[start.1..end.1].to_string());
        }

        let mut result = String::new();

        // First line: from start.1 to end of line
        if let Some(line) = lines.get(start.0) {
            result.push_str(&line[start.1..]);
        }

        // Middle lines: full lines
        for row in (start.0 + 1)..end.0 {
            if let Some(line) = lines.get(row) {
                result.push('\n');
                result.push_str(line);
            }
        }

        // Last line: from start to end.1
        if let Some(line) = lines.get(end.0) {
            result.push('\n');
            result.push_str(&line[..end.1]);
        }

        Some(result)
    }

    /// Deletes the current selection from the editor's lines.
    /// Returns the deleted text as `(text, new_cursor_row, new_cursor_col)`.
    pub fn delete_selection(
        &self,
        lines: &mut Vec<String>,
    ) -> Option<(String, usize, usize)> {
        let (start, end) = self.range()?;

        if start == end {
            return None;
        }

        let deleted_text = self.get_selected_text(lines)?;

        if start.0 == end.0 {
            // Single line deletion
            if let Some(line) = lines.get_mut(start.0) {
                line.drain(start.1..end.1);
            }
            Some((deleted_text, start.0, start.1))
        } else {
            // Multi-line deletion
            // Remove lines between start and end
            for row in (start.0 + 1..end.0).rev() {
                lines.remove(row);
            }

            // Remove from end of first line to end of that line
            if let Some(line) = lines.get_mut(start.0) {
                line.drain(start.1..);
            }
            // Remove from start of last line
            if let Some(last_line) = lines.get(end.0) {
                let remaining = last_line[end.1..].to_string();
                if let Some(first_line) = lines.get_mut(start.0) {
                    first_line.push_str(&remaining);
                }
                if end.0 > start.0 {
                    lines.remove(end.0);
                }
            }

            Some((deleted_text, start.0, start.1))
        }
    }
}
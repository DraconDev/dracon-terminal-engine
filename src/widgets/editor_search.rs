//!
//! Search, filter, and replace functionality for `TextEditor`.
//!
//! This module provides the [`SearchState`] struct which encapsulates all
//! search-related state and operations for [`TextEditor`](crate::widgets::TextEditor).
//!
//! ## Design
//!
//! `SearchState` is embedded in `TextEditor` as a public field so external code
//! can query or drive search without needing access to the full editor.
//! All methods that mutate the editor take `&mut TextEditor` as their first parameter
//! so they can read and mutate the editor's lines, cursor, and scroll state.
//!
//! ## Search Modes
//!
//! The editor supports multiple search modes driven by the [`SearchMode`]
//! enum, shown in the status bar. `Normal` mode means no active search.
//!
//! ## Filter Mode
//!
//! When a filter query is set via [`set_filter`](SearchState::set_filter),
//! `filtered_indices` maps display-row → real-line-index. Cursor navigation
//! uses display rows while content access uses real indices. Callers should
//! use `get_effective_line()`, `effective_len()`, and `get_real_line_idx()`
//! for any cursor/content operations when a filter is active.

use regex::Regex;
use std::cmp::min;
use std::mem;

/// Search mode displayed in the editor's status bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SearchMode {
    /// No active search UI.
    Normal,
    /// `/` — searching forward.
    Search,
    /// `r` — replace mode.
    Replace,
    /// `g` — goto line mode.
    GotoLine,
}

/// The result of a single find operation.
#[derive(Clone, Debug)]
pub struct SearchResult {
    /// The line index where the match was found.
    pub row: usize,
    /// The byte column within the line where the match starts.
    pub col: usize,
    /// The length of the matched text.
    pub len: usize,
}

/// All search/filter/replace state for [`TextEditor`](crate::widgets::TextEditor).
///
/// ## Usage
///
/// ```ignore
/// let mut editor = TextEditor::new();
/// editor.search.set_filter("TODO");
/// editor.search.replace_all(&mut editor, "TODO", "DONE");
/// ```
///
/// All methods that mutate the editor (`set_filter`, `replace_all`,
/// `replace_next`) take `&mut TextEditor` as their first parameter so they
/// can read and write lines, cursor position, and highlighting cache.
#[derive(Clone, Debug)]
pub struct SearchState {
    /// Current filter query string (case-insensitive unless it looks like a regex).
    pub filter_query: String,
    /// Line indices that match the current filter (display-row → real-line-index).
    /// Empty when no filter is active.
    pub filtered_indices: Vec<usize>,
    /// Current search/replace/goto mode for status bar rendering.
    pub mode: SearchMode,
    /// Current input string in search/replace/goto mode.
    pub mode_input: String,
    /// Whether we are in replace mode (`r`) vs search mode (`/`).
    pub is_replacing: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            filter_query: String::new(),
            filtered_indices: Vec::new(),
            mode: SearchMode::Normal,
            mode_input: String::new(),
            is_replacing: false,
        }
    }
}

impl SearchState {
    // ── Filter ────────────────────────────────────────────────────────────────

    /// Rebuilds `filtered_indices` from the current `filter_query`.
    ///
    /// Sets cursor to display row 0 and resets scroll after rebuilding.
    /// Calls `invalidate_from(0)` on the editor after updating.
    ///
    /// ## Regex Support
    ///
    /// If the query parses as a case-insensitive regex (`(?i)...`), uses it.
    /// Falls back to case-insensitive substring match.
    pub fn set_filter(&mut self, query: &str, editor: &mut crate::widgets::TextEditor) {
        if self.filter_query == query {
            return;
        }

        // If clearing filter, restore cursor to real line index
        if query.is_empty() && !self.filter_query.is_empty() {
            if editor.cursor_row < self.filtered_indices.len() {
                editor.cursor_row = self.filtered_indices[editor.cursor_row];
            } else if let Some(&last) = self.filtered_indices.last() {
                editor.cursor_row = last;
            } else {
                editor.cursor_row = 0;
            }
            self.filtered_indices.clear();
        }

        self.filter_query = query.to_string();

        if !self.filter_query.is_empty() {
            let use_regex = Regex::new(&format!("(?i){}", query)).is_ok();
            self.filtered_indices = editor
                .lines
                .iter()
                .enumerate()
                .filter(|(_, line)| {
                    if use_regex {
                        if let Ok(re) = Regex::new(&format!("(?i){}", query)) {
                            re.is_match(line)
                        } else {
                            line.to_lowercase()
                                .contains(&self.filter_query.to_lowercase())
                        }
                    } else {
                        line.to_lowercase()
                            .contains(&self.filter_query.to_lowercase())
                    }
                })
                .map(|(i, _)| i)
                .collect();
            editor.cursor_row = 0;
            editor.scroll_row = 0;
        }
        editor.scroll_col = 0;
        editor.cursor_col = 0;
        editor.invalidate_from(0);
    }

    /// Returns the number of visible lines (filtered or total).
    pub fn effective_len(&self, editor: &crate::widgets::TextEditor) -> usize {
        if !self.filter_query.is_empty() {
            self.filtered_indices.len()
        } else {
            editor.lines.len()
        }
    }

    /// Returns a reference to the line at display index `idx`.
    ///
    /// When filtered, returns `&editor.lines[filtered_indices[idx]]`.
    /// When not filtered, returns `&editor.lines[idx]`.
    /// Returns a fallback empty line if `idx` is out of range.
    pub fn get_effective_line(&self, editor: &crate::widgets::TextEditor, idx: usize) -> &String {
        if idx >= self.effective_len(editor) {
            return &editor.lines[0];
        } // Fallback safety
        if !self.filter_query.is_empty() {
            &editor.lines[self.filtered_indices[idx]]
        } else {
            &editor.lines[idx]
        }
    }

    /// Maps a display-row index to its real line index.
    ///
    /// When filtered, returns `filtered_indices[idx]`.
    /// When not filtered, returns `idx` unchanged.
    pub fn get_real_line_idx(&self, editor: &crate::widgets::TextEditor, idx: usize) -> usize {
        if !self.filter_query.is_empty() {
            self.filtered_indices.get(idx).copied().unwrap_or(0)
        } else {
            min(idx, editor.lines.len().saturating_sub(1))
        }
    }

    // ── Find / Replace ───────────────────────────────────────────────────────

    /// Returns `true` if `query` looks like a valid regex pattern.
    pub fn is_regex(query: &str) -> bool {
        Regex::new(query).is_ok()
    }

    /// Finds the next occurrence of `find` starting from the editor's cursor.
    ///
    /// Wraps around the end of the file. Skips the current cursor position
    /// (search starts from `cursor_col + 1` on the current line).
    ///
    /// Returns `None` if no match is found.
    pub fn find_next(&self, editor: &crate::widgets::TextEditor, find: &str) -> Option<SearchResult> {
        if find.is_empty() {
            return None;
        }

        let use_regex = Self::is_regex(find);
        let start_row = editor.cursor_row;
        let start_col = editor.cursor_col;

        for r in 0..editor.lines.len() {
            let row = (start_row + r) % editor.lines.len();
            let line = &editor.lines[row];
            let search_from = if r == 0 {
                start_col.saturating_add(1)
            } else {
                0
            };

            if search_from >= line.len() {
                continue;
            }

            if use_regex {
                if let Ok(re) = Regex::new(find) {
                    if let Some(mat) = re.find(&line[search_from..]) {
                        return Some(SearchResult {
                            row,
                            col: search_from + mat.start(),
                            len: mat.len(),
                        });
                    }
                }
            } else if let Some(col) = line[search_from..].find(find) {
                return Some(SearchResult {
                    row,
                    col: search_from + col,
                    len: find.len(),
                });
            }
        }
        None
    }

    /// Replaces all occurrences of `find` with `replace` across every line.
    ///
    /// Sets `modified = true` and invalidates the highlight cache from line 0.
    pub fn replace_all(&mut self, editor: &mut crate::widgets::TextEditor, find: &str, replace: &str) {
        if find.is_empty() {
            return;
        }
        if let Ok(re) = Regex::new(find) {
            for line in &mut editor.lines {
                *line = re.replace_all(line, replace).to_string();
            }
        } else {
            for line in &mut editor.lines {
                *line = line.replace(find, replace);
            }
        }
        editor.modified = true;
        editor.invalidate_from(0);
    }

    /// Replaces the next occurrence of `find` after the cursor with `replace`.
    ///
    /// Searches forward, wrapping around the end of the file.
    /// Moves the cursor to the end of the replacement.
    /// Returns `true` if a replacement was made.
    pub fn replace_next(&mut self, editor: &mut crate::widgets::TextEditor, find: &str, replace: &str) -> bool {
        if find.is_empty() {
            return false;
        }

        let start_row = editor.cursor_row;
        let start_col = editor.cursor_col;
        let use_regex = Self::is_regex(find);

        for r in 0..editor.lines.len() {
            let row = (start_row + r) % editor.lines.len();
            let line = &editor.lines[row];
            let search_from = if r == 0 { start_col } else { 0 };

            if search_from < line.len() {
                if use_regex {
                    if let Ok(re) = Regex::new(find) {
                        if let Some(mat) = re.find(&line[search_from..]) {
                            let actual_col = search_from + mat.start();
                            let mut new_line = line.clone();
                            new_line.replace_range(actual_col..actual_col + mat.len(), replace);
                            editor.lines[row] = new_line;

                            editor.cursor_row = row;
                            editor.cursor_col = actual_col + replace.len();
                            editor.modified = true;
                            editor.invalidate_from(0);
                            return true;
                        }
                    }
                } else if let Some(col) = line[search_from..].find(find) {
                    let actual_col = search_from + col;
                    let mut new_line = line.clone();
                    new_line.replace_range(actual_col..actual_col + find.len(), replace);
                    editor.lines[row] = new_line;

                    editor.cursor_row = row;
                    editor.cursor_col = actual_col + replace.len();
                    editor.modified = true;
                    editor.invalidate_from(0);
                    return true;
                }
            }
        }
        false
    }

    // ── Mode helpers ─────────────────────────────────────────────────────────

    /// Enters search mode (`/`), clearing mode input.
    pub fn enter_search(&mut self) {
        self.mode = SearchMode::Search;
        self.mode_input.clear();
        self.is_replacing = false;
    }

    /// Enters replace mode (`r`), clearing mode input.
    pub fn enter_replace(&mut self) {
        self.mode = SearchMode::Replace;
        self.mode_input.clear();
        self.is_replacing = true;
    }

    /// Enters goto line mode (`g`), clearing mode input.
    pub fn enter_goto_line(&mut self) {
        self.mode = SearchMode::GotoLine;
        self.mode_input.clear();
        self.is_replacing = false;
    }

    /// Returns to normal mode.
    pub fn exit_mode(&mut self) {
        self.mode = SearchMode::Normal;
        self.mode_input.clear();
    }

    /// Returns whether a search UI is currently active.
    pub fn is_active(&self) -> bool {
        self.mode != SearchMode::Normal
    }

    // ── Filter persistence helpers ───────────────────────────────────────────

    /// Swaps the filter query with a new value and rebuilds filtered_indices.
    ///
    /// Used internally to persist/restore filter state across mode transitions.
    pub fn swap_filter(&mut self, query: String) -> String {
        let prev = mem::replace(&mut self.filter_query, query);
        prev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_editor(lines: Vec<&str>) -> crate::widgets::TextEditor {
        let mut editor = crate::widgets::TextEditor::default();
        editor.lines = lines.into_iter().map(String::from).collect();
        editor
    }

    #[test]
    fn test_replace_all_exact() {
        let mut editor = make_editor(vec!["hello world", "hello again"]);
        let mut search = SearchState::default();
        search.replace_all(&mut editor, "hello", "hi");
        assert_eq!(editor.lines[0], "hi world");
        assert_eq!(editor.lines[1], "hi again");
    }

    #[test]
    fn test_replace_all_regex() {
        let mut editor = make_editor(vec!["foo123 bar", "test456"]);
        let mut search = SearchState::default();
        search.replace_all(&mut editor, r"\d+", "X");
        assert_eq!(editor.lines[0], "fooX bar");
        assert_eq!(editor.lines[1], "testX");
    }

    #[test]
    fn test_find_next_wraps() {
        let editor = make_editor(vec!["abc", "def", "ghi"]);
        let mut search = SearchState::default();
        editor.cursor_row = 2;
        editor.cursor_col = 2; // at "i"

        let result = search.find_next(&editor, "a");
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.row, 0);
        assert_eq!(r.col, 0);
    }

    #[test]
    fn test_find_next_current_line() {
        let editor = make_editor(vec!["abc", "def", "ghi"]);
        let mut search = SearchState::default();
        editor.cursor_row = 0;
        editor.cursor_col = 0; // at "a", skip it

        let result = search.find_next(&editor, "a");
        assert!(result.is_none());
    }

    #[test]
    fn test_set_filter() {
        let mut editor = make_editor(vec!["apple", "banana", "apricot", "cherry"]);
        let mut search = SearchState::default();
        search.set_filter("ap", &mut editor);
        assert_eq!(search.filtered_indices, vec![0, 2]);
        assert_eq!(search.effective_len(&editor), 2);
    }

    #[test]
    fn test_set_filter_clears() {
        let mut editor = make_editor(vec!["apple", "banana"]);
        let mut search = SearchState::default();
        search.set_filter("ap", &mut editor);
        search.set_filter("", &mut editor);
        assert!(search.filtered_indices.is_empty());
        assert_eq!(search.effective_len(&editor), 2);
    }

    #[test]
    fn test_get_real_line_idx() {
        let mut editor = make_editor(vec!["apple", "banana", "cherry"]);
        let mut search = SearchState::default();
        search.set_filter("a", &mut editor); // apple, banana
        assert_eq!(search.get_real_line_idx(&editor, 0), 0);
        assert_eq!(search.get_real_line_idx(&editor, 1), 1);
    }
}
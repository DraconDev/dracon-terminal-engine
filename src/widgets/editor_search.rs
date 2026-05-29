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

/// Search, filter, and replace functionality for `TextEditor`.
///
/// This module provides the [`SearchState`] struct which encapsulates all
/// search-related state and operations for [`TextEditor`](crate::widgets::TextEditor).
///
/// ## Usage
///
/// ```ignore
/// let mut editor = TextEditor::new();
/// editor.search.filter_query = "TODO".to_string();
/// ```
///
/// All methods that mutate the editor (`swap_filter`) take `&mut TextEditor` as their first parameter so they
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
    // ── Filter helpers (used by TextEditor::set_filter) ───────────────────────

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
    pub fn get_effective_line<'a>(
        &self,
        editor: &'a crate::widgets::TextEditor,
        idx: usize,
    ) -> &'a String {
        if !self.filter_query.is_empty() && idx < self.filtered_indices.len() {
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
        mem::replace(&mut self.filter_query, query)
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
    fn test_is_regex() {
        assert!(SearchState::is_regex(r"\d+"));
        assert!(SearchState::is_regex("hello"));
        assert!(!SearchState::is_regex("[invalid"));
    }

    #[test]
    fn test_effective_len() {
        let editor = make_editor(vec!["apple", "banana", "cherry"]);
        let search = SearchState::default();
        assert_eq!(search.effective_len(&editor), 3);
    }

    #[test]
    fn test_effective_len_filtered() {
        let editor = make_editor(vec!["apple", "banana", "apricot", "cherry"]);
        let search = SearchState {
            filter_query: "ap".to_string(),
            filtered_indices: vec![0, 2],
            ..Default::default()
        };
        // Simulate filter
        assert_eq!(search.effective_len(&editor), 2);
    }

    #[test]
    fn test_get_effective_line() {
        let editor = make_editor(vec!["apple", "banana", "cherry"]);
        let search = SearchState::default();
        assert_eq!(search.get_effective_line(&editor, 1).as_str(), "banana");
    }

    #[test]
    fn test_get_effective_line_filtered() {
        let editor = make_editor(vec!["apple", "banana", "apricot", "cherry"]);
        let search = SearchState {
            filter_query: "ap".to_string(),
            filtered_indices: vec![0, 2],
            ..Default::default()
        };
        // Display row 1 → real line 2 → "apricot"
        assert_eq!(search.get_effective_line(&editor, 1).as_str(), "apricot");
    }

    #[test]
    fn test_get_real_line_idx() {
        let editor = make_editor(vec!["apple", "banana", "cherry"]);
        let search = SearchState::default();
        assert_eq!(search.get_real_line_idx(&editor, 1), 1);
    }

    #[test]
    fn test_get_real_line_idx_filtered() {
        let editor = make_editor(vec!["apple", "banana", "cherry"]);
        let search = SearchState {
            filter_query: "a".to_string(),
            filtered_indices: vec![0, 1], // apple, banana
            ..Default::default()
        };
        assert_eq!(search.get_real_line_idx(&editor, 0), 0);
        assert_eq!(search.get_real_line_idx(&editor, 1), 1);
    }

    #[test]
    fn test_mode_helpers() {
        let mut search = SearchState::default();
        assert!(!search.is_active());

        search.enter_search();
        assert_eq!(search.mode, SearchMode::Search);
        assert!(!search.is_replacing);
        assert!(search.is_active());

        search.enter_replace();
        assert_eq!(search.mode, SearchMode::Replace);
        assert!(search.is_replacing);

        search.enter_goto_line();
        assert_eq!(search.mode, SearchMode::GotoLine);

        search.exit_mode();
        assert_eq!(search.mode, SearchMode::Normal);
        assert!(!search.is_active());
    }

    #[test]
    fn test_swap_filter() {
        let mut search = SearchState {
            filter_query: "initial".to_string(),
            ..Default::default()
        };
        let prev = search.swap_filter("replacement".to_string());
        assert_eq!(prev, "initial");
        assert_eq!(search.filter_query, "replacement");
    }
}

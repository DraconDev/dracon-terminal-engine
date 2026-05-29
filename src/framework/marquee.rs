//! Marquee (rubber-band) drag selection system.
//!
//! Provides rectangle-based drag selection for widgets with selectable rows
//! (List, Table, Tree, Kanban). Click+drag draws a selection rectangle;
//! on mouse release, all items within the rectangle are selected.
//!
//! # Interaction Model
//!
//! - **MouseDown** on empty space or a row → start tracking (`start = Some(...)`)
//! - **MouseDrag** → if distance exceeds threshold, activate marquee
//! - **MouseUp** → commit selection for all rows within the rect, then clear
//! - **Escape** or **MouseMove** (without drag) → cancel marquee
//!
//! Marquee and file drag-drop are **mutually exclusive** — when marquee
//! activates, it cancels any in-progress drag operation.
//!
//! # Deferred Click Pattern
//!
//! Plain left-clicks should NOT immediately change selection. Instead, set
//! `pending_click_idx`. On mouseUp, if no drag/marquee occurred, resolve
//! the pending click. Ctrl/Shift clicks fire immediately (not deferred).
//!
//! This prevents "marquee broken when row already selected" — the initial
//! click selection would break the marquee intent.

use crate::compositor::plane::{Plane, Styles};
use crate::framework::prelude::{Rect, Theme};

// ---------------------------------------------------------------------------
// MarqueeRect — normalized bounding rectangle
// ---------------------------------------------------------------------------

/// Normalized marquee selection rectangle.
///
/// `min_col/min_row` are always ≤ `max_col/max_row`,
/// regardless of drag direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MarqueeRect {
    pub min_col: u16,
    pub min_row: u16,
    pub max_col: u16,
    pub max_row: u16,
}

// ---------------------------------------------------------------------------
// MarqueeState — marquee drag selection state
// ---------------------------------------------------------------------------

/// State machine for marquee (rubber-band) drag selection.
///
/// # Lifecycle
///
/// ```text
/// Idle → Tracking (MouseDown)
/// Tracking → Active (Drag exceeds threshold)
/// Tracking → Idle (MouseUp without exceeding threshold → resolve pending_click)
/// Active → Idle (MouseUp → commit selection)
/// Active → Idle (Escape / MouseMove → cancel)
/// ```
///
/// # Example
///
/// ```no_run
/// use dracon_terminal_engine::framework::marquee::MarqueeState;
///
/// // Create with default 2px activation threshold
/// let mut marquee = MarqueeState::new();
///
/// // Or customize the threshold (in distance-squared)
/// let mut marquee = MarqueeState::new().with_activation_threshold(16.0); // 4px
///
/// // On MouseDown (left click):
/// let col: u16 = 10;
/// let row: u16 = 5;
/// let item_index: usize = 3;
/// marquee.start_tracking(col, row);
/// marquee.defer_click(item_index); // Plain click only
///
/// // On MouseDrag (e.g., moved 3px):
/// let just_activated = marquee.update(col + 3, row);
/// if just_activated {
///     // Marquee now active — cancel any file drag here
/// }
///
/// // On MouseUp:
/// if marquee.is_active {
///     if let Some(rect) = marquee.rect() {
///         // rect.min_row..=rect.max_row contains all rows in selection
///         let _ = rect;
///     }
///     marquee.clear();
/// } else if let Some(idx) = marquee.take_pending_click() {
///     // Resolve deferred click — no drag occurred
///     let _ = idx;
/// }
/// marquee.reset();
/// ```
#[derive(Clone, Debug)]
pub struct MarqueeState {
    /// Whether the marquee rectangle is actively being drawn.
    pub is_active: bool,

    /// Start corner of the drag (set on MouseDown).
    pub start: Option<(u16, u16)>,

    /// Current end corner of the drag (updated on MouseDrag).
    pub end: Option<(u16, u16)>,

    /// Distance-squared threshold to activate marquee (default: 4.0 = 2px).
    /// Must be less than any drag-drop threshold to ensure marquee wins.
    pub activation_threshold: f32,

    /// Deferred click index — set on MouseDown for plain clicks,
    /// resolved on MouseUp if no drag/marquee occurred.
    /// Ctrl/Shift clicks fire immediately and are NOT deferred.
    pub pending_click_idx: Option<usize>,
}

impl Default for MarqueeState {
    fn default() -> Self {
        Self {
            is_active: false,
            start: None,
            end: None,
            activation_threshold: 4.0, // 2px Euclidean distance
            pending_click_idx: None,
        }
    }
}

impl MarqueeState {
    /// Create a new marquee state with default threshold (2px).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom activation threshold (in distance-squared).
    ///
    /// Default is 4.0 (2px). For marquee to win over drag-drop,
    /// this must be **less** than the drag-drop threshold.
    pub fn with_activation_threshold(mut self, threshold: f32) -> Self {
        self.activation_threshold = threshold;
        self
    }

    /// Start tracking on MouseDown.
    ///
    /// Call this for any left-click in a selectable area (including empty space
    /// and row clicks). The marquee may or may not activate depending on drag distance.
    pub fn start_tracking(&mut self, col: u16, row: u16) {
        self.start = Some((col, row));
        self.end = Some((col, row));
        // Don't reset is_active here — it's set in update()
    }

    /// Update end position on MouseDrag.
    ///
    /// Returns `true` if the marquee just activated (crossed threshold).
    /// Returns `false` if already active or threshold not met.
    pub fn update(&mut self, col: u16, row: u16) -> bool {
        self.end = Some((col, row));

        if self.is_active {
            return false; // Already active
        }

        if let Some((sx, sy)) = self.start {
            let dist_sq = (col as f32 - sx as f32).powi(2) + (row as f32 - sy as f32).powi(2);
            if dist_sq >= self.activation_threshold {
                self.is_active = true;
                return true;
            }
        }
        false
    }

    /// Returns the normalized marquee rect, or `None` if not active.
    ///
    /// Normalization ensures min ≤ max regardless of drag direction.
    #[must_use]
    pub fn rect(&self) -> Option<MarqueeRect> {
        if !self.is_active {
            return None;
        }
        let (sx, sy) = self.start?;
        let (ex, ey) = self.end.unwrap_or((sx, sy));
        Some(MarqueeRect {
            min_col: sx.min(ex),
            min_row: sy.min(ey),
            max_col: sx.max(ex),
            max_row: sy.max(ey),
        })
    }

    /// Check if a screen row falls within the marquee rect.
    ///
    /// Returns `false` if marquee is not active.
    pub fn contains_row(&self, row: u16) -> bool {
        self.rect()
            .is_some_and(|r| row >= r.min_row && row <= r.max_row)
    }

    /// Check if a screen position falls within the marquee rect.
    ///
    /// Returns `false` if marquee is not active.
    pub fn contains(&self, col: u16, row: u16) -> bool {
        self.rect().is_some_and(|r| {
            col >= r.min_col && col <= r.max_col && row >= r.min_row && row <= r.max_row
        })
    }

    /// Clear all marquee state (cancel or after commit).
    pub fn clear(&mut self) {
        self.is_active = false;
        self.start = None;
        self.end = None;
    }

    /// Full reset including pending click.
    pub fn reset(&mut self) {
        self.clear();
        self.pending_click_idx = None;
    }

    /// Defer a click for resolution on mouseUp.
    ///
    /// Only call this for plain (non-Ctrl, non-Shift) left clicks.
    /// Ctrl/Shift clicks should fire immediately instead.
    pub fn defer_click(&mut self, idx: usize) {
        self.pending_click_idx = Some(idx);
    }

    /// Take the pending click index (consumes it).
    ///
    /// Returns `Some(idx)` if there's a deferred click to resolve,
    /// `None` if no click was deferred.
    pub fn take_pending_click(&mut self) -> Option<usize> {
        self.pending_click_idx.take()
    }

    /// Whether we are in tracking mode (start is set but marquee not yet active).
    pub fn is_tracking(&self) -> bool {
        self.start.is_some() && !self.is_active
    }
}

// ---------------------------------------------------------------------------
// Marquee rendering
// ---------------------------------------------------------------------------

/// Render a marquee selection rectangle onto a plane.
///
/// Draws a **border-only** rounded rectangle with the theme's primary color.
/// The background is transparent — content underneath remains visible.
///
/// Returns `true` if the marquee was rendered, `false` if not active.
///
/// # Example
///
/// ```no_run
/// use dracon_terminal_engine::framework::marquee::{MarqueeState, render_marquee};
/// use dracon_terminal_engine::prelude::*;
///
/// fn draw_selection(plane: &mut Plane, marquee: &MarqueeState, theme: &Theme) {
///     if marquee.is_active {
///         // Draw the rubber-band selection rectangle
///         let _ = render_marquee(plane, marquee, theme);
///     }
/// }
///
/// // Typical usage in a tick callback:
/// fn on_render(plane: &mut Plane, marquee: &MarqueeState, theme: &Theme) {
///     render_marquee(plane, marquee, theme);
/// }
/// ```
pub fn render_marquee(plane: &mut Plane, marquee: &MarqueeState, theme: &Theme) -> bool {
    let Some(rect) = marquee.rect() else {
        return false;
    };

    let area = Rect::new(0, 0, plane.width, plane.height);

    // Clamp to screen bounds
    let x = rect.min_col.min(area.width.saturating_sub(1));
    let y = rect.min_row.min(area.height.saturating_sub(1));
    let w = rect.max_col.saturating_sub(rect.min_col).saturating_add(1);
    let h = rect.max_row.saturating_sub(rect.min_row).saturating_add(1);
    let w = w.min(area.width.saturating_sub(x));
    let h = h.min(area.height.saturating_sub(y));

    if w < 2 || h < 2 {
        return false;
    }

    let fg = theme.primary;

    // Top border
    for cx in x..x + w {
        let idx = (y as usize) * plane.width as usize + cx as usize;
        if idx < plane.cells.len() {
            let ch = if cx == x {
                '╭'
            } else if cx == x + w - 1 {
                '╮'
            } else {
                '─'
            };
            plane.cells[idx].char = ch;
            plane.cells[idx].fg = fg;
            plane.cells[idx].style = Styles::BOLD;
            plane.cells[idx].transparent = false;
        }
    }

    // Bottom border
    let by = y + h - 1;
    for cx in x..x + w {
        let idx = (by as usize) * plane.width as usize + cx as usize;
        if idx < plane.cells.len() {
            let ch = if cx == x {
                '╰'
            } else if cx == x + w - 1 {
                '╯'
            } else {
                '─'
            };
            plane.cells[idx].char = ch;
            plane.cells[idx].fg = fg;
            plane.cells[idx].style = Styles::BOLD;
            plane.cells[idx].transparent = false;
        }
    }

    // Side borders
    for cy in (y + 1)..by {
        let left = (cy as usize) * plane.width as usize + x as usize;
        let right = (cy as usize) * plane.width as usize + (x + w - 1) as usize;
        if left < plane.cells.len() {
            plane.cells[left].char = '│';
            plane.cells[left].fg = fg;
            plane.cells[left].style = Styles::BOLD;
            plane.cells[left].transparent = false;
        }
        if right < plane.cells.len() {
            plane.cells[right].char = '│';
            plane.cells[right].fg = fg;
            plane.cells[right].style = Styles::BOLD;
            plane.cells[right].transparent = false;
        }
    }

    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marquee_default_inactive() {
        let m = MarqueeState::new();
        assert!(!m.is_active);
        assert!(m.start.is_none());
        assert!(m.end.is_none());
        assert!(m.rect().is_none());
    }

    #[test]
    fn marquee_start_tracking() {
        let mut m = MarqueeState::new();
        m.start_tracking(5, 3);
        assert!(m.start.is_some());
        assert!(m.end.is_some());
        assert!(!m.is_active); // Not yet — threshold not met
        assert!(m.is_tracking());
    }

    #[test]
    fn marquee_activates_on_threshold() {
        let mut m = MarqueeState::new();
        m.start_tracking(5, 5);
        assert!(!m.is_active);

        // Move 1px — below threshold
        let activated = m.update(6, 5);
        assert!(!activated);
        assert!(!m.is_active);

        // Move 2px — meets threshold (dist_sq = 4.0)
        let activated = m.update(7, 5);
        assert!(activated);
        assert!(m.is_active);
        assert!(!m.is_tracking());
    }

    #[test]
    fn marquee_rect_normalized() {
        let mut m = MarqueeState::new();
        m.start_tracking(10, 5);
        m.update(3, 8); // Drag left and down
        let rect = m.rect().unwrap();
        assert_eq!(rect.min_col, 3);
        assert_eq!(rect.min_row, 5);
        assert_eq!(rect.max_col, 10);
        assert_eq!(rect.max_row, 8);
    }

    #[test]
    fn marquee_rect_same_point() {
        let mut m = MarqueeState::new();
        m.start_tracking(5, 5);
        // Don't drag enough to activate
        assert!(m.rect().is_none());
    }

    #[test]
    fn marquee_rect_active_same_point() {
        let mut m = MarqueeState::new();
        m.start_tracking(5, 5);
        // Force active
        m.is_active = true;
        let rect = m.rect().unwrap();
        assert_eq!(rect.min_col, 5);
        assert_eq!(rect.max_col, 5);
    }

    #[test]
    fn marquee_clear() {
        let mut m = MarqueeState::new();
        m.start_tracking(5, 5);
        m.update(10, 10);
        assert!(m.is_active);
        m.clear();
        assert!(!m.is_active);
        assert!(m.start.is_none());
        assert!(m.end.is_none());
    }

    #[test]
    fn marquee_contains_row() {
        let mut m = MarqueeState::new();
        m.start_tracking(0, 3);
        m.update(80, 10);
        assert!(m.contains_row(3));
        assert!(m.contains_row(7));
        assert!(m.contains_row(10));
        assert!(!m.contains_row(2));
        assert!(!m.contains_row(11));
    }

    #[test]
    fn marquee_contains_point() {
        let mut m = MarqueeState::new();
        m.start_tracking(2, 3);
        m.update(40, 10);
        assert!(m.contains(10, 5));
        assert!(!m.contains(1, 5)); // Outside left
        assert!(!m.contains(10, 2)); // Above top
    }

    #[test]
    fn deferred_click() {
        let mut m = MarqueeState::new();
        m.defer_click(42);
        assert_eq!(m.pending_click_idx, Some(42));
        assert_eq!(m.take_pending_click(), Some(42));
        assert_eq!(m.pending_click_idx, None);
        assert_eq!(m.take_pending_click(), None);
    }

    #[test]
    fn reset_clears_all() {
        let mut m = MarqueeState::new();
        m.start_tracking(5, 5);
        m.update(10, 10);
        m.defer_click(3);
        m.reset();
        assert!(!m.is_active);
        assert!(m.start.is_none());
        assert!(m.pending_click_idx.is_none());
    }

    #[test]
    fn custom_threshold() {
        let m = MarqueeState::new().with_activation_threshold(9.0); // 3px
        assert_eq!(m.activation_threshold, 9.0);
    }
}

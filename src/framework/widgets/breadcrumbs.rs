//! Breadcrumb navigation widget.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Plane, Styles};
use crate::framework::hitzone::HitZone;
use crate::framework::theme::Theme;
use crate::framework::widget::{WidgetId, WidgetState};
use ratatui::layout::Rect;
use std::path::Path;

/// Callback type for navigation events.
pub type NavigateCallback = Box<dyn FnMut(usize)>;

/// A path breadcrumb navigation widget.
///
/// Renders a "/"-separated sequence of clickable path segments. Clicking a segment
/// fires the `on_navigate` callback with the segment index.
pub struct Breadcrumbs {
    id: WidgetId,
    segments: Vec<String>,
    theme: Theme,
    #[allow(dead_code)]
    height: u16,
    clickable: bool,
    on_navigate: Option<NavigateCallback>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Breadcrumbs {
    /// Creates a `Breadcrumbs` from a list of segment strings.
    pub fn new(segments: Vec<String>) -> Self {
        Self {
            id: WidgetId::next(),
            segments,
            theme: Theme::default(),
            height: 1,
            clickable: true,
            on_navigate: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 1)),
            dirty: true,
        }
    }

    /// Creates a `Breadcrumbs` with the given widget ID and segment strings.
    pub fn new_with_id(id: WidgetId, segments: Vec<String>) -> Self {
        Self {
            id,
            segments,
            theme: Theme::default(),
            height: 1,
            clickable: true,
            on_navigate: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 1)),
            dirty: true,
        }
    }

    /// Creates a `Breadcrumbs` from a `Path`, splitting each component into a segment.
    pub fn from_path(path: &Path) -> Self {
        let segments: Vec<String> = path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
            .collect();
        Self {
            id: WidgetId::next(),
            segments,
            theme: Theme::default(),
            height: 1,
            clickable: true,
            on_navigate: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 1)),
            dirty: true,
        }
    }

    /// Sets the theme for rendering.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Registers a callback invoked when the user clicks a breadcrumb segment.
    /// The callback receives the zero-based index of the clicked segment.
    pub fn on_navigate<F>(mut self, f: F) -> Self
    where
        F: FnMut(usize) + 'static,
    {
        self.on_navigate = Some(Box::new(f));
        self
    }

    /// Sets whether the breadcrumb segments are clickable.
    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }
}

impl crate::framework::widget::Widget for Breadcrumbs {
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

        // Calculate total width needed
        let total_width: usize = self.segments.iter()
            .map(|s| s.width() + 2) // +2 for padding
            .sum::<usize>()
            .saturating_sub(1); // Subtract separator overhead

        // If too wide, truncate middle segments
        let showable_segments = if total_width > area.width as usize && self.segments.len() > 2 {
            let ellipsis = "...".to_string();
            let ellipsis_width = ellipsis.width();

            // Find how many segments we can show
            let prefix = 1; // Always show first
            let suffix = 1; // Always show last

            // Calculate how many segments can fit
            let max_width = area.width as usize;
            let mut used_width = 0;
            let mut show_suffix = suffix;

            // Add prefix segments
            for i in 0..prefix {
                used_width += self.segments[i].width() + 2;
            }

            // Add suffix segments
            for i in (self.segments.len() - suffix)..self.segments.len() {
                used_width += self.segments[i].width() + 2;
            }

            // Add ellipsis
            used_width += ellipsis_width + 1;

            // Try to add middle segments until we hit the limit
            for i in prefix..(self.segments.len() - suffix) {
                let seg_width = self.segments[i].width() + 2;
                if used_width + seg_width <= max_width {
                    show_suffix += 1;
                    used_width += seg_width;
                } else {
                    break;
                }
            }

            Some((prefix, self.segments.len() - show_suffix))
        } else {
            None
        };

        let (omit_start, omit_end) = showable_segments.unwrap_or((0, self.segments.len()));

        let mut x: u16 = 0;

        // Render prefix segments
        for i in 0..omit_start {
            self.render_segment(&mut plane, i, area.width, &mut x, true);
        }

        // Render ellipsis if needed
        if omit_start < omit_end {
            let ellipsis = "...";
            for ch in ellipsis.chars() {
                let idx = x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = self.theme.fg_muted;
                }
                x += 1;
            }
            // Separator after ellipsis
            if x < area.width && omit_end < self.segments.len() {
                let sep_idx = x as usize;
                if sep_idx < plane.cells.len() {
                    plane.cells[sep_idx].char = '/';
                    plane.cells[sep_idx].fg = self.theme.fg_muted;
                }
                x += 1;
            }
        }

        // Render suffix segments
        for i in omit_end..self.segments.len() {
            self.render_segment(&mut plane, i, area.width, &mut x, false);
        }

        plane
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        if !self.clickable {
            return false;
        }
        if row != 0 {
            return false;
        }
        for zone in self.zones(self.area.get().width) {
            if zone.contains(col, row) {
                if let crate::input::event::MouseEventKind::Down(
                    crate::input::event::MouseButton::Left,
                ) = kind
                {
                    if let Some(f) = self.on_navigate.as_mut() {
                        f(zone.id);
                    }
                    return true;
                }
            }
        }
        false
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = theme.clone();
    }
}

impl WidgetState for Breadcrumbs {
    fn state_id(&self) -> Option<&str> { None }
    fn to_json(&self) -> serde_json::Value { serde_json::json!({}) }
    fn apply_json(&mut self, _json: &serde_json::Value) -> Result<(), crate::error::DraconError> { Ok(()) }
}

impl Breadcrumbs {
    fn render_segment(&self, plane: &mut Plane, i: usize, _width: u16, x: &mut u16, _is_prefix: bool) {
        let segment = &self.segments[i];
        let is_last = i == self.segments.len() - 1;
        let is_first = i == 0;

        let seg_width = (segment.width() as u16 + 2).min((80u16).saturating_sub(*x));
        if seg_width < 3 {
            return;
        }

        let fg = if is_last {
            self.theme.primary
        } else {
            self.theme.fg
        };
        let style = if is_last {
            Styles::BOLD
        } else {
            Styles::empty()
        };

        // Background
        for col in 0..seg_width {
            let idx = (*x + col) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = if is_last {
                    self.theme.primary_active
                } else {
                    self.theme.bg
                };
                plane.cells[idx].fg = fg;
                plane.cells[idx].char = ' ';
            }
        }

        // Separator
        if !is_first {
            let sep_idx = *x as usize;
            if sep_idx < plane.cells.len() {
                plane.cells[sep_idx].char = '/';
                plane.cells[sep_idx].fg = self.theme.fg_muted;
            }
            *x += 1;
        }

        // Segment text
        for (j, ch) in segment.chars().enumerate() {
            if j as u16 >= seg_width - 2 {
                break;
            }
            let idx = (*x as usize) + j;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = fg;
                plane.cells[idx].style = style;
            }
        }

        *x += seg_width;
    }

    fn zones(&self, width: u16) -> Vec<HitZone<usize>> {
        let mut zones = Vec::new();
        let mut x: u16 = 0;

        for (i, segment) in self.segments.iter().enumerate() {
            let is_first = i == 0;

            let seg_width = (segment.width() as u16 + 2).min(width.saturating_sub(x));
            if seg_width < 3 {
                break;
            }

            zones.push(HitZone::new(i, x, 0, seg_width, 1));

            if !is_first {
                x += 1;
            }
            x += seg_width;
        }

        zones
    }
}
//! Breadcrumb navigation widget.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Plane, Styles};
use crate::framework::hitzone::HitZone;
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;
use std::path::Path;

/// A path breadcrumb navigation widget.
///
/// Renders a "/"-separated sequence of clickable path segments. Clicking a segment
/// fires the `on_navigate` callback with the segment index.
pub struct Breadcrumbs {
    id: WidgetId,
    segments: Vec<String>,
    theme: Theme,
    height: u16,
    on_navigate: Option<Box<dyn FnMut(usize)>>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Breadcrumbs {
    /// Creates a `Breadcrumbs` from a list of segment strings.
    pub fn new(segments: Vec<String>) -> Self {
        Self {
            id: WidgetId::default_id(),
            segments,
            theme: Theme::default(),
            height: 1,
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
            id: WidgetId::default_id(),
            segments,
            theme: Theme::default(),
            height: 1,
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

        let mut x: u16 = 0;

        for (i, segment) in self.segments.iter().enumerate() {
            let is_last = i == self.segments.len() - 1;
            let is_first = i == 0;

            let seg_width = (segment.width() as u16 + 2).min(area.width.saturating_sub(x));
            if seg_width < 3 {
                break;
            }

            let _zone = HitZone::new(i, x, area.y, seg_width, self.height);

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

            for col in 0..seg_width {
                let idx = (x + col) as usize;
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

            if !is_first {
                let sep_idx = x as usize;
                if sep_idx < plane.cells.len() {
                    plane.cells[sep_idx].char = '/';
                    plane.cells[sep_idx].fg = self.theme.fg_muted;
                }
                x += 1;
            }

            for (j, ch) in segment.chars().enumerate() {
                if j as u16 >= seg_width - 2 {
                    break;
                }
                let idx = x as usize + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].style = style;
                }
            }

            x += seg_width;
            if x >= area.width {
                break;
            }
        }

        plane
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
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
}

impl Breadcrumbs {
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

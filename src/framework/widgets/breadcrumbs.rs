use crate::framework::hitzone::HitZone;
use crate::framework::theme::Theme;
use crate::compositor::{Plane, Styles};
use std::path::Path;
use ratatui::layout::Rect;

pub struct Breadcrumbs {
    segments: Vec<String>,
    theme: Theme,
    height: u16,
    on_navigate: Option<Box<dyn FnMut(usize)>>,
}

impl Breadcrumbs {
    pub fn new(segments: Vec<String>) -> Self {
        Self {
            segments,
            theme: Theme::default(),
            height: 1,
            on_navigate: None,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        let segments: Vec<String> = path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
            .collect();
        Self {
            segments,
            theme: Theme::default(),
            height: 1,
            on_navigate: None,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn on_navigate<F>(mut self, f: F) -> Self
    where
        F: FnMut(usize) + 'static,
    {
        self.on_navigate = Some(Box::new(f));
        self
    }

    pub fn render(&self, area: Rect) -> (Plane, Vec<HitZone<usize>>) {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let mut zones = Vec::new();
        let mut x: u16 = 0;

        for (i, segment) in self.segments.iter().enumerate() {
            let is_last = i == self.segments.len() - 1;
            let is_first = i == 0;

            let seg_width = (segment.len() as u16 + 2).min(area.width.saturating_sub(x));
            if seg_width < 3 {
                break;
            }

            let rect = Rect::new(x, area.y, seg_width, self.height);
            zones.push(HitZone::new(i, x, area.y, seg_width, self.height));

            let fg = if is_last {
                self.theme.accent
            } else {
                self.theme.fg
            };
            let style = if is_last { Styles::BOLD } else { Styles::empty() };

            for col in 0..seg_width {
                let idx = col as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = if is_last { self.theme.active_bg } else { self.theme.bg };
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].char = ' ';
                }
            }

            if !is_first {
                let sep_idx = x as usize;
                if sep_idx < plane.cells.len() {
                    plane.cells[sep_idx].char = '/';
                    plane.cells[sep_idx].fg = self.theme.inactive_fg;
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

        (plane, zones)
    }

    pub fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> Option<usize> {
        if row != 0 {
            return None;
        }
        for zone in self.zones() {
            if zone.contains(col, row) {
                match kind {
                    crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                        if let Some(f) = self.on_navigate.as_mut() {
                            f(zone.id);
                        }
                        return Some(zone.id);
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn zones(&self) -> Vec<HitZone<usize>> {
        let mut zones = Vec::new();
        let mut x: u16 = 0;

        for (i, segment) in self.segments.iter().enumerate() {
            let is_last = i == self.segments.len() - 1;
            let is_first = i == 0;

            let seg_width = (segment.len() as u16 + 2).min(80u16.saturating_sub(x));
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
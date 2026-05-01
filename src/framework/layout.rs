//! Constraint-based layout engine.
//!
//! Computes widget rectangles from constraint specifications (percentage,
//! fixed, min, max, ratio). Inspired by CSS flexbox and ratatui's Layout.

use ratatui::layout::Rect;

/// A constraint that defines how a dimension is sized.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constraint {
    /// Size as a percentage of the available space (0-100).
    Percentage(u16),
    /// Fixed size in cells.
    Fixed(u16),
    /// Minimum size in cells.
    Min(u16),
    /// Maximum size in cells.
    Max(u16),
    /// Ratio of remaining space after fixed/min constraints (numerator/denominator).
    Ratio(u16, u16),
}

impl Constraint {
    /// Resolves a constraint against the available space, given fixed amounts already consumed.
    pub fn resolve(self, available: u16, fixed_consumed: u16) -> u16 {
        let remaining = available.saturating_sub(fixed_consumed);
        match self {
            Constraint::Percentage(p) => (remaining as u32 * p as u32 / 100) as u16,
            Constraint::Fixed(f) => f.min(remaining),
            Constraint::Min(m) => m.max(remaining),
            Constraint::Max(m) => m.min(remaining),
            Constraint::Ratio(num, den) => {
                if den == 0 {
                    return remaining;
                }
                (remaining as u32 * num as u32 / den as u32) as u16
            }
        }
    }
}

/// Lays out child constraints into rectangles based on available space.
pub struct Layout {
    constraints: Vec<Constraint>,
    spacing: u16,
}

impl Layout {
    /// Creates a new Layout with the given constraints.
    pub fn new(constraints: Vec<Constraint>) -> Self {
        Self {
            constraints,
            spacing: 0,
        }
    }

    /// Sets the spacing between children in cells.
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Lays out children into rectangles within the given `area`.
    ///
    /// Returns a `Vec<Rect>`, one per constraint, in order.
    ///
    /// Constraints are resolved in two passes:
    /// 1. Fixed, Min, and Max constraints are pre-allocated.
    /// 2. Percentage and Ratio constraints share the remainder.
    pub fn layout(&self, area: Rect) -> Vec<Rect> {
        if self.constraints.is_empty() {
            return Vec::new();
        }

        let total_spacing = self.spacing * (self.constraints.len() as u16 - 1).saturating_sub(0);
        let available = area.width.saturating_sub(total_spacing);

        let mut results = Vec::with_capacity(self.constraints.len());

        let mut fixed_total: u32 = 0;
        let mut percentages: Vec<(usize, u16)> = Vec::new();
        let mut ratios: Vec<(usize, u16, u16)> = Vec::new();

        for (i, c) in self.constraints.iter().enumerate() {
            match c {
                Constraint::Fixed(f) => fixed_total += *f as u32,
                Constraint::Min(m) => fixed_total += *m as u32,
                Constraint::Max(_) => {}
                Constraint::Percentage(p) => percentages.push((i, *p)),
                Constraint::Ratio(n, d) => ratios.push((i, *n, *d)),
            }
        }

        let remaining = available.saturating_sub(fixed_total as u16);

        let mut sizes = vec![0u16; self.constraints.len()];

        for (i, c) in self.constraints.iter().enumerate() {
            match c {
                Constraint::Fixed(f) => sizes[i] = *f,
                Constraint::Min(m) => sizes[i] = (*m).min(remaining),
                Constraint::Max(max) => {
                    let computed = if let Some(idx) = percentages.iter().position(|(j, _)| *j == i) {
                        let p = percentages[idx].1;
                        (remaining as u32 * p as u32 / 100) as u16
                    } else if let Some(idx) = ratios.iter().position(|(j, _, _)| *j == i) {
                        let (n, d) = (ratios[idx].1, ratios[idx].2);
                        if d > 0 {
                            (remaining as u32 * n as u32 / d as u32) as u16
                        } else {
                            remaining
                        }
                    } else {
                        remaining
                    };
                    sizes[i] = computed.min(*max);
                }
                Constraint::Percentage(_) => {}
                Constraint::Ratio(_, _) => {}
            }
        }

        let percentage_total: u16 = percentages.iter().map(|(_, p)| p).sum();
        let pct_len = percentages.len();
        for (i, p) in percentages.iter() {
            let size = if percentage_total > 0 {
                (remaining as u32 * *p as u32 / percentage_total as u32) as u16
            } else {
                remaining.saturating_div(pct_len as u16)
            };
            sizes[*i] = sizes[*i].max(size);
        }

        let ratio_total: u32 = ratios.iter().map(|(_, n, _)| *n as u32).sum();
        for (i, n, d) in ratios.iter() {
            if *d > 0 && ratio_total > 0 {
                let size = (remaining as u32 * *n as u32 / ratio_total) as u16;
                sizes[*i] = sizes[*i].max(size);
            }
        }

        let mut x = area.x;
        for (i, size) in sizes.iter().enumerate() {
            let h = area.height;
            results.push(Rect::new(x, area.y, *size, h));
            if i < sizes.len() - 1 {
                x += *size + self.spacing;
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage_layout() {
        let layout = Layout::new(vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].width, 50);
        assert_eq!(rects[1].width, 50);
    }

    #[test]
    fn test_fixed_and_percentage() {
        let layout = Layout::new(vec![
            Constraint::Fixed(20),
            Constraint::Percentage(80),
        ]);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects[0].width, 20);
        assert_eq!(rects[1].width, 80);
    }

    #[test]
    fn test_min_constraint() {
        let layout = Layout::new(vec![
            Constraint::Min(30),
            Constraint::Percentage(50),
        ]);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects[0].width, 30);
        assert_eq!(rects[1].width, 70);
    }

    #[test]
    fn test_ratio() {
        let layout = Layout::new(vec![
            Constraint::Ratio(1, 3),
            Constraint::Ratio(2, 3),
        ]);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects[0].width, 33);
        assert_eq!(rects[1].width, 66);
    }

    #[test]
    fn test_spacing() {
        let layout = Layout::new(vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).spacing(5);
        let rects = layout.layout(Rect::new(0, 0, 105, 20));
        assert_eq!(rects[0].width, 50);
        assert_eq!(rects[1].width, 50);
        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[1].x, 55);
    }

    #[test]
    fn test_max_constraint() {
        let layout = Layout::new(vec![
            Constraint::Fixed(50),
            Constraint::Max(20),
        ]);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects[0].width, 50);
        assert_eq!(rects[1].width, 20);
    }

    #[test]
    fn test_empty_layout() {
        let layout = Layout::new(vec![]);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert!(rects.is_empty());
    }
}
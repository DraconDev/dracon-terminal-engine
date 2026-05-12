//! Constraint-based layout engine.
//!
//! Computes widget rectangles from constraint specifications (percentage,
//! fixed, min, max, ratio). Inspired by CSS flexbox and ratatui's Layout.

#[cfg(test)]
use proptest::prelude::*;

use ratatui::layout::Rect;

/// Axis along which constraints are resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Distribute constraints horizontally (default).
    #[default]
    Horizontal,
    /// Distribute constraints vertically.
    Vertical,
}

/// A constraint that defines how a dimension is sized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constraint {
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
#[derive(Clone, Debug)]
pub struct Layout {
    constraints: Vec<Constraint>,
    direction: Direction,
    spacing: u16,
    margin: u16,
    name: Option<&'static str>,
}

impl Layout {
    /// Creates a new Layout with horizontal direction.
    pub fn new(constraints: Vec<Constraint>) -> Self {
        Self {
            constraints,
            direction: Direction::Horizontal,
            spacing: 0,
            margin: 0,
            name: None,
        }
    }

    /// Creates a new Layout with horizontal direction (alias for `new()`).
    pub fn horizontal(constraints: Vec<Constraint>) -> Self {
        Self::new(constraints)
    }

    /// Creates a new Layout with vertical direction.
    pub fn vertical(constraints: Vec<Constraint>) -> Self {
        Self {
            constraints,
            direction: Direction::Vertical,
            spacing: 0,
            margin: 0,
            name: None,
        }
    }

    /// Creates a sub-layout from a child rect.
    pub fn nested(&self, _rect: Rect) -> Layout {
        Layout {
            constraints: self.constraints.clone(),
            direction: self.direction,
            spacing: 0, // Nested layouts don't apply spacing to children
            margin: 0,   // Nested layouts don't inherit margin by default
            name: self.name,
        }
    }

    /// Sets the layout direction (horizontal or vertical).
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the spacing between children in cells.
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the outer margin (padding) around the layout area.
    pub fn margin(mut self, margin: u16) -> Self {
        self.margin = margin;
        self
    }

    /// Sets a debug label for the layout.
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
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

        let is_vertical = self.direction == Direction::Vertical;
        let main_axis = if is_vertical { area.height } else { area.width };
        let cross_axis = if is_vertical { area.width } else { area.height };

        // Apply margin to both axes
        let main_axis = main_axis.saturating_sub(2 * self.margin);
        let cross_axis = cross_axis.saturating_sub(2 * self.margin);
        let main_start = if is_vertical { area.y + self.margin } else { area.x + self.margin };
        let cross_start = if is_vertical { area.x + self.margin } else { area.y + self.margin };

        let total_spacing = self.spacing * (self.constraints.len() as u16 - 1).saturating_sub(0);
        let available = main_axis.saturating_sub(total_spacing);

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
                    let computed = if let Some(idx) = percentages.iter().position(|(j, _)| *j == i)
                    {
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

        let mut pos = main_start;
        for (i, size) in sizes.iter().enumerate() {
            let rect = if is_vertical {
                Rect::new(cross_start, pos, cross_axis, *size)
            } else {
                Rect::new(pos, cross_start, *size, cross_axis)
            };
            results.push(rect);
            if i < sizes.len() - 1 {
                pos += *size + self.spacing;
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
        let layout =
            Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].width, 50);
        assert_eq!(rects[1].width, 50);
    }

    #[test]
    fn test_vertical_constructor() {
        let layout =
            Layout::vertical(vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
        let rects = layout.layout(Rect::new(0, 0, 100, 40));
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].height, 20);
        assert_eq!(rects[1].height, 20);
    }

    #[test]
    fn test_nested_layout() {
        let parent = Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .spacing(2);
        let child_rect = parent.layout(Rect::new(0, 0, 100, 20))[0];
        let nested = parent.nested(child_rect);
        let nested_rects = nested.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(nested_rects.len(), 2);
        assert_eq!(nested_rects[0].width, 50);
        assert_eq!(nested_rects[1].width, 50);
    }

    #[test]
    fn test_margin() {
        let layout = Layout::horizontal(vec![Constraint::Percentage(100)]).margin(5);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects[0].x, 5);
        assert_eq!(rects[0].y, 5);
        assert_eq!(rects[0].width, 90);
        assert_eq!(rects[0].height, 10);
    }

    #[test]
    fn test_margin_vertical() {
        let layout = Layout::vertical(vec![Constraint::Percentage(100)]).margin(3);
        let rects = layout.layout(Rect::new(0, 0, 80, 50));
        assert_eq!(rects[0].x, 3);
        assert_eq!(rects[0].y, 3);
        assert_eq!(rects[0].width, 74);
        assert_eq!(rects[0].height, 44);
    }

    #[test]
    fn test_spacing_with_margin() {
        let layout = Layout::horizontal(vec![Constraint::Fixed(20), Constraint::Fixed(20)])
            .spacing(5)
            .margin(10);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects[0].x, 10);
        assert_eq!(rects[1].x, 35);
        assert_eq!(rects[0].width, 20);
        assert_eq!(rects[1].width, 20);
    }

    #[test]
    fn test_nested_with_spacing() {
        let parent = Layout::horizontal(vec![Constraint::Fixed(50), Constraint::Fixed(50)])
            .spacing(2);
        let child_rect = parent.layout(Rect::new(0, 0, 102, 20))[0];
        let nested = parent.nested(child_rect).spacing(1);
        let nested_rects = nested.layout(Rect::new(0, 0, 50, 20));
        assert_eq!(nested_rects.len(), 2);
        assert_eq!(nested_rects[0].x, 0);
        assert_eq!(nested_rects[1].x, 51); // spacing=1 between two Fixed(50) in available=49
    }

    #[test]
    fn test_layout_debug_name() {
        let layout = Layout::horizontal(vec![Constraint::Percentage(100)]).name("main");
        let debug_str = format!("{:?}", layout);
        assert!(debug_str.contains("main"));
    }

    #[test]
    fn test_fixed_and_percentage() {
        let layout =
            Layout::horizontal(vec![Constraint::Fixed(20), Constraint::Percentage(80)]);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects[0].width, 20);
        assert_eq!(rects[1].width, 80);
    }

    #[test]
    fn test_horizontal_alias() {
        let layout1 = Layout::new(vec![Constraint::Percentage(100)]);
        let layout2 = Layout::horizontal(vec![Constraint::Percentage(100)]);
        let rects1 = layout1.layout(Rect::new(0, 0, 100, 20));
        let rects2 = layout2.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects1, rects2);
    }

    #[test]
    fn test_min_constraint() {
        let layout =
            Layout::horizontal(vec![Constraint::Min(30), Constraint::Percentage(50)]);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects[0].width, 30);
        assert_eq!(rects[1].width, 70);
    }

    #[test]
    fn test_ratio() {
        let layout = Layout::horizontal(vec![Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)]);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects[0].width, 33);
        assert_eq!(rects[1].width, 66);
    }

    #[test]
    fn test_spacing() {
        let layout = Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .spacing(5);
        let rects = layout.layout(Rect::new(0, 0, 105, 20));
        assert_eq!(rects[0].width, 50);
        assert_eq!(rects[1].width, 50);
        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[1].x, 55);
    }

    #[test]
    fn test_max_constraint() {
        let layout = Layout::horizontal(vec![Constraint::Fixed(50), Constraint::Max(20)]);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert_eq!(rects[0].width, 50);
        assert_eq!(rects[1].width, 20);
    }

    #[test]
    fn test_vertical_layout() {
        let layout = Layout::vertical(vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
        let rects = layout.layout(Rect::new(0, 0, 100, 40));
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].height, 20);
        assert_eq!(rects[1].height, 20);
        assert_eq!(rects[0].width, 100);
        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].y, 0);
        assert_eq!(rects[1].y, 20);
    }

    #[test]
    fn test_vertical_with_spacing() {
        let layout = Layout::vertical(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .spacing(2);
        let rects = layout.layout(Rect::new(0, 0, 100, 42));
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].height, 20);
        assert_eq!(rects[1].height, 20);
        // With default margin(0), widths should be 100 (cross_axis - 2*margin)
        assert_eq!(rects[0].width, 100);
        assert_eq!(rects[1].width, 100);
    }

    // Property-based tests
    use proptest::prelude::*;

    fn constraint_strategy() -> impl Strategy<Value = Constraint> {
        prop_oneof![
            any::<u16>().prop_map(Constraint::Percentage),
            any::<u16>().prop_map(Constraint::Fixed),
            any::<u16>().prop_map(Constraint::Min),
            any::<u16>().prop_map(Constraint::Max),
            (1u16..=100u16, 1u16..=100u16).prop_map(|(n, d)| Constraint::Ratio(n, d)),
        ]
    }

    fn direction_strategy() -> impl Strategy<Value = Direction> {
        prop_oneof![
            Just(Direction::Horizontal),
            Just(Direction::Vertical)
        ]
    }

    proptest! {
        fn constraint_never_exceeds_available(
            available in 0u16..=1000,
            fixed_consumed in 0u16..=1000,
            constraint in constraint_strategy(),
        ) {
            let result = constraint.resolve(available, fixed_consumed);
            prop_assert!(
                result <= available,
                "Constraint::{:?}.resolve({}, {}) = {} exceeds available {}",
                constraint, available, fixed_consumed, result, available
            );
        }

        #[test]
        fn layout_total_within_available_space(
            width in 1u16..=300,
            height in 1u16..=100,
            spacing in 0u16..=10,
            margin in 0u16..=20,
            constraints in proptest::collection::vec(constraint_strategy(), 1..=20),
            direction in direction_strategy(),
        ) {
            let layout = Layout {
                constraints,
                direction,
                spacing,
                margin,
                name: None,
            };

            let area = Rect::new(0, 0, width, height);
            let rects = layout.layout(area);

            let is_vertical = direction == Direction::Vertical;
            let main_axis = if is_vertical { height } else { width };
            let applied_margin = 2 * margin;
            let total_spacing = spacing * (rects.len() as u16).saturating_sub(1);

            let available = main_axis.saturating_sub(applied_margin).saturating_sub(total_spacing);
            let sum: u32 = rects.iter()
                .map(|r| {
                    // For vertical layouts, check heights; for horizontal, check widths
                    if is_vertical { r.height as u32 } else { r.width as u32 }
                })
                .sum();

            // For percentage constraints, sum should equal available (with tolerance for rounding)
            // For fixed constraints, sum will be <= available
            prop_assert!(
                sum <= available as u32 + 1, // Allow 1 cell tolerance for rounding
                "layout total {} exceeds available {} (margin={}, spacing={})",
                sum, available, margin, spacing
            );
        }
    }

    #[test]
    fn test_vertical_with_fixed_and_ratio() {
        let layout = Layout::vertical(vec![Constraint::Fixed(5), Constraint::Ratio(1, 1)]);
        let rects = layout.layout(Rect::new(0, 0, 80, 30));
        assert_eq!(rects[0].height, 5);
        assert_eq!(rects[1].height, 25);
        assert_eq!(rects[0].width, 80);
    }

    #[test]
    fn test_empty_layout() {
        let layout = Layout::horizontal(vec![]);
        let rects = layout.layout(Rect::new(0, 0, 100, 20));
        assert!(rects.is_empty());
    }

    #[test]
    fn test_vertical_layout_with_direction() {
        // Test backward compatibility with direction() method
        let layout = Layout::new(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(Direction::Vertical);
        let rects = layout.layout(Rect::new(0, 0, 100, 40));
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].height, 20);
        assert_eq!(rects[1].height, 20);
    }
}

//! Property-based tests for core invariants.
//!
//! These use `proptest` to test properties that should hold for all valid inputs,
//! catching edge cases that unit tests might miss.

use proptest::prelude::*;
use std::panic::Location;

// ---------------------------------------------------------------------------
// Layout constraint invariants
// ---------------------------------------------------------------------------

use dracon_terminal_engine::framework::layout::{Constraint, Layout, Direction};
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::text::grapheme_width;

/// Test: the sum of all layout segment sizes should not exceed available space.
///
/// For any valid width/height and constraint list, the sum of allocated rect
/// dimensions + spacing should be within 1 cell of the available axis (rounding
/// tolerance allowed).
#[test]
#[cfg(feature = "std")]
fn layout_total_within_available_space() {
    layout_total_within_available_space_prop();
}

proptest! {
    #[test]
    fn layout_total_within_available_space_prop(
        width in 1u16..=300,
        height in 1u16..=100,
        spacing in 0u16..=10,
        margin in 0u16..=20,
        constraints in proptest::collection::vec(
            any::<Constraint>(),
            1..=20
        ),
        direction in proptest::prop_oneof![Just(Direction::Horizontal), Just(Direction::Vertical)],
    ) {
        let layout = Layout {
            constraints,
            direction,
            spacing,
            margin,
            name: None,
        };

        let area = ratatui::layout::Rect::new(0, 0, width, height);
        let rects = layout.layout(area);

        let is_vertical = direction == Direction::Vertical;
        let main_axis = if is_vertical { height } else { width };
        let applied_margin = 2 * margin;
        let total_spacing = spacing * (rects.len() as u16).saturating_sub(1);

        let available = main_axis.saturating_sub(applied_margin).saturating_sub(total_spacing);
        let sum: u32 = rects.iter()
            .map(|r| {
                if is_vertical { r.height as u32 } else { r.width as u32 }
            })
            .sum();

        // Allow 1 cell of rounding tolerance
        prop_assert!(
            sum as i32 - available as i32 >= -1,
            "sum={} available={} tolerance=1", sum, available
        );
    }
}

// ---------------------------------------------------------------------------
// Grapheme width is always 0, 1, or 2
// ---------------------------------------------------------------------------

/// Test: `grapheme_width()` must return 0, 1, or 2 for any Unicode scalar value.
/// It must never be negative or exceed 2.
#[test]
fn grapheme_width_returns_valid_range() {
    grapheme_width_returns_valid_range_prop();
}

proptest! {
    #[test]
    fn grapheme_width_returns_valid_range_prop(c in "\\PC") {
        let w = grapheme_width(c);
        prop_assert!(w <= 2, "grapheme_width({:?}) = {} (expected 0, 1, or 2)", c, w);
    }
}

// ---------------------------------------------------------------------------
// Theme parsing invariants
// ---------------------------------------------------------------------------

/// Test: `Theme::from_name()` returns `Some` for all 21 known theme names.
#[test]
fn theme_from_name_returns_some_for_known_themes() {
    let known = [
        "dark",
        "light",
        "high_contrast",
        "cyberpunk",
        "dracula",
        "nord",
        "catppuccin_mocha",
        "gruvbox_dark",
        "tokyo_night",
        "solarized_dark",
        "solarized_light",
        "one_dark",
        "rose_pine",
        "kanagawa",
        "everforest",
        "monokai",
        "warm",
        "cool",
        "forest",
        "sunset",
        "mono",
    ];

    for name in known {
        assert!(
            Theme::from_name(name).is_some(),
            "Theme::from_name({:?}) returned None, expected Some",
            name
        );
    }
}

/// Test: `Theme::from_name()` returns `None` for unknown theme names.
#[test]
fn theme_from_name_returns_none_for_unknown() {
    let unknown = [
        "nonexistent",
        "default",
        "gruvbox-wrong",
        "catppuccin_moch",
        "tokyo-night",
        "",
        "x" * 100,
    ];

    for name in unknown {
        assert!(
            Theme::from_name(name).is_none(),
            "Theme::from_name({:?}) returned Some, expected None",
            name
        );
    }
}

/// Test: `Theme::from_name()` is case-insensitive.
#[test]
fn theme_from_name_is_case_insensitive() {
    let names = ["nord", "Nord", "NORD", "NoRd"];
    let results: Vec<_> = names.iter().map(|n| Theme::from_name(n)).collect();
    for (i, result) in results.iter().enumerate() {
        assert_eq!(
            *result, results[0],
            "Theme::from_name({:?}) = {:?}, expected {:?} (from {:?})",
            names[i], result, results[0], names[0]
        );
    }
}

/// Test: `Theme::from_name()` normalises hyphens to underscores.
#[test]
fn theme_from_name_normalises_hyphens() {
    assert_eq!(
        Theme::from_name("catppuccin-mocha"),
        Theme::from_name("catppuccin_mocha"),
        "hyphen vs underscore should resolve the same"
    );
    assert_eq!(
        Theme::from_name("tokyo-night"),
        Theme::from_name("tokyo_night"),
        "hyphen vs underscore should resolve the same"
    );
}

// ---------------------------------------------------------------------------
// Plane bounds invariants
// ---------------------------------------------------------------------------

/// Test: writing to any cell (x, y) within Plane bounds should not panic.
#[test]
fn plane_bounds_writes_do_not_panic() {
    plane_bounds_writes_do_not_panic_prop();
}

proptest! {
    #[test]
    fn plane_bounds_writes_do_not_panic_prop(
        width in 1u16..=100,
        height in 1u16..=50,
        x in 0u16..200,
        y in 0u16..100,
        ch in "\\PC",
    ) {
        let mut plane = Plane::new(0, width, height);

        // These should be safe — put_char and put_cell skip out-of-bounds
        plane.put_char(x, y, ch);
        plane.put_cell(x, y, plane.cells.first().cloned().unwrap_or_default());
    }
}

/// Test: Plane cell operations on edges do not panic.
#[test]
fn plane_edge_operations_are_safe() {
    for w in [1u16, 2, 10, 50] {
        for h in [1u16, 2, 10, 50] {
            let mut plane = Plane::new(0, w, h);

            // Write at all four corners
            plane.put_char(0, 0, 'a');
            plane.put_char(w - 1, 0, 'b');
            plane.put_char(0, h - 1, 'c');
            plane.put_char(w - 1, h - 1, 'd');

            // Write one past the edge (should be no-op)
            plane.put_char(w, h, 'x');

            // fill_bg should not panic
            plane.fill_bg(dracon_terminal_engine::compositor::Color::Rgb(10, 20, 30));

            // clear should not panic
            plane.clear();
        }
    }
}

/// Test: Plane cells vector size matches width * height.
#[test]
fn plane_cells_vector_size_matches_dimensions() {
    for w in 1u16..=50 {
        for h in 1u16..=50 {
            let plane = Plane::new(0, w, h);
            assert_eq!(
                plane.cells.len(),
                w as usize * h as usize,
                "Plane({}, {}) has {} cells, expected {}",
                w,
                h,
                plane.cells.len(),
                w as usize * h as usize
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Constraint resolve invariants
// ---------------------------------------------------------------------------

/// Test: `Constraint::resolve()` never returns more than available space.
#[test]
fn constraint_resolve_respects_available() {
    constraint_resolve_respects_available_prop();
}

proptest! {
    #[test]
    fn constraint_resolve_respects_available_prop(
        available in 0u16..=1000,
        fixed_consumed in 0u16..=1000,
        pct in 0u16..=100,
        fixed in 0u16..=100,
        min_val in 0u16..=100,
        max_val in 0u16..=100,
        ratio_num in 0u16..=10,
        ratio_den in 1u16..=10,
    ) {
        for constraint in [
            Constraint::Percentage(pct),
            Constraint::Fixed(fixed),
            Constraint::Min(min_val),
            Constraint::Max(max_val),
            Constraint::Ratio(ratio_num, ratio_den),
        ] {
            let result = constraint.resolve(available, fixed_consumed);
            prop_assert!(
                result <= available,
                "Constraint::{:?}.resolve({}, {}) = {} exceeds available {}",
                constraint, available, fixed_consumed, result, available
            );
        }
    }
}

/// Test: `Constraint::resolve()` returns non-negative values.
#[test]
fn constraint_resolve_never_negative() {
    constraint_resolve_never_negative_prop();
}

proptest! {
    #[test]
    fn constraint_resolve_never_negative_prop(
        available in 0u16..=1000,
        fixed_consumed in 0u16..=1000,
        pct in 0u16..=100,
        fixed in 0u16..=100,
        min_val in 0u16..=100,
        max_val in 0u16..=100,
        ratio_num in 0u16..=10,
        ratio_den in 1u16..=10,
    ) {
        for constraint in [
            Constraint::Percentage(pct),
            Constraint::Fixed(fixed),
            Constraint::Min(min_val),
            Constraint::Max(max_val),
            Constraint::Ratio(ratio_num, ratio_den),
        ] {
            let result = constraint.resolve(available, fixed_consumed);
            prop_assert!(
                result >= 0,
                "Constraint::{:?}.resolve({}, {}) = {} is negative",
                constraint, available, fixed_consumed, result
            );
        }
    }
}

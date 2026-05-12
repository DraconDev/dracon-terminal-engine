//! Property-based tests for core invariants.
//!
//! These use `proptest` to test properties that should hold for all valid inputs,
//! catching edge cases that unit tests might miss.

use proptest::prelude::*;

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
    fn grapheme_width_returns_valid_range_prop(c in any::<char>()) {
        let w = dracon_terminal_engine::text::grapheme_width(c);
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
            dracon_terminal_engine::framework::theme::Theme::from_name(name).is_some(),
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
        &"x".repeat(100),
    ];

    for name in unknown {
        assert!(
            dracon_terminal_engine::framework::theme::Theme::from_name(name).is_none(),
            "Theme::from_name({:?}) returned Some, expected None",
            name
        );
    }
}

/// Test: `Theme::from_name()` is case-insensitive.
#[test]
fn theme_from_name_is_case_insensitive() {
    let names = ["nord", "Nord", "NORD", "NoRd"];
    let results: Vec<_> = names
        .iter()
        .map(|n| dracon_terminal_engine::framework::theme::Theme::from_name(n))
        .collect();
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
        dracon_terminal_engine::framework::theme::Theme::from_name("catppuccin-mocha"),
        dracon_terminal_engine::framework::theme::Theme::from_name("catppuccin_mocha"),
        "hyphen vs underscore should resolve the same"
    );
    assert_eq!(
        dracon_terminal_engine::framework::theme::Theme::from_name("tokyo-night"),
        dracon_terminal_engine::framework::theme::Theme::from_name("tokyo_night"),
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
        ch in any::<char>(),
    ) {
        let mut plane = dracon_terminal_engine::compositor::Plane::new(0, width, height);

        // These should be safe — put_char and put_cell skip out-of-bounds
        plane.put_char(x, y, ch);
        plane.put_cell(
            x,
            y,
            plane.cells.first().cloned().unwrap_or_default(),
        );
    }
}

/// Test: Plane cell operations on edges do not panic.
#[test]
fn plane_edge_operations_are_safe() {
    for w in [1u16, 2, 10, 50] {
        for h in [1u16, 2, 10, 50] {
            let mut plane = dracon_terminal_engine::compositor::Plane::new(0, w, h);

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
            let plane = dracon_terminal_engine::compositor::Plane::new(0, w, h);
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

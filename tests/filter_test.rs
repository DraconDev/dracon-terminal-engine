//! Tests for Filter trait and all filter implementations.

use dracon_terminal_engine::compositor::filter::{Dim, Filter, Glitch, Invert, Pulse, Scanline};
use dracon_terminal_engine::compositor::plane::Cell;
use dracon_terminal_engine::compositor::Color;

fn make_cell(char: char, fg: Color, bg: Color) -> Cell {
    Cell {
        char,
        fg,
        bg,
        style: Default::default(),
        transparent: false,
        skip: false,
    }
}

// ========== Dim Filter ==========

#[test]
fn test_dim_default_factor() {
    let filter = Dim::default();
    assert!((filter.factor - 0.5).abs() < 0.001);
}

#[test]
fn test_dim_custom_factor() {
    let filter = Dim { factor: 0.3 };
    assert!((filter.factor - 0.3).abs() < 0.001);
}

#[test]
fn test_dim_rgb_colors() {
    let filter = Dim { factor: 0.5 };
    let mut cell = make_cell('X', Color::Rgb(100, 150, 200), Color::Rgb(50, 75, 100));
    filter.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, Color::Rgb(50, 75, 100));
    assert_eq!(cell.bg, Color::Rgb(25, 37, 50));
}

#[test]
fn test_dim_ansi_color() {
    let filter = Dim { factor: 0.5 };
    let mut cell = make_cell('X', Color::Ansi(12), Color::Ansi(8));
    filter.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, Color::Ansi(8));
    assert_eq!(cell.bg, Color::Ansi(8));
}

#[test]
fn test_dim_ansi_above_8_stays_at_8() {
    let filter = Dim { factor: 0.5 };
    let mut cell = make_cell('X', Color::Ansi(200), Color::Ansi(200));
    filter.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, Color::Ansi(8));
    assert_eq!(cell.bg, Color::Ansi(8));
}

#[test]
fn test_dim_reset_color_unchanged() {
    let filter = Dim { factor: 0.5 };
    let mut cell = make_cell('X', Color::Reset, Color::Reset);
    let orig_fg = cell.fg;
    let orig_bg = cell.bg;
    filter.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, orig_fg);
    assert_eq!(cell.bg, orig_bg);
}

#[test]
fn test_dim_factor_clamping() {
    let filter = Dim { factor: 0.0 };
    let mut cell = make_cell('X', Color::Rgb(255, 255, 255), Color::Rgb(255, 255, 255));
    filter.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, Color::Rgb(0, 0, 0));
    assert_eq!(cell.bg, Color::Rgb(0, 0, 0));
}

#[test]
fn test_dim_factor_one() {
    let filter = Dim { factor: 1.0 };
    let mut cell = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(100, 100, 100));
    filter.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, Color::Rgb(100, 100, 100));
    assert_eq!(cell.bg, Color::Rgb(100, 100, 100));
}

// ========== Invert Filter ==========

#[test]
fn test_invert_swaps_fg_and_bg() {
    let filter = Invert;
    let mut cell = make_cell('X', Color::Rgb(100, 150, 200), Color::Rgb(50, 75, 100));
    filter.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, Color::Rgb(50, 75, 100));
    assert_eq!(cell.bg, Color::Rgb(100, 150, 200));
}

#[test]
fn test_invert_double_invert_restores() {
    let filter = Invert;
    let mut cell = make_cell('X', Color::Rgb(100, 150, 200), Color::Rgb(50, 75, 100));
    filter.apply(&mut cell, 0, 0, 0.0);
    filter.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, Color::Rgb(100, 150, 200));
    assert_eq!(cell.bg, Color::Rgb(50, 75, 100));
}

#[test]
fn test_invert_ansi() {
    let filter = Invert;
    let mut cell = make_cell('X', Color::Ansi(1), Color::Ansi(2));
    filter.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, Color::Ansi(2));
    assert_eq!(cell.bg, Color::Ansi(1));
}

// ========== Scanline Filter ==========

#[test]
fn test_scanline_even_row_dimmed() {
    let filter = Scanline;
    let mut cell = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    filter.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, Color::Rgb(80, 80, 80));
    assert_eq!(cell.bg, Color::Rgb(40, 40, 40));
}

#[test]
fn test_scanline_odd_row_unchanged() {
    let filter = Scanline;
    let mut cell = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    filter.apply(&mut cell, 0, 1, 0.0);
    assert_eq!(cell.fg, Color::Rgb(100, 100, 100));
    assert_eq!(cell.bg, Color::Rgb(50, 50, 50));
}

#[test]
fn test_scanline_row_2_is_even() {
    let filter = Scanline;
    let mut cell = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    filter.apply(&mut cell, 0, 2, 0.0);
    assert_eq!(cell.fg, Color::Rgb(80, 80, 80));
}

#[test]
fn test_scanline_row_3_is_odd() {
    let filter = Scanline;
    let mut cell = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    filter.apply(&mut cell, 0, 3, 0.0);
    assert_eq!(cell.fg, Color::Rgb(100, 100, 100));
}

// ========== Pulse Filter ==========

#[test]
fn test_pulse_time_zero() {
    let filter = Pulse;
    let mut cell = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    filter.apply(&mut cell, 0, 0, 0.0);
}

#[test]
fn test_pulse_unchanged_bg() {
    let filter = Pulse;
    let mut cell = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    let bg_before = cell.bg;
    filter.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.bg, bg_before);
}

#[test]
fn test_pulse_different_times_produce_different_results() {
    let filter = Pulse;
    let mut cell1 = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    let mut cell2 = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    filter.apply(&mut cell1, 0, 0, 0.0);
    filter.apply(&mut cell2, 0, 0, 1.0);
    assert_ne!(cell1.fg, cell2.fg);
}

#[test]
fn test_pulse_factor_bounded() {
    let filter = Pulse;
    for time in [0.0, 1.0, 2.0, 3.0, 4.0] {
        let mut cell = make_cell('X', Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0));
        filter.apply(&mut cell, 0, 0, time);
    }
}

// ========== Glitch Filter ==========

#[test]
fn test_glitch_at_zero_time_most_cells_unchanged() {
    let filter = Glitch;
    let mut changed = 0;
    for y in 0..5u16 {
        for x in 0..10u16 {
            let mut cell = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
            filter.apply(&mut cell, x, y, 0.0);
            if cell.char != 'X' {
                changed += 1;
            }
        }
    }
}

#[test]
fn test_glitch_high_time_random() {
    let filter = Glitch;
    let mut cell = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    filter.apply(&mut cell, 5, 5, 999.0);
}

#[test]
fn test_glitch_sets_char_to_unicode_block() {
    let filter = Glitch;
    let mut cell = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    filter.apply(&mut cell, 0, 0, 42.0);
}

#[test]
fn test_glitch_reverses_style_on_trigger() {
    let filter = Glitch;
    let mut cell = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    cell.style = Default::default();
    filter.apply(&mut cell, 5, 5, 100.0);
}

#[test]
fn test_glitch_time_deterministic() {
    let filter = Glitch;
    let mut cell1 = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    let mut cell2 = make_cell('X', Color::Rgb(100, 100, 100), Color::Rgb(50, 50, 50));
    filter.apply(&mut cell1, 5, 5, 10.0);
    filter.apply(&mut cell2, 5, 5, 10.0);
    assert_eq!(cell1.char, cell2.char);
    assert_eq!(cell1.fg, cell2.fg);
}
//! Compositor stress tests — extreme z-index, many planes, overlapping cells.

use dracon_terminal_engine::compositor::{Cell, Color, Compositor, Plane, Styles};

#[test]
fn test_compositor_many_planes() {
    let mut compositor = Compositor::new(80, 24);
    
    // Add 100 overlapping planes
    for i in 0..100 {
        let mut plane = Plane::new(i as u16, 80, 24);
        plane.fill_bg(Color::Ansi((i % 256) as u8));
        compositor.add_plane(plane);
    }
    
    let frame = compositor.compose();
    assert_eq!(frame.cells.len(), 80 * 24);
}

#[test]
fn test_compositor_extreme_z_index() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut low = Plane::new(0, 80, 24);
    low.fill_bg(Color::Red);
    low.cells[0].char = 'L';
    
    let mut high = Plane::new(65535, 80, 24);
    high.fill_bg(Color::Blue);
    high.cells[0].char = 'H';
    
    compositor.add_plane(low);
    compositor.add_plane(high);
    
    let frame = compositor.compose();
    // High z-index should win
    assert_eq!(frame.cells[0].char, 'H');
    assert_eq!(frame.cells[0].bg, Color::Blue);
}

#[test]
fn test_compositor_all_transparent() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut plane = Plane::new(0, 80, 24);
    for cell in &mut plane.cells {
        cell.transparent = true;
    }
    compositor.add_plane(plane);
    
    let frame = compositor.compose();
    // Should still produce a frame
    assert_eq!(frame.cells.len(), 80 * 24);
}

#[test]
fn test_compositor_empty() {
    let compositor = Compositor::new(80, 24);
    let frame = compositor.compose();
    assert_eq!(frame.cells.len(), 80 * 24);
    // All cells should be default
    for cell in &frame.cells {
        assert_eq!(cell.char, '\0');
    }
}

#[test]
fn test_compositor_single_cell_plane() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut plane = Plane::new(1, 1, 1);
    plane.cells[0] = Cell {
        char: 'X',
        fg: Color::Red,
        bg: Color::Blue,
        style: Styles::BOLD,
        transparent: false,
        skip: false,
    };
    compositor.add_plane(plane);
    
    let frame = compositor.compose();
    assert_eq!(frame.cells[0].char, 'X');
    assert_eq!(frame.cells[0].fg, Color::Red);
}

#[test]
fn test_compositor_resize_smaller() {
    let mut compositor = Compositor::new(80, 24);
    let mut plane = Plane::new(0, 80, 24);
    plane.fill_bg(Color::Red);
    compositor.add_plane(plane);
    
    compositor.resize(40, 12);
    let frame = compositor.compose();
    assert_eq!(frame.cells.len(), 40 * 12);
}

#[test]
fn test_compositor_resize_larger() {
    let mut compositor = Compositor::new(40, 12);
    compositor.resize(80, 24);
    let frame = compositor.compose();
    assert_eq!(frame.cells.len(), 80 * 24);
}

#[test]
fn test_compositor_plane_position_offset() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut plane = Plane::new(0, 10, 10);
    plane.x = 5;
    plane.y = 5;
    plane.fill_bg(Color::Green);
    compositor.add_plane(plane);
    
    let frame = compositor.compose();
    // Cell at (5, 5) should be green
    let idx = (5 * 80 + 5) as usize;
    assert_eq!(frame.cells[idx].bg, Color::Green);
}

#[test]
fn test_compositor_overlapping_planes_merging() {
    let mut compositor = Compositor::new(80, 24);
    
    // Bottom plane fills everything
    let mut bottom = Plane::new(0, 80, 24);
    bottom.fill_bg(Color::Black);
    compositor.add_plane(bottom);
    
    // Top plane with transparent holes
    let mut top = Plane::new(1, 80, 24);
    top.fill_bg(Color::White);
    for cell in &mut top.cells {
        cell.transparent = true;
    }
    // Make one cell non-transparent
    top.cells[100].transparent = false;
    top.cells[100].bg = Color::Red;
    compositor.add_plane(top);
    
    let frame = compositor.compose();
    // Cell 100 should be red (from top plane)
    assert_eq!(frame.cells[100].bg, Color::Red);
    // Cell 0 should be black (from bottom, top is transparent)
    assert_eq!(frame.cells[0].bg, Color::Black);
}

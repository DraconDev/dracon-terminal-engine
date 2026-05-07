//! Compositor stress tests — overlapping planes, extreme z-index, large areas.

use dracon_terminal_engine::compositor::{Cell, Color, Compositor, Plane, Styles};
use ratatui::layout::Rect;

#[test]
fn test_compositor_many_overlapping_planes() {
    let mut compositor = Compositor::new(80, 24);
    
    // Add 100 small overlapping planes
    for i in 0..100 {
        let x = (i % 20) as u16;
        let y = (i % 10) as u16;
        let mut plane = Plane::new(i as u16, 10, 5);
        plane.x = x;
        plane.y = y;
        
        for cell in &mut plane.cells {
            cell.bg = Color::Ansi((i % 256) as u8);
        }
        
        compositor.add_plane(plane);
    }
    
    let frame = compositor.render();
    assert_eq!(frame.width, 80);
    assert_eq!(frame.height, 24);
}

#[test]
fn test_compositor_extreme_z_index() {
    let mut compositor = Compositor::new(80, 24);
    
    // Add planes with extreme z-indices
    let mut low = Plane::new(0, 10, 5);
    low.z_index = 0;
    low.cells[0].char = 'L';
    low.cells[0].transparent = false;
    
    let mut high = Plane::new(0, 10, 5);
    high.z_index = 65535;
    high.cells[0].char = 'H';
    high.cells[0].transparent = false;
    
    compositor.add_plane(low);
    compositor.add_plane(high);
    
    let frame = compositor.render();
    // High z-index should win
    let idx = 0;
    assert_eq!(frame.cells[idx].char, 'H');
}

#[test]
fn test_compositor_large_area() {
    let mut compositor = Compositor::new(200, 100);
    let plane = Plane::new(0, 200, 100);
    compositor.add_plane(plane);
    
    let frame = compositor.render();
    assert_eq!(frame.width, 200);
    assert_eq!(frame.height, 100);
}

#[test]
fn test_compositor_transparent_stacking() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut bottom = Plane::new(0, 80, 24);
    bottom.cells[0].char = 'B';
    bottom.cells[0].transparent = false;
    bottom.cells[0].bg = Color::Ansi(1);
    
    let mut top = Plane::new(1, 80, 24);
    top.cells[0].transparent = true; // Let bottom show through
    top.cells[1].char = 'T';
    top.cells[1].transparent = false;
    
    compositor.add_plane(bottom);
    compositor.add_plane(top);
    
    let frame = compositor.render();
    assert_eq!(frame.cells[0].char, 'B'); // Bottom shows through
    assert_eq!(frame.cells[1].char, 'T'); // Top covers
}

#[test]
fn test_compositor_out_of_bounds_planes() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut plane = Plane::new(0, 10, 5);
    plane.x = 100; // Outside compositor bounds
    plane.y = 50;
    
    compositor.add_plane(plane);
    
    let frame = compositor.render();
    assert_eq!(frame.width, 80);
    assert_eq!(frame.height, 24);
}

#[test]
fn test_compositor_negative_position() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut plane = Plane::new(0, 10, 5);
    // u16 can't be negative, but saturating_sub should handle edge cases
    plane.x = 0;
    plane.y = 0;
    
    compositor.add_plane(plane);
    
    let frame = compositor.render();
    assert_eq!(frame.width, 80);
}

#[test]
fn test_compositor_empty_planes() {
    let mut compositor = Compositor::new(80, 24);
    
    // Add plane with empty cells
    let mut plane = Plane::new(0, 80, 24);
    for cell in &mut plane.cells {
        cell.char = '\0'; // Null character = transparent skip
        cell.transparent = true;
    }
    
    compositor.add_plane(plane);
    
    let frame = compositor.render();
    assert_eq!(frame.width, 80);
}

#[test]
fn test_compositor_filter_application() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut plane = Plane::new(0, 80, 24);
    plane.cells[0].char = 'X';
    plane.cells[0].fg = Color::Ansi(7);
    
    compositor.add_plane(plane);
    
    // Apply a filter
    compositor.apply_filter(|cell| {
        cell.fg = Color::Ansi(15);
        cell
    });
    
    let frame = compositor.render();
    assert_eq!(frame.cells[0].fg, Color::Ansi(15));
}

#[test]
fn test_compositor_z_index_sorting() {
    let mut compositor = Compositor::new(80, 24);
    
    // Add planes in reverse z-order
    for i in (0..10).rev() {
        let mut plane = Plane::new(i, 80, 24);
        plane.z_index = i as u16;
        plane.cells[0].char = ('0' as u8 + i as u8) as char;
        plane.cells[0].transparent = false;
        compositor.add_plane(plane);
    }
    
    let frame = compositor.render();
    // Highest z-index (9) should win at cell 0
    assert_eq!(frame.cells[0].char, '9');
}

#[test]
fn test_compositor_remove_all_planes() {
    let mut compositor = Compositor::new(80, 24);
    
    compositor.add_plane(Plane::new(0, 10, 5));
    compositor.add_plane(Plane::new(1, 10, 5));
    
    compositor.clear_planes();
    
    let frame = compositor.render();
    assert_eq!(frame.width, 80);
    assert_eq!(frame.height, 24);
}

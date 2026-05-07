//! Compositor stress tests — overlapping planes, extreme z-index, large areas.

use dracon_terminal_engine::compositor::{Cell, Color, Compositor, Plane, Styles};

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
            cell.transparent = false;
        }
        
        compositor.add_plane(plane);
    }
    
    assert_eq!(compositor.planes.len(), 100);
    let hit = compositor.hit_test(5, 5);
    assert!(hit.is_some());
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
    
    // Hit test should return the plane at the given coordinates
    let hit = compositor.hit_test(0, 0);
    assert!(hit.is_some());
}

#[test]
fn test_compositor_large_area() {
    let mut compositor = Compositor::new(200, 100);
    let plane = Plane::new(0, 200, 100);
    compositor.add_plane(plane);
    
    assert_eq!(compositor.size(), (200, 100));
    assert_eq!(compositor.planes.len(), 1);
}

#[test]
fn test_compositor_all_transparent_planes() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut plane = Plane::new(0, 80, 24);
    for cell in &mut plane.cells {
        cell.transparent = true;
    }
    compositor.add_plane(plane);
    
    // Hit test should return None since all cells are transparent
    let hit = compositor.hit_test(5, 5);
    assert!(hit.is_none());
}

#[test]
fn test_compositor_empty() {
    let compositor = Compositor::new(80, 24);
    assert!(compositor.planes.is_empty());
    assert_eq!(compositor.size(), (80, 24));
}

#[test]
fn test_compositor_single_cell_plane() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut plane = Plane::new(0, 1, 1);
    plane.cells[0] = Cell {
        char: 'X',
        fg: Color::Red,
        bg: Color::Blue,
        style: Styles::BOLD,
        transparent: false,
        skip: false,
    };
    compositor.add_plane(plane);
    
    let hit = compositor.hit_test(0, 0);
    assert!(hit.is_some());
}

#[test]
fn test_compositor_resize() {
    let mut compositor = Compositor::new(80, 24);
    let mut plane = Plane::new(0, 80, 24);
    plane.fill_bg(Color::Red);
    compositor.add_plane(plane);
    
    compositor.resize(40, 12);
    assert_eq!(compositor.size(), (40, 12));
}

#[test]
fn test_compositor_plane_position_offset() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut plane = Plane::new(0, 10, 10);
    plane.x = 5;
    plane.y = 5;
    plane.fill_bg(Color::Green);
    compositor.add_plane(plane);
    
    // Hit inside the positioned plane
    let hit = compositor.hit_test(7, 7);
    assert!(hit.is_some());
    
    // Hit outside
    let miss = compositor.hit_test(2, 2);
    assert!(miss.is_none());
}

#[test]
fn test_compositor_z_index_ordering() {
    let mut compositor = Compositor::new(80, 24);
    
    // Add planes in reverse z-order
    for i in (0..10).rev() {
        let mut plane = Plane::new(i, 10, 10);
        plane.z_index = i as u16;
        plane.cells[0].char = ('0' as u8 + i as u8) as char;
        plane.cells[0].transparent = false;
        compositor.add_plane(plane);
    }
    
    // All planes added
    assert_eq!(compositor.planes.len(), 10);
    
    // Hit test should find a plane
    let hit = compositor.hit_test(0, 0);
    assert!(hit.is_some());
}

#[test]
fn test_compositor_clear_planes() {
    let mut compositor = Compositor::new(80, 24);
    
    compositor.add_plane(Plane::new(0, 10, 5));
    compositor.add_plane(Plane::new(1, 10, 5));
    assert_eq!(compositor.planes.len(), 2);
    
    compositor.planes.clear();
    assert!(compositor.planes.is_empty());
}

#[test]
fn test_compositor_hit_test_out_of_bounds() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut plane = Plane::new(0, 10, 10);
    plane.x = 10;
    plane.y = 10;
    compositor.add_plane(plane);
    
    // Hit outside compositor bounds
    let hit = compositor.hit_test(100, 100);
    assert!(hit.is_none());
}

#[test]
fn test_compositor_mixed_transparent_opaque() {
    let mut compositor = Compositor::new(80, 24);
    
    let mut plane = Plane::new(0, 10, 10);
    plane.cells[0].transparent = true;
    plane.cells[1].transparent = false;
    plane.cells[1].char = 'X';
    compositor.add_plane(plane);
    
    // Cell 0 is transparent - should return None
    let hit_transparent = compositor.hit_test(0, 0);
    assert!(hit_transparent.is_none());
    
    // Cell 1 is opaque - should return Some
    let hit_opaque = compositor.hit_test(1, 0);
    assert!(hit_opaque.is_some());
}

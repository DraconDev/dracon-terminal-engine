use dracon_terminal_engine::compositor::{Cell, Color, Compositor, Plane, Styles};

#[test]
fn test_plane_new() {
    let plane = Plane::new(0, 80, 24);
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 24);
    assert_eq!(plane.z_index, 0);
    assert_eq!(plane.cells.len(), 80 * 24);
}

#[test]
fn test_plane_set_position() {
    let mut plane = Plane::new(0, 10, 5);
    plane.set_absolute_position(5, 3);
    assert_eq!(plane.x, 5);
    assert_eq!(plane.y, 3);
}

#[test]
fn test_plane_set_z_index() {
    let mut plane = Plane::new(0, 10, 5);
    plane.set_z_index(100);
    assert_eq!(plane.z_index, 100);
}

#[test]
fn test_plane_cells_initialized() {
    let plane = Plane::new(0, 3, 2);
    assert!(plane.cells.iter().all(|c| c.char == ' '));
    assert!(plane.cells.iter().all(|c| c.fg == Color::Reset));
    assert!(plane.cells.iter().all(|c| c.bg == Color::Reset));
}

#[test]
fn test_plane_put_char() {
    let mut plane = Plane::new(0, 10, 10);
    plane.put_char(2, 3, 'X');
    let idx = 3 * 10 + 2;
    assert_eq!(plane.cells[idx].char, 'X');
}

#[test]
fn test_plane_clear() {
    let mut plane = Plane::new(0, 10, 10);
    plane.put_char(5, 5, 'A');
    plane.clear();
    assert!(plane.cells.iter().all(|c| c.char == ' '));
}

#[test]
fn test_cell_default() {
    let cell = Cell::default();
    assert_eq!(cell.char, ' ');
    assert_eq!(cell.fg, Color::Reset);
    assert_eq!(cell.bg, Color::Reset);
    assert_eq!(cell.style, Styles::empty());
    assert!(cell.transparent);
    assert!(!cell.skip);
}

#[test]
fn test_color_rgb() {
    let c = Color::Rgb(255, 128, 0);
    match c {
        Color::Rgb(r, g, b) => {
            assert_eq!(r, 255);
            assert_eq!(g, 128);
            assert_eq!(b, 0);
        }
        _ => panic!("expected Rgb"),
    }
}

#[test]
fn test_color_ansi() {
    let c = Color::Ansi(196);
    match c {
        Color::Ansi(n) => assert_eq!(n, 196),
        _ => panic!("expected Ansi"),
    }
}

#[test]
fn test_color_reset() {
    let c = Color::Reset;
    assert_eq!(c, Color::Reset);
}

#[test]
fn test_styles_bold() {
    let style = Styles::BOLD;
    assert!(style.contains(Styles::BOLD));
    assert!(!style.contains(Styles::ITALIC));
}

#[test]
fn test_styles_combined() {
    let style = Styles::BOLD | Styles::UNDERLINE;
    assert!(style.contains(Styles::BOLD));
    assert!(style.contains(Styles::UNDERLINE));
}

#[test]
fn test_compositor_new() {
    let comp = Compositor::new(80, 24);
    assert_eq!(comp.size(), (80, 24));
    assert!(comp.planes.is_empty());
}

#[test]
fn test_compositor_add_plane() {
    let mut comp = Compositor::new(80, 24);
    let plane = Plane::new(1, 80, 24);
    comp.add_plane(plane);
    assert_eq!(comp.planes.len(), 1);
}

#[test]
fn test_compositor_empty_planes_not_rendered() {
    let comp = Compositor::new(80, 24);
    assert!(comp.planes.is_empty());
}

#[test]
fn test_compositor_plane_ordering() {
    let mut comp = Compositor::new(80, 24);
    let low = Plane::new(1, 10, 10);
    let high = Plane::new(2, 10, 10);
    comp.add_plane(high);
    comp.add_plane(low);
    let ids: Vec<_> = comp.planes.iter().map(|p| p.id).collect();
    assert_eq!(ids, [2, 1]);
}

#[test]
fn test_compositor_set_clear_color() {
    let mut comp = Compositor::new(80, 24);
    comp.set_clear_color(Color::Rgb(30, 30, 30));
    assert_eq!(comp.size(), (80, 24));
}

#[test]
fn test_compositor_resize() {
    let mut comp = Compositor::new(80, 24);
    comp.resize(120, 40);
    assert_eq!(comp.size(), (120, 40));
}

#[test]
fn test_compositor_hit_test_empty() {
    let comp = Compositor::new(80, 24);
    assert!(comp.hit_test(10, 10).is_none());
}

#[test]
fn test_compositor_hit_test_with_plane() {
    let mut comp = Compositor::new(80, 24);
    let mut plane = Plane::new(1, 10, 10);
    plane.set_absolute_position(5, 5);
    plane.cells[0].transparent = false;
    comp.add_plane(plane);
    assert!(comp.hit_test(5, 5).is_some());
}

#[test]
fn test_compositor_hit_test_outside() {
    let mut comp = Compositor::new(80, 24);
    let mut plane = Plane::new(1, 10, 10);
    plane.set_absolute_position(5, 5);
    plane.cells[0].transparent = false;
    comp.add_plane(plane);
    assert!(comp.hit_test(20, 20).is_none());
}

#[test]
fn test_compositor_hit_test_transparent_cell() {
    let mut comp = Compositor::new(80, 24);
    let plane = Plane::new(1, 10, 10);
    comp.add_plane(plane);
    assert!(comp.hit_test(0, 0).is_none());
}

#[test]
fn test_compositor_draw_text() {
    let mut comp = Compositor::new(80, 24);
    comp.draw_text("Hello", 10, 5, Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), Styles::BOLD);
    assert!(!comp.planes.is_empty());
}

#[test]
fn test_compositor_draw_rect() {
    let mut comp = Compositor::new(80, 24);
    comp.draw_rect(5, 5, 10, 5, '#', Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0), Styles::empty());
    assert!(!comp.planes.is_empty());
}

#[test]
fn test_compositor_tick() {
    let mut comp = Compositor::new(80, 24);
    comp.tick(0.016);
}

#[test]
fn test_plane_visible() {
    let mut plane = Plane::new(0, 10, 10);
    assert!(plane.visible);
    plane.visible = false;
    assert!(!plane.visible);
}

#[test]
fn test_plane_opacity() {
    let mut plane = Plane::new(0, 10, 10);
    assert_eq!(plane.opacity, 1.0);
    plane.opacity = 0.5;
    assert_eq!(plane.opacity, 0.5);
}

#[test]
fn test_plane_filter() {
    use dracon_terminal_engine::compositor::filter::Dim;
    let mut plane = Plane::new(0, 10, 10);
    assert!(plane.filter.is_none());
    plane.filter = Some(Box::new(Dim::default()));
    assert!(plane.filter.is_some());
}

#[test]
fn test_compositor_plane_z_ordering() {
    let mut comp = Compositor::new(80, 24);
    let plane1 = Plane::new(1, 10, 10);
    let plane2 = Plane::new(2, 10, 10);
    let plane3 = Plane::new(0, 10, 10);
    comp.add_plane(plane1);
    comp.add_plane(plane2);
    comp.add_plane(plane3);
    let z_indices: Vec<_> = comp.planes.iter().map(|p| p.z_index).collect();
    assert!(z_indices.windows(2).all(|w| w[0] <= w[1]));
}

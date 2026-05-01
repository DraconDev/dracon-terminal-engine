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
fn test_compositor_plane_ordering() {
    let mut comp = Compositor::new(80, 24);
    let low = Plane::new(1, 10, 10);
    let high = Plane::new(2, 10, 10);
    comp.add_plane(high);
    comp.add_plane(low);
    let ids: Vec<_> = comp.planes.iter().map(|p| p.id).collect();
    assert_eq!(ids, [2, 1]);
}

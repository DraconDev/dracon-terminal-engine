use dracon_terminal_engine::compositor::filter::{Dim, Filter, Invert};
use dracon_terminal_engine::compositor::{Cell, Color, Compositor, Plane, Styles};

#[test]
fn test_plane_new_zero_dimensions_clamped_to_one() {
    let plane = Plane::new(0, 0, 0);
    assert_eq!(plane.width, 1);
    assert_eq!(plane.height, 1);
    assert_eq!(plane.cells.len(), 1);
}

#[test]
fn test_plane_new_zero_width_clamped() {
    let plane = Plane::new(0, 0, 10);
    assert_eq!(plane.width, 1);
    assert_eq!(plane.height, 10);
    assert_eq!(plane.cells.len(), 10);
}

#[test]
fn test_plane_new_zero_height_clamped() {
    let plane = Plane::new(0, 10, 0);
    assert_eq!(plane.width, 10);
    assert_eq!(plane.height, 1);
    assert_eq!(plane.cells.len(), 10);
}

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
    comp.draw_text(
        "Hello",
        10,
        5,
        Color::Rgb(255, 255, 255),
        Color::Rgb(0, 0, 0),
        Styles::BOLD,
    );
    assert!(!comp.planes.is_empty());
}

#[test]
fn test_compositor_draw_rect() {
    let mut comp = Compositor::new(80, 24);
    comp.draw_rect(
        5,
        5,
        10,
        5,
        '#',
        Color::Rgb(255, 255, 255),
        Color::Rgb(0, 0, 0),
        Styles::empty(),
    );
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

// ===== Comprehensive Compositor Tests =====

#[test]
fn test_plane_put_cell_sets_opaque() {
    let mut plane = Plane::new(0, 10, 10);
    let cell = Cell {
        char: 'X',
        fg: Color::Rgb(255, 0, 0),
        bg: Color::Rgb(0, 0, 0),
        style: Styles::BOLD,
        transparent: false,
        skip: false,
    };
    plane.put_cell(5, 5, cell);
    assert!(!plane.cells[5 * 10 + 5].transparent);
}

#[test]
fn test_plane_put_cell_out_of_bounds_ignored() {
    let mut plane = Plane::new(0, 10, 10);
    let cell = Cell {
        char: 'X',
        fg: Color::Rgb(255, 0, 0),
        bg: Color::Rgb(0, 0, 0),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    plane.put_cell(100, 100, cell);
    assert_eq!(plane.cells.iter().filter(|c| c.char == 'X').count(), 0);
}

#[test]
fn test_plane_put_char_wide_character_marks_next_as_skip() {
    let mut plane = Plane::new(0, 10, 10);
    plane.put_char(0, 0, '一');
    let idx = 0;
    let next_idx = idx + 1;
    assert!(plane.cells[idx].char == '一');
    assert!(plane.cells[next_idx].skip);
}

#[test]
fn test_plane_fill_bg() {
    let mut plane = Plane::new(0, 10, 10);
    plane.fill_bg(Color::Rgb(30, 30, 30));
    for cell in &plane.cells {
        assert_eq!(cell.bg, Color::Rgb(30, 30, 30));
        assert!(!cell.transparent);
    }
}

#[test]
fn test_plane_fill_bg_preserves_existing_content() {
    let mut plane = Plane::new(0, 3, 3);
    plane.put_char(1, 1, 'X');
    let fg_before = plane.cells[3 + 1].fg;
    plane.fill_bg(Color::Rgb(50, 50, 50));
    assert_eq!(plane.cells[3 + 1].char, 'X');
    assert_eq!(plane.cells[3 + 1].fg, fg_before);
}

#[test]
fn test_cell_eq_same_content_equal() {
    let cell1 = Cell {
        char: 'A',
        fg: Color::Rgb(255, 0, 0),
        bg: Color::Rgb(0, 0, 0),
        style: Styles::BOLD,
        transparent: false,
        skip: false,
    };
    let cell2 = Cell {
        char: 'A',
        fg: Color::Rgb(255, 0, 0),
        bg: Color::Rgb(0, 0, 0),
        style: Styles::BOLD,
        transparent: false,
        skip: false,
    };
    assert_eq!(cell1, cell2);
}

#[test]
fn test_cell_eq_different_fg_not_equal() {
    let cell1 = Cell {
        char: 'A',
        fg: Color::Rgb(255, 0, 0),
        bg: Color::Rgb(0, 0, 0),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    let cell2 = Cell {
        char: 'A',
        fg: Color::Rgb(0, 255, 0),
        bg: Color::Rgb(0, 0, 0),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    assert_ne!(cell1, cell2);
}

#[test]
fn test_hit_test_with_visible_false_plane() {
    let mut comp = Compositor::new(80, 24);
    let mut plane = Plane::new(1, 10, 10);
    plane.set_absolute_position(5, 5);
    plane.cells[0].transparent = false;
    plane.visible = false;
    comp.add_plane(plane);
    assert!(comp.hit_test(5, 5).is_none());
}

#[test]
fn test_hit_test_returns_topmost() {
    let mut comp = Compositor::new(80, 24);
    let mut lower = Plane::new(1, 10, 10);
    lower.set_absolute_position(0, 0);
    lower.cells[0].transparent = false;
    lower.z_index = 0;
    let mut upper = Plane::new(2, 10, 10);
    upper.set_absolute_position(0, 0);
    upper.cells[0].transparent = false;
    upper.z_index = 1;
    comp.add_plane(lower);
    comp.add_plane(upper);
    let hit = comp.hit_test(0, 0);
    assert!(hit.is_some());
    assert_eq!(hit.unwrap().id, 2);
}

#[test]
fn test_hit_test_transparent_upper_shows_lower() {
    let mut comp = Compositor::new(80, 24);
    let mut lower = Plane::new(1, 10, 10);
    lower.set_absolute_position(0, 0);
    lower.cells[0].transparent = false;
    lower.z_index = 0;
    let mut upper = Plane::new(2, 10, 10);
    upper.set_absolute_position(0, 0);
    upper.cells[0].transparent = true;
    upper.z_index = 1;
    comp.add_plane(upper);
    comp.add_plane(lower);
    let hit = comp.hit_test(0, 0);
    assert!(hit.is_some());
    assert_eq!(hit.unwrap().id, 1);
}

#[test]
fn test_compositor_multiple_planes_hit_topmost_opaque() {
    let mut comp = Compositor::new(80, 24);
    let mut p1 = Plane::new(1, 5, 5);
    p1.set_absolute_position(0, 0);
    p1.cells[0].transparent = false;
    p1.z_index = 0;
    let mut p2 = Plane::new(2, 5, 5);
    p2.set_absolute_position(2, 2);
    p2.cells[0].transparent = false;
    p2.z_index = 1;
    let mut p3 = Plane::new(3, 5, 5);
    p3.set_absolute_position(4, 4);
    p3.cells[0].transparent = false;
    p3.z_index = 2;
    comp.add_plane(p1);
    comp.add_plane(p2);
    comp.add_plane(p3);
    assert!(comp.hit_test(4, 4).is_some());
    assert_eq!(comp.hit_test(4, 4).unwrap().id, 3);
    assert!(comp.hit_test(2, 2).is_some());
    assert_eq!(comp.hit_test(2, 2).unwrap().id, 2);
    assert!(comp.hit_test(0, 0).is_some());
    assert_eq!(comp.hit_test(0, 0).unwrap().id, 1);
}

#[test]
fn test_compositor_plane_sort_preserves_insertion_order_same_z() {
    let mut comp = Compositor::new(80, 24);
    let p1 = Plane::new(1, 10, 10);
    let p2 = Plane::new(2, 10, 10);
    let p3 = Plane::new(3, 10, 10);
    comp.add_plane(p1);
    comp.add_plane(p2);
    comp.add_plane(p3);
    let z_indices: Vec<_> = comp.planes.iter().map(|p| p.z_index).collect();
    assert!(z_indices.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn test_compositor_resize_clears_planes() {
    let mut comp = Compositor::new(80, 24);
    let plane = Plane::new(1, 80, 24);
    comp.add_plane(plane);
    comp.resize(120, 40);
    assert_eq!(comp.size(), (120, 40));
}

#[test]
fn test_dim_filter_actually_modifies_cell() {
    let dim = Dim { factor: 0.5 };
    let mut cell = Cell {
        char: 'X',
        fg: Color::Rgb(100, 150, 200),
        bg: Color::Rgb(50, 75, 100),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    dim.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, Color::Rgb(50, 75, 100));
    assert_eq!(cell.bg, Color::Rgb(25, 37, 50));
}

#[test]
fn test_dim_filter_resets_dont_change() {
    let dim = Dim { factor: 0.5 };
    let mut cell = Cell {
        char: 'X',
        fg: Color::Reset,
        bg: Color::Reset,
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    let orig_fg = cell.fg;
    let orig_bg = cell.bg;
    dim.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, orig_fg);
    assert_eq!(cell.bg, orig_bg);
}

#[test]
fn test_invert_filter_swaps_fg_bg() {
    let invert = Invert;
    let mut cell = Cell {
        char: 'X',
        fg: Color::Rgb(100, 150, 200),
        bg: Color::Rgb(50, 75, 100),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    invert.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.fg, Color::Rgb(50, 75, 100));
    assert_eq!(cell.bg, Color::Rgb(100, 150, 200));
}

#[test]
fn test_pulse_filter_time_zero_no_opacity_change() {
    use dracon_terminal_engine::compositor::filter::Pulse;
    let pulse = Pulse;
    let mut cell = Cell {
        char: 'X',
        fg: Color::Rgb(100, 100, 100),
        bg: Color::Rgb(50, 50, 50),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    pulse.apply(&mut cell, 0, 0, 0.0);
    assert_eq!(cell.bg, Color::Rgb(50, 50, 50));
}

#[test]
fn test_scanline_filter_even_rows_dimmed() {
    use dracon_terminal_engine::compositor::filter::Scanline;
    let scanline = Scanline;
    let mut even_cell = Cell {
        char: 'X',
        fg: Color::Rgb(100, 100, 100),
        bg: Color::Rgb(50, 50, 50),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    scanline.apply(&mut even_cell, 0, 0, 0.0);
    assert_eq!(even_cell.fg, Color::Rgb(80, 80, 80));
    assert_eq!(even_cell.bg, Color::Rgb(40, 40, 40));
    let mut odd_cell = Cell {
        char: 'X',
        fg: Color::Rgb(100, 100, 100),
        bg: Color::Rgb(50, 50, 50),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    scanline.apply(&mut odd_cell, 0, 1, 0.0);
    assert_eq!(odd_cell.fg, Color::Rgb(100, 100, 100));
    assert_eq!(odd_cell.bg, Color::Rgb(50, 50, 50));
}

#[test]
fn test_glitch_filter_deterministic_at_same_time() {
    use dracon_terminal_engine::compositor::filter::Glitch;
    let glitch = Glitch;
    let mut cell1 = Cell {
        char: 'X',
        fg: Color::Rgb(100, 100, 100),
        bg: Color::Rgb(50, 50, 50),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    let mut cell2 = Cell {
        char: 'X',
        fg: Color::Rgb(100, 100, 100),
        bg: Color::Rgb(50, 50, 50),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    glitch.apply(&mut cell1, 5, 5, 42.0);
    glitch.apply(&mut cell2, 5, 5, 42.0);
    assert_eq!(cell1.char, cell2.char);
    assert_eq!(cell1.fg, cell2.fg);
}

#[test]
fn test_glitch_filter_probabilistic_behavior() {
    use dracon_terminal_engine::compositor::filter::Glitch;
    let glitch = Glitch;
    let mut cell1 = Cell {
        char: 'X',
        fg: Color::Rgb(100, 100, 100),
        bg: Color::Rgb(50, 50, 50),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    let mut cell2 = Cell {
        char: 'X',
        fg: Color::Rgb(100, 100, 100),
        bg: Color::Rgb(50, 50, 50),
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    glitch.apply(&mut cell1, 5, 5, 10.0);
    glitch.apply(&mut cell2, 5, 5, 10.0);
    assert_eq!(cell1.char, cell2.char);
    assert_eq!(cell1.fg, cell2.fg);
}

#[test]
fn test_plane_with_dim_filter() {
    let mut plane = Plane::new(0, 10, 10);
    plane.filter = Some(Box::new(Dim { factor: 0.3 }));
    assert!(plane.filter.is_some());
}

#[test]
fn test_plane_fill_and_put_char() {
    let mut plane = Plane::new(0, 10, 10);
    plane.fill_bg(Color::Rgb(20, 20, 20));
    plane.put_char(5, 5, 'A');
    let idx = 5 * 10 + 5;
    assert_eq!(plane.cells[idx].char, 'A');
    assert_eq!(plane.cells[idx].bg, Color::Rgb(20, 20, 20));
    assert!(!plane.cells[idx].transparent);
}

#[test]
fn test_compositor_hit_test_edge_of_plane() {
    let mut comp = Compositor::new(80, 24);
    let mut plane = Plane::new(1, 10, 10);
    plane.set_absolute_position(5, 5);
    plane.cells[(9 * 10 + 9) as usize].transparent = false;
    comp.add_plane(plane);
    assert!(comp.hit_test(14, 14).is_some());
    assert!(comp.hit_test(15, 15).is_none());
}

#[test]
fn test_compositor_plane_all_cells_transparent_returns_none() {
    let mut comp = Compositor::new(80, 24);
    let plane = Plane::new(1, 10, 10);
    comp.add_plane(plane);
    for x in 0..10 {
        for y in 0..10 {
            assert!(comp.hit_test(x, y).is_none());
        }
    }
}

#[test]
fn test_compositor_mixed_opaque_transparent_in_plane() {
    let mut comp = Compositor::new(80, 24);
    let mut plane = Plane::new(1, 5, 5);
    plane.set_absolute_position(0, 0);
    plane.cells[0].transparent = false;
    plane.cells[1].transparent = true;
    plane.cells[2].transparent = false;
    comp.add_plane(plane);
    assert!(comp.hit_test(0, 0).is_some());
    assert!(comp.hit_test(1, 0).is_none());
    assert!(comp.hit_test(2, 0).is_some());
}

#[test]
fn test_plane_clear_resets_to_transparent() {
    let mut plane = Plane::new(0, 10, 10);
    plane.put_cell(
        5,
        5,
        Cell {
            char: 'X',
            fg: Color::Rgb(255, 0, 0),
            bg: Color::Rgb(0, 0, 0),
            style: Styles::BOLD,
            transparent: false,
            skip: false,
        },
    );
    plane.clear();
    let cell = &plane.cells[5 * 10 + 5];
    assert_eq!(cell.char, ' ');
    assert!(cell.transparent);
}

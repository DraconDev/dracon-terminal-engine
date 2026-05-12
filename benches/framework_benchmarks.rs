//! Performance benchmarks for core framework operations.
//!
//! Run with: `cargo bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dracon_terminal_engine::compositor::pool::{
    acquire_plane_cells, release_plane_cells, CellPool,
};
use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::animation::{Animation, AnimationManager, Easing};
use dracon_terminal_engine::framework::focus::FocusManager;
use dracon_terminal_engine::framework::hitzone::{HitZoneGroup, ScopedZoneRegistry};
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Column, List, Table};
use ratatui::layout::Rect;
use std::time::Duration;

// =============================================================================
// Compositor Benchmarks
// =============================================================================

fn bench_plane_creation(c: &mut Criterion) {
    c.bench_function("plane_new_80x24", |b| {
        b.iter(|| Plane::new(black_box(0), black_box(80), black_box(24)))
    });

    c.bench_function("plane_new_200x60", |b| {
        b.iter(|| Plane::new(black_box(0), black_box(200), black_box(60)))
    });
}

fn bench_plane_fill_bg(c: &mut Criterion) {
    let theme = Theme::nord();
    c.bench_function("plane_fill_bg_80x24", |b| {
        let mut plane = Plane::new(0, 80, 24);
        b.iter(|| plane.fill_bg(black_box(theme.bg)))
    });

    c.bench_function("plane_fill_bg_200x60", |b| {
        let mut plane = Plane::new(0, 200, 60);
        b.iter(|| plane.fill_bg(black_box(theme.bg)))
    });
}

fn bench_plane_put_str(c: &mut Criterion) {
    c.bench_function("plane_put_str_short", |b| {
        let mut plane = Plane::new(0, 80, 24);
        let text = "Hello, world!";
        b.iter(|| plane.put_str(black_box(0), black_box(0), black_box(text)))
    });

    c.bench_function("plane_put_str_long", |b| {
        let mut plane = Plane::new(0, 80, 24);
        let text = "Hello, world! This is a test string for benchmarking plane operations.";
        b.iter(|| plane.put_str(black_box(0), black_box(0), black_box(text)))
    });
}

// =============================================================================
// Widget Rendering Benchmarks
// =============================================================================

fn bench_list_render(c: &mut Criterion) {
    let items_100: Vec<String> = (0..100).map(|i| format!("Item {}", i)).collect();
    c.bench_function("list_render_100_items", |b| {
        let list = List::new(items_100.clone());
        let area = Rect::new(0, 0, 40, 20);
        b.iter(|| list.render(black_box(area)))
    });

    let items_1k: Vec<String> = (0..1000).map(|i| format!("Item {}", i)).collect();
    c.bench_function("list_render_1000_items", |b| {
        let list = List::new(items_1k.clone());
        let area = Rect::new(0, 0, 40, 20);
        b.iter(|| list.render(black_box(area)))
    });
}

fn bench_table_render(c: &mut Criterion) {
    let rows_100: Vec<String> = (0..100).map(|i| format!("{}", i)).collect();
    let area = Rect::new(0, 0, 50, 20);
    c.bench_function("table_render_100_rows", |b| {
        b.iter_with_setup(
            || {
                Table::new(vec![
                    Column { header: "ID".into(), width: 10 },
                    Column { header: "Name".into(), width: 20 },
                    Column { header: "Value".into(), width: 15 },
                ])
                .with_rows(rows_100.clone())
            },
            |table| table.render(black_box(area)),
        )
    });

    let rows_1k: Vec<String> = (0..1000).map(|i| format!("{}", i)).collect();
    c.bench_function("table_render_1000_rows", |b| {
        b.iter_with_setup(
            || {
                Table::new(vec![
                    Column { header: "ID".into(), width: 10 },
                    Column { header: "Name".into(), width: 20 },
                    Column { header: "Value".into(), width: 15 },
                ])
                .with_rows(rows_1k.clone())
            },
            |table| table.render(black_box(area)),
        )
    });
}

// =============================================================================
// Focus Management Benchmarks
// =============================================================================

fn bench_focus_navigation(c: &mut Criterion) {
    c.bench_function("focus_tab_next_10_widgets", |b| {
        let mut fm = FocusManager::new();
        for i in 0..10 {
            fm.register(WidgetId::new(i), true);
        }
        fm.set_focus(WidgetId::new(0));
        b.iter(|| fm.tab_next())
    });

    c.bench_function("focus_tab_next_100_widgets", |b| {
        let mut fm = FocusManager::new();
        for i in 0..100 {
            fm.register(WidgetId::new(i), true);
        }
        fm.set_focus(WidgetId::new(0));
        b.iter(|| fm.tab_next())
    });

    c.bench_function("focus_tab_prev_100_widgets", |b| {
        let mut fm = FocusManager::new();
        for i in 0..100 {
            fm.register(WidgetId::new(i), true);
        }
        fm.set_focus(WidgetId::new(99));
        b.iter(|| fm.tab_prev())
    });
}

// =============================================================================
// Animation Benchmarks
// =============================================================================

fn bench_animation_tick(c: &mut Criterion) {
    c.bench_function("animation_tick_10", |b| {
        let mut manager = AnimationManager::new();
        for _ in 0..10 {
            manager.start(0.0, 100.0, Duration::from_secs(1));
        }
        b.iter(|| manager.tick())
    });

    c.bench_function("animation_tick_100", |b| {
        let mut manager = AnimationManager::new();
        for _ in 0..100 {
            manager.start(0.0, 100.0, Duration::from_secs(1));
        }
        b.iter(|| manager.tick())
    });
}

fn bench_animation_value(c: &mut Criterion) {
    c.bench_function("animation_value_linear", |b| {
        let anim = Animation::new(0.0, 100.0, Duration::from_secs(1));
        b.iter(|| anim.value())
    });

    c.bench_function("animation_value_ease_in", |b| {
        let anim = Animation::new(0.0, 100.0, Duration::from_secs(1)).with_easing(Easing::EaseIn);
        b.iter(|| anim.value())
    });
}

// =============================================================================
// HitZone Benchmarks
// =============================================================================

fn bench_hitzone_dispatch(c: &mut Criterion) {
    c.bench_function("hitzone_group_dispatch_10", |b| {
        let mut group = HitZoneGroup::new();
        for i in 0..10 {
            group.add_row(i, i as u16, 8, |_| {});
        }
        b.iter(|| {
            group.dispatch_mouse(
                dracon_terminal_engine::input::event::MouseEventKind::Down(
                    dracon_terminal_engine::input::event::MouseButton::Left,
                ),
                black_box(45),
                black_box(0),
                dracon_terminal_engine::input::event::KeyModifiers::empty(),
            )
        })
    });

    c.bench_function("hitzone_group_dispatch_100", |b| {
        let mut group = HitZoneGroup::new();
        for i in 0..100 {
            group.add_row(i, i as u16, 1, |_| {});
        }
        b.iter(|| {
            group.dispatch_mouse(
                dracon_terminal_engine::input::event::MouseEventKind::Down(
                    dracon_terminal_engine::input::event::MouseButton::Left,
                ),
                black_box(50),
                black_box(0),
                dracon_terminal_engine::input::event::KeyModifiers::empty(),
            )
        })
    });
}

fn bench_scoped_zone_registry(c: &mut Criterion) {
    c.bench_function("scoped_zone_dispatch_100", |b| {
        let mut registry = ScopedZoneRegistry::new();
        for i in 0..100 {
            registry.register(i, i as u16 * 2, 0, 1, 1);
        }
        b.iter(|| registry.dispatch(black_box(50), black_box(0)))
    });

    c.bench_function("scoped_zone_clear_and_register_100", |b| {
        let mut registry = ScopedZoneRegistry::new();
        b.iter(|| {
            registry.clear();
            for i in 0..100 {
                registry.register(i, i as u16 * 2, 0, 1, 1);
            }
        })
    });
}

// =============================================================================
// Theme Benchmarks
// =============================================================================

fn bench_theme_creation(c: &mut Criterion) {
    c.bench_function("theme_nord", |b| b.iter(Theme::nord));
    c.bench_function("theme_cyberpunk", |b| b.iter(Theme::cyberpunk));
    c.bench_function("theme_dracula", |b| b.iter(Theme::dracula));
    c.bench_function("theme_all_20", |b| {
        b.iter(|| {
            let _ = Theme::nord();
            let _ = Theme::cyberpunk();
            let _ = Theme::dracula();
            let _ = Theme::dark();
            let _ = Theme::light();
            let _ = Theme::catppuccin_mocha();
            let _ = Theme::gruvbox_dark();
            let _ = Theme::tokyo_night();
            let _ = Theme::solarized_dark();
            let _ = Theme::solarized_light();
            let _ = Theme::one_dark();
            let _ = Theme::rose_pine();
            let _ = Theme::kanagawa();
            let _ = Theme::everforest();
            let _ = Theme::monokai();
            let _ = Theme::warm();
            let _ = Theme::cool();
            let _ = Theme::forest();
            let _ = Theme::sunset();
            let _ = Theme::mono();
        })
    });
}

// =============================================================================
// Group Definitions
// =============================================================================

criterion_group!(
    compositor,
    bench_plane_creation,
    bench_plane_fill_bg,
    bench_plane_put_str
);
criterion_group!(widgets, bench_list_render, bench_table_render);
criterion_group!(focus, bench_focus_navigation);
criterion_group!(animation, bench_animation_tick, bench_animation_value);
criterion_group!(hitzone, bench_hitzone_dispatch, bench_scoped_zone_registry);
criterion_group!(theme, bench_theme_creation);

criterion_main!(compositor, widgets, focus, animation, hitzone, theme);

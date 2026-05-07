//! Performance benchmarks for core framework operations.
//!
//! Run with: `cargo bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::animation::{Animation, AnimationManager, Easing};
use dracon_terminal_engine::framework::focus::FocusManager;
use dracon_terminal_engine::framework::hitzone::{HitZone, HitZoneGroup};
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::{List, Table};
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
    c.bench_function("plane_fill_bg_80x24", |b| {
        let mut plane = Plane::new(0, 80, 24);
        let theme = Theme::nord();
        b.iter(|| plane.fill_bg(black_box(theme.bg)))
    });
}

fn bench_plane_put_str(c: &mut Criterion) {
    c.bench_function("plane_put_str_80x24", |b| {
        let mut plane = Plane::new(0, 80, 24);
        let text = "Hello, world! This is a test string for benchmarking.";
        b.iter(|| {
            plane.put_str(black_box(0), black_box(0), black_box(text));
        })
    });
}

// =============================================================================
// Widget Rendering Benchmarks
// =============================================================================

fn bench_list_render(c: &mut Criterion) {
    let items: Vec<String> = (0..100).map(|i| format!("Item {}", i)).collect();

    c.bench_function("list_render_100_items", |b| {
        let list = List::new(items.clone());
        let area = ratatui::layout::Rect::new(0, 0, 40, 20);
        b.iter(|| list.render(black_box(area)))
    });

    let items_1k: Vec<String> = (0..1000).map(|i| format!("Item {}", i)).collect();
    c.bench_function("list_render_1000_items", |b| {
        let list = List::new(items_1k.clone());
        let area = ratatui::layout::Rect::new(0, 0, 40, 20);
        b.iter(|| list.render(black_box(area)))
    });
}

fn bench_table_render(c: &mut Criterion) {
    let rows: Vec<Vec<String>> = (0..100)
        .map(|i| vec![format!("{}", i), format!("Name {}", i), format!("Value {}", i)])
        .collect();

    c.bench_function("table_render_100_rows", |b| {
        let table = Table::new(rows.clone(), vec![10, 20, 15]);
        let area = ratatui::layout::Rect::new(0, 0, 50, 20);
        b.iter(|| table.render(black_box(area)))
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
    c.bench_function("hitzone_dispatch_10_zones", |b| {
        let mut group = HitZoneGroup::new();
        for i in 0..10 {
            group.add(HitZone::new(i, i as u16 * 10, 0, 8, 1));
        }
        b.iter(|| group.dispatch(black_box(45), black_box(0)))
    });

    c.bench_function("hitzone_dispatch_100_zones", |b| {
        let mut group = HitZoneGroup::new();
        for i in 0..100 {
            group.add(HitZone::new(i, i as u16 * 2, 0, 1, 1));
        }
        b.iter(|| group.dispatch(black_box(50), black_box(0)))
    });
}

// =============================================================================
// Theme Benchmarks
// =============================================================================

fn bench_theme_creation(c: &mut Criterion) {
    c.bench_function("theme_nord", |b| b.iter(|| Theme::nord()));
    c.bench_function("theme_cyberpunk", |b| b.iter(|| Theme::cyberpunk()));
    c.bench_function("theme_dracula", |b| b.iter(|| Theme::dracula()));
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
criterion_group!(hitzone, bench_hitzone_dispatch);
criterion_group!(theme, bench_theme_creation);

criterion_main!(compositor, widgets, focus, animation, hitzone, theme);

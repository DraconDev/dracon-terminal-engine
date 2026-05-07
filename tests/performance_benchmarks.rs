//! Performance benchmarks — render large numbers of widgets, measure frame time.

use dracon_terminal_engine::compositor::{Compositor, Plane};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Button, Checkbox, Label, List};
use ratatui::layout::Rect;
use std::time::Instant;

#[test]
fn benchmark_render_100_buttons() {
    let start = Instant::now();
    
    for i in 0..100 {
        let mut btn = Button::with_id(WidgetId::new(i), "Test");
        btn.on_theme_change(&Theme::nord());
        let _ = btn.render(Rect::new(0, 0, 20, 3));
    }
    
    let elapsed = start.elapsed();
    println!("100 buttons rendered in {:?}", elapsed);
    assert!(elapsed.as_millis() < 1000, "Rendering 100 buttons took too long: {:?}", elapsed);
}

#[test]
fn benchmark_render_100_checkboxes() {
    let start = Instant::now();
    
    for i in 0..100 {
        let mut cb = Checkbox::new(WidgetId::new(i), "Option");
        cb.on_theme_change(&Theme::nord());
        let _ = cb.render(Rect::new(0, 0, 20, 1));
    }
    
    let elapsed = start.elapsed();
    println!("100 checkboxes rendered in {:?}", elapsed);
    assert!(elapsed.as_millis() < 1000);
}

#[test]
fn benchmark_compositor_50_planes() {
    let mut compositor = Compositor::new(80, 24);
    
    for i in 0..50 {
        let mut plane = Plane::new(i, 20, 10);
        plane.x = (i * 2) as u16 % 60;
        plane.y = (i * 2) as u16 % 14;
        plane.fill_bg(Color::Ansi((i % 256) as u8));
        compositor.add_plane(plane);
    }
    
    let start = Instant::now();
    let mut output = Vec::new();
    let _ = compositor.render(&mut output);
    let elapsed = start.elapsed();
    
    println!("Compositor with 50 planes: {:?}", elapsed);
    assert!(elapsed.as_millis() < 100);
}

#[test]
fn benchmark_compositor_200_planes() {
    let mut compositor = Compositor::new(80, 24);
    
    for i in 0..200 {
        let mut plane = Plane::new(i, 10, 5);
        plane.x = (i * 3) as u16 % 70;
        plane.y = (i * 3) as u16 % 19;
        plane.fill_bg(Color::Ansi((i % 256) as u8));
        compositor.add_plane(plane);
    }
    
    let start = Instant::now();
    let mut output = Vec::new();
    let _ = compositor.render(&mut output);
    let elapsed = start.elapsed();
    
    println!("Compositor with 200 planes: {:?}", elapsed);
    assert!(elapsed.as_millis() < 500);
}

#[test]
fn benchmark_list_with_1000_items() {
    let items: Vec<String> = (0..1000).map(|i| format!("Item {}", i)).collect();
    
    let mut list = List::new_with_id(WidgetId::new(1), items);
    list.on_theme_change(&Theme::nord());
    
    let start = Instant::now();
    let _ = list.render(Rect::new(0, 0, 40, 20));
    let elapsed = start.elapsed();
    
    println!("List with 1000 items: {:?}", elapsed);
    assert!(elapsed.as_millis() < 500);
}

#[test]
fn benchmark_theme_cycle_all_20_themes() {
    let themes = vec![
        Theme::dark(), Theme::light(), Theme::cyberpunk(),
        Theme::dracula(), Theme::nord(), Theme::catppuccin_mocha(),
        Theme::gruvbox_dark(), Theme::tokyo_night(), Theme::solarized_dark(),
        Theme::solarized_light(), Theme::one_dark(), Theme::rose_pine(),
        Theme::kanagawa(), Theme::everforest(), Theme::monokai(),
        Theme::warm(), Theme::cool(), Theme::forest(),
        Theme::sunset(), Theme::mono(),
    ];
    
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    
    let start = Instant::now();
    for theme in &themes {
        cb.on_theme_change(theme);
        let _ = cb.render(Rect::new(0, 0, 20, 1));
    }
    let elapsed = start.elapsed();
    
    println!("20 theme cycles: {:?}", elapsed);
    assert!(elapsed.as_millis() < 100);
}

#[test]
fn benchmark_large_terminal_200x100() {
    let mut compositor = Compositor::new(200, 100);
    
    let mut plane = Plane::new(0, 200, 100);
    plane.fill_bg(Color::Ansi(1));
    compositor.add_plane(plane);
    
    let start = Instant::now();
    let mut output = Vec::new();
    let _ = compositor.render(&mut output);
    let elapsed = start.elapsed();
    
    println!("200x100 terminal render: {:?}", elapsed);
    assert!(elapsed.as_millis() < 100);
}

#[test]
fn benchmark_widget_gallery_render() {
    // Simulate rendering all gallery widgets
    let widgets: Vec<Box<dyn Fn() -> Plane>> = vec![
        Box::new(|| {
            let mut cb = Checkbox::new(WidgetId::new(1), "Test");
            cb.on_theme_change(&Theme::nord());
            cb.render(Rect::new(0, 0, 20, 1))
        }),
        Box::new(|| {
            let mut btn = Button::with_id(WidgetId::new(2), "Click");
            btn.on_theme_change(&Theme::nord());
            btn.render(Rect::new(0, 0, 15, 3))
        }),
        Box::new(|| {
            let mut label = Label::new("Hello World");
            label.on_theme_change(&Theme::nord());
            label.render(Rect::new(0, 0, 20, 1))
        }),
    ];
    
    let start = Instant::now();
    for _ in 0..100 {
        for widget in &widgets {
            let _ = widget();
        }
    }
    let elapsed = start.elapsed();
    
    println!("100 gallery iterations: {:?}", elapsed);
    assert!(elapsed.as_millis() < 1000);
}

//! Performance benchmarks for framework systems.

use dracon_terminal_engine::framework::event_bus::EventBus;
use dracon_terminal_engine::framework::scene_router::{Scene, SceneRouter};
use dracon_terminal_engine::compositor::Plane;
use ratatui::layout::Rect;
use std::time::Instant;

// ═══════════════════════════════════════════════════════════════════════════════
// EventBus Throughput Benchmarks
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bench_event_bus_publish_throughput() {
    let bus = EventBus::new();
    let iterations = 100_000;

    let start = Instant::now();
    for i in 0..iterations {
        bus.publish(i);
    }
    let elapsed = start.elapsed();

    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
    println!("EventBus publish: {} ops/sec ({:?} total)", ops_per_sec as u64, elapsed);
    assert!(ops_per_sec > 1_000_000.0, "EventBus publish too slow: {} ops/sec", ops_per_sec);
}

#[test]
fn bench_event_bus_subscribe_and_publish() {
    let bus = EventBus::new();
    let _sub = bus.subscribe(|_msg: &usize| {});
    let iterations = 50_000;

    let start = Instant::now();
    for i in 0..iterations {
        bus.publish(i);
    }
    let elapsed = start.elapsed();

    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
    println!("EventBus publish+deliver: {} ops/sec ({:?} total)", ops_per_sec as u64, elapsed);
    assert!(ops_per_sec > 100_000.0, "EventBus delivery too slow: {} ops/sec", ops_per_sec);
}

#[test]
fn bench_event_bus_multiple_subscribers() {
    let bus = EventBus::new();
    for _ in 0..10 {
        let _sub = bus.subscribe(|_msg: &usize| {});
    }
    let iterations = 10_000;

    let start = Instant::now();
    for i in 0..iterations {
        bus.publish(i);
    }
    let elapsed = start.elapsed();

    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
    println!("EventBus 10 subscribers: {} ops/sec ({:?} total)", ops_per_sec as u64, elapsed);
    assert!(ops_per_sec > 50_000.0, "EventBus multi-subscriber too slow: {} ops/sec", ops_per_sec);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SceneRouter Benchmarks
// ═══════════════════════════════════════════════════════════════════════════════

struct BenchScene {
    id: &'static str,
    render_count: std::cell::Cell<usize>,
}

impl BenchScene {
    fn new(id: &'static str) -> Self {
        Self { id, render_count: std::cell::Cell::new(0) }
    }
}

impl Scene for BenchScene {
    fn scene_id(&self) -> &str { self.id }
    fn on_enter(&mut self) {}
    fn on_exit(&mut self) {}
    fn on_pause(&mut self) {}
    fn on_resume(&mut self) {}
    fn handle_key(&mut self, _key: dracon_terminal_engine::input::event::KeyEvent) -> bool { false }
    fn handle_mouse(&mut self, _kind: dracon_terminal_engine::input::event::MouseEventKind, _col: u16, _row: u16) -> bool { false }
    fn render(&self, _area: Rect) -> Plane {
        self.render_count.set(self.render_count.get() + 1);
        Plane::new(0, 80, 24)
    }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
}

#[test]
fn bench_scene_router_push_pop() {
    let mut router = SceneRouter::new();
    let iterations = 10_000;

    let start = Instant::now();
    for i in 0..iterations {
        router.register(&format!("scene_{}", i % 10), Box::new(BenchScene::new("bench")));
        router.push(&format!("scene_{}", i % 10));
    }
    for _ in 0..iterations {
        router.pop();
    }
    let elapsed = start.elapsed();

    let ops_per_sec = (iterations * 2) as f64 / elapsed.as_secs_f64();
    println!("SceneRouter push+pop: {} ops/sec ({:?} total)", ops_per_sec as u64, elapsed);
    assert!(ops_per_sec > 100_000.0, "SceneRouter too slow: {} ops/sec", ops_per_sec);
}

#[test]
fn bench_scene_router_render() {
    let mut router = SceneRouter::new();
    router.register("main", Box::new(BenchScene::new("main")));
    router.push("main");
    let iterations = 10_000;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = router.render(Rect::new(0, 0, 80, 24));
    }
    let elapsed = start.elapsed();

    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
    println!("SceneRouter render: {} ops/sec ({:?} total)", ops_per_sec as u64, elapsed);
    assert!(ops_per_sec > 10_000.0, "SceneRouter render too slow: {} ops/sec", ops_per_sec);
}

#[test]
fn bench_scene_router_deep_navigation() {
    let mut router = SceneRouter::new();
    for i in 0..100 {
        router.register(&format!("scene_{}", i), Box::new(BenchScene::new("bench")));
    }

    let start = Instant::now();
    for i in 0..100 {
        router.push(&format!("scene_{}", i));
    }
    let elapsed = start.elapsed();

    let ops_per_sec = 100.0 / elapsed.as_secs_f64();
    println!("SceneRouter deep push (100): {} ops/sec ({:?} total)", ops_per_sec as u64, elapsed);
    assert!(ops_per_sec > 1000.0, "SceneRouter deep nav too slow: {} ops/sec", ops_per_sec);
}

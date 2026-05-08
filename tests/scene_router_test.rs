//! Scene Router tests

use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::{Scene, SceneRouter};
use dracon_terminal_engine::input::event::KeyEvent;
use ratatui::layout::Rect;

// ═══════════════════════════════════════════════════════════════════════════════
// TEST SCENES
// ═══════════════════════════════════════════════════════════════════════════════

struct TestScene {
    id: String,
    entered: bool,
    exited: bool,
    paused: bool,
    resumed: bool,
    dirty: bool,
}

impl TestScene {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            entered: false,
            exited: false,
            paused: false,
            resumed: false,
            dirty: true,
        }
    }
}

impl Scene for TestScene {
    fn scene_id(&self) -> &str {
        &self.id
    }

    fn on_enter(&mut self) {
        self.entered = true;
    }

    fn on_exit(&mut self) {
        self.exited = true;
    }

    fn on_pause(&mut self) {
        self.paused = true;
    }

    fn on_resume(&mut self) {
        self.resumed = true;
    }

    fn render(&self, area: Rect) -> Plane {
        Plane::new(0, area.width, area.height)
    }

    fn handle_key(&mut self, _key: KeyEvent) -> bool {
        false
    }

    fn handle_mouse(&mut self, _kind: dracon_terminal_engine::input::event::MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BASIC NAVIGATION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_router_push_and_current() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));

    assert_eq!(router.current(), None);

    router.push("home");
    assert_eq!(router.current(), Some("home"));
    assert_eq!(router.stack_depth(), 1);
}

#[test]
fn test_router_push_unknown_scene_ignored() {
    let mut router = SceneRouter::new();
    router.push("nonexistent");
    assert_eq!(router.current(), None);
}

#[test]
fn test_router_pop() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.register("settings", Box::new(TestScene::new("settings")));

    router.push("home");
    router.push("settings");
    assert_eq!(router.current(), Some("settings"));

    let popped = router.pop();
    assert!(popped);
    assert_eq!(router.current(), Some("home"));
    assert_eq!(router.stack_depth(), 1);
}

#[test]
fn test_router_pop_root_not_allowed() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.push("home");

    let popped = router.pop();
    assert!(!popped);
    assert_eq!(router.current(), Some("home"));
}

#[test]
fn test_router_replace() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.register("settings", Box::new(TestScene::new("settings")));

    router.push("home");
    router.replace("settings");
    assert_eq!(router.current(), Some("settings"));
    assert_eq!(router.stack_depth(), 1);
}

#[test]
fn test_router_go() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.register("profile", Box::new(TestScene::new("profile")));

    router.push("home");
    router.push("profile");
    assert_eq!(router.stack_depth(), 2);

    router.go("home");
    assert_eq!(router.current(), Some("home"));
    assert_eq!(router.stack_depth(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// LIFECYCLE HOOKS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_router_lifecycle_hooks() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.register("settings", Box::new(TestScene::new("settings")));

    router.push("home");
    {
        let home = router.get_scene("home").unwrap();
        let home = home as &dyn std::any::Any;
        let home = home.downcast_ref::<TestScene>().unwrap();
        assert!(home.entered);
        assert!(!home.exited);
    }

    router.push("settings");
    {
        let home = router.get_scene("home").unwrap();
        let home = home as &dyn std::any::Any;
        let home = home.downcast_ref::<TestScene>().unwrap();
        assert!(home.paused);

        let settings = router.get_scene("settings").unwrap();
        let settings = settings as &dyn std::any::Any;
        let settings = settings.downcast_ref::<TestScene>().unwrap();
        assert!(settings.entered);
    }

    router.pop();
    {
        let settings = router.get_scene("settings").unwrap();
        let settings = settings as &dyn std::any::Any;
        let settings = settings.downcast_ref::<TestScene>().unwrap();
        assert!(settings.exited);

        let home = router.get_scene("home").unwrap();
        let home = home as &dyn std::any::Any;
        let home = home.downcast_ref::<TestScene>().unwrap();
        assert!(home.resumed);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// REGISTRATION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_router_register_and_unregister() {
    let mut router = SceneRouter::new();
    assert!(!router.has_scene("home"));

    router.register("home", Box::new(TestScene::new("home")));
    assert!(router.has_scene("home"));

    let removed = router.unregister("home");
    assert!(removed.is_some());
    assert!(!router.has_scene("home"));
}

#[test]
fn test_router_is_registered() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));

    assert!(router.is_registered("home"));
    assert!(!router.is_registered("unknown"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// STATE QUERIES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_router_can_go_back() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.register("settings", Box::new(TestScene::new("settings")));

    router.push("home");
    assert!(!router.can_go_back());

    router.push("settings");
    assert!(router.can_go_back());

    router.pop();
    assert!(!router.can_go_back());
}

// ═══════════════════════════════════════════════════════════════════════════════
// DIRTY TRACKING
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_router_needs_render() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));

    // No current scene
    assert!(!router.needs_render());

    router.push("home");
    // Scene is dirty by default
    assert!(router.needs_render());

    router.clear_dirty();
    // Also clear the scene's dirty flag
    if let Some(scene) = router.get_scene_mut("home") {
        scene.clear_dirty();
    }
    assert!(!router.needs_render());

    // Mark router dirty
    router.mark_dirty();
    assert!(router.needs_render());
}

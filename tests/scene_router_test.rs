//! Scene Router tests

use dracon_terminal_engine::compositor::Plane;
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

    fn handle_mouse(
        &mut self,
        _kind: dracon_terminal_engine::input::event::MouseEventKind,
        _col: u16,
        _row: u16,
    ) -> bool {
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
fn test_router_pop_force_clears_last_scene() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.push("home");

    let popped = router.pop_force();
    assert!(popped);
    assert_eq!(router.current(), None);
    assert_eq!(router.stack_depth(), 0);
}

#[test]
fn test_router_pop_force_empty_stack() {
    let mut router = SceneRouter::new();
    let popped = router.pop_force();
    assert!(!popped);
}

#[test]
fn test_router_pop_force_exits_scene() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.register("settings", Box::new(TestScene::new("settings")));

    router.push("home");
    router.push("settings");

    let popped = router.pop_force();
    assert!(popped);
    assert_eq!(router.current(), Some("home"));

    let settings = router.get_scene("settings").unwrap();
    let settings = settings as &dyn std::any::Any;
    let settings = settings.downcast_ref::<TestScene>().unwrap();
    assert!(settings.exited);

    let home = router.get_scene("home").unwrap();
    let home = home as &dyn std::any::Any;
    let home = home.downcast_ref::<TestScene>().unwrap();
    assert!(home.resumed);
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

#[test]
fn test_can_go_back_single_scene() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.push("home");
    assert!(!router.can_go_back());
}

#[test]
fn test_can_go_back_multiple_scenes() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.register("detail", Box::new(TestScene::new("detail")));
    router.push("home");
    router.push("detail");
    assert!(router.can_go_back());
    assert_eq!(router.stack_depth(), 2);
    router.pop();
    assert_eq!(router.stack_depth(), 1);
    assert!(!router.can_go_back());
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

use dracon_terminal_engine::framework::scene_router::SceneTransition;

#[test]
fn test_router_default_transition() {
    use dracon_terminal_engine::framework::scene_router::SceneTransition;
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));

    // Default is Fade
    assert_eq!(router.default_transition(), SceneTransition::Fade);

    // Set to SlideLeft via builder
    router = router.with_default_transition(SceneTransition::SlideLeft);
    assert_eq!(router.default_transition(), SceneTransition::SlideLeft);

    // Set to None
    router = router.with_default_transition(SceneTransition::None);
    assert_eq!(router.default_transition(), SceneTransition::None);
}

#[test]
fn test_router_default_transition_builder() {
    use dracon_terminal_engine::framework::scene_router::SceneTransition;
    let router = SceneRouter::new()
        .with_default_transition(SceneTransition::SlideUp)
        .with_default_duration(300.0);

    assert_eq!(router.default_transition(), SceneTransition::SlideUp);
    assert_eq!(router.default_duration_ms(), 300.0);
}

#[test]
fn test_router_default_duration() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));

    // Default is 200ms
    assert_eq!(router.default_duration_ms(), 200.0);

    // Set to 500ms via builder
    router = router.with_default_duration(500.0);
    assert_eq!(router.default_duration_ms(), 500.0);

    // Set to 0 (instant)
    router = router.with_default_duration(0.0);
    assert_eq!(router.default_duration_ms(), 0.0);
}

#[test]
fn test_router_is_transitioning() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.register("settings", Box::new(TestScene::new("settings")));

    // No transition initially
    assert!(!router.is_transitioning());

    // Push triggers transition
    router.push("home");
    // is_transitioning depends on timing, just verify no panic

    router.push("settings");
}

// ═══════════════════════════════════════════════════════════════════════════════
// THEME PROPAGATION
// ═══════════════════════════════════════════════════════════════════════════════

use dracon_terminal_engine::framework::theme::Theme;

#[test]
fn test_router_theme_propagation() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));

    // Set theme via on_theme_change
    let theme = Theme::nord();
    router.on_theme_change(&theme);

    router.push("home");

    // Theme should be stored
    assert!(router.theme().is_some());
}

#[test]
fn test_router_theme_stored() {
    let mut router = SceneRouter::new();
    let theme = Theme::dracula();

    // No theme initially
    assert!(router.theme().is_none());

    // Set theme
    router.on_theme_change(&theme);
    assert!(router.theme().is_some());

    // Theme name should match
    let stored = router.theme().unwrap();
    assert_eq!(stored.name, theme.name);
}

// ═══════════════════════════════════════════════════════════════════════════════
// NAVIGATION EVENTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_router_navigation_events() {
    use dracon_terminal_engine::framework::scene_router::NavigationEvent;

    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.register("settings", Box::new(TestScene::new("settings")));

    // Collect events
    let events = router.take_navigation_events();
    assert!(events.is_empty());

    // Push generates Navigated event
    router.push("home");
    let events = router.take_navigation_events();
    assert_eq!(events.len(), 1);
    match &events[0] {
        NavigationEvent::Navigated(from, to) => {
            assert!(from.is_none());
            assert_eq!(to, "home");
        }
        _ => panic!("Expected Navigated event"),
    }

    // Push another scene
    router.push("settings");
    let events = router.take_navigation_events();
    assert_eq!(events.len(), 1);
    match &events[0] {
        NavigationEvent::Navigated(from, to) => {
            assert_eq!(from.as_deref(), Some("home"));
            assert_eq!(to, "settings");
        }
        _ => panic!("Expected Navigated event"),
    }

    // Pop generates Popped event
    router.pop();
    let events = router.take_navigation_events();
    assert_eq!(events.len(), 1);
    match &events[0] {
        NavigationEvent::Popped(from, to) => {
            assert_eq!(from, "settings");
            assert_eq!(to, "home");
        }
        _ => panic!("Expected Popped event"),
    }
}

#[test]
fn test_router_replace_event() {
    use dracon_terminal_engine::framework::scene_router::NavigationEvent;

    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.register("settings", Box::new(TestScene::new("settings")));

    router.push("home");
    router.take_navigation_events(); // clear

    // Replace generates Replaced event
    router.replace("settings");
    let events = router.take_navigation_events();
    assert_eq!(events.len(), 1);
    match &events[0] {
        NavigationEvent::Replaced(from, to) => {
            assert_eq!(from, "home");
            assert_eq!(to, "settings");
        }
        _ => panic!("Expected Replaced event"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_router_push_same_scene_twice() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));

    router.push("home");
    router.push("home");

    // Should have two instances on stack
    assert_eq!(router.stack_depth(), 2);
    assert_eq!(router.current(), Some("home"));
}

#[test]
fn test_router_pop_empty_stack() {
    let mut router = SceneRouter::new();
    let popped = router.pop();
    assert!(!popped);
}

#[test]
fn test_router_go_unknown_scene() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.push("home");

    // Go to unknown scene should be ignored (no panic)
    router.go("unknown");
    assert_eq!(router.current(), Some("home"));
}

#[test]
fn test_router_replace_empty_stack() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));

    // Replace on empty stack should push
    router.replace("home");
    assert_eq!(router.current(), Some("home"));
    assert_eq!(router.stack_depth(), 1);
}

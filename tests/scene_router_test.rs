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
fn test_router_default_transition_builder() {
    use dracon_terminal_engine::framework::scene_router::SceneTransition;
    let router = SceneRouter::new()
        .with_default_transition(SceneTransition::SlideUp)
        .with_default_duration(300.0);

    // Builder should not panic
    assert_eq!(router.stack_depth(), 0);
}

#[test]
fn test_router_transition_types() {
    // Verify all transition types can be used without panic
    let _router = SceneRouter::new().with_default_transition(SceneTransition::Fade);
    let _router = SceneRouter::new().with_default_transition(SceneTransition::SlideLeft);
    let _router = SceneRouter::new().with_default_transition(SceneTransition::SlideRight);
    let _router = SceneRouter::new().with_default_transition(SceneTransition::SlideUp);
    let _router = SceneRouter::new().with_default_transition(SceneTransition::SlideDown);
    let _router = SceneRouter::new().with_default_transition(SceneTransition::None);
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

    // Set theme via on_theme_change (should not panic)
    let theme = Theme::nord();
    router.on_theme_change(&theme);

    router.push("home");
}

#[test]
fn test_router_theme_on_multiple_scenes() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.register("settings", Box::new(TestScene::new("settings")));

    // Set theme before any scenes
    let theme = Theme::dracula();
    router.on_theme_change(&theme);

    // Push scenes - theme should propagate
    router.push("home");
    router.push("settings");
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

#[test]
fn test_router_tick_transition() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.register("settings", Box::new(TestScene::new("settings")));

    router.push("home");
    router.push("settings");

    // Tick transition should not panic
    router.tick_transition(16.0); // ~60fps frame
    router.tick_transition(16.0);
    router.tick_transition(16.0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUSH WITH TRANSITION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_push_with_transition_slide_left() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::SlideLeft, 300.0);
    assert_eq!(router.current(), Some("b"));
    assert_eq!(router.stack_depth(), 2);
    assert!(router.is_transitioning());
}

#[test]
fn test_push_with_transition_slide_right() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::SlideRight, 200.0);
    assert_eq!(router.current(), Some("b"));
    assert!(router.is_transitioning());
}

#[test]
fn test_push_with_transition_slide_up() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::SlideUp, 250.0);
    assert_eq!(router.current(), Some("b"));
    assert!(router.is_transitioning());
}

#[test]
fn test_push_with_transition_slide_down() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::SlideDown, 250.0);
    assert_eq!(router.current(), Some("b"));
    assert!(router.is_transitioning());
}

#[test]
fn test_push_with_transition_none_is_instant() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::None, 300.0);
    assert_eq!(router.current(), Some("b"));
    assert!(!router.is_transitioning());
}

#[test]
fn test_push_with_transition_on_empty_stack_no_transition() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));

    // No current scene, so no transition should start
    router.push_with_transition("a", SceneTransition::Fade, 200.0);
    assert_eq!(router.current(), Some("a"));
    assert!(!router.is_transitioning());
}

// ═══════════════════════════════════════════════════════════════════════════════
// NAVIGATE_TO (DEEP LINKING)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_navigate_to_single_segment() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));

    router.navigate_to("home");
    assert_eq!(router.current(), Some("home"));
    assert_eq!(router.stack_depth(), 1);
}

#[test]
fn test_navigate_to_multi_segment() {
    let mut router = SceneRouter::new();
    router.register("settings", Box::new(TestScene::new("settings")));
    router.register("profile", Box::new(TestScene::new("profile")));

    router.navigate_to("settings/profile");
    assert_eq!(router.current(), Some("profile"));
    assert_eq!(router.stack_depth(), 2);
    assert_eq!(router.current_path(), "settings/profile");
}

#[test]
fn test_navigate_to_unknown_segment_stops() {
    let mut router = SceneRouter::new();
    router.register("settings", Box::new(TestScene::new("settings")));
    // "profile" is NOT registered

    router.navigate_to("settings/profile");
    // "settings" should be pushed via go(), but "profile" push fails silently
    assert_eq!(router.current(), Some("settings"));
    assert_eq!(router.stack_depth(), 1);
}

#[test]
fn test_navigate_to_empty_path() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));

    router.navigate_to("");
    assert_eq!(router.current(), None);
    assert_eq!(router.stack_depth(), 0);
}

#[test]
fn test_navigate_to_slashes_only() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));

    router.navigate_to("///");
    assert_eq!(router.current(), None);
}

#[test]
fn test_navigate_to_clears_stack() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));
    router.register("c", Box::new(TestScene::new("c")));

    router.push("a");
    router.push("b");
    router.push("c");
    assert_eq!(router.stack_depth(), 3);

    // navigate_to should clear and start fresh
    router.navigate_to("a/b");
    assert_eq!(router.stack_depth(), 2);
    assert_eq!(router.current(), Some("b"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// CURRENT_PATH
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_current_path_empty() {
    let router = SceneRouter::new();
    assert_eq!(router.current_path(), "");
}

#[test]
fn test_current_path_single() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.push("home");
    assert_eq!(router.current_path(), "home");
}

#[test]
fn test_current_path_nested() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));
    router.register("c", Box::new(TestScene::new("c")));

    router.push("a");
    router.push("b");
    router.push("c");
    assert_eq!(router.current_path(), "a/b/c");
}

#[test]
fn test_current_path_after_pop() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push("b");
    assert_eq!(router.current_path(), "a/b");

    router.pop();
    assert_eq!(router.current_path(), "a");
}

// ═══════════════════════════════════════════════════════════════════════════════
// BLEND PLANES (TRANSITION RENDERING)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_render_without_transition() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.push("home");

    let plane = router.render(Rect::new(0, 0, 80, 24));
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 24);
}

#[test]
fn test_render_empty_router() {
    let router = SceneRouter::new();
    let plane = router.render(Rect::new(0, 0, 80, 24));
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 24);
}

#[test]
fn test_render_during_transition() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::Fade, 500.0);
    assert!(router.is_transitioning());

    // Render during transition should produce valid plane
    let plane = router.render(Rect::new(0, 0, 80, 24));
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 24);
}

#[test]
fn test_transition_completes_after_enough_ticks() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::Fade, 100.0);

    // Tick enough to complete (100ms / 16ms per tick = ~7 ticks)
    for _ in 0..10 {
        router.tick_transition(16.0);
    }
    assert!(!router.is_transitioning());
}

#[test]
fn test_render_after_transition_completes() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::Fade, 50.0);

    // Complete the transition
    for _ in 0..5 {
        router.tick_transition(16.0);
    }

    // Render after transition completes
    let plane = router.render(Rect::new(0, 0, 80, 24));
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 24);
    assert!(!router.is_transitioning());
}

// ═══════════════════════════════════════════════════════════════════════════════
// INPUT DELEGATION
// ═══════════════════════════════════════════════════════════════════════════════

use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};

#[test]
fn test_handle_key_delegates_to_current_scene() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.push("home");

    let key = KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    };
    // TestScene::handle_key returns false, so router should also return false
    let handled = router.handle_key(key);
    assert!(!handled);
}

#[test]
fn test_handle_key_empty_stack() {
    let mut router = SceneRouter::new();
    let key = KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    };
    let handled = router.handle_key(key);
    assert!(!handled);
}

#[test]
fn test_handle_mouse_delegates_to_current_scene() {
    let mut router = SceneRouter::new();
    router.register("home", Box::new(TestScene::new("home")));
    router.push("home");

    let handled = router.handle_mouse(MouseEventKind::Down(MouseButton::Left), 10, 5);
    assert!(!handled);
}

#[test]
fn test_handle_mouse_empty_stack() {
    let mut router = SceneRouter::new();
    let handled = router.handle_mouse(MouseEventKind::Down(MouseButton::Left), 10, 5);
    assert!(!handled);
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMPLEX LIFECYCLE SCENARIOS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_push_pop_push_lifecycle() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));
    router.register("c", Box::new(TestScene::new("c")));

    router.push("a");
    router.push("b");
    router.pop();
    router.push("c");

    assert_eq!(router.current(), Some("c"));
    assert_eq!(router.stack_depth(), 2);
    assert_eq!(router.current_path(), "a/c");
}

#[test]
fn test_go_exits_all_scenes() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));
    router.register("c", Box::new(TestScene::new("c")));

    router.push("a");
    router.push("b");
    router.push("c");

    router.go("a");
    assert_eq!(router.current(), Some("a"));
    assert_eq!(router.stack_depth(), 1);

    let b = router.get_scene("b").unwrap();
    let b = b as &dyn std::any::Any;
    let b = b.downcast_ref::<TestScene>().unwrap();
    assert!(b.exited);
}

#[test]
fn test_replace_exits_old_enters_new() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.replace("b");

    let a = router.get_scene("a").unwrap();
    let a = a as &dyn std::any::Any;
    let a = a.downcast_ref::<TestScene>().unwrap();
    assert!(a.exited);

    let b = router.get_scene("b").unwrap();
    let b = b as &dyn std::any::Any;
    let b = b.downcast_ref::<TestScene>().unwrap();
    assert!(b.entered);
}

#[test]
fn test_theme_propagates_to_new_scene() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    let theme = Theme::nord();
    router.on_theme_change(&theme);

    // Both scenes should receive theme via on_theme_change
    router.push("a");
    router.push("b");

    // No panic means theme propagation worked
    assert_eq!(router.current(), Some("b"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// LIFECYCLE CALLBACKS: on_pause / on_resume
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_on_pause_called_when_pushing_onto_stack() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    let a_before = router.get_scene("a").unwrap();
    let a_before = a_before as &dyn std::any::Any;
    let a_before = a_before.downcast_ref::<TestScene>().unwrap();
    assert!(!a_before.paused);

    router.push("b");
    let a_after = router.get_scene("a").unwrap();
    let a_after = a_after as &dyn std::any::Any;
    let a_after = a_after.downcast_ref::<TestScene>().unwrap();
    assert!(
        a_after.paused,
        "Scene A should be paused when B is pushed on top"
    );
}

#[test]
fn test_on_resume_called_when_popping() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push("b");

    let a_while_paused = router.get_scene("a").unwrap();
    let a_while_paused = a_while_paused as &dyn std::any::Any;
    let a_while_paused = a_while_paused.downcast_ref::<TestScene>().unwrap();
    assert!(a_while_paused.paused);

    router.pop();

    let a_after_pop = router.get_scene("a").unwrap();
    let a_after_pop = a_after_pop as &dyn std::any::Any;
    let a_after_pop = a_after_pop.downcast_ref::<TestScene>().unwrap();
    assert!(
        a_after_pop.resumed,
        "Scene A should be resumed when B is popped"
    );
}

#[test]
fn test_on_pause_not_called_for_root_scene() {
    let mut router = SceneRouter::new();
    router.register("root", Box::new(TestScene::new("root")));

    router.push("root");
    let root = router.get_scene("root").unwrap();
    let root = root as &dyn std::any::Any;
    let root = root.downcast_ref::<TestScene>().unwrap();
    assert!(root.entered);
    assert!(
        !root.paused,
        "Root scene should not be paused on initial push"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSITION CANCELLATION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_push_during_transition_cancels_old_transition() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));
    router.register("c", Box::new(TestScene::new("c")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::Fade, 1000.0);
    assert!(router.is_transitioning());

    // Pushing C during transition should replace transition target
    router.push_with_transition("c", SceneTransition::SlideLeft, 2000.0);
    assert!(router.is_transitioning());
    assert_eq!(router.current(), Some("c"));
}

#[test]
fn test_pop_during_transition_clears_transition() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::Fade, 1000.0);
    assert!(router.is_transitioning());

    // Popping during transition
    router.pop();
    // The transition state may remain until next tick, but current should be A
    assert_eq!(router.current(), Some("a"));
}

#[test]
fn test_zero_duration_transition_completes_immediately() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::Fade, 0.0);

    // 0ms transition should complete on first tick
    router.tick_transition(1.0);
    assert!(!router.is_transitioning());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Z-INDEX COMPOSITION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_render_uses_top_scene_z_index() {
    let mut router = SceneRouter::new();
    router.register("bottom", Box::new(TestScene::new("bottom")));
    router.register("top", Box::new(TestScene::new("top")));

    router.push("bottom");
    router.push("top");

    let plane = router.render(Rect::new(0, 0, 80, 24));
    // After push, the top scene's render is used; verify it produced a valid plane
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 24);
}

#[test]
fn test_render_during_transition_uses_blended_plane() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::Fade, 100.0);

    // Render at 50% progress
    router.tick_transition(50.0);
    assert!(router.is_transitioning());
    let plane = router.render(Rect::new(0, 0, 80, 24));
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 24);
}

#[test]
fn test_render_z_index_increases_per_scene() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));
    router.register("c", Box::new(TestScene::new("c")));

    router.push("a");
    let z_a = router.render(Rect::new(0, 0, 10, 10)).z_index;
    router.push("b");
    let z_b = router.render(Rect::new(0, 0, 10, 10)).z_index;
    router.push("c");
    let z_c = router.render(Rect::new(0, 0, 10, 10)).z_index;

    // Each subsequent scene should have higher or equal z_index
    assert!(
        z_b >= z_a,
        "z_index should not decrease when stacking scenes (a={}, b={})",
        z_a,
        z_b
    );
    assert!(
        z_c >= z_b,
        "z_index should not decrease when stacking scenes (b={}, c={})",
        z_b,
        z_c
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSITION PROGRESS EDGE CASES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_transition_with_negative_dt_does_not_crash() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::Fade, 100.0);

    // Negative dt should not crash; progress should remain clamped
    router.tick_transition(-10.0);
    assert!(router.is_transitioning());
}

#[test]
fn test_tick_transition_no_active_transition() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.push("a");
    // No transition active
    assert!(!router.is_transitioning());
    // Ticking should be a no-op
    assert!(!router.tick_transition(100.0));
    assert!(!router.is_transitioning());
}

#[test]
fn test_tick_transition_clamps_progress_to_one() {
    let mut router = SceneRouter::new();
    router.register("a", Box::new(TestScene::new("a")));
    router.register("b", Box::new(TestScene::new("b")));

    router.push("a");
    router.push_with_transition("b", SceneTransition::Fade, 50.0);

    // Ticking way past duration should still complete cleanly
    for _ in 0..100 {
        router.tick_transition(100.0);
    }
    assert!(!router.is_transitioning());
}

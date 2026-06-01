//! Plugin Registry integration tests

use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::plugin::PluginRegistry;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use ratatui::layout::Rect;

// ═══════════════════════════════════════════════════════════════════════════════
// TEST WIDGETS
// ═══════════════════════════════════════════════════════════════════════════════

struct AlphaWidget {
    id: WidgetId,
    theme: Theme,
}

impl Widget for AlphaWidget {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn area(&self) -> Rect {
        Rect::new(0, 0, 20, 10)
    }
    fn set_area(&mut self, _: Rect) {}
    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        plane
    }
}

struct BetaWidget {
    id: WidgetId,
    theme: Theme,
}

impl Widget for BetaWidget {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn area(&self) -> Rect {
        Rect::new(0, 0, 30, 15)
    }
    fn set_area(&mut self, _: Rect) {}
    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.primary);
        plane
    }
}

struct GammaWidget {
    id: WidgetId,
    theme: Theme,
}

impl Widget for GammaWidget {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn area(&self) -> Rect {
        Rect::new(0, 0, 40, 20)
    }
    fn set_area(&mut self, _: Rect) {}
    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.char = 'X';
            cell.fg = self.theme.primary;
        }
        plane
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// REGISTRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn register_multiple_plugins() {
    let mut registry = PluginRegistry::new();
    assert!(registry.register("alpha", |id, theme| Box::new(AlphaWidget { id, theme })));
    assert!(registry.register("beta", |id, theme| Box::new(BetaWidget { id, theme })));
    assert!(registry.register("gamma", |id, theme| Box::new(GammaWidget { id, theme })));

    assert_eq!(registry.len(), 3);
    assert!(registry.has("alpha"));
    assert!(registry.has("beta"));
    assert!(registry.has("gamma"));
}

#[test]
fn register_duplicate_name_fails() {
    let mut registry = PluginRegistry::new();
    assert!(registry.register("widget_a", |id, theme| Box::new(AlphaWidget { id, theme })));
    assert!(!registry.register("widget_a", |id, theme| Box::new(BetaWidget { id, theme })));
    assert_eq!(registry.len(), 1);
}

#[test]
fn register_empty_name_succeeds() {
    let mut registry = PluginRegistry::new();
    assert!(registry.register("", |id, theme| Box::new(AlphaWidget { id, theme })));
    assert!(registry.has(""));
    assert_eq!(registry.len(), 1);
}

#[test]
fn register_special_characters_name() {
    let mut registry = PluginRegistry::new();
    assert!(
        registry.register("my-widget.v2!@#$%", |id, theme| Box::new(AlphaWidget {
            id,
            theme
        }))
    );
    assert!(registry.has("my-widget.v2!@#$%"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// CREATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn create_with_specific_id() {
    let mut registry = PluginRegistry::new();
    registry.register("alpha", |id, theme| Box::new(AlphaWidget { id, theme }));

    let widget = registry.create("alpha", WidgetId::new(42), Theme::default());
    assert!(widget.is_some());
    assert_eq!(widget.unwrap().id().0, 42);
}

#[test]
fn create_with_nord_theme() {
    let mut registry = PluginRegistry::new();
    registry.register("alpha", |id, theme| Box::new(AlphaWidget { id, theme }));

    let theme = Theme::nord();
    let widget = registry.create("alpha", WidgetId::new(1), theme.clone());
    assert!(widget.is_some());

    // Render to verify theme is applied
    let plane = widget.unwrap().render(Rect::new(0, 0, 20, 10));
    assert_eq!(plane.cells[0].bg, theme.bg);
}

#[test]
fn create_with_dracula_theme() {
    let mut registry = PluginRegistry::new();
    registry.register("beta", |id, theme| Box::new(BetaWidget { id, theme }));

    let theme = Theme::dracula();
    let widget = registry.create("beta", WidgetId::new(1), theme.clone());
    assert!(widget.is_some());

    let plane = widget.unwrap().render(Rect::new(0, 0, 30, 15));
    assert_eq!(plane.cells[0].bg, theme.primary);
}

#[test]
fn create_multiple_instances_same_plugin() {
    let mut registry = PluginRegistry::new();
    registry.register("alpha", |id, theme| Box::new(AlphaWidget { id, theme }));

    let w1 = registry.create("alpha", WidgetId::new(1), Theme::default());
    let w2 = registry.create("alpha", WidgetId::new(2), Theme::default());
    let w3 = registry.create("alpha", WidgetId::new(3), Theme::default());

    assert!(w1.is_some());
    assert!(w2.is_some());
    assert!(w3.is_some());
    assert_eq!(w1.unwrap().id().0, 1);
    assert_eq!(w2.unwrap().id().0, 2);
    assert_eq!(w3.unwrap().id().0, 3);
}

#[test]
fn create_unknown_plugin_returns_none() {
    let registry = PluginRegistry::new();
    assert!(registry
        .create("nonexistent", WidgetId::new(1), Theme::default())
        .is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNREGISTRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn unregister_existing_plugin() {
    let mut registry = PluginRegistry::new();
    registry.register("temp", |id, theme| Box::new(AlphaWidget { id, theme }));

    assert!(registry.has("temp"));
    assert!(registry.unregister("temp"));
    assert!(!registry.has("temp"));
    assert_eq!(registry.len(), 0);
}

#[test]
fn unregister_nonexistent_plugin() {
    let mut registry = PluginRegistry::new();
    assert!(!registry.unregister("ghost"));
}

#[test]
fn unregister_one_of_many() {
    let mut registry = PluginRegistry::new();
    registry.register("a", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.register("b", |id, theme| Box::new(BetaWidget { id, theme }));
    registry.register("c", |id, theme| Box::new(GammaWidget { id, theme }));

    registry.unregister("b");
    assert_eq!(registry.len(), 2);
    assert!(registry.has("a"));
    assert!(!registry.has("b"));
    assert!(registry.has("c"));
}

#[test]
fn unregister_then_reregister() {
    let mut registry = PluginRegistry::new();
    registry.register("widget", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.unregister("widget");

    // Should be able to re-register with a different factory
    assert!(registry.register("widget", |id, theme| Box::new(BetaWidget { id, theme })));
    let widget = registry.create("widget", WidgetId::new(1), Theme::default());
    assert!(widget.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// LIST TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn list_empty_registry() {
    let registry = PluginRegistry::new();
    let list = registry.list();
    assert!(list.is_empty());
}

#[test]
fn list_multiple_plugins() {
    let mut registry = PluginRegistry::new();
    registry.register("alpha", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.register("beta", |id, theme| Box::new(BetaWidget { id, theme }));
    registry.register("gamma", |id, theme| Box::new(GammaWidget { id, theme }));

    let mut list = registry.list();
    list.sort();
    assert_eq!(list, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn list_after_unregister() {
    let mut registry = PluginRegistry::new();
    registry.register("a", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.register("b", |id, theme| Box::new(BetaWidget { id, theme }));
    registry.register("c", |id, theme| Box::new(GammaWidget { id, theme }));

    registry.unregister("b");
    let mut list = registry.list();
    list.sort();
    assert_eq!(list, vec!["a", "c"]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// FACTORY FUNCTION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn factory_receives_correct_theme() {
    use std::sync::{Arc, Mutex};

    let captured_theme: Arc<Mutex<Option<Theme>>> = Arc::new(Mutex::new(None));
    let captured_theme_clone = captured_theme.clone();

    let mut registry = PluginRegistry::new();
    registry.register("spy", move |id, theme| {
        *captured_theme_clone.lock().unwrap() = Some(theme);
        Box::new(AlphaWidget {
            id,
            theme: Theme::default(),
        })
    });

    let theme = Theme::cyberpunk();
    let _ = registry.create("spy", WidgetId::new(1), theme.clone());

    let captured = captured_theme.lock().unwrap();
    assert!(captured.is_some());
    assert_eq!(captured.as_ref().unwrap().name, theme.name);
}

#[test]
fn factory_receives_correct_id() {
    use std::sync::{Arc, Mutex};

    let captured_id: Arc<Mutex<Option<WidgetId>>> = Arc::new(Mutex::new(None));
    let captured_id_clone = captured_id.clone();

    let mut registry = PluginRegistry::new();
    registry.register("spy_id", move |id, theme| {
        *captured_id_clone.lock().unwrap() = Some(id);
        Box::new(AlphaWidget { id, theme })
    });

    let _ = registry.create("spy_id", WidgetId::new(999), Theme::default());

    let captured = captured_id.lock().unwrap();
    assert!(captured.is_some());
    assert_eq!(captured.unwrap().0, 999);
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn register_unregister_cycle() {
    let mut registry = PluginRegistry::new();
    for i in 0..10 {
        let name = format!("plugin_{}", i);
        registry.register(&name, |id, theme| Box::new(AlphaWidget { id, theme }));
    }
    assert_eq!(registry.len(), 10);

    for i in 0..10 {
        let name = format!("plugin_{}", i);
        registry.unregister(&name);
    }
    assert_eq!(registry.len(), 0);
    assert!(registry.is_empty());
}

#[test]
fn create_after_unregister_returns_none() {
    let mut registry = PluginRegistry::new();
    registry.register("temp", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.unregister("temp");

    assert!(registry
        .create("temp", WidgetId::new(1), Theme::default())
        .is_none());
}

#[test]
fn many_plugins_performance() {
    let mut registry = PluginRegistry::new();
    for i in 0..100 {
        let name = format!("widget_{}", i);
        registry.register(&name, |id, theme| Box::new(AlphaWidget { id, theme }));
    }
    assert_eq!(registry.len(), 100);

    // Create all 100
    for i in 0..100 {
        let name = format!("widget_{}", i);
        let widget = registry.create(&name, WidgetId::new(i), Theme::default());
        assert!(widget.is_some());
    }

    // Unregister all
    for i in 0..100 {
        let name = format!("widget_{}", i);
        registry.unregister(&name);
    }
    assert!(registry.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// FAILURE PATH TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn reuse_name_after_unregister_succeeds() {
    let mut registry = PluginRegistry::new();
    assert!(
        registry.register("recyclable", |id, theme| Box::new(AlphaWidget {
            id,
            theme
        }))
    );
    assert!(!registry.register("recyclable", |id, theme| Box::new(BetaWidget { id, theme })));
    assert!(registry.unregister("recyclable"));
    assert!(registry.register("recyclable", |id, theme| Box::new(BetaWidget { id, theme })));
    assert!(registry.has("recyclable"));
    assert_eq!(registry.len(), 1);
}

#[test]
fn unregister_empty_string_returns_false() {
    let mut registry = PluginRegistry::new();
    assert!(!registry.unregister(""));
    assert!(registry.is_empty());
}

#[test]
fn unregister_after_multiple_registrations() {
    let mut registry = PluginRegistry::new();
    registry.register("a", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.register("b", |id, theme| Box::new(BetaWidget { id, theme }));
    registry.register("c", |id, theme| Box::new(GammaWidget { id, theme }));

    assert!(registry.unregister("b"));
    assert!(!registry.has("b"));
    assert!(registry.has("a"));
    assert!(registry.has("c"));
    assert_eq!(registry.len(), 2);
}

#[test]
fn create_with_empty_name_returns_none() {
    let registry = PluginRegistry::new();
    assert!(registry
        .create("", WidgetId::new(1), Theme::default())
        .is_none());
}

#[test]
fn create_with_unknown_name_returns_none() {
    let registry = PluginRegistry::new();
    assert!(registry
        .create("nonexistent", WidgetId::new(1), Theme::default())
        .is_none());
}

#[test]
fn has_returns_false_for_unknown() {
    let mut registry = PluginRegistry::new();
    registry.register("real", |id, theme| Box::new(AlphaWidget { id, theme }));
    assert!(!registry.has("fake"));
    assert!(!registry.has(""));
}

#[test]
fn default_constructor_creates_empty_registry() {
    let registry: PluginRegistry = PluginRegistry::default();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert!(!registry.has("anything"));
    assert!(registry.list().is_empty());
}

#[test]
fn list_returns_all_registered_names() {
    let mut registry = PluginRegistry::new();
    let names = vec!["first", "second", "third", "fourth"];
    for name in &names {
        registry.register(name, |id, theme| Box::new(AlphaWidget { id, theme }));
    }
    let listed = registry.list();
    assert_eq!(listed.len(), 4);
    for name in &names {
        assert!(listed.contains(&name.to_string()));
    }
}

#[test]
fn list_after_partial_unregister() {
    let mut registry = PluginRegistry::new();
    registry.register("a", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.register("b", |id, theme| Box::new(BetaWidget { id, theme }));
    registry.register("c", |id, theme| Box::new(GammaWidget { id, theme }));
    registry.unregister("b");

    let listed = registry.list();
    assert_eq!(listed.len(), 2);
    assert!(!listed.contains(&"b".to_string()));
}

#[test]
fn register_overwrites_keeps_old_count() {
    let mut registry = PluginRegistry::new();
    registry.register("widget", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.register("widget", |id, theme| Box::new(BetaWidget { id, theme }));
    assert_eq!(registry.len(), 1);

    let widget = registry.create("widget", WidgetId::new(1), Theme::default());
    assert!(widget.is_some());
    assert_eq!(widget.unwrap().id().0, 1);
}

#[test]
fn unregister_returns_true_then_false_for_same_name() {
    let mut registry = PluginRegistry::new();
    registry.register("x", |id, theme| Box::new(AlphaWidget { id, theme }));
    assert!(registry.unregister("x"));
    assert!(!registry.unregister("x"));
}

#[test]
fn create_widget_can_be_rendered() {
    let mut registry = PluginRegistry::new();
    registry.register("alpha", |id, theme| Box::new(AlphaWidget { id, theme }));

    let widget = registry
        .create("alpha", WidgetId::new(7), Theme::nord())
        .unwrap();
    let plane = widget.render(Rect::new(0, 0, 20, 10));
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 10);
}

#[test]
fn register_after_many_unregisters_still_works() {
    let mut registry = PluginRegistry::new();
    for i in 0..50 {
        let name = format!("plugin_{}", i);
        registry.register(&name, |id, theme| Box::new(AlphaWidget { id, theme }));
    }
    for i in 0..50 {
        let name = format!("plugin_{}", i);
        registry.unregister(&name);
    }
    assert!(registry.is_empty());

    assert!(registry.register("new_one", |id, theme| Box::new(BetaWidget { id, theme })));
    assert_eq!(registry.len(), 1);
    assert!(registry.has("new_one"));
}

#[test]
fn thread_safe_create() {
    use std::sync::Arc;
    use std::thread;

    let mut registry = PluginRegistry::new();
    registry.register("shared", |id, theme| Box::new(AlphaWidget { id, theme }));
    let registry = Arc::new(registry);

    let mut handles = vec![];
    for i in 0..4 {
        let reg = Arc::clone(&registry);
        let handle = thread::spawn(move || {
            reg.create("shared", WidgetId::new(i as usize), Theme::default())
                .is_some()
        });
        handles.push(handle);
    }

    for h in handles {
        assert!(h.join().unwrap());
    }
}

#[test]
fn thread_safe_register_and_unregister() {
    use std::sync::Arc;
    use std::thread;

    let registry = Arc::new(std::sync::Mutex::new(PluginRegistry::new()));
    let mut handles = vec![];

    for i in 0..4 {
        let reg = Arc::clone(&registry);
        let handle = thread::spawn(move || {
            let mut r = reg.lock().unwrap();
            let name = format!("widget_{}", i);
            r.register(&name, |id, theme| Box::new(AlphaWidget { id, theme }));
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let r = registry.lock().unwrap();
    assert_eq!(r.len(), 4);
}

#[test]
fn empty_name_lookup_after_registration() {
    let mut registry = PluginRegistry::new();
    registry.register("", |id, theme| Box::new(AlphaWidget { id, theme }));
    let widget = registry.create("", WidgetId::new(1), Theme::default());
    assert!(widget.is_some());
    assert!(registry.has(""));
}

#[test]
fn widget_id_passthrough() {
    let mut registry = PluginRegistry::new();
    registry.register("passthrough", |id, theme| {
        Box::new(AlphaWidget { id, theme })
    });

    let widget = registry
        .create("passthrough", WidgetId::new(12345), Theme::default())
        .unwrap();
    assert_eq!(widget.id().0, 12345);
}

#[test]
fn register_unicode_name() {
    let mut registry = PluginRegistry::new();
    assert!(
        registry.register("🎨_paint", |id, theme| Box::new(AlphaWidget {
            id,
            theme
        }))
    );
    assert!(
        registry.register("日本語_widget", |id, theme| Box::new(BetaWidget {
            id,
            theme
        }))
    );
    assert!(registry.has("🎨_paint"));
    assert!(registry.has("日本語_widget"));
    assert_eq!(registry.len(), 2);
}

#[test]
fn unregister_does_not_affect_other_widgets() {
    let mut registry = PluginRegistry::new();
    registry.register("keep", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.register("drop", |id, theme| Box::new(BetaWidget { id, theme }));

    let keep_widget = registry
        .create("keep", WidgetId::new(1), Theme::default())
        .unwrap();
    let original_id = keep_widget.id();
    drop(keep_widget);

    registry.unregister("drop");

    let keep_widget_again = registry
        .create("keep", WidgetId::new(1), Theme::default())
        .unwrap();
    assert_eq!(keep_widget_again.id(), original_id);
}

#[test]
fn many_creates_from_one_plugin() {
    let mut registry = PluginRegistry::new();
    registry.register("multi", |id, theme| Box::new(AlphaWidget { id, theme }));

    let mut widgets = vec![];
    for i in 0..50 {
        widgets.push(
            registry
                .create("multi", WidgetId::new(i), Theme::default())
                .unwrap(),
        );
    }
    for (i, w) in widgets.iter().enumerate() {
        assert_eq!(w.id().0, i);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PLUGIN LOAD FAILURE PATHS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn create_unknown_plugin_returns_none_v2() {
    let registry = PluginRegistry::new();
    let result = registry.create("does_not_exist_v2", WidgetId::new(1), Theme::default());
    assert!(result.is_none());
}

#[test]
fn has_returns_false_for_unknown_plugin() {
    let registry = PluginRegistry::new();
    assert!(!registry.has("nope"));
    assert!(registry.is_empty());
}

#[test]
fn unregister_unknown_plugin_returns_false() {
    let mut registry = PluginRegistry::new();
    assert!(!registry.unregister("nope"));
    assert_eq!(registry.len(), 0);
}

#[test]
fn register_same_plugin_twice_overwrites() {
    let mut registry = PluginRegistry::new();
    registry.register("p", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.register("p", |id, theme| Box::new(BetaWidget { id, theme }));

    assert_eq!(
        registry.len(),
        1,
        "Duplicate register should overwrite, not duplicate"
    );
    assert!(registry.has("p"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// PLUGIN UNLOAD LIFECYCLE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn unregister_removes_plugin_from_registry() {
    let mut registry = PluginRegistry::new();
    registry.register("temp", |id, theme| Box::new(AlphaWidget { id, theme }));
    assert!(registry.has("temp"));
    assert_eq!(registry.len(), 1);

    assert!(registry.unregister("temp"));
    assert!(!registry.has("temp"));
    assert!(registry.is_empty());
}

#[test]
fn unregister_one_plugin_keeps_others() {
    let mut registry = PluginRegistry::new();
    registry.register("a", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.register("b", |id, theme| Box::new(BetaWidget { id, theme }));
    registry.register("c", |id, theme| Box::new(AlphaWidget { id, theme }));

    assert!(registry.unregister("b"));
    assert_eq!(registry.len(), 2);
    assert!(registry.has("a"));
    assert!(!registry.has("b"));
    assert!(registry.has("c"));
}

#[test]
fn create_after_unregister_returns_none_v2() {
    let mut registry = PluginRegistry::new();
    registry.register("temp_v2", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.unregister("temp_v2");

    let result = registry.create("temp_v2", WidgetId::new(1), Theme::default());
    assert!(result.is_none());
}

#[test]
fn [DRACON_SECRET:YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBLSi91MVllWTJuQWdBczB1dzZkY1d4cktGS1FPMXUvbjlRQjNSRTh3dEVZCjd5Ylk1V2ttb1JleFkzNXpuTE5oenI2Wm4zL3VEVm4rUGlaWjJtdjViTlEKLT4gXjFILWdyZWFzZSAqRWouKXlFNyA1eWMgelE5XjU6ejwKZEpEcWhCcEJYYzNZblBsK09rejl6Ky9FdCtWVFI2WHkyTi82YzVmM0ZiWWJ1RllvQUZzZGtHVm1IaGphaVhiVgoyT2czTlJtS3ZDQjBtLzBpQ3lGUk9VOG1FZ0g5WFEKLS0tIHJ2dWxqM0Rza0w4SnpYL3hVbTJiQlgxd0tKdnM5dHVNSWJxempvWGtMeFUKyOHCjUCdiqmu8lCSf6gzjRUH97LXBUwFcXkB0+BnsKmLEZ+IDYQ11M/LHZknixAtbYLyDmedGoeLyYjP8EAckgER]() {
    let mut registry = PluginRegistry::new();
    registry.register("p", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.unregister("p");
    registry.register("p", |id, theme| Box::new(BetaWidget { id, theme }));

    let widget = registry
        .create("p", WidgetId::new(1), Theme::default())
        .unwrap();
    // BetaWidget has area 30x15 (different from AlphaWidget 20x10)
    assert_eq!(widget.area().width, 30);
    assert_eq!(widget.area().height, 15);
}

// ═══════════════════════════════════════════════════════════════════════════════
// PLUGIN DEPENDENCY / INTERACTION PATTERNS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn list_returns_all_registered_plugin_names() {
    let mut registry = PluginRegistry::new();
    registry.register("alpha", |id, theme| Box::new(AlphaWidget { id, theme }));
    registry.register("beta", |id, theme| Box::new(BetaWidget { id, theme }));
    registry.register("gamma", |id, theme| Box::new(AlphaWidget { id, theme }));

    let names = registry.list();
    assert_eq!(names.len(), 3);
    assert!(names.contains(&"alpha".to_string()));
    assert!(names.contains(&"beta".to_string()));
    assert!(names.contains(&"gamma".to_string()));
}

#[test]
fn multiple_widgets_from_same_plugin_get_unique_ids() {
    let mut registry = PluginRegistry::new();
    registry.register("p", |id, theme| Box::new(AlphaWidget { id, theme }));

    let w1 = registry
        .create("p", WidgetId::new(100), Theme::default())
        .unwrap();
    let w2 = registry
        .create("p", WidgetId::new(200), Theme::default())
        .unwrap();
    let w3 = registry
        .create("p", WidgetId::new(300), Theme::default())
        .unwrap();

    assert_eq!(w1.id().0, 100);
    assert_eq!(w2.id().0, 200);
    assert_eq!(w3.id().0, 300);
}

#[test]
fn unregister_does_not_invalidate_already_created_widgets() {
    let mut registry = PluginRegistry::new();
    registry.register("p", |id, theme| Box::new(AlphaWidget { id, theme }));

    let widget = registry
        .create("p", WidgetId::new(42), Theme::default())
        .unwrap();
    let original_id = widget.id();

    // Unregister the factory — existing widget instances should still be valid
    registry.unregister("p");
    assert_eq!(widget.id(), original_id);
    // And still renderable
    let plane = widget.render(Rect::new(0, 0, 10, 5));
    assert_eq!(plane.width, 10);
}

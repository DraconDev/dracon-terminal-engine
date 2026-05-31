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

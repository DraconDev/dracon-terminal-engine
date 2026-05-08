#![allow(missing_docs)]
//! Plugin Registry — Dynamic widget loading system.
//!
//! Allows third-party widgets to be registered and instantiated by name,
//! without modifying the framework core.
//!
//! # Example
//! ```rust,no_run
//! use dracon_terminal_engine::framework::prelude::*;
//! use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
//! use ratatui::layout::Rect;
//!
//! // Define a custom widget
//! struct MyWidget { id: WidgetId }
//! impl Widget for MyWidget {
//!     fn id(&self) -> WidgetId { self.id }
//!     fn set_id(&mut self, id: WidgetId) { self.id = id; }
//!     fn area(&self) -> Rect { Rect::default() }
//!     fn set_area(&mut self, _area: Rect) {}
//!     fn render(&self, area: Rect) -> Plane { Plane::new(0, area.width, area.height) }
//! }
//!
//! // Register it
//! let mut registry = PluginRegistry::new();
//! registry.register("my_widget", |id, _theme| Box::new(MyWidget { id }));
//!
//! // Create instance by name
//! if let Some(widget) = registry.create("my_widget", WidgetId::new(1), Theme::default()) {
//!     // Use the widget...
//! }
//! ```

use crate::framework::theme::Theme;
use crate::framework::widget::{Widget, WidgetId};
use std::collections::HashMap;

/// Type alias for widget factory functions.
///
/// Takes a `WidgetId` and `Theme`, returns a boxed `Widget`.
pub type WidgetFactory = Box<dyn Fn(WidgetId, Theme) -> Box<dyn Widget> + Send + Sync>;

/// Registry for dynamically-loadable widgets.
///
/// Third-party widgets register themselves by name, and can then be
/// instantiated on demand without compile-time dependencies.
pub struct PluginRegistry {
    widgets: HashMap<String, WidgetFactory>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
        }
    }

    /// Registers a widget factory under a unique name.
    ///
    /// Returns `true` if the registration was successful (name was not already in use).
    /// Returns `false` if the name was already registered.
    pub fn register(&mut self, name: &str, factory: impl Fn(WidgetId, Theme) -> Box<dyn Widget> + Send + Sync + 'static) -> bool {
        if self.widgets.contains_key(name) {
            return false;
        }
        self.widgets.insert(name.to_string(), Box::new(factory));
        true
    }

    /// Creates a widget instance by name.
    ///
    /// Returns `Some(Box<dyn Widget>)` if the name is registered,
    /// `None` otherwise.
    pub fn create(&self, name: &str, id: WidgetId, theme: Theme) -> Option<Box<dyn Widget>> {
        self.widgets.get(name).map(|factory| factory(id, theme))
    }

    /// Checks if a widget name is registered.
    pub fn has(&self, name: &str) -> bool {
        self.widgets.contains_key(name)
    }

    /// Unregisters a widget by name.
    ///
    /// Returns `true` if the widget was removed, `false` if not found.
    pub fn unregister(&mut self, name: &str) -> bool {
        self.widgets.remove(name).is_some()
    }

    /// Returns a list of all registered widget names.
    pub fn list(&self) -> Vec<String> {
        self.widgets.keys().cloned().collect()
    }

    /// Returns the number of registered widgets.
    pub fn len(&self) -> usize {
        self.widgets.len()
    }

    /// Returns true if no widgets are registered.
    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::{Cell, Color};
    use crate::framework::widget::Widget;
    use ratatui::layout::Rect;

    struct TestWidget {
        id: WidgetId,
        theme: Theme,
    }

    impl Widget for TestWidget {
        fn id(&self) -> WidgetId {
            self.id
        }
        fn set_id(&mut self, id: WidgetId) {
            self.id = id;
        }
        fn area(&self) -> Rect {
            Rect::new(0, 0, 10, 5)
        }
        fn set_area(&mut self, area: Rect) {
            let _ = area;
        }
        fn render(&self, area: Rect) -> Plane {
            let mut plane = Plane::new(0, area.width, area.height);
            for cell in plane.cells.iter_mut() {
                cell.fg = self.theme.primary;
            }
            plane
        }
    }

    #[test]
    fn test_register_and_create() {
        let mut registry = PluginRegistry::new();
        assert!(registry.register("test", |id, theme| Box::new(TestWidget { id, theme })));
        assert!(registry.has("test"));
        assert_eq!(registry.len(), 1);

        let widget = registry.create("test", WidgetId::new(1), Theme::default());
        assert!(widget.is_some());
        let w = widget.unwrap();
        assert_eq!(w.id().0, 1);
    }

    #[test]
    fn test_create_unknown_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.create("unknown", WidgetId::new(1), Theme::default()).is_none());
    }

    #[test]
    fn test_duplicate_registration_fails() {
        let mut registry = PluginRegistry::new();
        assert!(registry.register("dup", |id, theme| Box::new(TestWidget { id, theme })));
        assert!(!registry.register("dup", |id, theme| Box::new(TestWidget { id, theme })));
    }

    #[test]
    fn test_unregister() {
        let mut registry = PluginRegistry::new();
        registry.register("temp", |id, theme| Box::new(TestWidget { id, theme }));
        assert!(registry.unregister("temp"));
        assert!(!registry.has("temp"));
        assert!(!registry.unregister("temp"));
    }

    #[test]
    fn test_list() {
        let mut registry = PluginRegistry::new();
        registry.register("a", |id, theme| Box::new(TestWidget { id, theme }));
        registry.register("b", |id, theme| Box::new(TestWidget { id, theme }));
        let list = registry.list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"a".to_string()));
        assert!(list.contains(&"b".to_string()));
    }

    #[test]
    fn test_empty() {
        let registry = PluginRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }
}

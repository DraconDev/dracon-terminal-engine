//! Widget container for owning and managing a widget.
//!
//! Provides `WidgetContainer` which wraps a `Box<dyn Widget>` and
//! delegates `id()`, `render()`, `handle_key()`, and `handle_mouse()`.

use crate::framework::widget::{Widget, WidgetId};
use ratatui::layout::Rect;

pub struct WidgetContainer {
    inner: Box<dyn Widget>,
}

impl WidgetContainer {
    pub fn new(widget: Box<dyn Widget>) -> Self {
        Self { inner: widget }
    }

    pub fn id(&self) -> WidgetId {
        self.inner.id()
    }

    pub fn render(&self, area: Rect) -> crate::compositor::Plane {
        self.inner.render(area)
    }

    pub fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        self.inner.handle_key(key)
    }

    pub fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, col: u16, row: u16) -> bool {
        self.inner.handle_mouse(kind, col, row)
    }

    pub fn widget(&self) -> &dyn Widget {
        &*self.inner
    }

    pub fn widget_mut(&mut self) -> &mut dyn Widget {
        &mut *self.inner
    }
}

pub struct WidgetRegistry {
    containers: Vec<WidgetContainer>,
    next_id: usize,
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WidgetRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            containers: Vec::new(),
            next_id: 1,
        }
    }

    /// Registers a widget by moving it into a container.
    pub fn register(&mut self, widget: Box<dyn Widget>) -> WidgetId {
        let id = widget.id();
        self.containers.push(WidgetContainer::new(widget));
        id
    }

    pub fn unregister(&mut self, id: WidgetId) {
        self.containers.retain(|c| c.id() != id);
    }

    pub fn get(&self, id: WidgetId) -> Option<&WidgetContainer> {
        self.containers.iter().find(|c| c.id() == id)
    }

    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut WidgetContainer> {
        self.containers.iter_mut().find(|c| c.id() == id)
    }

    pub fn next_id(&mut self) -> WidgetId {
        let id = WidgetId::new(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn iter(&self) -> std::slice::Iter<'_, WidgetContainer> {
        self.containers.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, WidgetContainer> {
        self.containers.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::Plane;

    struct DummyWidget {
        id: WidgetId,
    }

    impl Widget for DummyWidget {
        fn id(&self) -> WidgetId {
            self.id
        }

        fn render(&self, _area: Rect) -> Plane {
            Plane::new(0, 1, 1)
        }
    }

    #[test]
    fn test_container_creates_and_delegates() {
        let widget = Box::new(DummyWidget { id: WidgetId::new(1) });
        let container = WidgetContainer::new(widget);
        assert_eq!(container.id(), WidgetId::new(1));
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = WidgetRegistry::new();
        let id = WidgetId::new(1);
        let widget = Box::new(DummyWidget { id });
        registry.register(widget);

        assert!(registry.get(id).is_some());
    }

    #[test]
    fn test_registry_unregister() {
        let mut registry = WidgetRegistry::new();
        let id = WidgetId::new(2);
        let widget = Box::new(DummyWidget { id });
        registry.register(widget);
        registry.unregister(id);

        assert!(registry.get(id).is_none());
    }
}
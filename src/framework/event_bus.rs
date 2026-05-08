//! Event Bus — Decoupled inter-widget communication.
//!
//! Provides a publish/subscribe mechanism so widgets can communicate
//! without direct references. This is the foundation of app architecture
//! in the Dracon framework.
//!
//! ## Design
//!
//! - **Type-safe**: Events are Rust enums, not strings
//! - **Scoped**: Widgets subscribe to specific event types
//! - **Synchronous**: Callbacks run immediately on publish (predictable)
//! - **Centralized**: Single bus per app, accessible via `Ctx`
//!
//! ## Example
//!
//! ```no_run
//! use dracon_terminal_engine::framework::event_bus::EventBus;
//!
//! // Define app events
//! #[derive(Clone, Debug)]
//! enum AppEvent {
//!     UserSelected(String),
//! }
//!
//! let event_bus = EventBus::new();
//!
//! // Subscribe to events
//! event_bus.subscribe(|event: &AppEvent| {
//!     if let AppEvent::UserSelected(name) = event {
//!         println!("Selected: {}", name);
//!     }
//! });
//!
//! // Publish an event
//! event_bus.publish(AppEvent::UserSelected("alice".into()));
//! ```

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

/// A recorded event with metadata for debugging.
#[derive(Clone)]
pub struct EventRecord {
    /// When the event was published.
    pub timestamp: Instant,
    /// The type name of the event.
    pub type_name: String,
    /// The event payload (type-erased).
    pub payload: Rc<dyn Any>,
}

impl std::fmt::Debug for EventRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventRecord")
            .field("timestamp", &self.timestamp.elapsed().as_millis())
            .field("type_name", &self.type_name)
            .field("payload", &"<dyn Any>")
            .finish()
    }
}

/// A type-erased callback that can handle any event type.
type EventCallback = Rc<dyn Fn(&dyn Any) + 'static>;

/// Internal storage for callbacks keyed by event type.
#[derive(Default)]
pub struct EventBus {
    /// Map from TypeId to list of callbacks for that event type.
    subscribers: RefCell<HashMap<TypeId, Vec<EventCallback>>>,
    /// Optional trace logging for debugging.
    trace: RefCell<bool>,
    /// Recent event history for debugging.
    history: RefCell<Vec<EventRecord>>,
    /// Maximum history size (0 = unlimited).
    max_history: RefCell<usize>,
}

impl EventBus {
    /// Creates a new empty event bus.
    pub fn new() -> Self {
        Self {
            subscribers: RefCell::new(HashMap::new()),
            trace: RefCell::new(false),
            history: RefCell::new(Vec::new()),
            max_history: RefCell::new(100),
        }
    }

    /// Creates a new event bus with tracing enabled.
    pub fn with_trace() -> Self {
        Self {
            subscribers: RefCell::new(HashMap::new()),
            trace: RefCell::new(true),
            history: RefCell::new(Vec::new()),
            max_history: RefCell::new(100),
        }
    }

    /// Sets the maximum history size (0 = unlimited).
    pub fn set_history_capacity(&self, capacity: usize) {
        *self.max_history.borrow_mut() = capacity;
        self.trim_history();
    }

    /// Returns a copy of the event history.
    pub fn history(&self) -> Vec<EventRecord> {
        self.history.borrow().clone()
    }

    /// Clears the event history.
    pub fn clear_history(&self) {
        self.history.borrow_mut().clear();
    }

    fn trim_history(&self) {
        let max = *self.max_history.borrow();
        if max > 0 {
            let mut history = self.history.borrow_mut();
            while history.len() > max {
                history.remove(0);
            }
        }
    }

    fn record_event<E: Any + Clone>(&self, event: &E) {
        let max = *self.max_history.borrow();
        if max == 0 {
            return;
        }
        let record = EventRecord {
            timestamp: Instant::now(),
            type_name: std::any::type_name::<E>().to_string(),
            payload: Rc::new(event.clone()),
        };
        self.history.borrow_mut().push(record);
        self.trim_history();
    }

    /// Publishes an event to all subscribers of that type.
    ///
    /// Subscribers are called in the order they were registered.
    /// If tracing is enabled, prints the event type and subscriber count.
    pub fn publish<E: Any + Clone>(&self, event: E) {
        let type_id = TypeId::of::<E>();

        // Record in history
        self.record_event(&event);

        if *self.trace.borrow() {
            let count = self
                .subscribers
                .borrow()
                .get(&type_id)
                .map(|v| v.len())
                .unwrap_or(0);
            eprintln!("[EventBus] publish<{}> → {} subscribers", std::any::type_name::<E>(), count);
        }

        // Clone subscribers to avoid borrow issues during callback
        let callbacks: Vec<EventCallback> = self
            .subscribers
            .borrow()
            .get(&type_id)
            .cloned()
            .unwrap_or_default();

        for callback in callbacks {
            callback(&event);
        }
    }

    /// Subscribes to events of type `E`.
    ///
    /// The callback receives a reference to the event. Use `clone()` if you
    /// need to store it.
    ///
    /// Returns a subscription ID that can be used to unsubscribe.
    pub fn subscribe<E: Any + Clone, F>(&self, callback: F) -> SubscriptionId
    where
        F: Fn(&E) + 'static,
    {
        let type_id = TypeId::of::<E>();
        let wrapped: EventCallback = Rc::new(move |any_event| {
            if let Some(event) = any_event.downcast_ref::<E>() {
                callback(event);
            }
        });

        let mut subs = self.subscribers.borrow_mut();
        let list = subs.entry(type_id).or_default();
        let id = SubscriptionId(list.len());
        list.push(wrapped);

        if *self.trace.borrow() {
            eprintln!(
                "[EventBus] subscribe<{}> → id={}",
                std::any::type_name::<E>(),
                id.0
            );
        }

        id
    }

    /// Unsubscribes a callback by ID.
    ///
    /// Note: This removes by index, so IDs may shift after removals.
    pub fn unsubscribe<E: Any>(&self, id: SubscriptionId) {
        let type_id = TypeId::of::<E>();
        let mut subs = self.subscribers.borrow_mut();
        if let Some(list) = subs.get_mut(&type_id) {
            if id.0 < list.len() {
                list.remove(id.0);
            }
        }
    }

    /// Returns the number of subscribers for a given event type.
    pub fn subscriber_count<E: Any>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        self.subscribers
            .borrow()
            .get(&type_id)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Clears all subscribers for a given event type.
    pub fn clear<E: Any>(&self) {
        let type_id = TypeId::of::<E>();
        self.subscribers.borrow_mut().remove(&type_id);
    }

    /// Clears all subscribers for all event types.
    pub fn clear_all(&self) {
        self.subscribers.borrow_mut().clear();
    }

    /// Enables or disables trace logging.
    pub fn set_trace(&self, enabled: bool) {
        *self.trace.borrow_mut() = enabled;
    }

    /// Replays the last N events from history (without re-recording them).
    pub fn replay_last(&self, n: usize) {
        let history = self.history.borrow();
        let start = history.len().saturating_sub(n);
        for record in &history[start..] {
            // Re-dispatch to current subscribers by type
            if let Some(callbacks) = self.subscribers.borrow().get(&record.payload.type_id()) {
                let callbacks: Vec<EventCallback> = callbacks.clone();
                for cb in callbacks {
                    cb(&*record.payload);
                }
            }
        }
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            subscribers: RefCell::new(HashMap::new()),
            trace: RefCell::new(*self.trace.borrow()),
            history: RefCell::new(self.history.borrow().clone()),
            max_history: RefCell::new(*self.max_history.borrow()),
        }
    }
}

/// Handle to a subscription. Use with `unsubscribe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub usize);

// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET TRAIT EXTENSION
// ═══════════════════════════════════════════════════════════════════════════════

use crate::framework::widget::Widget;

/// Extension trait for widgets that need event bus access.
///
/// Implement this on your widget to receive lifecycle hooks for
/// subscription management.
pub trait EventBusWidget: Widget {
    /// Called when the widget is mounted. Subscribe to events here.
    fn on_mount_bus(&mut self, _bus: &EventBus) {}

    /// Called when the widget is unmounted. Unsubscribe here.
    fn on_unmount_bus(&mut self, _bus: &EventBus) {}
}

// ═══════════════════════════════════════════════════════════════════════════════
// STATE MANAGEMENT HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

use std::sync::atomic::{AtomicUsize, Ordering};

/// A simple reactive value that publishes changes.
///
/// Use this for app state that multiple widgets need to observe.
pub struct Reactive<T: Clone + 'static> {
    value: RefCell<T>,
    bus: EventBus,
    change_count: AtomicUsize,
}

/// Event fired when a reactive value changes.
#[derive(Clone, Debug)]
pub struct ValueChanged<T: Clone> {
    pub old: Option<T>,
    pub new: T,
    pub count: usize,
}

impl<T: Clone + 'static + PartialEq> Reactive<T> {
    /// Creates a new reactive value.
    pub fn new(initial: T) -> Self {
        Self {
            value: RefCell::new(initial),
            bus: EventBus::new(),
            change_count: AtomicUsize::new(0),
        }
    }

    /// Gets the current value.
    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }

    /// Sets a new value and publishes a change event.
    pub fn set(&self, new_value: T) {
        let old = self.value.borrow().clone();
        if old == new_value {
            return;
        }
        let count = self.change_count.fetch_add(1, Ordering::SeqCst);
        *self.value.borrow_mut() = new_value.clone();
        self.bus.publish(ValueChanged {
            old: Some(old),
            new: new_value,
            count,
        });
    }

    /// Subscribes to value changes.
    pub fn on_change<F>(&self, callback: F) -> SubscriptionId
    where
        F: Fn(&ValueChanged<T>) + 'static,
    {
        self.bus.subscribe(callback)
    }

    /// Maps the value and returns a new reactive.
    pub fn map<U: Clone + 'static + PartialEq, F>(&self, f: F) -> Reactive<U>
    where
        F: Fn(&T) -> U + 'static,
    {
        let mapped = Reactive::new(f(&self.get()));
        let mapped_clone = mapped.clone();
        self.on_change(move |change| {
            mapped_clone.set(f(&change.new));
        });
        mapped
    }
}

impl<T: Clone + 'static + PartialEq> Clone for Reactive<T> {
    fn clone(&self) -> Self {
        Self {
            value: RefCell::new(self.get()),
            bus: self.bus.clone(),
            change_count: AtomicUsize::new(self.change_count.load(Ordering::SeqCst)),
        }
    }
}

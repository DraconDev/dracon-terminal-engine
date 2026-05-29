//! Event Bus tests

use dracon_terminal_engine::framework::event_bus::{
    EventBus, Reactive, SubscriptionId, ValueChanged,
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
struct TestEvent {
    pub value: i32,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
struct OtherEvent {
    pub flag: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BASIC PUBLISH/SUBSCRIBE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_event_bus_publish_and_receive() {
    let bus = EventBus::new();
    let received = Rc::new(RefCell::new(None));
    let received_clone = Rc::clone(&received);

    bus.subscribe(move |event: &TestEvent| {
        *received_clone.borrow_mut() = Some(event.clone());
    });

    bus.publish(TestEvent {
        value: 42,
        message: "hello".into(),
    });

    assert_eq!(
        received.borrow().clone(),
        Some(TestEvent {
            value: 42,
            message: "hello".into(),
        })
    );
}

#[test]
fn test_event_bus_multiple_subscribers() {
    let bus = EventBus::new();
    let count1 = Rc::new(RefCell::new(0));
    let count2 = Rc::new(RefCell::new(0));
    let c1 = Rc::clone(&count1);
    let c2 = Rc::clone(&count2);

    bus.subscribe(move |_: &TestEvent| {
        *c1.borrow_mut() += 1;
    });
    bus.subscribe(move |_: &TestEvent| {
        *c2.borrow_mut() += 1;
    });

    bus.publish(TestEvent {
        value: 1,
        message: "a".into(),
    });
    bus.publish(TestEvent {
        value: 2,
        message: "b".into(),
    });

    assert_eq!(*count1.borrow(), 2);
    assert_eq!(*count2.borrow(), 2);
}

#[test]
fn test_event_bus_type_isolation() {
    let bus = EventBus::new();
    let test_received = Rc::new(RefCell::new(false));
    let other_received = Rc::new(RefCell::new(false));
    let t = Rc::clone(&test_received);
    let o = Rc::clone(&other_received);

    bus.subscribe(move |_: &TestEvent| {
        *t.borrow_mut() = true;
    });
    bus.subscribe(move |_: &OtherEvent| {
        *o.borrow_mut() = true;
    });

    // Publish only TestEvent
    bus.publish(TestEvent {
        value: 1,
        message: "test".into(),
    });

    assert!(*test_received.borrow());
    assert!(!*other_received.borrow()); // OtherEvent subscriber should NOT fire
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNSUBSCRIBE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_event_bus_unsubscribe() {
    let bus = EventBus::new();
    let count = Rc::new(RefCell::new(0));
    let c = Rc::clone(&count);

    let id = bus.subscribe(move |_: &TestEvent| {
        *c.borrow_mut() += 1;
    });

    bus.publish(TestEvent {
        value: 1,
        message: "first".into(),
    });
    assert_eq!(*count.borrow(), 1);

    bus.unsubscribe::<TestEvent>(id);

    bus.publish(TestEvent {
        value: 2,
        message: "second".into(),
    });
    assert_eq!(*count.borrow(), 1); // Should still be 1
}

// ═══════════════════════════════════════════════════════════════════════════════
// REACTIVE STATE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_reactive_value() {
    let rx = Reactive::new(10);
    let received = Rc::new(RefCell::new(Vec::new()));
    let r = Rc::clone(&received);

    rx.on_change(move |change: &ValueChanged<i32>| {
        r.borrow_mut().push(change.new);
    });

    rx.set(20);
    rx.set(30);

    assert_eq!(*received.borrow(), vec![20, 30]);
}

#[test]
fn test_reactive_no_notification_on_same_value() {
    let rx = Reactive::new(5);
    let count = Rc::new(RefCell::new(0));
    let c = Rc::clone(&count);

    rx.on_change(move |_: &ValueChanged<i32>| {
        *c.borrow_mut() += 1;
    });

    rx.set(5); // Same value
    assert_eq!(*count.borrow(), 0);

    rx.set(10); // Different value
    assert_eq!(*count.borrow(), 1);
}

#[test]
fn test_reactive_get() {
    let rx = Reactive::new(42);
    assert_eq!(rx.get(), 42);

    rx.set(100);
    assert_eq!(rx.get(), 100);
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDGE CASES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_event_bus_no_subscribers_no_panic() {
    let bus = EventBus::new();
    // Should not panic even with no subscribers
    bus.publish(TestEvent {
        value: 1,
        message: "lonely".into(),
    });
}

#[test]
fn test_event_bus_subscriber_count() {
    let bus = EventBus::new();
    assert_eq!(bus.subscriber_count::<TestEvent>(), 0);

    let _id1 = bus.subscribe(|_: &TestEvent| {});
    let _id2 = bus.subscribe(|_: &TestEvent| {});
    assert_eq!(bus.subscriber_count::<TestEvent>(), 2);

    // Note: unsubscribe removes by index, so IDs shift after removal.
    // Unsubscribing in reverse order (newest first) works correctly.
    bus.unsubscribe::<TestEvent>(SubscriptionId(1));
    assert_eq!(bus.subscriber_count::<TestEvent>(), 1);

    bus.unsubscribe::<TestEvent>(SubscriptionId(0));
    assert_eq!(bus.subscriber_count::<TestEvent>(), 0);
}

#[test]
fn test_reactive_multiple_subscribers() {
    let rx = Reactive::new("hello");
    let vals1 = Rc::new(RefCell::new(Vec::new()));
    let vals2 = Rc::new(RefCell::new(Vec::new()));
    let v1 = Rc::clone(&vals1);
    let v2 = Rc::clone(&vals2);

    rx.on_change(move |change: &ValueChanged<&str>| {
        v1.borrow_mut().push(change.new.to_string());
    });
    rx.on_change(move |change: &ValueChanged<&str>| {
        v2.borrow_mut().push(change.new.to_string());
    });

    rx.set("world");

    assert_eq!(*vals1.borrow(), vec!["world"]);
    assert_eq!(*vals2.borrow(), vec!["world"]);
}

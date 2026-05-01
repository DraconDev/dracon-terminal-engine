//! Tests for HitZone system.

mod common;
use common::make_area;

use dracon_terminal_engine::framework::hitzone::{
    ClickKind, DragState, HitZone, HitZoneGroup, ScopedZone, ScopedZoneRegistry,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_hit_zone_new() {
    let zone = HitZone::new(42u32, 5, 10, 20, 5);
    assert_eq!(zone.id, 42);
    assert_eq!(zone.x, 5);
    assert_eq!(zone.y, 10);
    assert_eq!(zone.width, 20);
    assert_eq!(zone.height, 5);
}

#[test]
fn test_hit_zone_contains_inside() {
    let zone = HitZone::new(1u32, 5, 10, 20, 5);
    assert!(zone.contains(6, 10));
    assert!(zone.contains(24, 14));
    assert!(zone.contains(5, 10));
}

#[test]
fn test_hit_zone_contains_outside() {
    let zone = HitZone::new(1u32, 5, 10, 20, 5);
    assert!(!zone.contains(4, 10));
    assert!(!zone.contains(25, 10));
    assert!(!zone.contains(5, 9));
    assert!(!zone.contains(5, 15));
}

#[test]
fn test_hit_zone_contains_on_edge() {
    let zone = HitZone::new(1u32, 5, 10, 20, 5);
    assert!(zone.contains(24, 10));
    assert!(zone.contains(5, 14));
    assert!(!zone.contains(25, 10));
    assert!(!zone.contains(5, 15));
}

#[test]
fn test_hit_zone_on_click_callback_single() {
    let kind = Rc::new(Cell::new(None));
    let kind_clone = kind.clone();
    {
        let mut zone = HitZone::new(1u32, 5, 10, 20, 5).on_click(move |k| {
            kind_clone.set(Some(k));
        });
        zone.handle_mouse(
            MouseEventKind::Down(MouseButton::Left),
            10,
            12,
            KeyModifiers::empty(),
        );
    }
    assert_eq!(kind.get(), Some(ClickKind::Single));
}

#[test]
fn test_hit_zone_on_right_click() {
    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();
    {
        let mut zone = HitZone::new(1u32, 5, 10, 20, 5).on_right_click(move || {
            called_clone.set(true);
        });
        zone.handle_mouse(
            MouseEventKind::Down(MouseButton::Right),
            10,
            12,
            KeyModifiers::empty(),
        );
    }
    assert!(called.get());
}

#[test]
fn test_hit_zone_dispatch_mouse_returns_id() {
    let kind = Rc::new(Cell::new(None));
    let kind_clone = kind.clone();
    let mut zone = HitZone::new(99u32, 5, 10, 20, 5).on_click(move |k| {
        kind_clone.set(Some(k));
    });
    let result = zone.dispatch_mouse(
        MouseEventKind::Down(MouseButton::Left),
        10,
        12,
        KeyModifiers::empty(),
    );
    assert_eq!(result, Some(99));
}

#[test]
fn test_hit_zone_dispatch_mouse_outside_returns_none() {
    let kind = Rc::new(Cell::new(None));
    let kind_clone = kind.clone();
    let mut zone = HitZone::new(99u32, 5, 10, 20, 5).on_click(move |k| {
        kind_clone.set(Some(k));
    });
    let result = zone.dispatch_mouse(
        MouseEventKind::Down(MouseButton::Left),
        100,
        100,
        KeyModifiers::empty(),
    );
    assert_eq!(result, None);
}

#[test]
fn test_hit_zone_drag_start() {
    let started = Rc::new(Cell::new(false));
    let started_clone = started.clone();
    {
        let mut zone = HitZone::new(1u32, 5, 10, 20, 5).on_drag_start(move |state| {
            if let DragState::Started { .. } = state {
                started_clone.set(true);
            }
        });
        zone.handle_mouse(
            MouseEventKind::Down(MouseButton::Left),
            10,
            12,
            KeyModifiers::empty(),
        );
    }
    assert!(started.get());
}

#[test]
fn test_hit_zone_drag_move() {
    let positions = Rc::new(Cell::new(0));
    let positions_clone = positions.clone();
    {
        let mut zone = HitZone::new(1u32, 5, 10, 20, 5)
            .on_drag_start(|_| {})
            .on_drag_move(move |state| {
                if let DragState::Moved { .. } = state {
                    positions_clone.set(positions_clone.get() + 1);
                }
            });
        zone.handle_mouse(
            MouseEventKind::Down(MouseButton::Left),
            10,
            12,
            KeyModifiers::empty(),
        );
        zone.handle_mouse(
            MouseEventKind::Drag(MouseButton::Left),
            15,
            13,
            KeyModifiers::empty(),
        );
        zone.handle_mouse(
            MouseEventKind::Drag(MouseButton::Left),
            20,
            14,
            KeyModifiers::empty(),
        );
    }
    assert_eq!(positions.get(), 2);
}

#[test]
fn test_hit_zone_drag_end() {
    let ended = Rc::new(Cell::new(false));
    let ended_clone = ended.clone();
    {
        let mut zone = HitZone::new(1u32, 5, 10, 20, 5)
            .on_drag_start(|_| {})
            .on_drag_end(move |state| {
                if let DragState::Ended { .. } = state {
                    ended_clone.set(true);
                }
            });
        zone.handle_mouse(
            MouseEventKind::Down(MouseButton::Left),
            10,
            12,
            KeyModifiers::empty(),
        );
        zone.handle_mouse(
            MouseEventKind::Drag(MouseButton::Left),
            15,
            13,
            KeyModifiers::empty(),
        );
        zone.handle_mouse(
            MouseEventKind::Up(MouseButton::Left),
            20,
            14,
            KeyModifiers::empty(),
        );
    }
    assert!(ended.get());
}

#[test]
fn test_hit_zone_double_click_detection() {
    let click_kinds = Rc::new(Cell::new(0));
    let click_kinds_clone = click_kinds.clone();
    {
        let mut zone = HitZone::new(1u32, 5, 10, 20, 5).on_click(move |_| {
            click_kinds_clone.set(click_kinds_clone.get() + 1);
        });
        zone.handle_mouse(
            MouseEventKind::Down(MouseButton::Left),
            10,
            12,
            KeyModifiers::empty(),
        );
        zone.handle_mouse(
            MouseEventKind::Down(MouseButton::Left),
            10,
            12,
            KeyModifiers::empty(),
        );
    }
    assert_eq!(click_kinds.get(), 2);
}

#[test]
fn test_hit_zone_group_new() {
    let group: HitZoneGroup<u32> = HitZoneGroup::new();
    assert!(group.zones().is_empty());
}

#[test]
fn test_hit_zone_group_add_zone() {
    let kind = Rc::new(Cell::new(None));
    let kind_clone = kind.clone();
    let mut group = HitZoneGroup::new();
    let zone = HitZone::new(1u32, 5, 10, 20, 5).on_click(move |k| {
        kind_clone.set(Some(k));
    });
    group.zones_mut().push(zone);
    assert_eq!(group.zones().len(), 1);
}

#[test]
fn test_hit_zone_group_builder() {
    let kind = Rc::new(Cell::new(None));
    let kind_clone = kind.clone();
    let group = HitZoneGroup::new()
        .zone(HitZone::new(1u32, 5, 10, 20, 5).on_click(move |k| {
            kind_clone.set(Some(k));
        }))
        .zone(HitZone::new(2u32, 30, 10, 20, 5).on_click(|_| {}));
    assert_eq!(group.zones().len(), 2);
}

#[test]
fn test_hit_zone_group_dispatch_finds_first() {
    let kind = Rc::new(Cell::new(None));
    let kind_clone = kind.clone();
    let mut group = HitZoneGroup::new()
        .zone(HitZone::new(1u32, 5, 10, 20, 5).on_click(move |k| {
            kind_clone.set(Some(k));
        }))
        .zone(HitZone::new(2u32, 30, 10, 20, 5).on_click(|_| {}));
    let result = group.dispatch_mouse(
        MouseEventKind::Down(MouseButton::Left),
        35,
        12,
        KeyModifiers::empty(),
    );
    assert_eq!(result, Some(2));
}

#[test]
fn test_hit_zone_group_dispatch_miss() {
    let kind = Rc::new(Cell::new(None));
    let kind_clone = kind.clone();
    let mut group = HitZoneGroup::new()
        .zone(HitZone::new(1u32, 5, 10, 20, 5).on_click(move |k| {
            kind_clone.set(Some(k));
        }))
        .zone(HitZone::new(2u32, 30, 10, 20, 5).on_click(|_| {}));
    let result = group.dispatch_mouse(
        MouseEventKind::Down(MouseButton::Left),
        100,
        100,
        KeyModifiers::empty(),
    );
    assert_eq!(result, None);
}

#[test]
fn test_hit_zone_group_add_row() {
    let kind = Rc::new(Cell::new(None));
    let kind_clone = kind.clone();
    let mut group = HitZoneGroup::new();
    group.add_row(1u32, 5, 80, move |_| {
        kind_clone.set(Some(ClickKind::Single));
    });
    assert_eq!(group.zones().len(), 1);
    assert_eq!(group.zones()[0].x, 0);
    assert_eq!(group.zones()[0].y, 5);
    assert_eq!(group.zones()[0].width, 80);
    assert_eq!(group.zones()[0].height, 1);
}

#[test]
fn test_scoped_zone_new() {
    let zone = ScopedZone::new("id", 5, 10, 20, 5);
    assert_eq!(zone.id, "id");
    assert_eq!(zone.x, 5);
}

#[test]
fn test_scoped_zone_contains() {
    let zone = ScopedZone::new("id", 5, 10, 20, 5);
    assert!(zone.contains(6, 10));
    assert!(!zone.contains(100, 100));
}

#[test]
fn test_scoped_zone_registry_new() {
    let registry: ScopedZoneRegistry<u32> = ScopedZoneRegistry::new();
    assert!(registry.zones().is_empty());
}

#[test]
fn test_scoped_zone_registry_register() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(1u32, 5, 10, 20, 5);
    assert_eq!(registry.zones().len(), 1);
}

#[test]
fn test_scoped_zone_registry_dispatch() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(42u32, 5, 10, 20, 5);
    let result = registry.dispatch(10, 12);
    assert_eq!(result, Some(42));
}

#[test]
fn test_scoped_zone_registry_dispatch_miss() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(42u32, 5, 10, 20, 5);
    let result = registry.dispatch(100, 100);
    assert_eq!(result, None);
}

#[test]
fn test_scoped_zone_registry_clear() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(42u32, 5, 10, 20, 5);
    registry.clear();
    assert!(registry.zones().is_empty());
}

#[test]
fn test_click_kind_variants() {
    assert_eq!(ClickKind::Single, ClickKind::Single);
    assert_eq!(ClickKind::Double, ClickKind::Double);
    assert_eq!(ClickKind::Triple, ClickKind::Triple);
    assert_ne!(ClickKind::Single, ClickKind::Double);
}

#[test]
fn test_drag_state_variants() {
    let started = DragState::Started { x: 1, y: 2 };
    let moved = DragState::Moved { x: 3, y: 4 };
    let ended = DragState::Ended { x: 5, y: 6 };
    match started {
        DragState::Started { x, y } => {
            assert_eq!(x, 1);
            assert_eq!(y, 2);
        }
        _ => panic!("expected Started"),
    }
    match moved {
        DragState::Moved { x, y } => {
            assert_eq!(x, 3);
            assert_eq!(y, 4);
        }
        _ => panic!("expected Moved"),
    }
    match ended {
        DragState::Ended { x, y } => {
            assert_eq!(x, 5);
            assert_eq!(y, 6);
        }
        _ => panic!("expected Ended"),
    }
}

#[test]
fn test_hit_zone_hit_right_click_does_not_trigger_left() {
    let left_called = Rc::new(Cell::new(false));
    let left_clone = left_called.clone();
    let mut zone = HitZone::new(1u32, 5, 10, 20, 5)
        .on_click(move |_| {
            left_clone.set(true);
        })
        .on_right_click(|| {});
    zone.handle_mouse(
        MouseEventKind::Down(MouseButton::Right),
        10,
        12,
        KeyModifiers::empty(),
    );
    assert!(!left_called.get());
}

#[test]
fn test_hit_zone_ignores_non_left_buttons() {
    let kind = Rc::new(Cell::new(None));
    let kind_clone = kind.clone();
    let mut zone = HitZone::new(1u32, 5, 10, 20, 5).on_click(move |k| {
        kind_clone.set(Some(k));
    });
    zone.handle_mouse(
        MouseEventKind::Down(MouseButton::Middle),
        10,
        12,
        KeyModifiers::empty(),
    );
    assert!(kind.get().is_none());
}

#[test]
fn test_hit_zone_group_zones_mut() {
    let mut group = HitZoneGroup::new();
    group
        .zones_mut()
        .push(HitZone::new(1u32, 0, 0, 10, 1).on_click(|_| {}));
    assert_eq!(group.zones().len(), 1);
}

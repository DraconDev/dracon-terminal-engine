//! Hit zone system tests — ScopedZoneRegistry edge cases.

use dracon_terminal_engine::framework::hitzone::ScopedZoneRegistry;

#[test]
fn test_zone_registry_basic_dispatch() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(1u32, 10, 10, 20, 20);

    assert_eq!(registry.dispatch(15, 15), Some(1));
    assert_eq!(registry.dispatch(10, 10), Some(1)); // Edge
    assert_eq!(registry.dispatch(29, 29), Some(1)); // Edge
}

#[test]
fn test_zone_registry_outside_all_zones() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(1u32, 10, 10, 5, 5);

    assert_eq!(registry.dispatch(0, 0), None);
    assert_eq!(registry.dispatch(100, 100), None);
    assert_eq!(registry.dispatch(9, 15), None); // Just outside
}

#[test]
fn test_zone_registry_zero_width_zone() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(1u32, 10, 10, 0, 10);

    // Zero-width zone should never match
    assert_eq!(registry.dispatch(10, 10), None);
    assert_eq!(registry.dispatch(9, 10), None);
}

#[test]
fn test_zone_registry_zero_height_zone() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(1u32, 10, 10, 10, 0);

    // Zero-height zone should never match
    assert_eq!(registry.dispatch(10, 10), None);
    assert_eq!(registry.dispatch(10, 9), None);
}

#[test]
fn test_zone_registry_multiple_zones_first_match_wins() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(1u32, 10, 10, 20, 20);
    registry.register(2u32, 15, 15, 20, 20); // Overlaps with zone 1

    // Should return first registered zone (1) in overlap area
    assert_eq!(registry.dispatch(17, 17), Some(1));

    // Should return zone 2 in non-overlapping area
    assert_eq!(registry.dispatch(30, 30), Some(2));
}

#[test]
fn test_zone_registry_clear_removes_all() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(1u32, 10, 10, 20, 20);
    registry.clear();

    assert_eq!(registry.dispatch(15, 15), None);
    assert!(registry.zones().is_empty());
}

#[test]
fn test_zone_registry_coordinate_boundaries() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(1u32, 0, 0, 1, 1);

    assert_eq!(registry.dispatch(0, 0), Some(1));
    assert_eq!(registry.dispatch(1, 0), None); // At width boundary
    assert_eq!(registry.dispatch(0, 1), None); // At height boundary
}

#[test]
fn test_zone_registry_large_coordinates() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(1u32, 1000, 1000, 100, 100);

    assert_eq!(registry.dispatch(1050, 1050), Some(1));
    assert_eq!(registry.dispatch(999, 1000), None);
    assert_eq!(registry.dispatch(1000, 999), None);
}

#[test]
fn test_zone_registry_zones_list() {
    let mut registry = ScopedZoneRegistry::new();
    assert_eq!(registry.zones().len(), 0);

    registry.register(1u32, 10, 10, 5, 5);
    assert_eq!(registry.zones().len(), 1);

    registry.register(2u32, 20, 20, 5, 5);
    assert_eq!(registry.zones().len(), 2);
}

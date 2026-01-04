//! Comprehensive tests for editor selection system.
//!
//! Tests cover:
//! - Single and multi-entity selection
//! - Selection modes (Replace, Add, Remove, Toggle)
//! - Selection events and event handling
//! - Marquee selection
//! - Selection system integration with ECS
//! - Edge cases and error conditions

use bevy_ecs::entity::Entity;
use praxis_editor::{SelectionEvent, SelectionMode, SelectionSystem};
use praxis_math::Vec2;

// ============================================================================
// Basic Selection Tests
// ============================================================================

#[test]
fn test_selection_system_creation() {
    let selection = SelectionSystem::new();
    assert!(selection.is_empty());
    assert_eq!(selection.selected_count(), 0);
}

#[test]
fn test_select_single_entity_replace() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Replace);
    assert!(selection.is_selected(entity));
    assert_eq!(selection.selected_count(), 1);
}

#[test]
fn test_select_single_entity_add() {
    let mut selection = SelectionSystem::new();
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);

    selection.select_entity(entity1, SelectionMode::Add);
    selection.select_entity(entity2, SelectionMode::Add);

    assert!(selection.is_selected(entity1));
    assert!(selection.is_selected(entity2));
    assert_eq!(selection.selected_count(), 2);
}

#[test]
fn test_select_entity_already_selected() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Add);
    let count1 = selection.selected_count();

    selection.select_entity(entity, SelectionMode::Add);
    let count2 = selection.selected_count();

    assert_eq!(count1, count2);
}

// ============================================================================
// Selection Mode Tests
// ============================================================================

#[test]
fn test_selection_mode_replace() {
    let mut selection = SelectionSystem::new();
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);

    selection.select_entity(entity1, SelectionMode::Add);
    selection.select_entity(entity2, SelectionMode::Replace);

    assert!(!selection.is_selected(entity1));
    assert!(selection.is_selected(entity2));
    assert_eq!(selection.selected_count(), 1);
}

#[test]
fn test_selection_mode_remove() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Add);
    assert!(selection.is_selected(entity));

    selection.select_entity(entity, SelectionMode::Remove);
    assert!(!selection.is_selected(entity));
    assert!(selection.is_empty());
}

#[test]
fn test_selection_mode_remove_nonselected() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Remove);
    assert!(!selection.is_selected(entity));
}

#[test]
fn test_selection_mode_toggle() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Toggle);
    assert!(selection.is_selected(entity));

    selection.select_entity(entity, SelectionMode::Toggle);
    assert!(!selection.is_selected(entity));
}

#[test]
fn test_selection_mode_toggle_multiple() {
    let mut selection = SelectionSystem::new();
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);

    selection.select_entity(entity1, SelectionMode::Add);
    selection.select_entity(entity2, SelectionMode::Toggle);

    assert!(selection.is_selected(entity1));
    assert!(selection.is_selected(entity2));

    selection.select_entity(entity1, SelectionMode::Toggle);
    assert!(!selection.is_selected(entity1));
    assert!(selection.is_selected(entity2));
}

// ============================================================================
// Multi-Entity Selection Tests
// ============================================================================

#[test]
fn test_select_multiple_entities_batch() {
    let mut selection = SelectionSystem::new();
    let entities = vec![
        Entity::from_raw(1),
        Entity::from_raw(2),
        Entity::from_raw(3),
    ];

    selection.select_entities(entities.clone(), SelectionMode::Replace);
    assert_eq!(selection.selected_count(), 3);

    for entity in entities {
        assert!(selection.is_selected(entity));
    }
}

#[test]
fn test_select_multiple_entities_add() {
    let mut selection = SelectionSystem::new();
    let entity1 = Entity::from_raw(1);

    selection.select_entity(entity1, SelectionMode::Add);

    let more_entities = vec![Entity::from_raw(2), Entity::from_raw(3)];
    selection.select_entities(more_entities, SelectionMode::Add);

    assert_eq!(selection.selected_count(), 3);
}

#[test]
fn test_select_multiple_entities_replace() {
    let mut selection = SelectionSystem::new();
    let entity1 = Entity::from_raw(1);

    selection.select_entity(entity1, SelectionMode::Add);

    let new_entities = vec![Entity::from_raw(2), Entity::from_raw(3)];
    selection.select_entities(new_entities, SelectionMode::Replace);

    assert!(!selection.is_selected(entity1));
    assert_eq!(selection.selected_count(), 2);
}

#[test]
fn test_select_multiple_entities_remove() {
    let mut selection = SelectionSystem::new();
    let entities = vec![
        Entity::from_raw(1),
        Entity::from_raw(2),
        Entity::from_raw(3),
    ];

    selection.select_entities(entities.clone(), SelectionMode::Add);

    let remove_entities = vec![Entity::from_raw(1), Entity::from_raw(2)];
    selection.select_entities(remove_entities, SelectionMode::Remove);

    assert_eq!(selection.selected_count(), 1);
    assert!(selection.is_selected(Entity::from_raw(3)));
}

#[test]
fn test_select_multiple_entities_toggle() {
    let mut selection = SelectionSystem::new();
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);

    selection.select_entity(entity1, SelectionMode::Add);

    let toggle_entities = vec![entity1, entity2];
    selection.select_entities(toggle_entities, SelectionMode::Toggle);

    assert!(!selection.is_selected(entity1));
    assert!(selection.is_selected(entity2));
}

#[test]
fn test_select_empty_entity_list() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Add);

    selection.select_entities(vec![], SelectionMode::Replace);

    // Should not affect existing selection
    assert!(selection.is_selected(entity));
}

// ============================================================================
// Clear and Deselect Tests
// ============================================================================

#[test]
fn test_clear_selection() {
    let mut selection = SelectionSystem::new();
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);

    selection.select_entity(entity1, SelectionMode::Add);
    selection.select_entity(entity2, SelectionMode::Add);
    assert_eq!(selection.selected_count(), 2);

    selection.clear();
    assert!(selection.is_empty());
}

#[test]
fn test_clear_empty_selection() {
    let mut selection = SelectionSystem::new();
    selection.clear();
    assert!(selection.is_empty());
}

#[test]
fn test_deselect_entity() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Add);
    assert!(selection.is_selected(entity));

    selection.deselect_entity(entity);
    assert!(!selection.is_selected(entity));
}

#[test]
fn test_deselect_nonselected_entity() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.deselect_entity(entity);
    // Should not cause errors
    assert!(!selection.is_selected(entity));
}

// ============================================================================
// Selection Event Tests
// ============================================================================

#[test]
fn test_selection_events_select() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Replace);

    let events: Vec<SelectionEvent> = selection.events().iter().cloned().collect();
    assert!(!events.is_empty());
    assert!(matches!(events[0], SelectionEvent::Selected(_)));
}

#[test]
fn test_selection_events_deselect() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Add);
    selection.drain_events(); // Clear events

    selection.select_entity(entity, SelectionMode::Remove);

    let events: Vec<SelectionEvent> = selection.events().iter().cloned().collect();
    assert!(events
        .iter()
        .any(|e| matches!(e, SelectionEvent::Deselected(_))));
}

#[test]
fn test_selection_events_clear() {
    let mut selection = SelectionSystem::new();
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);

    selection.select_entity(entity1, SelectionMode::Add);
    selection.select_entity(entity2, SelectionMode::Add);
    selection.drain_events(); // Clear events

    selection.clear();

    let events: Vec<SelectionEvent> = selection.events().iter().cloned().collect();
    assert!(events.iter().any(|e| matches!(e, SelectionEvent::Cleared)));
}

#[test]
fn test_selection_events_changed() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Add);

    let events: Vec<SelectionEvent> = selection.events().iter().cloned().collect();
    assert!(events.iter().any(|e| matches!(e, SelectionEvent::Changed)));
}

#[test]
fn test_drain_events() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Replace);
    assert!(!selection.events().is_empty());

    let events = selection.drain_events();
    assert!(!events.is_empty());
    assert!(selection.events().is_empty());
}

#[test]
fn test_event_ordering() {
    let mut selection = SelectionSystem::new();
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);

    selection.select_entity(entity1, SelectionMode::Add);
    selection.select_entity(entity2, SelectionMode::Replace);

    let events = selection.drain_events();

    // First select entity1, then clear (for replace), then select entity2
    assert!(events.len() >= 3);
}

// ============================================================================
// Marquee Selection Tests
// ============================================================================

#[test]
fn test_marquee_selection_start() {
    let mut selection = SelectionSystem::new();

    selection.start_marquee(Vec2::new(10.0, 10.0));
    assert!(selection.is_marquee_active());
}

#[test]
fn test_marquee_selection_update() {
    let mut selection = SelectionSystem::new();

    selection.start_marquee(Vec2::new(10.0, 10.0));
    selection.update_marquee(Vec2::new(50.0, 50.0));
    assert!(selection.is_marquee_active());
}

#[test]
fn test_marquee_selection_end() {
    let mut selection = SelectionSystem::new();

    selection.start_marquee(Vec2::new(10.0, 10.0));
    selection.update_marquee(Vec2::new(50.0, 50.0));

    let rect = selection.end_marquee();
    assert!(rect.is_some());
    assert!(!selection.is_marquee_active());
}

#[test]
fn test_marquee_selection_cancel() {
    let mut selection = SelectionSystem::new();

    selection.start_marquee(Vec2::new(10.0, 10.0));
    assert!(selection.is_marquee_active());

    selection.cancel_marquee();
    assert!(!selection.is_marquee_active());
}

#[test]
fn test_marquee_selection_get_rect() {
    let mut selection = SelectionSystem::new();

    selection.start_marquee(Vec2::new(10.0, 20.0));
    selection.update_marquee(Vec2::new(50.0, 60.0));

    let rect = selection.get_marquee_rect();
    assert!(rect.is_some());

    let (min, max) = rect.unwrap();
    assert_eq!(min.x, 10.0);
    assert_eq!(min.y, 20.0);
    assert_eq!(max.x, 50.0);
    assert_eq!(max.y, 60.0);
}

#[test]
fn test_marquee_selection_negative_rect() {
    let mut selection = SelectionSystem::new();

    // Start at bottom-right, drag to top-left
    selection.start_marquee(Vec2::new(50.0, 60.0));
    selection.update_marquee(Vec2::new(10.0, 20.0));

    let rect = selection.get_marquee_rect();
    assert!(rect.is_some());

    let (min, max) = rect.unwrap();
    // Should normalize to correct min/max
    assert_eq!(min.x, 10.0);
    assert_eq!(min.y, 20.0);
    assert_eq!(max.x, 50.0);
    assert_eq!(max.y, 60.0);
}

// ============================================================================
// Input Control Tests
// ============================================================================

#[test]
fn test_input_enabled_default() {
    let selection = SelectionSystem::new();
    assert!(selection.is_input_enabled());
}

#[test]
fn test_set_input_enabled() {
    let mut selection = SelectionSystem::new();
    assert!(selection.is_input_enabled());

    selection.set_input_enabled(false);
    assert!(!selection.is_input_enabled());

    selection.set_input_enabled(true);
    assert!(selection.is_input_enabled());
}

// ============================================================================
// Iterator Tests
// ============================================================================

#[test]
fn test_selected_entities_iterator() {
    let mut selection = SelectionSystem::new();
    let entities = vec![
        Entity::from_raw(1),
        Entity::from_raw(2),
        Entity::from_raw(3),
    ];

    selection.select_entities(entities.clone(), SelectionMode::Add);

    let selected: Vec<Entity> = selection.selected_entities().collect();
    assert_eq!(selected.len(), 3);

    for entity in entities {
        assert!(selected.contains(&entity));
    }
}

#[test]
fn test_selected_entities_iterator_empty() {
    let selection = SelectionSystem::new();
    let selected: Vec<Entity> = selection.selected_entities().collect();
    assert!(selected.is_empty());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_select_same_entity_multiple_modes() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Add);
    selection.select_entity(entity, SelectionMode::Add);
    selection.select_entity(entity, SelectionMode::Add);

    assert_eq!(selection.selected_count(), 1);
}

#[test]
fn test_large_selection() {
    let mut selection = SelectionSystem::new();
    let entities: Vec<Entity> = (0..1000).map(Entity::from_raw).collect();

    selection.select_entities(entities.clone(), SelectionMode::Add);

    assert_eq!(selection.selected_count(), 1000);

    for entity in entities {
        assert!(selection.is_selected(entity));
    }
}

#[test]
fn test_selection_with_mixed_operations() {
    let mut selection = SelectionSystem::new();

    // Complex sequence of operations
    let e1 = Entity::from_raw(1);
    let e2 = Entity::from_raw(2);
    let e3 = Entity::from_raw(3);
    let e4 = Entity::from_raw(4);

    selection.select_entity(e1, SelectionMode::Add);
    selection.select_entity(e2, SelectionMode::Add);
    selection.select_entities(vec![e3, e4], SelectionMode::Add);
    selection.select_entity(e1, SelectionMode::Remove);
    selection.select_entity(e2, SelectionMode::Toggle);

    assert!(!selection.is_selected(e1));
    assert!(!selection.is_selected(e2));
    assert!(selection.is_selected(e3));
    assert!(selection.is_selected(e4));
    assert_eq!(selection.selected_count(), 2);
}

#[test]
fn test_replace_with_empty_selection() {
    let mut selection = SelectionSystem::new();
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);

    selection.select_entity(entity1, SelectionMode::Add);
    selection.select_entity(entity2, SelectionMode::Replace);

    assert!(!selection.is_selected(entity1));
    assert!(selection.is_selected(entity2));
}

#[test]
fn test_multiple_clears() {
    let mut selection = SelectionSystem::new();
    let entity = Entity::from_raw(1);

    selection.select_entity(entity, SelectionMode::Add);
    selection.clear();
    selection.clear();
    selection.clear();

    assert!(selection.is_empty());
}

#[test]
fn test_selection_count_accuracy() {
    let mut selection = SelectionSystem::new();

    assert_eq!(selection.selected_count(), 0);

    selection.select_entity(Entity::from_raw(1), SelectionMode::Add);
    assert_eq!(selection.selected_count(), 1);

    selection.select_entity(Entity::from_raw(2), SelectionMode::Add);
    assert_eq!(selection.selected_count(), 2);

    selection.select_entity(Entity::from_raw(1), SelectionMode::Remove);
    assert_eq!(selection.selected_count(), 1);

    selection.clear();
    assert_eq!(selection.selected_count(), 0);
}

#[test]
fn test_event_limit() {
    let mut selection = SelectionSystem::new();

    // Generate many events
    for i in 0..150 {
        let entity = Entity::from_raw(i);
        selection.select_entity(entity, SelectionMode::Add);
    }

    // Events should be limited (max 100)
    let events = selection.events();
    assert!(events.len() <= 100);
}

#[test]
fn test_selection_toggle_batch() {
    let mut selection = SelectionSystem::new();
    let e1 = Entity::from_raw(1);
    let e2 = Entity::from_raw(2);
    let e3 = Entity::from_raw(3);

    selection.select_entity(e1, SelectionMode::Add);
    selection.select_entity(e2, SelectionMode::Add);

    // Toggle e1 (deselect) and e3 (select)
    selection.select_entities(vec![e1, e3], SelectionMode::Toggle);

    assert!(!selection.is_selected(e1));
    assert!(selection.is_selected(e2));
    assert!(selection.is_selected(e3));
}

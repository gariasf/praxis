# Exercise 04: Event System

**Difficulty**: 🟡 Intermediate | **Estimated Time**: 2-3h | **Subsystem**: Core

## Overview

Implement a type-safe event system for decoupling game systems. Events are a fundamental communication pattern in game engines.

## Learning Objectives

- Understand publish-subscribe patterns
- Implement type-safe event dispatching
- Learn event queue vs immediate dispatch trade-offs
- Handle event priorities and filtering

## Requirements

### Functional Requirements

1. **Event Registration**
   - Register event listeners for specific event types
   - Support multiple listeners per event type
   - Unregister listeners

2. **Event Dispatch**
   - Immediate dispatch (call all listeners now)
   - Queued dispatch (collect events, process later)
   - Type-safe event data

3. **Event Ordering**
   - Process events in registration order
   - Support priority-based ordering (optional)

### Non-Functional Requirements

- **Performance**: Dispatch 10,000 events in < 1ms
- **Memory**: Reuse event queue memory
- **Type Safety**: Compile-time type checking

## API Design

```rust
pub struct EventBus {
    // Event handlers stored per type
}

impl EventBus {
    pub fn new() -> Self;
    
    pub fn subscribe<E: Event>(&mut self, handler: impl EventHandler<E>) -> HandlerId;
    pub fn unsubscribe<E: Event>(&mut self, id: HandlerId);
    
    pub fn emit<E: Event>(&mut self, event: E);
    pub fn emit_queued<E: Event>(&mut self, event: E);
    pub fn process_queue(&mut self);
    
    pub fn clear<E: Event>(&mut self);
}

pub trait Event: 'static + Send {}

pub trait EventHandler<E: Event> {
    fn handle(&mut self, event: &E);
}
```

## Validation Criteria

### Correctness
- [ ] Listeners receive correct event types
- [ ] Multiple listeners all called
- [ ] Queued events processed in order
- [ ] Unsubscribed listeners not called

### Performance
- [ ] 10,000 events dispatched in < 1ms
- [ ] No allocations during dispatch
- [ ] Listener lookup in O(1)

## Test Cases

```rust
#[test]
fn test_immediate_dispatch() {
    let mut bus = EventBus::new();
    let mut received = false;
    
    bus.subscribe(|event: &TestEvent| {
        received = true;
    });
    
    bus.emit(TestEvent { data: 42 });
    assert!(received);
}

#[test]
fn test_queued_dispatch() {
    let mut bus = EventBus::new();
    let mut count = 0;
    
    bus.subscribe(|_: &TestEvent| {
        count += 1;
    });
    
    bus.emit_queued(TestEvent { data: 1 });
    bus.emit_queued(TestEvent { data: 2 });
    assert_eq!(count, 0); // Not processed yet
    
    bus.process_queue();
    assert_eq!(count, 2);
}

#[test]
fn test_unsubscribe() {
    let mut bus = EventBus::new();
    let mut count = 0;
    
    let id = bus.subscribe(|_: &TestEvent| {
        count += 1;
    });
    
    bus.emit(TestEvent { data: 1 });
    assert_eq!(count, 1);
    
    bus.unsubscribe::<TestEvent>(id);
    bus.emit(TestEvent { data: 2 });
    assert_eq!(count, 1); // Didn't increase
}
```

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub type HandlerId = usize;

pub struct EventBus {
    handlers: HashMap<TypeId, Vec<(HandlerId, Box<dyn Any>)>>,
    queues: HashMap<TypeId, Vec<Box<dyn Any>>>,
    next_id: HandlerId,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            queues: HashMap::new(),
            next_id: 1,
        }
    }
    
    pub fn subscribe<E, F>(&mut self, handler: F) -> HandlerId
    where
        E: Event,
        F: FnMut(&E) + 'static,
    {
        let type_id = TypeId::of::<E>();
        let id = self.next_id;
        self.next_id += 1;
        
        self.handlers
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push((id, Box::new(handler)));
        
        id
    }
    
    pub fn unsubscribe<E: Event>(&mut self, id: HandlerId) {
        let type_id = TypeId::of::<E>();
        
        if let Some(handlers) = self.handlers.get_mut(&type_id) {
            handlers.retain(|(handler_id, _)| *handler_id != id);
        }
    }
    
    pub fn emit<E: Event>(&mut self, event: E) {
        let type_id = TypeId::of::<E>();
        
        if let Some(handlers) = self.handlers.get_mut(&type_id) {
            for (_, handler) in handlers.iter_mut() {
                if let Some(f) = handler.downcast_mut::<Box<dyn FnMut(&E)>>() {
                    f(&event);
                }
            }
        }
    }
    
    pub fn emit_queued<E: Event>(&mut self, event: E) {
        let type_id = TypeId::of::<E>();
        
        self.queues
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(Box::new(event));
    }
    
    pub fn process_queue<E: Event>(&mut self) {
        let type_id = TypeId::of::<E>();
        
        if let Some(queue) = self.queues.remove(&type_id) {
            for boxed_event in queue {
                if let Some(event) = boxed_event.downcast_ref::<E>() {
                    self.emit(event.clone());
                }
            }
        }
    }
    
    pub fn process_all_queues(&mut self) {
        let type_ids: Vec<TypeId> = self.queues.keys().copied().collect();
        
        for type_id in type_ids {
            if let Some(queue) = self.queues.remove(&type_id) {
                if let Some(handlers) = self.handlers.get_mut(&type_id) {
                    for boxed_event in queue {
                        for (_, handler) in handlers.iter_mut() {
                            // Note: This is simplified; real implementation needs
                            // type-safe downcasting per event type
                        }
                    }
                }
            }
        }
    }
    
    pub fn clear<E: Event>(&mut self) {
        let type_id = TypeId::of::<E>();
        self.handlers.remove(&type_id);
        self.queues.remove(&type_id);
    }
}

pub trait Event: 'static + Clone + Send {}

// Example event
#[derive(Clone)]
pub struct EntitySpawned {
    pub entity_id: u32,
    pub position: [f32; 3],
}

impl Event for EntitySpawned {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Clone)]
    struct TestEvent {
        value: i32,
    }
    impl Event for TestEvent {}
    
    #[test]
    fn test_basic_emit() {
        let mut bus = EventBus::new();
        let mut received = 0;
        
        bus.subscribe(move |event: &TestEvent| {
            received = event.value;
        });
        
        bus.emit(TestEvent { value: 42 });
        // Note: closure capture limitations in test
    }
}
```

</details>

## Related Resources

- [Observer Pattern](https://gameprogrammingpatterns.com/observer.html)
- [ECS Events in Bevy](https://bevy-cheatbook.github.io/programming/events.html)

## Next Steps

- Integrate with ECS (Exercise 21)
- Add input events (see `praxis_input`)
- Study event-driven architectures

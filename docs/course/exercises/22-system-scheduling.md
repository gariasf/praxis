# Exercise 22: System Scheduling

**Difficulty**: 🟡 Intermediate | **Estimated Time**: 3-4h | **Subsystem**: ECS

## Overview

Implement a system scheduler that determines the execution order of ECS systems based on dependencies and resource access. Critical for correct and efficient ECS execution.

## Learning Objectives

- Understand system dependencies and ordering
- Learn topological sorting for DAG execution
- Implement conflict detection (data races)
- Enable parallel system execution

## Requirements

### Functional Requirements

1. **System Registration**
   - Register systems with dependencies
   - Specify read/write access to components
   - Define explicit before/after relationships

2. **Schedule Generation**
   - Compute valid execution order
   - Detect circular dependencies
   - Group independent systems (for parallelization)

3. **Conflict Detection**
   - Two systems conflict if:
     - Both write to same component
     - One writes, one reads same component
   - Non-conflicting systems can run in parallel

4. **Schedule Execution**
   - Run systems in order
   - Support parallel execution stages
   - Handle system errors gracefully

### Non-Functional Requirements

- **Performance**: Schedule computation < 1ms
- **Correctness**: No data races, deterministic order
- **Usability**: Clear error messages for conflicts

## API Design

```rust
pub struct SystemScheduler {
    systems: Vec<Box<dyn System>>,
    schedule: Schedule,
}

pub struct Schedule {
    stages: Vec<Stage>,
}

pub struct Stage {
    systems: Vec<usize>, // System indices that can run in parallel
}

pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn run(&mut self, world: &mut World);
    fn reads(&self) -> Vec<TypeId>;
    fn writes(&self) -> Vec<TypeId>;
}

impl SystemScheduler {
    pub fn new() -> Self;
    pub fn add_system<S: System + 'static>(&mut self, system: S);
    pub fn build_schedule(&mut self) -> Result<(), ScheduleError>;
    pub fn execute(&mut self, world: &mut World);
}

#[derive(Debug)]
pub enum ScheduleError {
    CircularDependency(Vec<String>),
    ConflictingAccess { system1: String, system2: String, component: String },
}
```

## Validation Criteria

### Correctness
- [ ] Systems execute in valid order
- [ ] Circular dependencies detected
- [ ] Conflicting access detected
- [ ] Parallel stages correct
- [ ] Deterministic execution order

### Performance
- [ ] Schedule build < 1ms for 100 systems
- [ ] Execution overhead < 0.1ms per system

## Test Cases

```rust
#[test]
fn test_basic_ordering() {
    let mut scheduler = SystemScheduler::new();
    
    scheduler.add_system(SystemA); // Writes Position
    scheduler.add_system(SystemB); // Reads Position
    
    scheduler.build_schedule().unwrap();
    
    // SystemA must run before SystemB
    assert!(scheduler.get_order("SystemA") < scheduler.get_order("SystemB"));
}

#[test]
fn test_circular_dependency_detection() {
    let mut scheduler = SystemScheduler::new();
    
    // A depends on B, B depends on A
    scheduler.add_system_with_dependency(SystemA, vec!["SystemB"]);
    scheduler.add_system_with_dependency(SystemB, vec!["SystemA"]);
    
    let result = scheduler.build_schedule();
    assert!(matches!(result, Err(ScheduleError::CircularDependency(_))));
}

#[test]
fn test_parallel_execution() {
    let mut scheduler = SystemScheduler::new();
    
    // Independent systems can run in parallel
    scheduler.add_system(SystemA); // Writes Position
    scheduler.add_system(SystemB); // Writes Velocity (different component)
    
    scheduler.build_schedule().unwrap();
    
    // Both should be in the same stage
    assert_eq!(scheduler.schedule.stages[0].systems.len(), 2);
}

#[test]
fn test_conflict_detection() {
    let mut scheduler = SystemScheduler::new();
    
    // Both write to Position - conflict!
    scheduler.add_system(SystemA); // Writes Position
    scheduler.add_system(SystemC); // Writes Position
    
    let result = scheduler.build_schedule();
    // Should either error or sequence them
    assert!(result.is_ok());
    
    // If ok, they must be in different stages
    let schedule = scheduler.schedule;
    // Check they're not in same stage
}
```

## Algorithms

### Topological Sort (Kahn's Algorithm)
```rust
fn topological_sort(systems: &[SystemNode]) -> Result<Vec<usize>, Vec<usize>> {
    let mut in_degree = vec![0; systems.len()];
    let mut graph = vec![Vec::new(); systems.len()];
    
    // Build dependency graph
    for (i, system) in systems.iter().enumerate() {
        for dep in &system.dependencies {
            graph[*dep].push(i);
            in_degree[i] += 1;
        }
    }
    
    // Find systems with no dependencies
    let mut queue: Vec<usize> = (0..systems.len())
        .filter(|&i| in_degree[i] == 0)
        .collect();
    
    let mut result = Vec::new();
    
    while let Some(node) = queue.pop() {
        result.push(node);
        
        for &dependent in &graph[node] {
            in_degree[dependent] -= 1;
            if in_degree[dependent] == 0 {
                queue.push(dependent);
            }
        }
    }
    
    if result.len() == systems.len() {
        Ok(result)
    } else {
        // Circular dependency - return remaining nodes
        Err((0..systems.len())
            .filter(|&i| in_degree[i] > 0)
            .collect())
    }
}
```

### Conflict Detection
```rust
fn detect_conflicts(
    systems: &[SystemNode],
    order: &[usize],
) -> Result<(), (usize, usize, TypeId)> {
    for i in 0..order.len() {
        for j in (i + 1)..order.len() {
            let sys_a = &systems[order[i]];
            let sys_b = &systems[order[j]];
            
            // Check for write-write conflicts
            for write_a in &sys_a.writes {
                if sys_b.writes.contains(write_a) {
                    return Err((order[i], order[j], *write_a));
                }
            }
            
            // Check for read-write conflicts
            for write_a in &sys_a.writes {
                if sys_b.reads.contains(write_a) {
                    return Err((order[i], order[j], *write_a));
                }
            }
            
            for read_a in &sys_a.reads {
                if sys_b.writes.contains(read_a) {
                    return Err((order[i], order[j], *read_a));
                }
            }
        }
    }
    
    Ok(())
}
```

### Stage Generation (Parallel Groups)
```rust
fn generate_stages(
    systems: &[SystemNode],
    order: &[usize],
) -> Vec<Stage> {
    let mut stages = Vec::new();
    let mut scheduled = vec![false; systems.len()];
    
    while scheduled.iter().any(|&s| !s) {
        let mut stage = Stage { systems: Vec::new() };
        
        for &idx in order {
            if scheduled[idx] {
                continue;
            }
            
            // Check if this system conflicts with any in current stage
            let conflicts = stage.systems.iter().any(|&other| {
                systems_conflict(&systems[idx], &systems[other])
            });
            
            if !conflicts {
                stage.systems.push(idx);
                scheduled[idx] = true;
            }
        }
        
        if !stage.systems.is_empty() {
            stages.push(stage);
        }
    }
    
    stages
}

fn systems_conflict(a: &SystemNode, b: &SystemNode) -> bool {
    // Write-write conflict
    for write_a in &a.writes {
        if b.writes.contains(write_a) {
            return true;
        }
    }
    
    // Read-write conflicts
    for write_a in &a.writes {
        if b.reads.contains(write_a) {
            return true;
        }
    }
    
    for read_a in &a.reads {
        if b.writes.contains(read_a) {
            return true;
        }
    }
    
    false
}
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Schedule 10 systems | < 0.1ms |
| Schedule 100 systems | < 1ms |
| Detect circular dep | < 1ms |
| Execute overhead | < 0.1ms per system |

## Hints & Guidance

### Declaring System Access
```rust
struct PhysicsSystem;

impl System for PhysicsSystem {
    fn name(&self) -> &str {
        "Physics"
    }
    
    fn reads(&self) -> Vec<TypeId> {
        vec![TypeId::of::<Transform>()]
    }
    
    fn writes(&self) -> Vec<TypeId> {
        vec![TypeId::of::<Velocity>(), TypeId::of::<Position>()]
    }
    
    fn run(&mut self, world: &mut World) {
        // Update physics
    }
}
```

### Explicit Dependencies
```rust
scheduler
    .add_system(PhysicsSystem)
    .after(InputSystem)
    .before(RenderSystem);
```

### Common Pitfalls
- **Implicit Dependencies**: Systems may have implicit ordering requirements not captured by component access
- **Over-sequencing**: Too conservative scheduling reduces parallelism
- **Mutation vs Immutable Access**: Distinguish between reading and writing

## Reference Implementation

See `bevy_ecs` scheduler and `specs` parallel dispatcher for production examples.

## Related Resources

- [Bevy ECS Scheduling](https://bevyengine.org/learn/book/migration-guides/0.6-0.7/#system-stage-refactor)
- [Specs Parallel Dispatcher](https://specs.amethyst.rs/docs/tutorials/09_parallel_join.html)
- [ECS Back and Forth](https://skypjack.github.io/2019-02-14-ecs-baf-part-1/)

## Next Steps

- Implement parallel execution with rayon
- Add system profiling integration
- Study `bevy_ecs` schedule implementation

# Exercise XX: [Title]

**Difficulty**: 🟢/🟡/🔴 | **Estimated Time**: X-Yh | **Subsystem**: [Core/Graphics/ECS/Physics/Assets/etc.]

## Overview

Brief description of what this exercise teaches and why it's important in game engine development.

## Learning Objectives

- Objective 1
- Objective 2
- Objective 3

## Requirements

### Functional Requirements

1. **Requirement 1**
   - Detailed specification
   - Expected inputs/outputs
   - Edge cases to handle

2. **Requirement 2**
   - Detailed specification
   - Expected inputs/outputs
   - Edge cases to handle

### Non-Functional Requirements

- **Performance**: Specific benchmarks (e.g., "Process 10,000 entities in < 1ms")
- **Memory**: Memory constraints (e.g., "Use < 100MB for cache")
- **Thread Safety**: Concurrency requirements
- **Error Handling**: How to handle failures

## API Design

Suggested API surface (feel free to deviate based on your design):

```rust
// Rust example
pub struct ComponentName {
    // fields
}

impl ComponentName {
    pub fn new() -> Self {
        // ...
    }
    
    pub fn method_name(&self) -> Result<()> {
        // ...
    }
}
```

## Validation Criteria

Your implementation must satisfy all of the following:

### Correctness
- [ ] Test case 1 description
- [ ] Test case 2 description
- [ ] Edge case handling

### Performance
- [ ] Benchmark 1: Target < Xms
- [ ] Benchmark 2: Target < Y operations/sec

### Code Quality
- [ ] No unsafe code (unless explicitly required)
- [ ] Proper error handling
- [ ] Documentation for public APIs

## Expected Behavior

Describe what a working implementation should do:

1. **Scenario 1**: When X happens, the system should Y
2. **Scenario 2**: When A happens, the system should B

Include example output or screenshots if applicable.

## Test Cases

Provide specific test cases to validate the implementation:

```rust
#[test]
fn test_basic_functionality() {
    // Setup
    let component = ComponentName::new();
    
    // Execute
    let result = component.method_name();
    
    // Verify
    assert!(result.is_ok());
}

#[test]
fn test_edge_case() {
    // ...
}
```

## Performance Targets

| Scenario | Target | Measurement Method |
|----------|--------|-------------------|
| Operation 1 | < X ms | Time 1000 iterations |
| Operation 2 | > Y ops/sec | Throughput test |
| Memory usage | < Z MB | Profile with tool |

## Hints & Guidance

### Getting Started
1. Start by defining the data structures
2. Implement the simplest case first
3. Add complexity incrementally

### Common Pitfalls
- Pitfall 1: Description and how to avoid
- Pitfall 2: Description and how to avoid

### Key Concepts
- Concept 1: Explanation of relevant theory
- Concept 2: Explanation of relevant theory

## Extensions (Optional)

Ideas for extending the exercise once complete:

1. **Extension 1**: Description
2. **Extension 2**: Description

## Reference Implementation

Reference implementations are provided in multiple languages. Study these AFTER attempting your own implementation.

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
// Full working implementation
// File: reference/rust/exercise_XX.rs

// Implementation code here...
```

</details>

### C++ (Alternative)

<details>
<summary>Click to reveal C++ implementation</summary>

```cpp
// Full working implementation
// File: reference/cpp/exercise_XX.cpp

// Implementation code here...
```

</details>

### Python (Simplified)

<details>
<summary>Click to reveal Python implementation</summary>

```python
# Simplified implementation for understanding concepts
# File: reference/python/exercise_XX.py

# Implementation code here...
```

</details>

## Related Resources

- [Relevant documentation](link)
- [Related concept guide](link)
- [External resource](link)

## Next Steps

After completing this exercise:
- Try Extension 1
- Move to Exercise YY
- Read about [related topic]

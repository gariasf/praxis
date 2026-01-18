# Documentation Standards for Praxis Guides

This document defines the standards for technical documentation in the Praxis game engine, ensuring consistency, clarity, and completeness across all guides.

## Purpose

Well-structured documentation helps developers:
- **Learn quickly**: Consistent structure aids navigation and comprehension
- **Find information**: Clear organization and cross-references
- **Understand concepts**: Theory before implementation
- **Write correct code**: Verified, compilable examples
- **Solve problems**: Troubleshooting sections and common patterns

## Document Structure

All technical guides MUST follow this structure:

### 1. Title and Overview (Required)

```markdown
# Feature Name

Brief 1-2 paragraph overview explaining what this feature does and its primary use cases.

## Overview

- **Key Feature 1**: Brief description
- **Key Feature 2**: Brief description
- **Key Feature 3**: Brief description
```

### 2. When to Use (Required)

Explain the decision-making process for using this feature:

```markdown
## When to Use

**Use [feature] when:**
- Scenario 1
- Scenario 2
- Scenario 3

**Avoid [feature] when:**
- Anti-pattern 1
- Anti-pattern 2

**Alternatives:**
- Alternative A: Better for X
- Alternative B: Better for Y
```

### 3. Concepts (Optional but Recommended)

Theoretical foundation before diving into code:

```markdown
## Concepts

### Core Concept 1

Explanation of the underlying concept, including:
- Why this approach
- How it works
- Mathematical/algorithmic foundation (if applicable)
- Comparison with alternatives

### Core Concept 2

[...]
```

### 4. Components and Architecture (Required for System Docs)

```markdown
## Core Components

### Component Name

Description of component purpose and responsibility.

**Key Types:**
- `TypeA`: Purpose
- `TypeB`: Purpose

**Configuration:**
```rust
struct Config {
    param1: Type, // Description
    param2: Type, // Description
}
```

### 5. Implementation (Required)

Step-by-step setup with complete code examples:

```markdown
## Implementation

### Step 1: Basic Setup

```rust
use praxis_module::{Type1, Type2};

// Complete working code example
let instance = Type1::new(parameters)?;
```

**Explanation**: What this code does and why.

### Step 2: Configuration

[...]
```

### 6. Usage Examples (Required)

```markdown
## Usage Examples

### Example 1: Common Use Case

```rust
// Complete, self-contained example
use praxis_module::{RequiredImport1, RequiredImport2};

fn example() -> Result<()> {
    // Working code
    Ok(())
}
```

**When to use**: Scenario description.

### Example 2: Advanced Use Case

[...]
```

### 7. Performance (Required for Performance-Critical Features)

```markdown
## Performance

### Cost Analysis

| Operation | CPU Cost | GPU Cost | Memory |
|-----------|----------|----------|--------|
| Operation 1 | ~XY μs | N/A | AB KB |
| Operation 2 | ~XY μs | ~Z ms | CD MB |

### Optimization Strategies

1. **Strategy 1**: Description and impact
2. **Strategy 2**: Description and impact

### Scalability Guidelines

- ✓ **Excellent**: < X entities/objects
- ✓ **Good**: X-Y entities/objects
- ⚠ **Acceptable**: Y-Z entities/objects (with optimizations)
- ✗ **Poor**: > Z entities/objects (requires alternative approach)
```

### 8. Best Practices (Required)

```markdown
## Best Practices

### Do's

- ✓ **Do this**: Explanation why
- ✓ **Do that**: Explanation why

### Don'ts

- ✗ **Don't do this**: Explanation why (provide alternative)
- ✗ **Don't do that**: Explanation why (provide alternative)

### Common Patterns

```rust
// Well-established pattern for common task
```
```

### 9. Troubleshooting (Required)

```markdown
## Troubleshooting

### Issue: Problem Description

**Symptoms**: What the user observes

**Causes**:
- Possible cause 1
- Possible cause 2

**Solutions**:
1. Try solution A
2. If that doesn't work, try solution B
3. As a last resort, solution C

### Issue: Another Problem

[...]
```

### 10. Examples (Required)

```markdown
## Examples

Runnable examples from the repository:

```bash
# Example description
cargo run --example example_name

# Another example
cargo run --example another_example
```

See `examples/example_name.rs` for complete source code.
```

### 11. See Also (Required)

```markdown
## See Also

- [Related Guide](relative-path.md) - Brief description
- [Another Guide](another-path.md) - Brief description
- [Concept Doc](../../concepts/concept.md) - Brief description

### Related Crates

- [`praxis_module`](../../crates/praxis_module/README.md) - API documentation
```

### 12. References (Optional)

```markdown
## References

- [Paper Title](URL) - Author (Year)
- [Article Title](URL) - Source
- [Book Name](URL) - Chapter X
```

## Code Example Standards

### Complete Examples

❌ **Bad** - Fragment without context:
```rust
// Missing imports and setup
object.set_value(10);
```

✓ **Good** - Complete, runnable code:
```rust
use praxis_graphics::Object;

fn setup() -> Result<()> {
    let mut object = Object::new()?;
    object.set_value(10);
    Ok(())
}
```

### Imports and Dependencies

Always show necessary imports:

```rust
use praxis_core::Engine;
use praxis_ecs::{World, Query};
use praxis_math::{Vec3, Mat4};
```

### Error Handling

Show appropriate error handling:

```rust
// For examples that should handle errors
fn example() -> Result<()> {
    let resource = load_resource()?;
    Ok(())
}

// For examples where errors are intentionally ignored
fn example() {
    let resource = load_resource().expect("Resource must exist");
}
```

### Comments

- Comment non-obvious logic
- Don't comment obvious code
- Explain "why", not "what"

❌ **Bad**:
```rust
// Set x to 10
let x = 10;
```

✓ **Good**:
```rust
// Align to 16-byte boundary for SIMD
let aligned_size = (size + 15) & !15;
```

### Realistic Variable Names

Use meaningful names that reflect actual usage:

❌ **Bad**:
```rust
let x = SomeType::new(a, b, c);
```

✓ **Good**:
```rust
let material_manager = MaterialManager::new(device, allocator);
```

## Cross-Reference Standards

### Internal Links

Use relative paths from current document:

```markdown
See [HDR and Tone Mapping](hdr-tonemapping.md) for details.
```

### Links to Parent Directories

```markdown
See [Animation Concepts](../../concepts/animation.md).
```

### Links to Crate Documentation

```markdown
See [`praxis_graphics`](../../crates/praxis_graphics/README.md) for API reference.
```

### Anchors for Sections

Only when linking to specific section:

```markdown
See [Frustum Culling - Performance](frustum-culling.md#performance).
```

## Writing Style

### Tone

- **Direct and concise**: Get to the point quickly
- **Technical but accessible**: Assume basic knowledge, explain advanced concepts
- **Practical**: Focus on how to use, not just what it is
- **Structured**: Use headings, lists, and tables for scannability

### Language

- **Active voice**: "The renderer processes the scene" not "The scene is processed by the renderer"
- **Present tense**: "The system uses" not "The system will use"
- **Concrete examples**: Provide real scenarios, not abstractions
- **Avoid jargon**: Or define technical terms when first used

### Formatting

- **Bold** for emphasis on key terms
- `Code` formatting for types, functions, files
- > Blockquotes for important notes
- Tables for comparisons and structured data
- Lists for steps and options

## Common Patterns

### Configuration Tables

```markdown
| Parameter | Type | Description | Range | Default |
|-----------|------|-------------|-------|---------|
| `param1` | f32 | What it does | 0.0-1.0 | 0.5 |
| `param2` | u32 | What it does | 1-100 | 10 |
```

### Performance Metrics

```markdown
### Performance (1080p, RTX 3060)

- **Operation A**: ~2ms
- **Operation B**: ~0.5ms
- **Total**: ~2.5ms (~15% of 16.67ms budget at 60fps)
```

### Feature Comparison

```markdown
| Feature | Approach A | Approach B |
|---------|------------|------------|
| Performance | Fast | Slower |
| Quality | Lower | Higher |
| Use Case | Real-time | Cinematic |
```

### Version/Status Indicators

When documenting experimental features:

```markdown
> **Note**: This feature is experimental and may change in future releases.

> **Warning**: This operation is expensive and should be used sparingly.

> **Deprecated**: Use `new_function()` instead. This will be removed in version 0.5.
```

## Quality Checklist

Before finalizing documentation:

- [ ] Title clearly describes feature
- [ ] Overview explains purpose in 1-2 paragraphs
- [ ] "When to Use" section guides decision-making
- [ ] Concepts explained before implementation
- [ ] Code examples are complete and compilable
- [ ] All imports shown
- [ ] Error handling appropriate for context
- [ ] Performance section (if applicable)
- [ ] Best practices included
- [ ] Troubleshooting covers common issues
- [ ] Cargo example commands provided
- [ ] Cross-references use relative links
- [ ] Writing is concise and clear
- [ ] No line number references
- [ ] Tables formatted correctly
- [ ] Code blocks have language specification

## Maintenance

### Updating Documentation

When code changes:

1. Update affected examples
2. Verify code still compiles
3. Update performance numbers if applicable
4. Add migration notes if API changed
5. Update cross-references

### Adding New Features

When adding features:

1. Create new guide following this structure
2. Add to parent README index
3. Add cross-references from related guides
4. Create example in `examples/`
5. Reference example in guide

## Templates

### New Guide Template

```markdown
# Feature Name

Brief description of what this feature does and why it exists.

## Overview

- **Key aspect 1**: Description
- **Key aspect 2**: Description
- **Key aspect 3**: Description

## When to Use

**Use [feature] when:**
- Scenario 1
- Scenario 2

**Avoid when:**
- Anti-pattern 1

## Concepts

### Core Concept

Explanation...

## Components

### Component Name

Description...

## Implementation

### Step 1: Setup

[...]

## Usage Examples

### Example: Common Case

[...]

## Performance

[...]

## Best Practices

[...]

## Troubleshooting

[...]

## Examples

```bash
cargo run --example feature_demo
```

## See Also

- [Related](related.md)

## References

- [Source](URL)
```

---

## Summary

This standards document ensures Praxis documentation is:

✓ **Consistent**: Same structure across all guides
✓ **Complete**: All necessary information included
✓ **Correct**: Verified, compilable examples
✓ **Clear**: Well-organized and easy to navigate
✓ **Practical**: Focused on helping developers succeed

Follow these standards for all new documentation and gradually update existing docs to match.

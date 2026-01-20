# API Design Patterns - Quick Reference

This directory contains comprehensive analysis of API design patterns used in game engines, with examples from Praxis, Unity, Bevy, and Unreal.

## Contents

1. **[index.md](index.md)** - Overview and introduction to API design patterns
2. **[builder-patterns.md](builder-patterns.md)** - Building complex objects with fluent APIs
3. **[fluent-interfaces.md](fluent-interfaces.md)** - Method chaining for readable APIs
4. **[declarative-vs-imperative.md](declarative-vs-imperative.md)** - Different API philosophies
5. **[script-bindings.md](script-bindings.md)** - Exposing engine functionality to scripting languages
6. **[language-constraints.md](language-constraints.md)** - How language features shape API design
7. **[component-apis.md](component-apis.md)** - Entity-component composition patterns

## Quick Navigation

### By Topic

**Object Construction**:
- [Builder Patterns](builder-patterns.md) - Configure complex objects
- [Fluent Interfaces](fluent-interfaces.md) - Chain method calls

**API Philosophy**:
- [Declarative vs Imperative](declarative-vs-imperative.md) - What vs How
- [Component APIs](component-apis.md) - Entity composition approaches

**Language-Specific**:
- [Language Constraints](language-constraints.md) - Templates, traits, generics
- [Script Bindings](script-bindings.md) - FFI and scripting integration

### By Engine

**Praxis Examples**:
- Transform builders ([Builder Patterns](builder-patterns.md#real-world-examples))
- Scene definitions ([Declarative vs Imperative](declarative-vs-imperative.md#real-world-example-animation))
- ECS queries ([Component APIs](component-apis.md#query-based-access-ecs))
- Lua bindings ([Script Bindings](script-bindings.md#praxis-ecs-script-bindings))

**Unity Examples**:
- Extension methods ([Fluent Interfaces](fluent-interfaces.md#c-extension-methods))
- MonoBehaviour lifecycle ([Declarative vs Imperative](declarative-vs-imperative.md#unity-declarative-inspector-imperative-scripts))
- GameObject components ([Component APIs](component-apis.md#constructor-based-traditional-oop))

**Bevy Examples**:
- App builder ([Fluent Interfaces](fluent-interfaces.md#bevy-system-scheduling))
- Bundle pattern ([Component APIs](component-apis.md#bundle-pattern))
- Query filters ([Component APIs](component-apis.md#query-filtering-patterns))

**Unreal Examples**:
- Actor construction ([Component APIs](component-apis.md#constructor-based-traditional-oop))
- Template-based queries ([Language Constraints](language-constraints.md#template-metaprogramming))

### By Language Feature

**Rust**:
- Traits and ownership ([Language Constraints](language-constraints.md#rust-traits-and-ownership))
- Type-state builders ([Builder Patterns](builder-patterns.md#type-state-builder-pattern))
- Lifetime management ([Language Constraints](language-constraints.md#lifetime-elision-and-query-ergonomics))

**C++**:
- Templates ([Language Constraints](language-constraints.md#c-templates-and-sfinae))
- SFINAE ([Language Constraints](language-constraints.md#sfinae-substitution-failure-is-not-an-error))
- Perfect forwarding ([Language Constraints](language-constraints.md#perfect-forwarding))

**C#**:
- Generics ([Language Constraints](language-constraints.md#c-generics-and-extension-methods))
- Extension methods ([Language Constraints](language-constraints.md#extension-methods))
- LINQ patterns ([Fluent Interfaces](fluent-interfaces.md#method-reference-chaining))

## Learning Paths

### Beginner Path
1. Start with [Declarative vs Imperative](declarative-vs-imperative.md) to understand fundamental approaches
2. Read [Component APIs](component-apis.md) to learn entity-component patterns
3. Explore [Builder Patterns](builder-patterns.md) for object construction
4. Study [Fluent Interfaces](fluent-interfaces.md) for method chaining

### Intermediate Path
1. [Language Constraints](language-constraints.md) - How language shapes API
2. [Script Bindings](script-bindings.md) - FFI and cross-language APIs
3. [Component APIs](component-apis.md) - Advanced query patterns
4. [Builder Patterns](builder-patterns.md) - Type-state patterns

### Advanced Path
1. [Language Constraints](language-constraints.md) - Deep dive into type systems
2. [Script Bindings](script-bindings.md) - Safe FFI design and memory management
3. All patterns - Study trade-offs for engine architecture decisions

## Pattern Comparison Table

| Pattern | Best For | Primary Benefit | Main Trade-off |
|---------|----------|-----------------|----------------|
| **Builder** | Complex configuration | Ergonomics | Validation timing |
| **Fluent** | Command sequences | Readability | Debug difficulty |
| **Declarative** | Scene composition | Clarity | Less control |
| **Imperative** | Performance-critical | Full control | Verbosity |
| **Script Bindings** | Modding/iteration | Flexibility | Safety overhead |
| **ECS Queries** | Batch processing | Cache locality | Learning curve |
| **OOP Components** | Random access | Familiarity | Poor locality |

## Common Use Cases

### Spawning Entities
- [Component APIs - Entity Creation](component-apis.md#entity-creation-patterns)
- [Declarative vs Imperative - Scene Composition](declarative-vs-imperative.md#use-case-analysis)

### Configuring Components
- [Builder Patterns - Audio Source Example](builder-patterns.md#basic-builder-pattern)
- [Fluent Interfaces - Transform Configuration](fluent-interfaces.md#self-returning-methods)

### Querying Entities
- [Component APIs - Query Patterns](component-apis.md#component-access-patterns)
- [Language Constraints - Query Comparison](language-constraints.md#language-comparison-query-apis)

### Exposing to Scripts
- [Script Bindings - FFI Patterns](script-bindings.md#ffi-architecture-patterns)
- [Script Bindings - Safety and Sandboxing](script-bindings.md#sandboxing-and-security)

## Contributing

When adding new patterns or examples:

1. **Compare across engines** - Show how Praxis, Unity, Bevy, and Unreal solve the problem
2. **Include code examples** - Use the multi-language tab format
3. **Discuss trade-offs** - Every pattern has pros and cons
4. **Link related patterns** - Help readers build connections
5. **Provide context** - When and why to use each approach

## Related Documentation

- [Game Engine Patterns](../patterns/index.md) - Architecture patterns
- [Code Examples](../CODE_EXAMPLES.md) - Multi-language implementations
- [Curriculum](../CURRICULUM.md) - Structured learning path
- [Component Storage Strategies](../patterns/component-storage-strategies.md) - Data layout patterns

---

**Start here**: [API Design Patterns Overview](index.md)

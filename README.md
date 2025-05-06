# Praxis Engine

## Project goals
- Learn about Game Engine Foundations, 3D space, and systems programming using Rust.
- Create a game engine using idiomatic Rust practices.
- Build a practical engine capable of supporting game development within a 1-2 year timeframe.
- Eventually support complex game worlds and interactions.

## Project rules
- Use free/open, battle-proven libraries (crates) only.
- Avoid proprietary or costly tools.
- Prioritize simplicity and clarity in design.
- Focus on pragmatic solutions and iterative feature development.
- Minimize unnecessary abstractions.

## Technical Scope

### Core Technologies
- **Windowing & Input**: `winit` crate.
- **ECS**: `bevy_ecs` for entity-component-system architecture.
- **Math**: `glam` for SIMD-accelerated vector/matrix math.

## Coding Guidelines and Architecture

See [Architecture Docs](docs/ARCHITECTURE.md) for the detailed Rust-based architecture.

### Naming Conventions
Follow standard Rust API Guidelines:
- **Crates, Modules, Functions, Variables, Fields**: `snake_case`.
- **Types (Structs, Enums, Traits), Lifetimes, Type Parameters**: `PascalCase`.
- **Constants, Statics**: `SCREAMING_SNAKE_CASE`.

### Formatting and Linting
- **Formatting**: Enforced by `rustfmt` (use `cargo fmt`).
- **Linting**: Enforced by `clippy` (use `cargo clippy -- -D warnings`).

### Learning Resources Collection (Rust Focus)
- [The Rust Programming Language Book ("The Book")](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Are We Game Yet?](https://arewegameyet.rs/) (Overview of Rust game development ecosystem)
- [Learn WGPU](https://sotrh.github.io/learn-wgpu/)
- [Vulkan Tutorial](https://vulkan-tutorial.com/) (Still relevant for Vulkan concepts)
- [ash Crate Documentation](https://docs.rs/ash/)
- [bevy_ecs Documentation](https://docs.rs/bevy_ecs/)
- [glam Crate Documentation](https://docs.rs/glam/)
- Game Engine Architecture" by Jason Gregory (General concepts still apply)
- "Real-Time Rendering" by Tomas Akenine-Möller (General concepts still apply)
- [Physically Based Rendering Book](https://www.pbr-book.org/) (Graphics theory)

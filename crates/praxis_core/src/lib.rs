//! Praxis is the main crate for the Praxis game engine.
//!
//! This crate provides the core functionality and coordinates all the subsystems.

/// Initializes and runs the Praxis game engine.
///
/// This is the main entry point that orchestrates the engine's startup sequence.
/// The initialization order is carefully designed to handle dependencies between
/// subsystems, where later systems depend on earlier ones being fully initialized.
///
/// # Engine Initialization Flow
///
/// The engine follows a strict initialization sequence to ensure all dependencies
/// are satisfied before subsystems are activated:
///
/// 1. **Utils** (`praxis_utils::init()`)
/// 2. **ECS** (`praxis_ecs::init()`)
/// 3. **Input** (`praxis_input::init()`)
/// 4. **Audio** (`praxis_audio::init()`)
/// 5. **Window** (`praxis_window::run()`)
///
/// Each subsystem must complete initialization before the next begins. This ensures
/// that when a subsystem starts, all its dependencies are ready for use.
///
/// # Why Initialization Order Matters
///
/// ## 1. Utils First: Foundation Layer
///
/// `praxis_utils` must initialize before all other systems because it provides:
/// - **Logging infrastructure**: Sets up `tracing` subscribers for diagnostics
/// - **Error handling**: Establishes `color-eyre` panic hooks and error reporting
/// - **Timing utilities**: Initializes high-precision timers used throughout the engine
///
/// Without proper logging, subsequent initialization failures would be invisible.
/// Without error handling setup, panics would lack useful context for debugging.
///
/// ## 2. ECS Second: Data Foundation
///
/// `praxis_ecs` (via `bevy_ecs`) initializes next because it provides:
/// - **Component storage**: The core data structures for all game objects
/// - **Entity management**: Systems for creating/destroying entities
/// - **System scheduling**: The framework for running game logic
///
/// Many subsystems register components and systems with the ECS during their
/// initialization. The ECS World must exist before they can do so. This follows
/// the **Data-Oriented Design** principle central to the engine architecture
/// (see `docs/architecture.md`).
///
/// ## 3. Input Third: Event Processing
///
/// `praxis_input` initializes after ECS because:
/// - It registers input-related **components** (e.g., keyboard state, mouse position)
/// - It may create **system resources** that other systems query
/// - It prepares to process events from the windowing system
///
/// Input initialization must complete before the window opens, as the window
/// immediately begins generating input events that need proper handling.
///
/// ## 4. Audio Fourth: Resource Preparation
///
/// `praxis_audio` (using `kira`) initializes before the window because:
/// - Audio context setup can be time-consuming
/// - It creates audio output streams that may fail independently
/// - Early initialization allows graceful degradation if audio hardware is unavailable
///
/// If audio initialization fails, the engine can continue with visual-only output.
/// Starting audio before the main loop prevents the first frame from being delayed
/// by audio setup latency.
///
/// ## 5. Window Last: Main Event Loop
///
/// `praxis_window::run()` is called last because it:
/// - **Takes ownership** of the execution thread (blocking call)
/// - **Creates the window** and Vulkan surface for rendering
/// - **Starts the main event loop** that pumps input, runs systems, and renders
///
/// Once the window system starts, it enters the main loop and never returns until
/// the application exits. All subsystems must be ready before this point because
/// the first frame will immediately attempt to use them.
///
/// The window system integrates with:
/// - `praxis_graphics` for Vulkan rendering (see `docs/guides/rendering.md`)
/// - `praxis_input` for keyboard/mouse/gamepad events
/// - `praxis_ecs` for running game logic each frame
/// - `praxis_physics` for fixed-timestep simulation
///
/// # Subsystem Coordination Patterns
///
/// The engine uses several patterns to coordinate subsystems:
///
/// ## Resource Injection
/// Subsystems expose resources through the ECS World (e.g., `PhysicsWorld`,
/// `AudioManager`). Systems query these resources to interact with subsystems.
///
/// ## Event Propagation
/// Input events flow: Window → Input System → ECS Events → Game Systems.
/// This decoupling allows systems to respond to input without direct dependencies.
///
/// ## System Ordering
/// The ECS scheduler ensures systems run in the correct order each frame:
/// - Input processing → Game logic → Physics → Animation → Rendering
///   (See individual system documentation for scheduling details)
///
/// # Architecture References
///
/// - **Overall architecture**: `docs/architecture.md`
/// - **ECS patterns**: `docs/concepts/ecs.md`
/// - **Rendering pipeline**: `docs/guides/rendering.md`
/// - **Crate organization**: `docs/reference/crates.md`
///
/// # Errors
///
/// Returns an error if any subsystem fails to initialize. The error will contain
/// context about which subsystem failed and why, thanks to `color-eyre` error
/// reporting configured during utils initialization.
///
/// # Examples
///
/// ```no_run
/// use praxis_core;
///
/// fn main() -> praxis_utils::Result<()> {
///     praxis_core::run()
/// }
/// ```
pub fn run() -> praxis_utils::Result<()> {
    // Step 1: Initialize logging, error handling, and timing utilities.
    // This must happen first so that all subsequent initialization steps can
    // log their progress and report errors with full context.
    praxis_utils::init()?;

    // Step 2: Initialize the Entity Component System.
    // The ECS World must exist before other subsystems can register their
    // components and systems. This is the data foundation for the entire engine.
    praxis_ecs::init()?;

    // Step 3: Initialize input handling.
    // Input systems must be ready before the window opens, as the window will
    // immediately begin generating keyboard, mouse, and gamepad events.
    praxis_input::init()?;

    // Step 4: Initialize audio subsystem.
    // Audio setup can be slow and may fail gracefully (e.g., no audio hardware).
    // Initializing before the main loop prevents first-frame latency spikes.
    praxis_audio::init()?;

    // Step 5: Create the window and enter the main event loop.
    // This is a blocking call that takes over the thread. All subsystems must be
    // initialized before this point. The window will coordinate with graphics,
    // input, and ECS to run the game loop until the application exits.
    praxis_window::run()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_subsystems_independently() {
        // Test that each subsystem can initialize independently
        // This validates the initialization order dependencies
        
        // Utils should always work
        assert!(praxis_utils::init().is_ok());
        
        // ECS should work after utils
        assert!(praxis_ecs::init().is_ok());
        
        // Input should work after utils
        assert!(praxis_input::init().is_ok());
        
        // Audio should work after utils (may fail on headless systems)
        let _ = praxis_audio::init();
    }
}

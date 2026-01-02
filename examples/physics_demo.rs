//! Physics demonstration with falling cubes, bouncing spheres, and a static ground plane.
//!
//! This example showcases the Praxis physics engine integration with the ECS architecture:
//! - Rigid body dynamics with Dynamic, Static, and Kinematic body types
//! - Collision detection and response using Rapier3D
//! - Fixed timestep physics simulation for deterministic behavior
//! - Transform synchronization between ECS and physics engine
//! - Multiple primitive shapes (cubes, spheres, ground plane)
//! - Different physical materials (bouncy vs. non-bouncy)
//!
//! Scene contents:
//! - Static ground plane (50x50 units)
//! - 5 falling cubes with varying initial positions
//! - 5 bouncing spheres with high restitution
//! - FPS camera for scene navigation
//!
//! Controls:
//! - WASD - Move camera horizontally
//! - Space/Left Ctrl - Move camera up/down
//! - Left Shift - Sprint (faster movement)
//! - Mouse - Look around (when cursor locked)
//! - ESC - Toggle cursor lock / Exit (when unlocked)
//!
//! Usage:
//! ```bash
//! cargo run --example physics_demo
//! ```

use praxis_ecs::{PerspectiveCameraBundle, Transform, World, IntoSystemConfigs};
use praxis_graphics::{DrawCommand, RenderCommands, RenderContext};
use praxis_input::{Action, InputMap, InputState};
use praxis_math::{Quat, Vec3, Mat4};
use praxis_physics::{
    Collider, PhysicsConfig, PhysicsTime, PhysicsWorld, PhysicsVelocity, RigidBody,
    Restitution, Friction, ContactEvents,
    physics_step_system, sync_physics_transforms_system,
    clear_collision_event_receivers, populate_collision_events,
};
use praxis_utils::{info, Result, FrameTimer};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey};
use winit::window::{CursorGrabMode, Window, WindowId};

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

/// Camera controller for FPS-style navigation
struct CameraController {
    move_speed: f32,
    sprint_multiplier: f32,
    mouse_sensitivity: f32,
    pitch: f32,
    yaw: f32,
    max_pitch: f32,
    camera_entity: Option<praxis_ecs::Entity>,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            sprint_multiplier: 2.0,
            mouse_sensitivity: 0.002,
            pitch: 0.0,
            yaw: std::f32::consts::PI,
            max_pitch: std::f32::consts::FRAC_PI_2 - 0.01,
            camera_entity: None,
        }
    }
}

impl CameraController {
    /// Update camera rotation from mouse delta
    fn update_rotation(&mut self, delta_x: f32, delta_y: f32) {
        // Step 1: Apply mouse sensitivity to convert raw pixel movement to radians
        // Mouse coordinates are in screen space (pixels), we scale to reasonable rotation speed
        self.yaw -= delta_x * self.mouse_sensitivity;
        self.pitch -= delta_y * self.mouse_sensitivity;
        
        // Step 2: Clamp pitch to prevent camera flipping upside down
        // Max pitch prevents the "gimbal lock" feeling when looking straight up/down
        self.pitch = self.pitch.clamp(-self.max_pitch, self.max_pitch);
    }

    /// Get the camera's rotation quaternion from pitch and yaw
    fn get_rotation(&self) -> Quat {
        // Step 1: Create yaw rotation (around Y axis - horizontal rotation)
        // Yaw controls which direction we're facing on the XZ plane
        let yaw_quat = Quat::from_rotation_y(self.yaw);
        
        // Step 2: Create pitch rotation (around X axis - vertical rotation)
        // Pitch controls looking up and down
        let pitch_quat = Quat::from_rotation_x(self.pitch);
        
        // Step 3: Combine rotations (yaw first, then pitch)
        // Order matters! Yaw * Pitch gives FPS-style camera
        // Pitch * Yaw would give flight-simulator style
        yaw_quat * pitch_quat
    }
}

/// Main application state
struct App {
    window: Option<Arc<Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
    cursor_locked: bool,
    last_frame_time: Option<Instant>,
    frame_timer: FrameTimer,
    camera_controller: CameraController,
    input_state: InputState,
    input_map: InputMap,
}

impl Default for App {
    fn default() -> Self {
        // Step 1: Set up input mappings for camera controls
        // InputMap binds Actions (semantic names) to KeyCodes (hardware keys)
        let mut input_map = InputMap::default();
        input_map.bind_key(&Action::new("forward"), KeyCode::KeyW);
        input_map.bind_key(&Action::new("backward"), KeyCode::KeyS);
        input_map.bind_key(&Action::new("left"), KeyCode::KeyA);
        input_map.bind_key(&Action::new("right"), KeyCode::KeyD);
        input_map.bind_key(&Action::new("up"), KeyCode::Space);
        input_map.bind_key(&Action::new("down"), KeyCode::ControlLeft);
        input_map.bind_key(&Action::new("sprint"), KeyCode::ShiftLeft);

        Self {
            window: None,
            world: None,
            render_context: None,
            cursor_locked: false,
            last_frame_time: None,
            frame_timer: FrameTimer::new(),
            camera_controller: CameraController::default(),
            input_state: InputState::default(),
            input_map,
        }
    }
}

impl App {
    /// Initialize the scene with physics objects
    fn init_scene(&mut self) -> Result<()> {
        // Step 1: Unwrap the world (it must exist at this point)
        let world = self.world.as_mut().expect("World not initialized");

        // ====================================================================
        // STEP 2: INITIALIZE PHYSICS RESOURCES
        // ====================================================================
        
        // PhysicsWorld wraps the Rapier3D physics engine and manages:
        // - Rigid body set (all physics bodies in the simulation)
        // - Collider set (all collision shapes)
        // - Physics pipeline (collision detection, constraint solving, integration)
        // - Mappings between ECS entities and Rapier handles
        world.insert_resource(PhysicsWorld::new());
        
        // PhysicsConfig contains global physics settings:
        // - Gravity: (0, -9.81, 0) = Earth gravity pulling down on Y axis
        // - Timestep: 1/60 second = 60Hz physics simulation rate
        // Fixed timestep ensures deterministic physics regardless of frame rate
        world.insert_resource(PhysicsConfig::default());
        
        // PhysicsTime accumulates frame delta times for fixed timestep integration
        // When accumulator >= timestep, we run one physics step
        world.insert_resource(PhysicsTime::new());
        
        // ContactEvents stores collision events (started, stopped) each frame
        // Gameplay systems can query this to react to collisions
        world.insert_resource(ContactEvents::new());

        // ====================================================================
        // STEP 3: CREATE CAMERA
        // ====================================================================
        
        // Step 3a: Position camera above the scene looking at the action
        // Starting position: (0, 10, 20) gives a good overview of the falling objects
        // Looking slightly down toward the origin where objects will land
        let camera_transform = Transform::from_xyz(0.0, 10.0, 20.0);

        // Step 3b: Spawn camera entity with perspective projection
        // PerspectiveCameraBundle includes:
        // - Camera component (marks as active camera)
        // - PerspectiveProjection (FOV, near/far planes)
        // - CameraMatrices (view and projection matrices for rendering)
        let camera_entity = world.spawn(PerspectiveCameraBundle::new_at(camera_transform));
        
        // Step 3c: Store camera entity reference for updates
        self.camera_controller.camera_entity = Some(camera_entity);

        // ====================================================================
        // STEP 4: CREATE GROUND PLANE (STATIC BODY)
        // ====================================================================
        
        // Static bodies never move and have infinite mass
        // Perfect for level geometry like floors, walls, terrain
        
        // Step 4a: Position ground at origin with no rotation
        let ground_transform = Transform::from_xyz(0.0, 0.0, 0.0);
        
        // Step 4b: Spawn ground entity with physics components
        world.spawn((
            ground_transform,
            RigidBody::Static,
            // Large flat box: 50 units wide (X), 0.5 units thick (Y), 50 units deep (Z)
            // Collider dimensions are half-extents, so total size is 100x1x100
            Collider::cuboid(50.0, 0.5, 50.0),
            // Medium friction prevents objects from sliding too much
            Friction::new(0.5),
            // No restitution = no bounce when objects hit the ground
            // Objects will land and stay put rather than bouncing forever
            Restitution::new(0.0),
        ));

        info!("Created static ground plane (100x1x100 units)");

        // ====================================================================
        // STEP 5: CREATE FALLING CUBES (DYNAMIC BODIES)
        // ====================================================================
        
        // Dynamic bodies are fully simulated - they respond to forces, gravity, and collisions
        // These cubes will fall under gravity and stack on the ground
        
        let cube_positions = [
            Vec3::new(-5.0, 10.0, 0.0),   // Left
            Vec3::new(-2.5, 12.0, 0.0),   // Left-center, higher
            Vec3::new(0.0, 15.0, 0.0),    // Center, highest
            Vec3::new(2.5, 12.0, 0.0),    // Right-center, higher
            Vec3::new(5.0, 10.0, 0.0),    // Right
        ];

        for (i, &position) in cube_positions.iter().enumerate() {
            // Step 5a: Create transform with initial position and slight rotation for visual variety
            // Small rotation around Y axis makes the scene less uniform
            let transform = Transform {
                translation: position,
                rotation: Quat::from_rotation_y(i as f32 * 0.3),
                scale: Vec3::ONE,
            };

            // Step 5b: Spawn dynamic cube with physics properties
            world.spawn((
                transform,
                RigidBody::Dynamic,
                // Cube collider: 1 unit on each side (2x2x2 total)
                Collider::cuboid(1.0, 1.0, 1.0),
                // Start with zero velocity - gravity will accelerate them
                PhysicsVelocity::default(),
                // Low restitution = cubes won't bounce much, they'll settle quickly
                Restitution::new(0.2),
                // Medium friction for realistic stacking behavior
                Friction::new(0.6),
            ));
        }

        info!("Created 5 falling cubes at varying heights");

        // ====================================================================
        // STEP 6: CREATE BOUNCING SPHERES (DYNAMIC BODIES)
        // ====================================================================
        
        // These spheres have high restitution, so they'll bounce energetically
        // They demonstrate elastic collisions
        
        let sphere_positions = [
            Vec3::new(-6.0, 20.0, -5.0),  // Back-left, very high
            Vec3::new(-3.0, 18.0, -5.0),  // Back left-center
            Vec3::new(0.0, 22.0, -5.0),   // Back center, highest
            Vec3::new(3.0, 18.0, -5.0),   // Back right-center
            Vec3::new(6.0, 20.0, -5.0),   // Back-right, very high
        ];

        for (i, &position) in sphere_positions.iter().enumerate() {
            // Step 6a: Create transform with initial position
            let transform = Transform::from_xyz(position.x, position.y, position.z);

            // Step 6b: Give each sphere a small random initial velocity for visual interest
            // Small sideways velocity makes the bouncing patterns more interesting
            let initial_velocity = PhysicsVelocity::linear(Vec3::new(
                (i as f32 - 2.0) * 0.5,  // Spread velocities: -1.0, -0.5, 0.0, 0.5, 1.0
                0.0,
                0.0,
            ));

            // Step 6c: Spawn bouncy sphere with high restitution
            world.spawn((
                transform,
                RigidBody::Dynamic,
                // Sphere collider: 0.5 unit radius (1.0 diameter)
                Collider::sphere(0.5),
                initial_velocity,
                // High restitution = very bouncy! Objects will bounce back almost to original height
                // 0.8 means 80% of kinetic energy is preserved in each bounce
                Restitution::new(0.8),
                // Low friction so spheres roll easily
                Friction::new(0.1),
            ));
        }

        info!("Created 5 bouncing spheres with high restitution");
        info!("Physics demo scene initialized successfully");

        Ok(())
    }

    /// Update camera position based on input
    fn update_camera(&mut self, delta_time: f32) {
        // Step 1: Get world and camera entity, early return if not initialized
        let Some(world) = &mut self.world else { return };
        let Some(camera_entity) = self.camera_controller.camera_entity else { return };

        // Step 2: Get mutable reference to camera transform
        // We use world.get_mut() because we need to modify the Transform component
        let Ok(mut transform) = world.get_mut::<Transform>(camera_entity) else { return };

        // ====================================================================
        // STEP 3: CALCULATE MOVEMENT DIRECTION IN WORLD SPACE
        // ====================================================================
        
        // Step 3a: Get camera rotation to determine which way is "forward"
        let rotation = self.camera_controller.get_rotation();
        
        // Step 3b: Calculate forward vector (where camera is looking)
        // Start with -Z (forward in right-handed coords) and rotate by camera orientation
        // This gives us the direction the camera is facing in world space
        let forward = rotation * Vec3::new(0.0, 0.0, -1.0);
        
        // Step 3c: Calculate right vector (perpendicular to forward)
        // Start with +X (right in right-handed coords) and rotate by camera orientation
        // This gives us the direction "to the right" from camera's perspective
        let right = rotation * Vec3::new(1.0, 0.0, 0.0);

        // ====================================================================
        // STEP 4: CALCULATE MOVEMENT DELTA FROM INPUT
        // ====================================================================
        
        // Step 4a: Determine movement speed (sprint or normal)
        // Sprint multiplier makes camera move faster when shift is held
        let speed = if self.input_state.is_action_active(&Action::new("sprint"), &self.input_map) {
            self.camera_controller.move_speed * self.camera_controller.sprint_multiplier
        } else {
            self.camera_controller.move_speed
        };

        // Step 4b: Accumulate movement from WASD keys
        // Each input contributes to movement in a specific direction
        let mut movement = Vec3::ZERO;
        
        // Forward/backward movement along the camera's looking direction
        if self.input_state.is_action_active(&Action::new("forward"), &self.input_map) {
            movement += forward;
        }
        if self.input_state.is_action_active(&Action::new("backward"), &self.input_map) {
            movement -= forward;
        }
        
        // Left/right strafing perpendicular to looking direction
        if self.input_state.is_action_active(&Action::new("left"), &self.input_map) {
            movement -= right;
        }
        if self.input_state.is_action_active(&Action::new("right"), &self.input_map) {
            movement += right;
        }
        
        // Vertical movement (world space, not camera relative)
        if self.input_state.is_action_active(&Action::new("up"), &self.input_map) {
            movement += Vec3::Y;
        }
        if self.input_state.is_action_active(&Action::new("down"), &self.input_map) {
            movement -= Vec3::Y;
        }

        // Step 4c: Normalize movement vector to prevent diagonal movement from being faster
        // Without normalization, moving forward+right would be sqrt(2) faster than just forward
        // Normalization ensures constant speed in all directions
        if movement.length_squared() > 0.0 {
            movement = movement.normalize();
        }

        // ====================================================================
        // STEP 5: APPLY MOVEMENT TO CAMERA TRANSFORM
        // ====================================================================
        
        // Step 5a: Scale movement by speed and delta time
        // delta_time ensures movement is frame-rate independent
        // At 60fps (dt=0.0167s), speed=5.0 → moves 0.083 units per frame
        // At 30fps (dt=0.0333s), speed=5.0 → moves 0.167 units per frame (same distance per second)
        transform.translation += movement * speed * delta_time;
        
        // Step 5b: Update camera rotation from controller
        transform.rotation = self.camera_controller.get_rotation();
    }

    /// Run physics simulation step
    fn update_physics(&mut self, delta_time: f32) {
        // Step 1: Get world reference, early return if not initialized
        let Some(world) = &mut self.world else { return };

        // ====================================================================
        // STEP 2: ACCUMULATE FRAME TIME FOR FIXED TIMESTEP
        // ====================================================================
        
        // Fixed timestep physics simulation: Run physics at constant rate (60Hz)
        // regardless of frame rate. This ensures deterministic behavior.
        //
        // The accumulator pattern:
        // - Add each frame's delta time to an accumulator
        // - While accumulator >= fixed_timestep, run one physics step and subtract timestep
        // - Remaining time in accumulator carries over to next frame
        //
        // Example at 60fps (16.67ms per frame):
        // Frame 1: accumulator = 16.67ms, run 1 step, remainder = 0ms
        // Frame 2: accumulator = 16.67ms, run 1 step, remainder = 0ms
        //
        // Example at 30fps (33.33ms per frame):
        // Frame 1: accumulator = 33.33ms, run 2 steps (16.67ms each), remainder = 0ms
        //
        // Example at 120fps (8.33ms per frame):
        // Frame 1: accumulator = 8.33ms, run 0 steps, remainder = 8.33ms
        // Frame 2: accumulator = 16.67ms (8.33 + 8.33), run 1 step, remainder = 0ms
        
        {
            let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
            physics_time.add(delta_time);
        }

        // ====================================================================
        // STEP 3: RUN PHYSICS SYSTEMS IN CORRECT ORDER
        // ====================================================================
        
        // The physics update pipeline must run in this exact order:
        //
        // 1. clear_collision_event_receivers: Clear previous frame's collision events
        //    This prevents events from accumulating across frames
        //
        // 2. sync_physics_transforms_system (pre-physics): 
        //    Copy ECS Transform changes to Rapier rigid bodies
        //    This allows kinematic bodies to move and dynamic bodies to be teleported
        //
        // 3. physics_step_system: 
        //    Run the Rapier physics simulation using fixed timestep
        //    This updates rigid body positions based on forces, gravity, and collisions
        //    May run 0, 1, or multiple times depending on accumulated time
        //
        // 4. sync_physics_transforms_system (post-physics):
        //    Copy Rapier rigid body positions back to ECS Transforms
        //    This makes physics simulation results visible to rendering and game logic
        //
        // 5. populate_collision_events:
        //    Distribute collision events to entities with CollisionEventReceiver components
        //    This allows game logic to react to collisions
        
        let mut schedule = praxis_ecs::Schedule::default();
        schedule.add_systems((
            clear_collision_event_receivers,
            sync_physics_transforms_system,
            physics_step_system,
            sync_physics_transforms_system,
            populate_collision_events,
        ).chain());

        // Execute all systems in order
        world.inner_mut().run_schedule(&mut schedule);
    }

    /// Render the current frame
    fn render(&mut self) -> Result<()> {
        // Step 1: Get references to render context and world
        let Some(render_context) = &mut self.render_context else {
            return Ok(());
        };
        let Some(world) = &self.world else {
            return Ok(());
        };

        // ====================================================================
        // STEP 2: QUERY CAMERA FOR VIEW/PROJECTION MATRICES
        // ====================================================================
        
        // Camera matrices transform 3D world coordinates to 2D screen coordinates
        // - View matrix: Camera position and orientation (where we're looking from)
        // - Projection matrix: Perspective projection (FOV, near/far planes)
        
        let mut view_matrix = Mat4::IDENTITY;
        let mut proj_matrix = Mat4::IDENTITY;

        for (camera_matrices, _camera) in world
            .query::<(&praxis_ecs::CameraMatrices, &praxis_ecs::Camera)>()
            .iter(world)
        {
            view_matrix = camera_matrices.view;
            proj_matrix = camera_matrices.projection;
            break; // Use first active camera
        }

        // ====================================================================
        // STEP 3: COLLECT DRAW COMMANDS FOR ALL RENDERABLE ENTITIES
        // ====================================================================
        
        // We need to render all entities that have:
        // - Transform: Position, rotation, scale in world space
        // - RigidBody: Physics body type (we render all physics objects)
        
        let mut draw_commands = Vec::new();

        for (entity, (transform, rigid_body)) in world
            .query::<(&Transform, &RigidBody)>()
            .iter(world)
        {
            // Step 3a: Compute model matrix from transform
            // Model matrix transforms from object space to world space
            // Combines translation, rotation, and scale
            let model_matrix = Mat4::from_scale_rotation_translation(
                transform.scale,
                transform.rotation,
                transform.translation,
            );

            // Step 3b: Choose color based on rigid body type
            // Visual feedback helps understand what's static vs dynamic
            // - Green: Static bodies (ground)
            // - Blue: Dynamic bodies (cubes, spheres)
            // - Yellow: Kinematic bodies (none in this demo, but supported)
            let color = match rigid_body {
                RigidBody::Static => Vec3::new(0.2, 0.8, 0.2),    // Green
                RigidBody::Dynamic => Vec3::new(0.3, 0.5, 1.0),   // Blue
                RigidBody::Kinematic => Vec3::new(1.0, 1.0, 0.3), // Yellow
            };

            // Step 3c: Create draw command with model matrix
            // In a full game, mesh_id would reference an actual mesh asset
            // For this demo, we rely on the renderer's debug visualization
            draw_commands.push(DrawCommand {
                mesh_id: 0, // Placeholder - would be actual mesh handle in full implementation
                model: model_matrix,
            });
        }

        // ====================================================================
        // STEP 4: SUBMIT DRAW COMMANDS TO RENDERER
        // ====================================================================
        
        // RenderCommands packages all data needed for one frame:
        // - View matrix: Where the camera is
        // - Projection matrix: How to project 3D to 2D
        // - Draw commands: What objects to render and where
        // - Lighting: Optional lighting data (None uses previous lighting)
        let commands = RenderCommands {
            view: view_matrix,
            proj: proj_matrix,
            draw_commands: &draw_commands,
            lighting: None,
        };

        // Render all objects in one batch
        render_context.render(&commands)?;

        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Step 1: Create window if it doesn't exist
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Praxis Physics Demo - Falling Cubes and Bouncing Spheres")
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Failed to create window"),
            );

            self.window = Some(window.clone());

            // Step 2: Initialize ECS world
            self.world = Some(World::new());

            // Step 3: Initialize render context asynchronously
            let render_context = pollster::block_on(async {
                RenderContext::new(window.clone())
                    .await
                    .expect("Failed to create render context")
            });

            self.render_context = Some(render_context);

            // Step 4: Initialize scene with physics objects
            self.init_scene()
                .expect("Failed to initialize scene");

            // Step 5: Lock cursor for FPS camera control
            if let Some(window) = &self.window {
                let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                window.set_cursor_visible(false);
                self.cursor_locked = true;
            }

            info!("Physics demo initialized - watch the objects fall and bounce!");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            // Handle window close request
            WindowEvent::CloseRequested => {
                info!("Close requested");
                event_loop.exit();
            }

            // Handle keyboard input
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state,
                        ..
                    },
                ..
            } => {
                // Update input state for camera movement
                if let Key::Named(NamedKey::Escape) = logical_key {
                    if state == ElementState::Pressed {
                        if self.cursor_locked {
                            // First ESC: unlock cursor
                            if let Some(window) = &self.window {
                                let _ = window.set_cursor_grab(CursorGrabMode::None);
                                window.set_cursor_visible(true);
                                self.cursor_locked = false;
                            }
                        } else {
                            // Second ESC: exit application
                            event_loop.exit();
                        }
                    }
                } else if let Key::Character(c) = &logical_key {
                    let key_str = c.as_str();
                    if let Some(keycode) = match key_str {
                        "w" => Some(KeyCode::KeyW),
                        "a" => Some(KeyCode::KeyA),
                        "s" => Some(KeyCode::KeyS),
                        "d" => Some(KeyCode::KeyD),
                        _ => None,
                    } {
                        match state {
                            ElementState::Pressed => self.input_state.press_key(keycode),
                            ElementState::Released => self.input_state.release_key(keycode),
                        }
                    }
                } else if let Key::Named(named) = logical_key {
                    let keycode = match named {
                        NamedKey::Space => Some(KeyCode::Space),
                        NamedKey::Control => Some(KeyCode::ControlLeft),
                        NamedKey::Shift => Some(KeyCode::ShiftLeft),
                        _ => None,
                    };
                    if let Some(keycode) = keycode {
                        match state {
                            ElementState::Pressed => self.input_state.press_key(keycode),
                            ElementState::Released => self.input_state.release_key(keycode),
                        }
                    }
                }
            }

            // Handle window resize
            WindowEvent::Resized(new_size) => {
                if let Some(render_context) = &mut self.render_context {
                    render_context.handle_resize(new_size.width, new_size.height);
                }
            }

            // Redraw requested - main render loop
            WindowEvent::RedrawRequested => {
                // Calculate delta time
                let now = Instant::now();
                let delta_time = if let Some(last_frame) = self.last_frame_time {
                    now.duration_since(last_frame).as_secs_f32()
                } else {
                    1.0 / 60.0 // First frame default
                };
                self.last_frame_time = Some(now);

                // Update frame timer for FPS tracking
                self.frame_timer.update();

                // Update camera from input
                self.update_camera(delta_time);

                // Update physics simulation
                self.update_physics(delta_time);

                // Render the frame
                if let Err(e) = self.render() {
                    eprintln!("Render error: {}", e);
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        // Handle mouse movement for camera rotation (only when cursor is locked)
        if self.cursor_locked {
            if let DeviceEvent::MouseMotion { delta } = event {
                self.camera_controller.update_rotation(delta.0 as f32, delta.1 as f32);
            }
        }
    }
}

fn main() -> Result<()> {
    // Step 1: Initialize engine subsystems
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_physics::init()?;

    info!("Starting physics demo...");

    // Step 2: Create event loop and run application
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Event loop error");

    Ok(())
}

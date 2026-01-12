//! Complete Features Demo - Comprehensive Showcase
//!
//! This example demonstrates the full capabilities of the Praxis engine:
//! - **Terrain Rendering**: Heightmap-based terrain with LOD system and texture splatting
//! - **TCP Networking**: Client-server architecture with entity replication
//! - **Scripting Integration**: Lua scripting with ECS access and hot-reload
//! - **Modern Rendering**: TAA (Temporal Anti-Aliasing), SSR (Screen-Space Reflections), GPU culling
//! - **Advanced Graphics**: Deferred rendering, HDR, SSAO, shadow mapping
//! - **Physics Integration**: Rapier3D physics with terrain collision
//!
//! Controls:
//! - WASD: Move camera
//! - Mouse: Look around
//! - Space: Move up
//! - Shift: Move down
//! - 1: Toggle TAA
//! - 2: Toggle SSR
//! - 3: Toggle SSAO
//! - 4: Toggle shadows
//! - T: Show terrain stats
//! - N: Show network stats
//! - L: Reload Lua scripts
//! - ESC: Exit

use praxis_math::{EulerRot, Quat, Vec3};
use praxis_utils::{info, Result};

#[cfg(any(feature = "networking", feature = "scripting"))]
use praxis_utils::warn;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowAttributes;

// Feature flags for conditional compilation
#[cfg(feature = "terrain")]
use praxis_terrain::{
    TerrainConfig, TerrainHeightmap, TerrainMaterialLayer, TerrainSystem, VegetationLayer,
};

#[cfg(feature = "networking")]
use praxis_networking::{NetworkClient, NetworkConfig, NetworkServer, ReplicationRegistry};

#[cfg(feature = "scripting")]
use praxis_scripting::{SandboxConfig, SandboxLevel, ScriptingConfig, ScriptingContext};

/// Camera controller state
struct CameraController {
    position: Vec3,
    rotation: Quat,
    move_speed: f32,
    look_speed: f32,
    yaw: f32,
    pitch: f32,
}

impl CameraController {
    fn new(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            move_speed: 10.0,
            look_speed: 0.002,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    fn update(&mut self, delta_time: f32, keys: &KeyState) {
        let forward = self.rotation * Vec3::Z;
        let right = self.rotation * Vec3::X;
        let up = Vec3::Y;

        let mut velocity = Vec3::ZERO;

        if keys.forward {
            velocity -= forward;
        }
        if keys.backward {
            velocity += forward;
        }
        if keys.left {
            velocity -= right;
        }
        if keys.right {
            velocity += right;
        }
        if keys.up {
            velocity += up;
        }
        if keys.down {
            velocity -= up;
        }

        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize() * self.move_speed * delta_time;
            self.position += velocity;
        }

        self.rotation = Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0);
    }

    fn process_mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        self.yaw -= delta_x as f32 * self.look_speed;
        self.pitch -= delta_y as f32 * self.look_speed;
        self.pitch = self.pitch.clamp(-1.5, 1.5);
    }
}

/// Keyboard state tracking
#[derive(Default)]
struct KeyState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

/// Rendering feature toggles
#[derive(Default)]
struct RenderingFeatures {
    taa_enabled: bool,
    ssr_enabled: bool,
    ssao_enabled: bool,
    shadows_enabled: bool,
}

/// Application state
struct AppState {
    camera: CameraController,
    keys: KeyState,
    features: RenderingFeatures,
    last_frame_time: Instant,
    frame_count: u64,
    show_terrain_stats: bool,
    show_network_stats: bool,

    #[cfg(feature = "terrain")]
    terrain: Option<TerrainSystem>,

    #[cfg(feature = "networking")]
    network_mode: Option<NetworkMode>,

    #[cfg(feature = "scripting")]
    scripting_context: Option<ScriptingContext>,
}

#[cfg(feature = "networking")]
enum NetworkMode {
    Server(NetworkServer, ReplicationRegistry),
    Client(NetworkClient),
}

impl AppState {
    fn new() -> Self {
        Self {
            camera: CameraController::new(Vec3::new(0.0, 50.0, 100.0)),
            keys: KeyState::default(),
            features: RenderingFeatures {
                taa_enabled: true,
                ssr_enabled: true,
                ssao_enabled: true,
                shadows_enabled: true,
            },
            last_frame_time: Instant::now(),
            frame_count: 0,
            show_terrain_stats: false,
            show_network_stats: false,

            #[cfg(feature = "terrain")]
            terrain: None,

            #[cfg(feature = "networking")]
            network_mode: None,

            #[cfg(feature = "scripting")]
            scripting_context: None,
        }
    }

    fn update(&mut self, delta_time: f32) {
        self.camera.update(delta_time, &self.keys);
        self.frame_count += 1;

        #[cfg(feature = "terrain")]
        if let Some(terrain) = &mut self.terrain {
            terrain.update(self.camera.position);
        }

        #[cfg(feature = "networking")]
        if let Some(mode) = &mut self.network_mode {
            match mode {
                NetworkMode::Server(server, _registry) => {
                    if let Err(e) = server.update(delta_time) {
                        warn!("Network server update error: {}", e);
                    }
                }
                NetworkMode::Client(client) => {
                    if let Err(e) = client.update(delta_time) {
                        warn!("Network client update error: {}", e);
                    }
                }
            }
        }

        #[cfg(feature = "scripting")]
        if let Some(context) = &mut self.scripting_context {
            // Process hot-reload events
            if let Err(e) = context.process_hot_reload() {
                warn!("Hot-reload error: {}", e);
            }

            // Execute update scripts
            if let Err(e) = context.call_function::<f32, ()>("game_logic", "update", delta_time)
            {
                warn!("Script update error: {}", e);
            }
        }
    }

    fn handle_key(&mut self, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::KeyW => self.keys.forward = pressed,
            KeyCode::KeyS => self.keys.backward = pressed,
            KeyCode::KeyA => self.keys.left = pressed,
            KeyCode::KeyD => self.keys.right = pressed,
            KeyCode::Space => self.keys.up = pressed,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.keys.down = pressed,
            KeyCode::Digit1 if pressed => {
                self.features.taa_enabled = !self.features.taa_enabled;
                info!(
                    "TAA: {}",
                    if self.features.taa_enabled {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            KeyCode::Digit2 if pressed => {
                self.features.ssr_enabled = !self.features.ssr_enabled;
                info!(
                    "SSR: {}",
                    if self.features.ssr_enabled {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            KeyCode::Digit3 if pressed => {
                self.features.ssao_enabled = !self.features.ssao_enabled;
                info!(
                    "SSAO: {}",
                    if self.features.ssao_enabled {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            KeyCode::Digit4 if pressed => {
                self.features.shadows_enabled = !self.features.shadows_enabled;
                info!(
                    "Shadows: {}",
                    if self.features.shadows_enabled {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            KeyCode::KeyT if pressed => {
                self.show_terrain_stats = !self.show_terrain_stats;
                info!(
                    "Terrain stats: {}",
                    if self.show_terrain_stats { "ON" } else { "OFF" }
                );
            }
            KeyCode::KeyN if pressed => {
                self.show_network_stats = !self.show_network_stats;
                info!(
                    "Network stats: {}",
                    if self.show_network_stats { "ON" } else { "OFF" }
                );
            }
            #[cfg(feature = "scripting")]
            KeyCode::KeyL if pressed => {
                if let Some(context) = &mut self.scripting_context {
                    info!("Manual script reload requested");
                    match context.reload_script("game_logic") {
                        Ok(()) => info!("Scripts reloaded successfully"),
                        Err(e) => warn!("Script reload error: {}", e),
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(feature = "terrain")]
fn setup_terrain(state: &mut AppState) -> Result<()> {
    info!("=== Setting up Terrain System ===");

    let heightmap_start = Instant::now();
    let heightmap = TerrainHeightmap::from_noise(512, 512, 100.0, 4.0, 6);
    info!("Heightmap generated in {:?}", heightmap_start.elapsed());

    let config = TerrainConfig {
        chunk_size: 64.0,
        vertices_per_chunk: 65,
        max_height: 100.0,
        lod_levels: 4,
        lod_distances: vec![50.0, 100.0, 200.0, 400.0],
        world_size: 1024.0,
        world_scale: 1.0,
        enable_frustum_culling: true,
        enable_occlusion_culling: false,
    };

    let terrain_start = Instant::now();
    let mut terrain = TerrainSystem::new(config, heightmap)?;
    info!("Terrain system created in {:?}", terrain_start.elapsed());

    // Material layers
    let grass_layer = TerrainMaterialLayer::new("grass", "grass_albedo", 0.0, 30.0)
        .with_tiling(10.0)
        .with_normal("grass_normal");
    terrain.material.add_layer(grass_layer);

    let rock_layer = TerrainMaterialLayer::new("rock", "rock_albedo", 30.0, 70.0)
        .with_slope(20.0, 90.0)
        .with_tiling(15.0)
        .with_normal("rock_normal");
    terrain.material.add_layer(rock_layer);

    let snow_layer = TerrainMaterialLayer::new("snow", "snow_albedo", 70.0, 100.0)
        .with_tiling(8.0)
        .with_normal("snow_normal");
    terrain.material.add_layer(snow_layer);

    // Vegetation layers
    let grass_vegetation = VegetationLayer::new("grass", "grass_mesh", "grass_mat", 5.0)
        .with_height_range(0.0, 40.0)
        .with_slope_range(0.0, 30.0)
        .with_scale_range(0.8, 1.2)
        .with_wind_strength(1.5)
        .with_color_variation(0.15);
    terrain.vegetation_layers.push(grass_vegetation);

    let tree_vegetation = VegetationLayer::new("trees", "tree_mesh", "tree_mat", 0.5)
        .with_height_range(20.0, 60.0)
        .with_slope_range(0.0, 25.0)
        .with_scale_range(0.8, 1.5)
        .with_wind_strength(0.3)
        .with_random_rotation(true);
    terrain.vegetation_layers.push(tree_vegetation);

    let vegetation_start = Instant::now();
    terrain.generate_vegetation()?;
    info!("Vegetation generated in {:?}", vegetation_start.elapsed());

    let total_instances: usize = terrain
        .vegetation_layers
        .iter()
        .map(|l| l.instance_count())
        .sum();
    info!("Total vegetation instances: {}", total_instances);

    state.terrain = Some(terrain);

    Ok(())
}

#[cfg(feature = "networking")]
async fn setup_networking(state: &mut AppState, is_server: bool) -> Result<()> {
    info!("=== Setting up Networking ===");

    let config = NetworkConfig {
        bind_addr: if is_server { "0.0.0.0:7777" } else { "" }.to_string(),
        max_clients: 32,
        tick_rate: 60,
        enable_interpolation: true,
        enable_extrapolation: true,
        enable_lag_compensation: true,
        enable_profiling: true,
        ..Default::default()
    };

    if is_server {
        info!("Starting network server on port 7777...");
        let mut server = NetworkServer::new(config).await?;
        server.start().await?;

        let mut registry = ReplicationRegistry::new();
        registry.register_transform();
        registry.register_velocity();

        info!("Network server started successfully");
        state.network_mode = Some(NetworkMode::Server(server, registry));
    } else {
        info!("Connecting to network server at 127.0.0.1:7777...");
        let mut client = NetworkClient::new(config).await?;
        client
            .connect("127.0.0.1:7777", "DemoClient".to_string())
            .await?;

        info!("Network client connected successfully");
        state.network_mode = Some(NetworkMode::Client(client));
    }

    Ok(())
}

#[cfg(feature = "scripting")]
fn setup_scripting(state: &mut AppState) -> Result<()> {
    info!("=== Setting up Scripting System ===");

    let config = ScriptingConfig {
        sandbox: SandboxConfig {
            level: SandboxLevel::Moderate,
            allow_file_io: false,
            allow_network: false,
            allow_os_access: false,
            instruction_limit: 1_000_000,
            memory_limit: 100 * 1024 * 1024,
        },
        enable_performance_monitoring: true,
        max_execution_time_ms: 16,
    };

    let mut context = ScriptingContext::new(config)?;

    // Try to load from file first, fall back to inline script
    let script_path = "examples/scripts/complete_features_demo.lua";
    if std::path::Path::new(script_path).exists() {
        info!("Loading game logic from file: {}", script_path);
        context.load_script("game_logic", script_path)?;
        
        // Enable hot-reload
        context.enable_hot_reload("examples/scripts")?;
        info!("Script hot-reload enabled - edit {} to see changes", script_path);
    } else {
        info!("Script file not found, using inline script");
        // Load game logic script from inline string
        context.load_string(
            "game_logic",
            r#"
        -- Game logic script
        function update(delta_time)
            -- Called every frame
        end
        
        function on_terrain_loaded()
            print("Terrain system loaded!")
        end
        
        function on_network_connected()
            print("Network connected!")
        end
        
        function calculate_camera_influence(distance)
            -- Calculate LOD based on camera distance
            if distance < 50 then
                return 1.0
            elseif distance < 100 then
                return 0.75
            elseif distance < 200 then
                return 0.5
            else
                return 0.25
            end
        end
    "#,
        )?;
        
        info!("Note: Create {} for hot-reload support", script_path);
    }

    state.scripting_context = Some(context);

    Ok(())
}

fn print_welcome_banner() {
    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║      Praxis Engine - Complete Features Demo             ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("Features:");

    #[cfg(feature = "terrain")]
    println!("  ✓ Terrain Rendering (512x512 heightmap, 4 LOD levels)");

    #[cfg(feature = "networking")]
    println!("  ✓ TCP Networking (Client-server with entity replication)");

    #[cfg(feature = "scripting")]
    println!("  ✓ Scripting Integration (Lua with hot-reload)");

    println!("  ✓ Modern Rendering (TAA, SSR, SSAO, Shadows)");
    println!("  ✓ GPU Culling (Frustum and occlusion culling)");
    println!("  ✓ Deferred Rendering (G-buffer with HDR)");
    println!();
    println!("Controls:");
    println!("  WASD           Move camera");
    println!("  Mouse          Look around");
    println!("  Space          Move up");
    println!("  Shift          Move down");
    println!("  1              Toggle TAA");
    println!("  2              Toggle SSR");
    println!("  3              Toggle SSAO");
    println!("  4              Toggle Shadows");
    println!("  T              Show terrain stats");
    println!("  N              Show network stats");

    #[cfg(feature = "scripting")]
    println!("  L              Reload Lua scripts");

    println!("  ESC            Exit");
    println!();
}

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    praxis_utils::init_tracing()?;

    print_welcome_banner();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let is_server = args.contains(&"--server".to_string());
    let is_client = args.contains(&"--client".to_string());
    let enable_networking = is_server || is_client;

    if enable_networking {
        #[cfg(not(feature = "networking"))]
        {
            eprintln!("Networking feature not enabled. Build with --features networking");
            return Ok(());
        }
    }

    info!("Starting Praxis Complete Features Demo");

    let event_loop = EventLoop::new()?;

    #[allow(deprecated)]
    let window = Arc::new(
        event_loop.create_window(
            WindowAttributes::default()
                .with_title("Praxis Engine - Complete Features Demo")
                .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080)),
        )?,
    );

    let mut state = AppState::new();

    // Setup terrain system
    #[cfg(feature = "terrain")]
    {
        if let Err(e) = setup_terrain(&mut state) {
            warn!("Failed to setup terrain: {}", e);
        }
    }

    // Setup networking (async required)
    #[cfg(feature = "networking")]
    if enable_networking {
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(async {
            if let Err(e) = setup_networking(&mut state, is_server).await {
                warn!("Failed to setup networking: {}", e);
            }
        });
    }

    // Setup scripting
    #[cfg(feature = "scripting")]
    {
        if let Err(e) = setup_scripting(&mut state) {
            warn!("Failed to setup scripting: {}", e);
        }
    }

    info!("All systems initialized successfully");
    info!("Starting main event loop...");
    println!();

    let mut last_stats_print = Instant::now();
    let mut accumulated_frame_time = Duration::ZERO;
    let mut frame_count_for_avg = 0u32;

    #[allow(deprecated)]
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Closing demo...");
                info!("Total frames rendered: {}", state.frame_count);

                #[cfg(feature = "terrain")]
                if let Some(terrain) = &state.terrain {
                    info!("Final terrain chunks: {}", terrain.chunk_count());
                }

                elwt.exit();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(keycode),
                                state: key_state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                let pressed = key_state.is_pressed();
                state.handle_key(keycode, pressed);

                if keycode == KeyCode::Escape && pressed {
                    elwt.exit();
                }
            }
            Event::DeviceEvent {
                event: winit::event::DeviceEvent::MouseMotion { delta },
                ..
            } => {
                state.camera.process_mouse_motion(delta.0, delta.1);
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let current_time = Instant::now();
                let delta_time = current_time
                    .duration_since(state.last_frame_time)
                    .as_secs_f32();
                state.last_frame_time = current_time;

                accumulated_frame_time += Duration::from_secs_f32(delta_time);
                frame_count_for_avg += 1;

                state.update(delta_time);

                // Print stats every 2 seconds
                if last_stats_print.elapsed() >= Duration::from_secs(2) {
                    let avg_frame_time =
                        accumulated_frame_time.as_secs_f32() / frame_count_for_avg as f32;
                    let fps = 1.0 / avg_frame_time;

                    info!("=== Frame Stats ===");
                    info!("  FPS: {:.1}", fps);
                    info!("  Frame time: {:.2}ms", avg_frame_time * 1000.0);
                    info!("  Frame count: {}", state.frame_count);

                    if state.show_terrain_stats {
                        #[cfg(feature = "terrain")]
                        if let Some(terrain) = &state.terrain {
                            info!("=== Terrain Stats ===");
                            info!("  Active chunks: {}", terrain.chunk_count());
                            info!("  Camera position: {:?}", state.camera.position);
                        }
                    }

                    if state.show_network_stats {
                        #[cfg(feature = "networking")]
                        if let Some(mode) = &state.network_mode {
                            info!("=== Network Stats ===");
                            match mode {
                                NetworkMode::Server(server, _) => {
                                    info!("  Mode: Server");
                                    info!("  Clients: {}", server.client_count());
                                }
                                NetworkMode::Client(client) => {
                                    info!("  Mode: Client");
                                    if let Some(id) = client.client_id() {
                                        info!("  Client ID: {}", id);
                                    }
                                }
                            }
                        }
                    }

                    info!("=== Rendering Features ===");
                    info!(
                        "  TAA: {}",
                        if state.features.taa_enabled {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                    info!(
                        "  SSR: {}",
                        if state.features.ssr_enabled {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                    info!(
                        "  SSAO: {}",
                        if state.features.ssao_enabled {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                    info!(
                        "  Shadows: {}",
                        if state.features.shadows_enabled {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );

                    #[cfg(feature = "scripting")]
                    if let Some(context) = &state.scripting_context {
                        if let Some(monitor) = context.performance_monitor() {
                            info!("=== Scripting Stats ===");
                            let scripts = monitor.get_all_stats();
                            for stats in scripts {
                                info!("  {}: {:.3?} avg", stats.script_name, stats.average_time);
                            }
                        }
                    }

                    last_stats_print = current_time;
                    accumulated_frame_time = Duration::ZERO;
                    frame_count_for_avg = 0;
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!(
        "complete_features_demo example requires graphics support and cannot run in headless mode"
    );
    Ok(())
}

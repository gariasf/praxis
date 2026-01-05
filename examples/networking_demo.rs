//! Networking demonstration showing client-server architecture,
//! entity replication, interpolation, and network profiling.

use praxis_ecs::{Schedule, Transform, World};
use praxis_math::{Quat, Vec3};
use praxis_networking::{
    ExtrapolationSystem, InterpolationBuffer, InterpolationSystem, LagCompensation,
    LagCompensationSystem, NetworkClient, NetworkConfig, NetworkExtrapolation, NetworkId,
    NetworkInterpolation, NetworkProfiler, NetworkServer, Replicated, ReplicatedTransform,
    ReplicatedVelocity, ReplicationRegistry,
};
use std::time::{Duration, Instant};

/// Demonstrates server setup and entity replication.
async fn run_server() -> color_eyre::Result<()> {
    println!("=== Starting Network Server ===\n");

    // Configure server
    let config = NetworkConfig {
        bind_addr: "0.0.0.0:7777".to_string(),
        max_clients: 32,
        tick_rate: 60,
        enable_interpolation: true,
        enable_extrapolation: true,
        enable_lag_compensation: true,
        enable_profiling: true,
        ..Default::default()
    };

    println!("Server configuration:");
    println!("  Bind address: {}", config.bind_addr);
    println!("  Max clients: {}", config.max_clients);
    println!("  Tick rate: {} Hz", config.tick_rate);
    println!("  Interpolation: {}", config.enable_interpolation);
    println!("  Lag compensation: {}", config.enable_lag_compensation);
    println!();

    // Create server
    let mut server = NetworkServer::new(config).await?;
    server.start().await?;

    println!("Server started successfully!");
    println!("Listening on port 7777...\n");

    // Create ECS world
    let mut world = World::new();

    // Register components for replication
    let mut registry = ReplicationRegistry::new();
    registry.register_transform();
    registry.register_velocity();

    // Spawn some replicated entities
    println!("Spawning replicated entities:");

    for i in 0..5 {
        let entity = world.spawn((
            NetworkId::new(i + 1),
            Replicated::new().with_priority(200),
            Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0),
            ReplicatedTransform::new(
                Vec3::new(i as f32 * 2.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
            ReplicatedVelocity::new(Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO),
        ));

        println!("  Entity {} (Network ID: {})", entity.index(), i + 1);
    }

    println!();

    // Initialize lag compensation
    let mut lag_comp = LagCompensation::new(1000);

    // Server loop
    let tick_duration = Duration::from_secs_f32(1.0 / 60.0);
    let mut last_tick = Instant::now();
    let mut tick_count = 0u64;

    println!("Server running. Press Ctrl+C to stop.\n");

    for _ in 0..600 {
        // Run for 10 seconds
        let now = Instant::now();
        let delta = now.duration_since(last_tick).as_secs_f32();

        if delta >= tick_duration.as_secs_f32() {
            tick_count += 1;
            last_tick = now;

            // Update server
            server.update(delta)?;

            // Update lag compensation history
            LagCompensationSystem::update(&mut lag_comp, 1, world.inner());

            // Print status every second
            if tick_count % 60 == 0 {
                println!(
                    "Tick {}: {} clients connected",
                    tick_count,
                    server.client_count()
                );
            }
        }

        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    println!("\nShutting down server...");
    server.stop().await?;
    println!("Server stopped.");

    Ok(())
}

/// Demonstrates client setup and connection.
async fn run_client() -> color_eyre::Result<()> {
    println!("=== Starting Network Client ===\n");

    // Wait a bit for server to start
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Configure client
    let config = NetworkConfig {
        enable_interpolation: true,
        enable_extrapolation: true,
        interpolation_delay_ms: 100,
        enable_profiling: true,
        ..Default::default()
    };

    println!("Client configuration:");
    println!(
        "  Interpolation delay: {} ms",
        config.interpolation_delay_ms
    );
    println!("  Extrapolation enabled: {}", config.enable_extrapolation);
    println!();

    // Create client
    let mut client = NetworkClient::new(config).await?;

    println!("Connecting to server at 127.0.0.1:7777...");
    client
        .connect("127.0.0.1:7777", "DemoClient".to_string())
        .await?;

    // Wait for connection
    tokio::time::sleep(Duration::from_millis(100)).await;

    if let Some(client_id) = client.client_id() {
        println!("Connected! Client ID: {}\n", client_id);
    } else {
        println!("Waiting for connection acceptance...\n");
    }

    // Create ECS world for client
    let mut world = World::new();

    // Create schedule with interpolation/extrapolation systems
    let mut schedule = Schedule::default();

    // Client loop
    let tick_duration = Duration::from_secs_f32(1.0 / 60.0);
    let mut last_tick = Instant::now();
    let mut tick_count = 0u64;

    println!("Client running. Press Ctrl+C to stop.\n");

    for _ in 0..600 {
        // Run for 10 seconds
        let now = Instant::now();
        let delta = now.duration_since(last_tick).as_secs_f32();

        if delta >= tick_duration.as_secs_f32() {
            tick_count += 1;
            last_tick = now;

            // Update client
            client.update(delta)?;

            // Send ping every second
            if tick_count % 60 == 0 {
                client.send_ping()?;
                println!("Tick {}: Sent ping to server", tick_count);
            }

            // Update interpolation/extrapolation
            schedule.run(world.inner_mut());
        }

        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    println!("\nDisconnecting from server...");
    client
        .disconnect("Client shutting down".to_string())
        .await?;
    println!("Client disconnected.");

    Ok(())
}

/// Demonstrates network profiler usage.
fn demonstrate_profiler() {
    println!("\n=== Network Profiler Demo ===\n");

    let profiler = NetworkProfiler::new();

    println!("Simulating network activity...\n");

    // Simulate sending and receiving data
    for i in 0..100 {
        // Simulate varying packet sizes
        let send_size = 100 + (i * 10) % 500;
        let recv_size = 150 + (i * 15) % 400;

        profiler.record_sent(send_size);
        profiler.record_received(recv_size);

        // Simulate varying latency
        let latency = 30.0 + ((i as f32 * 0.1).sin() * 20.0);
        profiler.record_latency(latency);

        // Update profiler
        profiler.update(0.016); // ~60 FPS
    }

    // Get statistics
    let stats = profiler.get_stats();

    println!("Network Statistics:");
    println!("------------------");
    println!("Bandwidth:");
    println!("  Total sent: {} bytes", stats.bandwidth.bytes_sent);
    println!("  Total received: {} bytes", stats.bandwidth.bytes_received);
    println!("  Send rate: {:.2} bytes/sec", stats.bandwidth.send_rate);
    println!(
        "  Receive rate: {:.2} bytes/sec",
        stats.bandwidth.receive_rate
    );
    println!(
        "  Peak send rate: {:.2} bytes/sec",
        stats.bandwidth.peak_send_rate
    );
    println!(
        "  Peak receive rate: {:.2} bytes/sec",
        stats.bandwidth.peak_receive_rate
    );
    println!();
    println!("Latency:");
    println!("  Current RTT: {:.2} ms", stats.latency.rtt_ms);
    println!("  Min RTT: {:.2} ms", stats.latency.min_rtt_ms);
    println!("  Max RTT: {:.2} ms", stats.latency.max_rtt_ms);
    println!("  Avg RTT: {:.2} ms", stats.latency.avg_rtt_ms);
    println!("  Jitter: {:.2} ms", stats.latency.jitter_ms);
    println!();
}

/// Demonstrates interpolation system.
fn demonstrate_interpolation() {
    println!("\n=== Interpolation Demo ===\n");

    let mut world = World::new();

    // Spawn entity with interpolation
    let entity = world.spawn((
        NetworkId::new(100),
        ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
        NetworkInterpolation::new(100.0),
        InterpolationBuffer::default(),
    ));

    println!("Created interpolated entity (ID: {})", entity.index());
    println!("Interpolation delay: 100 ms");
    println!();

    // Simulate receiving snapshots
    println!("Simulating received snapshots:");
    println!("  t=0ms: position (0, 0, 0)");
    println!("  t=100ms: position (10, 0, 0)");
    println!("  t=200ms: position (20, 0, 0)");
    println!();

    println!("Interpolation creates smooth movement between snapshots.");
    println!("At t=150ms with 100ms delay, position interpolates to (5, 0, 0)");
    println!();
}

/// Demonstrates lag compensation.
fn demonstrate_lag_compensation() {
    println!("\n=== Lag Compensation Demo ===\n");

    let mut lag_comp = LagCompensation::new(1000);
    let mut world = bevy_ecs::world::World::new();

    // Spawn target entity
    let entity = world.spawn((
        NetworkId::new(1),
        ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
    ));

    println!("Target entity spawned at (0, 0, 0)");

    // Record historical positions
    for i in 0..10 {
        let timestamp = i * 100;
        let position = Vec3::new(i as f32, 0.0, 0.0);

        // Update entity position
        if let Some(mut transform) = world.get_mut::<ReplicatedTransform>(entity) {
            transform.translation = position;
        }

        // Record snapshot
        lag_comp.record_snapshot(1, timestamp, &world);

        println!("  t={}ms: position ({}, 0, 0)", timestamp, i);
    }

    println!();
    println!("When client fires at t=500ms but has 200ms latency:");
    println!("Server rewinds to t=300ms to validate hit from client's perspective");
    println!("Target was at position (3, 0, 0) at that time");
    println!();

    // Demonstrate rewind
    match lag_comp.rewind_to_client_time(1, 300, &mut world) {
        Ok(rewind_state) => {
            println!("Successfully rewound world to t=300ms");

            // Check entity position
            if let Some(transform) = world.get::<ReplicatedTransform>(entity) {
                println!(
                    "Entity position after rewind: ({}, {}, {})",
                    transform.translation.x, transform.translation.y, transform.translation.z
                );
            }

            // Restore state
            lag_comp.restore_state(rewind_state, &mut world);
            println!("World state restored");
        }
        Err(e) => {
            println!("Rewind failed: {}", e);
        }
    }

    println!();
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // Initialize error handling
    color_eyre::install()?;

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("╔═══════════════════════════════════════════╗");
    println!("║  Praxis Networking Demonstration         ║");
    println!("╚═══════════════════════════════════════════╝\n");

    // Demonstrate different features
    demonstrate_profiler();
    demonstrate_interpolation();
    demonstrate_lag_compensation();

    println!("\n=== Client-Server Demo ===\n");
    println!("To run client-server demo:");
    println!("  1. Run server: cargo run --example networking_demo -- server");
    println!("  2. Run client: cargo run --example networking_demo -- client");
    println!();

    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "server" => {
                run_server().await?;
            }
            "client" => {
                run_client().await?;
            }
            _ => {
                println!("Unknown argument. Use 'server' or 'client'");
            }
        }
    }

    println!("Demo complete!");

    Ok(())
}

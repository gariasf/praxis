//! Networking system for the Praxis engine.
//!
//! This crate provides comprehensive networking capabilities including:
//! - Client-server architecture with reliable and unreliable channels
//! - Entity replication with automatic component serialization
//! - Interpolation and extrapolation for smooth remote entity movement
//! - Lag compensation for fair gameplay
//! - Network profiler showing bandwidth and latency metrics
//!
//! # Architecture
//!
//! The networking system consists of several key components:
//!
//! - **Transport Layer**: Low-level socket management with TCP for reliability and UDP for speed
//! - **Message Protocol**: Serialization/deserialization of network messages
//! - **Replication System**: Tracks and synchronizes entity state across clients
//! - **Interpolation/Extrapolation**: Smooths remote entity movement
//! - **Lag Compensation**: Server-side hit detection and client prediction
//! - **Network Profiler**: Real-time monitoring of network performance
//!
//! # Client-Server Architecture
//!
//! ## Overview
//!
//! The networking system uses a **client-server architecture** where:
//! - The **server** is the authoritative source of truth for game state
//! - **Clients** send inputs to the server and receive state updates
//! - The server validates all actions and replicates state to clients
//!
//! This architecture prevents cheating by ensuring clients cannot directly modify game state.
//! All gameplay logic runs on the server, and clients simply render the replicated state.
//!
//! ## Benefits
//!
//! - **Security**: Server validates all actions, preventing cheating
//! - **Consistency**: Single source of truth ensures all clients see the same game state
//! - **Scalability**: Server can manage many clients efficiently
//!
//! ## Trade-offs
//!
//! - **Latency**: Client actions must round-trip to server, adding delay
//! - **Server load**: Server must process all game logic for all clients
//! - **Client prediction needed**: Clients must predict movements to hide latency
//!
//! # TCP vs UDP: When to Use Each
//!
//! ## TCP (Transmission Control Protocol)
//!
//! **Use for**: Critical data that must arrive reliably and in order
//!
//! ### Characteristics
//! - **Reliable delivery**: Guaranteed to arrive, automatic retransmission on packet loss
//! - **Ordered**: Packets arrive in the exact order they were sent
//! - **Connection-oriented**: Maintains a persistent connection with handshake
//! - **Flow control**: Automatically adjusts send rate based on network conditions
//! - **Higher overhead**: Extra bandwidth for acknowledgments and sequence numbers
//!
//! ### Best for
//! - Player chat messages
//! - Login/authentication
//! - Game state changes (spawn/destroy entities)
//! - Critical events (player death, score updates)
//! - File transfers
//!
//! ### Drawbacks
//! - **Head-of-line blocking**: Lost packets block all subsequent packets until retransmitted
//! - **Higher latency**: Retransmission delays can cause noticeable lag
//! - **Not suitable for real-time position updates**: Old retransmitted data is useless
//!
//! ## UDP (User Datagram Protocol)
//!
//! **Use for**: Time-sensitive data where freshness matters more than reliability
//!
//! ### Characteristics
//! - **Unreliable delivery**: No guarantee packets arrive (5-10% loss on poor connections)
//! - **Unordered**: Packets may arrive out of order
//! - **Connectionless**: Just fire-and-forget datagrams
//! - **Low overhead**: Minimal protocol overhead, maximum throughput
//! - **Low latency**: No retransmission delays
//!
//! ### Best for
//! - Player position updates (sent every frame, old data is worthless)
//! - Animation states
//! - Voice chat (prefer current audio over old audio)
//! - Frequent, non-critical updates
//! - Ping/pong latency measurements
//!
//! ### Drawbacks
//! - **Packet loss**: Must tolerate missing data
//! - **Duplication**: Same packet may arrive multiple times
//! - **No congestion control**: Can flood network if not careful
//!
//! ## Hybrid Approach (What This Engine Uses)
//!
//! Most modern multiplayer games use **both** TCP and UDP:
//!
//! - **TCP for reliability**: Important game events, chat, initial connection
//! - **UDP for performance**: High-frequency position updates, input commands
//!
//! This gives you the best of both worlds: reliability when you need it,
//! low latency when you don't.
//!
//! # Example
//!
//! ```rust,ignore
//! use praxis_networking::{NetworkServer, NetworkConfig, ReplicationRegistry};
//! use praxis_ecs::World;
//!
//! # tokio_test::block_on(async {
//! let mut world = World::new();
//! let config = NetworkConfig::default();
//! let mut server = NetworkServer::new(config).await.unwrap();
//!
//! // Register components for replication
//! let mut registry = ReplicationRegistry::new();
//! registry.register_transform();
//! registry.register_velocity();
//!
//! server.start().await.unwrap();
//! # });
//! ```

mod client;
mod components;
mod interpolation;
mod lag_compensation;
mod message;
mod profiler;
mod replication;
mod server;
mod transport;

pub use client::{ClientState, ConnectionError, NetworkClient};
pub use components::{
    ClientPredicted, NetworkExtrapolation, NetworkId, NetworkInterpolation, NetworkOwner,
    Replicated, ReplicatedTransform, ReplicatedVelocity, ServerAuthoritative,
};
pub use interpolation::{
    ExtrapolationSystem, InterpolationBuffer, InterpolationSystem, SnapshotBuffer,
};
pub use lag_compensation::{
    ClientStateHistory, HistoryBuffer, LagCompensation, LagCompensationSystem,
};
pub use message::{ComponentData, EntitySnapshot, MessageType, NetworkMessage, ReplicationMessage};
pub use profiler::{BandwidthMetrics, LatencyMetrics, NetworkProfiler, ProfilerStats};
pub use replication::{
    ComponentSerializer, EntityReplicator, ReplicationRegistry, ReplicationSystem,
};
pub use server::{ClientConnection, NetworkServer, ServerState};
pub use transport::{NetworkTransport, SocketAddr, TcpTransport, TransportConfig, UdpTransport};

use praxis_utils::Result;

/// Network configuration settings.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Server bind address (e.g., "0.0.0.0:7777")
    pub bind_addr: String,

    /// Maximum number of connected clients
    pub max_clients: usize,

    /// Network tick rate in Hz
    pub tick_rate: u32,

    /// Enable interpolation for remote entities
    pub enable_interpolation: bool,

    /// Enable extrapolation for remote entities
    pub enable_extrapolation: bool,

    /// Interpolation delay in milliseconds
    pub interpolation_delay_ms: u32,

    /// Enable lag compensation
    pub enable_lag_compensation: bool,

    /// Maximum lag compensation history in milliseconds
    pub lag_compensation_history_ms: u32,

    /// Maximum packet size in bytes
    pub max_packet_size: usize,

    /// Enable network profiling
    pub enable_profiling: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:7777".to_string(),
            max_clients: 32,
            tick_rate: 60,
            enable_interpolation: true,
            enable_extrapolation: true,
            interpolation_delay_ms: 100,
            enable_lag_compensation: true,
            lag_compensation_history_ms: 1000,
            max_packet_size: 1400,
            enable_profiling: true,
        }
    }
}

/// Initializes the networking system.
pub fn init() -> Result<()> {
    tracing::info!("Initializing networking system");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.bind_addr, "0.0.0.0:7777");
        assert_eq!(config.max_clients, 32);
        assert_eq!(config.tick_rate, 60);
        assert!(config.enable_interpolation);
        assert!(config.enable_extrapolation);
    }

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}

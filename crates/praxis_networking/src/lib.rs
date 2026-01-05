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
//! # Example
//!
//! ```rust,no_run
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

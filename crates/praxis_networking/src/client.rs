//! Network client implementation.

use crate::{
    NetworkConfig, NetworkMessage, NetworkProfiler, NetworkTransport, TcpTransport,
    TransportConfig, UdpTransport,
};
use parking_lot::RwLock;
use praxis_utils::Result;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Client connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    /// Client is disconnected
    Disconnected,
    /// Client is attempting to connect
    Connecting,
    /// Client is connected and active
    Connected,
    /// Client is disconnecting
    Disconnecting,
}

/// Connection error types.
#[derive(Debug, Clone)]
pub enum ConnectionError {
    /// Connection was rejected by server
    Rejected(String),
    /// Connection timed out
    Timeout,
    /// Network error
    NetworkError(String),
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected(reason) => write!(f, "Connection rejected: {reason}"),
            Self::Timeout => write!(f, "Connection timeout"),
            Self::NetworkError(err) => write!(f, "Network error: {err}"),
        }
    }
}

impl std::error::Error for ConnectionError {}

/// Network client.
pub struct NetworkClient {
    /// Client configuration
    config: NetworkConfig,

    /// Current client state
    state: Arc<RwLock<ClientState>>,

    /// Assigned client ID from server
    client_id: Arc<RwLock<Option<u64>>>,

    /// Server address
    server_address: Arc<RwLock<Option<SocketAddr>>>,

    /// Current client tick
    current_tick: Arc<AtomicU64>,

    /// Server tick (last known)
    server_tick: Arc<AtomicU64>,

    /// TCP transport
    tcp_transport: Option<Arc<TcpTransport>>,

    /// UDP transport
    udp_transport: Option<Arc<UdpTransport>>,

    /// Network profiler
    profiler: Arc<RwLock<Option<NetworkProfiler>>>,
}

impl NetworkClient {
    /// Creates a new network client.
    pub async fn new(config: NetworkConfig) -> Result<Self> {
        tracing::info!("Creating network client");

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            client_id: Arc::new(RwLock::new(None)),
            server_address: Arc::new(RwLock::new(None)),
            current_tick: Arc::new(AtomicU64::new(0)),
            server_tick: Arc::new(AtomicU64::new(0)),
            tcp_transport: None,
            udp_transport: None,
            profiler: Arc::new(RwLock::new(None)),
        })
    }

    /// Connects to a server.
    pub async fn connect(&mut self, server_addr: &str, client_name: String) -> Result<()> {
        tracing::info!("Connecting to server at {}", server_addr);

        *self.state.write() = ClientState::Connecting;

        // Parse server address
        let addr: SocketAddr = server_addr
            .parse()
            .map_err(|e| color_eyre::eyre::eyre!("Invalid server address: {}", e))?;

        *self.server_address.write() = Some(addr);

        // Initialize transports
        let transport_config = TransportConfig::default();
        let tcp = TcpTransport::new(transport_config.clone()).await?;
        let udp = UdpTransport::new(transport_config).await?;

        udp.receive_loop(self.config.max_packet_size).await?;

        self.tcp_transport = Some(Arc::new(tcp));
        self.udp_transport = Some(Arc::new(udp));

        // Initialize profiler if enabled
        if self.config.enable_profiling {
            *self.profiler.write() = Some(NetworkProfiler::new());
        }

        // Send connect message
        let connect_msg = NetworkMessage::Connect {
            protocol_version: 1,
            client_name,
        };

        self.send_reliable(&connect_msg)?;

        tracing::info!("Connection request sent to {}", addr);
        Ok(())
    }

    /// Disconnects from the server.
    pub async fn disconnect(&mut self, reason: String) -> Result<()> {
        tracing::info!("Disconnecting from server: {}", reason);

        *self.state.write() = ClientState::Disconnecting;

        // Send disconnect message
        let disconnect_msg = NetworkMessage::Disconnect { reason };
        self.send_reliable(&disconnect_msg)?;

        // Clean up
        *self.client_id.write() = None;
        *self.server_address.write() = None;

        *self.state.write() = ClientState::Disconnected;

        tracing::info!("Disconnected from server");
        Ok(())
    }

    /// Updates the client (should be called every tick).
    pub fn update(&mut self, delta_time: f32) -> Result<()> {
        let state = *self.state.read();
        if state == ClientState::Disconnected {
            return Ok(());
        }

        // Increment tick counter
        self.current_tick.fetch_add(1, Ordering::SeqCst);

        // Update profiler
        if let Some(profiler) = self.profiler.write().as_mut() {
            profiler.update(delta_time);
        }

        // Process incoming messages
        self.process_messages()?;

        Ok(())
    }

    /// Processes incoming network messages.
    fn process_messages(&self) -> Result<()> {
        // Process TCP messages
        if let Some(tcp) = &self.tcp_transport {
            while let Some((addr, data)) = tcp.receive() {
                self.handle_message(addr, &data)?;
            }
        }

        // Process UDP messages
        if let Some(udp) = &self.udp_transport {
            while let Some((addr, data)) = udp.receive() {
                self.handle_message(addr, &data)?;
            }
        }

        Ok(())
    }

    /// Handles a received message.
    fn handle_message(&self, addr: SocketAddr, data: &[u8]) -> Result<()> {
        if let Some(profiler) = self.profiler.write().as_mut() {
            profiler.record_received(data.len());
        }

        let message = NetworkMessage::deserialize(data)?;

        match message {
            NetworkMessage::ConnectionAccepted {
                client_id,
                server_tick,
            } => {
                self.handle_connection_accepted(client_id, server_tick)?;
            }
            NetworkMessage::ConnectionRejected { reason } => {
                self.handle_connection_rejected(reason)?;
            }
            NetworkMessage::Pong { timestamp } => {
                self.handle_pong(timestamp)?;
            }
            NetworkMessage::Replication(replication) => {
                self.handle_replication(replication)?;
            }
            NetworkMessage::CommandAck { tick } => {
                self.handle_command_ack(tick)?;
            }
            _ => {
                tracing::warn!("Unhandled message type from {}", addr);
            }
        }

        Ok(())
    }

    /// Handles connection acceptance.
    fn handle_connection_accepted(&self, client_id: u64, server_tick: u64) -> Result<()> {
        tracing::info!("Connection accepted! Client ID: {}", client_id);

        *self.client_id.write() = Some(client_id);
        self.server_tick.store(server_tick, Ordering::SeqCst);
        *self.state.write() = ClientState::Connected;

        Ok(())
    }

    /// Handles connection rejection.
    fn handle_connection_rejected(&self, reason: String) -> Result<()> {
        tracing::warn!("Connection rejected: {}", reason);
        *self.state.write() = ClientState::Disconnected;
        Ok(())
    }

    /// Handles pong response.
    fn handle_pong(&self, timestamp: u64) -> Result<()> {
        // Calculate RTT
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let rtt = now.saturating_sub(timestamp);

        if let Some(profiler) = self.profiler.write().as_mut() {
            profiler.record_latency(rtt as f32);
        }

        tracing::trace!("RTT: {} ms", rtt);
        Ok(())
    }

    /// Handles replication message.
    fn handle_replication(&self, replication: crate::ReplicationMessage) -> Result<()> {
        tracing::trace!("Received replication update: tick {}", replication.tick);
        self.server_tick.store(replication.tick, Ordering::SeqCst);

        // Process entity updates and destroyed entities
        // This would integrate with the ECS world

        Ok(())
    }

    /// Handles command acknowledgment.
    fn handle_command_ack(&self, tick: u64) -> Result<()> {
        tracing::trace!("Command acknowledged for tick {}", tick);
        Ok(())
    }

    /// Sends a reliable message to the server.
    fn send_reliable(&self, message: &NetworkMessage) -> Result<()> {
        let data = message.serialize()?;

        if let Some(profiler) = self.profiler.write().as_mut() {
            profiler.record_sent(data.len());
        }

        if let Some(addr) = *self.server_address.read() {
            if let Some(tcp) = &self.tcp_transport {
                tcp.send_reliable(addr, &data)?;
            }
        }

        Ok(())
    }

    /// Sends an unreliable message to the server.
    fn send_unreliable(&self, message: &NetworkMessage) -> Result<()> {
        let data = message.serialize()?;

        if let Some(profiler) = self.profiler.write().as_mut() {
            profiler.record_sent(data.len());
        }

        if let Some(addr) = *self.server_address.read() {
            if let Some(udp) = &self.udp_transport {
                udp.send_unreliable(addr, &data)?;
            }
        }

        Ok(())
    }

    /// Sends a ping to measure latency.
    pub fn send_ping(&self) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let ping = NetworkMessage::Ping { timestamp };
        self.send_unreliable(&ping)?;

        Ok(())
    }

    /// Gets the current client state.
    pub fn state(&self) -> ClientState {
        *self.state.read()
    }

    /// Gets the client ID (if connected).
    pub fn client_id(&self) -> Option<u64> {
        *self.client_id.read()
    }

    /// Gets the current client tick.
    pub fn current_tick(&self) -> u64 {
        self.current_tick.load(Ordering::SeqCst)
    }

    /// Gets the last known server tick.
    pub fn server_tick(&self) -> u64 {
        self.server_tick.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let config = NetworkConfig::default();
        let client = NetworkClient::new(config).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_client_state() {
        let config = NetworkConfig::default();
        let client = NetworkClient::new(config).await.unwrap();
        assert_eq!(client.state(), ClientState::Disconnected);
    }

    #[test]
    fn test_connection_error_display() {
        let error = ConnectionError::Rejected("Server full".to_string());
        assert!(error.to_string().contains("rejected"));

        let timeout = ConnectionError::Timeout;
        assert!(timeout.to_string().contains("timeout"));
    }
}

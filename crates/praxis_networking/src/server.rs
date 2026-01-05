//! Network server implementation.

use crate::{
    NetworkConfig, NetworkMessage, NetworkProfiler, NetworkTransport, ReplicationSystem,
    TcpTransport, TransportConfig, UdpTransport,
};
use dashmap::DashMap;
use parking_lot::RwLock;
use praxis_utils::Result;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Server connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Server is stopped
    Stopped,
    /// Server is starting up
    Starting,
    /// Server is running and accepting connections
    Running,
    /// Server is shutting down
    Stopping,
}

/// Information about a connected client.
#[derive(Debug, Clone)]
pub struct ClientConnection {
    /// Unique client ID
    pub client_id: u64,

    /// Client's socket address
    pub address: SocketAddr,

    /// Client name/username
    pub name: String,

    /// Time of last received message (milliseconds)
    pub last_activity_ms: u64,

    /// Round-trip time in milliseconds
    pub rtt_ms: f32,
}

/// Network server.
pub struct NetworkServer {
    /// Server configuration
    config: NetworkConfig,

    /// Current server state
    state: Arc<RwLock<ServerState>>,

    /// Connected clients
    clients: Arc<DashMap<u64, ClientConnection>>,

    /// Next client ID
    next_client_id: Arc<AtomicU64>,

    /// Current server tick
    current_tick: Arc<AtomicU64>,

    /// TCP transport
    tcp_transport: Option<Arc<TcpTransport>>,

    /// UDP transport
    udp_transport: Option<Arc<UdpTransport>>,

    /// Replication system
    replication: Arc<RwLock<Option<ReplicationSystem>>>,

    /// Network profiler
    profiler: Arc<RwLock<Option<NetworkProfiler>>>,
}

impl NetworkServer {
    /// Creates a new network server.
    pub async fn new(config: NetworkConfig) -> Result<Self> {
        tracing::info!("Creating network server with config: {:?}", config);

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(ServerState::Stopped)),
            clients: Arc::new(DashMap::new()),
            next_client_id: Arc::new(AtomicU64::new(1)),
            current_tick: Arc::new(AtomicU64::new(0)),
            tcp_transport: None,
            udp_transport: None,
            replication: Arc::new(RwLock::new(None)),
            profiler: Arc::new(RwLock::new(None)),
        })
    }

    /// Starts the server.
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting network server");

        *self.state.write() = ServerState::Starting;

        // Initialize transports
        let transport_config = TransportConfig {
            tcp_port: 7777,
            udp_port: 7778,
            buffer_size: self.config.max_packet_size,
            timeout_seconds: 30,
        };

        let tcp = TcpTransport::new(transport_config.clone()).await?;
        let udp = UdpTransport::new(transport_config).await?;

        udp.receive_loop(self.config.max_packet_size).await?;

        self.tcp_transport = Some(Arc::new(tcp));
        self.udp_transport = Some(Arc::new(udp));

        // Initialize replication system
        *self.replication.write() = Some(ReplicationSystem::new());

        // Initialize profiler if enabled
        if self.config.enable_profiling {
            *self.profiler.write() = Some(NetworkProfiler::new());
        }

        *self.state.write() = ServerState::Running;

        tracing::info!("Network server started successfully");
        Ok(())
    }

    /// Stops the server.
    pub async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping network server");

        *self.state.write() = ServerState::Stopping;

        // Disconnect all clients
        self.clients.clear();

        *self.state.write() = ServerState::Stopped;

        tracing::info!("Network server stopped");
        Ok(())
    }

    /// Updates the server (should be called every tick).
    pub fn update(&mut self, delta_time: f32) -> Result<()> {
        let state = *self.state.read();
        if state != ServerState::Running {
            return Ok(());
        }

        // Increment tick counter
        let tick = self.current_tick.fetch_add(1, Ordering::SeqCst) + 1;

        // Update profiler
        if let Some(profiler) = self.profiler.write().as_mut() {
            profiler.update(delta_time);
        }

        // Process incoming messages
        self.process_messages()?;

        // Send replication updates
        if let Some(_replication) = self.replication.read().as_ref() {
            // Replication logic would go here
            tracing::trace!("Tick {}: Processing replication", tick);
        }

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
            NetworkMessage::Connect {
                protocol_version,
                client_name,
            } => {
                self.handle_connect(addr, protocol_version, client_name)?;
            }
            NetworkMessage::Disconnect { reason } => {
                self.handle_disconnect(addr, reason)?;
            }
            NetworkMessage::Ping { timestamp } => {
                self.handle_ping(addr, timestamp)?;
            }
            NetworkMessage::ClientCommand { tick, command_data } => {
                self.handle_client_command(addr, tick, command_data)?;
            }
            _ => {
                tracing::warn!("Unhandled message type from {}", addr);
            }
        }

        Ok(())
    }

    /// Handles client connection request.
    fn handle_connect(
        &self,
        addr: SocketAddr,
        _protocol_version: u32,
        client_name: String,
    ) -> Result<()> {
        tracing::info!("Client {} connecting from {}", client_name, addr);

        let client_id = self.next_client_id.fetch_add(1, Ordering::SeqCst);

        let client = ClientConnection {
            client_id,
            address: addr,
            name: client_name,
            last_activity_ms: 0,
            rtt_ms: 0.0,
        };

        self.clients.insert(client_id, client);

        // Send acceptance message
        let response = NetworkMessage::ConnectionAccepted {
            client_id,
            server_tick: self.current_tick.load(Ordering::SeqCst),
        };

        self.send_reliable(addr, &response)?;

        Ok(())
    }

    /// Handles client disconnection.
    fn handle_disconnect(&self, addr: SocketAddr, reason: String) -> Result<()> {
        tracing::info!("Client disconnecting from {}: {}", addr, reason);

        // Find and remove client
        for entry in self.clients.iter() {
            if entry.value().address == addr {
                self.clients.remove(entry.key());
                break;
            }
        }

        Ok(())
    }

    /// Handles ping message.
    fn handle_ping(&self, addr: SocketAddr, timestamp: u64) -> Result<()> {
        let response = NetworkMessage::Pong { timestamp };
        self.send_unreliable(addr, &response)?;
        Ok(())
    }

    /// Handles client command.
    fn handle_client_command(
        &self,
        addr: SocketAddr,
        tick: u64,
        _command_data: Vec<u8>,
    ) -> Result<()> {
        tracing::trace!("Received client command from {} for tick {}", addr, tick);

        // Send acknowledgment
        let ack = NetworkMessage::CommandAck { tick };
        self.send_reliable(addr, &ack)?;

        Ok(())
    }

    /// Sends a reliable message to a client.
    fn send_reliable(&self, addr: SocketAddr, message: &NetworkMessage) -> Result<()> {
        let data = message.serialize()?;

        if let Some(profiler) = self.profiler.write().as_mut() {
            profiler.record_sent(data.len());
        }

        if let Some(tcp) = &self.tcp_transport {
            tcp.send_reliable(addr, &data)?;
        }

        Ok(())
    }

    /// Sends an unreliable message to a client.
    fn send_unreliable(&self, addr: SocketAddr, message: &NetworkMessage) -> Result<()> {
        let data = message.serialize()?;

        if let Some(profiler) = self.profiler.write().as_mut() {
            profiler.record_sent(data.len());
        }

        if let Some(udp) = &self.udp_transport {
            udp.send_unreliable(addr, &data)?;
        }

        Ok(())
    }

    /// Gets the current server state.
    pub fn state(&self) -> ServerState {
        *self.state.read()
    }

    /// Gets the number of connected clients.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Gets the current tick.
    pub fn current_tick(&self) -> u64 {
        self.current_tick.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let config = NetworkConfig::default();
        let server = NetworkServer::new(config).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_server_state() {
        let config = NetworkConfig::default();
        let server = NetworkServer::new(config).await.unwrap();
        assert_eq!(server.state(), ServerState::Stopped);
    }

    #[test]
    fn test_server_state_enum() {
        assert_eq!(ServerState::Stopped, ServerState::Stopped);
        assert_ne!(ServerState::Stopped, ServerState::Running);
    }
}

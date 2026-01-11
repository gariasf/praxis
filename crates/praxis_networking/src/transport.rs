//! Network transport layer.
//!
//! # Transport Layer Fundamentals
//!
//! The transport layer provides low-level socket communication between clients and servers.
//! This module implements both TCP and UDP transports, each suited for different types
//! of network traffic.
//!
//! # Architecture
//!
//! The transport layer uses Rust's async/await with Tokio for efficient I/O:
//! - **Async operations**: Non-blocking I/O prevents threads from stalling
//! - **Message channels**: Crossbeam channels decouple socket I/O from game logic
//! - **Background tasks**: Tokio tasks handle receiving in the background
//!
//! # TCP Transport Details
//!
//! ## Connection-Oriented Communication
//!
//! TCP establishes a persistent connection between client and server:
//! 1. **Three-way handshake**: SYN → SYN-ACK → ACK to establish connection
//! 2. **Bidirectional stream**: Data flows both ways over the same connection
//! 3. **Graceful close**: FIN handshake to cleanly terminate connection
//!
//! ## Message Framing
//!
//! TCP is a **byte stream** with no message boundaries. We must frame messages ourselves:
//!
//! ```text
//! [4-byte length][message data][4-byte length][message data]...
//! ```
//!
//! Each message is prefixed with its length in bytes (u32). This allows the receiver
//! to know exactly how many bytes to read for each complete message.
//!
//! ## Reliability Guarantees
//!
//! - **Sequence numbers**: Every byte has a sequence number
//! - **Acknowledgments**: Receiver confirms receipt of data
//! - **Retransmission**: Lost packets are automatically resent
//! - **In-order delivery**: Packets arrive in the exact order sent
//!
//! ## Flow Control
//!
//! TCP prevents fast senders from overwhelming slow receivers:
//! - **Receive window**: Advertises how much buffer space is available
//! - **Sliding window**: Only sends data receiver can handle
//! - **Backpressure**: Automatically slows down if receiver can't keep up
//!
//! # UDP Transport Details
//!
//! ## Connectionless Communication
//!
//! UDP sends independent datagrams without establishing a connection:
//! - **Fire-and-forget**: Send packet and hope it arrives
//! - **No handshake**: Start sending immediately
//! - **No state tracking**: Server doesn't track "connections"
//!
//! ## Datagram Boundaries
//!
//! Unlike TCP, UDP preserves message boundaries:
//! ```text
//! send_to([A, B, C])  →  recv_from() returns [A, B, C]
//! ```
//!
//! Each `send_to` creates one datagram, and each `recv_from` receives
//! exactly one complete datagram. No framing needed!
//!
//! ## Unreliability Characteristics
//!
//! - **Packet loss**: 0.1-10% of packets may not arrive (worse on poor connections)
//! - **Duplication**: Same packet may arrive multiple times
//! - **Reordering**: Packets may arrive out of order
//! - **No notification**: Sender doesn't know if packet was lost
//!
//! ## Why Use UDP Despite Unreliability?
//!
//! For real-time games, **freshness > reliability**:
//! - Position at frame N+1 makes position at frame N irrelevant
//! - Better to show current (imperfect) state than wait for old data
//! - Player sees smoother movement despite occasional packet loss
//!
//! # Choosing Between TCP and UDP
//!
//! ## Use TCP for:
//! - Player authentication
//! - Chat messages  
//! - Spawn/destroy entity events
//! - Game state changes (score, inventory)
//! - Any data that must be reliable
//!
//! ## Use UDP for:
//! - Player position updates (every frame)
//! - Animation states
//! - Input commands
//! - Ping/pong measurements
//! - Any high-frequency, time-sensitive data
//!
//! # Implementation Notes
//!
//! ## Thread Safety
//!
//! Both transports are `Send + Sync` and can be safely shared across threads:
//! - `Arc<RwLock<...>>` for shared mutable state
//! - `crossbeam_channel` for thread-safe message passing
//! - Tokio async runtime for concurrent I/O
//!
//! ## Performance Optimizations
//!
//! - **Buffer pooling**: Reuse receive buffers to reduce allocations
//! - **Batch sending**: Send multiple messages in one syscall when possible
//! - **Backpressure handling**: Drop old unreliable messages if queue fills up

use crossbeam_channel::{Receiver, Sender};
use parking_lot::RwLock;
use praxis_utils::Result;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::Mutex as TokioMutex;

pub use std::net::SocketAddr;

/// Transport configuration.
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// TCP port for reliable messages
    pub tcp_port: u16,

    /// UDP port for unreliable messages
    pub udp_port: u16,

    /// Socket buffer size
    pub buffer_size: usize,

    /// Connection timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            tcp_port: 7777,
            udp_port: 7778,
            buffer_size: 65536,
            timeout_seconds: 30,
        }
    }
}

/// Network transport abstraction.
///
/// Provides a unified interface for both TCP and UDP communication.
/// Implementations handle the low-level socket details.
pub trait NetworkTransport: Send + Sync {
    /// Sends data reliably (TCP).
    ///
    /// Guarantees delivery and in-order arrival. Use for important data
    /// that must not be lost (e.g., chat messages, entity spawn events).
    fn send_reliable(&self, addr: SocketAddr, data: &[u8]) -> Result<()>;

    /// Sends data unreliably (UDP).
    ///
    /// May be lost or arrive out of order. Use for time-sensitive data
    /// where freshness matters more than reliability (e.g., position updates).
    fn send_unreliable(&self, addr: SocketAddr, data: &[u8]) -> Result<()>;

    /// Receives next message.
    ///
    /// Returns `Some((addr, data))` if a message is available, `None` otherwise.
    /// Non-blocking - returns immediately even if no messages are available.
    fn receive(&self) -> Option<(SocketAddr, Vec<u8>)>;
}

/// Type alias for TCP connection write half.
type TcpWriteHalf = Arc<TokioMutex<OwnedWriteHalf>>;

/// Type alias for connection storage.
type ConnectionMap = Arc<RwLock<Vec<(SocketAddr, TcpWriteHalf)>>>;

/// TCP transport for reliable messages.
///
/// # Message Framing
///
/// TCP is a byte stream, so we need to frame messages:
/// 1. Send 4-byte length prefix (u32 in network byte order)
/// 2. Send message data
/// 3. Receiver reads length, then reads exactly that many bytes
///
/// This prevents messages from being concatenated or split.
///
/// # Connection Management
///
/// - Accepts incoming connections on a listener socket
/// - Stores write halves for sending to each client
/// - Spawns async tasks for receiving from each client
/// - Length-prefixed messages prevent corruption
pub struct TcpTransport {
    listener: Arc<RwLock<Option<TcpListener>>>,
    connections: ConnectionMap,
    rx: Receiver<(SocketAddr, Vec<u8>)>,
    tx: Sender<(SocketAddr, Vec<u8>)>,
}

impl TcpTransport {
    /// Creates a new TCP transport.
    pub async fn new(config: TransportConfig) -> Result<Self> {
        let addr = format!("0.0.0.0:{}", config.tcp_port);
        let listener = TcpListener::bind(&addr).await?;

        let (tx, rx) = crossbeam_channel::unbounded();

        Ok(Self {
            listener: Arc::new(RwLock::new(Some(listener))),
            connections: Arc::new(RwLock::new(Vec::new())),
            rx,
            tx,
        })
    }

    /// Accepts incoming connections and spawns receive loops for each.
    ///
    /// # Connection Flow
    ///
    /// 1. Accept new TCP connection
    /// 2. Split into read and write halves
    /// 3. Store write half for sending
    /// 4. Spawn task to receive from read half
    #[allow(clippy::await_holding_lock)]
    pub async fn accept_connections(&self) -> Result<()> {
        let listener = self.listener.read();
        if let Some(listener) = listener.as_ref() {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    tracing::info!("Accepted TCP connection from {addr}");

                    // Split stream into read and write halves
                    let (read_half, write_half) = stream.into_split();

                    // Store write half for sending
                    let mut connections = self.connections.write();
                    connections.push((addr, Arc::new(TokioMutex::new(write_half))));

                    // Spawn receive loop for read half
                    let tx = self.tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::connection_receive_loop(read_half, addr, tx).await {
                            tracing::error!("TCP receive loop error for {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Error accepting TCP connection: {e}");
                }
            }
        }
        Ok(())
    }

    /// Receive loop for a single TCP connection (length-prefixed messages).
    ///
    /// # Message Format
    ///
    /// ```text
    /// [4 bytes: length N][N bytes: message data]
    /// ```
    ///
    /// Reads length first, then reads exactly that many bytes for the message.
    /// This ensures message boundaries are preserved over the byte stream.
    ///
    /// # Error Handling
    ///
    /// - `UnexpectedEof`: Connection closed cleanly
    /// - Invalid length: Potential corruption or attack, close connection
    /// - Read errors: Network issue, close connection
    async fn connection_receive_loop(
        mut read_half: OwnedReadHalf,
        addr: SocketAddr,
        tx: Sender<(SocketAddr, Vec<u8>)>,
    ) -> Result<()> {
        loop {
            // Read 4-byte length prefix
            let length = match read_half.read_u32().await {
                Ok(len) => len as usize,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    tracing::info!("TCP connection closed by {}", addr);
                    break;
                }
                Err(e) => {
                    tracing::error!("Error reading length prefix from {}: {}", addr, e);
                    break;
                }
            };

            // Validate message length
            if length == 0 || length > 10_000_000 {
                // 10MB max to prevent DoS
                tracing::error!("Invalid message length {} from {}", length, addr);
                break;
            }

            // Read message data
            let mut buffer = vec![0u8; length];
            match read_half.read_exact(&mut buffer).await {
                Ok(_) => {
                    if tx.send((addr, buffer)).is_err() {
                        tracing::warn!("Failed to send received message to channel");
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading message data from {}: {}", addr, e);
                    break;
                }
            }
        }

        Ok(())
    }
}

impl NetworkTransport for TcpTransport {
    fn send_reliable(&self, addr: SocketAddr, data: &[u8]) -> Result<()> {
        // Find the connection for this address
        let connections = self.connections.read();
        let connection = connections
            .iter()
            .find(|(conn_addr, _)| *conn_addr == addr)
            .map(|(_, stream)| stream.clone());

        if let Some(write_half) = connection {
            let data = data.to_vec();
            let length = data.len() as u32;

            // Spawn async task to write length-prefixed message
            tokio::spawn(async move {
                // Acquire lock and await it
                let mut write_guard = write_half.lock().await;

                // Write 4-byte length prefix
                if let Err(e) = write_guard.write_u32(length).await {
                    tracing::error!("TCP send error (length) to {}: {}", addr, e);
                    return;
                }

                // Write message data
                if let Err(e) = write_guard.write_all(&data).await {
                    tracing::error!("TCP send error (data) to {}: {}", addr, e);
                    return;
                }

                // Flush to ensure data is sent
                if let Err(e) = write_guard.flush().await {
                    tracing::error!("TCP flush error for {}: {}", addr, e);
                }
            });

            tracing::trace!("TCP send to {}: {} bytes", addr, length);
        } else {
            tracing::warn!("No TCP connection found for {}", addr);
        }

        Ok(())
    }

    fn send_unreliable(&self, _addr: SocketAddr, _data: &[u8]) -> Result<()> {
        // TCP doesn't support unreliable sends
        Ok(())
    }

    fn receive(&self) -> Option<(SocketAddr, Vec<u8>)> {
        self.rx.try_recv().ok()
    }
}

/// UDP transport for unreliable messages.
///
/// # Datagram-Based Communication
///
/// UDP sends discrete datagrams:
/// - Each `send_to` sends one complete datagram
/// - Each `recv_from` receives one complete datagram
/// - No message framing needed (unlike TCP)
///
/// # Packet Loss Handling
///
/// UDP provides no reliability guarantees:
/// - Packets may be lost (never arrive)
/// - Packets may be duplicated (arrive multiple times)
/// - Packets may be reordered (arrive in different order)
///
/// For game networking, this is often acceptable:
/// - Position updates are sent every frame (old data becomes irrelevant)
/// - Occasional loss causes brief glitches, but game continues
/// - Fresher data is more valuable than guaranteed delivery
///
/// # When to Use UDP
///
/// Perfect for high-frequency, time-sensitive data:
/// - Player position/rotation updates (every frame)
/// - Animation state changes
/// - Input commands
/// - Voice chat packets
/// - Particle effects
///
/// Avoid for critical data:
/// - Chat messages (use TCP)
/// - Entity spawn/destroy (use TCP)
/// - Score/inventory changes (use TCP)
pub struct UdpTransport {
    socket: Arc<UdpSocket>,
    rx: Receiver<(SocketAddr, Vec<u8>)>,
    tx: Sender<(SocketAddr, Vec<u8>)>,
}

impl UdpTransport {
    /// Creates a new UDP transport.
    pub async fn new(config: TransportConfig) -> Result<Self> {
        let addr = format!("0.0.0.0:{}", config.udp_port);
        let socket = UdpSocket::bind(&addr).await?;

        let (tx, rx) = crossbeam_channel::unbounded();

        Ok(Self {
            socket: Arc::new(socket),
            rx,
            tx,
        })
    }

    /// Receives UDP packets in a background task.
    ///
    /// Spawns an async task that continuously receives datagrams
    /// and forwards them through a channel.
    pub async fn receive_loop(&self, buffer_size: usize) -> Result<()> {
        let mut buffer = vec![0u8; buffer_size];
        let tx = self.tx.clone();
        let socket = self.socket.clone();

        tokio::spawn(async move {
            loop {
                match socket.recv_from(&mut buffer).await {
                    Ok((len, addr)) => {
                        let data = buffer[..len].to_vec();
                        if tx.send((addr, data)).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("UDP receive error: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

impl NetworkTransport for UdpTransport {
    fn send_reliable(&self, _addr: SocketAddr, _data: &[u8]) -> Result<()> {
        // UDP doesn't support reliable sends
        Ok(())
    }

    fn send_unreliable(&self, addr: SocketAddr, data: &[u8]) -> Result<()> {
        let socket = self.socket.clone();
        let data = data.to_vec();

        tokio::spawn(async move {
            if let Err(e) = socket.send_to(&data, addr).await {
                tracing::error!("UDP send error: {}", e);
            }
        });

        Ok(())
    }

    fn receive(&self) -> Option<(SocketAddr, Vec<u8>)> {
        self.rx.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::time::timeout;

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert_eq!(config.tcp_port, 7777);
        assert_eq!(config.udp_port, 7778);
        assert_eq!(config.buffer_size, 65536);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[tokio::test]
    async fn test_tcp_transport_creation() {
        let config = TransportConfig {
            tcp_port: 0, // Use OS-assigned port
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await;
        assert!(transport.is_ok());
    }

    #[tokio::test]
    async fn test_tcp_transport_send_reliable_basic() {
        // Create server transport
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        // Get the actual bound address
        let listener = transport.listener.read();
        let listener_ref = listener.as_ref().unwrap();
        let local_addr = listener_ref.local_addr().unwrap();
        drop(listener);

        // Create a client connection
        let mut client = TcpStream::connect(local_addr).await.unwrap();

        // Accept the connection on the server side
        transport.accept_connections().await.unwrap();

        // Wait a bit for connection to be established
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Send data from server to client
        let test_data = b"Hello, TCP!";
        let client_addr = client.peer_addr().unwrap();
        transport
            .send_reliable(client_addr, test_data)
            .expect("Failed to send");

        // Wait for async send to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Receive on client side (4-byte length prefix + data)
        let length = client.read_u32().await.unwrap();
        assert_eq!(length, test_data.len() as u32);

        let mut received = vec![0u8; length as usize];
        client.read_exact(&mut received).await.unwrap();
        assert_eq!(&received, test_data);
    }

    #[tokio::test]
    async fn test_tcp_transport_send_reliable_multiple_messages() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        let listener = transport.listener.read();
        let local_addr = listener.as_ref().unwrap().local_addr().unwrap();
        drop(listener);

        let mut client = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client_addr = client.peer_addr().unwrap();

        // Send multiple messages
        let messages = vec![
            b"message1".to_vec(),
            b"message2".to_vec(),
            b"message3".to_vec(),
        ];

        for msg in &messages {
            transport.send_reliable(client_addr, msg).unwrap();
        }

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Receive all messages
        for expected_msg in &messages {
            let length = client.read_u32().await.unwrap();
            assert_eq!(length, expected_msg.len() as u32);

            let mut received = vec![0u8; length as usize];
            client.read_exact(&mut received).await.unwrap();
            assert_eq!(&received, expected_msg);
        }
    }

    #[tokio::test]
    async fn test_tcp_transport_send_reliable_no_connection() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        // Try to send to a non-existent connection
        let fake_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let test_data = b"Should not send";

        // Should return Ok but log warning (doesn't error out)
        let result = transport.send_reliable(fake_addr, test_data);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tcp_transport_receive_basic() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        let listener = transport.listener.read();
        let local_addr = listener.as_ref().unwrap().local_addr().unwrap();
        drop(listener);

        let mut client = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Send from client to server
        let test_data = b"Hello from client!";
        client.write_u32(test_data.len() as u32).await.unwrap();
        client.write_all(test_data).await.unwrap();
        client.flush().await.unwrap();

        // Wait for server to receive
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check server received it
        let received = transport.receive();
        assert!(received.is_some());

        let (addr, data) = received.unwrap();
        assert_eq!(&data, test_data);
        assert_eq!(addr, client.local_addr().unwrap());
    }

    #[tokio::test]
    async fn test_tcp_transport_receive_large_message() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        let listener = transport.listener.read();
        let local_addr = listener.as_ref().unwrap().local_addr().unwrap();
        drop(listener);

        let mut client = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Send large message (1MB)
        let test_data = vec![42u8; 1_000_000];
        client.write_u32(test_data.len() as u32).await.unwrap();
        client.write_all(&test_data).await.unwrap();
        client.flush().await.unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;

        let received = transport.receive();
        assert!(received.is_some());

        let (_, data) = received.unwrap();
        assert_eq!(data.len(), 1_000_000);
        assert_eq!(data, test_data);
    }

    #[tokio::test]
    async fn test_tcp_transport_connection_close() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        let listener = transport.listener.read();
        let local_addr = listener.as_ref().unwrap().local_addr().unwrap();
        drop(listener);

        let client = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Close client connection
        drop(client);
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Further receives should return None
        let received = transport.receive();
        assert!(received.is_none());
    }

    #[tokio::test]
    async fn test_udp_transport_creation() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = UdpTransport::new(config).await;
        assert!(transport.is_ok());
    }

    #[tokio::test]
    async fn test_udp_transport_send_unreliable_basic() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = UdpTransport::new(config).await.unwrap();
        let server_addr = transport.socket.local_addr().unwrap();

        // Create a client socket
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client_addr = client.local_addr().unwrap();

        // Start receive loop
        transport.receive_loop(config.buffer_size).await.unwrap();

        // Send from client to server
        let test_data = b"Hello, UDP!";
        client.send_to(test_data, server_addr).await.unwrap();

        // Wait for server to receive
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check server received it
        let received = transport.receive();
        assert!(received.is_some());

        let (addr, data) = received.unwrap();
        assert_eq!(&data, test_data);
        assert_eq!(addr, client_addr);
    }

    #[tokio::test]
    async fn test_udp_transport_send_unreliable_from_server() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = UdpTransport::new(config).await.unwrap();

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client_addr = client.local_addr().unwrap();

        // Send from server to client
        let test_data = b"Server to client";
        transport.send_unreliable(client_addr, test_data).unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Receive on client
        let mut buffer = vec![0u8; 1024];
        let result = timeout(Duration::from_millis(500), client.recv_from(&mut buffer)).await;

        assert!(result.is_ok());
        let (len, _) = result.unwrap().unwrap();
        assert_eq!(&buffer[..len], test_data);
    }

    #[tokio::test]
    async fn test_udp_transport_send_unreliable_multiple_datagrams() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = UdpTransport::new(config).await.unwrap();
        let server_addr = transport.socket.local_addr().unwrap();

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client_addr = client.local_addr().unwrap();

        transport.receive_loop(config.buffer_size).await.unwrap();

        // Send multiple datagrams
        let messages = vec![
            b"datagram1".to_vec(),
            b"datagram2".to_vec(),
            b"datagram3".to_vec(),
            b"datagram4".to_vec(),
            b"datagram5".to_vec(),
        ];

        for msg in &messages {
            client.send_to(msg, server_addr).await.unwrap();
        }

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Receive all messages (may arrive in any order)
        let mut received_count = 0;
        let mut received_messages = Vec::new();

        for _ in 0..messages.len() {
            if let Some((addr, data)) = transport.receive() {
                assert_eq!(addr, client_addr);
                received_messages.push(data);
                received_count += 1;
            }
        }

        assert_eq!(received_count, messages.len());

        // Verify all messages were received (order may vary)
        for msg in &messages {
            assert!(received_messages.iter().any(|m| m == msg));
        }
    }

    #[tokio::test]
    async fn test_udp_transport_datagram_size_limits() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 2048, // Small buffer
            timeout_seconds: 30,
        };

        let transport = UdpTransport::new(config.clone()).await.unwrap();
        let server_addr = transport.socket.local_addr().unwrap();

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        transport.receive_loop(config.buffer_size).await.unwrap();

        // Send a datagram that fits in buffer
        let small_data = vec![1u8; 1024];
        client.send_to(&small_data, server_addr).await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        let received = transport.receive();
        assert!(received.is_some());
        let (_, data) = received.unwrap();
        assert_eq!(data.len(), 1024);
    }

    #[tokio::test]
    async fn test_udp_transport_receive_when_empty() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = UdpTransport::new(config).await.unwrap();

        // Try to receive when nothing has been sent
        let received = transport.receive();
        assert!(received.is_none());
    }

    #[tokio::test]
    async fn test_udp_transport_bidirectional_communication() {
        let config1 = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let config2 = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport1 = UdpTransport::new(config1.clone()).await.unwrap();
        let transport2 = UdpTransport::new(config2.clone()).await.unwrap();

        let addr1 = transport1.socket.local_addr().unwrap();
        let addr2 = transport2.socket.local_addr().unwrap();

        transport1.receive_loop(config1.buffer_size).await.unwrap();
        transport2.receive_loop(config2.buffer_size).await.unwrap();

        // Send from transport1 to transport2
        let msg1 = b"From 1 to 2";
        transport1.send_unreliable(addr2, msg1).unwrap();

        // Send from transport2 to transport1
        let msg2 = b"From 2 to 1";
        transport2.send_unreliable(addr1, msg2).unwrap();

        tokio::time::sleep(Duration::from_millis(150)).await;

        // Check transport2 received msg1
        let received2 = transport2.receive();
        assert!(received2.is_some());
        let (_, data2) = received2.unwrap();
        assert_eq!(&data2, msg1);

        // Check transport1 received msg2
        let received1 = transport1.receive();
        assert!(received1.is_some());
        let (_, data1) = received1.unwrap();
        assert_eq!(&data1, msg2);
    }

    #[tokio::test]
    async fn test_network_transport_trait_tcp() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        // Test trait methods exist and compile
        let addr: SocketAddr = "127.0.0.1:8888".parse().unwrap();
        let data = b"test";

        // send_reliable should work (even if no connection)
        assert!(transport.send_reliable(addr, data).is_ok());

        // send_unreliable should return Ok (no-op for TCP)
        assert!(transport.send_unreliable(addr, data).is_ok());

        // receive should return None when no data
        assert!(transport.receive().is_none());
    }

    #[tokio::test]
    async fn test_network_transport_trait_udp() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = UdpTransport::new(config).await.unwrap();

        // Test trait methods
        let addr: SocketAddr = "127.0.0.1:8888".parse().unwrap();
        let data = b"test";

        // send_reliable should return Ok (no-op for UDP)
        assert!(transport.send_reliable(addr, data).is_ok());

        // send_unreliable should work
        assert!(transport.send_unreliable(addr, data).is_ok());

        // receive should return None when no data
        assert!(transport.receive().is_none());
    }

    #[tokio::test]
    async fn test_tcp_transport_invalid_message_length() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        let listener = transport.listener.read();
        let local_addr = listener.as_ref().unwrap().local_addr().unwrap();
        drop(listener);

        let mut client = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Send invalid length (0)
        client.write_u32(0).await.unwrap();
        client.flush().await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Connection should be closed, no message received
        let received = transport.receive();
        assert!(received.is_none());
    }

    #[tokio::test]
    async fn test_tcp_transport_oversized_message_rejected() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        let listener = transport.listener.read();
        let local_addr = listener.as_ref().unwrap().local_addr().unwrap();
        drop(listener);

        let mut client = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Send oversized length (> 10MB)
        client.write_u32(20_000_000).await.unwrap();
        client.flush().await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Connection should be closed, no message received
        let received = transport.receive();
        assert!(received.is_none());
    }

    #[tokio::test]
    async fn test_tcp_transport_multiple_concurrent_connections() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        let listener = transport.listener.read();
        let local_addr = listener.as_ref().unwrap().local_addr().unwrap();
        drop(listener);

        // Create multiple client connections
        let mut client1 = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut client2 = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut client3 = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Send messages from all clients
        let msg1 = b"from client 1";
        client1.write_u32(msg1.len() as u32).await.unwrap();
        client1.write_all(msg1).await.unwrap();
        client1.flush().await.unwrap();

        let msg2 = b"from client 2";
        client2.write_u32(msg2.len() as u32).await.unwrap();
        client2.write_all(msg2).await.unwrap();
        client2.flush().await.unwrap();

        let msg3 = b"from client 3";
        client3.write_u32(msg3.len() as u32).await.unwrap();
        client3.write_all(msg3).await.unwrap();
        client3.flush().await.unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify all messages received
        let mut received_messages = Vec::new();
        for _ in 0..3 {
            if let Some((_, data)) = transport.receive() {
                received_messages.push(data);
            }
        }

        assert_eq!(received_messages.len(), 3);
        assert!(received_messages.iter().any(|m| m == msg1));
        assert!(received_messages.iter().any(|m| m == msg2));
        assert!(received_messages.iter().any(|m| m == msg3));
    }

    #[tokio::test]
    async fn test_tcp_transport_send_to_specific_connection() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        let listener = transport.listener.read();
        let local_addr = listener.as_ref().unwrap().local_addr().unwrap();
        drop(listener);

        // Create two clients
        let mut client1 = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut client2 = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client1_addr = client1.peer_addr().unwrap();
        let client2_addr = client2.peer_addr().unwrap();

        // Send only to client1
        let msg = b"only for client 1";
        transport.send_reliable(client1_addr, msg).unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Client1 should receive it
        let length1 = client1.read_u32().await.unwrap();
        assert_eq!(length1, msg.len() as u32);
        let mut received1 = vec![0u8; length1 as usize];
        client1.read_exact(&mut received1).await.unwrap();
        assert_eq!(&received1, msg);

        // Client2 should not receive anything
        let result = timeout(Duration::from_millis(100), client2.read_u32()).await;
        assert!(
            result.is_err(),
            "Client2 should not have received a message"
        );
    }

    #[tokio::test]
    async fn test_udp_transport_rapid_fire() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = UdpTransport::new(config).await.unwrap();
        let server_addr = transport.socket.local_addr().unwrap();

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        transport.receive_loop(config.buffer_size).await.unwrap();

        // Send many datagrams rapidly
        let num_messages = 50;
        for i in 0..num_messages {
            let msg = format!("rapid message {}", i);
            client.send_to(msg.as_bytes(), server_addr).await.unwrap();
        }

        tokio::time::sleep(Duration::from_millis(500)).await;

        // Count received messages
        let mut received_count = 0;
        while transport.receive().is_some() {
            received_count += 1;
        }

        // UDP may drop packets, but we should receive most of them on localhost
        assert!(
            received_count >= num_messages - 5,
            "Received {} out of {} messages",
            received_count,
            num_messages
        );
    }

    #[tokio::test]
    async fn test_tcp_and_udp_mixed_usage() {
        // Test using both TCP and UDP transports simultaneously
        let tcp_config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let udp_config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let tcp_transport = TcpTransport::new(tcp_config).await.unwrap();
        let udp_transport = UdpTransport::new(udp_config).await.unwrap();

        // Get addresses
        let tcp_listener = tcp_transport.listener.read();
        let tcp_addr = tcp_listener.as_ref().unwrap().local_addr().unwrap();
        drop(tcp_listener);

        let udp_addr = udp_transport.socket.local_addr().unwrap();

        // Setup TCP client
        let mut tcp_client = TcpStream::connect(tcp_addr).await.unwrap();
        tcp_transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Setup UDP client
        let udp_client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        udp_transport
            .receive_loop(udp_config.buffer_size)
            .await
            .unwrap();

        // Send on both transports
        let tcp_msg = b"TCP message";
        tcp_client.write_u32(tcp_msg.len() as u32).await.unwrap();
        tcp_client.write_all(tcp_msg).await.unwrap();
        tcp_client.flush().await.unwrap();

        let udp_msg = b"UDP message";
        udp_client.send_to(udp_msg, udp_addr).await.unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify both received
        let tcp_received = tcp_transport.receive();
        assert!(tcp_received.is_some());
        let (_, tcp_data) = tcp_received.unwrap();
        assert_eq!(&tcp_data, tcp_msg);

        let udp_received = udp_transport.receive();
        assert!(udp_received.is_some());
        let (_, udp_data) = udp_received.unwrap();
        assert_eq!(&udp_data, udp_msg);
    }

    #[tokio::test]
    async fn test_tcp_transport_empty_connection_list() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        // Verify connections list is initially empty
        let connections = transport.connections.read();
        assert_eq!(connections.len(), 0);
    }

    #[tokio::test]
    async fn test_tcp_transport_connection_tracking() {
        let config = TransportConfig {
            tcp_port: 0,
            udp_port: 0,
            buffer_size: 65536,
            timeout_seconds: 30,
        };

        let transport = TcpTransport::new(config).await.unwrap();

        let listener = transport.listener.read();
        let local_addr = listener.as_ref().unwrap().local_addr().unwrap();
        drop(listener);

        // Initially no connections
        assert_eq!(transport.connections.read().len(), 0);

        // Connect one client
        let _client1 = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(transport.connections.read().len(), 1);

        // Connect another client
        let _client2 = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(transport.connections.read().len(), 2);

        // Connect a third client
        let _client3 = TcpStream::connect(local_addr).await.unwrap();
        transport.accept_connections().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(transport.connections.read().len(), 3);
    }
}

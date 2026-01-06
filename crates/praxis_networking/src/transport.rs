//! Network transport layer.

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
pub trait NetworkTransport: Send + Sync {
    /// Sends data reliably.
    fn send_reliable(&self, addr: SocketAddr, data: &[u8]) -> Result<()>;

    /// Sends data unreliably (may be lost or arrive out of order).
    fn send_unreliable(&self, addr: SocketAddr, data: &[u8]) -> Result<()>;

    /// Receives next message.
    fn receive(&self) -> Option<(SocketAddr, Vec<u8>)>;
}

/// Type alias for TCP connection write half.
type TcpWriteHalf = Arc<TokioMutex<OwnedWriteHalf>>;

/// Type alias for connection storage.
type ConnectionMap = Arc<RwLock<Vec<(SocketAddr, TcpWriteHalf)>>>;

/// TCP transport for reliable messages.
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

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert_eq!(config.tcp_port, 7777);
        assert_eq!(config.udp_port, 7778);
        assert_eq!(config.buffer_size, 65536);
        assert_eq!(config.timeout_seconds, 30);
    }
}

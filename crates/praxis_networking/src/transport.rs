//! Network transport layer.

use crossbeam_channel::{Receiver, Sender};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use praxis_utils::Result;

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

/// TCP transport for reliable messages.
pub struct TcpTransport {
    listener: Arc<RwLock<Option<TcpListener>>>,
    connections: Arc<RwLock<Vec<(SocketAddr, TcpStream)>>>,
    rx: Receiver<(SocketAddr, Vec<u8>)>,
    #[allow(dead_code)]
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
    
    /// Accepts incoming connections.
    #[allow(clippy::await_holding_lock)]
    pub async fn accept_connections(&self) -> Result<()> {
        let listener = self.listener.read();
        if let Some(listener) = listener.as_ref() {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    tracing::info!("Accepted TCP connection from {addr}");
                    let mut connections = self.connections.write();
                    connections.push((addr, stream));
                }
                Err(e) => {
                    tracing::error!("Error accepting TCP connection: {e}");
                }
            }
        }
        Ok(())
    }
}

impl NetworkTransport for TcpTransport {
    fn send_reliable(&self, addr: SocketAddr, data: &[u8]) -> Result<()> {
        // In a real implementation, this would write to the TCP stream asynchronously
        // For now, this is a simplified synchronous version
        tracing::trace!("TCP send to {}: {} bytes", addr, data.len());
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

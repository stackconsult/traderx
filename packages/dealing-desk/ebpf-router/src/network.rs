use std::net::{Ipv4Addr, SocketAddrV4};
use std::os::fd::AsRawFd;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::interval;
use log::{info, error, warn};

pub struct NetworkManager {
    b_book_socket: UdpSocket,
    a_book_addr: SocketAddrV4,
    b_book_addr: SocketAddrV4,
}

impl NetworkManager {
    pub async fn new() -> Result<Self, anyhow::Error> {
        // Create socket for B-Book internalization
        let b_book_socket = UdpSocket::bind("0.0.0.0:8766").await?;
        
        // A-Book (STP) destination - ferrumfix endpoint
        let a_book_addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8765);
        let b_book_addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8766);
        
        info!("Network manager initialized");
        info!("B-Book listening on: {}", b_book_addr);
        info!("A-Book forwarding to: {}", a_book_addr);
        
        Ok(Self {
            b_book_socket,
            a_book_addr,
            b_book_addr,
        })
    }
    
    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        let mut buf = vec![0u8; 1500];
        
        loop {
            match self.b_book_socket.recv_from(&mut buf).await {
                Ok((len, src)) => {
                    // Process B-Book order
                    self.process_b_book_order(&buf[..len], src).await?;
                }
                Err(e) => {
                    error!("Error receiving B-Book order: {}", e);
                }
            }
        }
    }
    
    async fn process_b_book_order(&self, data: &[u8], src: std::net::SocketAddr) -> Result<(), anyhow::Error> {
        // Parse OrderFrame
        if data.len() < std::mem::size_of::<traderx_ebpf_router_common::OrderFrame>() {
            warn!("Received truncated order frame");
            return Ok(());
        }
        
        let order: traderx_ebpf_router_common::OrderFrame = unsafe {
            std::ptr::read(data.as_ptr() as *const _)
        };
        
        info!(
            "B-Book order received: order_id={}, symbol={:?}, side={}, quantity={}",
            order.order_id,
            std::str::from_utf8(&order.symbol).unwrap_or("invalid"),
            order.side,
            order.quantity
        );
        
        // TODO: Process order internally
        // This would involve:
        // 1. Risk checks
        // 2. Internal matching
        // 3. Position management
        // 4. Execution
        
        Ok(())
    }
    
    pub async fn forward_to_a_book(&self, data: &[u8]) -> Result<(), anyhow::Error> {
        // Forward to A-Book (STP) via ferrumfix
        self.b_book_socket.send_to(data, self.a_book_addr).await?;
        info!("Forwarded {} bytes to A-Book", data.len());
        Ok(())
    }
    
    pub async fn shutdown(&self) -> Result<(), anyhow::Error> {
        info!("Network manager shutting down");
        Ok(())
    }
}

// XDP redirect helper functions
pub fn setup_xdp_redirect(iface: &str) -> Result<(), anyhow::Error> {
    use std::process::Command;
    
    // Create B-Book socket for XDP redirect
    let output = Command::new("ip")
        .args(&["link", "add", "traderx-bbook", "type", "veth", "peer", "name", "traderx-bbook-peer"])
        .output()?;
    
    if !output.status.success() {
        warn!("Failed to create veth pair: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Set up IP addresses
    Command::new("ip")
        .args(&["addr", "add", "127.0.0.2/24", "dev", "traderx-bbook"])
        .output()?;
    
    Command::new("ip")
        .args(&["addr", "add", "127.0.0.3/24", "dev", "traderx-bbook-peer"])
        .output()?;
    
    // Bring interfaces up
    Command::new("ip")
        .args(&["link", "set", "traderx-bbook", "up"])
        .output()?;
    
    Command::new("ip")
        .args(&["link", "set", "traderx-bbook-peer", "up"])
        .output()?;
    
    info!("XDP redirect network setup complete");
    
    Ok(())
}

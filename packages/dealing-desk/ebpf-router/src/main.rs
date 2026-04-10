use aya::{
    programs::{Xdp, ProgramError},
    Ebpf,
};
use aya_log::BpfLogger;
use clap::Parser;
use log::{info, warn};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::Ipv4Addr;
use std::time::Duration;
use tokio::signal;
use tokio::time::interval;
use traderx_ebpf_router_common::{OrderFrame, RoutingDecision, HSTR_KEY_LEN};
use traderx_ebpf_router_common::user::{HstrStateUpdate, hstr_key_from_strings, f32_to_fixed_point};

mod network;
use network::NetworkManager;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let opt = Opt::parse();

    // This will include your eBPF object file as raw bytes at compile time and load it at runtime. 
    // This approach is recommended for most real-world use cases. If you would like to
    // include the eBPF program from your host filesystem instead, comment out the line below
    // and uncomment the commented line below it.
    let mut bpf = Ebpf::load(include_bytes_aligned!("../ebpf/target/bpfel-unknown-none/release/traderx-ebpf-router"))?;
    // let mut bpf = Ebpf::load_file("ebpf/target/bpfel-unknown-none/release/traderx-ebpf-router")?;

    if let Err(e) = BpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program
        warn!("failed to initialize eBPF logger: {}", e);
    }

    let program: &mut Xdp = bpf.program_mut("traderx_router").unwrap().try_into()?;
    program.load()?;
    program.attach(&opt.iface, aya::programs::XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    // Initialize network manager
    let mut network_manager = NetworkManager::new().await?;
    
    // Start HSTR state sync task
    let hstr_handle = tokio::spawn(async move {
        sync_hstr_state().await;
    });

    // Start statistics reporting task
    let stats_handle = tokio::spawn(async move {
        report_statistics().await;
    });

    info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    info!("Exiting...");

    network_manager.shutdown().await?;
    hstr_handle.abort();
    stats_handle.abort();

    Ok(())
}

async fn sync_hstr_state() {
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    
    let mut interval = interval(Duration::from_millis(100));
    
    loop {
        interval.tick().await;
        
        // Get all HSTR keys from Redis
        let keys: Vec<String> = redis_conn
            .keys("hstr:sharpe:*")
            .await
            .unwrap_or_default();
        
        for key in keys {
            // Parse key to extract tenant and symbol
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let tenant_id = parts[2];
            let symbol = parts[3];
            
            // Get value from Redis
            let value: String = redis_conn.get(&key).await.unwrap_or_default();
            if value.is_empty() {
                continue;
            }
            
            // Parse JSON value
            let state: HstrStateUpdate = match serde_json::from_str(&value) {
                Ok(v) => v,
                Err(e) => {
                    warn!("Failed to parse HSTR state: {}", e);
                    continue;
                }
            };
            
            // Update eBPF map
            let hstr_key = hstr_key_from_strings(tenant_id, symbol);
            let hstr_value = traderx_ebpf_router_common::HstrValue {
                sharpe_ratio: f32_to_fixed_point(state.sharpe_ratio),
                confidence: f32_to_fixed_point(state.confidence),
                timestamp: state.timestamp,
            };
            
            // TODO: Update eBPF map with new value
            // This requires access to the eBPF maps from the sync task
            // For now, we'll store in a local cache
        }
    }
}

async fn report_statistics() {
    let mut interval = interval(Duration::from_secs(5));
    
    loop {
        interval.tick().await;
        
        // TODO: Read statistics from eBPF maps
        // Report metrics to monitoring system
        
        info!("Statistics report - TODO: Implement eBPF map reading");
    }
}

#[inline(always)]
pub fn include_bytes_aligned<S: AsRef<Path>>(path: S) -> &'static [u8] {
    #[repr(align(8))]
    struct Aligned<T: ?Sized>(T);
    
    let raw = include_bytes!(path.as_ref());
    unsafe { &Aligned(raw).0 }
}

use anyhow::{Context, Result};
use std::path::Path;

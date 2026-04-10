#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    programs::XdpContext,
    EbpfContext,
};
use aya_log_ebpf::{debug, error, info, warn};
use network_byte_order::{write_be16, write_be32, write_be64};
use traderx_ebpf_router_common::{OrderFrame, RoutingDecision, HSTR_KEY_LEN};

#[map(name = "HSTR_STATE")]
static mut HSTR_STATE: HashMap<[u8; HSTR_KEY_LEN], HstrValue> = HashMap::with_max_entries(10240, 0);

#[map(name = "ROUTING_STATS")]
static mut ROUTING_STATS: HashMap<u32, u64> = HashMap::with_max_entries(10, 0);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct HstrValue {
    sharpe_ratio: u32,  // Fixed point 16.16
    confidence: u32,    // Fixed point 16.16
    timestamp: u64,
}

const CONFIDENCE_THRESHOLD: u32 = 0x09999; // 0.6 in 16.16 fixed point
const SHARPE_THRESHOLD: u32 = 0x18000;    // 1.5 in 16.16 fixed point

#[xdp(name = "traderx_router")]
pub fn traderx_router(ctx: XdpContext) -> u32 {
    match try_traderx_router(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_traderx_router(ctx: XdpContext) -> Result<u32, u32> {
    // Get packet pointer
    let ptr = ctx.data();
    let ptr_end = ctx.data_end();
    
    // Minimum packet size check (Ethernet + IP + UDP + OrderFrame)
    if ptr.add(54) > ptr_end {
        warn!(ctx, "Packet too short");
        return Ok(xdp_action::XDP_PASS);
    }
    
    // Parse Ethernet header
    let eth_proto = u16::from_be(unsafe { *ptr.add(12).cast::<u16>() });
    if eth_proto != 0x0800 {
        // Not IPv4, pass through
        return Ok(xdp_action::XDP_PASS);
    }
    
    // Parse IP header
    let ip_proto = unsafe { *ptr.add(23) };
    if ip_proto != 17 {
        // Not UDP, pass through
        return Ok(xdp_action::XDP_PASS);
    }
    
    // Parse UDP header
    let udp_dest = u16::from_be(unsafe { *ptr.add(36).cast::<u16>() });
    if udp_dest != 8765 {
        // Not our order port, pass through
        return Ok(xdp_action::XDP_PASS);
    }
    
    // Extract OrderFrame from UDP payload
    let order_start = ptr.add(42);
    if order_start.add(core::mem::size_of::<OrderFrame>()) > ptr_end {
        warn!(ctx, "OrderFrame truncated");
        return Ok(xdp_action::XDP_PASS);
    }
    
    let order: OrderFrame = unsafe { core::ptr::read(order_start.cast::<OrderFrame>()) };
    
    // Validate order checksum
    let mut checksum = 0u32;
    let order_bytes = unsafe {
        core::slice::from_raw_parts(
            order_start.cast::<u8>(),
            core::mem::size_of::<OrderFrame>()
        )
    };
    
    for chunk in order_bytes.chunks_exact(4) {
        checksum += u32::from_le_bytes(chunk.try_into().unwrap());
    }
    
    if checksum != order.checksum {
        warn!(ctx, "Invalid order checksum");
        update_stats(STATS_INVALID_CHECKSUM);
        return Ok(xdp_action::XDP_PASS);
    }
    
    // Create HSTR key from tenant_id and symbol
    let mut hstr_key = [0u8; HSTR_KEY_LEN];
    let tenant_bytes = order.tenant_id.as_bytes();
    let symbol_bytes = order.symbol.as_bytes();
    
    let key_len = core::cmp::min(tenant_bytes.len(), 16);
    hstr_key[..key_len].copy_from_slice(&tenant_bytes[..key_len]);
    
    let symbol_start = 16;
    let symbol_len = core::cmp::min(symbol_bytes.len(), 16);
    hstr_key[symbol_start..symbol_start + symbol_len].copy_from_slice(&symbol_bytes[..symbol_len]);
    
    // Look up HSTR state
    let hstr_value = unsafe {
        HSTR_STATE.get(&hstr_key, 0)
    };
    
    let hstr_value = match hstr_value {
        Some(value) => value,
        None => {
            // No HSTR state, use defaults
            HstrValue {
                sharpe_ratio: 0x10000, // 1.0 in 16.16
                confidence: 0x10000,   // 1.0 in 16.16
                timestamp: 0,
            }
        }
    };
    
    // Make routing decision
    let route_decision = if order.confidence < CONFIDENCE_THRESHOLD 
        || hstr_value.sharpe_ratio > SHARPE_THRESHOLD {
        // Route to A-Book (STP)
        RoutingDecision {
            order_id: order.order_id,
            route: 1, // A_BOOK
            reason: 0, // Low confidence or high Sharpe
            sharpe_ratio: hstr_value.sharpe_ratio,
            confidence: order.confidence,
            timestamp: bpf_ktime_get_ns(),
        }
    } else {
        // Route to B-Book (internalization)
        RoutingDecision {
            order_id: order.order_id,
            route: 0, // B_BOOK
            reason: 1, // Normal routing
            sharpe_ratio: hstr_value.sharpe_ratio,
            confidence: order.confidence,
            timestamp: bpf_ktime_get_ns(),
        }
    };
    
    // Log routing decision
    info!(
        ctx,
        "Order routed",
        order_id = %u64::from_be(order.order_id),
        route = %route_decision.route,
        confidence = %fixed_point_to_f32(order.confidence),
        sharpe = %fixed_point_to_f32(hstr_value.sharpe_ratio)
    );
    
    // Update statistics
    update_stats(if route_decision.route == 0 {
        STATS_B_BOOK_ROUTES
    } else {
        STATS_A_BOOK_ROUTES
    });
    
    // Return XDP action
    Ok(if route_decision.route == 0 {
        xdp_action::XDP_REDIRECT
    } else {
        xdp_action::XDP_PASS
    })
}

#[inline(always)]
fn fixed_point_to_f32(value: u32) -> f32 {
    (value as f32) / 65536.0
}

#[inline(always)]
fn update_stats(stat_id: u32) {
    unsafe {
        let mut count = ROUTING_STATS.get(&stat_id, 0).unwrap_or(0);
        count += 1;
        ROUTING_STATS.insert(&stat_id, &count, 0);
    }
}

// Statistics keys
const STATS_B_BOOK_ROUTES: u32 = 0;
const STATS_A_BOOK_ROUTES: u32 = 1;
const STATS_INVALID_CHECKSUM: u32 = 2;
const STATS_NO_HSTR_STATE: u32 = 3;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unreachable!()
}

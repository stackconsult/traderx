#![allow(clippy::missing_safety_doc)]

#[cfg(feature = "user")]
use aya::{Pod, Ebpf};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "user", derive(Pod))]
pub struct OrderFrame {
    pub order_id: u64,
    pub tenant_id: [u8; 16],
    pub symbol: [u8; 16],
    pub side: u8,  // 0=BUY, 1=SELL
    pub quantity: u64,
    pub price: u64,
    pub confidence: u32,  // Fixed point 16.16
    pub timestamp: u64,
    pub checksum: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "user", derive(Pod))]
pub struct RoutingDecision {
    pub order_id: u64,
    pub route: u8,  // 0=B_BOOK, 1=A_BOOK
    pub reason: u8,
    pub sharpe_ratio: u32,  // Fixed point 16.16
    pub confidence: u32,    // Fixed point 16.16
    pub timestamp: u64,
}

pub const HSTR_KEY_LEN: usize = 32; // 16 bytes tenant_id + 16 bytes symbol

#[cfg(feature = "user")]
pub mod user {
    use super::*;
    use std::collections::HashMap;
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct HstrStateUpdate {
        pub tenant_id: String,
        pub symbol: String,
        pub sharpe_ratio: f32,
        pub confidence: f32,
        pub timestamp: u64,
    }
    
    pub fn hstr_key_from_strings(tenant_id: &str, symbol: &str) -> [u8; HSTR_KEY_LEN] {
        let mut key = [0u8; HSTR_KEY_LEN];
        
        let tenant_bytes = tenant_id.as_bytes();
        let key_len = std::cmp::min(tenant_bytes.len(), 16);
        key[..key_len].copy_from_slice(&tenant_bytes[..key_len]);
        
        let symbol_bytes = symbol.as_bytes();
        let symbol_start = 16;
        let symbol_len = std::cmp::min(symbol_bytes.len(), 16);
        key[symbol_start..symbol_start + symbol_len].copy_from_slice(&symbol_bytes[..symbol_len]);
        
        key
    }
    
    pub fn f32_to_fixed_point(value: f32) -> u32 {
        (value * 65536.0) as u32
    }
    
    pub fn fixed_point_to_f32(value: u32) -> f32 {
        value as f32 / 65536.0
    }
}

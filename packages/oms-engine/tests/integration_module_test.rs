//! Integration Module Test
//! 
//! Tests the canonical integration module to ensure it provides the correct
//! API for wiring all TraderX components together.

use oms_engine::{
    integration::{create_trading_system, SystemConfig},
    AgentSignal,
};

#[tokio::test]
async fn test_integration_module_creates_system() {
    // Create system configuration
    let config = SystemConfig::default();
    
    // Create trading system using canonical factory
    let (system, _handles) = create_trading_system(config).await.expect("Failed to create trading system");
    
    // Verify system is healthy
    assert!(system.is_healthy(), "System should be healthy initially");
    
    // Create test signal
    let signal = AgentSignal {
        agent_id: "test_agent".to_string(),
        symbol: "AAPL".to_string(),
        direction: "long".to_string(),
        conviction: 0.7,
        max_notional_usd: 10_000.0,
        ttl_ms: 5000,
        meta: serde_json::json!({"test": true}),
    };
    
    // Route signal through system
    let outcome = system.route_signal(signal).await;
    
    // Verify signal was processed
    assert!(outcome.order_id.is_some(), "Signal should generate an order");
    
    println!("✅ Integration module test passed");
    println!("   Order ID: {:?}", outcome.order_id);
    println!("   Status: {:?}", outcome.status);
}

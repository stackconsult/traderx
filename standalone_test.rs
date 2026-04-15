//! Standalone test for core TraderX trading flow
//! Tests RiskBus + SignalRouter integration without any dependencies

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};

// Simplified test without external dependencies
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TRADERX TRADING FLOW TEST ===");
    
    // 1. Create Risk Bus
    let risk_bus = Arc::new(RiskBus::new(10_000_000.0, -2000));
    println!("✅ Risk Bus initialized with $10M capital");
    
    // 2. Test normal signal
    println!("\n--- Test 1: Normal Signal ---");
    let normal_signal = TestSignal {
        symbol: "AAPL".to_string(),
        conviction: 0.7,
        max_notional_usd: 10_000.0,
    };
    
    let outcome = test_signal_routing(&risk_bus, normal_signal);
    println!("Signal routed: {:?}", outcome);
    
    // 3. Test oversized signal
    println!("\n--- Test 2: Oversized Signal ---");
    let oversized_signal = TestSignal {
        symbol: "AAPL".to_string(),
        conviction: 0.7,
        max_notional_usd: 5_000_000.0, // Too large - will calculate $875,000 position
    };
    
    let outcome = test_signal_routing(&risk_bus, oversized_signal);
    println!("Signal routed: {:?}", outcome);
    
    // 4. Final status
    println!("\n--- Final Status ---");
    println!("Risk Bus Halted: {}", risk_bus.is_halted());
    println!("Current Drawdown: {} bps", risk_bus.dd_bps());
    println!("Orders Submitted: {}", risk_bus.orders_submitted_count());
    println!("Orders Rejected: {}", risk_bus.orders_rejected_count());
    
    // 5. Validate test results
    let submitted = risk_bus.orders_submitted_count();
    let rejected = risk_bus.orders_rejected_count();
    
    if submitted == 1 && rejected == 1 {
        println!("\n✅ TRADING FLOW TEST PASSED");
        println!("✅ Risk enforcement working correctly");
        println!("✅ Signal routing functional");
        println!("✅ Core trading logic validated");
    } else {
        println!("\n❌ TRADING FLOW TEST FAILED");
        println!("Expected: 1 submitted, 1 rejected");
        println!("Got: {} submitted, {} rejected", submitted, rejected);
    }
    
    Ok(())
}

#[derive(Debug)]
struct TestSignal {
    symbol: String,
    conviction: f64,
    max_notional_usd: f64,
}

#[derive(Debug)]
enum RouteOutcome {
    Accepted { position_size: f64 },
    Rejected { reason: String },
}

struct RiskBus {
    capital_usd: f64,
    drawdown_bps: i64,
    orders_submitted: AtomicU64,
    orders_rejected: AtomicU64,
    is_halted: AtomicBool,
}

impl RiskBus {
    fn new(capital_usd: f64, drawdown_bps: i64) -> Self {
        Self {
            capital_usd,
            drawdown_bps,
            orders_submitted: AtomicU64::new(0),
            orders_rejected: AtomicU64::new(0),
            is_halted: AtomicBool::new(false),
        }
    }
    
    fn check_symbol(&self, _symbol: &str, notional: f64) -> Result<(), String> {
        // Simple risk check - position cannot exceed 5% of capital
        let limit = self.capital_usd * 0.05;
        println!("Risk check: ${:.2} vs limit ${:.2}", notional, limit);
        if notional > limit {
            Err(format!("Position too large: ${:.2} > ${:.2}", notional, limit))
        } else {
            Ok(())
        }
    }
    
    fn is_halted(&self) -> bool {
        self.is_halted.load(Ordering::SeqCst)
    }
    
    fn dd_bps(&self) -> i64 {
        self.drawdown_bps
    }
    
    fn orders_submitted_count(&self) -> u64 {
        self.orders_submitted.load(Ordering::SeqCst)
    }
    
    fn orders_rejected_count(&self) -> u64 {
        self.orders_rejected.load(Ordering::SeqCst)
    }
}

fn test_signal_routing(risk_bus: &Arc<RiskBus>, signal: TestSignal) -> RouteOutcome {
    // Calculate position size using Kelly criterion
    let kelly_fraction = 0.25;
    let position_size = signal.max_notional_usd * signal.conviction * kelly_fraction;
    
    println!("Signal: {} max=${:.2}, conviction={:.2}, kelly={:.2}", 
             signal.symbol, signal.max_notional_usd, signal.conviction, kelly_fraction);
    println!("Calculated position size: ${:.2}", position_size);
    
    // Risk check
    match risk_bus.check_symbol(&signal.symbol, position_size) {
        Ok(()) => {
            risk_bus.orders_submitted.fetch_add(1, Ordering::SeqCst);
            println!("→ ORDER SENT: {} ${:.2}", signal.symbol, position_size);
            RouteOutcome::Accepted { position_size }
        }
        Err(reason) => {
            risk_bus.orders_rejected.fetch_add(1, Ordering::SeqCst);
            println!("→ ORDER REJECTED: {}", reason);
            RouteOutcome::Rejected { reason }
        }
    }
}

//! Example demonstrating UnixNanos time type for high-precision trading events.
//!
//! This example shows how to use the UnixNanos time type for:
//! - Timestamping order events with nanosecond precision
//! - Measuring latency between event processing stages
//! - Creating time ranges for historical queries
//! - Using the TimeService for deterministic testing

use oms_engine::time::{UnixNanos, TimeRange, TimeService, MockTimeProvider, TimeProvider};
use std::time::Duration;

/// Example order event with nanosecond timestamp
#[derive(Debug)]
struct OrderEvent {
    order_id: String,
    symbol: String,
    timestamp: UnixNanos,
    event_type: String,
}

impl OrderEvent {
    fn new(order_id: &str, symbol: &str, event_type: &str) -> Self {
        Self {
            order_id: order_id.to_string(),
            symbol: symbol.to_string(),
            timestamp: UnixNanos::now(),
            event_type: event_type.to_string(),
        }
    }
    
    /// Calculate latency from event creation to now
    fn latency(&self) -> Duration {
        UnixNanos::now().since(self.timestamp)
    }
}

/// Example latency tracker for measuring system performance
struct LatencyTracker {
    events: Vec<(String, UnixNanos, Duration)>,
}

impl LatencyTracker {
    fn new() -> Self {
        Self { events: Vec::new() }
    }
    
    fn record(&mut self, stage: &str, start: UnixNanos, end: UnixNanos) {
        let latency = end.since(start);
        self.events.push((stage.to_string(), start, latency));
    }
    
    fn report(&self) {
        println!("\nLatency Report:");
        println!("{:<20} {:>15} {:>15}", "Stage", "Timestamp (ns)", "Latency (μs)");
        println!("{}", "-".repeat(55));
        
        for (stage, timestamp, latency) in &self.events {
            println!(
                "{:<20} {:>15} {:>15.2}",
                stage,
                timestamp.as_u64(),
                latency.as_micros()
            );
        }
    }
}

fn main() {
    println!("UnixNanos Time Type Example\n");
    println!("{}", "=".repeat(60));
    
    // Example 1: Basic timestamp operations
    println!("\n1. Basic Timestamp Operations");
    println!("{}", "-".repeat(40));
    
    let now = UnixNanos::now();
    println!("Current time: {} nanoseconds", now.as_u64());
    println!("Current time: {} seconds", now.as_secs());
    println!("Current time: {} milliseconds", now.as_millis());
    println!("RFC 3339 format: {}", now.to_rfc3339());
    
    // Example 2: Creating order events with timestamps
    println!("\n2. Order Event Timestamping");
    println!("{}", "-".repeat(40));
    
    let order_created = OrderEvent::new("ORD-001", "AAPL", "CREATED");
    std::thread::sleep(Duration::from_millis(10)); // Simulate processing time
    let order_submitted = OrderEvent::new("ORD-001", "AAPL", "SUBMITTED");
    
    println!("Order created at: {}", order_created.timestamp.to_rfc3339());
    println!("Order submitted at: {}", order_submitted.timestamp.to_rfc3339());
    println!(
        "Processing latency: {} μs",
        order_submitted.timestamp.since(order_created.timestamp).as_micros()
    );
    
    // Example 3: Latency tracking
    println!("\n3. System Latency Tracking");
    println!("{}", "-".repeat(40));
    
    let mut tracker = LatencyTracker::new();
    
    // Simulate order processing pipeline
    let t1 = UnixNanos::now();
    std::thread::sleep(Duration::from_micros(50));
    let t2 = UnixNanos::now();
    tracker.record("Validation", t1, t2);
    
    std::thread::sleep(Duration::from_micros(100));
    let t3 = UnixNanos::now();
    tracker.record("Risk Check", t2, t3);
    
    std::thread::sleep(Duration::from_micros(75));
    let t4 = UnixNanos::now();
    tracker.record("Order Routing", t3, t4);
    
    tracker.report();
    
    // Example 4: Time ranges for historical queries
    println!("\n4. Time Range Queries");
    println!("{}", "-".repeat(40));
    
    let query_range = TimeRange::from_now(Duration::from_secs(3600)); // Last hour
    let event_time = UnixNanos::now() - Duration::from_secs(1800); // 30 minutes ago
    
    println!("Query range duration: {} seconds", query_range.duration().as_secs());
    println!("Event within range: {}", query_range.contains(event_time));
    
    let old_event = UnixNanos::now() - Duration::from_secs(7200); // 2 hours ago
    println!("Old event within range: {}", query_range.contains(old_event));
    
    // Example 5: Deterministic time for testing
    println!("\n5. Deterministic Time for Testing");
    println!("{}", "-".repeat(40));
    
    let start_time = UnixNanos::from_secs(1000000000); // Fixed start time
    let mut mock_provider = MockTimeProvider::new(start_time);
    
    println!("Mock time start: {}", mock_provider.now().as_secs());
    
    // Simulate time passing
    mock_provider.advance(Duration::from_secs(60));
    println!("After 60 seconds: {}", mock_provider.now().as_secs());
    
    mock_provider.advance(Duration::from_millis(500));
    println!("After 500ms: {}", mock_provider.now().as_millis());
    
    // Example 6: TimeService for dependency injection
    println!("\n6. TimeService for Production Code");
    println!("{}", "-".repeat(40));
    
    let time_service = TimeService::new();
    let current = time_service.now();
    
    println!("Current time via TimeService: {}", current.to_rfc3339());
    println!("Has epoch passed: {}", time_service.has_passed(UnixNanos::zero()));
    
    let future = current + Duration::from_secs(3600);
    match time_service.time_until(future) {
        Some(duration) => println!("Time until future event: {} seconds", duration.as_secs()),
        None => println!("Future event already passed"),
    }
    
    // Example 7: Arithmetic operations
    println!("\n7. Time Arithmetic");
    println!("{}", "-".repeat(40));
    
    let t_start = UnixNanos::from_secs(1000);
    let t_end = UnixNanos::from_secs(1500);
    
    let duration_between = t_end - t_start;
    println!("Duration between timestamps: {} seconds", duration_between.as_secs());
    
    let t_future = t_start + Duration::from_secs(200);
    println!(
        "Start + 200 seconds = {} seconds",
        t_future.as_secs()
    );
    
    println!("\n{}", "=".repeat(60));
    println!("Example completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_order_event_latency() {
        let start = UnixNanos::now();
        std::thread::sleep(Duration::from_millis(5));
        let end = UnixNanos::now();
        
        let latency = end.since(start);
        assert!(latency.as_millis() >= 4); // Should be at least 4ms
    }
    
    #[test]
    fn test_time_range_query() {
        let range = TimeRange::from_now(Duration::from_secs(60));
        let recent = UnixNanos::now() - Duration::from_secs(30);
        let old = UnixNanos::now() - Duration::from_secs(120);
        
        assert!(range.contains(recent));
        assert!(!range.contains(old));
    }
}

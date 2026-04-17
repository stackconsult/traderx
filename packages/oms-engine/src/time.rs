//! UnixNanos time type for deterministic high-precision timestamp handling.
//! 
//! Extracted from NautilusTrader patterns for nanosecond-precision time management
//! in trading systems. Provides zero-cost abstractions for timestamp operations.

use std::fmt;
use std::ops::{Add, Sub, Deref};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc, TimeZone};
use serde::{Serialize, Deserialize};

/// Nanoseconds duration type
pub type Nanos = u64;

/// High-precision Unix timestamp in nanoseconds
/// 
/// Provides deterministic time handling for trading systems with:
/// - Nanosecond precision
/// - Zero-cost abstractions
/// - Conversion utilities
/// - Arithmetic operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UnixNanos(u64);

impl UnixNanos {
    /// Create from raw nanoseconds value
    pub const fn new(nanos: u64) -> Self {
        Self(nanos)
    }
    
    /// Get current time as UnixNanos
    pub fn now() -> Self {
        Self::from_system_time(SystemTime::now())
    }
    
    /// Create from SystemTime
    pub fn from_system_time(time: SystemTime) -> Self {
        let duration = time.duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        Self(duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64)
    }
    
    /// Create from DateTime<Utc>
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt.timestamp_nanos_opt().unwrap_or(0) as u64)
    }
    
    /// Create from seconds (converts to nanoseconds)
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs * 1_000_000_000)
    }
    
    /// Create from milliseconds (converts to nanoseconds)
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis * 1_000_000)
    }
    
    /// Create from microseconds (converts to nanoseconds)
    pub const fn from_micros(micros: u64) -> Self {
        Self(micros * 1_000)
    }
    
    /// Get raw nanoseconds value
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
    
    /// Convert to SystemTime
    pub fn to_system_time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_nanos(self.0)
    }
    
    /// Convert to DateTime<Utc>
    pub fn to_datetime(&self) -> DateTime<Utc> {
        Utc.timestamp_nanos(self.0 as i64)
    }
    
    /// Convert to seconds (truncates nanoseconds)
    pub const fn as_secs(&self) -> u64 {
        self.0 / 1_000_000_000
    }
    
    /// Convert to milliseconds (truncates microseconds/nanoseconds)
    pub const fn as_millis(&self) -> u64 {
        self.0 / 1_000_000
    }
    
    /// Convert to microseconds (truncates nanoseconds)
    pub const fn as_micros(&self) -> u64 {
        self.0 / 1_000
    }
    
    /// Calculate duration since another UnixNanos
    pub fn since(&self, other: UnixNanos) -> Duration {
        if self.0 >= other.0 {
            Duration::from_nanos(self.0 - other.0)
        } else {
            Duration::from_secs(0)
        }
    }
    
    /// Calculate duration until another UnixNanos
    pub fn until(&self, other: UnixNanos) -> Duration {
        if other.0 >= self.0 {
            Duration::from_nanos(other.0 - self.0)
        } else {
            Duration::from_secs(0)
        }
    }
    
    /// Check if this timestamp is in the past relative to now
    pub fn is_past(&self) -> bool {
        self.0 < Self::now().0
    }
    
    /// Check if this timestamp is in the future relative to now
    pub fn is_future(&self) -> bool {
        self.0 > Self::now().0
    }
    
    /// Format as RFC 3339 string
    pub fn to_rfc3339(&self) -> String {
        self.to_datetime().to_rfc3339()
    }
    
    /// Parse from RFC 3339 string
    pub fn from_rfc3339(s: &str) -> Result<Self, chrono::ParseError> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| Self::from_datetime(dt.with_timezone(&Utc)))
    }
    
    /// Get zero timestamp (Unix epoch)
    pub const fn zero() -> Self {
        Self(0)
    }
    
    /// Check if timestamp is zero (Unix epoch)
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Default for UnixNanos {
    fn default() -> Self {
        Self::now()
    }
}

impl Deref for UnixNanos {
    type Target = u64;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add<Duration> for UnixNanos {
    type Output = Self;
    
    fn add(self, duration: Duration) -> Self::Output {
        Self(self.0 + duration.as_nanos() as u64)
    }
}

impl Sub<Duration> for UnixNanos {
    type Output = Self;
    
    fn sub(self, duration: Duration) -> Self::Output {
        Self(self.0.saturating_sub(duration.as_nanos() as u64))
    }
}

impl Sub<UnixNanos> for UnixNanos {
    type Output = Duration;
    
    fn sub(self, other: Self) -> Self::Output {
        self.since(other)
    }
}

impl fmt::Display for UnixNanos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ns", self.0)
    }
}

/// Time range for querying historical data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: UnixNanos,
    pub end: UnixNanos,
}

impl TimeRange {
    /// Create new time range
    pub fn new(start: UnixNanos, end: UnixNanos) -> Self {
        Self { start, end }
    }
    
    /// Create range from now going back specified duration
    pub fn from_now(duration: Duration) -> Self {
        let end = UnixNanos::now();
        let start = end - duration;
        Self { start, end }
    }
    
    /// Check if a timestamp falls within this range
    pub fn contains(&self, timestamp: UnixNanos) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }
    
    /// Get duration of the range
    pub fn duration(&self) -> Duration {
        self.end.since(self.start)
    }
    
    /// Check if range is valid (start <= end)
    pub fn is_valid(&self) -> bool {
        self.start <= self.end
    }
}

/// Time provider trait for dependency injection in testing
pub trait TimeProvider: Send + Sync {
    fn now(&self) -> UnixNanos;
}

/// Real time provider using system clock
pub struct SystemTimeProvider;

impl TimeProvider for SystemTimeProvider {
    fn now(&self) -> UnixNanos {
        UnixNanos::now()
    }
}

/// Mock time provider for testing
pub struct MockTimeProvider {
    current_time: UnixNanos,
}

impl MockTimeProvider {
    pub fn new(start_time: UnixNanos) -> Self {
        Self { current_time: start_time }
    }
    
    pub fn advance(&mut self, duration: Duration) {
        self.current_time = self.current_time + duration;
    }
    
    pub fn set_time(&mut self, time: UnixNanos) {
        self.current_time = time;
    }
}

impl TimeProvider for MockTimeProvider {
    fn now(&self) -> UnixNanos {
        self.current_time
    }
}

/// Time service for centralized time management
pub struct TimeService {
    provider: Box<dyn TimeProvider>,
}

impl TimeService {
    /// Create with system time provider
    pub fn new() -> Self {
        Self {
            provider: Box::new(SystemTimeProvider),
        }
    }
    
    /// Create with custom time provider
    pub fn with_provider(provider: Box<dyn TimeProvider>) -> Self {
        Self { provider }
    }
    
    /// Get current time
    pub fn now(&self) -> UnixNanos {
        self.provider.now()
    }
    
    /// Check if a timestamp has passed
    pub fn has_passed(&self, timestamp: UnixNanos) -> bool {
        self.now().0 >= timestamp.0
    }
    
    /// Calculate time until a timestamp
    pub fn time_until(&self, timestamp: UnixNanos) -> Option<Duration> {
        let now = self.now();
        if timestamp.0 > now.0 {
            Some(timestamp.since(now))
        } else {
            None
        }
    }
}

impl Default for TimeService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unix_nanos_creation() {
        let nanos = UnixNanos::new(1_000_000_000);
        assert_eq!(nanos.as_u64(), 1_000_000_000);
    }
    
    #[test]
    fn test_unix_nanos_from_secs() {
        let nanos = UnixNanos::from_secs(1);
        assert_eq!(nanos.as_u64(), 1_000_000_000);
        assert_eq!(nanos.as_secs(), 1);
    }
    
    #[test]
    fn test_unix_nanos_from_millis() {
        let nanos = UnixNanos::from_millis(1);
        assert_eq!(nanos.as_u64(), 1_000_000);
        assert_eq!(nanos.as_millis(), 1);
    }
    
    #[test]
    fn test_unix_nanos_datetime_conversion() {
        let nanos = UnixNanos::from_secs(1000);
        let dt = nanos.to_datetime();
        let back_to_nanos = UnixNanos::from_datetime(dt);
        
        // May have slight precision loss due to datetime conversion
        let diff = if back_to_nanos.as_secs() >= 1000 {
            back_to_nanos.as_secs() - 1000
        } else {
            1000 - back_to_nanos.as_secs()
        };
        assert!(diff <= 1);
    }
    
    #[test]
    fn test_unix_nanos_arithmetic() {
        let nanos = UnixNanos::from_secs(10);
        let duration = Duration::from_secs(5);
        
        let added = nanos + duration;
        assert_eq!(added.as_secs(), 15);
        
        let subtracted = nanos - duration;
        assert_eq!(subtracted.as_secs(), 5);
        
        let diff = nanos - UnixNanos::from_secs(3);
        assert_eq!(diff.as_secs(), 7);
    }
    
    #[test]
    fn test_time_range() {
        let start = UnixNanos::from_secs(0);
        let end = UnixNanos::from_secs(10);
        let range = TimeRange::new(start, end);
        
        assert!(range.contains(UnixNanos::from_secs(5)));
        assert!(!range.contains(UnixNanos::from_secs(15)));
        assert!(range.is_valid());
        assert_eq!(range.duration().as_secs(), 10);
    }
    
    #[test]
    fn test_mock_time_provider() {
        let start = UnixNanos::from_secs(1000);
        let mut provider = MockTimeProvider::new(start);
        
        assert_eq!(provider.now(), start);
        
        provider.advance(Duration::from_secs(100));
        assert_eq!(provider.now().as_secs(), 1100);
        
        provider.set_time(UnixNanos::from_secs(2000));
        assert_eq!(provider.now().as_secs(), 2000);
    }
    
    #[test]
    fn test_time_service() {
        let service = TimeService::new();
        let now = service.now();
        
        // Should return a reasonable timestamp (not zero)
        assert!(now.as_secs() > 1_000_000_000); // After year 2001
        
        // Test has_passed
        let past = UnixNanos::from_secs(now.as_secs() - 10);
        assert!(service.has_passed(past));
        
        let future = UnixNanos::from_secs(now.as_secs() + 10);
        assert!(!service.has_passed(future));
    }
}

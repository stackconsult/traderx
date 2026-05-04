use crate::middleware::llm_message_bus::types::LlmProvider;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Circuit breaker for LLM providers
pub struct CircuitBreaker {
    pub provider: LlmProvider,
    pub state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    recovery_timeout: Duration,
    last_failure_time: Option<Instant>,
    request_count: u64,
    success_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing recovery
}

impl CircuitBreaker {
    pub fn new(provider: LlmProvider, failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            provider,
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            recovery_timeout,
            last_failure_time: None,
            request_count: 0,
            success_count: 0,
        }
    }

    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() > self.recovery_timeout {
                        self.state = CircuitState::HalfOpen;
                        info!(
                            "Circuit breaker for {:?} transitioning to half-open",
                            self.provider
                        );
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        self.request_count += 1;
        self.success_count += 1;

        match self.state {
            CircuitState::HalfOpen => {
                self.state = CircuitState::Closed;
                self.failure_count = 0;
                info!("Circuit breaker for {:?} reset to closed", self.provider);
            }
            _ => {}
        }
    }

    pub fn record_failure(&mut self) {
        self.request_count += 1;
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    warn!(
                        "Circuit breaker for {:?} opened after {} failures",
                        self.provider, self.failure_count
                    );
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                warn!("Circuit breaker for {:?} re-opened", self.provider);
            }
            CircuitState::Open => {}
        }
    }

    pub fn error_rate(&self) -> f64 {
        if self.request_count == 0 {
            0.0
        } else {
            (self.request_count - self.success_count) as f64 / self.request_count as f64
        }
    }
}

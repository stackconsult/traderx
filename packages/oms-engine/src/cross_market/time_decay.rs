use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tracing::{info, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

/// Time decay parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayParams {
    pub short_decay_hours: i64,
    pub medium_decay_hours: i64,
    pub long_decay_hours: i64,
    pub base_decay: f64,
}

impl Default for DecayParams {
    fn default() -> Self {
        Self {
            short_decay_hours: 1,
            medium_decay_hours: 24,
            long_decay_hours: 168, // 1 week
            base_decay: 0.95,
        }
    }
}

/// Time decay adjustment for asset classes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayAdjustment {
    pub equity: f64,
    pub fixed_income: f64,
    pub fx: f64,
    pub crypto: f64,
    pub commodity: f64,
}

/// Time decay model for signal aging
pub struct TimeDecayModel {
    params: DecayParams,
    signal_timestamps: HashMap<String, DateTime<Utc>>,
}

impl TimeDecayModel {
    pub fn new(params: DecayParams) -> Self {
        Self {
            params,
            signal_timestamps: HashMap::new(),
        }
    }

    /// Update signal timestamp for an asset
    pub fn update_signal(&mut self, asset: String, timestamp: DateTime<Utc>) {
        self.signal_timestamps.insert(asset, timestamp);
    }

    /// Get time decay adjustment for all asset classes
    pub fn get_decay_adjustment(&self) -> DecayAdjustment {
        let now = Utc::now();
        
        let equity_decay = self.calculate_decay("equity", now);
        let fi_decay = self.calculate_decay("fixed_income", now);
        let fx_decay = self.calculate_decay("fx", now);
        let crypto_decay = self.calculate_decay("crypto", now);
        let commodity_decay = self.calculate_decay("commodity", now);

        DecayAdjustment {
            equity: equity_decay,
            fixed_income: fi_decay,
            fx: fx_decay,
            crypto: crypto_decay,
            commodity: commodity_decay,
        }
    }

    fn calculate_decay(&self, asset: &str, now: DateTime<Utc>) -> f64 {
        if let Some(timestamp) = self.signal_timestamps.get(asset) {
            let elapsed = now.signed_duration_since(*timestamp);
            let hours = elapsed.num_hours();
            
            let decay_factor = if hours < self.params.short_decay_hours {
                1.0
            } else if hours < self.params.medium_decay_hours {
                self.params.base_decay.powf((hours as f64) / self.params.medium_decay_hours as f64)
            } else if hours < self.params.long_decay_hours {
                self.params.base_decay.powf((hours as f64) / self.params.long_decay_hours as f64)
            } else {
                self.params.base_decay.powf((self.params.long_decay_hours as f64) / self.params.long_decay_hours as f64) * 0.5
            };
            
            decay_factor.max(0.1)
        } else {
            1.0
        }
    }
}

impl Default for TimeDecayModel {
    fn default() -> Self {
        Self::new(DecayParams::default())
    }
}

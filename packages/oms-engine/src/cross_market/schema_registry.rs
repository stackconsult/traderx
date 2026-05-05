use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Market class identifier — matches BamDomain encoding where applicable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketClass {
    Equities = 0,
    FX = 1,
    Metals = 2,
    Commodities = 3,
    Crypto = 4,
    Indices = 5,
}

impl From<u8> for MarketClass {
    fn from(value: u8) -> Self {
        match value {
            0 => MarketClass::Equities,
            1 => MarketClass::FX,
            2 => MarketClass::Metals,
            3 => MarketClass::Commodities,
            4 => MarketClass::Crypto,
            _ => MarketClass::Indices,
        }
    }
}

impl From<usize> for MarketClass {
    fn from(value: usize) -> Self {
        MarketClass::from(value as u8)
    }
}

/// Trading hours specification per market class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradingHours {
    pub pre_market_start_utc: u16,   // minutes from midnight UTC
    pub market_open_utc: u16,
    pub market_close_utc: u16,
    pub after_hours_end_utc: u16,
    pub is_24h: bool,
}

/// Per-market-class schema defining cell interpretation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSchema {
    pub market_class: MarketClass,
    pub tick_size: f64,
    pub lot_size: f64,
    pub price_decimal_places: u8,
    pub qty_decimal_places: u8,
    pub trading_hours: TradingHours,
    pub venue_count: u8,
    /// Bitflags for market-specific behavior
    /// bit 0 = supports short selling
    /// bit 1 = supports margin
    /// bit 2 = has central clearing
    /// bit 3 = has options
    /// bit 4 = futures available
    /// bit 5 = has pre/after market
    pub special_flags: u16,
}

/// Cross-market correlation schema — pre-computed from historical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationSchema {
    /// Average lead-lag in microseconds: positive = row leads col
    pub lead_lag_matrix_us: [[i64; 6]; 6],
    /// Correlation strength 0.0–1.0
    pub correlation_matrix: [[f64; 6]; 6],
    /// How often to recompute (seconds)
    pub recompute_interval: u64,
    /// Schema version for invalidation
    pub version: String,
}

impl Default for CorrelationSchema {
    fn default() -> Self {
        Self {
            lead_lag_matrix_us: [
                [0, 150_000, 200_000, 300_000, 500_000, 100_000],
                [-150_000, 0, 100_000, 250_000, 400_000, 80_000],
                [-200_000, -100_000, 0, 150_000, 600_000, 120_000],
                [-300_000, -250_000, -150_000, 0, 700_000, 200_000],
                [-500_000, -400_000, -600_000, -700_000, 0, 300_000],
                [-100_000, -80_000, -120_000, -200_000, -300_000, 0],
            ],
            correlation_matrix: [
                [1.0, 0.65, 0.40, 0.30, 0.25, 0.85],
                [0.65, 1.0, 0.55, 0.35, 0.30, 0.60],
                [0.40, 0.55, 1.0, 0.45, 0.20, 0.35],
                [0.30, 0.35, 0.45, 1.0, 0.15, 0.30],
                [0.25, 0.30, 0.20, 0.15, 1.0, 0.28],
                [0.85, 0.60, 0.35, 0.30, 0.28, 1.0],
            ],
            recompute_interval: 3_600,
            version: "2025.05.04-v1".to_string(),
        }
    }
}

/// Schema registry — loaded once at startup, versioned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamSchemaRegistry {
    pub version: String,
    pub market_schemas: HashMap<MarketClass, MarketSchema>,
    pub correlation_schema: CorrelationSchema,
}

impl BamSchemaRegistry {
    pub fn new() -> Self {
        let mut market_schemas = HashMap::new();

        market_schemas.insert(MarketClass::Equities, MarketSchema {
            market_class: MarketClass::Equities,
            tick_size: 0.01,
            lot_size: 1.0,
            price_decimal_places: 2,
            qty_decimal_places: 0,
            trading_hours: TradingHours {
                pre_market_start_utc: 780,  // 13:00 UTC
                market_open_utc: 840,        // 14:00 UTC
                market_close_utc: 1200,      // 20:00 UTC
                after_hours_end_utc: 1320,   // 22:00 UTC
                is_24h: false,
            },
            venue_count: 16,
            special_flags: 0b0010_1111,
        });

        market_schemas.insert(MarketClass::FX, MarketSchema {
            market_class: MarketClass::FX,
            tick_size: 0.00001,
            lot_size: 100_000.0,
            price_decimal_places: 5,
            qty_decimal_places: 2,
            trading_hours: TradingHours {
                pre_market_start_utc: 0,
                market_open_utc: 0,
                market_close_utc: 0,
                after_hours_end_utc: 0,
                is_24h: true,
            },
            venue_count: 8,
            special_flags: 0b0000_0111,
        });

        market_schemas.insert(MarketClass::Metals, MarketSchema {
            market_class: MarketClass::Metals,
            tick_size: 0.01,
            lot_size: 100.0,
            price_decimal_places: 2,
            qty_decimal_places: 2,
            trading_hours: TradingHours {
                pre_market_start_utc: 720,   // 12:00 UTC
                market_open_utc: 780,        // 13:00 UTC
                market_close_utc: 1020,      // 17:00 UTC
                after_hours_end_utc: 1080,   // 18:00 UTC
                is_24h: false,
            },
            venue_count: 4,
            special_flags: 0b0001_0011,
        });

        market_schemas.insert(MarketClass::Commodities, MarketSchema {
            market_class: MarketClass::Commodities,
            tick_size: 0.01,
            lot_size: 1_000.0,
            price_decimal_places: 2,
            qty_decimal_places: 0,
            trading_hours: TradingHours {
                pre_market_start_utc: 780,
                market_open_utc: 840,
                market_close_utc: 1260,
                after_hours_end_utc: 1320,
                is_24h: false,
            },
            venue_count: 6,
            special_flags: 0b0011_0011,
        });

        market_schemas.insert(MarketClass::Crypto, MarketSchema {
            market_class: MarketClass::Crypto,
            tick_size: 0.00000001,
            lot_size: 1.0,
            price_decimal_places: 8,
            qty_decimal_places: 8,
            trading_hours: TradingHours {
                pre_market_start_utc: 0,
                market_open_utc: 0,
                market_close_utc: 0,
                after_hours_end_utc: 0,
                is_24h: true,
            },
            venue_count: 24,
            special_flags: 0b0000_0101,
        });

        market_schemas.insert(MarketClass::Indices, MarketSchema {
            market_class: MarketClass::Indices,
            tick_size: 0.01,
            lot_size: 1.0,
            price_decimal_places: 2,
            qty_decimal_places: 0,
            trading_hours: TradingHours {
                pre_market_start_utc: 780,
                market_open_utc: 840,
                market_close_utc: 1200,
                after_hours_end_utc: 1320,
                is_24h: false,
            },
            venue_count: 4,
            special_flags: 0b0000_0011,
        });

        Self {
            version: "2025.05.04-v1".to_string(),
            market_schemas,
            correlation_schema: CorrelationSchema::default(),
        }
    }

    pub fn get(&self, class: MarketClass) -> Option<&MarketSchema> {
        self.market_schemas.get(&class)
    }

    pub fn lead_lag_us(&self, from: MarketClass, to: MarketClass) -> i64 {
        self.correlation_schema.lead_lag_matrix_us[from as usize][to as usize]
    }

    pub fn correlation(&self, a: MarketClass, b: MarketClass) -> f64 {
        self.correlation_schema.correlation_matrix[a as usize][b as usize]
    }
}

impl Default for BamSchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_registry_loads() {
        let reg = BamSchemaRegistry::new();
        assert_eq!(reg.market_schemas.len(), 6);
        assert!(reg.get(MarketClass::Equities).is_some());
        assert!(reg.get(MarketClass::Crypto).is_some());
    }

    #[test]
    fn test_correlation_matrix_bounds() {
        let reg = BamSchemaRegistry::new();
        for i in 0..6 {
            for j in 0..6 {
                let c = reg.correlation(MarketClass::from(i), MarketClass::from(j));
                assert!(c >= 0.0 && c <= 1.0, "correlation out of bounds at ({},{})", i, j);
            }
        }
    }

    #[test]
    fn test_crypto_is_24h() {
        let reg = BamSchemaRegistry::new();
        let crypto = reg.get(MarketClass::Crypto).unwrap();
        assert!(crypto.trading_hours.is_24h);
    }
}

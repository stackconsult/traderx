use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetClass { Stock, Crypto, Metal, Commodity }

impl std::fmt::Display for AssetClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetClass::Stock => write!(f, "Stock"),
            AssetClass::Crypto => write!(f, "Crypto"),
            AssetClass::Metal => write!(f, "Metal"),
            AssetClass::Commodity => write!(f, "Commodity"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssetConfig {
    pub symbol: String,
    pub asset_class: AssetClass,
    pub base_price: f64,
    pub daily_volatility: f64,
    pub avg_volume: f64,
    pub spread_bps: f64,
    pub tick_size: f64,
}

pub fn default_universe() -> Vec<AssetConfig> {
    vec![
        AssetConfig { symbol: "AAPL".to_string(), asset_class: AssetClass::Stock, base_price: 195.0, daily_volatility: 0.22, avg_volume: 55_000_000.0, spread_bps: 0.5, tick_size: 0.01 },
        AssetConfig { symbol: "MSFT".to_string(), asset_class: AssetClass::Stock, base_price: 415.0, daily_volatility: 0.20, avg_volume: 28_000_000.0, spread_bps: 0.5, tick_size: 0.01 },
        AssetConfig { symbol: "TSLA".to_string(), asset_class: AssetClass::Stock, base_price: 175.0, daily_volatility: 0.55, avg_volume: 110_000_000.0, spread_bps: 1.0, tick_size: 0.01 },
        AssetConfig { symbol: "NVDA".to_string(), asset_class: AssetClass::Stock, base_price: 138.0, daily_volatility: 0.45, avg_volume: 250_000_000.0, spread_bps: 0.8, tick_size: 0.01 },
        AssetConfig { symbol: "AMZN".to_string(), asset_class: AssetClass::Stock, base_price: 200.0, daily_volatility: 0.28, avg_volume: 45_000_000.0, spread_bps: 0.6, tick_size: 0.01 },
        AssetConfig { symbol: "GOOGL".to_string(), asset_class: AssetClass::Stock, base_price: 178.0, daily_volatility: 0.25, avg_volume: 22_000_000.0, spread_bps: 0.6, tick_size: 0.01 },
        AssetConfig { symbol: "META".to_string(), asset_class: AssetClass::Stock, base_price: 595.0, daily_volatility: 0.32, avg_volume: 18_000_000.0, spread_bps: 0.7, tick_size: 0.01 },
        AssetConfig { symbol: "JPM".to_string(), asset_class: AssetClass::Stock, base_price: 245.0, daily_volatility: 0.18, avg_volume: 12_000_000.0, spread_bps: 0.4, tick_size: 0.01 },
        AssetConfig { symbol: "BTC-USD".to_string(), asset_class: AssetClass::Crypto, base_price: 88_000.0, daily_volatility: 0.65, avg_volume: 35_000_000_000.0, spread_bps: 2.0, tick_size: 0.01 },
        AssetConfig { symbol: "ETH-USD".to_string(), asset_class: AssetClass::Crypto, base_price: 2_150.0, daily_volatility: 0.78, avg_volume: 18_000_000_000.0, spread_bps: 3.0, tick_size: 0.01 },
        AssetConfig { symbol: "SOL-USD".to_string(), asset_class: AssetClass::Crypto, base_price: 142.0, daily_volatility: 0.95, avg_volume: 4_500_000_000.0, spread_bps: 5.0, tick_size: 0.001 },
        AssetConfig { symbol: "XRP-USD".to_string(), asset_class: AssetClass::Crypto, base_price: 2.35, daily_volatility: 1.10, avg_volume: 3_200_000_000.0, spread_bps: 8.0, tick_size: 0.0001 },
        AssetConfig { symbol: "XAUUSD".to_string(), asset_class: AssetClass::Metal, base_price: 2_850.0, daily_volatility: 0.14, avg_volume: 25_000_000_000.0, spread_bps: 1.5, tick_size: 0.01 },
        AssetConfig { symbol: "XAGUSD".to_string(), asset_class: AssetClass::Metal, base_price: 32.0, daily_volatility: 0.22, avg_volume: 8_000_000_000.0, spread_bps: 2.5, tick_size: 0.001 },
        AssetConfig { symbol: "HG".to_string(), asset_class: AssetClass::Metal, base_price: 4.35, daily_volatility: 0.18, avg_volume: 2_000_000_000.0, spread_bps: 3.0, tick_size: 0.0005 },
        AssetConfig { symbol: "CL".to_string(), asset_class: AssetClass::Commodity, base_price: 68.0, daily_volatility: 0.32, avg_volume: 1_500_000_000.0, spread_bps: 2.0, tick_size: 0.01 },
        AssetConfig { symbol: "NG".to_string(), asset_class: AssetClass::Commodity, base_price: 3.40, daily_volatility: 0.45, avg_volume: 800_000_000.0, spread_bps: 4.0, tick_size: 0.001 },
        AssetConfig { symbol: "ZC".to_string(), asset_class: AssetClass::Commodity, base_price: 4.80, daily_volatility: 0.20, avg_volume: 400_000_000.0, spread_bps: 2.5, tick_size: 0.0025 },
        AssetConfig { symbol: "ZS".to_string(), asset_class: AssetClass::Commodity, base_price: 10.20, daily_volatility: 0.22, avg_volume: 350_000_000.0, spread_bps: 3.0, tick_size: 0.0025 },
    ]
}

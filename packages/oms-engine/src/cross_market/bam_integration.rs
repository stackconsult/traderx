use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Cross-market BAM domain codes (5-bit binary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BamDomain {
    Equity = 0b00000,
    FixedIncome = 0b00001,
    Fx = 0b00010,
    Crypto = 0b00100,
    Commodity = 0b00101,
    Macro = 0b00110,
    Sentiment = 0b00111,
    HftSignals = 0b01100,
    Portfolio = 0b10000,
    CrossAsset = 0b01000,
}

/// BAM layer codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BamLayer {
    Prim = 0b00,
    Sub = 0b01,
    Type = 0b10,
    Axis = 0b11,
}

/// BAM signal - 10-bit binary address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamSignal {
    pub domain: BamDomain,
    pub layer: BamLayer,
    pub type_flag: u8,
    pub axis_flag: u8,
}

impl BamSignal {
    /// Encode BAM signal to 10-bit binary
    pub fn encode(&self) -> u16 {
        let domain_bits = (self.domain as u8) as u16;
        let layer_bits = (self.layer as u8) as u16;
        let type_bits = (self.type_flag & 0b11) as u16;
        let axis_bits = (self.axis_flag & 0b11) as u16;
        
        (domain_bits << 5) | (layer_bits << 3) | (type_bits << 1) | axis_bits
    }

    /// Decode BAM signal from 10-bit binary
    pub fn decode(value: u16) -> Self {
        let domain = (value >> 5) as u8;
        let layer = ((value >> 3) & 0b11) as u8;
        let type_flag = ((value >> 1) & 0b11) as u8;
        let axis_flag = (value & 0b1) as u8;

        BamSignal {
            domain: match domain {
                0 => BamDomain::Equity,
                1 => BamDomain::FixedIncome,
                2 => BamDomain::Fx,
                4 => BamDomain::Crypto,
                5 => BamDomain::Commodity,
                6 => BamDomain::Macro,
                7 => BamDomain::Sentiment,
                12 => BamDomain::HftSignals,
                16 => BamDomain::Portfolio,
                8 => BamDomain::CrossAsset,
                _ => BamDomain::Equity,
            },
            layer: match layer {
                0 => BamLayer::Prim,
                1 => BamLayer::Sub,
                2 => BamLayer::Type,
                _ => BamLayer::Axis,
            },
            type_flag,
            axis_flag,
        }
    }
}

/// Asymmetry pattern for cross-market correlation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AsymmetryPattern {
    EquityBond,
    EquityFx,
    FxCommodity,
    CryptoEquity,
    CryptoFx,
    MacroHft,
}

/// BAM integration for cross-market analytics
pub struct BamCrossMarketIntegration {
    domain_map: HashMap<String, BamDomain>,
    asymmetry_patterns: HashMap<AsymmetryPattern, (BamDomain, BamDomain)>,
    signal_cache: HashMap<u16, DateTime<Utc>>,
}

impl BamCrossMarketIntegration {
    pub fn new() -> Self {
        let mut domain_map = HashMap::new();
        domain_map.insert("equity".to_string(), BamDomain::Equity);
        domain_map.insert("fixed_income".to_string(), BamDomain::FixedIncome);
        domain_map.insert("fx".to_string(), BamDomain::Fx);
        domain_map.insert("crypto".to_string(), BamDomain::Crypto);
        domain_map.insert("commodity".to_string(), BamDomain::Commodity);
        domain_map.insert("macro".to_string(), BamDomain::Macro);
        domain_map.insert("sentiment".to_string(), BamDomain::Sentiment);
        domain_map.insert("hft".to_string(), BamDomain::HftSignals);
        domain_map.insert("portfolio".to_string(), BamDomain::Portfolio);
        domain_map.insert("cross_asset".to_string(), BamDomain::CrossAsset);

        let mut asymmetry_patterns = HashMap::new();
        asymmetry_patterns.insert(AsymmetryPattern::EquityBond, (BamDomain::Equity, BamDomain::FixedIncome));
        asymmetry_patterns.insert(AsymmetryPattern::EquityFx, (BamDomain::Equity, BamDomain::Fx));
        asymmetry_patterns.insert(AsymmetryPattern::FxCommodity, (BamDomain::Fx, BamDomain::Commodity));
        asymmetry_patterns.insert(AsymmetryPattern::CryptoEquity, (BamDomain::Crypto, BamDomain::Equity));
        asymmetry_patterns.insert(AsymmetryPattern::CryptoFx, (BamDomain::Crypto, BamDomain::Fx));
        asymmetry_patterns.insert(AsymmetryPattern::MacroHft, (BamDomain::Macro, BamDomain::HftSignals));

        Self {
            domain_map,
            asymmetry_patterns,
            signal_cache: HashMap::new(),
        }
    }

    /// Get BAM signal for asset type
    pub fn get_signal_for_asset(&self, asset_type: &str, layer: BamLayer) -> BamSignal {
        let domain = self.domain_map.get(asset_type).copied().unwrap_or(BamDomain::Equity);
        BamSignal {
            domain,
            layer,
            type_flag: 0,
            axis_flag: 0,
        }
    }

    /// Get BAM signal for asymmetry pattern
    pub fn get_asymmetry_signal(&self, pattern: AsymmetryPattern) -> Option<(BamSignal, BamSignal)> {
        if let Some((domain1, domain2)) = self.asymmetry_patterns.get(&pattern) {
            Some((
                BamSignal {
                    domain: *domain1,
                    layer: BamLayer::Type,
                    type_flag: 1,
                    axis_flag: 0,
                },
                BamSignal {
                    domain: *domain2,
                    layer: BamLayer::Type,
                    type_flag: 1,
                    axis_flag: 0,
                },
            ))
        } else {
            None
        }
    }

    /// Encode weight vector to BAM signals
    pub fn encode_weights(&self, _weights: &super::weight_engine::WeightVector) -> Vec<(String, BamSignal)> {
        vec![
            ("equity".to_string(), self.get_signal_for_asset("equity", BamLayer::Prim)),
            ("fixed_income".to_string(), self.get_signal_for_asset("fixed_income", BamLayer::Prim)),
            ("fx".to_string(), self.get_signal_for_asset("fx", BamLayer::Prim)),
            ("crypto".to_string(), self.get_signal_for_asset("crypto", BamLayer::Prim)),
            ("commodity".to_string(), self.get_signal_for_asset("commodity", BamLayer::Prim)),
        ]
    }

    /// Check if signal is in cache (for diagonal shortcut activation)
    pub fn is_signal_cached(&self, signal: &BamSignal) -> bool {
        let encoded = signal.encode();
        self.signal_cache.contains_key(&encoded)
    }

    /// Cache signal for diagonal shortcut
    pub fn cache_signal(&mut self, signal: BamSignal) {
        let encoded = signal.encode();
        self.signal_cache.insert(encoded, Utc::now());
    }

    /// Get cross-market BAM routing for fusion result
    pub fn get_cross_market_routing(&self, result: &super::signal_fusion::FusionResult) -> Vec<BamSignal> {
        let mut signals = Vec::new();
        
        // Route to highest contributing domains
        let mut contributions: Vec<_> = result.contributing_signals.iter().collect();
        contributions.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        for (asset_type, _) in contributions.iter().take(3) {
            if let Some(domain) = self.domain_map.get(*asset_type) {
                signals.push(BamSignal {
                    domain: *domain,
                    layer: BamLayer::Axis,
                    type_flag: 1,
                    axis_flag: 1,
                });
            }
        }
        
        signals
    }
}

impl Default for BamCrossMarketIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bam_signal_encoding() {
        let signal = BamSignal {
            domain: BamDomain::Equity,
            layer: BamLayer::Prim,
            type_flag: 0,
            axis_flag: 0,
        };
        
        let encoded = signal.encode();
        assert_eq!(encoded, 0b00000_00_0_0);
    }

    #[test]
    fn test_bam_signal_decoding() {
        let encoded = 0b00000_00_0_0;
        let decoded = BamSignal::decode(encoded);
        assert_eq!(decoded.domain, BamDomain::Equity);
        assert_eq!(decoded.layer, BamLayer::Prim);
    }

    #[test]
    fn test_asymmetry_patterns() {
        let integration = BamCrossMarketIntegration::new();
        let equity_bond = integration.get_asymmetry_signal(AsymmetryPattern::EquityBond);
        assert!(equity_bond.is_some());
    }
}

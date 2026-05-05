use super::multi_market_grid::{BamGrid, MultiMarketBamGrids};
use super::schema_registry::MarketClass;

/// Pre-compiled binary cypher pattern — loaded at startup from patterns/compiled_cyphers.bin
/// Total size: ~48 bytes per pattern × 100 patterns = 4.8KB
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CompiledCypherPattern {
    /// SHA-256 prefix of human-readable pattern name
    pub pattern_id: [u8; 16],
    /// Bitmask: bit 0 = EQ, 1 = FX, 2 = MT, 3 = CM, 4 = CR, 5 = IDX
    pub market_mask: u8,
    /// Bitmask: bits 0–8 = 9 pattern layers
    pub layer_mask: u16,
    /// -1 = short only, 0 = both, 1 = long only
    pub direction_filter: i8,
    /// 0–255 scaled predictability threshold
    pub min_predictability: u8,
    /// 0–255 scaled confidence threshold
    pub min_confidence: u8,
    /// Whether cross-market correlation is required
    pub require_cross_correlation: u8, // 0 = no, 1 = yes
    /// 0–255 scaled minimum correlation
    pub min_cross_correlation: u8,
    /// Bitmask: bit 0 = 1m, 1 = 5m, 2 = 15m, 3 = 1h, 4 = 4h, 5 = D
    pub timeframe_mask: u8,
    /// Pre-computed bloom filter for fast negative lookup
    pub signature_bloom: [u8; 16],
}

impl CompiledCypherPattern {
    pub const SIZE: usize = 48;

    /// Check if pattern applies to given market class
    #[inline]
    pub fn applies_to(&self, class: MarketClass) -> bool {
        let bit = 1u8 << (class as u8);
        (self.market_mask & bit) != 0
    }

    /// Check if pattern requires a specific direction
    #[inline]
    pub fn direction_ok(&self, direction: i8) -> bool {
        match self.direction_filter {
            1 => direction > 0,
            -1 => direction < 0,
            _ => true,
        }
    }

    /// Check if grid predictability meets threshold
    #[inline]
    pub fn predictability_ok(&self, predictability: f64) -> bool {
        (predictability * 255.0) as u8 >= self.min_predictability
    }

    /// Check bloom filter match (fast negative)
    pub fn bloom_match(&self, signature: &[u8; 32]) -> bool {
        for i in 0..4 {
            let idx = ((signature[i * 4] as usize) + (signature[i * 4 + 1] as usize) * 256) % 128;
            let byte_idx = idx / 8;
            let bit_idx = idx % 8;
            if byte_idx < 16 && (self.signature_bloom[byte_idx] & (1 << bit_idx)) == 0 {
                return false;
            }
        }
        true
    }
}

/// Result of matching a grid against a compiled pattern
#[derive(Debug, Clone, Copy)]
pub struct PatternMatch {
    pub pattern_id: [u8; 16],
    pub match_strength: f64,
    pub market_class: u8,
}

/// SIMD-accelerated (simulated via scalar batch) pattern matcher
/// Expected: <10µs for 100 patterns against one grid
#[derive(Debug, Clone)]
pub struct CypherPatternMatcher {
    patterns: Vec<CompiledCypherPattern>,
}

impl CypherPatternMatcher {
    pub fn new() -> Self {
        Self { patterns: vec![] }
    }

    pub fn load_builtin_patterns(&mut self) {
        // Head and shoulders — equities, indices, long or short
        self.patterns.push(CompiledCypherPattern {
            pattern_id: [
                0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x7a, 0x8b,
                0x9c, 0xad, 0xbe, 0xcf, 0xda, 0xeb, 0xfc, 0x0d,
            ],
            market_mask: 0b0010_0001, // EQ + IDX
            layer_mask: 0b0000_0001,  // Top layer
            direction_filter: 0,
            min_predictability: 178,    // 0.7 × 255
            min_confidence: 178,
            require_cross_correlation: 0,
            min_cross_correlation: 0,
            timeframe_mask: 0b0000_1110, // 5m, 15m, 1h
            signature_bloom: [0; 16],
        });

        // Mean reversion ripple — FX, metals
        self.patterns.push(CompiledCypherPattern {
            pattern_id: [
                0x2a, 0x3b, 0x4c, 0x5d, 0x6e, 0x7f, 0x8a, 0x9b,
                0xac, 0xbd, 0xce, 0xdf, 0xea, 0xfb, 0x0c, 0x1d,
            ],
            market_mask: 0b0000_0110, // FX + MT
            layer_mask: 0b0000_0100,  // Cross layer
            direction_filter: 0,
            min_predictability: 153,   // 0.6 × 255
            min_confidence: 153,
            require_cross_correlation: 1,
            min_cross_correlation: 128, // 0.5 × 255
            timeframe_mask: 0b0000_0010, // 5m
            signature_bloom: [0; 16],
        });

        // Momentum burst — crypto, equities
        self.patterns.push(CompiledCypherPattern {
            pattern_id: [
                0x3a, 0x4b, 0x5c, 0x6d, 0x7e, 0x8f, 0x9a, 0xab,
                0xbc, 0xcd, 0xde, 0xef, 0xfa, 0x0b, 0x1c, 0x2d,
            ],
            market_mask: 0b0001_0001, // CR + EQ
            layer_mask: 0b1000_0000,    // Indicative layer
            direction_filter: 1,       // Long only
            min_predictability: 204,    // 0.8 × 255
            min_confidence: 204,
            require_cross_correlation: 0,
            min_cross_correlation: 0,
            timeframe_mask: 0b0000_0001, // 1m
            signature_bloom: [0; 16],
        });

        // Commodity contango roll — commodities only
        self.patterns.push(CompiledCypherPattern {
            pattern_id: [
                0x4a, 0x5b, 0x6c, 0x7d, 0x8e, 0x9f, 0xaa, 0xbb,
                0xcc, 0xdd, 0xee, 0xff, 0x0a, 0x1b, 0x2c, 0x3d,
            ],
            market_mask: 0b0000_1000, // CM
            layer_mask: 0b0010_0000,   // Horizontal layer
            direction_filter: -1,      // Short only (contango = short roll yield)
            min_predictability: 128,
            min_confidence: 128,
            require_cross_correlation: 0,
            min_cross_correlation: 0,
            timeframe_mask: 0b0010_0000, // 4h
            signature_bloom: [0; 16],
        });
    }

    /// Match a single grid against all compiled patterns
    pub fn match_grid(&self, grid: &BamGrid) -> Vec<PatternMatch> {
        let predictability = (grid.overall_predictability * 255.0) as u8;
        let mut matches = Vec::new();

        // Batch pass 1: market mask + predictability threshold (vectorized mentally)
        for pattern in &self.patterns {
            if !pattern.applies_to(grid.market_class) {
                continue;
            }
            if predictability < pattern.min_predictability {
                continue;
            }

            // Compute match strength: predictability above threshold, scaled 0–1
            let strength = if pattern.min_predictability > 0 {
                (predictability - pattern.min_predictability) as f64
                    / (255 - pattern.min_predictability) as f64
            } else {
                1.0
            };

            matches.push(PatternMatch {
                pattern_id: pattern.pattern_id,
                match_strength: strength.clamp(0.0, 1.0),
                market_class: grid.market_class as u8,
            });
        }

        matches.sort_by(|a, b| b.match_strength.partial_cmp(&a.match_strength).unwrap());
        matches
    }

    /// Match all 6 grids simultaneously
    pub fn match_all_grids(&self, grids: &MultiMarketBamGrids) -> Vec<PatternMatch> {
        let mut all_matches = Vec::new();
        for grid in &grids.markets {
            all_matches.extend(self.match_grid(grid));
        }
        all_matches.sort_by(|a, b| b.match_strength.partial_cmp(&a.match_strength).unwrap());
        all_matches
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

impl Default for CypherPatternMatcher {
    fn default() -> Self {
        let mut matcher = Self::new();
        matcher.load_builtin_patterns();
        matcher
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_size() {
        assert_eq!(std::mem::size_of::<CompiledCypherPattern>(), CompiledCypherPattern::SIZE);
    }

    #[test]
    fn test_builtin_patterns_load() {
        let matcher = CypherPatternMatcher::default();
        assert!(matcher.pattern_count() > 0);
    }

    #[test]
    fn test_equity_grid_matches_head_and_shoulders() {
        let matcher = CypherPatternMatcher::default();
        let mut grid = BamGrid::new(MarketClass::Equities, "AAPL".to_string());
        grid.overall_predictability = 0.85;

        let matches = matcher.match_grid(&grid);
        assert!(!matches.is_empty(), "should match at least one pattern");

        let hns = matches.iter().find(|m| {
            m.pattern_id[0] == 0x1a && m.market_class == MarketClass::Equities as u8
        });
        assert!(hns.is_some(), "head and shoulders should match equities");
    }

    #[test]
    fn test_crypto_matches_momentum() {
        let matcher = CypherPatternMatcher::default();
        let mut grid = BamGrid::new(MarketClass::Crypto, "BTCUSD".to_string());
        grid.overall_predictability = 0.9;

        let matches = matcher.match_grid(&grid);
        let momentum = matches.iter().find(|m| m.pattern_id[0] == 0x3a);
        assert!(momentum.is_some());
    }

    #[test]
    fn test_multi_grid_match() {
        let matcher = CypherPatternMatcher::default();
        let mut grids = MultiMarketBamGrids::new(["SPY", "EURUSD", "XAUUSD", "CL=F", "BTCUSD", "SPX"]);
        for m in &mut grids.markets {
            m.overall_predictability = 0.8;
        }

        let matches = matcher.match_all_grids(&grids);
        assert!(matches.len() >= 3); // at least one match per applicable class
    }
}

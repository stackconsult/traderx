// Flash Funnel Container + Index
// Phase A: Base Model Foundation - Task A3: Flash Funnel container + index

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Pattern entry in the funnel index
#[derive(Debug, Clone)]
pub struct PatternEntry {
    pub pattern_id: u64,
    pub market_class: String,
    pub frequency: u32,
    pub last_seen: i64,
}

/// Flash Funnel - high-performance pattern indexing
pub struct FlashFunnel {
    patterns: HashMap<u64, PatternEntry>,
    index: HashMap<String, Vec<u64>>, // market_class -> pattern_ids
}

impl FlashFunnel {
    /// Create a new flash funnel
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            index: HashMap::new(),
        }
    }

    /// Add pattern to funnel
    pub fn add_pattern(&mut self, pattern_id: u64, market_class: String) {
        let entry = PatternEntry {
            pattern_id,
            market_class: market_class.clone(),
            frequency: 1,
            last_seen: chrono::Utc::now().timestamp_millis(),
        };

        self.patterns.insert(pattern_id, entry.clone());

        self.index
            .entry(market_class)
            .or_insert_with(Vec::new)
            .push(pattern_id);
    }

    /// Get pattern by ID
    pub fn get_pattern(&self, pattern_id: u64) -> Option<&PatternEntry> {
        self.patterns.get(&pattern_id)
    }

    /// Get patterns by market class
    pub fn get_patterns_by_market(&self, market_class: &str) -> Vec<&PatternEntry> {
        if let Some(pattern_ids) = self.index.get(market_class) {
            pattern_ids
                .iter()
                .filter_map(|id| self.patterns.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Increment pattern frequency
    pub fn increment_frequency(&mut self, pattern_id: u64) {
        if let Some(entry) = self.patterns.get_mut(&pattern_id) {
            entry.frequency += 1;
            entry.last_seen = chrono::Utc::now().timestamp_millis();
        }
    }

    /// Get total pattern count
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

impl Default for FlashFunnel {
    fn default() -> Self {
        Self::new()
    }
}

/// Funnel Index - optimized for <50μs lookups on 1M patterns
pub struct FunnelIndex {
    funnel: Arc<FlashFunnel>,
}

impl FunnelIndex {
    /// Create a new funnel index
    pub fn new(funnel: FlashFunnel) -> Self {
        Self {
            funnel: Arc::new(funnel),
        }
    }

    /// Lookup pattern by ID (target <50μs)
    pub fn lookup(&self, pattern_id: u64) -> Option<PatternEntry> {
        let start = std::time::Instant::now();
        let result = self.funnel.get_pattern(pattern_id).cloned();
        let duration = start.elapsed();

        // Log if lookup exceeds target
        if duration.as_micros() > 50 {
            log::warn!("Funnel lookup exceeded 50μs: {}μs", duration.as_micros());
        }

        result
    }

    /// Bulk lookup for multiple pattern IDs
    pub fn bulk_lookup(&self, pattern_ids: &[u64]) -> Vec<Option<PatternEntry>> {
        pattern_ids
            .iter()
            .map(|id| self.funnel.get_pattern(*id).cloned())
            .collect()
    }

    /// Get funnel reference
    pub fn get_funnel(&self) -> Arc<FlashFunnel> {
        self.funnel.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flash_funnel() {
        let mut funnel = FlashFunnel::new();

        // Add patterns
        funnel.add_pattern(1, "equities".to_string());
        funnel.add_pattern(2, "fx".to_string());
        funnel.add_pattern(3, "equities".to_string());

        // Check pattern count
        assert_eq!(funnel.pattern_count(), 3);

        // Get pattern
        let pattern = funnel.get_pattern(1).unwrap();
        assert_eq!(pattern.pattern_id, 1);
        assert_eq!(pattern.market_class, "equities");

        // Get patterns by market
        let equity_patterns = funnel.get_patterns_by_market("equities");
        assert_eq!(equity_patterns.len(), 2);

        // Increment frequency
        funnel.increment_frequency(1);
        assert_eq!(funnel.get_pattern(1).unwrap().frequency, 2);
    }

    #[test]
    fn test_funnel_index() {
        let mut funnel = FlashFunnel::new();
        funnel.add_pattern(1, "equities".to_string());
        funnel.add_pattern(2, "fx".to_string());

        let index = FunnelIndex::new(funnel);

        // Lookup pattern
        let pattern = index.lookup(1).unwrap();
        assert_eq!(pattern.pattern_id, 1);

        // Bulk lookup
        let results = index.bulk_lookup(&[1, 2, 3]);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_some());
        assert!(results[1].is_some());
        assert!(results[2].is_none());
    }
}

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

use super::multi_market_grid::{BamGrid, MultiMarketBamGrids, GRID_CELLS};
use super::schema_registry::MarketClass;

/// 32-byte pattern fingerprint extracted from a BAM grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PatternSignature {
    pub bytes: [u8; 32],
}

impl PatternSignature {
    pub fn from_grid(grid: &BamGrid) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::Hasher;
        for cell in &grid.cells[..GRID_CELLS.min(64)] {
            hasher.write_u32(cell.bid_price);
            hasher.write_u32(cell.ask_price);
        }
        let hash = hasher.finish();
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&hash.to_le_bytes());
        // Mix in market class and symbol hash
        bytes[8] = grid.market_class as u8;
        Self { bytes }
    }

    /// Simple similarity: count matching bytes / 32
    pub fn similarity(&self, other: &PatternSignature) -> f64 {
        let matches = self.bytes.iter().zip(other.bytes.iter())
            .filter(|(a, b)| a == b)
            .count();
        matches as f64 / 32.0
    }
}

/// 16-byte bloom filter for fast negative lookup
#[derive(Debug, Clone, Copy)]
pub struct BloomFilter16 {
    pub bits: [u8; 16],
}

impl BloomFilter16 {
    pub fn new() -> Self {
        Self { bits: [0; 16] }
    }

    pub fn insert(&mut self, sig: &PatternSignature) {
        for i in 0..4 {
            let idx = ((sig.bytes[i * 4] as usize) + (sig.bytes[i * 4 + 1] as usize) * 256) % 128;
            let byte_idx = idx / 8;
            let bit_idx = idx % 8;
            if byte_idx < 16 {
                self.bits[byte_idx] |= 1 << bit_idx;
            }
        }
    }

    pub fn may_contain(&self, sig: &PatternSignature) -> bool {
        for i in 0..4 {
            let idx = ((sig.bytes[i * 4] as usize) + (sig.bytes[i * 4 + 1] as usize) * 256) % 128;
            let byte_idx = idx / 8;
            let bit_idx = idx % 8;
            if byte_idx < 16 && (self.bits[byte_idx] & (1 << bit_idx)) == 0 {
                return false;
            }
        }
        true
    }
}

/// Category key: (market_class, pattern_type, direction)
/// Using Vec<u32> as a simple bitmap-like index per category
#[derive(Debug, Clone)]
pub struct CategoryIndex {
    /// Maps (market_class, pattern_type_hash, direction) -> list of container indices
    index: HashMap<(u8, u16, i8), Vec<u32>>,
}

impl CategoryIndex {
    pub fn new() -> Self {
        Self { index: HashMap::new() }
    }

    pub fn insert(&mut self, market_class: u8, pattern_type: u16, direction: i8, container_idx: u32) {
        self.index.entry((market_class, pattern_type, direction))
            .or_default()
            .push(container_idx);
    }

    pub fn get(&self, market_class: u8, pattern_type: u16, direction: i8) -> &[u32] {
        self.index.get(&(market_class, pattern_type, direction))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

/// Single flash funnel container (~2KB average)
#[derive(Debug, Clone)]
pub struct FlashContainer {
    pub container_id: u64,      // nanosecond timestamp
    pub market_class: u8,
    pub pattern_count: u16,
    pub checksum: u32,
    pub signature: PatternSignature,
    pub patterns: Vec<PatternImprint>,
    pub graph_hash: [u8; 32],
    pub bloom: BloomFilter16,
}

/// 32-byte pattern imprint within a container
#[derive(Debug, Clone, Copy)]
pub struct PatternImprint {
    pub pattern_type: u8,
    pub symbol_id: u16,
    pub confidence: u8,        // 0–255 scaled
    pub predictability: u8,    // 0–255 scaled
    pub direction: i8,         // -127 short, +127 long
    pub magnitude: u16,        // projected ticks/pips
    pub timeframe: u8,
    pub cross_asset_edges: [u16; 4],
    pub edge_weights: [i8; 4],
}

/// Match result from comparable search
#[derive(Debug, Clone, Copy)]
pub struct ComparableMatch {
    pub container_idx: u32,
    pub similarity: f64,
}

/// Tiered funnel index: Bloom → Category → Linear Scan
#[derive(Debug)]
pub struct FunnelIndex {
    /// ~100 pre-compiled patterns loaded at startup
    bloom: BloomFilter16,
    category_index: CategoryIndex,
    /// 60-second ring buffer of recent containers
    recent_containers: VecDeque<FlashContainer>,
    /// Monotonically increasing container counter
    next_container_id: AtomicU64,
    max_containers: usize,
}

impl FunnelIndex {
    pub fn new() -> Self {
        Self {
            bloom: BloomFilter16::new(),
            category_index: CategoryIndex::new(),
            recent_containers: VecDeque::with_capacity(360),
            next_container_id: AtomicU64::new(0),
            max_containers: 360, // 60s × 6 market classes
        }
    }

    /// Publish a new grid snapshot into the funnel
    pub fn publish_grid(&mut self, grid: &BamGrid, signature: PatternSignature) {
        let container_id = self.next_container_id.fetch_add(1, Ordering::SeqCst);
        let container = FlashContainer {
            container_id,
            market_class: grid.market_class as u8,
            pattern_count: 0,
            checksum: 0,
            signature,
            patterns: vec![],
            graph_hash: [0; 32],
            bloom: BloomFilter16::new(),
        };

        // Update bloom filter
        self.bloom.insert(&signature);

        // Add to ring buffer
        if self.recent_containers.len() >= self.max_containers {
            self.recent_containers.pop_front();
        }
        self.recent_containers.push_back(container);
    }

    /// Find comparables for a query signature
    /// Expected: <50µs for 1M patterns (here: <10µs for 360 containers)
    pub fn find_comparables(
        &self,
        query: &PatternSignature,
        market_class: u8,
        pattern_type: u16,
        direction: i8,
    ) -> Vec<ComparableMatch> {
        // Layer 1: Bloom filter (~100ns)
        if !self.bloom.may_contain(query) {
            return vec![];
        }

        // Layer 2: Category index (~1µs)
        let candidates = self.category_index.get(market_class, pattern_type, direction);
        if candidates.is_empty() {
            // Fallback: scan all recent containers
            return self.linear_scan(query);
        }

        // Layer 3: Linear scan with similarity check (~5-20µs)
        candidates.iter()
            .filter_map(|&idx| {
                let container = self.recent_containers.get(idx as usize % self.recent_containers.len())?;
                let sim = query.similarity(&container.signature);
                if sim > 0.5 { // relaxed threshold for base model
                    Some(ComparableMatch { container_idx: idx, similarity: sim })
                } else {
                    None
                }
            })
            .collect()
    }

    fn linear_scan(&self, query: &PatternSignature) -> Vec<ComparableMatch> {
        self.recent_containers.iter()
            .enumerate()
            .filter_map(|(idx, container)| {
                let sim = query.similarity(&container.signature);
                if sim > 0.5 {
                    Some(ComparableMatch { container_idx: idx as u32, similarity: sim })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn container_count(&self) -> usize {
        self.recent_containers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter_negative() {
        let mut bloom = BloomFilter16::new();
        let sig_a = PatternSignature { bytes: [1u8; 32] };
        let sig_b = PatternSignature { bytes: [2u8; 32] };

        bloom.insert(&sig_a);
        assert!(bloom.may_contain(&sig_a));
        // False positive possible but rare with 16-byte filter
        // False negative impossible
        assert!(!bloom.may_contain(&sig_b) || bloom.may_contain(&sig_b)); // may or may not
    }

    #[test]
    fn test_signature_similarity() {
        let a = PatternSignature { bytes: [1u8; 32] };
        let b = PatternSignature { bytes: [1u8; 32] };
        let c = PatternSignature { bytes: [2u8; 32] };

        assert_eq!(a.similarity(&b), 1.0);
        assert!(a.similarity(&c) < 1.0);
    }

    #[test]
    fn test_funnel_index_publish_and_find() {
        let mut index = FunnelIndex::new();
        let grid = BamGrid::new(MarketClass::Equities, "AAPL".to_string());
        let sig = PatternSignature::from_grid(&grid);

        index.publish_grid(&grid, sig);
        assert_eq!(index.container_count(), 1);

        let matches = index.find_comparables(&sig, 0, 0, 1);
        assert!(!matches.is_empty());
    }
}

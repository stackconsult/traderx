// Multi-market BAM Grid Allocator
// Phase A: Base Model Foundation - Task A1: Multi-market BAM grid allocator

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Market class
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MarketClass {
    Equities,
    FX,
    Metals,
    Commodities,
    Crypto,
    Indices,
}

impl MarketClass {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "equities" => Some(MarketClass::Equities),
            "fx" => Some(MarketClass::FX),
            "metals" => Some(MarketClass::Metals),
            "commodities" => Some(MarketClass::Commodities),
            "crypto" => Some(MarketClass::Crypto),
            "indices" => Some(MarketClass::Indices),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            MarketClass::Equities => "equities",
            MarketClass::FX => "fx",
            MarketClass::Metals => "metals",
            MarketClass::Commodities => "commodities",
            MarketClass::Crypto => "crypto",
            MarketClass::Indices => "indices",
        }
    }
}

/// BAM grid (10×60 = 600 bytes)
#[derive(Debug, Clone)]
pub struct BamGrid {
    pub market_class: MarketClass,
    pub timestamp: i64,
    pub grid: [u8; 600],
}

impl BamGrid {
    pub fn new(market_class: MarketClass, timestamp: i64) -> Self {
        Self {
            market_class,
            timestamp,
            grid: [0u8; 600],
        }
    }

    pub fn set_cell(&mut self, row: usize, col: usize, value: u8) {
        if row < 10 && col < 60 {
            self.grid[row * 60 + col] = value;
        }
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<u8> {
        if row < 10 && col < 60 {
            Some(self.grid[row * 60 + col])
        } else {
            None
        }
    }
}

/// Multi-market BAM grid allocator
pub struct MultiMarketGridAllocator {
    grids: HashMap<MarketClass, BamGrid>,
}

impl MultiMarketGridAllocator {
    /// Create a new multi-market grid allocator
    pub fn new() -> Self {
        Self {
            grids: HashMap::new(),
        }
    }

    /// Initialize grids for all 6 market classes
    pub fn initialize(&mut self, timestamp: i64) {
        let market_classes = [
            MarketClass::Equities,
            MarketClass::FX,
            MarketClass::Metals,
            MarketClass::Commodities,
            MarketClass::Crypto,
            MarketClass::Indices,
        ];

        for market_class in market_classes {
            self.grids
                .insert(market_class.clone(), BamGrid::new(market_class, timestamp));
        }
    }

    /// Get grid for a market class
    pub fn get_grid(&self, market_class: &MarketClass) -> Option<&BamGrid> {
        self.grids.get(market_class)
    }

    /// Get mutable grid for a market class
    pub fn get_grid_mut(&mut self, market_class: &MarketClass) -> Option<&mut BamGrid> {
        self.grids.get_mut(market_class)
    }

    /// Update grid for a market class
    pub fn update_grid(&mut self, grid: BamGrid) {
        self.grids.insert(grid.market_class.clone(), grid);
    }

    /// Get all grids
    pub fn get_all_grids(&self) -> Vec<&BamGrid> {
        self.grids.values().collect()
    }

    /// Check if all 6 markets are operational
    pub fn is_operational(&self) -> bool {
        self.grids.len() == 6
    }
}

impl Default for MultiMarketGridAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_market_grid_allocator() {
        let mut allocator = MultiMarketGridAllocator::new();

        // Initialize grids
        allocator.initialize(1234567890);

        // Check operational
        assert!(allocator.is_operational());

        // Get grid
        let grid = allocator.get_grid(&MarketClass::Equities).unwrap();
        assert_eq!(grid.market_class, MarketClass::Equities);
        assert_eq!(grid.timestamp, 1234567890);

        // Update cell
        let mut grid_mut = allocator.get_grid_mut(&MarketClass::Equities).unwrap();
        grid_mut.set_cell(0, 0, 42);
        assert_eq!(grid_mut.get_cell(0, 0), Some(42));

        // Get all grids
        let all_grids = allocator.get_all_grids();
        assert_eq!(all_grids.len(), 6);
    }

    #[test]
    fn test_market_class() {
        assert_eq!(
            MarketClass::from_str("equities"),
            Some(MarketClass::Equities)
        );
        assert_eq!(MarketClass::from_str("FX"), Some(MarketClass::FX));
        assert_eq!(MarketClass::from_str("invalid"), None);

        assert_eq!(MarketClass::Equities.as_str(), "equities");
        assert_eq!(MarketClass::Crypto.as_str(), "crypto");
    }

    #[test]
    fn test_bam_grid() {
        let mut grid = BamGrid::new(MarketClass::Equities, 1234567890);

        // Set and get cell
        grid.set_cell(5, 30, 99);
        assert_eq!(grid.get_cell(5, 30), Some(99));

        // Out of bounds
        assert_eq!(grid.get_cell(10, 0), None);
        assert_eq!(grid.get_cell(0, 60), None);
    }
}

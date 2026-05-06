// Property-based Tests
// Phase A: Base Model Foundation - Task A8: Property-based tests

use proptest::prelude::*;
use crate::mesh::{BamGrid, MarketClass, MultiMarketGridAllocator};

/// Property: BAM grid cells should always be within 0-255 range
proptest! {
    #[test]
    fn prop_grid_cells_in_range(row in 0..10usize, col in 0..60usize, value in 0u8..=255u8) {
        let mut grid = BamGrid::new(MarketClass::Equities, 0);
        grid.set_cell(row, col, value);
        
        let retrieved = grid.get_cell(row, col);
        prop_assert_eq!(retrieved, Some(value));
    }
}

/// Property: Multi-market allocator should maintain grid count
proptest! {
    #[test]
    fn prop_multi_market_allocator_grid_count(timestamp in 0i64..=1_000_000_0000i64) {
        let mut allocator = MultiMarketGridAllocator::new();
        allocator.initialize(timestamp);
        
        prop_assert_eq!(allocator.get_all_grids().len(), 6);
        prop_assert!(allocator.is_operational());
    }
}

/// Property: Grid operations should be idempotent
proptest! {
    #[test]
    fn prop_grid_set_get_idempotent(row in 0..10usize, col in 0..60usize, value in 0u8..=255u8) {
        let mut grid = BamGrid::new(MarketClass::Equities, 0);
        
        grid.set_cell(row, col, value);
        grid.set_cell(row, col, value);
        
        let retrieved = grid.get_cell(row, col);
        prop_assert_eq!(retrieved, Some(value));
    }
}

/// Property: Grid should reject out-of-bounds access
proptest! {
    #[test]
    fn prop_grid_bounds_check(row in 10usize..=20usize, col in 0usize..=100usize) {
        let mut grid = BamGrid::new(MarketClass::Equities, 0);
        
        grid.set_cell(row, col, 42);
        let retrieved = grid.get_cell(row, col);
        
        prop_assert_eq!(retrieved, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_property_tests() {
        // Run property tests
        prop_grid_cells_in_range();
        prop_multi_market_allocator_grid_count();
        prop_grid_set_get_idempotent();
        prop_grid_bounds_check();
    }
}

// BamSchemaRegistry Loader
// Phase A: Base Model Foundation - Task A2: BamSchemaRegistry loader

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BAM schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamSchema {
    pub market_class: String,
    pub grid_dimensions: (usize, usize), // (rows, cols)
    pub cell_encoding: String,
    pub timestamp_field: String,
    pub metadata: HashMap<String, String>,
}

impl BamSchema {
    pub fn new(market_class: String) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());
        metadata.insert("encoding".to_string(), "binary".to_string());
        
        Self {
            market_class,
            grid_dimensions: (10, 60),
            cell_encoding: "uint8".to_string(),
            timestamp_field: "unix_ms".to_string(),
            metadata,
        }
    }
    
    pub fn grid_size(&self) -> usize {
        self.grid_dimensions.0 * self.grid_dimensions.1
    }
}

/// BamSchemaRegistry - manages BAM schemas for all markets
pub struct BamSchemaRegistry {
    schemas: HashMap<String, BamSchema>,
}

impl BamSchemaRegistry {
    /// Create a new schema registry
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }
    
    /// Load schema from configuration
    pub fn load_schema(&mut self, schema: BamSchema) {
        self.schemas.insert(schema.market_class.clone(), schema);
    }
    
    /// Get schema for a market class
    pub fn get_schema(&self, market_class: &str) -> Option<&BamSchema> {
        self.schemas.get(market_class)
    }
    
    /// Get mutable schema for a market class
    pub fn get_schema_mut(&mut self, market_class: &str) -> Option<&mut BamSchema> {
        self.schemas.get_mut(market_class)
    }
    
    /// Load default schemas for all 6 market classes
    pub fn load_defaults(&mut self) {
        let market_classes = [
            "equities", "fx", "metals", "commodities", "crypto", "indices"
        ];
        
        for market_class in market_classes {
            self.load_schema(BamSchema::new(market_class.to_string()));
        }
    }
    
    /// Validate schema consistency
    pub fn validate_consistency(&self) -> Result<(), String> {
        if self.schemas.is_empty() {
            return Err("No schemas loaded".to_string());
        }
        
        let expected_dims = (10, 60);
        let expected_encoding = "uint8";
        
        for (market_class, schema) in &self.schemas {
            if schema.grid_dimensions != expected_dims {
                return Err(format!("Market {} has invalid grid dimensions: {:?}", 
                                  market_class, schema.grid_dimensions));
            }
            
            if schema.cell_encoding != expected_encoding {
                return Err(format!("Market {} has invalid encoding: {}", 
                                  market_class, schema.cell_encoding));
            }
        }
        
        Ok(())
    }
    
    /// List all registered market classes
    pub fn list_market_classes(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
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
    fn test_bam_schema_registry() {
        let mut registry = BamSchemaRegistry::new();
        
        // Load defaults
        registry.load_defaults();
        
        // Check schemas loaded
        assert_eq!(registry.schemas.len(), 6);
        
        // Get schema
        let schema = registry.get_schema("equities").unwrap();
        assert_eq!(schema.market_class, "equities");
        assert_eq!(schema.grid_dimensions, (10, 60));
        assert_eq!(schema.grid_size(), 600);
        
        // Validate consistency
        assert!(registry.validate_consistency().is_ok());
    }
    
    #[test]
    fn test_bam_schema() {
        let schema = BamSchema::new("test".to_string());
        
        assert_eq!(schema.market_class, "test");
        assert_eq!(schema.grid_dimensions, (10, 60));
        assert_eq!(schema.grid_size(), 600);
        assert_eq!(schema.cell_encoding, "uint8");
        assert!(schema.metadata.contains_key("version"));
    }
    
    #[test]
    fn test_schema_validation() {
        let mut registry = BamSchemaRegistry::new();
        
        // Empty registry should fail validation
        assert!(registry.validate_consistency().is_err());
        
        // Load defaults
        registry.load_defaults();
        
        // Should pass validation
        assert!(registry.validate_consistency().is_ok());
    }
}

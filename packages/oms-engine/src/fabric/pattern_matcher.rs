// Compiled Cypher Pattern Matcher
// Phase A: Base Model Foundation - Task A4: Compiled cypher pattern matcher

use std::collections::HashMap;

/// Compiled pattern for fast matching
#[derive(Debug, Clone)]
pub struct CompiledPattern {
    pub pattern_id: u64,
    pub cypher: String,
    pub compiled_rules: Vec<PatternRule>,
}

/// Pattern rule for matching
#[derive(Debug, Clone)]
pub struct PatternRule {
    pub row: usize,
    pub col: usize,
    pub operator: PatternOperator,
    pub value: u8,
}

/// Pattern matching operators
#[derive(Debug, Clone, PartialEq)]
pub enum PatternOperator {
    Equal,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    NotEqual,
}

impl PatternOperator {
    pub fn matches(&self, actual: u8, expected: u8) -> bool {
        match self {
            PatternOperator::Equal => actual == expected,
            PatternOperator::GreaterThan => actual > expected,
            PatternOperator::LessThan => actual < expected,
            PatternOperator::GreaterOrEqual => actual >= expected,
            PatternOperator::LessOrEqual => actual <= expected,
            PatternOperator::NotEqual => actual != expected,
        }
    }
}

/// Pattern Matcher - compiled pattern matching for <200μs end-to-end
pub struct PatternMatcher {
    patterns: Vec<CompiledPattern>,
}

impl PatternMatcher {
    /// Create a new pattern matcher
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }
    
    /// Compile and add a pattern
    pub fn compile_pattern(&mut self, pattern_id: u64, cypher: String) -> Result<(), String> {
        let rules = self.parse_cypher(&cypher)?;
        let compiled = CompiledPattern {
            pattern_id,
            cypher,
            compiled_rules: rules,
        };
        self.patterns.push(compiled);
        Ok(())
    }
    
    /// Parse cypher string into pattern rules
    fn parse_cypher(&self, cypher: &str) -> Result<Vec<PatternRule>, String> {
        let mut rules = Vec::new();
        
        // Simple cypher parser: "row,col,op,value"
        // Example: "0,0,eq,42" means grid[0][0] == 42
        for part in cypher.split(';') {
            let tokens: Vec<&str> = part.split(',').collect();
            if tokens.len() != 4 {
                return Err(format!("Invalid cypher part: {}", part));
            }
            
            let row: usize = tokens[0].parse().map_err(|_| "Invalid row".to_string())?;
            let col: usize = tokens[1].parse().map_err(|_| "Invalid col".to_string())?;
            let operator = self.parse_operator(tokens[2])?;
            let value: u8 = tokens[3].parse().map_err(|_| "Invalid value".to_string())?;
            
            rules.push(PatternRule {
                row,
                col,
                operator,
                value,
            });
        }
        
        Ok(rules)
    }
    
    /// Parse operator string
    fn parse_operator(&self, op: &str) -> Result<PatternOperator, String> {
        match op.to_lowercase().as_str() {
            "eq" => Ok(PatternOperator::Equal),
            "gt" => Ok(PatternOperator::GreaterThan),
            "lt" => Ok(PatternOperator::LessThan),
            "ge" => Ok(PatternOperator::GreaterOrEqual),
            "le" => Ok(PatternOperator::LessOrEqual),
            "ne" => Ok(PatternOperator::NotEqual),
            _ => Err(format!("Unknown operator: {}", op)),
        }
    }
    
    /// Match pattern against grid (target <200μs end-to-end)
    pub fn match_pattern(&self, pattern_id: u64, grid: &[u8; 600]) -> bool {
        let start = std::time::Instant::now();
        
        let pattern = match self.patterns.iter().find(|p| p.pattern_id == pattern_id) {
            Some(p) => p,
            None => return false,
        };
        
        let mut matches = true;
        for rule in &pattern.compiled_rules {
            let idx = rule.row * 60 + rule.col;
            if idx >= 600 {
                matches = false;
                break;
            }
            
            if !rule.operator.matches(grid[idx], rule.value) {
                matches = false;
                break;
            }
        }
        
        let duration = start.elapsed();
        if duration.as_micros() > 200 {
            log::warn!("Pattern matching exceeded 200μs: {}μs", duration.as_micros());
        }
        
        matches
    }
    
    /// Match all patterns against grid
    pub fn match_all(&self, grid: &[u8; 600]) -> Vec<u64> {
        self.patterns.iter()
            .filter(|pattern| self.match_pattern(pattern.pattern_id, grid))
            .map(|pattern| pattern.pattern_id)
            .collect()
    }
    
    /// Get pattern count
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_matcher() {
        let mut matcher = PatternMatcher::new();
        
        // Compile a pattern
        matcher.compile_pattern(1, "0,0,eq,42;5,30,gt,10".to_string()).unwrap();
        
        // Create test grid
        let mut grid = [0u8; 600];
        grid[0] = 42;
        grid[5 * 60 + 30] = 20;
        
        // Match pattern
        assert!(matcher.match_pattern(1, &grid));
        
        // Change grid to not match
        grid[0] = 0;
        assert!(!matcher.match_pattern(1, &grid));
    }
    
    #[test]
    fn test_pattern_operator() {
        assert!(PatternOperator::Equal.matches(42, 42));
        assert!(PatternOperator::GreaterThan.matches(50, 42));
        assert!(PatternOperator::LessThan.matches(30, 42));
        assert!(PatternOperator::GreaterOrEqual.matches(42, 42));
        assert!(PatternOperator::LessOrEqual.matches(42, 42));
        assert!(PatternOperator::NotEqual.matches(40, 42));
    }
    
    #[test]
    fn test_match_all() {
        let mut matcher = PatternMatcher::new();
        matcher.compile_pattern(1, "0,0,eq,42".to_string()).unwrap();
        matcher.compile_pattern(2, "0,0,eq,99".to_string()).unwrap();
        
        let mut grid = [0u8; 600];
        grid[0] = 42;
        
        let matches = matcher.match_all(&grid);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], 1);
    }
}

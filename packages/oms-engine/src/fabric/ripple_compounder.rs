// Ripple Compounder
// Phase A: Base Model Foundation - Task A5: Ripple compounder

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ripple configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RippleConfig {
    pub decay_rate: f64,
    pub propagation_threshold: f64,
    pub max_depth: usize,
}

impl Default for RippleConfig {
    fn default() -> Self {
        Self {
            decay_rate: 0.5,
            propagation_threshold: 0.1,
            max_depth: 3,
        }
    }
}

/// Ripple effect from a pattern match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RippleEffect {
    pub source_pattern_id: u64,
    pub affected_pattern_id: u64,
    pub strength: f64,
    pub depth: usize,
}

/// Ripple Compounder - propagates pattern effects across markets
pub struct RippleCompounder {
    config: RippleConfig,
}

impl RippleCompounder {
    /// Create a new ripple compounder
    pub fn new(config: RippleConfig) -> Self {
        Self { config }
    }

    /// Calculate ripple effects from a pattern match
    pub fn calculate_ripples(
        &self,
        source_pattern_id: u64,
        connected_patterns: &[u64],
        base_strength: f64,
    ) -> Vec<RippleEffect> {
        let mut ripples = Vec::new();

        for (idx, pattern_id) in connected_patterns.iter().enumerate() {
            let depth = (idx / 10) + 1; // Simple depth calculation
            if depth > self.config.max_depth {
                continue;
            }

            let decay = self.config.decay_rate.powi(depth as i32);
            let strength = base_strength * decay;

            if strength < self.config.propagation_threshold {
                continue;
            }

            ripples.push(RippleEffect {
                source_pattern_id,
                affected_pattern_id: *pattern_id,
                strength,
                depth,
            });
        }

        ripples
    }

    /// Compound multiple ripple effects
    pub fn compound_ripples(
        &self,
        ripples: &[RippleEffect],
    ) -> std::collections::HashMap<u64, f64> {
        let mut compounded = std::collections::HashMap::new();

        for ripple in ripples {
            *compounded.entry(ripple.affected_pattern_id).or_insert(0.0) += ripple.strength;
        }

        compounded
    }

    /// Get strongest ripple effects
    pub fn get_strongest_ripples(
        &self,
        compounded: &std::collections::HashMap<u64, f64>,
        top_n: usize,
    ) -> Vec<(u64, f64)> {
        let mut effects: Vec<(u64, f64)> = compounded
            .iter()
            .map(|(id, strength)| (*id, *strength))
            .collect();

        effects.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        effects.into_iter().take(top_n).collect()
    }
}

impl Default for RippleCompounder {
    fn default() -> Self {
        Self::new(RippleConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ripple_compounder() {
        let compounder = RippleCompounder::default();

        let connected = vec![10, 11, 12, 20, 21, 22];
        let ripples = compounder.calculate_ripples(1, &connected, 1.0);

        assert!(!ripples.is_empty());

        // Check decay
        let first_strength = ripples[0].strength;
        let last_strength = ripples.last().unwrap().strength;
        assert!(last_strength <= first_strength);
    }

    #[test]
    fn test_compound_ripples() {
        let compounder = RippleCompounder::default();

        let ripples = vec![
            RippleEffect {
                source_pattern_id: 1,
                affected_pattern_id: 10,
                strength: 0.5,
                depth: 1,
            },
            RippleEffect {
                source_pattern_id: 2,
                affected_pattern_id: 10,
                strength: 0.3,
                depth: 1,
            },
        ];

        let compounded = compounder.compound_ripples(&ripples);
        assert_eq!(compounded.get(&10), Some(&0.8));
    }

    #[test]
    fn test_get_strongest_ripples() {
        let compounder = RippleCompounder::default();

        let mut compounded = HashMap::new();
        compounded.insert(10, 0.5);
        compounded.insert(11, 0.8);
        compounded.insert(12, 0.3);

        let strongest = compounder.get_strongest_ripples(&compounded, 2);
        assert_eq!(strongest.len(), 2);
        assert_eq!(strongest[0].0, 11);
        assert_eq!(strongest[1].0, 10);
    }
}

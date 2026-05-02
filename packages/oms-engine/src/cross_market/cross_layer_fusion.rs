use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::cross_market::pattern_layers::{LayerPatternDetection, LayerPattern, PatternLayerParams, TopLayerPattern, BottomLayerPattern, MiddleLayerPattern, SqueezeLayerPattern, IndicativeLayerPattern, MatchingLayerPattern};
use crate::cross_market::ripple_sync::{RipplePattern, RippleType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionWeights {
    pub top: f64,
    pub bottom: f64,
    pub middle: f64,
    pub squeeze: f64,
    pub indicative: f64,
    pub matching: f64,
    pub cross: f64,
    pub vertical: f64,
    pub horizontal: f64,
}

impl Default for FusionWeights {
    fn default() -> Self {
        Self {
            top: 0.85, bottom: 0.85, middle: 0.65,
            squeeze: 0.75, indicative: 0.9, matching: 0.8,
            cross: 0.7, vertical: 0.6, horizontal: 0.55,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedSignal {
    pub symbol: String,
    pub fused_predictability: f64,
    pub fused_confidence: f64,
    pub direction: f64, // +1 long, -1 short
    pub expected_return: f64,
    pub entry_price: f64,
    pub target_price: f64,
    pub stop_loss: f64,
    pub contributing_layers: Vec<LayerPattern>,
    pub hash: String,
    pub generated_at: DateTime<Utc>,
}

pub struct CrossLayerFusion {
    weights: FusionWeights,
    params: PatternLayerParams,
}

impl CrossLayerFusion {
    pub fn new(weights: FusionWeights, params: PatternLayerParams) -> Self {
        Self { weights, params }
    }

    pub fn fuse(&self, patterns: &[LayerPatternDetection]) -> Vec<FusedSignal> {
        let mut per_symbol: std::collections::HashMap<String, Vec<&LayerPatternDetection>> = std::collections::HashMap::new();
        for p in patterns {
            per_symbol.entry(p.symbol.clone()).or_default().push(p);
        }
        let mut out = Vec::new();
        for (sym, ps) in per_symbol {
            if let Some(f) = self.fuse_symbol(&sym, &ps) {
                out.push(f);
            }
        }
        out.sort_by(|a, b| b.fused_predictability.partial_cmp(&a.fused_predictability).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    fn fuse_symbol(&self, sym: &str, patterns: &[&LayerPatternDetection]) -> Option<FusedSignal> {
        if patterns.is_empty() { return None; }
        let mut weighted_conf = 0.0;
        let mut weighted_pred = 0.0;
        let mut weighted_ret = 0.0;
        let mut total_w = 0.0;
        let mut layers = Vec::new();
        let mut long_score = 0.0;
        let mut short_score = 0.0;
        let mut entry = 0.0;
        let mut target = 0.0;
        let mut stop = 0.0;
        let mut count = 0;

        for p in patterns {
            let w = match p.layer {
                LayerPattern::Top(_) => self.weights.top,
                LayerPattern::Bottom(_) => self.weights.bottom,
                LayerPattern::Middle(_) => self.weights.middle,
                LayerPattern::Squeeze(_) => self.weights.squeeze,
                LayerPattern::Indicative(_) => self.weights.indicative,
                LayerPattern::Matching(_) => self.weights.matching,
                LayerPattern::Cross(_) => self.weights.cross,
                LayerPattern::Vertical(_) => self.weights.vertical,
                LayerPattern::Horizontal(_) => self.weights.horizontal,
            };
            weighted_conf += p.confidence * w;
            weighted_pred += p.predictability * w;
            weighted_ret += p.expected_return * w;
            total_w += w;
            layers.push(p.layer);

            // Direction scoring based on layer type
            match p.layer {
                LayerPattern::Top(_) => short_score += p.confidence * w,
                LayerPattern::Bottom(_) => long_score += p.confidence * w,
                LayerPattern::Indicative(IndicativeLayerPattern::LiquiditySweep) |
                LayerPattern::Indicative(IndicativeLayerPattern::ChangeOfCharacter) |
                LayerPattern::Indicative(IndicativeLayerPattern::MarketStructureBreak) |
                LayerPattern::Indicative(IndicativeLayerPattern::VolumePrecedesPrice) => {
                    if p.target_price > p.entry_price { long_score += p.confidence * w; }
                    else { short_score += p.confidence * w; }
                }
                LayerPattern::Matching(MatchingLayerPattern::FibRetrace618) |
                LayerPattern::Matching(MatchingLayerPattern::FibExtension1618) |
                LayerPattern::Matching(MatchingLayerPattern::ABCDHarmonic) |
                LayerPattern::Matching(MatchingLayerPattern::MeasuredMoveUp) => {
                    if p.target_price > p.entry_price { long_score += p.confidence * w; }
                    else { short_score += p.confidence * w; }
                }
                LayerPattern::Squeeze(SqueezeLayerPattern::BollingerSqueeze) |
                LayerPattern::Squeeze(SqueezeLayerPattern::KeltnerSqueeze) |
                LayerPattern::Squeeze(SqueezeLayerPattern::RangeContraction) |
                LayerPattern::Squeeze(SqueezeLayerPattern::VolumeDryUp) => {
                    // Squeeze is direction-neutral; add to both with half weight
                    long_score += p.confidence * w * 0.5;
                    short_score += p.confidence * w * 0.5;
                }
                _ => {
                    if p.target_price > p.entry_price { long_score += p.confidence * w; }
                    else { short_score += p.confidence * w; }
                }
            }

            entry += p.entry_price;
            target += p.target_price;
            stop += p.stop_loss;
            count += 1;
        }

        if total_w == 0.0 || count == 0 { return None; }
        let fc = weighted_conf / total_w;
        let fp = weighted_pred / total_w;
        let fer = weighted_ret / total_w;

        if fc < self.params.min_confidence || fp < self.params.min_predictability { return None; }

        let direction = if long_score > short_score { 1.0 } else { -1.0 };
        let avg_entry = entry / count as f64;
        let avg_target = target / count as f64;
        let avg_stop = stop / count as f64;

        let hash = hash_fusion(sym, &layers, fc, fp, Utc::now());

        Some(FusedSignal {
            symbol: sym.to_string(),
            fused_predictability: fp,
            fused_confidence: fc,
            direction,
            expected_return: fer,
            entry_price: avg_entry,
            target_price: avg_target,
            stop_loss: avg_stop,
            contributing_layers: layers,
            hash,
            generated_at: Utc::now(),
        })
    }
}

impl Default for CrossLayerFusion {
    fn default() -> Self { Self::new(FusionWeights::default(), PatternLayerParams::default()) }
}

fn hash_fusion(sym: &str, layers: &[LayerPattern], conf: f64, pred: f64, ts: DateTime<Utc>) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    sym.hash(&mut h);
    for l in layers { format!("{:?}", l).hash(&mut h); }
    conf.to_bits().hash(&mut h);
    pred.to_bits().hash(&mut h);
    ts.timestamp().hash(&mut h);
    format!("{:016x}", h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pattern(sym: &str, layer: LayerPattern, conf: f64, pred: f64, entry: f64, target: f64, stop: f64) -> LayerPatternDetection {
        LayerPatternDetection {
            symbol: sym.to_string(), layer, confidence: conf, predictability: pred,
            expected_return: (target - entry).abs() / entry, entry_price: entry,
            target_price: target, stop_loss: stop, detected_at: Utc::now(), hash: "test".to_string(),
        }
    }

    #[test]
    fn test_fusion_single_pattern() {
        let fusion = CrossLayerFusion::default();
        let p = make_pattern("AAPL", LayerPattern::Bottom(BottomLayerPattern::DoubleBottom), 0.8, 0.7, 100.0, 110.0, 95.0);
        let r = fusion.fuse(&[p]);
        assert_eq!(r.len(), 1);
        assert!(r[0].fused_predictability > 0.5);
        assert_eq!(r[0].direction, 1.0); // bottom = long
    }

    #[test]
    fn test_fusion_conflicting_directions() {
        let fusion = CrossLayerFusion::default();
        let p1 = make_pattern("AAPL", LayerPattern::Bottom(BottomLayerPattern::DoubleBottom), 0.9, 0.8, 100.0, 110.0, 95.0);
        let p2 = make_pattern("AAPL", LayerPattern::Top(TopLayerPattern::DoubleTop), 0.5, 0.4, 110.0, 100.0, 115.0);
        let r = fusion.fuse(&[p1, p2]);
        assert_eq!(r.len(), 1);
        // Bottom has higher confidence (0.9 vs 0.5) and higher weight (0.85), so long wins
        assert_eq!(r[0].direction, 1.0);
    }

    #[test]
    fn test_fusion_multiple_symbols() {
        let fusion = CrossLayerFusion::default();
        let p1 = make_pattern("AAPL", LayerPattern::Bottom(BottomLayerPattern::DoubleBottom), 0.8, 0.7, 100.0, 110.0, 95.0);
        let p2 = make_pattern("TSLA", LayerPattern::Top(TopLayerPattern::DoubleTop), 0.8, 0.7, 200.0, 190.0, 210.0);
        let r = fusion.fuse(&[p1, p2]);
        assert_eq!(r.len(), 2);
    }
}

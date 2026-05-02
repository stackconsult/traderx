use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TopLayerPattern { DoubleTop, HeadAndShoulders, RisingWedge, DistributionRange }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BottomLayerPattern { DoubleBottom, InverseHeadAndShoulders, FallingWedge, AccumulationRange }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MiddleLayerPattern { SymmetricalTriangle, Rectangle, Flag, Pennant }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrossLayerPattern { LeadLagRipple, SectorRotation, PairsDivergence, VolatilitySpillover }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerticalLayerPattern { TimeframeAlignment, TimeframeConflict, HigherTFSupport, LowerTFBreakout }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HorizontalLayerPattern { SectorBreadthThrust, MarketBreadthDivergence, PutCallExtreme, VIXTermStructureInvert }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchingLayerPattern { MeasuredMoveUp, FibExtension1618, FibRetrace618, ABCDHarmonic }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SqueezeLayerPattern { BollingerSqueeze, KeltnerSqueeze, RangeContraction, VolumeDryUp }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndicativeLayerPattern { VolumePrecedesPrice, MarketStructureBreak, LiquiditySweep, ChangeOfCharacter }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerPattern {
    Top(TopLayerPattern), Middle(MiddleLayerPattern), Bottom(BottomLayerPattern),
    Cross(CrossLayerPattern), Vertical(VerticalLayerPattern), Horizontal(HorizontalLayerPattern),
    Matching(MatchingLayerPattern), Squeeze(SqueezeLayerPattern), Indicative(IndicativeLayerPattern),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerPatternDetection {
    pub symbol: String, pub layer: LayerPattern, pub confidence: f64,
    pub predictability: f64, pub expected_return: f64, pub entry_price: f64,
    pub target_price: f64, pub stop_loss: f64, pub detected_at: DateTime<Utc>, pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternLayerParams {
    pub min_confidence: f64, pub min_predictability: f64,
    pub max_patterns: usize, pub squeeze_threshold: f64,
}
impl Default for PatternLayerParams {
    fn default() -> Self { Self { min_confidence: 0.55, min_predictability: 0.45, max_patterns: 20, squeeze_threshold: 0.05 } }
}

pub struct PatternLayerEngine {
    params: PatternLayerParams,
    windows: HashMap<String, VecDeque<f64>>,
    highs: HashMap<String, VecDeque<f64>>,
    lows: HashMap<String, VecDeque<f64>>,
    volumes: HashMap<String, VecDeque<f64>>,
    patterns: Vec<LayerPatternDetection>,
}

impl PatternLayerEngine {
    pub fn new(params: PatternLayerParams) -> Self {
        Self { params, windows: HashMap::new(), highs: HashMap::new(), lows: HashMap::new(), volumes: HashMap::new(), patterns: Vec::new() }
    }
    pub fn record(&mut self, symbol: &str, high: f64, low: f64, close: f64, volume: f64) {
        let w = self.windows.entry(symbol.to_string()).or_default();
        w.push_back(close); if w.len() > 200 { w.pop_front(); }
        let h = self.highs.entry(symbol.to_string()).or_default();
        h.push_back(high); if h.len() > 200 { h.pop_front(); }
        let l = self.lows.entry(symbol.to_string()).or_default();
        l.push_back(low); if l.len() > 200 { l.pop_front(); }
        let v = self.volumes.entry(symbol.to_string()).or_default();
        v.push_back(volume); if v.len() > 200 { v.pop_front(); }
    }
    pub fn scan_all(&mut self) -> Vec<LayerPatternDetection> {
        let mut out = Vec::new();
        for sym in self.windows.keys().cloned().collect::<Vec<_>>() {
            let p = self.windows.get(&sym).cloned().unwrap_or_default();
            let h = self.highs.get(&sym).cloned().unwrap_or_default();
            let l = self.lows.get(&sym).cloned().unwrap_or_default();
            let v = self.volumes.get(&sym).cloned().unwrap_or_default();
            out.extend(self.scan_symbol(&sym, &p, &h, &l, &v));
        }
        self.patterns.extend(out.clone()); self.prune(); out
    }
    pub fn top(&self, n: usize) -> Vec<&LayerPatternDetection> {
        let mut v: Vec<_> = self.patterns.iter().collect();
        v.sort_by(|a, b| b.predictability.partial_cmp(&a.predictability).unwrap_or(std::cmp::Ordering::Equal));
        v.into_iter().take(n).collect()
    }
    fn prune(&mut self) {
        let mut count: HashMap<String, usize> = HashMap::new();
        self.patterns.retain(|p| { *count.entry(p.symbol.clone()).or_insert(0) += 1; *count.get(&p.symbol).unwrap() <= self.params.max_patterns });
    }
    fn scan_symbol(&self, sym: &str, prices: &VecDeque<f64>, highs: &VecDeque<f64>, lows: &VecDeque<f64>, vols: &VecDeque<f64>) -> Vec<LayerPatternDetection> {
        let mut out = Vec::new(); let now = Utc::now(); let cp = *prices.back().unwrap_or(&0.0);
        if cp == 0.0 || prices.len() < 20 { return out; }
        let pv: Vec<f64> = prices.iter().copied().collect();
        let hv: Vec<f64> = highs.iter().copied().collect();
        let lv: Vec<f64> = lows.iter().copied().collect();
        let vv: Vec<f64> = vols.iter().copied().collect();
        if let Some(p) = self.dt(sym, &pv, &hv, cp, now) { out.push(p); }
        if let Some(p) = self.db(sym, &pv, &lv, cp, now) { out.push(p); }
        if let Some(p) = self.tri(sym, &pv, &hv, &lv, cp, now) { out.push(p); }
        if let Some(p) = self.sqz(sym, &pv, cp, now) { out.push(p); }
        if let Some(p) = self.vol(sym, &pv, &vv, cp, now) { out.push(p); }
        if let Some(p) = self.liq(sym, &pv, &hv, &lv, cp, now) { out.push(p); }
        if let Some(p) = self.fib(sym, &pv, cp, now) { out.push(p); }
        out.into_iter().filter(|p| p.confidence >= self.params.min_confidence && p.predictability >= self.params.min_predictability).collect()
    }
    fn dt(&self, sym: &str, prices: &[f64], highs: &[f64], cp: f64, now: DateTime<Utc>) -> Option<LayerPatternDetection> {
        if highs.len() < 30 { return None; }
        let peaks = extrema(highs, true); if peaks.len() < 2 { return None; }
        let p1 = peaks[peaks.len()-2]; let p2 = peaks[peaks.len()-1]; let diff = (p2-p1).abs()/p1;
        if diff < 0.025 && cp < p2*0.985 {
            let c = (1.0-diff/0.025).min(1.0); Some(build(sym, LayerPattern::Top(TopLayerPattern::DoubleTop), c, c*0.7, cp, p1*0.96, p2*1.015, now))
        } else { None }
    }
    fn db(&self, sym: &str, prices: &[f64], lows: &[f64], cp: f64, now: DateTime<Utc>) -> Option<LayerPatternDetection> {
        if lows.len() < 30 { return None; }
        let vals = extrema(lows, false); if vals.len() < 2 { return None; }
        let v1 = vals[vals.len()-2]; let v2 = vals[vals.len()-1]; let diff = (v2-v1).abs()/v1;
        if diff < 0.025 && cp > v2*1.015 {
            let c = (1.0-diff/0.025).min(1.0); Some(build(sym, LayerPattern::Bottom(BottomLayerPattern::DoubleBottom), c, c*0.7, cp, v1*1.04, v2*0.985, now))
        } else { None }
    }
    fn tri(&self, sym: &str, prices: &[f64], highs: &[f64], lows: &[f64], cp: f64, now: DateTime<Utc>) -> Option<LayerPatternDetection> {
        if prices.len() < 30 { return None; }
        let sh = slope(highs, prices.len()-20, prices.len()); let sl = slope(lows, prices.len()-20, prices.len());
        if sh < -0.001 && sl > 0.001 {
            Some(build(sym, LayerPattern::Middle(MiddleLayerPattern::SymmetricalTriangle), 0.65, 0.55, cp, cp*1.03, cp*0.97, now))
        } else { None }
    }
    fn sqz(&self, sym: &str, prices: &[f64], cp: f64, now: DateTime<Utc>) -> Option<LayerPatternDetection> {
        if prices.len() < 20 { return None; }
        let (mean, std) = ms(&prices[prices.len()-20..]);
        let bw = if mean > 0.0 { (2.0*std)/mean } else { 1.0 };
        if bw < self.params.squeeze_threshold {
            let c = 1.0-bw/self.params.squeeze_threshold; Some(build(sym, LayerPattern::Squeeze(SqueezeLayerPattern::BollingerSqueeze), c, c*0.65, cp, cp*1.04, cp*0.96, now))
        } else { None }
    }
    fn vol(&self, sym: &str, prices: &[f64], vols: &[f64], cp: f64, now: DateTime<Utc>) -> Option<LayerPatternDetection> {
        if vols.len() < 15 || prices.len() < 15 { return None; }
        let rv = &vols[vols.len()-10..vols.len()-1]; let avg = rv.iter().sum::<f64>()/rv.len() as f64;
        let cur = *vols.last()?; let pp = prices[prices.len()-2];
        if avg > 0.0 && cur/avg > 2.5 && (cp-pp).abs()/pp > 0.005 {
            Some(build(sym, LayerPattern::Indicative(IndicativeLayerPattern::VolumePrecedesPrice), 0.7, 0.55, cp, cp*1.03, cp*0.97, now))
        } else { None }
    }
    fn liq(&self, sym: &str, prices: &[f64], highs: &[f64], lows: &[f64], cp: f64, now: DateTime<Utc>) -> Option<LayerPatternDetection> {
        if lows.len() < 20 || highs.len() < 20 { return None; }
        let pl = lows[lows.len()-10..lows.len()-1].iter().fold(f64::INFINITY, |a,&b| a.min(b));
        let ph = highs[highs.len()-10..highs.len()-1].iter().fold(f64::NEG_INFINITY, |a,&b| a.max(b));
        let rl = lows[lows.len()-5..].iter().fold(f64::INFINITY, |a,&b| a.min(b));
        let rh = highs[highs.len()-5..].iter().fold(f64::NEG_INFINITY, |a,&b| a.max(b));
        if (rl < pl*0.995 && cp > pl) || (rh > ph*1.005 && cp < ph) {
            Some(build(sym, LayerPattern::Indicative(IndicativeLayerPattern::LiquiditySweep), 0.75, 0.65, cp, cp*1.025, cp*0.975, now))
        } else { None }
    }
    fn fib(&self, sym: &str, prices: &[f64], cp: f64, now: DateTime<Utc>) -> Option<LayerPatternDetection> {
        if prices.len() < 30 { return None; }
        let sh = prices[prices.len()-25..prices.len()-5].iter().fold(f64::NEG_INFINITY, |a,&b| a.max(b));
        let sl = prices[prices.len()-25..prices.len()-5].iter().fold(f64::INFINITY, |a,&b| a.min(b));
        let range = sh - sl;
        if range > 0.0 {
            let f618 = sh - range*0.618;
            let diff = (cp - f618).abs() / range;
            if diff < 0.03 {
                let c = 1.0 - diff/0.03;
                Some(build(sym, LayerPattern::Matching(MatchingLayerPattern::FibRetrace618), c, c*0.6, cp, sh, sl, now))
            } else { None }
        } else { None }
    }
}
impl Default for PatternLayerEngine { fn default() -> Self { Self::new(PatternLayerParams::default()) } }

fn extrema(data: &[f64], high: bool) -> Vec<f64> {
    let mut out = Vec::new();
    for i in 1..data.len()-1 {
        if high && data[i] > data[i-1] && data[i] > data[i+1] { out.push(data[i]); }
        if !high && data[i] < data[i-1] && data[i] < data[i+1] { out.push(data[i]); }
    }
    out
}
fn slope(data: &[f64], start: usize, end: usize) -> f64 {
    let s = &data[start.min(data.len()).min(end)..end.min(data.len())];
    if s.len() < 2 { return 0.0; }
    let n = s.len() as f64; let xm = (n-1.0)/2.0; let ym = s.iter().sum::<f64>()/n;
    let mut num = 0.0; let mut den = 0.0;
    for (i,&y) in s.iter().enumerate() { let x = i as f64 - xm; num += x*(y-ym); den += x*x; }
    if den == 0.0 { 0.0 } else { num/den }
}
fn ms(data: &[f64]) -> (f64, f64) {
    if data.is_empty() { return (0.0, 0.0); }
    let m = data.iter().sum::<f64>()/data.len() as f64;
    let v = data.iter().map(|&x| (x-m)*(x-m)).sum::<f64>()/data.len() as f64;
    (m, v.sqrt())
}
fn hash_layer(sym: &str, layer: LayerPattern, price: f64, ts: DateTime<Utc>) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    sym.hash(&mut h);
    format!("{:?}", layer).hash(&mut h);
    price.to_bits().hash(&mut h);
    ts.timestamp().hash(&mut h);
    format!("{:016x}", h.finish())
}
fn build(sym: &str, layer: LayerPattern, conf: f64, pred: f64, entry: f64, target: f64, stop: f64, now: DateTime<Utc>) -> LayerPatternDetection {
    let er = (target-entry).abs()/entry;
    let hash = hash_layer(sym, layer, entry, now);
    LayerPatternDetection { symbol: sym.to_string(), layer, confidence: conf, predictability: pred, expected_return: er, entry_price: entry, target_price: target, stop_loss: stop, detected_at: now, hash }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn prices_trend(base: f64, n: usize, step: f64) -> Vec<f64> { (0..n).map(|i| base + step * i as f64).collect() }
    fn prices_range(base: f64, n: usize, amp: f64) -> Vec<f64> { (0..n).map(|i| base + amp * (i as f64 / n as f64).sin()).collect() }
    fn flat_vol(n: usize, v: f64) -> Vec<f64> { vec![v; n] }
    fn highs_from(p: &[f64], pad: f64) -> Vec<f64> { p.iter().map(|&x| x + pad).collect() }
    fn lows_from(p: &[f64], pad: f64) -> Vec<f64> { p.iter().map(|&x| x - pad).collect() }

    #[test]
    fn test_double_top() {
        let params = PatternLayerParams { min_confidence: 0.3, min_predictability: 0.2, ..Default::default() };
        let mut eng = PatternLayerEngine::new(params);
        let p = prices_range(100.0, 50, 5.0); // sine range
        let h = highs_from(&p, 2.0); let l = lows_from(&p, 2.0); let v = flat_vol(50, 1000.0);
        // create two peaks in highs
        let mut h2 = h.clone(); h2[15] = 115.0; h2[16] = 112.0; h2[35] = 114.0; h2[36] = 113.0;
        for i in 0..50 { eng.record("AAPL", h2[i], l[i], p[i], v[i]); }
        let r = eng.scan_all();
        let dt: Vec<_> = r.iter().filter(|x| matches!(x.layer, LayerPattern::Top(TopLayerPattern::DoubleTop))).collect();
        assert!(!dt.is_empty(), "Should detect double top; patterns: {:?}", r.iter().map(|x| format!("{:?} conf={:.2}", x.layer, x.confidence)).collect::<Vec<_>>());
    }

    #[test]
    fn test_squeeze() {
        let params = PatternLayerParams { min_confidence: 0.3, min_predictability: 0.2, squeeze_threshold: 0.1, ..Default::default() };
        let mut eng = PatternLayerEngine::new(params);
        let p: Vec<f64> = (0..50).map(|_| 100.0).collect(); // flat = squeeze
        let h = highs_from(&p, 0.2); let l = lows_from(&p, 0.2); let v = flat_vol(50, 1000.0);
        for i in 0..50 { eng.record("AAPL", h[i], l[i], p[i], v[i]); }
        let r = eng.scan_all();
        let sq: Vec<_> = r.iter().filter(|x| matches!(x.layer, LayerPattern::Squeeze(SqueezeLayerPattern::BollingerSqueeze))).collect();
        assert!(!sq.is_empty(), "Should detect squeeze on flat range; patterns: {:?}", r.iter().map(|x| format!("{:?} conf={:.2}", x.layer, x.confidence)).collect::<Vec<_>>());
    }

    #[test]
    fn test_double_bottom() {
        let params = PatternLayerParams { min_confidence: 0.3, min_predictability: 0.2, ..Default::default() };
        let mut eng = PatternLayerEngine::new(params);
        let p = prices_range(100.0, 50, 5.0);
        let h = highs_from(&p, 2.0); let mut l = lows_from(&p, 2.0);
        l[15] = 85.0; l[16] = 86.0; l[35] = 84.0; l[36] = 85.0;
        let v = flat_vol(50, 1000.0);
        for i in 0..50 { eng.record("AAPL", h[i], l[i], p[i], v[i]); }
        let r = eng.scan_all();
        let db: Vec<_> = r.iter().filter(|x| matches!(x.layer, LayerPattern::Bottom(BottomLayerPattern::DoubleBottom))).collect();
        assert!(!db.is_empty(), "Should detect double bottom; patterns: {:?}", r.iter().map(|x| format!("{:?} conf={:.2}", x.layer, x.confidence)).collect::<Vec<_>>());
    }

    #[test]
    fn test_triangle() {
        let params = PatternLayerParams { min_confidence: 0.3, min_predictability: 0.2, ..Default::default() };
        let mut eng = PatternLayerEngine::new(params);
        let mut h: Vec<f64> = (0..50).map(|i| 110.0 - i as f64 * 0.1).collect();
        let mut l: Vec<f64> = (0..50).map(|i| 90.0 + i as f64 * 0.1).collect();
        let p: Vec<f64> = (0..50).map(|i| (h[i] + l[i]) / 2.0).collect();
        let v = flat_vol(50, 1000.0);
        for i in 0..50 { eng.record("AAPL", h[i], l[i], p[i], v[i]); }
        let r = eng.scan_all();
        let tri: Vec<_> = r.iter().filter(|x| matches!(x.layer, LayerPattern::Middle(MiddleLayerPattern::SymmetricalTriangle))).collect();
        assert!(!tri.is_empty(), "Should detect triangle; patterns: {:?}", r.iter().map(|x| format!("{:?} conf={:.2}", x.layer, x.confidence)).collect::<Vec<_>>());
    }

    #[test]
    fn test_volume_lead() {
        let params = PatternLayerParams { min_confidence: 0.3, min_predictability: 0.2, ..Default::default() };
        let mut eng = PatternLayerEngine::new(params);
        let p = prices_trend(100.0, 20, 1.0); // 1% step ensures >0.5% move
        let h = highs_from(&p, 0.5); let l = lows_from(&p, 0.5);
        let mut v = flat_vol(19, 1000.0); v.push(5000.0);
        for i in 0..20 { eng.record("AAPL", h[i], l[i], p[i], v[i]); }
        let r = eng.scan_all();
        let vl: Vec<_> = r.iter().filter(|x| matches!(x.layer, LayerPattern::Indicative(IndicativeLayerPattern::VolumePrecedesPrice))).collect();
        assert!(!vl.is_empty(), "Should detect volume precedes price; patterns: {:?}", r.iter().map(|x| format!("{:?} conf={:.2}", x.layer, x.confidence)).collect::<Vec<_>>());
    }

    #[test]
    fn test_liquidity_sweep() {
        let params = PatternLayerParams { min_confidence: 0.3, min_predictability: 0.2, ..Default::default() };
        let mut eng = PatternLayerEngine::new(params);
        let p: Vec<f64> = (0..30).map(|_| 100.0).collect();
        let mut h = highs_from(&p, 1.0); let mut l = lows_from(&p, 1.0);
        l[29] = 95.0; // fake break below at very end, outside prior range [20..28]
        let v = flat_vol(30, 1000.0);
        for i in 0..30 { eng.record("AAPL", h[i], l[i], p[i], v[i]); }
        let r = eng.scan_all();
        let ls: Vec<_> = r.iter().filter(|x| matches!(x.layer, LayerPattern::Indicative(IndicativeLayerPattern::LiquiditySweep))).collect();
        assert!(!ls.is_empty(), "Should detect liquidity sweep; patterns: {:?}", r.iter().map(|x| format!("{:?} conf={:.2}", x.layer, x.confidence)).collect::<Vec<_>>());
    }

    #[test]
    fn test_fib_618() {
        let params = PatternLayerParams { min_confidence: 0.3, min_predictability: 0.2, ..Default::default() };
        let mut eng = PatternLayerEngine::new(params);
        let mut p: Vec<f64> = Vec::new();
        // flat base at 100, then spike to 120, pullback to 0.618 level = 107.64
        for _ in 0..15 { p.push(100.0); }
        for i in 0..10 { p.push(100.0 + i as f64 * 2.0); } // 100→118
        p.push(120.0); p.push(120.0);
        for i in 0..11 { p.push(120.0 - i as f64 * 1.236); } // 120→107.64 at end
        let h = highs_from(&p, 0.5); let l = lows_from(&p, 0.5);
        let v = flat_vol(38, 1000.0);
        for i in 0..38 { eng.record("AAPL", h[i], l[i], p[i], v[i]); }
        let r = eng.scan_all();
        let fib: Vec<_> = r.iter().filter(|x| matches!(x.layer, LayerPattern::Matching(MatchingLayerPattern::FibRetrace618))).collect();
        assert!(!fib.is_empty(), "Should detect fib 0.618; patterns: {:?}", r.iter().map(|x| format!("{:?} conf={:.2}", x.layer, x.confidence)).collect::<Vec<_>>());
    }
}

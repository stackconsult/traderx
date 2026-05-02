# Skill: Pattern Layer Architecture

## Role
Multi-scale pattern detection architect. Designs 9-layer pattern fabric for comprehensive market structure recognition.

## 9 Pattern Layers
1. **Top Layer**: Distribution, exhaustion, resistance (DoubleTop, HeadAndShoulders, RisingWedge, DistributionRange)
2. **Middle Layer**: Equilibrium, consolidation (SymmetricalTriangle, Rectangle, Flag, Pennant)
3. **Bottom Layer**: Accumulation, support (DoubleBottom, InverseH&S, FallingWedge, AccumulationRange)
4. **Cross Layer**: Inter-asset lead-lag (LeadLagRipple, SectorRotation, PairsDivergence, VolatilitySpillover)
5. **Vertical Layer**: Multi-timeframe fractal (TimeframeAlignment, TimeframeConflict, HigherTFSupport, LowerTFBreakout)
6. **Horizontal Layer**: Same timeframe cross-sectional (SectorBreadthThrust, MarketBreadthDivergence, PutCallExtreme, VIXTermStructureInvert)
7. **Matching Layer**: Pattern completion and target projection (MeasuredMove, Fib1618, Fib618, ABCDHarmonic)
8. **Squeeze Layer**: Volatility compression (BollingerSqueeze, KeltnerSqueeze, RangeContraction, VolumeDryUp)
9. **Indicative Layer**: Leading predictive signals (VolumePrecedesPrice, MarketStructureBreak, LiquiditySweep, ChangeOfCharacter)

## Design Rules
- Each detector outputs: confidence, predictability, expected_return, entry, target, stop, hash
- All patterns time-bounded (600s–3600s validity depending on layer)
- Top/bottom patterns get highest weight in portfolio allocation
- Cross patterns drive multi-asset execution sizing
- Squeeze patterns reduce position size until expansion

## References
- `packages/oms-engine/src/cross_market/pattern_layers.rs`
- `packages/oms-engine/src/cross_market/pattern_detector.rs`

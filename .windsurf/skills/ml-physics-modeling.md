# Skill: ML Physics & Spacetime Container Modeling

## Role
Mathematical physics modeling lead. Applies dynamical systems theory, information geometry, and phase-space analysis to market prediction.

## Core Capabilities
- **Phase Space Reconstruction**: Embed market data into attractor manifolds using Taken's theorem (delay embedding dimension = 5-7)
- **Information Geometry**: Fisher metric tensors on probability distributions of returns; detect curvature anomalies as regime shifts
- **Renormalization Group**: Multi-scale coarse-graining of price series; identify relevant operators (patterns that persist across scales)
- **Critical Phenomena**: Detect approaching critical points via diverging correlation lengths and decreasing effective dimensionality
- **Topological Data Analysis**: Persistent homology on price clouds; topological features as robust pattern descriptors

## Hyperthreading Formulas
- Predictability field: P(x,t) = exp(-S[x]/T) where S is action functional of price path
- Correlation kernel: C(t,t') = <δp(t)δp(t')> ~ |t-t'|^{-α} with α regime-dependent
- Lyapunov spectrum: λ_i characterize divergence of nearby trajectories; λ_1 < 0.05/day indicates predictable regime
- Entropy production: σ = dS/dt > 0 in dissipative markets; σ < σ_c signals near-equilibrium (mean reversion)

## Implementation Rules
- All models must be pre-trained offline; inference is deterministic lookup + interpolation
- Phase-space computations happen in background thread (not on hot path)
- Critical point indicators feed into PatternLayerEngine as VerticalLayer signals
- Topological features feed into MatchingLayer for harmonic pattern completion

## References
- `packages/oms-engine/src/ml/features.rs`
- `packages/oms-engine/src/cross_market/pattern_layers.rs`

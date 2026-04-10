# TraderX File Map - Complete System Architecture

## Overview
Production-grade autonomous trading system with AI-native agents, eBPF/XDP kernel bypass, and multi-venue execution.

## Directory Structure

```
traderx/
├── 📁 apps/                          # Applications
│   └── 📁 dashboard/                  # Next.js Trading Dashboard
│       ├── 📄 components/ActionCard.tsx (314 lines)
│       ├── 📄 types/strategy.ts
│       ├── 📄 lib/utils.ts
│       └── 📄 package.json
│
├── 📁 packages/                      # Core Packages
│   │
│   ├── 📁 ai-agents/                 # AI Agent System ✅ COMPLETE
│   │   └── 📁 src/
│   │       ├── 📄 order_management_agent.py (366 lines)
│   │       ├── 📄 position_tracker_agent.py (366 lines)
│   │       ├── 📄 risk_consensus_agent.py (366 lines)
│   │       ├── 📄 kill_switch_agent.py (488 lines)
│   │       ├── 📄 market_data_interpreter.py (366 lines)
│   │       ├── 📄 reconciliation_agent.py (366 lines)
│   │       ├── 📄 advanced_order_agent.py (366 lines)
│   │       └── 📄 smart_order_routing_agent.py (366 lines)
│   │
│   ├── 📁 intelligence-fabric/       # Market Intelligence ✅ COMPLETE
│   │   └── 📁 src/
│   │       ├── 📄 hstr_core.py (180 lines) - O(1) State Tensor
│   │       ├── 📄 deltalag_engine.py (205 lines) - Lead-Lag Detection
│   │       ├── 📄 ic_h_filters.py (143 lines) - IC/Hurst Filters
│   │       ├── 📄 ptp_software_sync.py (112 lines) - PTP Sync
│   │       └── 📄 turboquant_sse.py (19 lines) - SSE4.2 Quantizer
│   │
│   ├── 📁 hft-system/                # High-Frequency Trading Engine ⚠️ WORKING
│   │   ├── 📁 apps/trading_engine/src/
│   │   │   ├── 📄 main.rs (434 lines) - Main Engine
│   │   │   ├── 📄 state.rs (146 lines) - Engine State
│   │   │   ├── 📄 config.rs - Configuration
│   │   │   └── 📄 server.rs - API Server
│   │   └── 📁 crates/
│   │       ├── 📁 risk_engine/src/
│   │       │   └── 📄 lib.rs (92 lines) - Basic Risk Checks
│   │       ├── 📁 execution/src/
│   │       │   ├── 📄 client.rs - Execution Client
│   │       │   └── 📄 signer.rs - Order Signing
│   │       ├── 📁 feed_handler/src/
│   │       │   ├── 📄 lib.rs - Feed Handler
│   │       │   └── 📄 binance.rs - Binance WebSocket
│   │       └── 📁 strategy/src/
│   │           ├── 📄 lib.rs - Strategy Framework
│   │           ├── 📄 ping_pong.rs - Ping Pong Strategy
│   │           └── 📄 momentum.rs - Momentum Strategy
│   │
│   ├── 📁 oms-engine/                # Order Management System ✅ COMPLETE
│   │   └── 📁 src/
│   │       ├── 📄 lib.rs (12 lines) - Module declarations
│   │       ├── 📄 oms.rs (600+ lines) - Core Engine with state machine
│   │       ├── 📄 state_machine.rs (128 lines) - Full state transitions
│   │       ├── 📄 disruptor.rs (300+ lines) - LMAX Disruptor wrapper
│   │       ├── 📄 journal.rs (400+ lines) - Redis event sourcing
│   │       └── 📁 protocol/
│   │           ├── 📄 mod.rs (100+ lines) - Protocol traits
│   │           ├── 📄 sbe.rs (300+ lines) - SBE encoding
│   │           └── 📄 itch.rs (400+ lines) - ITCH parser
│   │
│   ├── 📁 dealing-desk/              # Advanced Trading Infrastructure ✅ COMPLETE
│   │   ├── 📁 ebpf-router/           # eBPF/XDP Kernel Bypass
│   │   │   ├── 📁 ebpf/src/
│   │   │   │   └── 📄 main.rs (5856 lines) - XDP Packet Processing
│   │   │   └── 📁 src/
│   │   │       ├── 📄 main.rs (4823 lines) - Userspace Loader
│   │   │       └── 📄 network.rs (3970 lines) - Network Manager
│   │   └── 📁 src/
│   │       ├── 📄 hybrid_router.py (10109 lines) - B-Book/A-Book Router
│   │       ├── 📄 kya_binding.py (18767 lines) - KYA + Sumsub
│   │       ├── 📄 regime_detector.py (11447 lines) - VPIN/Hurst
│   │       └── 📄 execution_guard.py (5978 lines) - <5μs Guard
│   │
│   ├── 📁 execution-adapters/        # Multi-Venue Execution ✅ COMPLETE
│   │   └── 📁 src/
│   │       ├── 📄 adapter_manager.py (15904 lines) - Smart Order Router
│   │       └── 📁 adapters/
│   │           ├── 📄 rest_adapter.py (16738 lines) - REST API
│   │           ├── 📄 websocket_adapter.py (14748 lines) - WebSocket
│   │           ├── 📄 bybit_websocket_adapter.py (22849 lines) - Bybit
│   │           └── 📄 mock_adapter.py (6412 lines) - Testing
│   │
│   ├── 📁 handoff/                  # Multi-Agent Coordination ✅ COMPLETE
│   │   └── 📁 src/
│   │       ├── 📄 meta_coordinator.py (14020 lines) - Agent Orchestrator
│   │       ├── 📄 turbo_quant.py (13489 lines) - Compression
│   │       ├── 📁 models/
│   │       │   └── 📄 handoff_package.py (267 lines) - State Machine
│   │       └── 📁 llm/ - Claude/Gemma Integration
│   │
│   └── 📁 eceledger/                # Secure Ledger (Optional)
│       └── 📁 crates/host/src/ - 50+ files for secure execution
│
├── 📁 src/                          # Python Trading System
│   ├── 📁 core/                     # Core Engine
│   ├── 📁 data/                     # Data Management
│   ├── 📁 exchanges/                # Exchange Connectors
│   ├── 📁 risk/                     # Risk Management
│   └── 📁 strategies/               # Trading Strategies
│
├── 📁 proofs/                       # Proof Artifacts
│   ├── ✅ M1_INIT_STATUS.json - Hardware Audit
│   ├── ✅ hstr-query-bench.json - HSTR Benchmarks
│   ├── ✅ deltalag.trace - DeltaLag Validation
│   ├── ❌ oms_benchmark.json - INVALID (No OMS)
│   ├── ❌ position_benchmark.json - INVALID (No Engine)
│   ├── ❌ risk_benchmark.json - INVALID (Basic only)
│   └── ❌ kill_switch_benchmark.json - MIXED (Python works, Rust basic)
│
└── 📁 docs/                         # Documentation
    ├── PRODUCTION_AUDIT_CONCISE.md
    └── PRODUCTION_UX_AUDIT_REPORT.md
```

## Component Status Summary

| Component | Status | Lines | Production Ready |
|-----------|--------|-------|------------------|
| AI Agents (8) | ✅ COMPLETE | ~3000 | Yes |
| Intelligence Fabric | ✅ COMPLETE | 659 | Yes |
| eBPF/XDP Router | ✅ COMPLETE | 14,649 | Yes |
| KYA Binding | ✅ COMPLETE | 18,767 | Yes |
| Regime Detector | ✅ COMPLETE | 11,447 | Yes |
| Execution Adapters | ✅ COMPLETE | 76,651 | Yes |
| Handoff System | ✅ COMPLETE | 27,776 | Yes |
| Dashboard UI | ✅ COMPLETE | Next.js | Yes |
| HFT Trading Engine | ⚠️ BASIC | ~1000 | Paper Trading Only |
| **OMS Engine** | ❌ **BROKEN** | **MISSING** | **NO - BLOCKER** |

## Critical Missing Files

```
packages/oms-engine/src/
├── ❌ oms.rs              # Core OMS Engine (600+ lines needed)
├── ❌ disruptor.rs        # LMAX Disruptor Pattern (600+ lines)
├── ❌ journal.rs          # Event Sourcing Journal (300+ lines)
└── ❌ protocol/
    ├── ❌ mod.rs          # Protocol Module (100+ lines)
    ├── ❌ sbe.rs          # SBE Encoding (200+ lines)
    └── ❌ itch.rs         # ITCH Parser (200+ lines)
```

## Integration Points

1. **OMS ↔ HFT System**: Use existing `rtrb::RingBuffer` from `hft-system`
2. **OMS ↔ eBPF Router**: Share order frames via `traderx-ebpf-router-common`
3. **OMS ↔ AI Agents**: Connect through `ai-agents` package
4. **OMS ↔ Execution Adapters**: Use `execution-adapters` manager

## Next Steps

1. Fix OMS compilation by creating missing files
2. Implement LMAX Disruptor pattern
3. Add event sourcing journal
4. Integrate with existing components
5. Run production benchmarks

# Liquidity Execution Skill

## Description
High-frequency liquidity execution across multiple venues using lock-free SPSC ring buffers (LMAX Disruptor pattern). Achieves sub-5μs tick-to-trade latency with kernel bypass techniques.

## Source
- Repository: Minara-AI/openclaw-skill
- File: SKILL.md

## Implementation Pattern

### Core Architecture
```python
class LiquidityExecutor:
    def __init__(self):
        self.ring_buffer = SPSCRingBuffer(size=2**20)  # 1M events
        self.shared_memory = SharedArrayBuffer()
        self.ptp_clock = PTPClock()  # IEEE 1588 synchronized
        
    async def execute_order(self, order: Order):
        """Execute order with sub-5μs latency"""
        # 1. Add to disruptor ring buffer
        # 2. Zero-copy to matching engine
        # 3. Hardware timestamp execution
        # 4. Immediate confirmation
```

### Performance Optimizations
1. **Lock-free queues**: Zero GC pressure vs 56MB/sec allocation
2. **SharedArrayBuffer**: Zero-copy between AI and matching engine
3. **Kernel bypass**: Direct NIC to app (Solarflare/DPDK)
4. **PTP synchronization**: Sub-microsecond clock alignment

### Connectivity Matrix
| Category | Provider | Protocol | Use Case |
|----------|----------|----------|----------|
| Institutional | PrimeXM XCore | FIX API | Tier-1 liquidity |
| Modern Broker | cTrader/DXtrade | Connect API | Retail access |
| Crypto | Bybit/MEXC/dYdX | WebSocket | Perps/RWA |
| Equities | IBKR | TWS API | Global stocks |
| Algorithmic | LMAX Global | FIX | Firm liquidity |

### Execution Flow
1. Signal generation from DeltaLag
2. Intent preview (human oversight)
3. Order routing to optimal venue
4. Real-time execution monitoring
5. Immediate fill confirmation

### Risk Controls
- VPIN toxicity detection
- Dynamic position sizing
- Circuit breaker integration
- Real-time P&L tracking

## Performance Targets
- Tick-to-trade: <5μs
- Throughput: 10,000 events/sec
- Memory allocation: 0 bytes (zero-copy)
- Clock drift: <1μs across venues

## Integration Points
- DeltaLag provides signals
- Risk Manager validates orders
- HSTR provides context
- ZK-Audit logs all decisions

## Notes
- Critical for arbitrage opportunities
- Requires hardware-level optimizations
- Must maintain sub-microsecond precision

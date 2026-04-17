# Sentinel-Nexus Architecture - Complete Trading System

A production-ready, composite trading system integrating multiple best-in-class open-source repositories for live and paper trading.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    SENTINEL-NEXUS ARCHITECTURE                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Trading    │  │   FinRL-     │  │   Finnews-   │         │
│  │   Agents     │  │   Trading    │  │   Hunter     │         │
│  │   (Python)   │  │   (Python)   │  │   (Python)   │         │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
│         │                 │                 │                  │
│         └─────────────────┼─────────────────┘                  │
│                           │                                    │
│                  ┌────────┴────────┐                          │
│                  │  Redis Pub/Sub  │                          │
│                  └────────┬────────┘                          │
│                           │                                    │
│  ┌──────────────┐  ┌──────┴──────┐  ┌──────────────┐          │
│  │   AgentTrade │  │   TraderX   │  │   Nautilus  │          │
│  │   (Node.js)  │  │   Bridge    │  │   Trader    │          │
│  └──────┬───────┘  └──────┬──────┘  └──────┬──────┘          │
│         │                 │                 │                  │
│         └─────────────────┼─────────────────┘                  │
│                           │                                    │
│                  ┌────────┴────────┐                          │
│                  │   QuestDB       │                          │
│                  │   (Tick Data)   │                          │
│                  └─────────────────┘                          │
│                                                                 │
│  ┌────────────────────────────────────────────────────────┐  │
│  │              Alpaca Markets API                        │  │
│  │         (Paper Trading / Live Trading)                 │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Integrated Repositories

| Repository | Purpose | Language | Location |
|------------|---------|----------|----------|
| **TradingAgents** | Multi-agent reasoning, sentiment analysis, risk guardrails | Python | `packages/tradingagents/` |
| **FinRL-Trading (FinRL-X)** | RL-based portfolio weight generation, live trading | Python | `packages/finrl-trading/` |
| **AgentTrade** | Fastify matching engine for internal order routing | Node.js | `packages/agenttrade/` |
| **NautilusTrader** | Deterministic execution core, order book management | Rust/Python | `packages/nautilus_trader/` |
| **QuestDB** | High-frequency tick data storage, time-series analytics | Java | `packages/questdb/` |
| **JaxMARL** | GPU-accelerated multi-agent RL for HFT strategies | Python/JAX | `packages/jaxmarl/` |
| **FinnewsHunter** | Live news sentiment analysis, alternative data | Python | `packages/finnews-hunter/` |
| **UltraLowLatencyFeedHandler** | ITCH/FIX protocol parsing for exchange feeds | C++ | `packages/feed-handler/` |

## Quick Start

### 1. Clone and Setup

```bash
# Clone the repository
git clone https://github.com/your-org/traderx.git
cd traderx

# Switch to sentinel-nexus branch
git checkout sentinel-nexus-integration

# Run setup script
chmod +x setup_sentinel_nexus.sh
./setup_sentinel_nexus.sh
```

### 2. Configure Environment

```bash
# Edit .env file with your API keys
nano .env

# Required variables:
# - ALPACA_API_KEY
# - ALPACA_API_SECRET
# - OPENAI_API_KEY
# - ANTHROPIC_API_KEY
```

### 3. Start Services

```bash
# Start all infrastructure services
docker-compose -f docker-compose.sentinel.yml up -d questdb redis postgres

# Wait for services to be healthy
sleep 30

# Start trading services
docker-compose -f docker-compose.sentinel.yml up -d ai-agents finrl-trading agenttrade oms-engine
```

### 4. Verify Installation

```bash
# Check service status
docker-compose -f docker-compose.sentinel.yml ps

# View logs
docker-compose -f docker-compose.sentinel.yml logs -f ai-agents

# Test QuestDB connection
curl http://localhost:9000

# Access Grafana dashboard
open http://localhost:3000  # admin/admin
```

## Trading Modes

### Paper Trading (Recommended for Testing)

```bash
# Set paper mode
export TRADING_MODE=paper
export ALPACA_PAPER=true

# Start in paper mode
python sentinel_nexus_live.py --mode paper --symbols AAPL MSFT GOOGL
```

### Live Trading (Production)

⚠️ **WARNING**: Only use with real money after thorough testing!

```bash
# Set live credentials
export ALPACA_API_KEY="your_live_key"
export ALPACA_API_SECRET="your_live_secret"
export ALPACA_PAPER=false
export TRADING_MODE=live

# Confirm and start
python sentinel_nexus_live.py --mode live --symbols AAPL MSFT
```

### Backtest Mode

```bash
# Run on historical data
python sentinel_nexus_live.py --mode backtest --symbols AAPL MSFT --start-date 2024-01-01 --end-date 2024-12-31
```

## System Components

### 1. AI Agents Layer (`packages/ai-agents/`)
- **BaseAnalysisAgent**: LangGraph-based analysis with circuit breakers
- **FundamentalsAnalyst**: Financial metrics analysis
- **MarketAnalyst**: Technical indicators and market regime detection
- **NewsAnalyst**: Live news sentiment analysis
- **SocialMediaAnalyst**: Social media sentiment tracking
- **Bull/Bear Researchers**: Debate-based research system

### 2. ML Trading Layer (`packages/finrl-trading/`)
- **FinRLLiveTrader**: Live portfolio weight generation
- **Weight-centric Architecture**: S→A→T→R pipeline (Selection → Allocation → Timing → Risk)
- **Alpaca Integration**: Paper and live trading execution
- **Risk Overlay**: Dynamic position sizing and drawdown protection

### 3. Matching Engine (`packages/agenttrade/`)
- **Fastify-based**: High-throughput order matching
- **Internal Order Book**: L2 limit order book management
- **PostgreSQL Backend**: Persistent order storage
- **WebSocket API**: Real-time order updates

### 4. Execution Core (`packages/nautilus_trader/`)
- **Deterministic Execution**: Nanosecond-precision timestamps
- **Rust-native**: Ultra-low latency order processing
- **Multiple Adapters**: Support for various brokers
- **Backtesting Engine**: Historical replay capabilities

### 5. Data Storage (`packages/questdb/`)
- **Time-Series Database**: Optimized for financial data
- **InfluxDB Protocol**: High-speed ingestion
- **N-dimensional Arrays**: Full order book storage
- **SQL Interface**: Complex analytical queries

## Configuration

### Environment Variables

Create `.env` file in project root:

```bash
# Trading Configuration
TRADING_MODE=paper                    # paper, live, backtest
TRADING_SYMBOLS=AAPL,MSFT,GOOGL,TSLA  # Comma-separated symbols
MAX_POSITION_SIZE=0.10                # 10% max per position
MAX_DRAWDOWN=0.05                     # 5% max drawdown
DAILY_LOSS_LIMIT=-10000               # $10k daily loss limit

# API Keys (Required)
ALPACA_API_KEY=pk_...
ALPACA_API_SECRET=...
ALPACA_PAPER=true

# LLM Keys (Required for AI Agents)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# Optional: News Data
FINNHUB_API_KEY=...
NEWSAPI_KEY=...
TWITTER_BEARER_TOKEN=...

# Database
QUESTDB_HOST=localhost
QUESTDB_PORT=8812
REDIS_URL=redis://localhost:6379
```

### Docker Compose Profiles

Use profiles to start specific components:

```bash
# Core services only
docker-compose -f docker-compose.sentinel.yml up -d

# With GPU-accelerated JAX agents
docker-compose -f docker-compose.sentinel.yml --profile jaxmarl up -d

# With NautilusTrader execution
docker-compose -f docker-compose.sentinel.yml --profile nautilus up -d

# Full stack with monitoring
docker-compose -f docker-compose.sentinel.yml --profile jaxmarl --profile nautilus up -d
```

## Development

### Local Development Setup

```bash
# Python services
cd packages/ai-agents
python -m venv venv
source venv/bin/activate
pip install -e .

# Node.js services
cd packages/agenttrade
pnpm install
pnpm dev

# Rust services
cd packages/oms-engine
cargo build --release
```

### Testing

```bash
# Run all tests
docker-compose -f docker-compose.sentinel.yml exec ai-agents pytest

# Run specific test suite
docker-compose -f docker-compose.sentinel.yml exec ai-agents pytest tests/test_integration.py

# Integration tests with live data (dry-run mode)
python sentinel_nexus_live.py --mode dry_run --test-mode
```

## Monitoring

### QuestDB Console
- URL: http://localhost:9000
- Query real-time tick data
- Run SQL analytics
- Monitor ingestion rates

### Grafana Dashboards
- URL: http://localhost:3000
- Default credentials: admin/admin
- Pre-configured dashboards:
  - System Health
  - Trading Performance
  - P&L Tracking
  - Risk Metrics

### Prometheus Metrics
- URL: http://localhost:9090
- Custom metrics from all services
- Alert rules for risk thresholds

### Redis Monitoring
```bash
# Monitor Redis commands
redis-cli monitor

# Check queue depths
redis-cli LLEN traderx:signals
```

## Troubleshooting

### Common Issues

1. **QuestDB Connection Failed**
   ```bash
   # Check if QuestDB is running
   docker ps | grep questdb
   
   # Check logs
   docker logs sentinel-questdb
   
   # Restart
   docker-compose -f docker-compose.sentinel.yml restart questdb
   ```

2. **Alpaca API Errors**
   ```bash
   # Test API connectivity
   curl -H "APCA-API-KEY-ID: $ALPACA_API_KEY" \
        -H "APCA-API-SECRET-KEY: $ALPACA_API_SECRET" \
        https://paper-api.alpaca.markets/v2/account
   ```

3. **Redis Connection Issues**
   ```bash
   # Test Redis
   redis-cli ping
   # Should return: PONG
   ```

4. **High Memory Usage**
   - Reduce symbols in TRADING_SYMBOLS
   - Decrease QUESTDB retention period
   - Add more RAM or use swap

### Performance Tuning

1. **Network Optimization**
   ```bash
   # Increase network buffers
   sudo sysctl -w net.core.rmem_max=134217728
   sudo sysctl -w net.core.wmem_max=134217728
   ```

2. **QuestDB Optimization**
   ```sql
   -- Partition by hour for high-frequency data
   ALTER TABLE tick_data PARTITION BY HOUR;
   
   -- Drop old data
   ALTER TABLE tick_data DROP PARTITION WHERE timestamp < dateadd('d', -7, now());
   ```

3. **Database Connection Pooling**
   - Configure pool sizes in `.env`
   - Monitor connection counts
   - Use connection multiplexing

## Security

### Best Practices

1. **Never commit API keys**
   - Use `.env` file (already in `.gitignore`)
   - Rotate keys regularly
   - Use dedicated trading accounts

2. **Network Security**
   - Use VPC/Private networks
   - Enable API IP whitelisting
   - Use TLS for all connections

3. **Access Control**
   - Restrict QuestDB access
   - Use strong Redis passwords
   - Enable Docker network isolation

4. **Monitoring**
   - Set up alerts for unusual activity
   - Monitor API rate limits
   - Log all trading actions

## API Documentation

### REST Endpoints

- **AI Agents**: `http://localhost/api/agents`
- **Matching Engine**: `http://localhost/api/matching`
- **OMS Engine**: `http://localhost/api/oms`

### WebSocket Streams

- **Market Data**: `ws://localhost:3002/market-data`
- **Order Updates**: `ws://localhost:3002/orders`
- **Trade Execution**: `ws://localhost:3002/trades`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Submit a pull request

## License

This project integrates multiple open-source components:
- TradingAgents: MIT License
- FinRL-Trading: MIT License
- AgentTrade: MIT License
- NautilusTrader: LGPL-3.0 License
- QuestDB: Apache-2.0 License
- JaxMARL: Apache-2.0 License

See individual repositories for full license details.

## Disclaimer

⚠️ **Trading involves substantial risk of loss. Past performance does not guarantee future results. Use at your own risk.**

This system is for educational and research purposes. Always:
- Test thoroughly in paper mode first
- Start with small position sizes
- Monitor risk limits continuously
- Never trade with money you cannot afford to lose

## Support

- **Documentation**: See `docs/` directory
- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions
- **Discord**: [Join our community](https://discord.gg/traderx)

## Acknowledgments

This project stands on the shoulders of these excellent open-source projects:
- [TradingAgents](https://github.com/TauricResearch/TradingAgents) by Tauric Research
- [FinRL-Trading](https://github.com/AI4Finance-Foundation/FinRL-Trading) by AI4Finance Foundation
- [AgentTrade](https://github.com/satashili/agenttrade) by satashili
- [NautilusTrader](https://github.com/nautechsystems/nautilus_trader) by Nautech Systems
- [QuestDB](https://github.com/questdb/questdb) by QuestDB
- [JaxMARL](https://github.com/FLAIROx/JaxMARL) by FLAIR Oxford
- [FinnewsHunter](https://github.com/DemonDamon/FinnewsHunter) by DemonDamon

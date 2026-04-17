# Sentinel-Nexus Live Trading System Setup

This guide walks you through setting up the production-ready Sentinel-Nexus architecture with real market data feeds and live/paper trading capabilities.

## Architecture Overview

```
Exchange Feeds (Alpaca/Other)
    ↓
UltraLowLatencyFeedHandler (C++) - Optional for HFT
    ↓
QuestDB (High-Frequency Tick Storage)
    ↓
FinRL-Trading (ML Signal Generation)
    ↓
TraderX Execution Layer
    ↓
Alpaca API (Paper/Live Trading)
```

## Prerequisites

### System Requirements
- **OS**: Linux (recommended) or Windows with WSL2
- **RAM**: 16GB minimum, 32GB recommended for high-frequency data
- **Storage**: SSD with 100GB+ for historical tick data
- **Network**: Low-latency internet connection for live trading

### Software Requirements
- Python 3.10+
- Rust/Cargo (for oms-engine)
- CMake 3.28+ (for C++ feed handler)
- Docker & Docker Compose (for QuestDB)
- Redis 6.0+

## Step 1: Install QuestDB

QuestDB is the time-series database for high-frequency tick storage.

### Using Docker (Recommended)

```bash
docker run -p 9000:9000 -p 8812:8812 --name questdb questdb/questdb:latest
```

Or use Docker Compose:

```yaml
version: '3'
services:
  questdb:
    image: questdb/questdb:latest
    ports:
      - "9000:9000"  # HTTP API
      - "8812:8812"  # PostgreSQL wire protocol
      - "9009:9009"  # InfluxDB Line Protocol
    volumes:
      - questdb_data:/var/lib/questdb
    restart: unless-stopped

volumes:
  questdb_data:
```

### Verify Installation

```bash
# Check QuestDB console at http://localhost:9000
# Test PostgreSQL connection
psql -h localhost -p 8812 -U admin -d qdb
```

## Step 2: Install Redis

Redis is used for inter-process communication and caching.

### Using Docker

```bash
docker run -p 6379:6379 --name redis redis:latest
```

### Or Install Native

**Ubuntu/Debian:**
```bash
sudo apt-get install redis-server
sudo systemctl enable redis
sudo systemctl start redis
```

**macOS:**
```bash
brew install redis
brew services start redis
```

## Step 3: Set Up Python Environment

### Create Virtual Environment

```bash
cd /path/to/traderx
cd packages/ai-agents
python -m venv venv
source venv/bin/activate  # Linux/macOS
# or
venv\Scripts\activate  # Windows
```

### Install Dependencies

```bash
# Install ai-agents package
pip install -e .

# Install additional dependencies
pip install asyncpg aiohttp websockets yfinance textblob

# Install TA-Lib (Technical Analysis Library)
# Windows: Download from https://www.lfd.uci.edu/~gohlke/pythonlibs/#ta-lib
# Linux/macOS:
wget http://prdownloads.sourceforge.net/ta-lib/ta-lib-0.4.0-src.tar.gz
tar -xzf ta-lib-0.4.0-src.tar.gz
cd ta-lib
./configure --prefix=/usr
make
sudo make install
pip install TA-Lib
```

## Step 4: Configure API Credentials

### Alpaca Markets (Required for Live/Paper Trading)

1. Create an account at [Alpaca Markets](https://alpaca.markets)
2. Generate API keys from the dashboard
3. Set environment variables:

```bash
export ALPACA_API_KEY="your_api_key_here"
export ALPACA_API_SECRET="your_secret_key_here"

# For paper trading (default)
export ALPACA_PAPER="true"

# For live trading (use with caution!)
export ALPACA_PAPER="false"
```

### Optional: News API (for FinnewsHunter)

```bash
export NEWSAPI_KEY="your_newsapi_key"
export TWITTER_BEARER_TOKEN="your_twitter_token"
export REDDIT_CLIENT_ID="your_reddit_client_id"
export REDDIT_CLIENT_SECRET="your_reddit_secret"
```

## Step 5: Build C++ Feed Handler (Optional for HFT)

For ultra-low latency market data, build the C++ feed handler:

```bash
cd packages/feed-handler
mkdir build && cd build
cmake -DCMAKE_BUILD_TYPE=Release ..
make -j$(nproc)
```

## Step 6: Configure the System

Create configuration file at `config/sentinel_nexus.json`:

```json
{
  "data": {
    "questdb_host": "localhost",
    "questdb_port": 8812,
    "alpaca_paper": true,
    "buffer_size": 100,
    "use_feed_handler": false
  },
  "finrl": {
    "rebalance_interval": 300,
    "lookback_minutes": 60,
    "model_type": "random_forest",
    "risk_overlay": true
  },
  "news": {
    "sources": ["twitter", "reddit"],
    "update_interval": 60,
    "sentiment_threshold": 0.5
  },
  "execution": {
    "max_position_size": 0.1,
    "max_drawdown": 0.05,
    "use_traderx_bridge": true,
    "dry_run": false
  },
  "risk": {
    "var_limit": 0.02,
    "max_leverage": 2.0,
    "circuit_breaker": true,
    "daily_loss_limit": -10000
  }
}
```

## Step 7: Test the System

### 1. Verify QuestDB Connection

```bash
cd packages/data-ingestion/src
python -c "
import asyncio
from live_data_pipeline import QuestDBClient

async def test():
    client = QuestDBClient()
    await client.connect()
    print('QuestDB connection successful!')
    
asyncio.run(test())
"
```

### 2. Test Data Pipeline (Dry Run Mode)

```bash
python sentinel_nexus_live.py --mode dry_run --symbols AAPL MSFT
```

This will:
- Connect to data feeds
- Store data in QuestDB
- Generate ML signals
- NOT execute any trades

### 3. Test Paper Trading

```bash
python sentinel_nexus_live.py --mode paper --symbols AAPL MSFT GOOGL
```

This uses Alpaca's paper trading environment with simulated money.

### 4. Test Backtest Mode

```bash
python sentinel_nexus_live.py --mode backtest --symbols AAPL MSFT
```

This runs on historical data without live market connection.

## Step 8: Go Live (Production)

⚠️ **WARNING**: Only proceed if you understand the risks and have tested thoroughly in paper mode!

```bash
# Set live trading credentials
export ALPACA_API_KEY="your_live_key"
export ALPACA_API_SECRET="your_live_secret"
export ALPACA_PAPER="false"

# Start with small position sizes
python sentinel_nexus_live.py --mode live --symbols AAPL --config config/live_config.json
```

## Monitoring & Operations

### Real-time Monitoring

1. **QuestDB Console**: http://localhost:9000
   - View real-time tick data
   - Run SQL queries for analysis
   - Monitor ingestion rates

2. **System Logs**:
   ```bash
   tail -f sentinel_nexus.log
   ```

3. **Redis Monitoring**:
   ```bash
   redis-cli monitor
   ```

### Key Metrics to Watch

- **Ingestion Rate**: Ticks/second into QuestDB (target: >10k/s)
- **ML Latency**: Time from data to signal generation (target: <100ms)
- **Execution Latency**: Time from signal to order submission (target: <50ms)
- **P&L**: Real-time profit/loss tracking
- **Drawdown**: Maximum portfolio decline from peak

### Risk Circuit Breakers

The system has automatic circuit breakers:
- Daily loss limit exceeded
- Maximum drawdown reached
- API connection failures
- Data feed interruptions

When triggered, trading is automatically halted until manual review.

## Troubleshooting

### QuestDB Connection Issues

```bash
# Check if QuestDB is running
docker ps | grep questdb

# Check logs
docker logs questdb

# Restart
sudo systemctl restart questdb  # native install
# or
docker restart questdb
```

### Alpaca API Issues

```bash
# Test API connectivity
curl -H "APCA-API-KEY-ID: $ALPACA_API_KEY" \
     -H "APCA-API-SECRET-KEY: $ALPACA_API_SECRET" \
     https://paper-api.alpaca.markets/v2/account
```

### Redis Connection Issues

```bash
# Test Redis
redis-cli ping
# Should return: PONG
```

### High Memory Usage

If QuestDB uses too much memory:
1. Reduce retention period in config
2. Increase partition frequency
3. Add more RAM or reduce symbols

## Performance Optimization

### Network Optimization

1. **Use colocation** for ultra-low latency (<1ms)
2. **Dedicated network interface** for market data
3. **Disable WiFi**, use wired connection
4. **Network tuning**:
   ```bash
   sudo sysctl -w net.core.rmem_max=134217728
   sudo sysctl -w net.core.wmem_max=134217728
   ```

### Database Optimization

1. **SSD storage** for QuestDB
2. **Partition by hour** for high-frequency data
3. **Regular maintenance**:
   ```sql
   -- Drop old partitions
   ALTER TABLE tick_data DROP PARTITION WHERE timestamp < dateadd('d', -7, now());
   ```

### Code Optimization

1. **Async/await** for all I/O operations
2. **Connection pooling** for database
3. **Batch operations** where possible
4. **Profile code** to find bottlenecks:
   ```bash
   python -m cProfile -o profile.stats sentinel_nexus_live.py
   ```

## Security Best Practices

1. **Never commit API keys** to git
2. **Use environment variables** or secure vaults
3. **Restrict API permissions** to minimum required
4. **Enable IP whitelisting** on exchange accounts
5. **Use dedicated trading accounts** (not personal)
6. **Monitor for unusual activity**
7. **Regular key rotation**

## Backup & Recovery

### QuestDB Backup

```bash
# Backup to file
docker exec questdb /app/bin/questdb-backup -d /var/lib/questdb -o /backup/questdb_$(date +%Y%m%d).backup

# Restore
docker exec questdb /app/bin/questdb-restore -d /var/lib/questdb -i /backup/questdb_20240101.backup
```

### Configuration Backup

```bash
# Backup configs
tar -czvf backup_$(date +%Y%m%d).tar.gz config/ *.json .env
```

## Support & Resources

- **QuestDB Docs**: https://questdb.io/docs/
- **Alpaca API**: https://alpaca.markets/docs/
- **FinRL-X Paper**: https://arxiv.org/abs/2603.21330
- **System Issues**: Check `sentinel_nexus.log`

## License & Disclaimer

This system is for educational and research purposes. Trading involves substantial risk of loss. Past performance does not guarantee future results. Use at your own risk.

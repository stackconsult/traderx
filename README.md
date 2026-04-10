# TraderX - Professional Trading Automation System

TraderX is a robust, production-ready automated trading system built with Python. It features event-driven architecture, comprehensive risk management, and support for multiple trading strategies.

## Features

- **Event-Driven Architecture**: High-performance async/await implementation
- **Multi-Exchange Support**: Pluggable exchange adapters (Binance implemented)
- **Advanced Risk Management**: Position limits, drawdown controls, circuit breakers
- **Strategy Framework**: Easy-to-extend base classes for custom strategies
- **Real-time Monitoring**: Prometheus metrics and Grafana dashboards
- **Paper Trading Mode**: Safe testing environment
- **Comprehensive Logging**: Structured logging with multiple levels

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Strategies    │───▶│  Trading Engine │───▶│    Exchanges    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │ Risk Manager    │
                       └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │  Data Storage   │
                       └─────────────────┘
```

## Quick Start

### Prerequisites

- Python 3.9+
- PostgreSQL 13+
- Redis 6+
- Docker & Docker Compose (optional)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/stackconsult/traderx.git
cd traderx
```

2. Create virtual environment:
```bash
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
```

3. Install dependencies:
```bash
pip install -r requirements.txt
```

4. Set up environment:
```bash
cp .env.example .env
# Edit .env with your API keys and settings
```

5. Start services with Docker:
```bash
docker-compose up -d
```

6. Run the system:
```bash
python main.py
```

## Configuration

### Environment Variables

Key configuration options in `.env`:

```env
# Trading Mode
ENVIRONMENT=development  # development, paper_trading, live

# Exchange Settings
DEFAULT_EXCHANGE=binance
BINANCE_API_KEY=your_api_key
BINANCE_SECRET_KEY=your_secret_key
BINANCE_SANDBOX=true  # Use sandbox for testing

# Risk Management
MAX_POSITION_SIZE=1000.0
MAX_DAILY_LOSS=100.0
MAX_DRAWDOWN=0.20
```

## Trading Strategies

### Creating a Custom Strategy

1. Inherit from `BaseStrategy`:
```python
from src.strategies.base import BaseStrategy, Signal, SignalType

class MyStrategy(BaseStrategy):
    async def generate_signals(self, symbol: str, data: List[List[float]]) -> List[Signal]:
        # Your strategy logic here
        return signals
```

### Built-in Strategies

- **MovingAverageStrategy**: Classic MA crossover strategy

## Risk Management

The system includes comprehensive risk controls:

- Position Limits: Maximum position size per trade
- Daily Loss Limits: Stop trading if daily loss exceeds threshold
- Drawdown Protection: Circuit breaker on maximum drawdown
- Leverage Controls: Limit total exposure relative to balance

## Security Best Practices

1. **API Keys**: Never commit API keys to version control
2. **Environment Variables**: Use `.env` file for sensitive data
3. **Sandbox Mode**: Always test in sandbox before live trading

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

⚠️ **WARNING**: Trading cryptocurrencies involves substantial risk of loss. Use at your own risk.

#!/usr/bin/env python3
"""
Live Data Pipeline for Sentinel-Nexus Architecture

Integrates real-time market data from multiple sources:
- UltraLowLatencyFeedHandler (C++) for exchange feeds
- QuestDB for high-frequency tick storage
- FinRL-Trading for ML-based signal generation
- FinnewsHunter for news sentiment

This module provides production-ready live and paper trading capabilities.
"""

import asyncio
import json
import logging
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Callable
from enum import Enum
import os

# Add paths for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..', 'finrl-trading', 'src'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..', 'finnews-hunter'))

import aiohttp
import websockets
import asyncpg
import redis.asyncio as redis

logger = logging.getLogger(__name__)


class TradingMode(Enum):
    """Trading mode - affects data sources and execution"""
    LIVE = "live"           # Real money, real exchanges
    PAPER = "paper"         # Simulated execution, real data
    BACKTEST = "backtest"   # Historical data replay
    DRY_RUN = "dry_run"     # Real data, no execution


@dataclass
class LiveMarketData:
    """Standardized live market data structure"""
    symbol: str
    timestamp: datetime
    bid: float
    ask: float
    bid_size: float
    ask_size: float
    last_price: float
    last_size: float
    volume: float
    vwap: float
    source: str  # 'exchange', 'websocket', 'rest'
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'symbol': self.symbol,
            'timestamp': self.timestamp.isoformat(),
            'bid': self.bid,
            'ask': self.ask,
            'bid_size': self.bid_size,
            'ask_size': self.ask_size,
            'last_price': self.last_price,
            'last_size': self.last_size,
            'volume': self.volume,
            'vwap': self.vwap,
            'source': self.source
        }


@dataclass
class TickData:
    """High-frequency tick data for QuestDB storage"""
    symbol: str
    timestamp: datetime
    price: float
    size: float
    side: str  # 'buy' or 'sell'
    exchange: str
    
    def to_questdb_row(self) -> tuple:
        """Convert to QuestDB ingestion format"""
        return (
            self.symbol,
            self.timestamp,
            self.price,
            self.size,
            self.side,
            self.exchange
        )


class QuestDBClient:
    """
    Client for QuestDB high-frequency tick data storage.
    
    Features:
    - Ultra-fast ingestion via InfluxDB Line Protocol
    - SQL queries for historical analysis
    - Time-series aggregation for ML features
    """
    
    def __init__(
        self,
        host: str = "localhost",
        port: int = 8812,
        username: str = "admin",
        password: str = "quest"
    ):
        self.host = host
        self.port = port
        self.http_port = 9000
        self.username = username
        self.password = password
        self.pool: Optional[asyncpg.Pool] = None
        
    async def connect(self):
        """Establish connection to QuestDB"""
        try:
            self.pool = await asyncpg.create_pool(
                host=self.host,
                port=self.port,
                database="qdb",
                user=self.username,
                password=self.password,
                min_size=5,
                max_size=20
            )
            logger.info("Connected to QuestDB")
        except Exception as e:
            logger.error(f"Failed to connect to QuestDB: {e}")
            raise
            
    async def create_tick_table(self):
        """Create optimized tick data table"""
        create_table_sql = """
        CREATE TABLE IF NOT EXISTS tick_data (
            symbol SYMBOL CAPACITY 1000,
            timestamp TIMESTAMP,
            price DOUBLE,
            size DOUBLE,
            side SYMBOL,
            exchange SYMBOL
        ) TIMESTAMP(timestamp) PARTITION BY DAY;
        """
        
        async with self.pool.acquire() as conn:
            await conn.execute(create_table_sql)
            logger.info("Tick data table created/verified")
            
    async def ingest_tick(self, tick: TickData):
        """Ingest single tick via InfluxDB Line Protocol for speed"""
        line = (
            f"tick_data,symbol={tick.symbol},side={tick.side},exchange={tick.exchange} "
            f"price={tick.price},size={tick.size} "
            f"{int(tick.timestamp.timestamp() * 1e9)}"
        )
        
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    f"http://{self.host}:{self.http_port}/write",
                    params={'db': 'qdb'},
                    data=line
                ) as response:
                    if response.status != 204:
                        logger.warning(f"QuestDB ingest failed: {response.status}")
        except Exception as e:
            logger.error(f"Failed to ingest tick: {e}")
            
    async def ingest_batch(self, ticks: List[TickData]):
        """Batch ingest for higher throughput"""
        lines = []
        for tick in ticks:
            line = (
                f"tick_data,symbol={tick.symbol},side={tick.side},exchange={tick.exchange} "
                f"price={tick.price},size={tick.size} "
                f"{int(tick.timestamp.timestamp() * 1e9)}"
            )
            lines.append(line)
            
        data = "\n".join(lines)
        
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    f"http://{self.host}:{self.http_port}/write",
                    params={'db': 'qdb'},
                    data=data
                ) as response:
                    if response.status != 204:
                        logger.warning(f"QuestDB batch ingest failed: {response.status}")
        except Exception as e:
            logger.error(f"Failed to batch ingest: {e}")
            
    async def query_recent_ticks(
        self,
        symbol: str,
        lookback_seconds: int = 60
    ) -> List[Dict]:
        """Query recent ticks for ML feature generation"""
        query = """
        SELECT * FROM tick_data
        WHERE symbol = $1
        AND timestamp > dateadd('s', -$2, now())
        ORDER BY timestamp DESC
        """
        
        async with self.pool.acquire() as conn:
            rows = await conn.fetch(query, symbol, lookback_seconds)
            return [dict(row) for row in rows]
            
    async def get_ohlcv(
        self,
        symbol: str,
        timeframe: str = "1m",
        lookback_bars: int = 100
    ) -> List[Dict]:
        """Get OHLCV bars for technical analysis"""
        query = f"""
        SELECT 
            timestamp,
            first(price) as open,
            max(price) as high,
            min(price) as low,
            last(price) as close,
            sum(size) as volume
        FROM tick_data
        WHERE symbol = $1
        SAMPLE BY {timeframe}
        ORDER BY timestamp DESC
        LIMIT $2
        """
        
        async with self.pool.acquire() as conn:
            rows = await conn.fetch(query, symbol, lookback_bars)
            return [dict(row) for row in rows]


class AlpacaDataFeed:
    """
    Live market data feed from Alpaca Markets.
    
    Provides:
    - Real-time quotes via WebSocket
    - Historical data via REST API
    - Both paper and live trading modes
    """
    
    def __init__(
        self,
        api_key: str,
        api_secret: str,
        paper: bool = True,
        symbols: List[str] = None
    ):
        self.api_key = api_key
        self.api_secret = api_secret
        self.paper = paper
        self.symbols = symbols or []
        
        # Endpoints
        self.base_url = "https://paper-api.alpaca.markets" if paper else "https://api.alpaca.markets"
        self.ws_url = "wss://stream.data.alpaca.markets/v2/iex"  # Free tier
        self.ws_auth_url = "wss://stream.data.alpaca.markets/v2/sip"  # Paid tier
        
        # State
        self.ws: Optional[websockets.WebSocketClientProtocol] = None
        self.running = False
        self.data_handlers: List[Callable] = []
        
    async def connect_websocket(self):
        """Connect to Alpaca WebSocket stream"""
        try:
            self.ws = await websockets.connect(self.ws_url)
            
            # Authenticate
            auth_msg = {
                "action": "auth",
                "key": self.api_key,
                "secret": self.api_secret
            }
            await self.ws.send(json.dumps(auth_msg))
            
            # Wait for auth response
            response = await self.ws.recv()
            logger.info(f"Alpaca WebSocket connected: {response}")
            
            # Subscribe to symbols
            if self.symbols:
                subscribe_msg = {
                    "action": "subscribe",
                    "quotes": self.symbols,
                    "trades": self.symbols
                }
                await self.ws.send(json.dumps(subscribe_msg))
                
            self.running = True
            
            # Start listening
            asyncio.create_task(self._listen())
            
        except Exception as e:
            logger.error(f"Failed to connect Alpaca WebSocket: {e}")
            raise
            
    async def _listen(self):
        """Listen for incoming market data"""
        while self.running and self.ws:
            try:
                message = await self.ws.recv()
                data = json.loads(message)
                
                for item in data:
                    await self._process_message(item)
                    
            except websockets.exceptions.ConnectionClosed:
                logger.warning("Alpaca WebSocket closed")
                break
            except Exception as e:
                logger.error(f"Error processing message: {e}")
                
    async def _process_message(self, item: Dict):
        """Process incoming market data"""
        try:
            msg_type = item.get('T')
            
            if msg_type == 'q':  # Quote
                market_data = LiveMarketData(
                    symbol=item['S'],
                    timestamp=datetime.fromisoformat(item['t'].replace('Z', '+00:00')),
                    bid=item['bp'],
                    ask=item['ap'],
                    bid_size=item['bs'],
                    ask_size=item['as'],
                    last_price=(item['bp'] + item['ap']) / 2,
                    last_size=0,
                    volume=0,
                    vwap=0,
                    source='websocket'
                )
                
                # Notify handlers
                for handler in self.data_handlers:
                    await handler(market_data)
                    
            elif msg_type == 't':  # Trade
                tick = TickData(
                    symbol=item['S'],
                    timestamp=datetime.fromisoformat(item['t'].replace('Z', '+00:00')),
                    price=item['p'],
                    size=item['s'],
                    side='buy' if item['t'] == 'b' else 'sell',
                    exchange=item['x']
                )
                
                # Notify handlers
                for handler in self.data_handlers:
                    await handler(tick)
                    
        except Exception as e:
            logger.error(f"Error processing message item: {e}")
            
    def add_data_handler(self, handler: Callable):
        """Add data handler callback"""
        self.data_handlers.append(handler)
        
    async def get_historical_bars(
        self,
        symbol: str,
        timeframe: str = "1Min",
        limit: int = 100
    ) -> List[Dict]:
        """Get historical price bars via REST API"""
        url = f"{self.base_url}/v2/stocks/{symbol}/bars"
        
        headers = {
            "APCA-API-KEY-ID": self.api_key,
            "APCA-API-SECRET-KEY": self.api_secret
        }
        
        params = {
            "timeframe": timeframe,
            "limit": limit
        }
        
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(url, headers=headers, params=params) as response:
                    if response.status == 200:
                        data = await response.json()
                        return data.get('bars', [])
                    else:
                        logger.error(f"Failed to get historical bars: {response.status}")
                        return []
        except Exception as e:
            logger.error(f"Error fetching historical bars: {e}")
            return []
            
    async def disconnect(self):
        """Disconnect from WebSocket"""
        self.running = False
        if self.ws:
            await self.ws.close()


class FeedHandlerBridge:
    """
    Bridge to UltraLowLatencyFeedHandler C++ engine.
    
    Provides Python interface to C++ feed handler for:
    - ITCH protocol parsing
    - FIX protocol parsing
    - Zero-copy data ingestion
    """
    
    def __init__(
        self,
        executable_path: str = "./packages/feed-handler/build/feed_handler",
        config_path: str = "./config/feed_handler.json"
    ):
        self.executable_path = executable_path
        self.config_path = config_path
        self.process: Optional[subprocess.Popen] = None
        self.redis_client: Optional[redis.Redis] = None
        self.data_handlers: List[Callable] = []
        
    async def start(self):
        """Start the C++ feed handler process"""
        try:
            # Start feed handler subprocess
            self.process = subprocess.Popen(
                [self.executable_path, self.config_path],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE
            )
            
            logger.info(f"Feed handler started with PID: {self.process.pid}")
            
            # Connect to Redis for IPC
            self.redis_client = redis.from_url("redis://localhost:6379/1")
            
            # Start data listener
            asyncio.create_task(self._listen_to_feed())
            
        except Exception as e:
            logger.error(f"Failed to start feed handler: {e}")
            raise
            
    async def _listen_to_feed(self):
        """Listen to feed handler output via Redis"""
        pubsub = self.redis_client.pubsub()
        await pubsub.subscribe("feed_handler:market_data")
        
        async for message in pubsub.listen():
            if message['type'] == 'message':
                try:
                    data = json.loads(message['data'])
                    
                    # Convert to TickData
                    tick = TickData(
                        symbol=data['symbol'],
                        timestamp=datetime.fromisoformat(data['timestamp']),
                        price=data['price'],
                        size=data['size'],
                        side=data['side'],
                        exchange=data['exchange']
                    )
                    
                    # Notify handlers
                    for handler in self.data_handlers:
                        await handler(tick)
                        
                except Exception as e:
                    logger.error(f"Error processing feed data: {e}")
                    
    def add_data_handler(self, handler: Callable):
        """Add data handler callback"""
        self.data_handlers.append(handler)
        
    async def stop(self):
        """Stop the feed handler process"""
        if self.process:
            self.process.terminate()
            self.process.wait()
            logger.info("Feed handler stopped")


class LiveDataPipeline:
    """
    Main pipeline orchestrating all data sources.
    
    Flow:
    Exchange Feeds → FeedHandler/Alpaca → QuestDB → ML Features → Trading Signals
    """
    
    def __init__(
        self,
        mode: TradingMode = TradingMode.PAPER,
        alpaca_key: Optional[str] = None,
        alpaca_secret: Optional[str] = None,
        symbols: List[str] = None
    ):
        self.mode = mode
        self.symbols = symbols or ["AAPL", "MSFT", "GOOGL", "TSLA"]
        
        # Components
        self.questdb = QuestDBClient()
        self.alpaca = None
        self.feed_handler = None
        
        if alpaca_key and alpaca_secret:
            self.alpaca = AlpacaDataFeed(
                api_key=alpaca_key,
                api_secret=alpaca_secret,
                paper=(mode == TradingMode.PAPER),
                symbols=self.symbols
            )
            
        # State
        self.running = False
        self.tick_buffer: List[TickData] = []
        self.buffer_size = 100
        
    async def initialize(self):
        """Initialize all components"""
        logger.info(f"Initializing Live Data Pipeline in {self.mode.value} mode")
        
        # Connect to QuestDB
        await self.questdb.connect()
        await self.questdb.create_tick_table()
        
        # Connect to market data feeds
        if self.alpaca:
            await self.alpaca.connect_websocket()
            self.alpaca.add_data_handler(self._handle_market_data)
            
        # Start feed handler if in live mode
        if self.mode == TradingMode.LIVE:
            self.feed_handler = FeedHandlerBridge()
            await self.feed_handler.start()
            self.feed_handler.add_data_handler(self._handle_tick)
            
        self.running = True
        logger.info("Live Data Pipeline initialized")
        
    async def _handle_market_data(self, data: LiveMarketData):
        """Handle live market data"""
        # Convert to tick and buffer
        tick = TickData(
            symbol=data.symbol,
            timestamp=data.timestamp,
            price=data.last_price,
            size=data.last_size,
            side='unknown',
            exchange='alpaca'
        )
        
        await self._handle_tick(tick)
        
    async def _handle_tick(self, tick: TickData):
        """Handle tick data - buffer and ingest"""
        self.tick_buffer.append(tick)
        
        # Batch ingest when buffer is full
        if len(self.tick_buffer) >= self.buffer_size:
            await self.questdb.ingest_batch(self.tick_buffer)
            self.tick_buffer.clear()
            
    async def get_ml_features(
        self,
        symbol: str,
        lookback_seconds: int = 300
    ) -> Dict[str, Any]:
        """Generate ML features from recent tick data"""
        ticks = await self.questdb.query_recent_ticks(symbol, lookback_seconds)
        
        if not ticks:
            return {}
            
        prices = [t['price'] for t in ticks]
        sizes = [t['size'] for t in ticks]
        
        features = {
            'symbol': symbol,
            'timestamp': datetime.now().isoformat(),
            'price_current': prices[0],
            'price_mean': sum(prices) / len(prices),
            'price_std': self._calc_std(prices),
            'price_range': max(prices) - min(prices),
            'volume_total': sum(sizes),
            'volume_mean': sum(sizes) / len(sizes),
            'tick_count': len(ticks),
            'buy_ratio': len([t for t in ticks if t['side'] == 'buy']) / len(ticks)
        }
        
        return features
        
    def _calc_std(self, values: List[float]) -> float:
        """Calculate standard deviation"""
        if len(values) < 2:
            return 0.0
        mean = sum(values) / len(values)
        variance = sum((x - mean) ** 2 for x in values) / len(values)
        return variance ** 0.5
        
    async def run(self):
        """Run the pipeline"""
        logger.info("Live Data Pipeline running...")
        
        try:
            while self.running:
                # Flush remaining buffer periodically
                if self.tick_buffer:
                    await self.questdb.ingest_batch(self.tick_buffer)
                    self.tick_buffer.clear()
                    
                await asyncio.sleep(1)
                
        except KeyboardInterrupt:
            logger.info("Pipeline stopping...")
        finally:
            await self.cleanup()
            
    async def cleanup(self):
        """Cleanup resources"""
        self.running = False
        
        # Flush remaining data
        if self.tick_buffer:
            await self.questdb.ingest_batch(self.tick_buffer)
            
        # Disconnect
        if self.alpaca:
            await self.alpaca.disconnect()
        if self.feed_handler:
            await self.feed_handler.stop()
            
        logger.info("Live Data Pipeline cleaned up")


# Configuration helper
async def create_live_pipeline_from_env() -> LiveDataPipeline:
    """Create pipeline from environment variables"""
    import os
    
    # Get credentials
    alpaca_key = os.getenv("ALPACA_API_KEY")
    alpaca_secret = os.getenv("ALPACA_API_SECRET")
    trading_mode = os.getenv("TRADING_MODE", "paper")
    symbols = os.getenv("TRADING_SYMBOLS", "AAPL,MSFT,GOOGL").split(",")
    
    mode = TradingMode(trading_mode)
    
    pipeline = LiveDataPipeline(
        mode=mode,
        alpaca_key=alpaca_key,
        alpaca_secret=alpaca_secret,
        symbols=symbols
    )
    
    await pipeline.initialize()
    return pipeline


if __name__ == "__main__":
    # Example usage
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    
    # For testing without real credentials
    pipeline = LiveDataPipeline(
        mode=TradingMode.DRY_RUN,
        symbols=["AAPL", "MSFT"]
    )
    
    asyncio.run(pipeline.initialize())
    asyncio.run(pipeline.run())

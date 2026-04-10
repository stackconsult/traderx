"""
Databento Market Data Adapter
Institutional-grade market data with sub-millisecond latency
"""

import asyncio
import logging
import time
import zlib
from typing import Dict, List, Optional, AsyncGenerator, Callable
from datetime import datetime, timezone
import databento as db
import numpy as np
import pandas as pd
import pyarrow as pa
import structlog

from ..ports.market_data_port import MarketDataPort
from ..domain.models import MarketData, Trade, Quote, OrderBook, OrderBookLevel

logger = structlog.get_logger(__name__)


class DatabentoAdapter(MarketDataPort):
    """High-performance market data adapter using Databento feeds."""
    
    def __init__(self, config: Dict[str, any]):
        super().__init__("databento", config)
        
        # Databento API key
        self.api_key = config.get("api_key")
        if not self.api_key:
            raise ValueError("Databento API key required")
        
        # Dataset and symbols
        self.dataset = config.get("dataset", "XNAS.ITCH")  # NASDAQ ITCH by default
        self.symbols = config.get("symbols", ["AAPL", "MSFT", "SPY"])
        
        # Data types to subscribe
        self.data_types = config.get("data_types", [
            db.Definition.Trades,
            db.Definition.Quotes,
            db.Definition.Mbo,
            db.Definition.Bbo,
        ])
        
        # Databento client
        self._client: Optional[db.Live] = None
        self._historical: Optional[db.Historical] = None
        
        # Order books for each symbol
        self._order_books: Dict[str, OrderBook] = {}
        
        # Performance tracking
        self._stats = {
            "messages_received": 0,
            "bytes_received": 0,
            "last_message_ns": 0,
            "latency_ns": [],
            "gap_events": 0,
        }
        
        # Event handlers
        self._trade_handlers: List[Callable] = []
        self._quote_handlers: List[Callable] = []
        self._orderbook_handlers: List[Callable] = []
        
        # Timestamp synchronization
        self._pts_offset_ns = 0  # Exchange to local time offset
        
    async def connect(self) -> bool:
        """Connect to Databento and start data stream."""
        try:
            # Initialize Databento client
            self._client = db.Live(api_key=self.api_key)
            self._historical = db.Historical(api_key=self.api_key)
            
            # Initialize order books
            for symbol in self.symbols:
                self._order_books[symbol] = OrderBook(
                    symbol=symbol,
                    max_depth=20,
                    price_precision=4,
                )
            
            # Start time synchronization
            await self._sync_time()
            
            # Subscribe to live data
            await self._subscribe_live()
            
            self.is_connected = True
            logger.info("Connected to Databento", dataset=self.dataset)
            return True
            
        except Exception as e:
            logger.error("Failed to connect to Databento", error=str(e))
            self.is_connected = False
            return False
    
    async def disconnect(self) -> None:
        """Disconnect from Databento."""
        if self._client:
            try:
                await self._client.stop()
            except:
                pass
            
        self._client = None
        self._historical = None
        self.is_connected = False
        logger.info("Disconnected from Databento")
    
    async def _sync_time(self) -> None:
        """Synchronize with exchange timestamp."""
        try:
            # Get current time from exchange
            response = await self._historical.metadata.get_range(
                dataset=self.dataset,
                start="2024-01-01T00:00:00.000000000Z",
                end="2024-01-01T00:00:01.000000000Z",
                symbols=["SPY"],
                schema=db.Definition.Mbo,
            )
            
            # Calculate offset (simplified - in production use PTP)
            local_time = time.time_ns()
            exchange_time = response.ts_event[0] if len(response) > 0 else local_time
            self._pts_offset_ns = exchange_time - local_time
            
            logger.info("Time synchronized", offset_ns=self._pts_offset_ns)
            
        except Exception as e:
            logger.warning("Time sync failed, using system time", error=str(e))
            self._pts_offset_ns = 0
    
    async def _subscribe_live(self) -> None:
        """Subscribe to live market data."""
        if not self._client:
            return
        
        # Create task for each data type
        for data_type in self.data_types:
            task = asyncio.create_task(
                self._process_stream(data_type)
            )
            task.add_done_callback(self._handle_task_completion)
    
    async def _process_stream(self, data_type: db.Definition) -> None:
        """Process live data stream."""
        if not self._client:
            return
        
        try:
            # Start live subscription
            job = self._client.subscribe(
                dataset=self.dataset,
                symbols=self.symbols,
                schema=data_type,
                start=db.PublisherTimestamp.Now(),
            )
            
            # Process messages
            async for record in job:
                await self._process_record(record, data_type)
                
        except Exception as e:
            logger.error("Stream processing error", 
                        data_type=str(data_type), 
                        error=str(e))
    
    async def _process_record(self, record, data_type: db.Definition) -> None:
        """Process individual market data record."""
        start_time = time.time_ns()
        
        try:
            # Update statistics
            self._stats["messages_received"] += 1
            self._stats["last_message_ns"] = start_time
            
            # Convert exchange timestamp to local time
            exchange_ts = record.ts_event + self._pts_offset_ns
            
            # Process based on data type
            if data_type == db.Definition.Trades:
                await self._process_trade(record, exchange_ts)
            elif data_type == db.Definition.Quotes:
                await self._process_quote(record, exchange_ts)
            elif data_type == db.Definition.Mbo:
                await self._process_order_book_update(record, exchange_ts)
            elif data_type == db.Definition.Bbo:
                await self._process_bbo(record, exchange_ts)
            
            # Track latency
            latency = time.time_ns() - start_time
            self._stats["latency_ns"].append(latency)
            
            # Keep only last 10000 latency samples
            if len(self._stats["latency_ns"]) > 10000:
                self._stats["latency_ns"] = self._stats["latency_ns"][-10000:]
            
        except Exception as e:
            logger.error("Record processing error", error=str(e))
            self._stats["gap_events"] += 1
    
    async def _process_trade(self, record, ts_event: int) -> None:
        """Process trade record."""
        trade = Trade(
            symbol=record.symbol,
            price=record.price,
            size=record.size,
            side="buy" if record.side == db.Side.Buy else "sell",
            exchange=record.exchange,
            timestamp_ns=ts_event,
            trade_id=str(record.trd_match_id),
        )
        
        # Notify handlers
        for handler in self._trade_handlers:
            try:
                if asyncio.iscoroutinefunction(handler):
                    await handler(trade)
                else:
                    handler(trade)
            except:
                logger.exception("Trade handler error")
    
    async def _process_quote(self, record, ts_event: int) -> None:
        """Process quote record."""
        quote = Quote(
            symbol=record.symbol,
            bid_price=record.bid_px,
            ask_price=record.ask_px,
            bid_size=record.bid_sz,
            ask_size=record.ask_sz,
            exchange=record.exchange,
            timestamp_ns=ts_event,
            quote_id=str(record.quote_seq),
        )
        
        # Notify handlers
        for handler in self._quote_handlers:
            try:
                if asyncio.iscoroutinefunction(handler):
                    await handler(quote)
                else:
                    handler(quote)
            except:
                logger.exception("Quote handler error")
    
    async def _process_order_book_update(self, record, ts_event: int) -> None:
        """Process order book update from MBO data."""
        symbol = record.symbol
        
        if symbol not in self._order_books:
            self._order_books[symbol] = OrderBook(
                symbol=symbol,
                max_depth=20,
                price_precision=4,
            )
        
        order_book = self._order_books[symbol]
        
        # Update order book based on action
        if record.action == db.MboAction.Add:
            order_book.add_level(
                side="bid" if record.side == db.Side.Buy else "ask",
                price=record.price,
                quantity=record.size,
                order_id=str(record.order_id),
            )
        elif record.action == db.MboAction.Cancel:
            order_book.remove_level(
                side="bid" if record.side == db.Side.Buy else "ask",
                order_id=str(record.order_id),
            )
        elif record.action == db.MboAction.Modify:
            order_book.update_level(
                side="bid" if record.side == db.Side.Buy else "ask",
                price=record.price,
                quantity=record.size,
                order_id=str(record.order_id),
            )
        
        order_book.timestamp_ns = ts_event
        
        # Notify handlers
        for handler in self._orderbook_handlers:
            try:
                if asyncio.iscoroutinefunction(handler):
                    await handler(order_book)
                else:
                    handler(order_book)
            except:
                logger.exception("Order book handler error")
    
    async def _process_bbo(self, record, ts_event: int) -> None:
        """Process best bid/offer update."""
        # Update top of book
        symbol = record.symbol
        
        if symbol not in self._order_books:
            self._order_books[symbol] = OrderBook(
                symbol=symbol,
                max_depth=20,
                price_precision=4,
            )
        
        order_book = self._order_books[symbol]
        
        # Update best bid
        if record.bid_px > 0:
            order_book.update_best_bid(
                price=record.bid_px,
                size=record.bid_sz,
            )
        
        # Update best ask
        if record.ask_px > 0:
            order_book.update_best_ask(
                price=record.ask_px,
                size=record.ask_sz,
            )
        
        order_book.timestamp_ns = ts_event
    
    def add_trade_handler(self, handler: Callable) -> None:
        """Add trade event handler."""
        self._trade_handlers.append(handler)
    
    def add_quote_handler(self, handler: Callable) -> None:
        """Add quote event handler."""
        self._quote_handlers.append(handler)
    
    def add_orderbook_handler(self, handler: Callable) -> None:
        """Add order book event handler."""
        self._orderbook_handlers.append(handler)
    
    def get_order_book(self, symbol: str) -> Optional[OrderBook]:
        """Get current order book for symbol."""
        return self._order_books.get(symbol)
    
    async def get_historical_data(
        self,
        symbol: str,
        start: datetime,
        end: datetime,
        data_type: db.Definition = db.Definition.Trades,
    ) -> pd.DataFrame:
        """Retrieve historical market data."""
        if not self._historical:
            raise RuntimeError("Not connected")
        
        response = await self._historical.get_range(
            dataset=self.dataset,
            start=start.isoformat(),
            end=end.isoformat(),
            symbols=[symbol],
            schema=data_type,
        )
        
        # Convert to pandas DataFrame
        if data_type == db.Definition.Trades:
            return pd.DataFrame({
                'timestamp': pd.to_datetime(response.ts_event, unit='ns'),
                'price': response.price,
                'size': response.size,
                'side': response.side,
                'exchange': response.exchange,
            })
        elif data_type == db.Definition.Quotes:
            return pd.DataFrame({
                'timestamp': pd.to_datetime(response.ts_event, unit='ns'),
                'bid_price': response.bid_px,
                'ask_price': response.ask_px,
                'bid_size': response.bid_sz,
                'ask_size': response.ask_sz,
            })
        else:
            # Generic conversion
            return response.to_pandas()
    
    def get_statistics(self) -> Dict[str, any]:
        """Get adapter statistics."""
        latency_stats = self._stats["latency_ns"]
        avg_latency = np.mean(latency_stats) if latency_stats else 0
        p99_latency = np.percentile(latency_stats, 99) if latency_stats else 0
        
        return {
            "messages_received": self._stats["messages_received"],
            "bytes_received": self._stats["bytes_received"],
            "avg_latency_us": avg_latency / 1000,
            "p99_latency_us": p99_latency / 1000,
            "gap_events": self._stats["gap_events"],
            "order_books": len(self._order_books),
        }
    
    def _handle_task_completion(self, task: asyncio.Task) -> None:
        """Handle task completion."""
        if task.exception():
            logger.error("Stream task failed", error=str(task.exception()))


# Market data port interface
class MarketDataPort:
    """Base class for market data adapters."""
    
    def __init__(self, venue_name: str, config: Dict[str, any]):
        self.venue_name = venue_name
        self.config = config
        self.is_connected = False
    
    async def connect(self) -> bool:
        raise NotImplementedError
    
    async def disconnect(self) -> None:
        raise NotImplementedError

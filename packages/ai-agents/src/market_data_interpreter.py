"""
LLM Market Data Interpreter
Replaces traditional FIX/FAST protocols with AI-powered market data understanding.
"""

import asyncio
import json
import re
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass, asdict
from enum import Enum
import anthropic
import numpy as np
import structlog

logger = structlog.get_logger(__name__)


class MarketDataType(Enum):
    """Types of market data the AI can interpret."""
    ORDER_BOOK = "order_book"
    TRADES = "trades"
    QUOTES = "quotes"
    NEWS = "news"
    SENTIMENT = "sentiment"
    ANALYSIS = "analysis"


@dataclass
class MarketInsight:
    """AI-generated market insight."""
    symbol: str
    data_type: MarketDataType
    confidence: float
    insight: str
    key_points: List[str]
    implications: List[str]
    timestamp: datetime
    metadata: Dict[str, Any]


@dataclass
class ProcessedMarketData:
    """Structured market data after AI interpretation."""
    symbol: str
    bid: Optional[float]
    ask: Optional[float]
    bid_size: Optional[float]
    ask_size: Optional[float]
    last_price: Optional[float]
    volume: Optional[float]
    volatility: Optional[float]
    trend: Optional[str]
    sentiment: Optional[str]
    insights: List[MarketInsight]
    timestamp: datetime


class MarketDataInterpreter:
    """
    AI-powered market data interpreter that understands and processes
    various forms of market data without traditional protocols.
    """
    
    def __init__(self, anthropic_api_key: str):
        self.client = anthropic.AsyncAnthropic(api_key=anthropic_api_key)
        self.market_cache: Dict[str, ProcessedMarketData] = {}
        self.insight_history: List[MarketInsight] = []
        
        # Interpretation prompts
        self.data_interpretation_prompt = """
        You are an AI Market Data Interpreter. Analyze this market data:
        
        Raw Data: {raw_data}
        Context: {context}
        
        Extract and provide:
        1. Data type identification (ORDER_BOOK/TRADES/QUOTES/NEWS/SENTIMENT/ANALYSIS)
        2. Key numeric values (bid, ask, price, volume, etc.)
        3. Market sentiment/trend
        4. Trading implications
        5. Confidence in interpretation (0-1)
        6. Key insights and observations
        
        Format as JSON.
        """
        
        self.order_book_prompt = """
        Interpret this order book data:
        
        Data: {data}
        
        Provide structured output:
        1. Best bid and ask prices
        2. Bid and ask sizes
        3. Spread analysis
        4. Liquidity assessment
        5. Imbalance indicators
        6. Trading opportunities
        
        Format as JSON.
        """
        
        self.news_sentiment_prompt = """
        Analyze market sentiment from this news/data:
        
        Content: {content}
        Symbol: {symbol}
        
        Provide:
        1. Sentiment score (-1 to 1)
        2. Key themes/topics
        3. Market impact assessment
        4. Price direction implication
        5. Confidence level
        6. Actionable insights
        
        Format as JSON.
        """
    
    async def interpret_market_data(self,
                                   raw_data: Union[str, Dict, List],
                                   symbol: Optional[str] = None,
                                   context: Optional[Dict] = None) -> ProcessedMarketData:
        """
        Interpret raw market data using AI understanding.
        """
        # Prepare interpretation prompt
        prompt = self.data_interpretation_prompt.format(
            raw_data=raw_data,
            context=context or {}
        )
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=1500,
                messages=[{
                    "role": "user",
                    "content": prompt
                }]
            )
            
            interpretation = json.loads(response.content[0].text)
            
            # Extract structured data
            processed_data = ProcessedMarketData(
                symbol=symbol or self._extract_symbol(raw_data),
                bid=interpretation.get("bid"),
                ask=interpretation.get("ask"),
                bid_size=interpretation.get("bid_size"),
                ask_size=interpretation.get("ask_size"),
                last_price=interpretation.get("last_price"),
                volume=interpretation.get("volume"),
                volatility=interpretation.get("volatility"),
                trend=interpretation.get("trend"),
                sentiment=interpretation.get("sentiment"),
                insights=[],
                timestamp=datetime.utcnow()
            )
            
            # Create insight
            insight = MarketInsight(
                symbol=processed_data.symbol,
                data_type=MarketDataType(interpretation.get("data_type", "QUOTES")),
                confidence=float(interpretation.get("confidence", 0.5)),
                insight=interpretation.get("insights", ""),
                key_points=interpretation.get("key_points", []),
                implications=interpretation.get("implications", []),
                timestamp=datetime.utcnow(),
                metadata={"raw_data": str(raw_data)[:500]}
            )
            
            processed_data.insights.append(insight)
            self.insight_history.append(insight)
            
            # Cache processed data
            self.market_cache[processed_data.symbol] = processed_data
            
            logger.info(
                "Market data interpreted",
                symbol=processed_data.symbol,
                data_type=interpretation.get("data_type"),
                confidence=interpretation.get("confidence")
            )
            
            return processed_data
            
        except Exception as e:
            logger.error(
                "Market data interpretation failed",
                symbol=symbol,
                error=str(e)
            )
            
            # Return basic processed data
            return ProcessedMarketData(
                symbol=symbol or "UNKNOWN",
                bid=None,
                ask=None,
                bid_size=None,
                ask_size=None,
                last_price=None,
                volume=None,
                volatility=None,
                trend=None,
                sentiment=None,
                insights=[],
                timestamp=datetime.utcnow()
            )
    
    async def interpret_order_book(self,
                                  order_book_data: Union[str, Dict],
                                  symbol: str) -> ProcessedMarketData:
        """
        Specialized order book interpretation.
        """
        prompt = self.order_book_prompt.format(data=order_book_data)
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=1000,
                messages=[{
                    "role": "user",
                    "content": prompt
                }]
            )
            
            book_analysis = json.loads(response.content[0].text)
            
            processed_data = ProcessedMarketData(
                symbol=symbol,
                bid=book_analysis.get("best_bid"),
                ask=book_analysis.get("best_ask"),
                bid_size=book_analysis.get("bid_size"),
                ask_size=book_analysis.get("ask_size"),
                last_price=None,
                volume=None,
                volatility=None,
                trend=self._calculate_trend_from_book(book_analysis),
                sentiment=None,
                insights=[],
                timestamp=datetime.utcnow()
            )
            
            # Add order book insight
            insight = MarketInsight(
                symbol=symbol,
                data_type=MarketDataType.ORDER_BOOK,
                confidence=0.9,
                insight=book_analysis.get("liquidity_assessment", ""),
                key_points=[
                    f"Spread: {book_analysis.get('spread_analysis', 'N/A')}",
                    f"Imbalance: {book_analysis.get('imbalance_indicators', 'N/A')}"
                ],
                implications=book_analysis.get("trading_opportunities", []),
                timestamp=datetime.utcnow(),
                metadata={"order_book_depth": len(str(order_book_data))}
            )
            
            processed_data.insights.append(insight)
            
            return processed_data
            
        except Exception as e:
            logger.error(
                "Order book interpretation failed",
                symbol=symbol,
                error=str(e)
            )
            
            # Try to extract basic values with regex
            return self._extract_basic_order_book(order_book_data, symbol)
    
    async def interpret_news_sentiment(self,
                                     content: str,
                                     symbol: str) -> MarketInsight:
        """
        Analyze news sentiment for market impact.
        """
        prompt = self.news_sentiment_prompt.format(
            content=content,
            symbol=symbol
        )
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=1000,
                messages=[{
                    "role": "user",
                    "content": prompt
                }]
            )
            
            sentiment_analysis = json.loads(response.content[0].text)
            
            insight = MarketInsight(
                symbol=symbol,
                data_type=MarketDataType.SENTIMENT,
                confidence=float(sentiment_analysis.get("confidence", 0.5)),
                insight=sentiment_analysis.get("market_impact_assessment", ""),
                key_points=sentiment_analysis.get("key_themes", []),
                implications=[
                    f"Price direction: {sentiment_analysis.get('price_direction_implication', 'NEUTRAL')}",
                    f"Sentiment score: {sentiment_analysis.get('sentiment_score', 0)}"
                ],
                timestamp=datetime.utcnow(),
                metadata={
                    "sentiment_score": sentiment_analysis.get("sentiment_score", 0),
                    "content_preview": content[:200]
                }
            )
            
            self.insight_history.append(insight)
            
            return insight
            
        except Exception as e:
            logger.error(
                "News sentiment analysis failed",
                symbol=symbol,
                error=str(e)
            )
            
            # Fallback sentiment
            return MarketInsight(
                symbol=symbol,
                data_type=MarketDataType.SENTIMENT,
                confidence=0.3,
                insight="Sentiment analysis unavailable",
                key_points=["Manual review required"],
                implications=["Neutral sentiment assumed"],
                timestamp=datetime.utcnow(),
                metadata={"error": str(e)}
            )
    
    async def interpret_trade_flow(self,
                                  trades: List[Dict],
                                  symbol: str) -> ProcessedMarketData:
        """
        Interpret trade flow for market insights.
        """
        if not trades:
            return ProcessedMarketData(
                symbol=symbol,
                bid=None,
                ask=None,
                bid_size=None,
                ask_size=None,
                last_price=None,
                volume=None,
                volatility=None,
                trend=None,
                sentiment=None,
                insights=[],
                timestamp=datetime.utcnow()
            )
        
        # Analyze trade patterns
        prices = [t.get("price", 0) for t in trades if t.get("price")]
        volumes = [t.get("volume", 0) for t in trades if t.get("volume")]
        
        if prices:
            last_price = prices[-1]
            price_change = (prices[-1] - prices[0]) / prices[0] if len(prices) > 1 else 0
            volume_total = sum(volumes)
            
            # Determine trend from trades
            if price_change > 0.001:
                trend = "BULLISH"
            elif price_change < -0.001:
                trend = "BEARISH"
            else:
                trend = "SIDEWAYS"
            
            # Calculate volatility
            if len(prices) > 1:
                returns = np.diff(prices) / prices[:-1]
                volatility = np.std(returns) * np.sqrt(252)  # Annualized
            else:
                volatility = None
            
            processed_data = ProcessedMarketData(
                symbol=symbol,
                bid=None,
                ask=None,
                bid_size=None,
                ask_size=None,
                last_price=last_price,
                volume=volume_total,
                volatility=volatility,
                trend=trend,
                sentiment=None,
                insights=[],
                timestamp=datetime.utcnow()
            )
            
            # Add trade flow insight
            insight = MarketInsight(
                symbol=symbol,
                data_type=MarketDataType.TRADES,
                confidence=0.8,
                insight=f"Trade flow shows {trend.lower()} momentum with {len(trades)} trades",
                key_points=[
                    f"Price change: {price_change:.2%}",
                    f"Total volume: {volume_total}",
                    f"Trade count: {len(trades)}"
                ],
                implications=[f"Consider {trend.lower()} strategies"] if trend != "SIDEWAYS" else [],
                timestamp=datetime.utcnow(),
                metadata={"trade_count": len(trades)}
            )
            
            processed_data.insights.append(insight)
            
            return processed_data
        
        return ProcessedMarketData(
            symbol=symbol,
            bid=None,
            ask=None,
            bid_size=None,
            ask_size=None,
            last_price=None,
            volume=None,
            volatility=None,
            trend=None,
            sentiment=None,
            insights=[],
            timestamp=datetime.utcnow()
        )
    
    async def get_market_summary(self,
                                symbols: List[str]) -> Dict[str, Any]:
        """
        Generate AI-powered market summary for multiple symbols.
        """
        summary_prompt = f"""
        Create a market summary for these symbols:
        
        Market Data: {[asdict(self.market_cache.get(s, {})) for s in symbols]}
        Recent Insights: {[asdict(i) for i in self.insight_history[-10:]]}
        
        Provide:
        1. Overall market sentiment
        2. Key market themes
        3. Notable movements
        4. Trading opportunities
        5. Risk factors
        
        Format as JSON.
        """
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=2000,
                messages=[{
                    "role": "user",
                    "content": summary_prompt
                }]
            )
            
            return json.loads(response.content[0].text)
            
        except Exception as e:
            logger.error("Market summary generation failed", error=str(e))
            return {"error": "Summary unavailable"}
    
    def _extract_symbol(self, raw_data: Any) -> str:
        """Extract symbol from raw data."""
        if isinstance(raw_data, dict):
            return raw_data.get("symbol", raw_data.get("ticker", "UNKNOWN"))
        elif isinstance(raw_data, str):
            # Try to extract symbol with regex
            symbol_match = re.search(r'\$([A-Z]+)', raw_data)
            if symbol_match:
                return symbol_match.group(1)
        return "UNKNOWN"
    
    def _calculate_trend_from_book(self, book_analysis: Dict) -> str:
        """Calculate trend from order book analysis."""
        imbalance = book_analysis.get("imbalance_indicators", "").lower()
        if "buy" in imbalance:
            return "BULLISH"
        elif "sell" in imbalance:
            return "BEARISH"
        return "NEUTRAL"
    
    def _extract_basic_order_book(self, data: Any, symbol: str) -> ProcessedMarketData:
        """Extract basic order book data with regex as fallback."""
        data_str = str(data)
        
        # Try to extract prices
        bid_match = re.search(r'bid[:\s]+(\d+\.?\d*)', data_str, re.IGNORECASE)
        ask_match = re.search(r'ask[:\s]+(\d+\.?\d*)', data_str, re.IGNORECASE)
        
        bid = float(bid_match.group(1)) if bid_match else None
        ask = float(ask_match.group(1)) if ask_match else None
        
        return ProcessedMarketData(
            symbol=symbol,
            bid=bid,
            ask=ask,
            bid_size=None,
            ask_size=None,
            last_price=None,
            volume=None,
            volatility=None,
            trend=None,
            sentiment=None,
            insights=[],
            timestamp=datetime.utcnow()
        )
    
    def get_cached_data(self, symbol: str) -> Optional[ProcessedMarketData]:
        """Get cached market data for symbol."""
        return self.market_cache.get(symbol)
    
    def get_recent_insights(self,
                           symbol: Optional[str] = None,
                           hours: int = 1) -> List[MarketInsight]:
        """Get recent market insights."""
        cutoff_time = datetime.utcnow() - timedelta(hours=hours)
        
        insights = [
            i for i in self.insight_history
            if i.timestamp > cutoff_time
        ]
        
        if symbol:
            insights = [i for i in insights if i.symbol == symbol]
        
        return insights

"""
Market analysis agent for technical analysis and market sentiment.
"""

from typing import Dict, Any, List, Optional
import pandas as pd
import numpy as np
from datetime import datetime, timedelta
import talib

from .base import BaseAnalysisAgent, ExecutionMode


class MarketAnalyst(BaseAnalysisAgent):
    """
    Analyzes market conditions including:
    - Technical indicators (RSI, MACD, Bollinger Bands)
    - Price action patterns
    - Volume analysis
    - Market sentiment indicators
    - Support/resistance levels
    - Trend analysis
    """
    
    def __init__(
        self,
        llm_provider: str = "openai",
        model_name: str = "gpt-4-turbo-preview",
        execution_mode: ExecutionMode = ExecutionMode.HYBRID
    ):
        super().__init__(
            name="market_analyst",
            llm_provider=llm_provider,
            model_name=model_name,
            execution_mode=execution_mode
        )
        
    def _build_prompt(self, input_data: Dict[str, Any]) -> str:
        """Build market analysis prompt"""
        symbol = input_data.get("symbol", "")
        timeframe = input_data.get("timeframe", "1d")
        period = input_data.get("period", "1y")
        
        if not symbol:
            return "Error: No symbol provided for market analysis"
            
        # Get market data
        market_data = self._get_market_data(symbol, timeframe, period)
        technical_analysis = self._get_technical_analysis(symbol, timeframe, period)
        
        prompt = f"""As a Market Analyst, analyze {symbol}'s technical and market conditions.

MARKET DATA:
{market_data}

TECHNICAL INDICATORS:
{technical_analysis}

ANALYSIS FRAMEWORK:
1. Trend Analysis
   - Primary trend direction (bullish/bearish/sideways)
   - Trend strength and duration
   - Higher highs/higher lows pattern
   - Moving average relationships

2. Momentum Indicators
   - RSI levels and divergence
   - MACD crossover signals
   - Stochastic oscillator
   - Rate of change (ROC)

3. Volatility Analysis
   - Bollinger Band width and position
   - Average True Range (ATR)
   - Volatility contraction/expansion
   - Implied volatility vs historical

4. Volume Analysis
   - Volume trend and spikes
   - On-balance volume (OBV)
   - Volume price trend
   - Accumulation/distribution

5. Support/Resistance
   - Key price levels
   - Fibonacci retracements
   - Pivot points
   - Previous highs/lows

6. Pattern Recognition
   - Chart patterns (head & shoulders, triangles, flags)
   - Candlestick patterns
   - Breakout patterns
   - Reversal signals

Provide a comprehensive technical analysis with:
- Overall market sentiment (Bullish/Bearish/Neutral)
- Key technical signals
- Entry/exit levels
- Risk management points
- Confidence level (0-100%)
- Time horizon for analysis

Format your response with clear technical analysis and specific price levels."""
        
        return prompt
        
    def _get_market_data(self, symbol: str, timeframe: str, period: str) -> str:
        """Retrieve and format market data"""
        try:
            import yfinance as yf
            
            # Map period to days
            period_map = {
                "1mo": 30,
                "3mo": 90,
                "6mo": 180,
                "1y": 365,
                "2y": 730
            }
            days = period_map.get(period, 365)
            
            # Get historical data
            ticker = yf.Ticker(symbol.upper())
            hist = ticker.history(period=period, interval=timeframe)
            
            if hist.empty:
                return "No historical data available"
                
            # Calculate basic metrics
            current_price = hist['Close'].iloc[-1]
            price_change = hist['Close'].iloc[-1] - hist['Close'].iloc[0]
            price_change_pct = (price_change / hist['Close'].iloc[0]) * 100
            
            # Recent volatility
            returns = hist['Close'].pct_change().dropna()
            volatility = returns.std() * np.sqrt(252)  # Annualized
            
            # Volume analysis
            avg_volume = hist['Volume'].mean()
            recent_volume = hist['Volume'].tail(10).mean()
            volume_ratio = recent_volume / avg_volume if avg_volume > 0 else 1
            
            # Price ranges
            high_52w = hist['High'].max()
            low_52w = hist['Low'].min()
            current_percentile = ((current_price - low_52w) / (high_52w - low_52w)) * 100
            
            data = []
            data.append(f"Current Price: ${current_price:.2f}")
            data.append(f"Period Change: {price_change:+.2f} ({price_change_pct:+.2f}%)")
            data.append(f"Annualized Volatility: {volatility:.2%}")
            data.append(f"Volume Ratio (recent/avg): {volume_ratio:.2f}")
            data.append(f"52W Range: ${low_52w:.2f} - ${high_52w:.2f}")
            data.append(f"Current Percentile: {current_percentile:.1f}%")
            
            # Recent price action
            data.append(f"\nRecent Price Action (last 10 periods):")
            recent_data = hist.tail(10)[['Close', 'Volume']]
            for date, row in recent_data.iterrows():
                data.append(f"  {date.strftime('%Y-%m-%d')}: ${row['Close']:.2f} (Vol: {row['Volume']:,.0f})")
                
            return "\n".join(data)
            
        except Exception as e:
            return f"Error retrieving market data: {str(e)}"
            
    def _get_technical_analysis(self, symbol: str, timeframe: str, period: str) -> str:
        """Calculate technical indicators"""
        try:
            import yfinance as yf
            
            # Get data
            ticker = yf.Ticker(symbol.upper())
            hist = ticker.history(period=period, interval=timeframe)
            
            if hist.empty or len(hist) < 50:
                return "Insufficient data for technical analysis"
                
            closes = hist['Close'].values
            highs = hist['High'].values
            lows = hist['Low'].values
            volumes = hist['Volume'].values
            
            indicators = []
            
            # Moving averages
            sma_20 = talib.SMA(closes, timeperiod=20)
            sma_50 = talib.SMA(closes, timeperiod=50)
            sma_200 = talib.SMA(closes, timeperiod=200)
            
            if len(sma_20) > 0 and not np.isnan(sma_20[-1]):
                indicators.append(f"SMA(20): ${sma_20[-1]:.2f}")
                indicators.append(f"Price vs SMA(20): {((closes[-1] - sma_20[-1]) / sma_20[-1] * 100):+.2f}%")
                
            if len(sma_50) > 0 and not np.isnan(sma_50[-1]):
                indicators.append(f"SMA(50): ${sma_50[-1]:.2f}")
                indicators.append(f"Price vs SMA(50): {((closes[-1] - sma_50[-1]) / sma_50[-1] * 100):+.2f}%")
                
            if len(sma_200) > 0 and not np.isnan(sma_200[-1]):
                indicators.append(f"SMA(200): ${sma_200[-1]:.2f}")
                indicators.append(f"Price vs SMA(200): {((closes[-1] - sma_200[-1]) / sma_200[-1] * 100):+.2f}%")
                
            # RSI
            rsi = talib.RSI(closes, timeperiod=14)
            if len(rsi) > 0 and not np.isnan(rsi[-1]):
                indicators.append(f"RSI(14): {rsi[-1]:.2f}")
                if rsi[-1] > 70:
                    indicators.append("RSI Signal: Overbought")
                elif rsi[-1] < 30:
                    indicators.append("RSI Signal: Oversold")
                else:
                    indicators.append("RSI Signal: Neutral")
                    
            # MACD
            macd, macd_signal, macd_hist = talib.MACD(closes)
            if len(macd) > 0 and not np.isnan(macd[-1]):
                indicators.append(f"MACD: {macd[-1]:.4f}")
                indicators.append(f"MACD Signal: {macd_signal[-1]:.4f}")
                indicators.append(f"MACD Histogram: {macd_hist[-1]:.4f}")
                if macd[-1] > macd_signal[-1]:
                    indicators.append("MACD Trend: Bullish")
                else:
                    indicators.append("MACD Trend: Bearish")
                    
            # Bollinger Bands
            bb_upper, bb_middle, bb_lower = talib.BBANDS(closes, timeperiod=20, nbdevup=2, nbdevdn=2)
            if len(bb_upper) > 0 and not np.isnan(bb_upper[-1]):
                bb_position = (closes[-1] - bb_lower[-1]) / (bb_upper[-1] - bb_lower[-1])
                indicators.append(f"BB Position: {bb_position:.2%}")
                indicators.append(f"BB Upper: ${bb_upper[-1]:.2f}")
                indicators.append(f"BB Lower: ${bb_lower[-1]:.2f}")
                
            # Stochastic
            slowk, slowd = talib.STOCH(highs, lows, closes, fastk_period=14, slowk_period=3, slowd_period=3)
            if len(slowk) > 0 and not np.isnan(slowk[-1]):
                indicators.append(f"Stochastic K: {slowk[-1]:.2f}")
                indicators.append(f"Stochastic D: {slowd[-1]:.2f}")
                
            # ATR (Average True Range)
            atr = talib.ATR(highs, lows, closes, timeperiod=14)
            if len(atr) > 0 and not np.isnan(atr[-1]):
                indicators.append(f"ATR(14): ${atr[-1]:.2f}")
                atr_pct = (atr[-1] / closes[-1]) * 100
                indicators.append(f"ATR %: {atr_pct:.2f}%")
                
            # ADX (Average Directional Index)
            adx = talib.ADX(highs, lows, closes, timeperiod=14)
            if len(adx) > 0 and not np.isnan(adx[-1]):
                indicators.append(f"ADX(14): {adx[-1]:.2f}")
                if adx[-1] > 25:
                    indicators.append("ADX Signal: Strong Trend")
                else:
                    indicators.append("ADX Signal: Weak Trend")
                    
            # On-Balance Volume
            obv = talib.OBV(closes, volumes)
            if len(obv) > 1:
                obv_trend = obv[-1] - obv[-10] if len(obv) >= 10 else obv[-1] - obv[0]
                indicators.append(f"OBV Trend: {'Positive' if obv_trend > 0 else 'Negative'}")
                
            return "\n".join(indicators)
            
        except Exception as e:
            return f"Error calculating technical indicators: {str(e)}"
            
    async def analyze_market_regime(self, symbol: str, lookback_days: int = 252) -> Dict[str, Any]:
        """
        Analyze market regime (trending, ranging, volatile)
        
        Returns:
            Dictionary with regime classification and metrics
        """
        try:
            import yfinance as yf
            
            # Get data
            ticker = yf.Ticker(symbol.upper())
            hist = ticker.history(period=f"{lookback_days}d")
            
            if hist.empty:
                return {'error': 'No data available'}
                
            closes = hist['Close']
            
            # Calculate returns
            returns = closes.pct_change().dropna()
            
            # Trend strength (using linear regression)
            x = np.arange(len(closes))
            slope, intercept = np.polyfit(x, closes, 1)
            r_squared = 1 - (np.sum((closes - (slope * x + intercept)) ** 2) / 
                            np.sum((closes - closes.mean()) ** 2))
            
            # Volatility regime
            volatility = returns.std() * np.sqrt(252)
            vol_percentile = np.percentile(returns.rolling(20).std().dropna() * np.sqrt(252), 70)
            
            # Range detection
            price_range = closes.max() - closes.min()
            avg_range = price_range / closes.mean()
            
            # Classify regime
            if r_squared > 0.7 and abs(slope) > closes.mean() * 0.001:
                regime = "trending"
                direction = "bullish" if slope > 0 else "bearish"
            elif volatility > vol_percentile:
                regime = "volatile"
                direction = "neutral"
            else:
                regime = "ranging"
                direction = "neutral"
                
            return {
                'symbol': symbol,
                'regime': regime,
                'direction': direction,
                'trend_strength': r_squared,
                'volatility': volatility,
                'volatility_percentile': volatility / vol_percentile if vol_percentile > 0 else 1,
                'price_range_pct': avg_range * 100,
                'slope': slope,
                'confidence': min(r_squared * 100, 100),
                'timestamp': datetime.now()
            }
            
        except Exception as e:
            return {'error': str(e), 'timestamp': datetime.now()}

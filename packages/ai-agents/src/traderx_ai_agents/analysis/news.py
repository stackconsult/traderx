"""
News analysis agent for sentiment analysis and news impact assessment.
"""

from typing import Dict, Any, List, Optional
import aiohttp
import asyncio
from datetime import datetime, timedelta
import re
from textblob import TextBlob

from .base import BaseAnalysisAgent, ExecutionMode


class NewsAnalyst(BaseAnalysisAgent):
    """
    Analyzes news and information flow including:
    - News sentiment analysis
    - Breaking news detection
    - Earnings call analysis
    - Regulatory news impact
    - Analyst recommendations
    - Social media sentiment
    """
    
    def __init__(
        self,
        llm_provider: str = "openai",
        model_name: str = "gpt-4-turbo-preview",
        execution_mode: ExecutionMode = ExecutionMode.HYBRID,
        news_api_key: Optional[str] = None
    ):
        super().__init__(
            name="news_analyst",
            llm_provider=llm_provider,
            model_name=model_name,
            execution_mode=execution_mode
        )
        self.news_api_key = news_api_key
        
    def _build_prompt(self, input_data: Dict[str, Any]) -> str:
        """Build news analysis prompt"""
        symbol = input_data.get("symbol", "")
        company_name = input_data.get("company_name", "")
        lookback_days = input_data.get("lookback_days", 7)
        
        if not symbol and not company_name:
            return "Error: No symbol or company name provided for news analysis"
            
        # Get news data
        news_data = self._get_news_data(symbol, company_name, lookback_days)
        sentiment_analysis = self._analyze_sentiment(news_data)
        
        prompt = f"""As a News Analyst, analyze recent news and information flow for {symbol or company_name}.

NEWS DATA:
{news_data}

SENTIMENT ANALYSIS:
{sentiment_analysis}

ANALYSIS FRAMEWORK:
1. News Volume & Velocity
   - Recent news frequency
   - Breaking news detection
   - Unusual activity spikes
   - Information flow patterns

2. Sentiment Assessment
   - Overall news sentiment (Bullish/Bearish/Neutral)
   - Sentiment trend over time
   - Key sentiment drivers
   - Contrarian signals

3. News Categories Impact
   - Earnings-related news
   - Product/Service announcements
   - M&A and strategic news
   - Regulatory/Legal developments
   - Management changes
   - Competitive landscape

4. Source Quality Analysis
   - Credible sources vs rumors
   - Official company announcements
   - Analyst coverage
   - Media sentiment bias

5. Market Reaction
   - Historical news impact
   - Price movement correlation
   - Volume spikes on news
   - Pre/post market behavior

Provide a comprehensive news analysis with:
- Overall news sentiment score (-100 to +100)
- Key news themes and their impact
- Breaking news alerts if any
- Expected market reaction
- Confidence level (0-100%)
- Time-sensitive considerations

Format your response with clear news impact assessment and actionable insights."""
        
        return prompt
        
    def _get_news_data(self, symbol: str, company_name: str, lookback_days: int) -> str:
        """Retrieve and format news data"""
        try:
            # For demo purposes, create mock news data
            # In production, integrate with NewsAPI, Bloomberg, Reuters, etc.
            
            news_items = [
                {
                    "title": f"{company_name or symbol} Reports Strong Q3 Earnings",
                    "source": "Financial Times",
                    "timestamp": datetime.now() - timedelta(hours=2),
                    "sentiment": "positive",
                    "summary": "Company exceeded earnings expectations with 15% YoY growth"
                },
                {
                    "title": f"Analysts Upgrade {company_name or symbol} to Buy",
                    "source": "Reuters",
                    "timestamp": datetime.now() - timedelta(hours=5),
                    "sentiment": "positive",
                    "summary": "Multiple investment banks raised price targets following strong results"
                },
                {
                    "title": f"{company_name or symbol} Announces New Product Launch",
                    "source": "TechCrunch",
                    "timestamp": datetime.now() - timedelta(days=1),
                    "sentiment": "positive",
                    "summary": "Innovative product line expansion expected to drive growth"
                },
                {
                    "title": f"Regulatory Review for {company_name or symbol} Industry",
                    "source": "Wall Street Journal",
                    "timestamp": datetime.now() - timedelta(days=2),
                    "sentiment": "negative",
                    "summary": "New regulations may impact operating margins"
                },
                {
                    "title": f"{company_name or symbol} Competitor Announces Partnership",
                    "source": "Bloomberg",
                    "timestamp": datetime.now() - timedelta(days=3),
                    "sentiment": "neutral",
                    "summary": "Competitive landscape shift with strategic alliance"
                }
            ]
            
            # Format news data
            formatted_news = []
            for item in news_items:
                formatted_news.append(
                    f"[{item['timestamp'].strftime('%Y-%m-%d %H:%M')}] {item['source']}: {item['title']}\n"
                    f"Sentiment: {item['sentiment']}\n"
                    f"Summary: {item['summary']}\n"
                )
                
            return "\n".join(formatted_news)
            
        except Exception as e:
            return f"Error retrieving news data: {str(e)}"
            
    def _analyze_sentiment(self, news_data: str) -> str:
        """Analyze sentiment from news data"""
        try:
            # Extract news items
            news_items = news_data.split("\n\n")
            
            sentiments = []
            for item in news_items:
                if "Sentiment:" in item:
                    sentiment_line = [line for line in item.split("\n") if "Sentiment:" in line]
                    if sentiment_line:
                        sentiment = sentiment_line[0].split(":")[1].strip()
                        sentiments.append(sentiment)
                        
            if not sentiments:
                return "No sentiment data available"
                
            # Calculate sentiment distribution
            positive_count = sentiments.count("positive")
            negative_count = sentiments.count("negative")
            neutral_count = sentiments.count("neutral")
            
            total_count = len(sentiments)
            
            sentiment_score = (
                (positive_count * 1.0) + 
                (negative_count * -1.0) + 
                (neutral_count * 0.0)
            ) / total_count if total_count > 0 else 0
            
            # Convert to -100 to +100 scale
            sentiment_score_normalized = sentiment_score * 100
            
            analysis = []
            analysis.append(f"Overall Sentiment Score: {sentiment_score_normalized:+.1f}/100")
            analysis.append(f"Positive News: {positive_count}/{total_count} ({positive_count/total_count*100:.1f}%)")
            analysis.append(f"Negative News: {negative_count}/{total_count} ({negative_count/total_count*100:.1f}%)")
            analysis.append(f"Neutral News: {neutral_count}/{total_count} ({neutral_count/total_count*100:.1f}%)")
            
            # Sentiment trend
            if sentiment_score_normalized > 20:
                analysis.append("Sentiment Trend: Bullish")
            elif sentiment_score_normalized < -20:
                analysis.append("Sentiment Trend: Bearish")
            else:
                analysis.append("Sentiment Trend: Neutral")
                
            return "\n".join(analysis)
            
        except Exception as e:
            return f"Error analyzing sentiment: {str(e)}"
            
    async def analyze_news_impact_score(self, symbol: str, lookback_days: int = 7) -> Dict[str, Any]:
        """
        Calculate news impact score based on volume, sentiment, and importance
        
        Returns:
            Dictionary with impact score and components
        """
        try:
            # Get news data
            company_name = symbol  # In production, map symbol to company name
            news_data = self._get_news_data(symbol, company_name, lookback_days)
            
            # Count news items
            news_items = news_data.split("\n\n")
            news_count = len([item for item in news_items if item.strip()])
            
            # Calculate sentiment score
            sentiment_analysis = self._analyze_sentiment(news_data)
            sentiment_match = re.search(r"Overall Sentiment Score: ([+-]?\d+\.?\d*)", sentiment_analysis)
            sentiment_score = float(sentiment_match.group(1)) if sentiment_match else 0
            
            # Calculate recency score (more recent news has higher impact)
            recency_scores = []
            for item in news_items:
                if "[" in item and "]" in item:
                    timestamp_str = item.split("[")[1].split("]")[0]
                    try:
                        news_time = datetime.strptime(timestamp_str, "%Y-%m-%d %H:%M")
                        hours_ago = (datetime.now() - news_time).total_seconds() / 3600
                        recency_score = max(0, 100 - (hours_ago / 24 * 20))  # Decay over 5 days
                        recency_scores.append(recency_score)
                    except:
                        recency_scores.append(0)
                        
            avg_recency = sum(recency_scores) / len(recency_scores) if recency_scores else 0
            
            # Calculate source quality score
            high_quality_sources = ["Reuters", "Bloomberg", "Wall Street Journal", "Financial Times"]
            source_score = 0
            for item in news_items:
                for source in high_quality_sources:
                    if source.lower() in item.lower():
                        source_score += 20
                        break
                        
            source_score = min(100, source_score)
            
            # Calculate overall impact score
            volume_score = min(100, news_count * 20)  # 20 points per news item, max 100
            impact_score = (
                (volume_score * 0.4) +
                (abs(sentiment_score) * 0.3) +
                (avg_recency * 0.2) +
                (source_score * 0.1)
            )
            
            return {
                'symbol': symbol,
                'impact_score': min(100, impact_score),
                'components': {
                    'volume_score': volume_score,
                    'sentiment_score': abs(sentiment_score),
                    'recency_score': avg_recency,
                    'source_score': source_score
                },
                'news_count': news_count,
                'sentiment_direction': 'bullish' if sentiment_score > 0 else 'bearish' if sentiment_score < 0 else 'neutral',
                'sentiment_magnitude': abs(sentiment_score),
                'timestamp': datetime.now()
            }
            
        except Exception as e:
            return {
                'symbol': symbol,
                'impact_score': 0,
                'error': str(e),
                'timestamp': datetime.now()
            }
            
    async def detect_breaking_news(self, symbol: str, minutes_back: int = 60) -> List[Dict[str, Any]]:
        """
        Detect breaking news within specified time window
        
        Returns:
            List of breaking news items
        """
        try:
            # Get recent news
            company_name = symbol
            news_data = self._get_news_data(symbol, company_name, 1)
            
            breaking_news = []
            current_time = datetime.now()
            
            # Parse news items
            news_items = news_data.split("\n\n")
            for item in news_items:
                if not item.strip():
                    continue
                    
                # Extract timestamp
                if "[" in item and "]" in item:
                    timestamp_str = item.split("[")[1].split("]")[0]
                    try:
                        news_time = datetime.strptime(timestamp_str, "%Y-%m-%d %H:%M")
                        minutes_ago = (current_time - news_time).total_seconds() / 60
                        
                        if minutes_ago <= minutes_back:
                            # Extract title and source
                            lines = item.split("\n")
                            title = lines[0].split("]: ")[1] if "]: " in lines[0] else ""
                            source = lines[0].split("[")[1].split("]")[0] if "[" in lines[0] else ""
                            
                            # Extract sentiment
                            sentiment_line = [line for line in lines if "Sentiment:" in line]
                            sentiment = sentiment_line[0].split(":")[1].strip() if sentiment_line else "unknown"
                            
                            breaking_news.append({
                                'title': title,
                                'source': source,
                                'timestamp': news_time,
                                'minutes_ago': minutes_ago,
                                'sentiment': sentiment,
                                'urgency': 'high' if minutes_ago < 15 else 'medium'
                            })
                            
                    except:
                        continue
                        
            # Sort by recency
            breaking_news.sort(key=lambda x: x['minutes_ago'])
            
            return breaking_news
            
        except Exception as e:
            return [{'error': str(e), 'timestamp': datetime.now()}]

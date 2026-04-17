"""
Social media analysis agent for sentiment tracking from social platforms.
"""

from typing import Dict, Any, List, Optional
import asyncio
import aiohttp
from datetime import datetime, timedelta
import re
from textblob import TextBlob

from .base import BaseAnalysisAgent, ExecutionMode


class SocialMediaAnalyst(BaseAnalysisAgent):
    """
    Analyzes social media sentiment including:
    - Twitter/X sentiment tracking
    - Reddit discussion analysis
    - StockTwits sentiment
    - Influencer opinions
    - Viral content detection
    - Community engagement metrics
    """
    
    def __init__(
        self,
        llm_provider: str = "openai",
        model_name: str = "gpt-4-turbo-preview",
        execution_mode: ExecutionMode = ExecutionMode.HYBRID
    ):
        super().__init__(
            name="social_media_analyst",
            llm_provider=llm_provider,
            model_name=model_name,
            execution_mode=execution_mode
        )
        
    def _build_prompt(self, input_data: Dict[str, Any]) -> str:
        """Build social media analysis prompt"""
        symbol = input_data.get("symbol", "")
        company_name = input_data.get("company_name", "")
        lookback_hours = input_data.get("lookback_hours", 24)
        
        if not symbol and not company_name:
            return "Error: No symbol or company name provided for social media analysis"
            
        # Get social media data
        social_data = self._get_social_media_data(symbol, company_name, lookback_hours)
        sentiment_analysis = self._analyze_social_sentiment(social_data)
        
        prompt = f"""As a Social Media Analyst, analyze social media sentiment for {symbol or company_name}.

SOCIAL MEDIA DATA:
{social_data}

SENTIMENT ANALYSIS:
{sentiment_analysis}

ANALYSIS FRAMEWORK:
1. Platform-Specific Sentiment
   - Twitter/X sentiment trends
   - Reddit discussion sentiment
   - StockTwits community sentiment
   - Other platform insights

2. Engagement Metrics
   - Mention volume and velocity
   - Like/retweet ratios
   - Comment sentiment depth
   - Influencer engagement

3. Sentiment Drivers
   - Key discussion themes
   - Viral content analysis
   - News-driven sentiment
   - Technical analysis discussions

4. Community Insights
   - Retail vs institutional sentiment
   - Bull/bear community ratios
   - Sentiment consistency across platforms
   - Contrarian signals

5. Trend Analysis
   - Sentiment momentum
   - Reversal patterns
   - Unusual activity spikes
   - Predictive indicators

Provide a comprehensive social media analysis with:
- Overall social sentiment score (-100 to +100)
- Platform-specific insights
- Key discussion themes
- Influencer opinions summary
- Predictive signals
- Confidence level (0-100%)
- Time-sensitive considerations

Format your response with clear social media insights and actionable sentiment indicators."""
        
        return prompt
        
    def _get_social_media_data(self, symbol: str, company_name: str, lookback_hours: int) -> str:
        """Retrieve and format social media data"""
        try:
            # For demo purposes, create mock social media data
            # In production, integrate with Twitter API, Reddit API, StockTwits, etc.
            
            social_posts = [
                {
                    "platform": "Twitter/X",
                    "author": "TraderJoe",
                    "followers": 50000,
                    "timestamp": datetime.now() - timedelta(hours=1),
                    "content": f"Bullish on {symbol}! Strong technical setup and great fundamentals. #investing",
                    "likes": 245,
                    "retweets": 52,
                    "sentiment": "positive"
                },
                {
                    "platform": "Reddit",
                    "author": "StockAnalysis",
                    "timestamp": datetime.now() - timedelta(hours=3),
                    "content": f"Anyone else concerned about {company_name or symbol}'s valuation? P/E ratio seems high given growth prospects.",
                    "upvotes": 156,
                    "comments": 89,
                    "sentiment": "negative"
                },
                {
                    "platform": "StockTwits",
                    "author": "BullTrader2024",
                    "timestamp": datetime.now() - timedelta(hours=5),
                    "content": f"{symbol} to the moon! 🚀 Breaking resistance levels. Next stop: $200!",
                    "sentiment": "positive",
                    "likes": 45
                },
                {
                    "platform": "Twitter/X",
                    "author": "FinanceGuru",
                    "followers": 150000,
                    "timestamp": datetime.now() - timedelta(hours=8),
                    "content": f"Analysis: {company_name or symbol} facing headwinds from regulatory changes. Short-term caution advised.",
                    "likes": 892,
                    "retweets": 234,
                    "sentiment": "negative"
                },
                {
                    "platform": "Reddit",
                    "author": "ValueInvestor",
                    "timestamp": datetime.now() - timedelta(hours=12),
                    "content": f"Deep dive on {company_name or symbol}: Cash flow strong, debt manageable. Long-term value play.",
                    "upvotes": 412,
                    "comments": 67,
                    "sentiment": "positive"
                }
            ]
            
            # Format social media data
            formatted_posts = []
            for post in social_posts:
                engagement = ""
                if post["platform"] == "Twitter/X":
                    engagement = f"Likes: {post['likes']}, Retweets: {post['retweets']}"
                elif post["platform"] == "Reddit":
                    engagement = f"Upvotes: {post['upvotes']}, Comments: {post['comments']}"
                elif post["platform"] == "StockTwits":
                    engagement = f"Likes: {post['likes']}"
                    
                follower_info = f" (Followers: {post['followers']})" if "followers" in post else ""
                
                formatted_posts.append(
                    f"[{post['timestamp'].strftime('%Y-%m-%d %H:%M')}] {post['platform']}\n"
                    f"Author: {post['author']}{follower_info}\n"
                    f"Content: {post['content']}\n"
                    f"Engagement: {engagement}\n"
                    f"Sentiment: {post['sentiment']}\n"
                )
                
            return "\n".join(formatted_posts)
            
        except Exception as e:
            return f"Error retrieving social media data: {str(e)}"
            
    def _analyze_social_sentiment(self, social_data: str) -> str:
        """Analyze sentiment from social media data"""
        try:
            # Extract posts
            posts = social_data.split("\n\n")
            
            sentiments = []
            platform_sentiments = {}
            
            for post in posts:
                if "Sentiment:" in post:
                    # Extract sentiment
                    sentiment_line = [line for line in post.split("\n") if "Sentiment:" in line]
                    if sentiment_line:
                        sentiment = sentiment_line[0].split(":")[1].strip()
                        sentiments.append(sentiment)
                        
                    # Extract platform
                    platform_line = [line for line in post.split("\n") if "]" in line and "Twitter" in line or "Reddit" in line or "StockTwits" in line]
                    if platform_line:
                        platform = platform_line[0].split("]")[1].strip()
                        if platform not in platform_sentiments:
                            platform_sentiments[platform] = []
                        platform_sentiments[platform].append(sentiment)
                        
            if not sentiments:
                return "No sentiment data available"
                
            # Calculate overall sentiment
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
            analysis.append(f"Overall Social Sentiment: {sentiment_score_normalized:+.1f}/100")
            analysis.append(f"Positive Posts: {positive_count}/{total_count} ({positive_count/total_count*100:.1f}%)")
            analysis.append(f"Negative Posts: {negative_count}/{total_count} ({negative_count/total_count*100:.1f}%)")
            analysis.append(f"Neutral Posts: {neutral_count}/{total_count} ({neutral_count/total_count*100:.1f}%)")
            
            # Platform-specific sentiment
            analysis.append("\nPlatform-Specific Sentiment:")
            for platform, platform_sentiment_list in platform_sentiments.items():
                pos = platform_sentiment_list.count("positive")
                neg = platform_sentiment_list.count("negative")
                neu = platform_sentiment_list.count("neutral")
                total = len(platform_sentiment_list)
                
                platform_score = ((pos - neg) / total * 100) if total > 0 else 0
                analysis.append(f"  {platform}: {platform_score:+.1f}/100")
                
            # Influencer sentiment (high follower accounts)
            influencer_posts = []
            for post in posts:
                if "Followers:" in post and int(post.split("Followers: ")[1].split("\n")[0]) > 10000:
                    sentiment_line = [line for line in post.split("\n") if "Sentiment:" in line]
                    if sentiment_line:
                        influencer_posts.append(sentiment_line[0].split(":")[1].strip())
                        
            if influencer_posts:
                influencer_pos = influencer_posts.count("positive")
                influencer_neg = influencer_posts.count("negative")
                influencer_total = len(influencer_posts)
                influencer_score = ((influencer_pos - influencer_neg) / influencer_total * 100) if influencer_total > 0 else 0
                analysis.append(f"\nInfluencer Sentiment: {influencer_score:+.1f}/100")
                
            return "\n".join(analysis)
            
        except Exception as e:
            return f"Error analyzing social sentiment: {str(e)}"
            
    async def analyze_social_influence_score(self, symbol: str, lookback_hours: int = 24) -> Dict[str, Any]:
        """
        Calculate social influence score based on reach, engagement, and sentiment
        
        Returns:
            Dictionary with influence score and components
        """
        try:
            # Get social media data
            company_name = symbol
            social_data = self._get_social_media_data(symbol, company_name, lookback_hours)
            
            # Extract metrics
            posts = social_data.split("\n\n")
            
            # Calculate volume score
            volume_score = min(100, len(posts) * 20)  # 20 points per post
            
            # Calculate reach score (based on followers)
            total_reach = 0
            for post in posts:
                if "Followers:" in post:
                    followers = int(post.split("Followers: ")[1].split("\n")[0])
                    total_reach += followers
                    
            reach_score = min(100, total_reach / 10000)  # Normalize to 0-100
            
            # Calculate engagement score
            total_engagement = 0
            for post in posts:
                if "Likes:" in post:
                    likes = int(post.split("Likes: ")[1].split(",")[0])
                    total_engagement += likes
                if "Retweets:" in post:
                    retweets = int(post.split("Retweets: ")[1])
                    total_engagement += retweets * 2  # Weight retweets higher
                if "Upvotes:" in post:
                    upvotes = int(post.split("Upvotes: ")[1].split(",")[0])
                    total_engagement += upvotes
                    
            engagement_score = min(100, total_engagement / 100)
            
            # Calculate sentiment score
            sentiment_analysis = self._analyze_social_sentiment(social_data)
            sentiment_match = re.search(r"Overall Social Sentiment: ([+-]?\d+\.?\d*)", sentiment_analysis)
            sentiment_score = float(sentiment_match.group(1)) if sentiment_match else 0
            
            # Calculate overall influence score
            influence_score = (
                (volume_score * 0.3) +
                (reach_score * 0.3) +
                (engagement_score * 0.2) +
                (abs(sentiment_score) * 0.2)
            )
            
            return {
                'symbol': symbol,
                'influence_score': min(100, influence_score),
                'components': {
                    'volume_score': volume_score,
                    'reach_score': reach_score,
                    'engagement_score': engagement_score,
                    'sentiment_score': abs(sentiment_score)
                },
                'post_count': len(posts),
                'total_reach': total_reach,
                'total_engagement': total_engagement,
                'sentiment_direction': 'bullish' if sentiment_score > 0 else 'bearish' if sentiment_score < 0 else 'neutral',
                'sentiment_magnitude': abs(sentiment_score),
                'timestamp': datetime.now()
            }
            
        except Exception as e:
            return {
                'symbol': symbol,
                'influence_score': 0,
                'error': str(e),
                'timestamp': datetime.now()
            }
            
    async def detect_viral_content(self, symbol: str, hours_back: int = 6) -> List[Dict[str, Any]]:
        """
        Detect viral content related to the symbol
        
        Returns:
            List of viral posts with engagement metrics
        """
        try:
            # Get recent social media data
            company_name = symbol
            social_data = self._get_social_media_data(symbol, company_name, hours_back)
            
            viral_posts = []
            posts = social_data.split("\n\n")
            
            # Define viral thresholds
            viral_thresholds = {
                'Twitter/X': {'likes': 500, 'retweets': 100},
                'Reddit': {'upvotes': 1000, 'comments': 100},
                'StockTwits': {'likes': 100}
            }
            
            for post in posts:
                if not post.strip():
                    continue
                    
                # Extract platform and metrics
                platform = None
                metrics = {}
                
                if "Twitter/X" in post:
                    platform = "Twitter/X"
                    if "Likes:" in post:
                        metrics['likes'] = int(post.split("Likes: ")[1].split(",")[0])
                    if "Retweets:" in post:
                        metrics['retweets'] = int(post.split("Retweets: ")[1])
                        
                elif "Reddit" in post:
                    platform = "Reddit"
                    if "Upvotes:" in post:
                        metrics['upvotes'] = int(post.split("Upvotes: ")[1].split(",")[0])
                    if "Comments:" in post:
                        metrics['comments'] = int(post.split("Comments: ")[1])
                        
                elif "StockTwits" in post:
                    platform = "StockTwits"
                    if "Likes:" in post:
                        metrics['likes'] = int(post.split("Likes: ")[1])
                        
                # Check if viral
                if platform and platform in viral_thresholds:
                    is_viral = False
                    for metric, threshold in viral_thresholds[platform].items():
                        if metric in metrics and metrics[metric] >= threshold:
                            is_viral = True
                            break
                            
                    if is_viral:
                        # Extract content details
                        content_line = [line for line in post.split("\n") if "Content:" in line]
                        author_line = [line for line in post.split("\n") if "Author:" in line]
                        timestamp_line = [line for line in post.split("\n") if "[" in line and "]" in line]
                        
                        viral_posts.append({
                            'platform': platform,
                            'author': author_line[0].split(": ")[1] if author_line else "Unknown",
                            'content': content_line[0].split(": ")[1] if content_line else "",
                            'metrics': metrics,
                            'timestamp': timestamp_line[0].split("[")[1].split("]")[0] if timestamp_line else "",
                            'viral_score': sum(metrics.values())  # Simple score based on total engagement
                        })
                        
            # Sort by viral score
            viral_posts.sort(key=lambda x: x['viral_score'], reverse=True)
            
            return viral_posts
            
        except Exception as e:
            return [{'error': str(e), 'timestamp': datetime.now()}]

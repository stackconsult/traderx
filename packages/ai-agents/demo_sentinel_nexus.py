#!/usr/bin/env python3
"""
Demonstration of Sentinel-Nexus architecture integrated with TraderX.

This script shows:
1. Analysis agents processing market data
2. Research team debate (bull/bear)
3. Risk consensus building
4. Portfolio decision making
5. Integration with TraderX execution layer
"""

import asyncio
import logging
import sys
import os
from datetime import datetime
from unittest.mock import Mock, patch, AsyncMock

# Add src to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'src'))

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


async def mock_external_dependencies():
    """Mock all external dependencies for demo"""
    
    # Mock yfinance
    mock_ticker = Mock()
    mock_ticker.info = {
        'longName': 'DemoTech Corporation',
        'sector': 'Technology',
        'marketCap': 50000000000,
        'trailingPE': 25.0,
        'priceToBook': 4.0,
        'returnOnEquity': 0.20,
        'returnOnAssets': 0.12,
        'profitMargins': 0.15,
        'debtToEquity': 0.4,
        'currentRatio': 2.5,
        'revenueGrowth': 0.20,
        'earningsGrowth': 0.25
    }
    
    mock_financials = Mock()
    mock_financials.empty = False
    mock_financials.iloc = Mock()
    mock_financials.iloc.__getitem__ = Mock(return_value=500000000)
    
    # Create mock DataFrame that supports item access
    mock_close_series = Mock()
    mock_close_series.iloc = Mock()
    mock_close_series.iloc.__getitem__ = Mock(return_value=150.0)
    mock_close_series.pct_change = Mock(return_value=mock_close_series)
    mock_close_series.dropna = Mock(return_value=[0.01, -0.02, 0.03])
    
    mock_hist = Mock()
    mock_hist.empty = False
    mock_hist.__getitem__ = Mock(return_value=mock_close_series)
    mock_hist.__iter__ = Mock(return_value=iter(['Close', 'Volume']))
    mock_hist.get = Mock(return_value=mock_close_series)
    
    mock_ticker_instance = Mock()
    mock_ticker_instance.info = mock_ticker.info
    mock_ticker_instance.financials = mock_financials
    mock_ticker_instance.history = Mock(return_value=mock_hist)
    
    # Mock LLM
    mock_llm = Mock()
    mock_llm.model_name = "gpt-4-demo"
    
    def mock_ainvoke(messages):
        response = Mock()
        if "BULLISH" in str(messages):
            response.content = """
            Bullish Thesis for DEMO:
            
            Strong growth catalysts:
            - 20% revenue growth accelerating
            - New AI product launch Q3
            - Market expansion in Asia
            
            Valuation attractive at 25x P/E given 25% earnings growth.
            Target price: $200 (33% upside)
            
            Confidence: 85%
            """
        elif "BEARISH" in str(messages):
            response.content = """
            Bearish Thesis for DEMO:
            
            Significant risks:
            - Valuation stretched at 4x P/B
            - Competition intensifying
            - Regulatory scrutiny increasing
            
            Potential 20% downside if growth slows.
            
            Confidence: 75%
            """
        elif "MODERATOR" in str(messages):
            response.content = """
            Moderated Conclusion:
            
            Recommendation: BUY
            
            Bullish arguments outweigh bearish risks due to:
            - Strong execution track record
            - Large addressable market
            - Innovation pipeline
            
            Position Size: 8% of portfolio
            Stop Loss: $120
            Time Horizon: 12-18 months
            
            Confidence: 80%
            """
        else:
            response.content = "Analysis complete. Confidence: 75%"
            
        return AsyncMock(return_value=response)
    
    mock_llm.ainvoke = mock_ainvoke
    
    # Apply patches - patch at the source module level
    patches = [
        patch('yfinance.Ticker', return_value=mock_ticker_instance),
        patch('langchain_openai.ChatOpenAI', return_value=mock_llm),
        patch('talib.SMA', return_value=[145, 148, 150]),
        patch('talib.RSI', return_value=[55]),
        patch('talib.MACD', return_value=([1, 2, 3], [1, 2, 3], [0, 1, 2])),
    ]
    
    for p in patches:
        p.start()
        
    return patches


async def run_sentinel_nexus_demo():
    """Run the complete Sentinel-Nexus demonstration"""
    
    print("\n" + "="*80)
    print("SENTINEL-NEXUS ARCHITECTURE DEMONSTRATION")
    print("="*80)
    
    # Mock external dependencies
    patches = await mock_external_dependencies()
    
    try:
        # Import after patching
        from traderx_ai_agents.analysis import (
            FundamentalsAnalyst,
            MarketAnalyst,
            NewsAnalyst,
            SocialMediaAnalyst
        )
        from traderx_ai_agents.analysis.researchers import ResearchTeam
        from traderx_ai_agents.risk import PortfolioManager, RiskConsensusAgent
        from traderx_ai_agents.integration.traderx_bridge import TraderXBridge, ExecutionAgentInterface
        
        # Initialize components
        print("\n1. INITIALIZING SENTINEL-NEXUS COMPONENTS")
        print("-" * 40)
        
        # Analysis layer
        fundamentals = FundamentalsAnalyst()
        market = MarketAnalyst()
        news = NewsAnalyst()
        social = SocialMediaAnalyst()
        
        print("✓ Analysis agents initialized")
        
        # Research team
        research_team = ResearchTeam()
        print("✓ Research team initialized")
        
        # Risk layer
        portfolio_manager = PortfolioManager(max_position_size=0.10)
        risk_consensus = RiskConsensusAgent()
        print("✓ Risk agents initialized")
        
        # Integration bridge
        bridge = TraderXBridge(redis_url="redis://localhost:6379/0")  # Use DB 0 for demo
        execution_interface = ExecutionAgentInterface(bridge)
        
        print("✓ Integration bridge initialized")
        
        # Demo symbol
        symbol = "DEMO"
        current_price = 150.0
        
        print(f"\n2. RUNNING ANALYSIS FOR {symbol}")
        print("-" * 40)
        
        # Step 1: Run analysis agents
        analysis_results = {}
        
        print("\nRunning fundamental analysis...")
        fundamentals_result = await fundamentals.analyze({"symbol": symbol})
        analysis_results["fundamentals"] = fundamentals_result
        print(f"✓ Confidence: {fundamentals_result.confidence:.2%}")
        
        print("\nRunning market analysis...")
        market_result = await market.analyze({"symbol": symbol})
        analysis_results["market"] = market_result
        print(f"✓ Confidence: {market_result.confidence:.2%}")
        
        print("\nRunning news analysis...")
        news_result = await news.analyze({"symbol": symbol})
        analysis_results["news"] = news_result
        print(f"✓ Confidence: {news_result.confidence:.2%}")
        
        print("\nRunning social media analysis...")
        social_result = await social.analyze({"symbol": symbol})
        analysis_results["social"] = social_result
        print(f"✓ Confidence: {social_result.confidence:.2%}")
        
        # Step 2: Research team debate
        print("\n3. RESEARCH TEAM DEBATE")
        print("-" * 40)
        
        research_result = await research_team.conduct_research(symbol, analysis_results)
        
        bull_thesis = research_result["research_team"]["bull_thesis"]
        bear_thesis = research_result["research_team"]["bear_thesis"]
        moderated = research_result["research_team"]["moderated_conclusion"]
        
        print(f"\nBull thesis confidence: {bull_thesis['confidence']:.2%}")
        print(f"Bear thesis confidence: {bear_thesis['confidence']:.2%}")
        print(f"Moderated recommendation: {moderated['recommendation']}")
        print(f"Moderated confidence: {moderated['confidence']:.2%}")
        
        # Step 3: Risk assessment
        print("\n4. RISK CONSENSUS BUILDING")
        print("-" * 40)
        
        risk_data = {
            "volatility": 0.25,
            "beta": 1.2,
            "debt_to_equity": 0.4,
            "volume_ratio": 1.5,
            "bid_ask_spread": 0.02,
            "liquidity_score": 0.8
        }
        
        risk_assessment = await risk_consensus.assess_risk({
            "symbol": symbol,
            "market_data": risk_data
        })
        
        print(f"Risk level: {risk_assessment.risk_level.value}")
        print(f"Risk score: {risk_assessment.risk_score:.2%}")
        print(f"Risk confidence: {risk_assessment.confidence:.2%}")
        print(f"Key risks: {', '.join(risk_assessment.factors[:3])}")
        
        # Step 4: Portfolio decision
        print("\n5. PORTFOLIO DECISION")
        print("-" * 40)
        
        portfolio_decision = await portfolio_manager.make_portfolio_decision(
            symbol=symbol,
            research_data=research_result["research_team"],
            risk_data=risk_data,
            current_price=current_price
        )
        
        print(f"Recommendation: {portfolio_decision.recommendation.value}")
        print(f"Position size: {portfolio_decision.position_size:.2%}")
        print(f"Entry price: ${portfolio_decision.entry_price:.2f}")
        print(f"Target price: ${portfolio_decision.target_price:.2f}")
        print(f"Stop loss: ${portfolio_decision.stop_loss:.2f}")
        print(f"Time horizon: {portfolio_decision.time_horizon}")
        print(f"Conviction: {portfolio_decision.conviction:.2%}")
        
        # Step 5: Integration with TraderX
        print("\n6. TRADERX INTEGRATION")
        print("-" * 40)
        
        # Simulate bridge initialization (without actual Redis)
        print("\nInitializing TraderX bridge...")
        print("✓ Bridge connected to Redis")
        print("✓ Execution agent interface started")
        
        # Publish signals
        print("\nPublishing signals to TraderX...")
        
        # Analysis signals
        for name, result in analysis_results.items():
            print(f"  → Published {name} analysis (confidence: {result.confidence:.2%})")
            
        # Portfolio decision
        print(f"  → Published portfolio decision: {portfolio_decision.recommendation.value}")
        
        # Simulate execution
        print("\nSimulating execution...")
        if portfolio_decision.recommendation.value in ['buy', 'strong_buy']:
            order = {
                'symbol': symbol,
                'side': 'buy',
                'quantity': int(1000000 * portfolio_decision.position_size / current_price),
                'price': portfolio_decision.entry_price
            }
            print(f"  → Buy order submitted: {order['quantity']} shares at ${order['price']:.2f}")
            
        # Summary
        print("\n7. SUMMARY")
        print("-" * 40)
        print(f"Symbol: {symbol}")
        print(f"Final Recommendation: {portfolio_decision.recommendation.value.upper()}")
        print(f"Position Size: {portfolio_decision.position_size:.2%} of portfolio")
        print(f"Expected Return: {((portfolio_decision.target_price / current_price - 1) * 100):.1f}%")
        print(f"Risk Level: {risk_assessment.risk_level.value}")
        print(f"Overall Confidence: {portfolio_decision.conviction:.2%}")
        
        print("\n" + "="*80)
        print("DEMONSTRATION COMPLETE")
        print("="*80)
        
    except Exception as e:
        logger.error(f"Demo failed: {e}", exc_info=True)
    finally:
        # Clean up patches
        for p in patches:
            p.stop()


if __name__ == "__main__":
    print("Starting Sentinel-Nexus Demo...")
    print("Note: This demo uses mock data for illustration purposes.")
    print()
    
    # Run the demo
    asyncio.run(run_sentinel_nexus_demo())

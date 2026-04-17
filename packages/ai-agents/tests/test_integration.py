"""
Integration tests for Sentinel-Nexus architecture components.
"""

import pytest
import asyncio
from unittest.mock import Mock, patch, AsyncMock
from datetime import datetime
import sys
import os

# Add src to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

# Test imports with error handling
try:
    from traderx_ai_agents.analysis.base import BaseAnalysisAgent, ExecutionMode, CircuitBreaker
    from traderx_ai_agents.analysis.fundamentals import FundamentalsAnalyst
    from traderx_ai_agents.analysis.market import MarketAnalyst
    from traderx_ai_agents.analysis.news import NewsAnalyst
    from traderx_ai_agents.analysis.social_media import SocialMediaAnalyst
    from traderx_ai_agents.analysis.researchers import BullResearcher, BearResearcher, ResearchTeam
    from traderx_ai_agents.risk.base import BaseRiskAgent, RiskLevel
    from traderx_ai_agents.risk.portfolio_manager import PortfolioManager, Recommendation
    from traderx_ai_agents.risk.risk_consensus import RiskConsensusAgent, ConsensusMethod
    ANALYSIS_AVAILABLE = True
except ImportError as e:
    print(f"Import error: {e}")
    ANALYSIS_AVAILABLE = False


@pytest.mark.skipif(not ANALYSIS_AVAILABLE, reason="Analysis components not available")
class TestCircuitBreaker:
    """Test circuit breaker functionality"""
    
    @pytest.mark.asyncio
    async def test_circuit_breaker_closed_state(self):
        """Test circuit breaker in closed state"""
        circuit = CircuitBreaker(failure_threshold=3, timeout_seconds=60)
        
        async def mock_func():
            return "success"
            
        result = await circuit.call(mock_func)
        assert result == "success"
        assert circuit.state == CircuitState.CLOSED
        
    @pytest.mark.asyncio
    async def test_circuit_breaker_opens_on_failures(self):
        """Test circuit breaker opens after threshold failures"""
        circuit = CircuitBreaker(failure_threshold=2, timeout_seconds=1)
        
        async def failing_func():
            raise Exception("Test failure")
            
        # Fail twice to trigger open state
        with pytest.raises(Exception):
            await circuit.call(failing_func)
            
        with pytest.raises(Exception):
            await circuit.call(failing_func)
            
        assert circuit.state == CircuitBreaker.CircuitState.OPEN
        
        # Should reject calls when open
        with pytest.raises(Exception, match="Circuit breaker is OPEN"):
            await circuit.call(failing_func)


@pytest.mark.skipif(not ANALYSIS_AVAILABLE, reason="Analysis components not available")
class TestBaseAnalysisAgent:
    """Test base analysis agent functionality"""
    
    @pytest.fixture
    def mock_llm(self):
        """Create a mock LLM"""
        llm = Mock()
        llm.model_name = "gpt-4-test"
        llm.ainvoke = AsyncMock(return_value=Mock(content="Test analysis response"))
        return llm
        
    @pytest.fixture
    def test_agent(self, mock_llm):
        """Create a test analysis agent"""
        class TestAgent(BaseAnalysisAgent):
            def _build_prompt(self, input_data):
                return f"Analyze: {input_data}"
                
        with patch('traderx_ai_agents.analysis.base.ChatOpenAI', return_value=mock_llm):
            agent = TestAgent(
                name="test_agent",
                llm_provider="openai",
                model_name="gpt-4-test",
                execution_mode=ExecutionMode.HYBRID
            )
            return agent
            
    @pytest.mark.asyncio
    async def test_agent_analyze(self, test_agent):
        """Test basic analysis functionality"""
        input_data = {"symbol": "TEST", "data": "test"}
        result = await test_agent.analyze(input_data)
        
        assert result.agent_name == "test_agent"
        assert result.analysis == "Test analysis response"
        assert 0 <= result.confidence <= 1.0
        assert isinstance(result.timestamp, datetime)
        
    @pytest.mark.asyncio
    async def test_agent_caching(self, test_agent):
        """Test result caching"""
        input_data = {"symbol": "TEST", "data": "test"}
        
        # First call
        result1 = await test_agent.analyze(input_data)
        
        # Second call should use cache
        result2 = await test_agent.analyze(input_data)
        
        assert result1.analysis == result2.analysis
        assert result1.timestamp == result2.timestamp
        
    @pytest.mark.asyncio
    async def test_batch_analyze_sequential(self, test_agent):
        """Test batch analysis in sequential mode"""
        test_agent.execution_mode = ExecutionMode.SEQUENTIAL
        
        inputs = [{"symbol": f"TEST{i}"} for i in range(3)]
        results = await test_agent.batch_analyze(inputs)
        
        assert len(results) == 3
        for i, result in enumerate(results):
            assert f"TEST{i}" in result.analysis or result.analysis == "Test analysis response"


@pytest.mark.skipif(not ANALYSIS_AVAILABLE, reason="Analysis components not available")
class TestFundamentalsAnalyst:
    """Test fundamentals analyst"""
    
    @pytest.mark.asyncio
    @patch('yfinance.Ticker')
    async def test_fundamentals_analysis(self, mock_ticker):
        """Test fundamentals analysis with mock data"""
        # Mock yfinance data
        mock_info = {
            'longName': 'Test Corporation',
            'sector': 'Technology',
            'marketCap': 1000000000,
            'trailingPE': 20.5,
            'priceToBook': 3.2,
            'returnOnEquity': 0.15,
            'returnOnAssets': 0.08,
            'profitMargins': 0.12,
            'debtToEquity': 0.5,
            'currentRatio': 2.0,
            'revenueGrowth': 0.10,
            'earningsGrowth': 0.15
        }
        
        mock_financials = Mock()
        mock_financials.empty = False
        mock_financials.iloc = Mock()
        mock_financials.iloc.__getitem__ = Mock(return_value=100000000)
        
        mock_ticker_instance = Mock()
        mock_ticker_instance.info = mock_info
        mock_ticker_instance.financials = mock_financials
        mock_ticker.return_value = mock_ticker_instance
        
        # Create analyst with mock LLM
        with patch('traderx_ai_agents.analysis.fundamentals.ChatOpenAI') as mock_llm_class:
            mock_llm = Mock()
            mock_llm.ainvoke = AsyncMock(return_value=Mock(content="Fundamental analysis complete"))
            mock_llm.model_name = "gpt-4"
            mock_llm_class.return_value = mock_llm
            
            analyst = FundamentalsAnalyst()
            result = await analyst.analyze({"symbol": "TEST"})
            
            assert result.agent_name == "fundamentals_analyst"
            assert "Test Corporation" in result.data.get('financial_data', '')
            
    @pytest.mark.asyncio
    @patch('yfinance.Ticker')
    async def test_financial_health_score(self, mock_ticker):
        """Test financial health scoring"""
        # Mock data for health scoring
        mock_info = {
            'returnOnEquity': 0.20,
            'returnOnAssets': 0.10,
            'profitMargins': 0.15,
            'trailingPE': 15.0,
            'priceToBook': 2.0,
            'debtToEquity': 0.3,
            'currentRatio': 2.5,
            'revenueGrowth': 0.25,
            'earningsGrowth': 0.30
        }
        
        mock_ticker_instance = Mock()
        mock_ticker_instance.info = mock_info
        mock_ticker.return_value = mock_ticker_instance
        
        analyst = FundamentalsAnalyst()
        score = await analyst.analyze_financial_health_score("TEST")
        
        assert 'symbol' in score
        assert 'total_score' in score
        assert 0 <= score['total_score'] <= 100
        assert 'component_scores' in score
        assert 'rating' in score


@pytest.mark.skipif(not ANALYSIS_AVAILABLE, reason="Analysis components not available")
class TestResearchTeam:
    """Test research team functionality"""
    
    @pytest.mark.asyncio
    async def test_research_team_debate(self):
        """Test bull/bear debate system"""
        with patch('traderx_ai_agents.analysis.researchers.ChatOpenAI') as mock_llm_class:
            # Mock LLM responses
            mock_llm = Mock()
            mock_llm.ainvoke = AsyncMock(side_effect=[
                Mock(content="Bullish thesis with strong growth prospects"),
                Mock(content="Bearish thesis with significant risks"),
                Mock(content="Moderated conclusion: BUY with confidence 0.7")
            ])
            mock_llm.model_name = "gpt-4"
            mock_llm_class.return_value = mock_llm
            
            team = ResearchTeam()
            analysis_data = {
                "fundamentals": "Strong financials",
                "market": "Bullish trends",
                "news": "Positive news flow",
                "social": "Positive sentiment"
            }
            
            result = await team.conduct_research("TEST", analysis_data)
            
            assert 'symbol' in result
            assert 'research_team' in result
            assert 'bull_thesis' in result['research_team']
            assert 'bear_thesis' in result['research_team']
            assert 'moderated_conclusion' in result['research_team']
            
            # Check theses
            bull_thesis = result['research_team']['bull_thesis']
            assert bull_thesis['thesis_type'] == 'bullish'
            assert 'growth' in bull_thesis['thesis'].lower()
            
            bear_thesis = result['research_team']['bear_thesis']
            assert bear_thesis['thesis_type'] == 'bearish'
            assert 'risk' in bear_thesis['thesis'].lower()


@pytest.mark.skipif(not ANALYSIS_AVAILABLE, reason="Risk components not available")
class TestPortfolioManager:
    """Test portfolio manager functionality"""
    
    @pytest.fixture
    def portfolio_manager(self):
        """Create portfolio manager for testing"""
        return PortfolioManager(
            max_position_size=0.10,
            risk_tolerance=0.15,
            rebalance_threshold=0.05
        )
        
    @pytest.mark.asyncio
    async def test_risk_assessment(self, portfolio_manager):
        """Test risk assessment functionality"""
        input_data = {
            "symbol": "TEST",
            "research_data": {
                "moderated_conclusion": {
                    "confidence": 0.8,
                    "bull_confidence": 0.9,
                    "bear_confidence": 0.3
                }
            },
            "risk_data": {
                "volatility": 0.25,
                "liquidity_score": 0.8
            }
        }
        
        risk_assessment = await portfolio_manager.assess_risk(input_data)
        
        assert risk_assessment.agent_name == "portfolio_manager"
        assert isinstance(risk_assessment.risk_level, RiskLevel)
        assert 0 <= risk_assessment.risk_score <= 1.0
        assert isinstance(risk_assessment.factors, list)
        assert isinstance(risk_assessment.mitigation, list)
        
    @pytest.mark.asyncio
    async def test_portfolio_decision(self, portfolio_manager):
        """Test portfolio decision making"""
        research_data = {
            "moderated_conclusion": {
                "recommendation": "BUY",
                "confidence": 0.8,
                "moderated_thesis": "Strong buy recommendation based on growth prospects"
            }
        }
        
        risk_data = {
            "volatility": 0.2,
            "liquidity_score": 0.9
        }
        
        decision = await portfolio_manager.make_portfolio_decision(
            symbol="TEST",
            research_data=research_data,
            risk_data=risk_data,
            current_price=100.0
        )
        
        assert decision.symbol == "TEST"
        assert decision.recommendation == Recommendation.BUY
        assert 0 <= decision.position_size <= portfolio_manager.max_position_size
        assert decision.entry_price == 100.0
        assert decision.target_price > 100.0  # Should have upside target
        assert decision.stop_loss < 100.0  # Should have stop loss
        assert 0 <= decision.conviction <= 1.0
        assert decision.time_horizon in ["short", "medium", "long"]


@pytest.mark.skipif(not ANALYSIS_AVAILABLE, reason="Risk components not available")
class TestRiskConsensusAgent:
    """Test risk consensus agent"""
    
    @pytest.fixture
    def consensus_agent(self):
        """Create consensus agent for testing"""
        return RiskConsensusAgent(
            consensus_method=ConsensusMethod.WEIGHTED_AVERAGE,
            disagreement_threshold=0.3,
            min_confidence=0.6
        )
        
    @pytest.mark.asyncio
    async def test_consensus_building(self, consensus_agent):
        """Test consensus building among risk agents"""
        input_data = {
            "symbol": "TEST",
            "market_data": {
                "volatility": 0.25,
                "beta": 1.2,
                "debt_to_equity": 0.5,
                "volume_ratio": 1.5,
                "bid_ask_spread": 0.02
            }
        }
        
        consensus = await consensus_agent.assess_risk(input_data)
        
        assert consensus.agent_name == "risk_consensus_agent"
        assert isinstance(consensus.risk_level, RiskLevel)
        assert 0 <= consensus.risk_score <= 1.0
        assert 0 <= consensus.confidence <= 1.0
        assert isinstance(consensus.factors, list)
        assert isinstance(consensus.mitigation, list)
        
    def test_disagreement_calculation(self, consensus_agent):
        """Test disagreement score calculation"""
        from traderx_ai_agents.risk.base import RiskAssessment
        
        assessments = [
            (Mock(name="agent1"), RiskAssessment("agent1", RiskLevel.MEDIUM, 0.5, [], [], 0.8, datetime.now())),
            (Mock(name="agent2"), RiskAssessment("agent2", RiskLevel.HIGH, 0.7, [], [], 0.8, datetime.now())),
            (Mock(name="agent3"), RiskAssessment("agent3", RiskLevel.LOW, 0.3, [], [], 0.8, datetime.now())),
        ]
        
        disagreement = consensus_agent._calculate_disagreement(assessments)
        
        assert 0 <= disagreement <= 1.0
        assert disagreement > 0  # Should have some disagreement


@pytest.mark.skipif(not ANALYSIS_AVAILABLE, reason="Components not available")
class TestEndToEndFlow:
    """Test end-to-end analysis flow"""
    
    @pytest.mark.asyncio
    async def test_complete_analysis_pipeline(self):
        """Test complete pipeline from analysis to portfolio decision"""
        # Mock all external dependencies
        with patch('yfinance.Ticker') as mock_ticker, \
             patch('traderx_ai_agents.analysis.base.ChatOpenAI') as mock_llm_class:
            
            # Setup mocks
            mock_ticker.return_value.info = {
                'longName': 'Test Corp',
                'sector': 'Tech',
                'trailingPE': 20.0,
                'returnOnEquity': 0.15
            }
            
            mock_llm = Mock()
            mock_llm.ainvoke = AsyncMock(return_value=Mock(content="Analysis complete"))
            mock_llm.model_name = "gpt-4"
            mock_llm_class.return_value = mock_llm
            
            # Create components
            fundamentals = FundamentalsAnalyst()
            market = MarketAnalyst()
            news = NewsAnalyst()
            social = SocialMediaAnalyst()
            
            research_team = ResearchTeam()
            portfolio_manager = PortfolioManager()
            
            # Step 1: Run analysis agents
            symbol = "TEST"
            analysis_data = {}
            
            # Mock market data for other agents
            with patch('traderx_ai_agents.analysis.market.yfinance.Ticker', mock_ticker), \
                 patch('talib.SMA', return_value=[100, 105]), \
                 patch('talib.RSI', return_value=[50]):
                
                agents = [fundamentals, market, news, social]
                for agent in agents:
                    result = await agent.analyze({"symbol": symbol})
                    analysis_data[agent.name] = result.analysis
                    
            # Step 2: Run research team debate
            research_result = await research_team.conduct_research(symbol, analysis_data)
            
            # Step 3: Portfolio decision
            risk_data = {"volatility": 0.2, "liquidity_score": 0.8}
            decision = await portfolio_manager.make_portfolio_decision(
                symbol=symbol,
                research_data=research_result["research_team"],
                risk_data=risk_data,
                current_price=100.0
            )
            
            # Verify end-to-end flow
            assert decision.symbol == symbol
            assert decision.recommendation in [r.value for r in Recommendation]
            assert 0 <= decision.position_size <= 0.10
            assert decision.rationale is not None
            assert len(decision.key_risks) > 0
            assert isinstance(decision.timestamp, datetime)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

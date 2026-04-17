"""
Researcher agents for bull/bear debate system.
"""

from typing import Dict, Any, List, Optional
from datetime import datetime
import asyncio

from .base import BaseAnalysisAgent, ExecutionMode


class BullResearcher(BaseAnalysisAgent):
    """
    Bullish researcher focused on identifying positive catalysts and growth opportunities.
    
    Analyzes:
    - Growth drivers and catalysts
    - Competitive advantages
    - Market expansion opportunities
    - Positive industry trends
    - Undervaluation indicators
    - Management strength
    """
    
    def __init__(
        self,
        llm_provider: str = "openai",
        model_name: str = "gpt-4-turbo-preview",
        execution_mode: ExecutionMode = ExecutionMode.HYBRID
    ):
        super().__init__(
            name="bull_researcher",
            llm_provider=llm_provider,
            model_name=model_name,
            execution_mode=execution_mode
        )
        
    def _build_prompt(self, input_data: Dict[str, Any]) -> str:
        """Build bull research prompt"""
        symbol = input_data.get("symbol", "")
        company_name = input_data.get("company_name", "")
        analysis_data = input_data.get("analysis_data", {})
        
        fundamentals = analysis_data.get("fundamentals", "")
        market_data = analysis_data.get("market", "")
        news_data = analysis_data.get("news", "")
        social_data = analysis_data.get("social", "")
        
        prompt = f"""As a BULLISH RESEARCHER, identify compelling reasons to be optimistic about {symbol or company_name}.

ANALYSIS INPUTS:
Fundamentals: {fundamentals}
Market Analysis: {market_data}
News Sentiment: {news_data}
Social Media: {social_data}

BULLISH RESEARCH FRAMEWORK:
1. Growth Catalysts
   - Revenue/earnings growth drivers
   - New product/service launches
   - Market expansion opportunities
   - M&A synergies
   - Innovation pipeline

2. Competitive Advantages
   - Market leadership position
   - Brand strength and loyalty
   - Technology/IP advantages
   - Network effects
   - Cost leadership

3. Industry Tailwinds
   - Favorable regulatory environment
   - Industry growth trends
   - Demographic shifts
   - Technological adoption
   - Macro-economic factors

4. Financial Strength
   - Strong balance sheet
   - Cash flow generation
   - ROIC above cost of capital
   - Dividend growth potential
   - Share buyback programs

5. Valuation Opportunity
   - Discount to intrinsic value
   - Low P/E relative to growth
   - Below historical averages
   - Analyst price targets
   - Peer comparison advantages

Provide a compelling BULLISH investment thesis with:
- Top 3 bullish catalysts
- Investment timeline (short/medium/long-term)
- Price target and upside potential
- Key risks to monitor
- Confidence level (0-100%)
- Strong supporting evidence

Focus on the most compelling bullish arguments while acknowledging key risks."""
        
        return prompt
        
    async def generate_bull_thesis(self, symbol: str, analysis_data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate comprehensive bullish thesis"""
        input_data = {
            "symbol": symbol,
            "company_name": symbol,  # In production, map to full name
            "analysis_data": analysis_data
        }
        
        result = await self.analyze(input_data)
        
        return {
            'thesis_type': 'bullish',
            'symbol': symbol,
            'thesis': result.analysis,
            'confidence': result.confidence,
            'timestamp': result.timestamp,
            'data': result.data
        }


class BearResearcher(BaseAnalysisAgent):
    """
    Bearish researcher focused on identifying risks and downside potential.
    
    Analyzes:
    - Competitive threats
    - Regulatory risks
    - Market saturation
    - Financial weaknesses
    - Valuation concerns
    - Management issues
    """
    
    def __init__(
        self,
        llm_provider: str = "openai",
        model_name: str = "gpt-4-turbo-preview",
        execution_mode: ExecutionMode = ExecutionMode.HYBRID
    ):
        super().__init__(
            name="bear_researcher",
            llm_provider=llm_provider,
            model_name=model_name,
            execution_mode=execution_mode
        )
        
    def _build_prompt(self, input_data: Dict[str, Any]) -> str:
        """Build bear research prompt"""
        symbol = input_data.get("symbol", "")
        company_name = input_data.get("company_name", "")
        analysis_data = input_data.get("analysis_data", {})
        
        fundamentals = analysis_data.get("fundamentals", "")
        market_data = analysis_data.get("market", "")
        news_data = analysis_data.get("news", "")
        social_data = analysis_data.get("social", "")
        
        prompt = f"""As a BEARISH RESEARCHER, identify compelling risks and concerns about {symbol or company_name}.

ANALYSIS INPUTS:
Fundamentals: {fundamentals}
Market Analysis: {market_data}
News Sentiment: {news_data}
Social Media: {social_data}

BEARISH RESEARCH FRAMEWORK:
1. Competitive Threats
   - New market entrants
   - Disruptive technologies
   - Price competition
   - Market share erosion
   - Customer concentration

2. Regulatory & Legal Risks
   - Pending regulations
   - Compliance costs
   - Litigation exposure
   - Government investigations
   - Policy changes

3. Financial Weaknesses
   - High debt levels
   - Cash flow concerns
   - Margin pressure
   - Asset quality issues
   - Pension/retirement obligations

4. Operational Challenges
   - Supply chain disruptions
   - Labor issues
   - Technology obsolescence
   - Execution failures
   - Key person dependencies

5. Valuation Concerns
   - Overvaluation metrics
   - Growth expectations too high
   - Bubble characteristics
   - Peak cycle indicators
   - Historical correction patterns

Provide a compelling BEARISH investment thesis with:
- Top 3 bearish risks
- Downside potential estimate
- Timeline for risks to materialize
- Mitigating factors (if any)
- Confidence level (0-100%)
- Strong supporting evidence

Focus on the most serious risks while acknowledging any mitigating factors."""
        
        return prompt
        
    async def generate_bear_thesis(self, symbol: str, analysis_data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate comprehensive bearish thesis"""
        input_data = {
            "symbol": symbol,
            "company_name": symbol,
            "analysis_data": analysis_data
        }
        
        result = await self.analyze(input_data)
        
        return {
            'thesis_type': 'bearish',
            'symbol': symbol,
            'thesis': result.analysis,
            'confidence': result.confidence,
            'timestamp': result.timestamp,
            'data': result.data
        }


class DebateModerator(BaseAnalysisAgent):
    """
    Moderates and synthesizes bull/bear debate to reach balanced conclusion.
    
    Facilitates:
    - Structured debate process
    - Evidence evaluation
    - Risk/reward assessment
    - Consensus building
    - Final recommendation
    """
    
    def __init__(
        self,
        llm_provider: str = "openai",
        model_name: str = "gpt-4-turbo-preview",
        execution_mode: ExecutionMode = ExecutionMode.HYBRID
    ):
        super().__init__(
            name="debate_moderator",
            llm_provider=llm_provider,
            model_name=model_name,
            execution_mode=execution_mode
        )
        
    def _build_prompt(self, input_data: Dict[str, Any]) -> str:
        """Build debate moderation prompt"""
        symbol = input_data.get("symbol", "")
        bull_thesis = input_data.get("bull_thesis", "")
        bear_thesis = input_data.get("bear_thesis", "")
        
        prompt = f"""As a DEBATE MODERATOR, synthesize the bull and bear arguments for {symbol} to reach a balanced investment conclusion.

BULLISH THESIS:
{bull_thesis}

BEARISH THESIS:
{bear_thesis}

MODERATION FRAMEWORK:
1. Evidence Evaluation
   - Quality of supporting data
   - Logical consistency
   - Historical accuracy
   - Forward-looking validity
   - Quantifiable metrics

2. Risk/Reward Analysis
   - Upside potential magnitude
   - Downside risk assessment
   - Probability weighting
   - Time horizon considerations
   - Risk-adjusted returns

3. Consensus Building
   - Areas of agreement
   - Points of contention
   - Mitigating factors
   - Critical uncertainties
   - Information gaps

4. Investment Recommendation
   - Overall rating (Strong Buy/Buy/Hold/Sell/Strong Sell)
   - Position sizing guidance
   - Entry/exit strategies
   - Risk management parameters
   - Monitoring requirements

5. Confidence Assessment
   - Conviction level (0-100%)
   - Key assumptions
   - Potential surprises
   - Information reliability
   - Model limitations

Provide a MODERATED investment conclusion with:
- Final recommendation and rationale
- Balanced risk/reward assessment
- Key factors to monitor
- Confidence level with justification
- Actionable investment guidance
- Contrarian opportunities if any

Strive for objectivity and avoid bias toward either viewpoint."""
        
        return prompt
        
    async def moderate_debate(
        self,
        symbol: str,
        bull_thesis: Dict[str, Any],
        bear_thesis: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Moderate bull/bear debate and synthesize conclusion"""
        input_data = {
            "symbol": symbol,
            "bull_thesis": bull_thesis.get("thesis", ""),
            "bear_thesis": bear_thesis.get("thesis", "")
        }
        
        result = await self.analyze(input_data)
        
        # Calculate weighted confidence based on thesis quality
        bull_confidence = bull_thesis.get("confidence", 0.5)
        bear_confidence = bear_thesis.get("confidence", 0.5)
        
        # Extract recommendation from analysis
        recommendation = "HOLD"  # Default
        if "BUY" in result.analysis.upper():
            recommendation = "BUY"
        elif "SELL" in result.analysis.upper():
            recommendation = "SELL"
            
        return {
            'symbol': symbol,
            'recommendation': recommendation,
            'moderated_thesis': result.analysis,
            'confidence': result.confidence,
            'bull_confidence': bull_confidence,
            'bear_confidence': bear_confidence,
            'timestamp': result.timestamp,
            'data': result.data
        }


class ResearchTeam:
    """
    Coordinates the research team (bull, bear, moderator) for comprehensive analysis.
    """
    
    def __init__(
        self,
        llm_provider: str = "openai",
        model_name: str = "gpt-4-turbo-preview",
        execution_mode: ExecutionMode = ExecutionMode.HYBRID
    ):
        self.bull_researcher = BullResearcher(llm_provider, model_name, execution_mode)
        self.bear_researcher = BearResearcher(llm_provider, model_name, execution_mode)
        self.moderator = DebateModerator(llm_provider, model_name, execution_mode)
        
    async def conduct_research(
        self,
        symbol: str,
        analysis_data: Dict[str, Any]
    ) -> Dict[str, Any]:
        """
        Conduct full research process: bull thesis → bear thesis → moderated conclusion
        
        Returns:
            Complete research package with all perspectives
        """
        # Generate bull thesis
        bull_thesis = await self.bull_researcher.generate_bull_thesis(symbol, analysis_data)
        
        # Generate bear thesis
        bear_thesis = await self.bear_researcher.generate_bear_thesis(symbol, analysis_data)
        
        # Moderate debate and synthesize conclusion
        moderated_result = await self.moderator.moderate_debate(symbol, bull_thesis, bear_thesis)
        
        return {
            'symbol': symbol,
            'research_team': {
                'bull_thesis': bull_thesis,
                'bear_thesis': bear_thesis,
                'moderated_conclusion': moderated_result
            },
            'timestamp': datetime.now()
        }

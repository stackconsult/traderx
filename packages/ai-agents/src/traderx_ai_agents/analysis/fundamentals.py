"""
Fundamentals analysis agent for evaluating company financial metrics.
"""

from typing import Dict, Any, Optional
import yfinance as yf
import pandas as pd
from datetime import datetime, timedelta

from .base import BaseAnalysisAgent, ExecutionMode


class FundamentalsAnalyst(BaseAnalysisAgent):
    """
    Analyzes company fundamentals including:
    - Financial statements (income, balance sheet, cash flow)
    - Valuation metrics (P/E, P/B, EV/EBITDA)
    - Profitability ratios (ROE, ROA, margins)
    - Growth metrics (revenue, earnings growth)
    - Financial health (debt ratios, cash position)
    """
    
    def __init__(
        self,
        llm_provider: str = "openai",
        model_name: str = "gpt-4-turbo-preview",
        execution_mode: ExecutionMode = ExecutionMode.HYBRID
    ):
        super().__init__(
            name="fundamentals_analyst",
            llm_provider=llm_provider,
            model_name=model_name,
            execution_mode=execution_mode
        )
        
    def _build_prompt(self, input_data: Dict[str, Any]) -> str:
        """Build fundamentals analysis prompt"""
        symbol = input_data.get("symbol", "")
        if not symbol:
            return "Error: No symbol provided for fundamentals analysis"
            
        # Get financial data
        financial_data = self._get_financial_data(symbol)
        
        prompt = f"""As a Fundamentals Analyst, analyze {symbol}'s financial health and valuation.

FINANCIAL DATA:
{financial_data}

ANALYSIS FRAMEWORK:
1. Profitability Analysis
   - Revenue growth trend (3-year)
   - Gross, operating, and net margins
   - Return on Equity (ROE) and Return on Assets (ROA)
   - Earnings quality and consistency

2. Valuation Assessment
   - P/E ratio vs industry and historical
   - P/B ratio and tangible book value
   - EV/EBITDA for enterprise value
   - Price/Sales and PEG ratio
   - Discounted cash flow considerations

3. Financial Health
   - Debt-to-equity and interest coverage
   - Current ratio and working capital
   - Cash flow generation
   - Capital expenditure needs

4. Competitive Position
   - Market share and moat
   - Product diversification
   - Geographic exposure
   - R&D investment

Provide a comprehensive analysis with:
- Bullish factors (strengths)
- Bearish factors (weaknesses)
- Overall assessment (Buy/Hold/Sell)
- Confidence level (0-100%)
- Key risks to monitor

Format your response with clear sections and specific metrics."""
        
        return prompt
        
    def _get_financial_data(self, symbol: str) -> str:
        """Retrieve and format financial data"""
        try:
            # Get ticker data
            ticker = yf.Ticker(symbol.upper())
            
            # Get info
            info = ticker.info
            
            # Get financial statements
            financials = ticker.financials
            balance_sheet = ticker.balance_sheet
            cash_flow = ticker.cash_flow
            
            # Format key metrics
            data = []
            
            # Basic info
            data.append(f"Company: {info.get('longName', 'N/A')}")
            data.append(f"Sector: {info.get('sector', 'N/A')}")
            data.append(f"Market Cap: ${info.get('marketCap', 0):,.0f}")
            
            # Valuation metrics
            pe_ratio = info.get('trailingPE', 'N/A')
            pb_ratio = info.get('priceToBook', 'N/A')
            ps_ratio = info.get('priceToSalesTrailing12Months', 'N/A')
            
            data.append(f"\nValuation Metrics:")
            data.append(f"  P/E Ratio: {pe_ratio}")
            data.append(f"  P/B Ratio: {pb_ratio}")
            data.append(f"  P/S Ratio: {ps_ratio}")
            
            # Profitability metrics
            roe = info.get('returnOnEquity', 'N/A')
            roa = info.get('returnOnAssets', 'N/A')
            profit_margin = info.get('profitMargins', 'N/A')
            operating_margin = info.get('operatingMargins', 'N/A')
            
            data.append(f"\nProfitability Metrics:")
            data.append(f"  ROE: {roe:.2%}" if isinstance(roe, (int, float)) else f"  ROE: {roe}")
            data.append(f"  ROA: {roa:.2%}" if isinstance(roa, (int, float)) else f"  ROA: {roa}")
            data.append(f"  Profit Margin: {profit_margin:.2%}" if isinstance(profit_margin, (int, float)) else f"  Profit Margin: {profit_margin}")
            data.append(f"  Operating Margin: {operating_margin:.2%}" if isinstance(operating_margin, (int, float)) else f"  Operating Margin: {operating_margin}")
            
            # Financial health
            debt_to_equity = info.get('debtToEquity', 'N/A')
            current_ratio = info.get('currentRatio', 'N/A')
            
            data.append(f"\nFinancial Health:")
            data.append(f"  Debt-to-Equity: {debt_to_equity}")
            data.append(f"  Current Ratio: {current_ratio}")
            
            # Growth metrics
            revenue_growth = info.get('revenueGrowth', 'N/A')
            earnings_growth = info.get('earningsGrowth', 'N/A')
            
            data.append(f"\nGrowth Metrics:")
            data.append(f"  Revenue Growth: {revenue_growth:.2%}" if isinstance(revenue_growth, (int, float)) else f"  Revenue Growth: {revenue_growth}")
            data.append(f"  Earnings Growth: {earnings_growth:.2%}" if isinstance(earnings_growth, (int, float)) else f"  Earnings Growth: {earnings_growth}")
            
            # Recent financial highlights
            if not financials.empty:
                data.append(f"\nRecent Financial Highlights:")
                latest_revenue = financials.iloc[0].get('Total Revenue', 0)
                latest_net_income = financials.iloc[0].get('Net Income', 0)
                
                data.append(f"  Latest Revenue: ${latest_revenue:,.0f}" if isinstance(latest_revenue, (int, float)) else f"  Latest Revenue: {latest_revenue}")
                data.append(f"  Latest Net Income: ${latest_net_income:,.0f}" if isinstance(latest_net_income, (int, float)) else f"  Latest Net Income: {latest_net_income}")
                
            return "\n".join(data)
            
        except Exception as e:
            return f"Error retrieving financial data: {str(e)}"
            
    async def analyze_financial_health_score(self, symbol: str) -> Dict[str, Any]:
        """
        Calculate a comprehensive financial health score (0-100)
        
        Returns:
            Dictionary with health score and component scores
        """
        try:
            ticker = yf.Ticker(symbol.upper())
            info = ticker.info
            
            scores = {}
            
            # Profitability score (0-25)
            roe = info.get('returnOnEquity', 0)
            roa = info.get('returnOnAssets', 0)
            profit_margin = info.get('profitMargins', 0)
            
            profitability_score = 0
            if isinstance(roe, (int, float)) and roe > 0.15:  # 15% ROE threshold
                profitability_score += 10
            if isinstance(roa, (int, float)) and roa > 0.05:  # 5% ROA threshold
                profitability_score += 10
            if isinstance(profit_margin, (int, float)) and profit_margin > 0.10:  # 10% margin threshold
                profitability_score += 5
                
            scores['profitability'] = min(25, profitability_score)
            
            # Valuation score (0-25)
            pe_ratio = info.get('trailingPE', 100)
            pb_ratio = info.get('priceToBook', 10)
            
            valuation_score = 0
            if isinstance(pe_ratio, (int, float)):
                if 0 < pe_ratio < 15:
                    valuation_score += 15
                elif 15 <= pe_ratio < 25:
                    valuation_score += 10
                elif 25 <= pe_ratio < 35:
                    valuation_score += 5
                    
            if isinstance(pb_ratio, (int, float)):
                if 0 < pb_ratio < 1.5:
                    valuation_score += 10
                elif 1.5 <= pb_ratio < 3:
                    valuation_score += 5
                    
            scores['valuation'] = min(25, valuation_score)
            
            # Financial health score (0-25)
            debt_to_equity = info.get('debtToEquity', 2)
            current_ratio = info.get('currentRatio', 0.5)
            
            health_score = 0
            if isinstance(debt_to_equity, (int, float)):
                if debt_to_equity < 0.5:
                    health_score += 15
                elif debt_to_equity < 1.0:
                    health_score += 10
                elif debt_to_equity < 1.5:
                    health_score += 5
                    
            if isinstance(current_ratio, (int, float)):
                if current_ratio > 2.0:
                    health_score += 10
                elif current_ratio > 1.5:
                    health_score += 5
                    
            scores['financial_health'] = min(25, health_score)
            
            # Growth score (0-25)
            revenue_growth = info.get('revenueGrowth', 0)
            earnings_growth = info.get('earningsGrowth', 0)
            
            growth_score = 0
            if isinstance(revenue_growth, (int, float)):
                if revenue_growth > 0.20:  # 20% growth
                    growth_score += 15
                elif revenue_growth > 0.10:  # 10% growth
                    growth_score += 10
                elif revenue_growth > 0.05:  # 5% growth
                    growth_score += 5
                    
            if isinstance(earnings_growth, (int, float)):
                if earnings_growth > 0.20:
                    growth_score += 10
                elif earnings_growth > 0.10:
                    growth_score += 5
                    
            scores['growth'] = min(25, growth_score)
            
            # Calculate total score
            total_score = sum(scores.values())
            
            return {
                'symbol': symbol,
                'total_score': total_score,
                'component_scores': scores,
                'rating': self._get_health_rating(total_score),
                'timestamp': datetime.now()
            }
            
        except Exception as e:
            return {
                'symbol': symbol,
                'total_score': 0,
                'error': str(e),
                'timestamp': datetime.now()
            }
            
    def _get_health_rating(self, score: float) -> str:
        """Convert health score to rating"""
        if score >= 80:
            return "Excellent"
        elif score >= 65:
            return "Good"
        elif score >= 50:
            return "Average"
        elif score >= 35:
            return "Below Average"
        else:
            return "Poor"

"""
Integration bridge between Sentinel-Nexus AI agents and TraderX execution system.

This module provides the interface for the analysis layer to communicate
with the existing TraderX execution agents.
"""

import asyncio
import json
import logging
from typing import Dict, Any, List, Optional, Callable
from dataclasses import dataclass, asdict
from datetime import datetime
import redis.asyncio as redis

from ..analysis.base import AnalysisResult
from ..risk.portfolio_manager import PortfolioDecision
from ..enhanced_agent_hierarchy import AgentSignal, SignalRouter

logger = logging.getLogger(__name__)


@dataclass
class TraderXSignal:
    """Standardized signal format for TraderX execution agents"""
    signal_type: str
    symbol: str
    data: Dict[str, Any]
    timestamp: datetime
    source: str
    priority: int = 2  # 1=highest, 5=lowest
    
    def to_json(self) -> str:
        """Convert to JSON for transmission"""
        data = asdict(self)
        data['timestamp'] = self.timestamp.isoformat()
        return json.dumps(data)
        
    @classmethod
    def from_json(cls, json_str: str) -> 'TraderXSignal':
        """Create from JSON"""
        data = json.loads(json_str)
        data['timestamp'] = datetime.fromisoformat(data['timestamp'])
        return cls(**data)


class TraderXBridge:
    """
    Bridge component for integrating Sentinel-Nexus agents with TraderX execution.
    
    Features:
    - Redis-based signal publishing
    - Signal transformation and routing
    - Execution agent interface
    - Response handling
    - Monitoring and metrics
    """
    
    def __init__(
        self,
        redis_url: str = "redis://localhost:6379",
        signal_channel: str = "traderx_signals",
        response_channel: str = "traderx_responses",
        oms_endpoint: str = "http://localhost:8080",
        enable_monitoring: bool = True
    ):
        self.redis_url = redis_url
        self.signal_channel = signal_channel
        self.response_channel = response_channel
        self.oms_endpoint = oms_endpoint
        self.enable_monitoring = enable_monitoring
        
        # Redis connection
        self.redis_client: Optional[redis.Redis] = None
        
        # Signal handlers
        self.signal_handlers: Dict[str, Callable] = {}
        
        # Metrics
        self.metrics = {
            'signals_sent': 0,
            'signals_received': 0,
            'errors': 0,
            'last_signal_time': None
        }
        
    async def initialize(self):
        """Initialize the bridge connection with reconnection handling"""
        max_retries = 5
        base_delay = 1.0
        
        for attempt in range(max_retries):
            try:
                self.redis_client = redis.from_url(
                    self.redis_url,
                    retry_on_timeout=True,
                    retry_on_error=[redis.ConnectionError, redis.TimeoutError],
                    socket_keepalive=True,
                    socket_keepalive_options={}
                )
                await self.redis_client.ping()
                logger.info("TraderX bridge initialized successfully")
                
                # Subscribe to responses
                asyncio.create_task(self._listen_for_responses())
                
                # Start health check task
                asyncio.create_task(self._health_check())
                
                return
                
            except Exception as e:
                if attempt == max_retries - 1:
                    logger.error(f"Failed to initialize TraderX bridge after {max_retries} attempts: {e}")
                    raise
                    
                delay = base_delay * (2 ** attempt)
                logger.warning(f"Connection attempt {attempt + 1} failed, retrying in {delay}s: {e}")
                await asyncio.sleep(delay)
            
    async def cleanup(self):
        """Cleanup bridge resources"""
        if self.redis_client:
            await self.redis_client.close()
            
    def register_signal_handler(self, signal_type: str, handler: Callable):
        """Register a handler for specific signal types"""
        self.signal_handlers[signal_type] = handler
        logger.info(f"Registered handler for signal type: {signal_type}")
        
    async def publish_analysis_result(self, result: AnalysisResult):
        """
        Publish analysis result to TraderX execution layer.
        
        Args:
            result: Analysis result from Sentinel-Nexus agents
        """
        try:
            # Transform analysis result to TraderX signal
            signal = self._transform_analysis_result(result)
            
            # Publish to Redis
            await self.redis_client.publish(
                self.signal_channel,
                signal.to_json()
            )
            
            self.metrics['signals_sent'] += 1
            self.metrics['last_signal_time'] = datetime.now()
            
            logger.info(f"Published analysis signal for {result.agent_name}")
            
        except Exception as e:
            self.metrics['errors'] += 1
            logger.error(f"Failed to publish analysis result: {e}")
            
    async def publish_portfolio_decision(self, decision: PortfolioDecision):
        """
        Publish portfolio decision to TraderX execution layer.
        
        Args:
            decision: Portfolio decision from risk layer
        """
        try:
            # Transform portfolio decision to TraderX signal
            signal = self._transform_portfolio_decision(decision)
            
            # Publish to Redis
            await self.redis_client.publish(
                self.signal_channel,
                signal.to_json()
            )
            
            self.metrics['signals_sent'] += 1
            self.metrics['last_signal_time'] = datetime.now()
            
            logger.info(f"Published portfolio decision for {decision.symbol}: {decision.recommendation}")
            
        except Exception as e:
            self.metrics['errors'] += 1
            logger.error(f"Failed to publish portfolio decision: {e}")
            
    async def send_execution_request(
        self,
        order_request: Dict[str, Any],
        timeout: float = 5.0
    ) -> Optional[Dict[str, Any]]:
        """
        Send execution request to TraderX OMS.
        
        Args:
            order_request: Order request details
            timeout: Request timeout in seconds
            
        Returns:
            Execution response or None if timeout/error
        """
        try:
            # This would integrate with actual TraderX OMS API
            # For now, simulate the response
            
            logger.info(f"Sending execution request: {order_request}")
            
            # Simulate async execution
            await asyncio.sleep(0.1)
            
            # Mock response
            response = {
                'order_id': 'mock_order_123',
                'status': 'submitted',
                'timestamp': datetime.now().isoformat(),
                'symbol': order_request.get('symbol'),
                'quantity': order_request.get('quantity'),
                'price': order_request.get('price')
            }
            
            return response
            
        except Exception as e:
            self.metrics['errors'] += 1
            logger.error(f"Failed to send execution request: {e}")
            return None
            
    def _transform_analysis_result(self, result: AnalysisResult) -> TraderXSignal:
        """Transform analysis result to TraderX signal format"""
        return TraderXSignal(
            signal_type="analysis_update",
            symbol=result.data.get('symbol', 'UNKNOWN'),
            data={
                'agent_name': result.agent_name,
                'analysis': result.analysis,
                'confidence': result.confidence,
                'execution_time_ms': result.execution_time_ms,
                'model_used': result.model_used,
                'raw_data': result.data
            },
            timestamp=result.timestamp,
            source="sentinel_nexus",
            priority=3
        )
        
    def _transform_portfolio_decision(self, decision: PortfolioDecision) -> TraderXSignal:
        """Transform portfolio decision to TraderX signal format"""
        return TraderXSignal(
            signal_type="portfolio_decision",
            symbol=decision.symbol,
            data={
                'recommendation': decision.recommendation.value,
                'position_size': decision.position_size,
                'entry_price': decision.entry_price,
                'target_price': decision.target_price,
                'stop_loss': decision.stop_loss,
                'time_horizon': decision.time_horizon,
                'conviction': decision.conviction,
                'rationale': decision.rationale,
                'key_risks': decision.key_risks,
                'catalysts': decision.catalysts
            },
            timestamp=decision.timestamp,
            source="portfolio_manager",
            priority=1  # High priority for portfolio decisions
        )
        
    async def _listen_for_responses(self):
        """Listen for responses from TraderX execution layer"""
        try:
            pubsub = self.redis_client.pubsub()
            await pubsub.subscribe(self.response_channel)
            
            async for message in pubsub.listen():
                if message['type'] == 'message':
                    await self._handle_response(message['data'])
                    
        except Exception as e:
            logger.error(f"Error listening for responses: {e}")
            
    async def _handle_response(self, response_data: bytes):
        """Handle response from TraderX execution layer"""
        try:
            response = json.loads(response_data)
            signal_type = response.get('signal_type')
            
            self.metrics['signals_received'] += 1
            
            # Route to registered handler
            if signal_type in self.signal_handlers:
                await self.signal_handlers[signal_type](response)
            else:
                logger.warning(f"No handler for signal type: {signal_type}")
                
        except Exception as e:
            self.metrics['errors'] += 1
            logger.error(f"Error handling response: {e}")
            
    async def _health_check(self):
        """Periodic health check for Redis connection"""
        while True:
            try:
                await asyncio.sleep(30)  # Check every 30 seconds
                
                if not self.redis_client:
                    logger.warning("Redis client not initialized, attempting reconnection...")
                    await self.initialize()
                    continue
                    
                await self.redis_client.ping()
                
            except Exception as e:
                logger.error(f"Health check failed, attempting reconnection: {e}")
                try:
                    await self.cleanup()
                except:
                    pass
                await self.initialize()
            
    async def get_metrics(self) -> Dict[str, Any]:
        """Get bridge performance metrics"""
        return {
            **self.metrics,
            'uptime_seconds': (datetime.now() - (self.metrics.get('start_time', datetime.now()))).total_seconds() if self.metrics.get('start_time') else 0,
            'error_rate': self.metrics['errors'] / max(self.metrics['signals_sent'], 1) * 100
        }


class ExecutionAgentInterface:
    """
    Interface for TraderX execution agents to receive Sentinel-Nexus signals.
    """
    
    def __init__(self, bridge: TraderXBridge):
        self.bridge = bridge
        
    async def start(self):
        """Start the execution agent interface"""
        # Register handlers for different signal types
        self.bridge.register_signal_handler(
            "analysis_update",
            self._handle_analysis_update
        )
        
        self.bridge.register_signal_handler(
            "portfolio_decision",
            self._handle_portfolio_decision
        )
        
        self.bridge.register_signal_handler(
            "risk_alert",
            self._handle_risk_alert
        )
        
        logger.info("Execution agent interface started")
        
    async def _handle_analysis_update(self, signal_data: Dict[str, Any]):
        """Handle analysis update from Sentinel-Nexus"""
        symbol = signal_data.get('symbol')
        agent_name = signal_data.get('agent_name')
        confidence = signal_data.get('confidence')
        
        logger.info(f"Received analysis update for {symbol} from {agent_name} (confidence: {confidence})")
        
        # Route to appropriate execution agent
        if agent_name == "market_analyst":
            await self._route_to_market_data_agent(signal_data)
        elif agent_name == "fundamentals_analyst":
            await self._route_to_strategy_agent(signal_data)
            
    async def _handle_portfolio_decision(self, signal_data: Dict[str, Any]):
        """Handle portfolio decision from risk layer"""
        symbol = signal_data.get('symbol')
        recommendation = signal_data.get('recommendation')
        position_size = signal_data.get('position_size')
        
        logger.info(f"Received portfolio decision for {symbol}: {recommendation} (size: {position_size})")
        
        # Convert to order if needed
        if recommendation in ['buy', 'strong_buy']:
            await self._create_buy_order(signal_data)
        elif recommendation in ['sell', 'strong_sell']:
            await self._create_sell_order(signal_data)
            
    async def _handle_risk_alert(self, signal_data: Dict[str, Any]):
        """Handle risk alert from risk layer"""
        risk_level = signal_data.get('risk_level')
        message = signal_data.get('message')
        
        logger.warning(f"Risk alert - Level: {risk_level}, Message: {message}")
        
        # Take appropriate action based on risk level
        if risk_level == 'critical':
            await self._emergency_stop(signal_data)
        elif risk_level == 'high':
            await self._reduce_positions(signal_data)
            
    async def _route_to_market_data_agent(self, signal_data: Dict[str, Any]):
        """Route analysis to market data agent"""
        # Implementation would integrate with actual market_data_interpreter
        pass
        
    async def _route_to_strategy_agent(self, signal_data: Dict[str, Any]):
        """Route analysis to strategy agent"""
        # Implementation would integrate with actual strategy agents
        pass
        
    async def _create_buy_order(self, signal_data: Dict[str, Any]):
        """Create buy order based on portfolio decision"""
        symbol = signal_data.get('symbol')
        position_size = signal_data.get('position_size')
        entry_price = signal_data.get('entry_price')
        
        order_request = {
            'symbol': symbol,
            'side': 'buy',
            'quantity': self._calculate_quantity(position_size, entry_price),
            'order_type': 'limit',
            'price': entry_price,
            'time_in_force': 'day'
        }
        
        response = await self.bridge.send_execution_request(order_request)
        if response:
            logger.info(f"Buy order submitted: {response['order_id']}")
        else:
            logger.error("Failed to submit buy order")
            
    async def _create_sell_order(self, signal_data: Dict[str, Any]):
        """Create sell order based on portfolio decision"""
        symbol = signal_data.get('symbol')
        position_size = signal_data.get('position_size')
        entry_price = signal_data.get('entry_price')
        
        order_request = {
            'symbol': symbol,
            'side': 'sell',
            'quantity': self._calculate_quantity(position_size, entry_price),
            'order_type': 'limit',
            'price': entry_price,
            'time_in_force': 'day'
        }
        
        response = await self.bridge.send_execution_request(order_request)
        if response:
            logger.info(f"Sell order submitted: {response['order_id']}")
        else:
            logger.error("Failed to submit sell order")
            
    async def _emergency_stop(self, signal_data: Dict[str, Any]):
        """Emergency stop all trading"""
        logger.critical("Emergency stop triggered - halting all trading")
        # Implementation would call kill_switch_agent
        
    async def _reduce_positions(self, signal_data: Dict[str, Any]):
        """Reduce position sizes due to high risk"""
        logger.warning("Reducing positions due to high risk")
        # Implementation would reduce existing positions
        
    def _calculate_quantity(self, position_size: float, price: float) -> int:
        """Calculate order quantity based on position size and price"""
        # Simplified calculation - would use portfolio value in production
        portfolio_value = 1000000  # $1M portfolio
        position_value = portfolio_value * position_size
        return int(position_value / price)

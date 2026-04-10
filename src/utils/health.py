import asyncio
import logging
from typing import Dict, Any
from datetime import datetime, timedelta
from dataclasses import dataclass


@dataclass
class HealthStatus:
    healthy: bool
    timestamp: datetime
    components: Dict[str, Dict[str, Any]]
    uptime: timedelta


class HealthChecker:
    """Health monitoring system for TraderX."""
    
    def __init__(self):
        self.logger = logging.getLogger(__name__)
        self.start_time = datetime.utcnow()
        self.last_check = None
        self.component_status = {}
    
    async def check_health(self, engine) -> HealthStatus:
        """Check the health of all system components."""
        self.last_check = datetime.utcnow()
        uptime = self.last_check - self.start_time
        
        components = {
            'engine': await self._check_engine(engine),
            'exchanges': await self._check_exchanges(engine),
            'strategies': await self._check_strategies(engine),
            'risk_manager': await self._check_risk_manager(engine),
            'data_storage': await self._check_data_storage(engine)
        }
        
        # Overall health is healthy if all critical components are healthy
        healthy = all(
            comp.get('healthy', False) 
            for comp in components.values() 
            if comp.get('critical', True)
        )
        
        return HealthStatus(
            healthy=healthy,
            timestamp=self.last_check,
            components=components,
            uptime=uptime
        )
    
    async def _check_engine(self, engine) -> Dict[str, Any]:
        """Check trading engine health."""
        try:
            return {
                'healthy': engine.running if engine else False,
                'critical': True,
                'message': 'Running' if engine and engine.running else 'Stopped',
                'orders_count': len(engine.orders) if engine else 0,
                'positions_count': len(engine.positions) if engine else 0
            }
        except Exception as e:
            return {
                'healthy': False,
                'critical': True,
                'message': f'Error: {str(e)}'
            }
    
    async def _check_exchanges(self, engine) -> Dict[str, Any]:
        """Check exchange connections."""
        try:
            if not engine:
                return {'healthy': False, 'critical': True, 'message': 'Engine not available'}
            
            exchange_status = {}
            all_healthy = True
            
            for name, exchange in engine.exchanges.items():
                is_connected = exchange.connected
                exchange_status[name] = {
                    'connected': is_connected,
                    'last_check': datetime.utcnow().isoformat()
                }
                if not is_connected:
                    all_healthy = False
            
            return {
                'healthy': all_healthy,
                'critical': True,
                'exchanges': exchange_status,
                'message': 'All connected' if all_healthy else 'Some exchanges disconnected'
            }
        except Exception as e:
            return {
                'healthy': False,
                'critical': True,
                'message': f'Error: {str(e)}'
            }
    
    async def _check_strategies(self, engine) -> Dict[str, Any]:
        """Check strategy health."""
        try:
            if not engine:
                return {'healthy': False, 'critical': False, 'message': 'Engine not available'}
            
            strategy_status = {}
            all_active = True
            
            for strategy in engine.strategies:
                strategy_status[strategy.name] = {
                    'active': strategy.active,
                    'symbols': strategy.symbols,
                    'trades_count': strategy.trades_count
                }
                if not strategy.active:
                    all_active = False
            
            return {
                'healthy': all_active,
                'critical': False,
                'strategies': strategy_status,
                'message': 'All active' if all_active else 'Some strategies inactive'
            }
        except Exception as e:
            return {
                'healthy': False,
                'critical': False,
                'message': f'Error: {str(e)}'
            }
    
    async def _check_risk_manager(self, engine) -> Dict[str, Any]:
        """Check risk manager status."""
        try:
            if not engine or not engine.risk_manager:
                return {'healthy': False, 'critical': True, 'message': 'Risk manager not available'}
            
            risk_manager = engine.risk_manager
            
            return {
                'healthy': not risk_manager.circuit_breaker_active,
                'critical': True,
                'circuit_breaker_active': risk_manager.circuit_breaker_active,
                'circuit_breaker_reason': risk_manager.circuit_breaker_reason,
                'daily_pnl': risk_manager.daily_pnl,
                'total_exposure': risk_manager.total_exposure,
                'message': 'Normal' if not risk_manager.circuit_breaker_active else 'Circuit breaker active'
            }
        except Exception as e:
            return {
                'healthy': False,
                'critical': True,
                'message': f'Error: {str(e)}'
            }
    
    async def _check_data_storage(self, engine) -> Dict[str, Any]:
        """Check data storage connectivity."""
        try:
            if not engine or not engine.data_storage:
                return {'healthy': False, 'critical': False, 'message': 'Data storage not available'}
            
            # Try a simple operation
            storage = engine.data_storage
            
            # Check if we can get positions (this tests database connection)
            positions = await storage.get_positions()
            
            return {
                'healthy': True,
                'critical': False,
                'positions_count': len(positions),
                'message': 'Connected'
            }
        except Exception as e:
            return {
                'healthy': False,
                'critical': False,
                'message': f'Error: {str(e)}'
            }
    
    def get_health_dict(self, health_status: HealthStatus) -> Dict[str, Any]:
        """Convert health status to dictionary for API response."""
        return {
            'healthy': health_status.healthy,
            'timestamp': health_status.timestamp.isoformat(),
            'uptime_seconds': int(health_status.uptime.total_seconds()),
            'components': health_status.components
        }

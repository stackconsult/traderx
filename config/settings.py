from pydantic_settings import BaseSettings
from typing import Optional, Dict, Any
from enum import Enum


class Environment(str, Enum):
    DEVELOPMENT = "development"
    PAPER_TRADING = "paper_trading"
    LIVE = "live"


class Settings(BaseSettings):
    # Application settings
    app_name: str = "TraderX"
    environment: Environment = Environment.DEVELOPMENT
    debug: bool = False
    
    # Database settings
    database_url: str = "postgresql://user:password@localhost/traderx"
    
    # Redis settings
    redis_url: str = "redis://localhost:6379"
    
    # Exchange settings
    default_exchange: str = "binance"
    exchange_configs: Dict[str, Dict[str, Any]] = {}
    
    # Risk management
    max_position_size: float = 1000.0
    max_daily_loss: float = 100.0
    max_drawdown: float = 0.20
    
    # API settings
    api_rate_limit: int = 10  # requests per second
    
    # Logging
    log_level: str = "INFO"
    
    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"


settings = Settings()

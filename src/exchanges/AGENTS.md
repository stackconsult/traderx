# Exchange Integration Governance

## Specialized Laws for Exchange Adapters

### BANNED Patterns
- Direct REST API calls without rate limiting
- Order submission without error handling
- Market data caching without TTL
- Hardcoded API endpoints
- Synchronous operations in async context

### REQUIRED Patterns
```python
# Inherit from BaseExchange
class MyExchange(BaseExchange):
    async def connect(self):
        # Must implement connection logic
        pass
    
    async def submit_order(self, order: Order) -> bool:
        # Must handle all exceptions
        try:
            # Submit order
            return True
        except Exception as e:
            self.logger.error(f"Order failed: {e}")
            raise
```

### Rate Limiting
- Respect exchange rate limits
- Implement exponential backoff
- Queue orders during rate limit hits
- Track remaining requests

### Error Handling
- All API errors logged with context
- Network timeouts handled gracefully
- Invalid order responses rejected
- Authentication failures logged

### Data Validation
- Validate all market data timestamps
- Check for missing or corrupted data
- Normalize symbol formats
- Validate price ranges

### Security Requirements
- API keys never logged
- Use HTTPS for all requests
- Validate SSL certificates
- IP whitelisting when available

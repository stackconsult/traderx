# Pattern: Resolving Circular Imports in Python

## Problem
Circular import occurs when module A imports module B, and module B imports module A.

## Solution
Create a separate models.py file for shared dataclasses/enums.

## Implementation
```python
# src/core/models.py - Shared models
from dataclasses import dataclass
from enum import Enum

class OrderStatus(Enum):
    PENDING = "pending"
    # ... other statuses

@dataclass
class Order:
    id: str
    symbol: str
    # ... other fields

# In other modules, import from models instead
from ..core.models import Order, OrderStatus
```

## Benefits
- Eliminates circular dependencies
- Centralized model definitions
- Easier maintenance

## Performance Impact
- Negligible import overhead
- No runtime performance penalty

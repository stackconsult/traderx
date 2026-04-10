# B2B Multi-Tenancy with Row Level Security

## Description
Implements secure multi-tenant isolation using PostgreSQL Row Level Security (RLS) for B2B white-label broker partners. Ensures data isolation at the database level without application-side filtering.

## Source
- Repository: logto-io/implement-multi-tenancy
- Reference: https://blog.logto.io/implement-multi-tenancy

## Implementation Pattern

### Core Architecture
1. **Tenant Identification**: Every table includes `tenant_id` column
2. **Session Management**: Use `SET LOCAL app.current_tenant` in transactions
3. **Policy Enforcement**: PostgreSQL RLS policies automatically filter rows
4. **Migration Strategy**: Add tenant columns to existing tables with default values

### Key Components

#### Database Schema
```sql
-- Enable RLS on all tables
ALTER TABLE table_name ENABLE ROW LEVEL SECURITY;

-- Create tenant policy
CREATE POLICY tenant_isolation ON table_name
    USING (tenant_id = current_setting('app.current_tenant')::uuid);
```

#### Application Integration
```typescript
// Transaction wrapper with tenant context
async function withTenant<T>(tenantId: string, fn: () => Promise<T>): Promise<T> {
  await prisma.$executeRaw`SET LOCAL app.current_tenant = ${tenantId}`;
  try {
    return await fn();
  } finally {
    // Context automatically cleared at transaction end
  }
}
```

### Integration Points
- **Phase 2 Tables**: Add tenant_id to HSTR snapshots/deltas, ZK-Audit entries
- **API Gateway**: Extract tenant from subdomain/JWT token
- **Connection Pooling**: Use PgBouncer with transaction pooling

### Performance Considerations
- Index tenant_id columns for query optimization
- Use partitioning by tenant for large tables
- Monitor RLS policy overhead in production

### Security Features
- Automatic tenant isolation prevents data leaks
- No application-side filtering required
- Database-enforced security guarantees

### Testing Strategy
- rls-leak-test.sh verifies cross-tenant query blocking
- Test with missing/mismatched tenant session variables
- Performance benchmarks for RLS overhead

## Success Criteria
- rls-leak-test.sh fails to retrieve data without proper tenant context
- All Phase 2 tables support multi-tenant access
- Zero application-side tenant filtering code

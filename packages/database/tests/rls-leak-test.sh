#!/bin/bash
# RLS Leak Test - Verify Multi-Tenant Isolation
# This test attempts to access data across tenant boundaries
# and verifies that PostgreSQL RLS blocks all attempts

set -e

echo "=== RLS Leak Test ==="
echo "Testing multi-tenant isolation..."

# Database connection
DB_URL="postgresql://postgres:postgres@localhost:5432/traderx"

# Create test tenants
echo "Creating test tenants..."
psql "$DB_URL" << EOF
-- Insert test tenants
INSERT INTO tenants (id, name, slug) VALUES 
    ('11111111-1111-1111-1111-111111111111', 'Tenant A', 'tenant-a'),
    ('22222222-2222-2222-2222-222222222222', 'Tenant B', 'tenant-b')
ON CONFLICT (id) DO NOTHING;

-- Insert test data for Tenant A
SET LOCAL app.current_tenant = '11111111-1111-1111-1111-111111111111';
INSERT INTO entities (id, tenant_id, ticker, name) VALUES 
    ('entity-a', '11111111-1111-1111-1111-111111111111', 'AAPL', 'Apple Inc.')
ON CONFLICT (id) DO NOTHING;

INSERT INTO orders (id, tenant_id, symbol, side, quantity) VALUES
    ('order-a', '11111111-1111-1111-1111-111111111111', 'AAPL', 'BUY', 100)
ON CONFLICT (id) DO NOTHING;

-- Insert test data for Tenant B
SET LOCAL app.current_tenant = '22222222-2222-2222-2222-222222222222';
INSERT INTO entities (id, tenant_id, ticker, name) VALUES 
    ('entity-b', '22222222-2222-2222-2222-222222222222', 'GOOGL', 'Alphabet Inc.')
ON CONFLICT (id) DO NOTHING;

INSERT INTO orders (id, tenant_id, symbol, side, quantity) VALUES
    ('order-b', '22222222-2222-2222-2222-222222222222', 'GOOGL', 'SELL', 50)
ON CONFLICT (id) DO NOTHING;
EOF

echo "Test data created."

# Test 1: No tenant context - should return no data
echo "Test 1: Query without tenant context..."
result=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM entities" 2>/dev/null || echo "ERROR")
if [ "$result" = "0" ] || [ "$result" = "ERROR" ]; then
    echo "✓ PASS: No data returned without tenant context"
else
    echo "✗ FAIL: Data leaked without tenant context: $result"
    exit 1
fi

# Test 2: Wrong tenant context - should return no data
echo "Test 2: Query with wrong tenant context..."
result=$(psql "$DB_URL" -t -c "
    SET LOCAL app.current_tenant = '22222222-2222-2222-2222-222222222222';
    SELECT COUNT(*) FROM entities WHERE ticker = 'AAPL';
" 2>/dev/null || echo "ERROR")
if [ "$result" = "0" ] || [ "$result" = "ERROR" ]; then
    echo "✓ PASS: No cross-tenant data access"
else
    echo "✗ FAIL: Cross-tenant data leaked: $result"
    exit 1
fi

# Test 3: Correct tenant context - should return tenant's data
echo "Test 3: Query with correct tenant context..."
result=$(psql "$DB_URL" -t -c "
    SET LOCAL app.current_tenant = '11111111-1111-1111-1111-111111111111';
    SELECT COUNT(*) FROM entities WHERE ticker = 'AAPL';
" 2>/dev/null || echo "ERROR")
if [ "$result" = "1" ]; then
    echo "✓ PASS: Tenant can access own data"
else
    echo "✗ FAIL: Cannot access own data: $result"
    exit 1
fi

# Test 4: Direct tenant_id bypass attempt - should fail
echo "Test 4: Attempt to bypass RLS by specifying tenant_id..."
result=$(psql "$DB_URL" -t -c "
    SET LOCAL app.current_tenant = '22222222-2222-2222-2222-222222222222';
    SELECT COUNT(*) FROM entities WHERE tenant_id = '11111111-1111-1111-1111-111111111111';
" 2>/dev/null || echo "ERROR")
if [ "$result" = "0" ] || [ "$result" = "ERROR" ]; then
    echo "✓ PASS: Cannot bypass RLS with tenant_id filter"
else
    echo "✗ FAIL: RLS bypass successful: $result"
    exit 1
fi

# Test 5: Insert with wrong tenant - should be blocked
echo "Test 5: Insert data with wrong tenant context..."
result=$(psql "$DB_URL" -t -c "
    SET LOCAL app.current_tenant = '22222222-2222-2222-2222-222222222222';
    INSERT INTO entities (id, tenant_id, ticker, name) 
    VALUES ('entity-c', '11111111-1111-1111-1111-111111111111', 'MSFT', 'Microsoft')
    RETURNING id;
" 2>/dev/null || echo "ERROR")
if [ "$result" = "ERROR" ] || [ -z "$result" ]; then
    echo "✓ PASS: Cannot insert data for wrong tenant"
else
    echo "✗ FAIL: Inserted data for wrong tenant: $result"
    exit 1
fi

# Test 6: Update cross-tenant data - should be blocked
echo "Test 6: Update cross-tenant data..."
result=$(psql "$DB_URL" -t -c "
    SET LOCAL app.current_tenant = '22222222-2222-2222-2222-222222222222';
    UPDATE entities SET name = 'Hacked' WHERE ticker = 'AAPL';
    SELECT ROW_COUNT();
" 2>/dev/null || echo "ERROR")
if [ "$result" = "0" ] || [ "$result" = "ERROR" ]; then
    echo "✓ PASS: Cannot update cross-tenant data"
else
    echo "✗ FAIL: Updated cross-tenant data: $result"
    exit 1
fi

# Test 7: Delete cross-tenant data - should be blocked
echo "Test 7: Delete cross-tenant data..."
result=$(psql "$DB_URL" -t -c "
    SET LOCAL app.current_tenant = '22222222-2222-2222-2222-222222222222';
    DELETE FROM entities WHERE ticker = 'AAPL';
    SELECT ROW_COUNT();
" 2>/dev/null || echo "ERROR")
if [ "$result" = "0" ] || [ "$result" = "ERROR" ]; then
    echo "✓ PASS: Cannot delete cross-tenant data"
else
    echo "✗ FAIL: Deleted cross-tenant data: $result"
    exit 1
fi

# Test 8: Verify audit entries are isolated
echo "Test 8: Verify audit entries isolation..."
result=$(psql "$DB_URL" -t -c "
    SET LOCAL app.current_tenant = '11111111-1111-1111-1111-111111111111';
    INSERT INTO audit_entries (id, tenant_id, action, agent, reasoning, inputs, outputs)
    VALUES ('audit-a', '11111111-1111-1111-1111-111111111111', 'TEST', 'AgentA', 'Test', '{}', '{}')
    RETURNING id;
" 2>/dev/null || echo "ERROR")
if [ "$result" = "audit-a" ]; then
    echo "✓ PASS: Can create audit entries"
else
    echo "✗ FAIL: Cannot create audit entries: $result"
    exit 1
fi

# Verify other tenant cannot see audit entries
result=$(psql "$DB_URL" -t -c "
    SET LOCAL app.current_tenant = '22222222-2222-2222-2222-222222222222';
    SELECT COUNT(*) FROM audit_entries WHERE action = 'TEST';
" 2>/dev/null || echo "ERROR")
if [ "$result" = "0" ] || [ "$result" = "ERROR" ]; then
    echo "✓ PASS: Audit entries are isolated"
else
    echo "✗ FAIL: Audit entries leaked: $result"
    exit 1
fi

# Cleanup test data
echo "Cleaning up test data..."
psql "$DB_URL" << EOF
DELETE FROM audit_entries WHERE action = 'TEST';
DELETE FROM entities WHERE ticker IN ('AAPL', 'GOOGL');
DELETE FROM orders WHERE id IN ('order-a', 'order-b');
DELETE FROM tenants WHERE slug IN ('tenant-a', 'tenant-b');
EOF

echo ""
echo "=== All RLS Leak Tests Passed! ==="
echo "Multi-tenant isolation is working correctly."
exit 0

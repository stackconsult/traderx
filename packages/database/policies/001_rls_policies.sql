-- Row Level Security Policies for Multi-Tenant Isolation
-- Every query must have app.current_tenant set to access data

-- Enable RLS on all tables
ALTER TABLE entities ENABLE ROW LEVEL SECURITY;
ALTER TABLE snapshots ENABLE ROW LEVEL SECURITY;
ALTER TABLE deltas ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_entries ENABLE ROW LEVEL SECURITY;
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE positions ENABLE ROW LEVEL SECURITY;
ALTER TABLE trades ENABLE ROW LEVEL SECURITY;
ALTER TABLE strategies ENABLE ROW LEVEL SECURITY;
ALTER TABLE signals ENABLE ROW LEVEL SECURITY;

-- Create tenant isolation policies
-- Policy: Users can only access rows belonging to their tenant

CREATE POLICY tenant_isolation_entities ON entities
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_isolation_snapshots ON snapshots
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_isolation_deltas ON deltas
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_isolation_audit_entries ON audit_entries
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_isolation_orders ON orders
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_isolation_positions ON positions
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_isolation_trades ON trades
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_isolation_strategies ON strategies
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_isolation_signals ON signals
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- Allow tenants to insert their own data
CREATE POLICY tenant_insert_entities ON entities
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_insert_snapshots ON snapshots
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_insert_deltas ON deltas
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_insert_audit_entries ON audit_entries
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_insert_orders ON orders
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_insert_positions ON positions
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_insert_trades ON trades
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_insert_strategies ON strategies
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_insert_signals ON signals
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- Allow tenants to update their own data
CREATE POLICY tenant_update_entities ON entities
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_update_snapshots ON snapshots
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_update_deltas ON deltas
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_update_audit_entries ON audit_entries
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_update_orders ON orders
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_update_positions ON positions
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_update_trades ON trades
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_update_strategies ON strategies
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_update_signals ON signals
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- Allow tenants to delete their own data
CREATE POLICY tenant_delete_entities ON entities
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_delete_snapshots ON snapshots
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_delete_deltas ON deltas
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_delete_orders ON orders
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_delete_positions ON positions
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_delete_trades ON trades
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_delete_strategies ON strategies
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY tenant_delete_signals ON signals
    USING (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- Note: audit_entries should not be deletable (compliance requirement)

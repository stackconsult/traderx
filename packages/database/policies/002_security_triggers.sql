-- Security Triggers for Additional RLS Protection
-- Prevents accidental tenant_id manipulation at database level

-- Function to validate tenant context on INSERT/UPDATE
CREATE OR REPLACE FUNCTION validate_tenant_context()
RETURNS TRIGGER AS $$
DECLARE
    current_tenant UUID;
BEGIN
    -- Get current tenant from session
    current_tenant := current_setting('app.current_tenant', true)::UUID;
    
    -- If no tenant context, block operation
    IF current_tenant IS NULL THEN
        RAISE EXCEPTION 'No tenant context set for operation on table %', TG_TABLE_NAME;
    END IF;
    
    -- For INSERT: Ensure tenant_id matches context
    IF TG_OP = 'INSERT' THEN
        IF NEW.tenant_id IS NULL THEN
            NEW.tenant_id := current_tenant;
        ELSIF NEW.tenant_id != current_tenant THEN
            RAISE EXCEPTION 'Cannot insert data for different tenant on table %', TG_TABLE_NAME;
        END IF;
    END IF;
    
    -- For UPDATE: Prevent changing tenant_id
    IF TG_OP = 'UPDATE' THEN
        IF OLD.tenant_id != current_tenant THEN
            RAISE EXCEPTION 'Cannot update data for different tenant on table %', TG_TABLE_NAME;
        END IF;
        
        IF NEW.tenant_id != OLD.tenant_id THEN
            RAISE EXCEPTION 'Cannot change tenant_id on table %', TG_TABLE_NAME;
        END IF;
    END IF;
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Function to block DELETE on audit entries (compliance)
CREATE OR REPLACE FUNCTION block_audit_delete()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Cannot delete audit entries - compliance requirement';
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Apply validation triggers to all tables
-- Entities
CREATE TRIGGER validate_entities_tenant
    BEFORE INSERT OR UPDATE ON entities
    FOR EACH ROW EXECUTE FUNCTION validate_tenant_context();

-- Snapshots
CREATE TRIGGER validate_snapshots_tenant
    BEFORE INSERT OR UPDATE ON snapshots
    FOR EACH ROW EXECUTE FUNCTION validate_tenant_context();

-- Deltas
CREATE TRIGGER validate_deltas_tenant
    BEFORE INSERT OR UPDATE ON deltas
    FOR EACH ROW EXECUTE FUNCTION validate_tenant_context();

-- Audit Entries (with delete block)
CREATE TRIGGER validate_audit_entries_tenant
    BEFORE INSERT OR UPDATE ON audit_entries
    FOR EACH ROW EXECUTE FUNCTION validate_tenant_context();

CREATE TRIGGER block_audit_entries_delete
    BEFORE DELETE ON audit_entries
    FOR EACH ROW EXECUTE FUNCTION block_audit_delete();

-- Orders
CREATE TRIGGER validate_orders_tenant
    BEFORE INSERT OR UPDATE ON orders
    FOR EACH ROW EXECUTE FUNCTION validate_tenant_context();

-- Positions
CREATE TRIGGER validate_positions_tenant
    BEFORE INSERT OR UPDATE ON positions
    FOR EACH ROW EXECUTE FUNCTION validate_tenant_context();

-- Trades
CREATE TRIGGER validate_trades_tenant
    BEFORE INSERT OR UPDATE ON trades
    FOR EACH ROW EXECUTE FUNCTION validate_tenant_context();

-- Strategies
CREATE TRIGGER validate_strategies_tenant
    BEFORE INSERT OR UPDATE ON strategies
    FOR EACH ROW EXECUTE FUNCTION validate_tenant_context();

-- Signals
CREATE TRIGGER validate_signals_tenant
    BEFORE INSERT OR UPDATE ON signals
    FOR EACH ROW EXECUTE FUNCTION validate_tenant_context();

-- Function to log tenant access attempts (audit)
CREATE OR REPLACE FUNCTION log_tenant_access()
RETURNS TRIGGER AS $$
DECLARE
    current_tenant UUID;
BEGIN
    current_tenant := current_setting('app.current_tenant', true)::UUID;
    
    -- Log access attempt (this bypasses RLS as it's a system function)
    INSERT INTO audit_entries (id, tenant_id, action, agent, reasoning, inputs, outputs)
    VALUES (
        gen_random_uuid(),
        current_tenant,
        'TABLE_ACCESS',
        current_setting('app.current_user', true),
        sprintf('Accessed table %s with %d rows', TG_TABLE_NAME, TG_OP),
        json_build_object('table', TG_TABLE_NAME, 'operation', TG_OP),
        json_build_object('timestamp', NOW())
    );
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Optional: Uncomment to enable access logging
-- CREATE TRIGGER log_entities_access
--     AFTER SELECT OR INSERT OR UPDATE OR DELETE ON entities
--     FOR EACH STATEMENT EXECUTE FUNCTION log_tenant_access();

-- Create view for tenant isolation verification
CREATE OR REPLACE VIEW tenant_isolation_check AS
SELECT 
    'entities' as table_name,
    tenant_id,
    COUNT(*) as row_count
FROM entities
GROUP BY tenant_id
UNION ALL
SELECT 
    'snapshots' as table_name,
    tenant_id,
    COUNT(*) as row_count
FROM snapshots
GROUP BY tenant_id
UNION ALL
SELECT 
    'deltas' as table_name,
    tenant_id,
    COUNT(*) as row_count
FROM deltas
GROUP BY tenant_id
UNION ALL
SELECT 
    'audit_entries' as table_name,
    tenant_id,
    COUNT(*) as row_count
FROM audit_entries
GROUP BY tenant_id;

-- Grant necessary permissions
-- These would be set up per tenant in production
GRANT USAGE ON SCHEMA public TO PUBLIC;
GRANT SELECT ON tenant_isolation_check TO PUBLIC;

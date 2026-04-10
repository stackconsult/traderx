-- Add tenant_id columns to all Phase 2 tables
-- Enable multi-tenant isolation with Row Level Security

-- Enable UUID extension if not exists
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create tenants table
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    settings JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true
);

-- Add tenant_id to HSTR tables
ALTER TABLE entities ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE snapshots ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE deltas ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);

-- Add tenant_id to ZK-Audit table
ALTER TABLE audit_entries ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);

-- Add tenant_id to trading tables
ALTER TABLE orders ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE positions ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE trades ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);

-- Add tenant_id to strategy tables
ALTER TABLE strategies ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE signals ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);

-- Create indexes for tenant_id columns
CREATE INDEX IF NOT EXISTS idx_entities_tenant ON entities(tenant_id);
CREATE INDEX IF NOT EXISTS idx_snapshots_tenant ON snapshots(tenant_id);
CREATE INDEX IF NOT EXISTS idx_deltas_tenant ON deltas(tenant_id);
CREATE INDEX IF NOT EXISTS idx_audit_entries_tenant ON audit_entries(tenant_id);
CREATE INDEX IF NOT EXISTS idx_orders_tenant ON orders(tenant_id);
CREATE INDEX IF NOT EXISTS idx_positions_tenant ON positions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_trades_tenant ON trades(tenant_id);
CREATE INDEX IF NOT EXISTS idx_strategies_tenant ON strategies(tenant_id);
CREATE INDEX IF NOT EXISTS idx_signals_tenant ON signals(tenant_id);

-- Set default tenant for existing data (migration)
-- This will be replaced by actual tenant IDs in production
UPDATE entities SET tenant_id = '00000000-0000-0000-0000-000000000000'::uuid WHERE tenant_id IS NULL;
UPDATE snapshots SET tenant_id = '00000000-0000-0000-0000-000000000000'::uuid WHERE tenant_id IS NULL;
UPDATE deltas SET tenant_id = '00000000-0000-0000-0000-000000000000'::uuid WHERE tenant_id IS NULL;
UPDATE audit_entries SET tenant_id = '00000000-0000-0000-0000-000000000000'::uuid WHERE tenant_id IS NULL;
UPDATE orders SET tenant_id = '00000000-0000-0000-0000-000000000000'::uuid WHERE tenant_id IS NULL;
UPDATE positions SET tenant_id = '00000000-0000-0000-0000-000000000000'::uuid WHERE tenant_id IS NULL;
UPDATE trades SET tenant_id = '00000000-0000-0000-0000-000000000000'::uuid WHERE tenant_id IS NULL;
UPDATE strategies SET tenant_id = '00000000-0000-0000-0000-000000000000'::uuid WHERE tenant_id IS NULL;
UPDATE signals SET tenant_id = '00000000-0000-0000-0000-000000000000'::uuid WHERE tenant_id IS NULL;

-- Make tenant_id NOT NULL after migration
ALTER TABLE entities ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE snapshots ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE deltas ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE audit_entries ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE orders ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE positions ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE trades ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE strategies ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE signals ALTER COLUMN tenant_id SET NOT NULL;

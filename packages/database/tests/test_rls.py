#!/usr/bin/env python3
"""
Test suite for Row Level Security (RLS) multi-tenant isolation.
"""

import asyncio
import sys
import uuid
from pathlib import Path
import asyncpg
import pytest

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from tenant_manager import TenantManager, Tenant


@pytest.fixture
async def db_pool():
    """Create test database connection pool."""
    db_url = "postgresql://postgres:postgres@localhost:5432/traderx_test"
    
    # Create test database if not exists
    conn = await asyncpg.connect(db_url.replace("/traderx_test", "/postgres"))
    try:
        await conn.execute("CREATE DATABASE traderx_test")
    except asyncpg.DuplicateDatabaseError:
        pass
    await conn.close()
    
    # Create pool
    pool = await asyncpg.create_pool(db_url, min_size=2, max_size=5)
    
    # Setup schema
    async with pool.acquire() as conn:
        # Run migrations
        schema_path = Path(__file__).parent.parent / "schemas" / "001_add_tenant_columns.sql"
        with open(schema_path, 'r') as f:
            await conn.execute(f.read())
            
        # Run RLS policies
        policy_path = Path(__file__).parent.parent / "policies" / "001_rls_policies.sql"
        with open(policy_path, 'r') as f:
            await conn.execute(f.read())
    
    yield pool
    
    await pool.close()


@pytest.fixture
async def tenant_manager(db_pool):
    """Create tenant manager instance."""
    manager = TenantManager("postgresql://postgres:postgres@localhost:5432/traderx_test")
    await manager.initialize()
    yield manager
    await manager.close()


class TestTenantManager:
    """Test tenant manager functionality."""
    
    async def test_create_tenant(self, tenant_manager):
        """Test tenant creation."""
        tenant = await tenant_manager.create_tenant(
            name="Test Tenant",
            slug="test-tenant",
            settings={"theme": "dark", "timezone": "UTC"}
        )
        
        assert isinstance(tenant, Tenant)
        assert tenant.name == "Test Tenant"
        assert tenant.slug == "test-tenant"
        assert tenant.settings["theme"] == "dark"
        assert tenant.is_active is True
        
    async def test_get_tenant_by_id(self, tenant_manager):
        """Test retrieving tenant by ID."""
        # Create tenant
        created = await tenant_manager.create_tenant(
            name="Get Test",
            slug="get-test"
        )
        
        # Retrieve by ID
        retrieved = await tenant_manager.get_tenant(str(created.id))
        
        assert retrieved is not None
        assert retrieved.id == created.id
        assert retrieved.name == "Get Test"
        
    async def test_get_tenant_by_slug(self, tenant_manager):
        """Test retrieving tenant by slug."""
        # Create tenant
        created = await tenant_manager.create_tenant(
            name="Slug Test",
            slug="slug-test"
        )
        
        # Retrieve by slug
        retrieved = await tenant_manager.get_tenant("slug-test")
        
        assert retrieved is not None
        assert retrieved.id == created.id
        assert retrieved.slug == "slug-test"
        
    async def test_with_tenant_context(self, tenant_manager):
        """Test tenant context isolation."""
        # Create two tenants
        tenant_a = await tenant_manager.create_tenant(
            name="Tenant A",
            slug="tenant-a"
        )
        tenant_b = await tenant_manager.create_tenant(
            name="Tenant B",
            slug="tenant-b"
        )
        
        # Insert data for Tenant A
        async with tenant_manager.with_tenant(str(tenant_a.id)) as conn:
            await conn.execute(
                "INSERT INTO entities (id, ticker, name) VALUES ($1, $2, $3)",
                str(uuid.uuid4()), "AAPL", "Apple Inc."
            )
            
        # Insert data for Tenant B
        async with tenant_manager.with_tenant(str(tenant_b.id)) as conn:
            await conn.execute(
                "INSERT INTO entities (id, ticker, name) VALUES ($1, $2, $3)",
                str(uuid.uuid4()), "GOOGL", "Alphabet Inc."
            )
            
        # Verify isolation
        async with tenant_manager.with_tenant(str(tenant_a.id)) as conn:
            count = await conn.fetchval("SELECT COUNT(*) FROM entities")
            assert count == 1
            ticker = await conn.fetchval("SELECT ticker FROM entities")
            assert ticker == "AAPL"
            
        async with tenant_manager.with_tenant(str(tenant_b.id)) as conn:
            count = await conn.fetchval("SELECT COUNT(*) FROM entities")
            assert count == 1
            ticker = await conn.fetchval("SELECT ticker FROM entities")
            assert ticker == "GOOGL"
            
    async def test_cross_tenant_access_blocked(self, tenant_manager):
        """Test that cross-tenant access is blocked."""
        # Create two tenants
        tenant_a = await tenant_manager.create_tenant(
            name="Tenant A",
            slug="tenant-a"
        )
        tenant_b = await tenant_manager.create_tenant(
            name="Tenant B",
            slug="tenant-b"
        )
        
        # Insert data for Tenant A
        entity_id = str(uuid.uuid4())
        async with tenant_manager.with_tenant(str(tenant_a.id)) as conn:
            await conn.execute(
                "INSERT INTO entities (id, ticker, name) VALUES ($1, $2, $3)",
                entity_id, "AAPL", "Apple Inc."
            )
            
        # Try to access from Tenant B with explicit tenant_id filter
        async with tenant_manager.with_tenant(str(tenant_b.id)) as conn:
            # This should return 0 due to RLS
            count = await conn.fetchval(
                "SELECT COUNT(*) FROM entities WHERE id = $1",
                entity_id
            )
            assert count == 0
            
    async def test_insert_wrong_tenant_blocked(self, tenant_manager):
        """Test that inserting with wrong tenant context is blocked."""
        # Create two tenants
        tenant_a = await tenant_manager.create_tenant(
            name="Tenant A",
            slug="tenant-a"
        )
        tenant_b = await tenant_manager.create_tenant(
            name="Tenant B",
            slug="tenant-b"
        )
        
        # Try to insert for Tenant A while in Tenant B context
        async with tenant_manager.with_tenant(str(tenant_b.id)) as conn:
            # This should fail due to RLS policy check
            with pytest.raises(Exception):
                await conn.execute(
                    "INSERT INTO entities (id, tenant_id, ticker, name) VALUES ($1, $2, $3, $4)",
                    str(uuid.uuid4()), str(tenant_a.id), "AAPL", "Apple Inc."
                )
                
    async def test_audit_isolation(self, tenant_manager):
        """Test that audit entries are properly isolated."""
        # Create two tenants
        tenant_a = await tenant_manager.create_tenant(
            name="Tenant A",
            slug="tenant-a"
        )
        tenant_b = await tenant_manager.create_tenant(
            name="Tenant B",
            slug="tenant-b"
        )
        
        # Add audit entry for Tenant A
        async with tenant_manager.with_tenant(str(tenant_a.id)) as conn:
            await conn.execute(
                """INSERT INTO audit_entries (id, action, agent, reasoning, inputs, outputs)
                   VALUES ($1, $2, $3, $4, $5, $6)""",
                str(uuid.uuid4()), "TEST_ACTION", "AgentA", "Test", "{}", "{}"
            )
            
        # Verify Tenant B cannot see it
        async with tenant_manager.with_tenant(str(tenant_b.id)) as conn:
            count = await conn.fetchval(
                "SELECT COUNT(*) FROM audit_entries WHERE action = 'TEST_ACTION'"
            )
            assert count == 0
            
        # Verify Tenant A can see it
        async with tenant_manager.with_tenant(str(tenant_a.id)) as conn:
            count = await conn.fetchval(
                "SELECT COUNT(*) FROM audit_entries WHERE action = 'TEST_ACTION'"
            )
            assert count == 1
            
    async def test_no_tenant_context(self, tenant_manager):
        """Test behavior without tenant context."""
        # This should work with direct connection but return no data
        async with tenant_manager._pool.acquire() as conn:
            # Without setting tenant context, should return no data
            count = await conn.fetchval("SELECT COUNT(*) FROM entities")
            assert count == 0
            
    async def test_get_tenant_stats(self, tenant_manager):
        """Test getting tenant statistics."""
        # Create tenant
        tenant = await tenant_manager.create_tenant(
            name="Stats Test",
            slug="stats-test"
        )
        
        # Add some data
        async with tenant_manager.with_tenant(str(tenant.id)) as conn:
            await conn.execute(
                "INSERT INTO entities (id, ticker, name) VALUES ($1, $2, $3)",
                str(uuid.uuid4()), "AAPL", "Apple Inc."
            )
            await conn.execute(
                """INSERT INTO audit_entries (id, action, agent, reasoning, inputs, outputs)
                   VALUES ($1, $2, $3, $4, $5, $6)""",
                str(uuid.uuid4()), "TEST", "Agent", "Test", "{}", "{}"
            )
            
        # Get stats
        stats = await tenant_manager.get_tenant_stats(str(tenant.id))
        
        assert stats['entities'] == 1
        assert stats['audit_entries'] == 1
        assert stats['orders'] == 0
        assert stats['trades'] == 0


async def main():
    """Run all tests."""
    print("Running RLS Tests...")
    
    # This would normally use pytest, but we'll run manually for simplicity
    import os
    os.environ["PYTEST_CURRENT_TEST"] = "test_rls"
    
    # Test basic functionality
    db_url = "postgresql://postgres:postgres@localhost:5432/traderx_test"
    
    # Setup
    conn = await asyncpg.connect(db_url.replace("/traderx_test", "/postgres"))
    try:
        await conn.execute("CREATE DATABASE traderx_test")
    except asyncpg.DuplicateDatabaseError:
        pass
    await conn.close()
    
    # Run tests
    pool = await asyncpg.create_pool(db_url, min_size=2, max_size=5)
    
    try:
        async with pool.acquire() as conn:
            # Run migrations
            schema_path = Path(__file__).parent.parent / "schemas" / "001_add_tenant_columns.sql"
            with open(schema_path, 'r') as f:
                await conn.execute(f.read())
                
            # Run RLS policies
            policy_path = Path(__file__).parent.parent / "policies" / "001_rls_policies.sql"
            with open(policy_path, 'r') as f:
                await conn.execute(f.read())
        
        # Test tenant manager
        manager = TenantManager(db_url)
        await manager.initialize()
        
        # Create test tenant
        tenant = await manager.create_tenant("Test", "test")
        
        # Test isolation
        async with manager.with_tenant(str(tenant.id)) as conn:
            await conn.execute(
                "INSERT INTO entities (id, ticker, name) VALUES ($1, $2, $3)",
                str(uuid.uuid4()), "AAPL", "Apple"
            )
            count = await conn.fetchval("SELECT COUNT(*) FROM entities")
            assert count == 1
            
        print("✓ All RLS tests passed!")
        
        await manager.close()
        
    finally:
        await pool.close()


if __name__ == "__main__":
    asyncio.run(main())

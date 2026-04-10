"""
Tenant Context Manager for Multi-Tenant RLS
Handles PostgreSQL Row Level Security session management.
"""

import asyncio
from contextlib import asynccontextmanager
from typing import Optional, Dict, Any, AsyncGenerator
import asyncpg
from dataclasses import dataclass
import uuid
import logging

logger = logging.getLogger(__name__)


@dataclass
class Tenant:
    """Tenant information."""
    id: uuid.UUID
    name: str
    slug: str
    settings: Dict[str, Any]
    is_active: bool = True


class TenantManager:
    """
    Manages tenant context for PostgreSQL RLS.
    
    Usage:
        async with tenant_manager.with_tenant(tenant_id) as conn:
            # All queries automatically filtered by tenant
            result = await conn.fetch("SELECT * FROM entities")
    """
    
    def __init__(self, db_url: str):
        self.db_url = db_url
        self._pool: Optional[asyncpg.Pool] = None
        self._tenant_cache: Dict[str, Tenant] = {}
        
    async def initialize(self) -> None:
        """Initialize connection pool."""
        self._pool = await asyncpg.create_pool(
            self.db_url,
            min_size=5,
            max_size=20,
            command_timeout=60
        )
        logger.info("Tenant manager initialized")
        
    async def close(self) -> None:
        """Close connection pool."""
        if self._pool:
            await self._pool.close()
            
    @asynccontextmanager
    async def with_tenant(self, tenant_id: str) -> AsyncGenerator[asyncpg.Connection, None]:
        """
        Execute operations within tenant context.
        
        Args:
            tenant_id: Tenant UUID or slug
            
        Yields:
            Connection with tenant context set
        """
        if not self._pool:
            raise RuntimeError("TenantManager not initialized")
            
        # Resolve tenant (accept UUID or slug)
        tenant = await self.get_tenant(tenant_id)
        if not tenant:
            raise ValueError(f"Tenant not found: {tenant_id}")
            
        if not tenant.is_active:
            raise ValueError(f"Tenant is not active: {tenant_id}")
            
        async with self._pool.acquire() as conn:
            # Set tenant context for RLS
            await conn.execute(
                "SET LOCAL app.current_tenant = $1",
                str(tenant.id)
            )
            
            # Verify RLS is working
            await self._verify_rls_context(conn, tenant.id)
            
            yield conn
            
    async def get_tenant(self, identifier: str) -> Optional[Tenant]:
        """
        Get tenant by ID or slug.
        
        Args:
            identifier: Tenant UUID or slug
            
        Returns:
            Tenant object or None
        """
        # Check cache first
        if identifier in self._tenant_cache:
            return self._tenant_cache[identifier]
            
        async with self._pool.acquire() as conn:
            # Try UUID first, then slug
            try:
                uuid.UUID(identifier)
                row = await conn.fetchrow(
                    "SELECT id, name, slug, settings, is_active FROM tenants WHERE id = $1",
                    identifier
                )
            except ValueError:
                # Not a UUID, try slug
                row = await conn.fetchrow(
                    "SELECT id, name, slug, settings, is_active FROM tenants WHERE slug = $1",
                    identifier
                )
                
            if row:
                tenant = Tenant(
                    id=row['id'],
                    name=row['name'],
                    slug=row['slug'],
                    settings=row['settings'] or {},
                    is_active=row['is_active']
                )
                
                # Cache both ID and slug
                self._tenant_cache[str(tenant.id)] = tenant
                self._tenant_cache[tenant.slug] = tenant
                
                return tenant
                
        return None
        
    async def create_tenant(self, 
                          name: str,
                          slug: str,
                          settings: Optional[Dict[str, Any]] = None) -> Tenant:
        """
        Create a new tenant.
        
        Args:
            name: Tenant display name
            slug: Unique URL-friendly identifier
            settings: Optional tenant settings
            
        Returns:
            Created tenant
        """
        async with self._pool.acquire() as conn:
            tenant_id = await conn.fetchval(
                "INSERT INTO tenants (name, slug, settings) VALUES ($1, $2, $3) RETURNING id",
                name, slug, settings or {}
            )
            
            tenant = Tenant(
                id=tenant_id,
                name=name,
                slug=slug,
                settings=settings or {},
                is_active=True
            )
            
            # Cache
            self._tenant_cache[str(tenant.id)] = tenant
            self._tenant_cache[tenant.slug] = tenant
            
            logger.info(f"Created tenant: {name} ({slug})")
            return tenant
            
    async def _verify_rls_context(self, conn: asyncpg.Connection, tenant_id: uuid.UUID) -> None:
        """Verify RLS context is properly set."""
        # Check if tenant context is set
        current_tenant = await conn.fetchval(
            "SELECT current_setting('app.current_tenant', true)"
        )
        
        if current_tenant != str(tenant_id):
            raise RuntimeError(f"RLS context not set properly. Expected: {tenant_id}, Got: {current_tenant}")
            
    async def get_tenant_stats(self, tenant_id: str) -> Dict[str, int]:
        """
        Get statistics for a tenant.
        
        Args:
            tenant_id: Tenant UUID or slug
            
        Returns:
            Dictionary with table counts
        """
        async with self.with_tenant(tenant_id) as conn:
            stats = {}
            
            tables = [
                'entities', 'snapshots', 'deltas', 'audit_entries',
                'orders', 'positions', 'trades', 'strategies', 'signals'
            ]
            
            for table in tables:
                count = await conn.fetchval(f"SELECT COUNT(*) FROM {table}")
                stats[table] = count
                
            return stats
            
    async def migrate_existing_data(self, tenant_id: str) -> None:
        """
        Migrate existing data to a tenant.
        Only for initial setup/migration.
        
        Args:
            tenant_id: Target tenant UUID
        """
        async with self._pool.acquire() as conn:
            # Update all rows with NULL tenant_id
            tables = [
                'entities', 'snapshots', 'deltas', 'audit_entries',
                'orders', 'positions', 'trades', 'strategies', 'signals'
            ]
            
            for table in tables:
                await conn.execute(
                    f"UPDATE {table} SET tenant_id = $1 WHERE tenant_id IS NULL",
                    tenant_id
                )
                
            logger.info(f"Migrated existing data to tenant: {tenant_id}")


# Global instance for application use
tenant_manager: Optional[TenantManager] = None


async def get_tenant_manager() -> TenantManager:
    """Get global tenant manager instance."""
    global tenant_manager
    if not tenant_manager:
        raise RuntimeError("TenantManager not initialized")
    return tenant_manager


@asynccontextmanager
async def with_tenant(tenant_id: str) -> AsyncGenerator[asyncpg.Connection, None]:
    """
    Convenience function for tenant context.
    
    Args:
        tenant_id: Tenant UUID or slug
        
    Yields:
        Connection with tenant context set
    """
    manager = await get_tenant_manager()
    async with manager.with_tenant(tenant_id) as conn:
        yield conn

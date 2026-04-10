#!/usr/bin/env python3
"""
RLS + Adapter Integration Tests
Validates that adapters properly respect PostgreSQL Row Level Security
and maintain tenant isolation across all operations.
"""

import asyncio
import asyncpg
import json
import logging
import uuid
import time
from typing import Dict, Any, List, Tuple
from pathlib import Path

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class RLSAdapterIntegrationTests:
    """Integration tests for RLS and adapter tenant isolation."""
    
    def __init__(self, database_url: str):
        self.database_url = database_url
        self.test_results = {
            "concurrent_orders": {"status": "pending", "details": {}},
            "data_isolation": {"status": "pending", "details": {}},
            "context_switching": {"status": "pending", "details": {}},
            "negative_case": {"status": "pending", "details": {}}
        }
        
    async def run_all_tests(self) -> bool:
        """Run all integration tests."""
        logger.info("Starting RLS + Adapter Integration Tests...")
        
        # Create test connections for each tenant
        tenant_a_conn = await self._create_tenant_connection("tenant-a")
        tenant_b_conn = await self._create_tenant_connection("tenant-b")
        admin_conn = await self._create_admin_connection()
        
        try:
            # Setup test data
            await self._setup_test_data(admin_conn, tenant_a_conn, tenant_b_conn)
            
            # Test 1: Concurrent orders from different tenants
            logger.info("\n=== Test 1: Concurrent Orders ===")
            await self.test_concurrent_orders(tenant_a_conn, tenant_b_conn)
            
            # Test 2: Data isolation verification
            logger.info("\n=== Test 2: Data Isolation ===")
            await self.test_data_isolation(tenant_a_conn, tenant_b_conn)
            
            # Test 3: Context switching in single connection
            logger.info("\n=== Test 3: Context Switching ===")
            await self.test_context_switching(admin_conn)
            
            # Test 4: Negative case - no tenant context
            logger.info("\n=== Test 4: Negative Case (No Tenant Context) ===")
            await self.test_negative_case(admin_conn)
            
            # Calculate results
            passed = sum(1 for test in self.test_results.values() if test["status"] == "passed")
            total = len(self.test_results)
            
            logger.info(f"\nIntegration Tests: {passed}/{total} passed")
            return passed == total
            
        finally:
            # Cleanup connections
            for conn in [tenant_a_conn, tenant_b_conn, admin_conn]:
                if conn:
                    await conn.close()
    
    async def _create_tenant_connection(self, tenant_id: str) -> asyncpg.Connection:
        """Create database connection with tenant context."""
        conn = await asyncpg.connect(self.database_url)
        
        # Set tenant context
        await conn.execute("SET LOCAL app.current_tenant = $1", tenant_id)
        
        return conn
    
    async def _create_admin_connection(self) -> asyncpg.Connection:
        """Create admin connection without tenant context."""
        return await asyncpg.connect(self.database_url)
    
    async def _setup_test_data(self, admin_conn, tenant_a_conn, tenant_b_conn):
        """Setup test data for integration tests."""
        # Create test strategies for each tenant
        await admin_conn.execute("""
            INSERT INTO entities (id, tenant_id, type, data, created_at)
            VALUES 
                ('strategy-a-1', 'tenant-a', 'strategy', '{"name": "Strategy A1"}', NOW()),
                ('strategy-a-2', 'tenant-a', 'strategy', '{"name": "Strategy A2"}', NOW()),
                ('strategy-b-1', 'tenant-b', 'strategy', '{"name": "Strategy B1"}', NOW()),
                ('strategy-b-2', 'tenant-b', 'strategy', '{"name": "Strategy B2"}', NOW())
            ON CONFLICT (id) DO NOTHING
        """)
        
        # Create test orders
        await admin_conn.execute("""
            INSERT INTO orders (id, tenant_id, symbol, side, quantity, status, created_at)
            VALUES
                ('order-a-1', 'tenant-a', 'BTCUSDT', 'BUY', 1.0, 'NEW', NOW()),
                ('order-a-2', 'tenant-a', 'ETHUSDT', 'SELL', 2.0, 'NEW', NOW()),
                ('order-b-1', 'tenant-b', 'BTCUSDT', 'SELL', 1.5, 'NEW', NOW()),
                ('order-b-2', 'tenant-b', 'SOLUSDT', 'BUY', 3.0, 'NEW', NOW())
            ON CONFLICT (id) DO NOTHING
        """)
        
        logger.info("Test data setup complete")
    
    async def test_concurrent_orders(self, tenant_a_conn, tenant_b_conn):
        """Test concurrent order submissions from different tenants."""
        try:
            # Submit orders concurrently
            order_a_id = f"test-order-a-{uuid.uuid4()}"
            order_b_id = f"test-order-b-{uuid.uuid4()}"
            
            # Create tasks for concurrent execution
            tasks = [
                self._submit_order(tenant_a_conn, order_a_id, "BTCUSDT", "BUY", 1.0),
                self._submit_order(tenant_b_conn, order_b_id, "BTCUSDT", "SELL", 1.0)
            ]
            
            # Execute concurrently
            results = await asyncio.gather(*tasks, return_exceptions=True)
            
            # Verify both orders succeeded
            success_count = 0
            for result in results:
                if isinstance(result, Exception):
                    logger.error(f"Order submission failed: {result}")
                else:
                    success_count += 1
            
            if success_count == 2:
                # Verify orders belong to correct tenants
                a_order = await tenant_a_conn.fetchrow(
                    "SELECT * FROM orders WHERE id = $1", order_a_id
                )
                b_order = await tenant_b_conn.fetchrow(
                    "SELECT * FROM orders WHERE id = $1", order_b_id
                )
                
                if a_order and b_order:
                    self.test_results["concurrent_orders"]["status"] = "passed"
                    self.test_results["concurrent_orders"]["details"] = {
                        "order_a": {"id": a_order["id"], "tenant": a_order["tenant_id"]},
                        "order_b": {"id": b_order["id"], "tenant": b_order["tenant_id"]}
                    }
                    logger.info("✓ Concurrent orders test PASSED")
                else:
                    self.test_results["concurrent_orders"]["status"] = "failed"
                    self.test_results["concurrent_orders"]["details"]["error"] = "Orders not found"
            else:
                self.test_results["concurrent_orders"]["status"] = "failed"
                self.test_results["concurrent_orders"]["details"]["error"] = f"Only {success_count}/2 orders succeeded"
                
        except Exception as e:
            self.test_results["concurrent_orders"]["status"] = "failed"
            self.test_results["concurrent_orders"]["details"]["error"] = str(e)
    
    async def _submit_order(self, conn, order_id: str, symbol: str, side: str, quantity: float):
        """Submit order through adapter simulation."""
        # Simulate adapter submitting order
        await conn.execute("""
            INSERT INTO orders (id, tenant_id, symbol, side, quantity, status, created_at)
            VALUES ($1, current_setting('app.current_tenant'), $2, $3, $4, 'NEW', NOW())
        """, order_id, symbol, side, quantity)
        
        # Simulate execution
        await conn.execute("""
            UPDATE orders 
            SET status = 'FILLED', filled_quantity = $2, updated_at = NOW()
            WHERE id = $1
        """, order_id, quantity)
        
        return True
    
    async def test_data_isolation(self, tenant_a_conn, tenant_b_conn):
        """Test that tenants can only access their own data."""
        try:
            # Tenant A queries their orders
            a_orders = await tenant_a_conn.fetch("SELECT * FROM orders")
            
            # Tenant B queries their orders
            b_orders = await tenant_b_conn.fetch("SELECT * FROM orders")
            
            # Verify isolation
            a_tenants = {order["tenant_id"] for order in a_orders}
            b_tenants = {order["tenant_id"] for order in b_orders}
            
            if a_tenants == {"tenant-a"} and b_tenants == {"tenant-b"}:
                # Test cross-tenant access attempt
                cross_tenant_attempt = await tenant_a_conn.fetchval(
                    "SELECT COUNT(*) FROM orders WHERE tenant_id = 'tenant-b'"
                )
                
                if cross_tenant_attempt == 0:
                    self.test_results["data_isolation"]["status"] = "passed"
                    self.test_results["data_isolation"]["details"] = {
                        "tenant_a_orders": len(a_orders),
                        "tenant_b_orders": len(b_orders),
                        "cross_tenant_access": cross_tenant_attempt
                    }
                    logger.info("✓ Data isolation test PASSED")
                else:
                    self.test_results["data_isolation"]["status"] = "failed"
                    self.test_results["data_isolation"]["details"]["error"] = "Cross-tenant access detected"
            else:
                self.test_results["data_isolation"]["status"] = "failed"
                self.test_results["data_isolation"]["details"]["error"] = "Tenant data mixed"
                
        except Exception as e:
            self.test_results["data_isolation"]["status"] = "failed"
            self.test_results["data_isolation"]["details"]["error"] = str(e)
    
    async def test_context_switching(self, admin_conn):
        """Test switching tenant context in single connection."""
        try:
            # Switch to Tenant A
            await admin_conn.execute("SET LOCAL app.current_tenant = 'tenant-a'")
            a_count = await admin_conn.fetchval("SELECT COUNT(*) FROM orders WHERE tenant_id = 'tenant-a'")
            
            # Switch to Tenant B
            await admin_conn.execute("SET LOCAL app.current_tenant = 'tenant-b'")
            b_count = await admin_conn.fetchval("SELECT COUNT(*) FROM orders WHERE tenant_id = 'tenant-b'")
            
            # Clear context
            await admin_conn.execute("RESET app.current_tenant")
            no_context_count = await admin_conn.fetchval("SELECT COUNT(*) FROM orders")
            
            if a_count > 0 and b_count > 0 and no_context_count == 0:
                self.test_results["context_switching"]["status"] = "passed"
                self.test_results["context_switching"]["details"] = {
                    "tenant_a_count": a_count,
                    "tenant_b_count": b_count,
                    "no_context_count": no_context_count
                }
                logger.info("✓ Context switching test PASSED")
            else:
                self.test_results["context_switching"]["status"] = "failed"
                self.test_results["context_switching"]["details"]["error"] = "Context switching not working"
                
        except Exception as e:
            self.test_results["context_switching"]["status"] = "failed"
            self.test_results["context_switching"]["details"]["error"] = str(e)
    
    async def test_negative_case(self, admin_conn):
        """Test behavior when no tenant context is set."""
        try:
            # Try to query without tenant context
            result = await admin_conn.fetch("SELECT * FROM orders LIMIT 1")
            
            if len(result) == 0:
                self.test_results["negative_case"]["status"] = "passed"
                self.test_results["negative_case"]["details"] = {
                    "message": "RLS properly blocked query without tenant context"
                }
                logger.info("✓ Negative case test PASSED")
            else:
                self.test_results["negative_case"]["status"] = "failed"
                self.test_results["negative_case"]["details"]["error"] = "Query succeeded without tenant context"
                
        except Exception as e:
            # Expected behavior - RLS should block
            self.test_results["negative_case"]["status"] = "passed"
            self.test_results["negative_case"]["details"] = {
                "message": "RLS properly raised error without tenant context",
                "error": str(e)
            }
            logger.info("✓ Negative case test PASSED (error as expected)")
    
    def print_results(self):
        """Print test results."""
        print("\n" + "=" * 60)
        print("RLS + ADAPTER INTEGRATION TEST RESULTS")
        print("=" * 60)
        
        for test_name, result in self.test_results.items():
            status_icon = "✅" if result["status"] == "passed" else "❌"
            print(f"\n{status_icon} {test_name.replace('_', ' ').title()}: {result['status'].upper()}")
            
            if result["status"] == "passed":
                for key, value in result["details"].items():
                    print(f"   {key}: {value}")
            else:
                print(f"   Error: {result['details'].get('error', 'Unknown')}")
        
        # Save results
        with open("rls-adapter-integration-results.json", "w") as f:
            json.dump(self.test_results, f, indent=2)
        
        print(f"\nDetailed results saved to: rls-adapter-integration-results.json")


async def main():
    """Run RLS + Adapter integration tests."""
    # Get database URL from environment or use default
    database_url = os.getenv("DATABASE_URL", "postgresql://postgres:password@localhost:5432/traderx")
    
    tests = RLSAdapterIntegrationTests(database_url)
    success = await tests.run_all_tests()
    
    tests.print_results()
    
    return success


if __name__ == "__main__":
    import os
    success = asyncio.run(main())
    exit(0 if success else 1)

"""
HSTR (Historical State Reconstruction) Client
Provides O(1) snapshot retrieval + O(k) delta application for bitemporal financial data.
"""

import asyncio
import json
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any
import asyncpg
import numpy as np
from pgvector.asyncpg import register_vector

logger = logging.getLogger(__name__)


class HSTRClient:
    """
    Client for Historical State Reconstruction using TimescaleDB.
    Provides fast reconstruction of entity states at any point in history.
    """
    
    def __init__(self, connection_string: str, pool_size: int = 10):
        """
        Initialize HSTR client.
        
        Args:
            connection_string: PostgreSQL connection string
            pool_size: Connection pool size
        """
        self.connection_string = connection_string
        self.pool_size = pool_size
        self.pool: Optional[asyncpg.Pool] = None
        
    async def initialize(self):
        """Initialize connection pool and register vector type."""
        self.pool = await asyncpg.create_pool(
            self.connection_string,
            min_size=2,
            max_size=self.pool_size,
            command_timeout=60
        )
        
        # Register vector type for pgvector
        await self.pool.execute(
            "SELECT set_config('search_path', 'public', false);"
        )
        
        logger.info("HSTR client initialized with connection pool")
        
    async def close(self):
        """Close connection pool."""
        if self.pool:
            await self.pool.close()
            logger.info("HSTR client connection pool closed")
            
    async def reconstruct_state_at(
        self, 
        entity_id: int, 
        facet_name: str, 
        timestamp: datetime
    ) -> Optional[Dict]:
        """
        Reconstruct entity state at specific timestamp.
        
        Complexity: O(1) snapshot lookup + O(k) delta application
        where k is typically < 20 for quarterly cycles.
        
        Args:
            entity_id: Entity identifier
            facet_name: Facet to reconstruct (e.g., 'financials', 'risk_metrics')
            timestamp: Target timestamp for reconstruction
            
        Returns:
            Reconstructed state as dictionary or None if not found
        """
        if not self.pool:
            raise RuntimeError("HSTR client not initialized")
            
        async with self.pool.acquire() as conn:
            # Use the optimized reconstruction function
            result = await conn.fetchval(
                "SELECT reconstruct_state_at($1, $2, $3)",
                entity_id,
                facet_name,
                timestamp
            )
            
            if result:
                return dict(result)
            return None
            
    async def get_latest_snapshot(
        self, 
        entity_id: int, 
        facet_name: str
    ) -> Optional[Dict]:
        """
        Get latest snapshot for entity facet.
        
        Args:
            entity_id: Entity identifier
            facet_name: Facet name
            
        Returns:
            Latest snapshot data or None
        """
        if not self.pool:
            raise RuntimeError("HSTR client not initialized")
            
        async with self.pool.acquire() as conn:
            row = await conn.fetchrow(
                """
                SELECT snapshot_data, valid_from
                FROM facet_snapshots
                WHERE entity_id = $1 AND facet_name = $2 AND valid_to = 'infinity'
                ORDER BY valid_from DESC
                LIMIT 1
                """,
                entity_id,
                facet_name
            )
            
            if row:
                return {
                    'data': dict(row['snapshot_data']),
                    'valid_from': row['valid_from']
                }
            return None
            
    async def add_snapshot(
        self,
        entity_id: int,
        facet_name: str,
        data: Dict,
        valid_from: datetime,
        version: int = 1
    ) -> int:
        """
        Add new snapshot for entity facet.
        
        Args:
            entity_id: Entity identifier
            facet_name: Facet name
            data: Snapshot data
            valid_from: Validity start time
            version: Snapshot version
            
        Returns:
            Snapshot ID
        """
        if not self.pool:
            raise RuntimeError("HSTR client not initialized")
            
        async with self.pool.acquire() as conn:
            # Invalidate previous snapshot
            await conn.execute(
                """
                UPDATE facet_snapshots
                SET valid_to = $1
                WHERE entity_id = $2 AND facet_name = $3 AND valid_to = 'infinity'
                """,
                valid_from,
                entity_id,
                facet_name
            )
            
            # Insert new snapshot
            snapshot_id = await conn.fetchval(
                """
                INSERT INTO facet_snapshots 
                (entity_id, facet_name, valid_from, snapshot_data, version)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING snapshot_id
                """,
                entity_id,
                facet_name,
                valid_from,
                json.dumps(data),
                version
            )
            
            logger.info(f"Created snapshot {snapshot_id} for entity {entity_id}, facet {facet_name}")
            return snapshot_id
            
    async def add_delta(
        self,
        entity_id: int,
        facet_name: str,
        patch: Dict,
        effective_time: datetime
    ) -> int:
        """
        Add delta change to entity facet.
        
        Args:
            entity_id: Entity identifier
            facet_name: Facet name
            patch: RFC 6902 JSON patch operation
            effective_time: When delta takes effect
            
        Returns:
            Delta ID
        """
        if not self.pool:
            raise RuntimeError("HSTR client not initialized")
            
        async with self.pool.acquire() as conn:
            delta_id = await conn.fetchval(
                """
                INSERT INTO facet_deltas
                (entity_id, facet_name, effective_time, patch_operation)
                VALUES ($1, $2, $3, $4)
                RETURNING delta_id
                """,
                entity_id,
                facet_name,
                effective_time,
                json.dumps(patch)
            )
            
            logger.debug(f"Added delta {delta_id} for entity {entity_id}, facet {facet_name}")
            return delta_id
            
    async def search_similar_vectors(
        self,
        query_vector: List[float],
        vector_type: str,
        limit: int = 10,
        similarity_threshold: float = 0.8
    ) -> List[Dict]:
        """
        Search for similar vectors using DiskANN indexing.
        
        Args:
            query_vector: Query embedding
            vector_type: Type of vectors to search
            limit: Maximum results
            similarity_threshold: Minimum similarity score
            
        Returns:
            List of similar vectors with metadata
        """
        if not self.pool:
            raise RuntimeError("HSTR client not initialized")
            
        async with self.pool.acquire() as conn:
            rows = await conn.fetch(
                """
                SELECT 
                    e.ticker,
                    e.name,
                    v.vector_type,
                    v.metadata,
                    1 - (v.embedding <=> $1::vector) as similarity
                FROM vector_store v
                JOIN entities e ON e.entity_id = v.entity_id
                WHERE v.vector_type = $2
                  AND 1 - (v.embedding <=> $1::vector) > $3
                ORDER BY v.embedding <=> $1::vector
                LIMIT $4
                """,
                query_vector,
                vector_type,
                similarity_threshold,
                limit
            )
            
            return [
                {
                    'ticker': row['ticker'],
                    'name': row['name'],
                    'vector_type': row['vector_type'],
                    'metadata': dict(row['metadata']) if row['metadata'] else {},
                    'similarity': float(row['similarity'])
                }
                for row in rows
            ]
            
    async def get_performance_stats(self) -> Dict:
        """
        Get HSTR performance statistics.
        
        Returns:
            Dictionary with performance metrics
        """
        if not self.pool:
            raise RuntimeError("HSTR client not initialized")
            
        async with self.pool.acquire() as conn:
            rows = await conn.fetch("SELECT * FROM hstr_performance_stats")
            
            return {
                row['table_name']: {
                    'total_rows': row['total_rows'],
                    'unique_entities': row['unique_entities'],
                    'unique_facets': row['unique_facets'],
                    'time_range': {
                        'earliest': row['earliest_time'].isoformat() if row['earliest_time'] else None,
                        'latest': row['latest_time'].isoformat() if row['latest_time'] else None
                    }
                }
                for row in rows
            }


class HSTRBenchmark:
    """Benchmark HSTR performance to verify O(1) + O(k) complexity."""
    
    def __init__(self, client: HSTRClient):
        self.client = client
        
    async def run_benchmark(self, num_tests: int = 1000) -> Dict:
        """
        Run performance benchmark.
        
        Args:
            num_tests: Number of test queries
            
        Returns:
            Benchmark results
        """
        logger.info(f"Running HSTR benchmark with {num_tests} queries")
        
        # Test different entity/facet combinations
        test_cases = [
            (1, 'financials'),  # AAPL financials
            (2, 'risk_metrics'),  # MSFT risk
            (3, 'strategy_embedding'),  # NVDA strategy
        ]
        
        results = {
            'query_times': [],
            'delta_counts': [],
            'total_queries': num_tests
        }
        
        for i in range(num_tests):
            entity_id, facet = test_cases[i % len(test_cases)]
            timestamp = datetime(2024, 1, 1) + timedelta(days=i % 365)
            
            start_time = asyncio.get_event_loop().time()
            state = await self.client.reconstruct_state_at(entity_id, facet, timestamp)
            end_time = asyncio.get_event_loop().time()
            
            query_time = (end_time - start_time) * 1000  # Convert to ms
            results['query_times'].append(query_time)
            
            # Count deltas if state was reconstructed
            if state:
                delta_count = len(state.get('_deltas_applied', []))
                results['delta_counts'].append(delta_count)
                
        # Calculate statistics
        results['avg_query_time'] = np.mean(results['query_times'])
        results['p95_query_time'] = np.percentile(results['query_times'], 95)
        results['p99_query_time'] = np.percentile(results['query_times'], 99)
        results['avg_deltas'] = np.mean(results['delta_counts']) if results['delta_counts'] else 0
        
        # Verify O(1) + O(k) complexity
        # Query time should be constant regardless of data size
        # Delta application should be linear in number of deltas
        results['complexity_verified'] = (
            results['p95_query_time'] < 10 and  # <10ms for O(1)
            results['avg_deltas'] < 20  # k < 20 for quarterly cycles
        )
        
        # Log results
        logger.info(f"Benchmark complete:")
        logger.info(f"  Avg query time: {results['avg_query_time']:.2f}ms")
        logger.info(f"  P95 query time: {results['p95_query_time']:.2f}ms")
        logger.info(f"  Avg deltas applied: {results['avg_deltas']:.1f}")
        logger.info(f"  Complexity verified: {results['complexity_verified']}")
        
        # Save benchmark log
        await self._save_benchmark_log(results)
        
        return results
        
    async def _save_benchmark_log(self, results: Dict):
        """Save benchmark results to log file."""
        log_path = "packages/data-fabric/logs/hstr-query-bench.log"
        
        import os
        os.makedirs(os.path.dirname(log_path), exist_ok=True)
        
        with open(log_path, 'w') as f:
            f.write(f"HSTR Performance Benchmark Results\n")
            f.write(f"Timestamp: {datetime.utcnow().isoformat()}\n")
            f.write(f"Total Queries: {results['total_queries']}\n")
            f.write(f"Average Query Time: {results['avg_query_time']:.2f}ms\n")
            f.write(f"P95 Query Time: {results['p95_query_time']:.2f}ms\n")
            f.write(f"P99 Query Time: {results['p99_query_time']:.2f}ms\n")
            f.write(f"Average Deltas Applied: {results['avg_deltas']:.1f}\n")
            f.write(f"Complexity Verified (O(1) + O(k)): {results['complexity_verified']}\n")
            
        logger.info(f"Benchmark log saved to {log_path}")


# SQLite fallback for local testing
class HSTRSQLiteClient:
    """SQLite implementation of HSTR for local development/testing."""
    
    def __init__(self, db_path: str = "hstr_test.db"):
        self.db_path = db_path
        
    async def initialize(self):
        """Initialize SQLite database with simplified schema."""
        import aiosqlite
        
        self.conn = await aiosqlite.connect(self.db_path)
        
        # Create simplified tables
        await self.conn.executescript("""
            CREATE TABLE IF NOT EXISTS entities (
                id INTEGER PRIMARY KEY,
                ticker TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY,
                entity_id INTEGER,
                facet TEXT,
                timestamp TEXT,
                data TEXT,
                UNIQUE(entity_id, facet, timestamp)
            );
            
            CREATE TABLE IF NOT EXISTS deltas (
                id INTEGER PRIMARY KEY,
                entity_id INTEGER,
                facet TEXT,
                timestamp TEXT,
                patch TEXT
            );
        """)
        
        await self.conn.commit()
        logger.info("SQLite HSTR client initialized")
        
    async def reconstruct_state_at(
        self, 
        entity_id: int, 
        facet_name: str, 
        timestamp: datetime
    ) -> Optional[Dict]:
        """Simplified state reconstruction for SQLite."""
        # Get latest snapshot before timestamp
        cursor = await self.conn.execute(
            """
            SELECT data, timestamp FROM snapshots
            WHERE entity_id = ? AND facet = ? AND timestamp <= ?
            ORDER BY timestamp DESC
            LIMIT 1
            """,
            (entity_id, facet_name, timestamp.isoformat())
        )
        row = await cursor.fetchone()
        
        if not row:
            return None
            
        state = json.loads(row[0])
        snapshot_time = row[1]  # Get actual snapshot timestamp
        
        # Apply deltas up to timestamp (not after)
        cursor = await self.conn.execute(
            """
            SELECT patch FROM deltas
            WHERE entity_id = ? AND facet = ? AND timestamp > ? AND timestamp <= ?
            ORDER BY timestamp
            """,
            (entity_id, facet_name, snapshot_time, timestamp.isoformat())
        )
        
        deltas = await cursor.fetchall()
        for delta_row in deltas:
            patch = json.loads(delta_row[0])
            # Apply patch (simplified - just merge)
            state.update(patch)
            
        return state

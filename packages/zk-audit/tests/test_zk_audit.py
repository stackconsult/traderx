#!/usr/bin/env python3
"""
Test suite for ZK-Audit Flight Recorder.
"""

import asyncio
import sys
from pathlib import Path
from datetime import datetime, timedelta
import json
import tempfile

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from src.zk_audit_ledger import ZKAuditLedger, AuditEntry


async def test_ledger_initialization():
    """Test ZK-Audit ledger initialization."""
    print("Testing ZK-Audit ledger initialization...")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        ledger_path = Path(tmpdir) / "test-ledger.log"
        key_path = Path(tmpdir) / "test-keys.json"
        
        ledger = ZKAuditLedger(
            ledger_path=ledger_path,
            key_path=key_path
        )
        
        # Initialize ledger
        success = await ledger.initialize()
        assert success, "Ledger initialization failed"
        
        # Check keys were generated
        assert key_path.exists(), "Keys file not created"
        assert ledger.private_key is not None, "Private key not generated"
        assert ledger.public_key is not None, "Public key not generated"
        
        # Check ledger is empty
        assert ledger.entry_count == 0, "Ledger should be empty"
        assert ledger.tip_hash is None, "Tip hash should be None"
        
        print("✓ Ledger initialization test passed")
        

async def test_add_entry():
    """Test adding entries to the audit ledger."""
    print("Testing add entry...")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        ledger_path = Path(tmpdir) / "test-ledger.log"
        key_path = Path(tmpdir) / "test-keys.json"
        
        ledger = ZKAuditLedger(
            ledger_path=ledger_path,
            key_path=key_path
        )
        
        await ledger.initialize()
        
        # Add test entry
        entry_hash = await ledger.add_entry(
            action="EXECUTE_TRADE",
            agent="DeltaLag-SGY",
            reasoning="SPY leads DAX by 2.3h with IC=0.08",
            inputs={"symbol": "DAX", "side": "BUY", "size": 1000000},
            outputs={"order_id": "ORD-12345", "status": "FILLED"}
        )
        
        assert entry_hash is not None, "Entry hash should not be None"
        assert ledger.entry_count == 1, "Should have 1 entry"
        assert ledger.tip_hash == entry_hash, "Tip hash should match entry hash"
        
        # Check ledger file
        assert ledger_path.exists(), "Ledger file not created"
        
        # Verify entry content
        entries = await ledger.get_entries()
        assert len(entries) == 1, "Should have 1 entry"
        
        entry = entries[0]
        assert entry.action == "EXECUTE_TRADE"
        assert entry.agent == "DeltaLag-SGY"
        assert entry.signature is not None, "Entry should be signed"
        assert entry.prev_hash is None, "First entry should have no previous hash"
        
        print("✓ Add entry test passed")
        

async def test_chain_verification():
    """Test audit chain verification."""
    print("Testing chain verification...")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        ledger_path = Path(tmpdir) / "test-ledger.log"
        key_path = Path(tmpdir) / "test-keys.json"
        
        ledger = ZKAuditLedger(
            ledger_path=ledger_path,
            key_path=key_path
        )
        
        await ledger.initialize()
        
        # Add multiple entries
        for i in range(3):
            await ledger.add_entry(
                action=f"ACTION_{i}",
                agent=f"AGENT_{i}",
                reasoning=f"Test reasoning {i}",
                inputs={"test": i},
                outputs={"result": i * 2}
            )
            
        # Verify chain
        is_valid = await ledger.verify_chain()
        assert is_valid, "Chain should be valid"
        
        # Verify partial chain
        is_valid = await ledger.verify_chain(start_index=1)
        assert is_valid, "Partial chain should be valid"
        
        print("✓ Chain verification test passed")
        

async def test_entry_persistence():
    """Test that entries persist across sessions."""
    print("Testing entry persistence...")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        ledger_path = Path(tmpdir) / "test-ledger.log"
        key_path = Path(tmpdir) / "test-keys.json"
        
        # First session
        ledger1 = ZKAuditLedger(
            ledger_path=ledger_path,
            key_path=key_path
        )
        
        await ledger1.initialize()
        
        entry_hash = await ledger1.add_entry(
            action="PERSIST_TEST",
            agent="TEST_AGENT",
            reasoning="Testing persistence",
            inputs={"test": True},
            outputs={"success": True}
        )
        
        # Second session
        ledger2 = ZKAuditLedger(
            ledger_path=ledger_path,
            key_path=key_path
        )
        
        await ledger2.initialize()
        
        # Verify entry loaded
        assert ledger2.entry_count == 1, "Entry should be persisted"
        assert ledger2.tip_hash == entry_hash, "Tip hash should match"
        
        entries = await ledger2.get_entries()
        assert len(entries) == 1, "Should have 1 persisted entry"
        assert entries[0].action == "PERSIST_TEST", "Entry content should match"
        
        print("✓ Entry persistence test passed")
        

async def test_query_entries():
    """Test querying audit entries."""
    print("Testing query entries...")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        ledger_path = Path(tmpdir) / "test-ledger.log"
        key_path = Path(tmpdir) / "test-keys.json"
        
        ledger = ZKAuditLedger(
            ledger_path=ledger_path,
            key_path=key_path
        )
        
        await ledger.initialize()
        
        # Add test entries
        base_time = datetime.utcnow()
        
        await ledger.add_entry(
            action="TRADE",
            agent="AGENT_A",
            reasoning="Test trade",
            inputs={"symbol": "AAPL"},
            outputs={"status": "FILLED"}
        )
        
        await asyncio.sleep(0.01)  # Small delay
        
        await ledger.add_entry(
            action="RISK_CHECK",
            agent="AGENT_B",
            reasoning="Risk assessment",
            inputs={"position": 100000},
            outputs={"risk": "LOW"}
        )
        
        # Query by agent
        agent_entries = await ledger.get_entries(agent="AGENT_A")
        assert len(agent_entries) == 1, "Should have 1 entry for AGENT_A"
        assert agent_entries[0].action == "TRADE", "Should be trade entry"
        
        # Query by action
        risk_entries = await ledger.get_entries(action="RISK_CHECK")
        assert len(risk_entries) == 1, "Should have 1 risk check entry"
        assert risk_entries[0].agent == "AGENT_B", "Should be from AGENT_B"
        
        # Query with limit
        limited_entries = await ledger.get_entries(limit=1)
        assert len(limited_entries) == 1, "Should limit to 1 entry"
        
        print("✓ Query entries test passed")
        

async def test_compliance_report():
    """Test EU AI Act compliance report generation."""
    print("Testing compliance report...")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        ledger_path = Path(tmpdir) / "test-ledger.log"
        key_path = Path(tmpdir) / "test-keys.json"
        
        ledger = ZKAuditLedger(
            ledger_path=ledger_path,
            key_path=key_path
        )
        
        await ledger.initialize()
        
        # Add entries for report period
        start_date = datetime.utcnow() - timedelta(hours=1)  # 1 hour ago
        end_date = datetime.utcnow() + timedelta(hours=1)   # 1 hour from now
        
        await ledger.add_entry(
            action="EXECUTE_TRADE",
            agent="DeltaLag-SGY",
            reasoning="AI-driven trade execution",
            inputs={"symbol": "DAX"},
            outputs={"executed": True}
        )
        
        await ledger.add_entry(
            action="RISK_VALIDATION",
            agent="RiskManager",
            reasoning="Human oversight check",
            inputs={"trade_id": "123"},
            outputs={"approved": True}
        )
        
        # Generate compliance report
        report = await ledger.get_compliance_report(start_date, end_date)
        
        # Verify report structure
        assert 'report_period' in report, "Missing report period"
        assert 'total_entries' in report, "Missing total entries"
        assert 'chain_integrity' in report, "Missing chain integrity"
        assert 'compliance' in report, "Missing compliance section"
        
        # Verify compliance claims
        assert report['compliance']['eu_ai_act_article_12'], "Should be EU AI Act compliant"
        assert report['compliance']['human_oversight'], "Should have human oversight"
        assert report['compliance']['transparency'], "Should be transparent"
        assert report['compliance']['explainability'], "Should be explainable"
        
        # Verify action summary
        print(f"Debug - Actions in report: {list(report['actions_summary'].keys())}")
        assert any('TRADE' in action for action in report['actions_summary']), "Should track trades"
        assert 'RISK_VALIDATION' in report['actions_summary'], "Should track risk checks"
        
        print("✓ Compliance report test passed")
        

async def test_export_functionality():
    """Test ledger export functionality."""
    print("Testing export functionality...")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        ledger_path = Path(tmpdir) / "test-ledger.log"
        key_path = Path(tmpdir) / "test-keys.json"
        
        ledger = ZKAuditLedger(
            ledger_path=ledger_path,
            key_path=key_path
        )
        
        await ledger.initialize()
        
        # Add test entry
        await ledger.add_entry(
            action="EXPORT_TEST",
            agent="TEST_AGENT",
            reasoning="Testing export",
            inputs={"format": "test"},
            outputs={"exported": True}
        )
        
        # Test JSONL export
        jsonl_path = Path(tmpdir) / "export.jsonl"
        success = await ledger.export_chain(jsonl_path, format='jsonl')
        assert success, "JSONL export should succeed"
        assert jsonl_path.exists(), "JSONL file should exist"
        
        # Verify exported content
        with open(jsonl_path, 'r') as f:
            lines = f.readlines()
        assert len(lines) == 1, "Should have 1 exported line"
        
        exported_entry = json.loads(lines[0])
        assert exported_entry['action'] == "EXPORT_TEST", "Exported content should match"
        
        # Test JSON export
        json_path = Path(tmpdir) / "export.json"
        success = await ledger.export_chain(json_path, format='json')
        assert success, "JSON export should succeed"
        
        with open(json_path, 'r') as f:
            data = json.load(f)
        assert len(data) == 1, "Should have 1 exported entry"
        
        print("✓ Export functionality test passed")
        

async def test_merkle_chain_property():
    """Test Merkle chain property: H(n) = SHA256(Action_n + H(n-1))"""
    print("Testing Merkle chain property...")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        ledger_path = Path(tmpdir) / "test-ledger.log"
        key_path = Path(tmpdir) / "test-keys.json"
        
        ledger = ZKAuditLedger(
            ledger_path=ledger_path,
            key_path=key_path
        )
        
        await ledger.initialize()
        
        # Add first entry
        hash1 = await ledger.add_entry(
            action="ENTRY_1",
            agent="AGENT_1",
            reasoning="First entry",
            inputs={},
            outputs={}
        )
        
        # Add second entry
        hash2 = await ledger.add_entry(
            action="ENTRY_2",
            agent="AGENT_2",
            reasoning="Second entry",
            inputs={},
            outputs={}
        )
        
        # Verify Merkle property
        entries = ledger.chain
        assert len(entries) == 2, "Should have 2 entries"
        
        # First entry should have no previous hash
        assert entries[0].prev_hash is None, "First entry prev_hash should be None"
        
        # Second entry should reference first
        assert entries[1].prev_hash == hash1, "Second entry should reference first"
        
        # Verify hash calculation
        import hashlib
        
        entry_data = {
            'timestamp': entries[1].timestamp,
            'action': entries[1].action,
            'agent': entries[1].agent,
            'reasoning': entries[1].reasoning,
            'inputs': entries[1].inputs,
            'outputs': entries[1].outputs,
            'prev_hash': entries[1].prev_hash
        }
        
        calculated_hash = hashlib.sha256(
            json.dumps(entry_data, sort_keys=True).encode()
        ).hexdigest()
        
        assert calculated_hash == hash2, "Hash should match Merkle property"
        
        print("✓ Merkle chain property test passed")
        

async def main():
    """Run all ZK-Audit tests."""
    print("Starting ZK-Audit Flight Recorder Tests...\n")
    
    tests = [
        test_ledger_initialization,
        test_add_entry,
        test_chain_verification,
        test_entry_persistence,
        test_query_entries,
        test_compliance_report,
        test_export_functionality,
        test_merkle_chain_property
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            await test()
            passed += 1
        except Exception as e:
            print(f"✗ Test failed: {test.__name__} - {e}")
            failed += 1
        print()
    
    print(f"ZK-Audit Test Results:")
    print(f"✓ Passed: {passed}")
    print(f"✗ Failed: {failed}")
    
    if failed == 0:
        print("\n🎉 All ZK-Audit tests passed!")
        return True
    else:
        print(f"\n❌ {failed} tests failed!")
        return False


if __name__ == "__main__":
    success = asyncio.run(main())
    sys.exit(0 if success else 1)

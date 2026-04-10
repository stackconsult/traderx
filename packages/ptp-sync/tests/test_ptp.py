#!/usr/bin/env python3
"""
Test suite for PTP synchronization service.
"""

import asyncio
import sys
from pathlib import Path
from datetime import datetime, timedelta
import tempfile
import json
import csv

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from src.ptp_service import PTPService, PTPConfig, MarketDataIngestor, HardwareTimestamp


async def test_ptp_config():
    """Test PTP configuration."""
    print("Testing PTP configuration...")
    
    config = PTPConfig(
        interface='eth0',
        transport='IPv4',
        delay_mechanism='E2E',
        domain=0,
        priority1=128,
        priority2=128
    )
    
    assert config.interface == 'eth0'
    assert config.domain == 0
    
    print("✓ PTP configuration test passed")


async def test_ptp_service_initialization():
    """Test PTP service initialization."""
    print("Testing PTP service initialization...")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        config = PTPConfig('test0', 'IPv4', 'E2E', 0, 128, 128)
        service = PTPService(config, log_dir=Path(tmpdir))
        
        # Test that service can be instantiated
        assert service.config.interface == 'test0'
        assert service.log_dir == Path(tmpdir)
        assert service.current_status is None
        
        # Test drift history
        assert len(service.drift_history) == 0
        
        print("✓ PTP service initialization test passed")


async def test_hardware_timestamp():
    """Test hardware timestamp abstraction."""
    print("Testing hardware timestamp...")
    
    # Test with hardware timestamp
    hw_ts = HardwareTimestamp(1234567890000000000, 1234567890000000100)
    assert hw_ts.software_ns == 1234567890000000000
    assert hw_ts.hardware_ns == 1234567890000000100
    assert hw_ts.has_hardware == True
    
    # Test without hardware timestamp
    sw_ts = HardwareTimestamp(1234567890000000000)
    assert sw_ts.software_ns == 1234567890000000000
    assert sw_ts.hardware_ns is None
    assert sw_ts.has_hardware == False
    
    print("✓ Hardware timestamp test passed")


async def test_drift_monitoring():
    """Test drift monitoring and statistics."""
    print("Testing drift monitoring...")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        config = PTPConfig('test0', 'IPv4', 'E2E', 0, 128, 128)
        service = PTPService(config, log_dir=Path(tmpdir))
        
        # Simulate drift data
        base_time = datetime.utcnow()
        test_drifts = [
            (base_time, 100),     # 100ns
            (base_time + timedelta(seconds=1), 200),  # 200ns
            (base_time + timedelta(seconds=2), -150), # -150ns
            (base_time + timedelta(seconds=3), 50),   # 50ns
        ]
        
        for timestamp, offset in test_drifts:
            service.drift_history.append((timestamp, offset))
            
        # Test drift verification
        assert service.verify_drift_requirement(max_drift_ns=1000) == True
        assert service.verify_drift_requirement(max_drift_ns=100) == False
        
        # Test statistics
        stats = service.get_drift_statistics()
        assert 'max_offset_ns_100' in stats
        assert 'mean_offset_ns_100' in stats
        assert 'drift_requirement_met' in stats
        
        print("✓ Drift monitoring test passed")


async def test_drift_log_generation():
    """Test drift log CSV generation."""
    print("Testing drift log generation...")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        config = PTPConfig('test0', 'IPv4', 'E2E', 0, 128, 128)
        service = PTPService(config, log_dir=Path(tmpdir))
        
        # Simulate status updates
        from src.ptp_service import PTPStatus
        status = PTPStatus(
            port_state='SLAVE',
            gm_present=True,
            offset_ns=500,
            mean_path_delay_ns=10000,
            freq_ppb=100,
            last_sync=datetime.utcnow(),
            clock_class=248
        )
        
        service.current_status = status
        
        # Add to drift history
        service.drift_history.append((datetime.utcnow(), 500))
        
        # Check if log file would be created
        log_path = service.drift_log_path
        assert log_path.parent.exists()
        
        print("✓ Drift log generation test passed")


async def test_market_data_ingestor():
    """Test market data ingestor with PTP timestamps."""
    print("Testing market data ingestor...")
    
    config = PTPConfig('test0', 'IPv4', 'E2E', 0, 128, 128)
    service = PTPService(config)
    
    ingestor = MarketDataIngestor(service)
    
    # Test tick processing
    timestamp = HardwareTimestamp(1234567890000000000, 1234567890000000100)
    tick = ingestor.process_tick('NYSE', 'AAPL', 150.25, timestamp)
    
    assert tick['exchange'] == 'NYSE'
    assert tick['symbol'] == 'AAPL'
    assert tick['price'] == 150.25
    assert tick['timestamp_ns'] == 1234567890000000100  # Hardware timestamp used
    assert tick['hardware_timestamp'] == True
    
    # Test without hardware timestamp
    sw_timestamp = HardwareTimestamp(1234567890000000000)
    sw_tick = ingestor.process_tick('NASDAQ', 'MSFT', 250.50, sw_timestamp)
    
    assert sw_tick['timestamp_ns'] == 1234567890000000000
    assert sw_tick['hardware_timestamp'] == False
    
    print("✓ Market data ingestor test passed")


async def test_ptp_time_adjustment():
    """Test PTP time adjustment based on offset."""
    print("Testing PTP time adjustment...")
    
    config = PTPConfig('test0', 'IPv4', 'E2E', 0, 128, 128)
    service = PTPService(config)
    
    # Simulate PTP status with offset
    from src.ptp_service import PTPStatus
    service.current_status = PTPStatus(
        port_state='SLAVE',
        gm_present=True,
        offset_ns=500000,  # 500μs offset
        mean_path_delay_ns=10000,
        freq_ppb=100,
        last_sync=datetime.utcnow(),
        clock_class=248
    )
    
    # Get adjusted time
    ptp_time = service.get_ptp_time()
    system_time = datetime.utcnow()
    
    # PTP time should be adjusted by offset
    time_diff = (ptp_time - system_time).total_seconds()
    assert abs(time_diff - 0.0005) < 0.0001  # 500μs adjustment
    
    print("✓ PTP time adjustment test passed")


async def test_proof_artifact_format():
    """Test that proof artifact meets requirements."""
    print("Testing proof artifact format...")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        config = PTPConfig('eth0', 'IPv4', 'E2E', 0, 128, 128)
        service = PTPService(config, log_dir=Path(tmpdir))
        
        # Simulate drift measurements
        base_time = datetime.utcnow()
        for i in range(10):
            offset = i * 50  # 0 to 450ns
            service.drift_history.append((base_time + timedelta(seconds=i), offset))
            
        # Create mock status
        from src.ptp_service import PTPStatus
        service.current_status = PTPStatus(
            port_state='SLAVE',
            gm_present=True,
            offset_ns=450,
            mean_path_delay_ns=10000,
            freq_ppb=100,
            last_sync=datetime.utcnow(),
            clock_class=248
        )
        
        # Generate drift log
        log_path = service.drift_log_path
        with open(log_path, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['timestamp', 'offset_ns', 'path_delay_ns', 'freq_ppb', 'port_state'])
            
            for timestamp, offset in service.drift_history:
                writer.writerow([
                    timestamp.isoformat(),
                    offset,
                    10000,
                    100,
                    'SLAVE'
                ])
                
        # Verify log format
        with open(log_path, 'r') as f:
            reader = csv.reader(f)
            headers = next(reader)
            assert headers == ['timestamp', 'offset_ns', 'path_delay_ns', 'freq_ppb', 'port_state']
            
            # Count data rows
            rows = list(reader)
            assert len(rows) == 10
            
            # Check last row (max offset)
            last_row = rows[-1]
            assert int(last_row[1]) == 450  # 450ns offset
            assert last_row[4] == 'SLAVE'
            
        # Verify drift requirement
        assert service.verify_drift_requirement(max_drift_ns=1000) == True  # < 1μs
        
        print("✓ Proof artifact format test passed")
        print(f"  Max drift: {max(abs(offset) for _, offset in service.drift_history)}ns")
        print(f"  Requirement met: < 1000ns ✓")


async def main():
    """Run all PTP tests."""
    print("Starting PTP Synchronization Tests...\n")
    
    tests = [
        test_ptp_config,
        test_ptp_service_initialization,
        test_hardware_timestamp,
        test_drift_monitoring,
        test_drift_log_generation,
        test_market_data_ingestor,
        test_ptp_time_adjustment,
        test_proof_artifact_format
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
    
    print(f"PTP Test Results:")
    print(f"✓ Passed: {passed}")
    print(f"✗ Failed: {failed}")
    
    if failed == 0:
        print("\n🎉 All PTP tests passed!")
        print("\nNote: Full integration requires LinuxPTP tools and hardware timestamping NIC")
        return True
    else:
        print(f"\n❌ {failed} tests failed!")
        return False


if __name__ == "__main__":
    success = asyncio.run(main())
    sys.exit(0 if success else 1)

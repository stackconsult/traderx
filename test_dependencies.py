#!/usr/bin/env python3
"""
Test script to verify all dependencies are installed and working
"""

import sys
import subprocess

def test_python_dependencies():
    """Test Python package imports"""
    print("Testing Python dependencies...")
    
    required_packages = [
        'numpy',
        'pandas',
        'asyncio',
        'aiohttp',
        'websockets',
        'structlog',
        'pydantic',
    ]
    
    optional_packages = [
        'databento',
        'polygon',
        'quickfix',
        'pyarrow',
    ]
    
    # Test required packages
    for package in required_packages:
        try:
            __import__(package)
            print(f"✅ {package}")
        except ImportError as e:
            print(f"❌ {package}: {e}")
            return False
    
    # Test optional packages
    for package in optional_packages:
        try:
            __import__(package)
            print(f"✅ {package} (optional)")
        except ImportError:
            print(f"⚠️ {package} (optional - not installed)")
    
    return True

def test_rust_dependencies():
    """Test Rust dependencies via cargo check"""
    print("\nTesting Rust dependencies...")
    
    packages = [
        'packages/oms-engine',
        'packages/dealing-desk/ebpf-router',
    ]
    
    for package in packages:
        print(f"\nChecking {package}...")
        result = subprocess.run(
            ['cargo', 'check'],
            cwd=package,
            capture_output=True,
            text=True
        )
        
        if result.returncode == 0:
            print(f"✅ {package} compiles successfully")
        else:
            print(f"❌ {package} compilation failed:")
            print(result.stderr)
            return False
    
    return True

def main():
    """Run all dependency tests"""
    print("TraderX Dependency Test Suite")
    print("=" * 40)
    
    python_ok = test_python_dependencies()
    rust_ok = test_rust_dependencies()
    
    print("\n" + "=" * 40)
    if python_ok and rust_ok:
        print("✅ All dependencies satisfied!")
        return 0
    else:
        print("❌ Some dependencies missing or broken")
        return 1

if __name__ == "__main__":
    sys.exit(main())

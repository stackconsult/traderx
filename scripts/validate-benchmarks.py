#!/usr/bin/env python3
"""
Benchmark Validation Script for TraderX OMS
Validates performance benchmarks against strict HFT thresholds
"""

import sys
import re
import json
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass

@dataclass
class Thresholds:
    """Performance thresholds for HFT system"""
    latency_p99_us: float = 100.0
    latency_p50_us: float = 10.0
    latency_p95_us: float = 50.0
    throughput_ops_per_sec: int = 100000
    memory_usage_mb: int = 8192
    cpu_usage_percent: int = 80

@dataclass
class BenchmarkResult:
    """Parsed benchmark result"""
    name: str
    p99_us: Optional[float] = None
    p95_us: Optional[float] = None
    p50_us: Optional[float] = None
    throughput: Optional[int] = None
    unit: str = "µs"

class BenchmarkValidator:
    """Validates benchmark results against HFT thresholds"""
    
    def __init__(self, thresholds: Optional[Thresholds] = None):
        self.thresholds = thresholds or Thresholds()
        self.results: List[BenchmarkResult] = []
        self.failures: List[str] = []
        
    def parse_benchmark_output(self, content: str) -> List[BenchmarkResult]:
        """Parse cargo bench output with multiple pattern support"""
        results = []
        
        # Split by benchmark sections
        sections = re.split(r'\n(?=test\s+\w+\s+\.\.\.?)', content)
        
        for section in sections:
            if not section.strip():
                continue
                
            # Try to extract benchmark name
            name_match = re.search(r'test\s+(\w+)', section)
            if not name_match:
                continue
                
            name = name_match.group(1)
            result = BenchmarkResult(name=name)
            
            # Parse latency percentiles with multiple patterns
            patterns = [
                (r'p99:\s*([\d.,]+)\s*µs', 'p99_us'),
                (r'p99:\s*([\d.,]+)\s*us', 'p99_us'),
                (r'99th percentile:\s*([\d.,]+)\s*µs', 'p99_us'),
                (r'99th percentile:\s*([\d.,]+)\s*us', 'p99_us'),
                
                (r'p95:\s*([\d.,]+)\s*µs', 'p95_us'),
                (r'p95:\s*([\d.,]+)\s*us', 'p95_us'),
                (r'95th percentile:\s*([\d.,]+)\s*µs', 'p95_us'),
                
                (r'p50:\s*([\d.,]+)\s*µs', 'p50_us'),
                (r'p50:\s*([\d.,]+)\s*us', 'p50_us'),
                (r'50th percentile:\s*([\d.,]+)\s*µs', 'p50_us'),
                (r'median:\s*([\d.,]+)\s*µs', 'p50_us'),
            ]
            
            for pattern, field in patterns:
                match = re.search(pattern, section, re.IGNORECASE)
                if match:
                    value_str = match.group(1).replace(',', '')
                    try:
                        value = float(value_str)
                        setattr(result, field, value)
                    except ValueError:
                        pass
            
            # Parse throughput
            throughput_patterns = [
                r'(\d+)\s+ops/sec',
                r'throughput:\s*(\d+)\s+ops',
                r'(\d+)\s+operations/second',
            ]
            
            for pattern in throughput_patterns:
                match = re.search(pattern, section, re.IGNORECASE)
                if match:
                    try:
                        result.throughput = int(match.group(1))
                        break
                    except ValueError:
                        pass
            
            # Determine unit
            if 'ns' in section.lower():
                result.unit = "ns"
            elif 'ms' in section.lower():
                result.unit = "ms"
            else:
                result.unit = "µs"
            
            results.append(result)
        
        self.results = results
        return results
    
    def validate_result(self, result: BenchmarkResult) -> bool:
        """Validate a single benchmark result"""
        is_valid = True
        
        # Convert to microseconds if needed
        unit_multiplier = 1.0
        if result.unit == "ns":
            unit_multiplier = 0.001
        elif result.unit == "ms":
            unit_multiplier = 1000.0
        
        # Validate p99 latency
        if result.p99_us is not None:
            p99_converted = result.p99_us * unit_multiplier
            if p99_converted > self.thresholds.latency_p99_us:
                self.failures.append(
                    f"{result.name}: p99 latency {p99_converted:.2f}µs exceeds threshold "
                    f"{self.thresholds.latency_p99_us}µs"
                )
                is_valid = False
        
        # Validate p50 latency
        if result.p50_us is not None:
            p50_converted = result.p50_us * unit_multiplier
            if p50_converted > self.thresholds.latency_p50_us:
                self.failures.append(
                    f"{result.name}: p50 latency {p50_converted:.2f}µs exceeds threshold "
                    f"{self.thresholds.latency_p50_us}µs"
                )
                is_valid = False
        
        # Validate p95 latency
        if result.p95_us is not None:
            p95_converted = result.p95_us * unit_multiplier
            if p95_converted > self.thresholds.latency_p95_us:
                self.failures.append(
                    f"{result.name}: p95 latency {p95_converted:.2f}µs exceeds threshold "
                    f"{self.thresholds.latency_p95_us}µs"
                )
                is_valid = False
        
        # Validate throughput
        if result.throughput is not None:
            if result.throughput < self.thresholds.throughput_ops_per_sec:
                self.failures.append(
                    f"{result.name}: throughput {result.throughput} ops/sec below threshold "
                    f"{self.thresholds.throughput_ops_per_sec} ops/sec"
                )
                is_valid = False
        
        return is_valid
    
    def validate_all(self) -> Tuple[bool, List[str]]:
        """Validate all benchmark results"""
        if not self.results:
            self.failures.append("No benchmark results found")
            return False, self.failures
        
        all_valid = True
        for result in self.results:
            if not self.validate_result(result):
                all_valid = False
        
        return all_valid, self.failures
    
    def generate_report(self) -> Dict:
        """Generate validation report"""
        report = {
            "thresholds": {
                "latency_p99_us": self.thresholds.latency_p99_us,
                "latency_p50_us": self.thresholds.latency_p50_us,
                "latency_p95_us": self.thresholds.latency_p95_us,
                "throughput_ops_per_sec": self.thresholds.throughput_ops_per_sec,
            },
            "results": [
                {
                    "name": r.name,
                    "p99_us": r.p99_us,
                    "p95_us": r.p95_us,
                    "p50_us": r.p50_us,
                    "throughput": r.throughput,
                    "unit": r.unit,
                }
                for r in self.results
            ],
            "valid": len(self.failures) == 0,
            "failures": self.failures,
        }
        return report
    
    def save_report(self, output_path: str):
        """Save report to file"""
        report = self.generate_report()
        with open(output_path, 'w') as f:
            json.dump(report, f, indent=2)
        print(f"Report saved to {output_path}")


def main():
    if len(sys.argv) < 2:
        print("Usage: python validate-benchmarks.py <benchmark-output-file> [thresholds.json]")
        sys.exit(1)
    
    input_file = sys.argv[1]
    
    # Load custom thresholds if provided
    thresholds = None
    if len(sys.argv) > 2:
        thresholds_file = sys.argv[2]
        try:
            with open(thresholds_file, 'r') as f:
                threshold_data = json.load(f)
                thresholds = Thresholds(**threshold_data)
        except (FileNotFoundError, json.JSONDecodeError, TypeError) as e:
            print(f"Warning: Could not load thresholds from {thresholds_file}: {e}")
            print("Using default thresholds")
    
    # Read benchmark output
    try:
        with open(input_file, 'r') as f:
            content = f.read()
    except FileNotFoundError:
        print(f"Error: File not found: {input_file}")
        sys.exit(1)
    except IOError as e:
        print(f"Error reading file: {e}")
        sys.exit(1)
    
    # Validate
    validator = BenchmarkValidator(thresholds)
    validator.parse_benchmark_output(content)
    is_valid, failures = validator.validate_all()
    
    # Print results
    print("=" * 80)
    print("BENCHMARK VALIDATION RESULTS")
    print("=" * 80)
    
    print(f"\nThresholds:")
    print(f"  p99 latency: {validator.thresholds.latency_p99_us}µs")
    print(f"  p50 latency: {validator.thresholds.latency_p50_us}µs")
    print(f"  p95 latency: {validator.thresholds.latency_p95_us}µs")
    print(f"  throughput: {validator.thresholds.throughput_ops_per_sec} ops/sec")
    
    print(f"\nParsed {len(validator.results)} benchmark results:")
    for result in validator.results:
        print(f"\n  {result.name}:")
        if result.p99_us:
            print(f"    p99: {result.p99_us}{result.unit}")
        if result.p95_us:
            print(f"    p95: {result.p95_us}{result.unit}")
        if result.p50_us:
            print(f"    p50: {result.p50_us}{result.unit}")
        if result.throughput:
            print(f"    throughput: {result.throughput} ops/sec")
    
    if failures:
        print(f"\n❌ VALIDATION FAILED - {len(failures)} issues:")
        for failure in failures:
            print(f"  - {failure}")
    else:
        print("\n✅ ALL BENCHMARKS PASSED")
    
    # Save report
    report_path = input_file.replace('.txt', '-report.json')
    validator.save_report(report_path)
    
    # Exit with appropriate code
    sys.exit(0 if is_valid else 1)


if __name__ == "__main__":
    main()

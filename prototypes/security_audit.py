#!/usr/bin/env python3
"""
Security Audit Prototype for TraderX Critical Components
Validates security requirements and identifies vulnerabilities
"""

import subprocess
import json
import re
import os
import tempfile
import shutil
from pathlib import Path
from typing import List, Dict, Tuple
import yaml

class SecurityAuditor:
    def __init__(self, repo_root: str):
        self.repo_root = Path(repo_root)
        self.findings: List[Dict] = []
        
    def audit_rust_dependencies(self) -> List[Dict]:
        """Audit Rust dependencies for known vulnerabilities"""
        print("\n=== Auditing Rust Dependencies ===")
        
        # Find all Cargo.toml files
        cargo_files = list(self.repo_root.glob("**/Cargo.toml"))
        
        for cargo_file in cargo_files:
            print(f"\nAuditing {cargo_file.relative_to(self.repo_root)}")
            
            # Run cargo audit
            result = subprocess.run(
                ["cargo", "audit", "--json"],
                cwd=cargo_file.parent,
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                data = json.loads(result.stdout)
                if "vulnerabilities" in data and data["vulnerabilities"]["count"] > 0:
                    for vuln in data["vulnerabilities"]["list"]:
                        self.findings.append({
                            "type": "rust_dependency",
                            "file": str(cargo_file.relative_to(self.repo_root)),
                            "package": vuln["package"]["name"],
                            "version": vuln["package"]["version"],
                            "severity": vuln["advisory"]["severity"],
                            "description": vuln["advisory"]["description"],
                            "patched_versions": vuln["advisory"].get("patched_versions", []),
                            "unaffected_versions": vuln["advisory"].get("unaffected_versions", [])
                        })
                        print(f"  ❌ {vuln['package']['name']} v{vuln['package']['version']} - {vuln['advisory']['severity']}")
                else:
                    print("  ✅ No vulnerabilities found")
            else:
                print(f"  ⚠️  Cargo audit failed: {result.stderr}")
    
    def audit_python_dependencies(self) -> List[Dict]:
        """Audit Python dependencies for known vulnerabilities"""
        print("\n=== Auditing Python Dependencies ===")
        
        # Find all pyproject.toml and requirements.txt files
        pyproject_files = list(self.repo_root.glob("**/pyproject.toml"))
        requirements_files = list(self.repo_root.glob("**/requirements.txt"))
        
        for pyproject_file in pyproject_files:
            print(f"\nAuditing {pyproject_file.relative_to(self.repo_root)}")
            
            # Run pip-audit
            result = subprocess.run(
                ["pip-audit", "--requirement", str(pyproject_file), "--format", "json"],
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                data = json.loads(result.stdout)
                if data.get("vulnerabilities"):
                    for vuln in data["vulnerabilities"]:
                        self.findings.append({
                            "type": "python_dependency",
                            "file": str(pyproject_file.relative_to(self.repo_root)),
                            "package": vuln["name"],
                            "version": vuln["version"],
                            "severity": vuln["fix_versions"][0] if vuln.get("fix_versions") else "unknown",
                            "description": vuln.get("description", ""),
                            "fix_versions": vuln.get("fix_versions", [])
                        })
                        print(f"  ❌ {vuln['name']} v{vuln['version']}")
                else:
                    print("  ✅ No vulnerabilities found")
            else:
                print(f"  ⚠️  Pip-audit failed: {result.stderr}")
    
    def audit_unix_socket_security(self) -> List[Dict]:
        """Audit Unix socket security in Signal Router"""
        print("\n=== Auditing Unix Socket Security ===")
        
        signal_router_path = self.repo_root / "packages/oms-engine/src/signal_router.rs"
        
        if not signal_router_path.exists():
            print("  ⚠️  Signal router not found")
            return []
        
        content = signal_router_path.read_text()
        
        # Check for socket permissions
        if "0o600" not in content and "chmod" not in content:
            self.findings.append({
                "type": "socket_security",
                "file": "packages/oms-engine/src/signal_router.rs",
                "issue": "Unix socket permissions not explicitly set to 600",
                "severity": "high",
                "recommendation": "Set socket permissions to 0o600 to restrict access"
            })
            print("  ❌ Unix socket permissions not set")
        else:
            print("  ✅ Socket permissions configured")
        
        # Check for authentication mechanism
        if "authenticate" not in content.lower() and "verify" not in content.lower():
            self.findings.append({
                "type": "socket_authentication",
                "file": "packages/oms-engine/src/signal_router.rs",
                "issue": "No authentication mechanism for signal router",
                "severity": "medium",
                "recommendation": "Implement authentication for remote connections"
            })
            print("  ⚠️  No authentication mechanism found")
        
        # Check for input validation
        if "validate" not in content.lower():
            self.findings.append({
                "type": "input_validation",
                "file": "packages/oms-engine/src/signal_router.rs",
                "issue": "Signal input validation not implemented",
                "severity": "high",
                "recommendation": "Validate all incoming signal fields"
            })
            print("  ❌ Input validation missing")
        else:
            print("  ✅ Input validation present")
    
    def audit_atomic_operations(self) -> List[Dict]:
        """Audit atomic operations in Risk Bus"""
        print("\n=== Auditing Atomic Operations ===")
        
        risk_bus_path = self.repo_root / "packages/oms-engine/src/risk_bus.rs"
        
        if not risk_bus_path.exists():
            print("  ⚠️  Risk bus not found")
            return []
        
        content = risk_bus_path.read_text()
        
        # Check for proper atomic ordering
        if "Ordering::SeqCst" not in content:
            self.findings.append({
                "type": "atomic_ordering",
                "file": "packages/oms-engine/src/risk_bus.rs",
                "issue": "Critical atomic operations not using SeqCst ordering",
                "severity": "critical",
                "recommendation": "Use Ordering::SeqCst for halt and kill_switch operations"
            })
            print("  ❌ SeqCst ordering not used for critical operations")
        else:
            print("  ✅ Proper atomic ordering found")
        
        # Check for integer overflow protection
        if "checked_add" not in content and "saturating_add" not in content:
            self.findings.append({
                "type": "integer_overflow",
                "file": "packages/oms-engine/src/risk_bus.rs",
                "issue": "No integer overflow protection in arithmetic operations",
                "severity": "medium",
                "recommendation": "Use checked_add or saturating_add for fixed-point arithmetic"
            })
            print("  ⚠️  No overflow protection")
        else:
            print("  ✅ Overflow protection present")
    
    def audit_sql_injection(self) -> List[Dict]:
        """Audit SQL injection vulnerabilities"""
        print("\n=== Auditing SQL Injection ===")
        
        sql_files = list(self.repo_root.glob("**/*.sql")) + \
                   list(self.repo_root.glob("**/*.py")) + \
                   list(self.repo_root.glob("**/*.rs"))
        
        for file_path in sql_files:
            content = file_path.read_text()
            
            # Check for string concatenation in SQL
            if "format!(" in content and "SELECT" in content or "INSERT" in content:
                # Check if it's parameterized
                if "$" not in content and "?" not in content:
                    self.findings.append({
                        "type": "sql_injection",
                        "file": str(file_path.relative_to(self.repo_root)),
                        "issue": "Potential SQL injection via string formatting",
                        "severity": "critical",
                        "recommendation": "Use parameterized queries"
                    })
                    print(f"  ❌ {file_path.relative_to(self.repo_root)} - Potential SQL injection")
    
    def audit_redis_security(self) -> List[Dict]:
        """Audit Redis security configuration"""
        print("\n=== Auditing Redis Security ===")
        
        # Check Redis configuration in feature store
        feature_store_path = self.repo_root / "packages/feature-store/src/online.py"
        
        if feature_store_path.exists():
            content = feature_store_path.read_text()
            
            # Check for authentication
            if "password" not in content.lower() and "auth" not in content.lower():
                self.findings.append({
                    "type": "redis_auth",
                    "file": "packages/feature-store/src/online.py",
                    "issue": "Redis connection without authentication",
                    "severity": "high",
                    "recommendation": "Configure Redis password authentication"
                })
                print("  ❌ Redis authentication not configured")
            else:
                print("  ✅ Redis authentication configured")
            
            # Check for TLS
            if "ssl" not in content.lower() and "tls" not in content.lower():
                self.findings.append({
                    "type": "redis_tls",
                    "file": "packages/feature-store/src/online.py",
                    "issue": "Redis connection without TLS",
                    "severity": "medium",
                    "recommendation": "Enable TLS for Redis connections"
                })
                print("  ⚠️  Redis TLS not enabled")
    
    def generate_security_report(self) -> str:
        """Generate comprehensive security report"""
        report = []
        report.append("# TraderX Security Audit Report\n")
        report.append(f"Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
        
        # Summary
        critical = len([f for f in self.findings if f["severity"] == "critical"])
        high = len([f for f in self.findings if f["severity"] == "high"])
        medium = len([f for f in self.findings if f["severity"] == "medium"])
        low = len([f for f in self.findings if f["severity"] == "low"])
        
        report.append("## Summary\n")
        report.append(f"- Critical: {critical}")
        report.append(f"- High: {high}")
        report.append(f"- Medium: {medium}")
        report.append(f"- Low: {low}")
        report.append(f"- Total: {len(self.findings)}\n")
        
        # Critical findings first
        report.append("## Critical Findings\n")
        for finding in [f for f in self.findings if f["severity"] == "critical"]:
            report.append(f"### {finding['type']}\n")
            report.append(f"**File:** {finding['file']}\n")
            report.append(f"**Issue:** {finding['issue']}\n")
            report.append(f"**Recommendation:** {finding['recommendation']}\n")
            report.append("---\n")
        
        # High findings
        report.append("## High Priority Findings\n")
        for finding in [f for f in self.findings if f["severity"] == "high"]:
            report.append(f"### {finding['type']}\n")
            report.append(f"**File:** {finding['file']}\n")
            report.append(f"**Issue:** {finding['issue']}\n")
            report.append(f"**Recommendation:** {finding['recommendation']}\n")
            report.append("---\n")
        
        # Remediation plan
        report.append("## Remediation Plan\n")
        report.append("1. **Immediate (Critical)**: Fix atomic ordering and SQL injection\n")
        report.append("2. **This Week (High)**: Configure authentication, fix socket permissions\n")
        report.append("3. **Next Week (Medium)**: Enable TLS, add overflow protection\n")
        report.append("4. **Ongoing**: Regular dependency updates and security scans\n")
        
        return "\n".join(report)
    
    def create_fixes(self) -> None:
        """Create automated fixes for common issues"""
        print("\n=== Creating Automated Fixes ===")
        
        # Fix Unix socket permissions
        signal_router = self.repo_root / "packages/oms-engine/src/signal_router.rs"
        if signal_router.exists():
            content = signal_router.read_text()
            if "0o600" not in content:
                # Add socket permission setting
                content = content.replace(
                    "let socket = UnixListener::bind(socket_path)?;",
                    """let socket = UnixListener::bind(socket_path)?;
                    // Set secure permissions (owner read/write only)
                    std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o600))?;"""
                )
                signal_router.write_text(content)
                print("  ✅ Fixed Unix socket permissions")
        
        # Create security requirements files
        security_req = """
# Security requirements for TraderX
bandit>=1.7.4
safety>=2.3.1
pip-audit>=2.4.0
"""
        (self.repo_root / "requirements-security.txt").write_text(security_req)
        print("  ✅ Created security requirements file")

def main():
    """Run security audit"""
    import time
    
    repo_root = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.dirname(repo_root)  # Go up to traderx root
    
    auditor = SecurityAuditor(repo_root)
    
    print("=== TraderX Security Audit ===")
    
    # Run all audits
    auditor.audit_rust_dependencies()
    auditor.audit_python_dependencies()
    auditor.audit_unix_socket_security()
    auditor.audit_atomic_operations()
    auditor.audit_sql_injection()
    auditor.audit_redis_security()
    
    # Generate report
    report = auditor.generate_security_report()
    
    # Save report
    report_path = Path(repo_root) / "SECURITY_AUDIT_REPORT.md"
    report_path.write_text(report)
    
    print(f"\n=== Security Audit Complete ===")
    print(f"Report saved to: {report_path}")
    print(f"Total findings: {len(auditor.findings)}")
    
    # Create fixes
    auditor.create_fixes()
    
    # Print critical issues
    critical = [f for f in auditor.findings if f["severity"] == "critical"]
    if critical:
        print("\n⚠️  CRITICAL ISSUES REQUIRING IMMEDIATE ATTENTION:")
        for issue in critical:
            print(f"  - {issue['issue']} in {issue['file']}")

if __name__ == "__main__":
    main()

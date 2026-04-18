#!/usr/bin/env python3
"""
Deployment Validation Script for TraderX OMS
Validates deployment packages and Kubernetes manifests
"""

import sys
import os
import json
import tarfile
import hashlib
import re
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
import subprocess

@dataclass
class ValidationResult:
    """Validation result with details"""
    name: str
    valid: bool
    message: str
    details: Optional[Dict] = None

class DeploymentValidator:
    """Validates deployment packages and configurations"""
    
    def __init__(self, artifact_dir: str = "artifacts", deploy_dir: str = "deployment"):
        self.artifact_dir = Path(artifact_dir)
        self.deploy_dir = Path(deploy_dir)
        self.results: List[ValidationResult] = []
        
    def validate_artifact_exists(self, version: str) -> ValidationResult:
        """Validate artifact directory exists"""
        artifact_path = self.artifact_dir / version
        
        if not artifact_path.exists():
            return ValidationResult(
                name="artifact_exists",
                valid=False,
                message=f"Artifact directory not found: {artifact_path}"
            )
        
        return ValidationResult(
            name="artifact_exists",
            valid=True,
            message=f"Artifact directory exists: {artifact_path}"
        )
    
    def validate_binary_exists(self, version: str) -> ValidationResult:
        """Validate binary file exists and is executable"""
        binary_path = self.artifact_dir / version / "oms-engine"
        
        if not binary_path.exists():
            return ValidationResult(
                name="binary_exists",
                valid=False,
                message=f"Binary not found: {binary_path}"
            )
        
        if not os.access(binary_path, os.X_OK):
            return ValidationResult(
                name="binary_executable",
                valid=False,
                message=f"Binary not executable: {binary_path}"
            )
        
        # Check size
        size = binary_path.stat().st_size
        max_size = 50 * 1024 * 1024  # 50MB
        
        if size > max_size:
            return ValidationResult(
                name="binary_size",
                valid=False,
                message=f"Binary too large: {size} bytes (max: {max_size})",
                details={"size": size, "max_size": max_size}
            )
        
        return ValidationResult(
            name="binary",
            valid=True,
            message=f"Binary valid: {size} bytes",
            details={"size": size}
        )
    
    def validate_checksums(self, version: str) -> ValidationResult:
        """Validate checksum files"""
        artifact_path = self.artifact_dir / version
        
        required_checksums = ["sha256", "sha512", "md5"]
        missing = []
        
        for checksum_type in required_checksums:
            checksum_file = artifact_path / f"oms-engine.{checksum_type}"
            if not checksum_file.exists():
                missing.append(checksum_type)
        
        if missing:
            return ValidationResult(
                name="checksums",
                valid=False,
                message=f"Missing checksum files: {', '.join(missing)}"
            )
        
        # Verify checksums are valid
        try:
            for checksum_type in required_checksums:
                checksum_file = artifact_path / f"oms-engine.{checksum_type}"
                with open(checksum_file, 'r') as f:
                    line = f.readline().strip()
                    if not re.match(r'^[a-f0-9]+\s+', line):
                        return ValidationResult(
                            name="checksum_format",
                            valid=False,
                            message=f"Invalid checksum format in {checksum_file}"
                        )
        except Exception as e:
            return ValidationResult(
                name="checksum_read",
                valid=False,
                message=f"Error reading checksums: {e}"
            )
        
        return ValidationResult(
            name="checksums",
            valid=True,
            message="All checksum files present and valid"
        )
    
    def verify_checksum(self, version: str, checksum_type: str = "sha256") -> ValidationResult:
        """Verify binary checksum matches"""
        artifact_path = self.artifact_dir / version
        binary_path = artifact_path / "oms-engine"
        checksum_file = artifact_path / f"oms-engine.{checksum_type}"
        
        if not binary_path.exists() or not checksum_file.exists():
            return ValidationResult(
                name=f"checksum_verify_{checksum_type}",
                valid=False,
                message="Binary or checksum file missing"
            )
        
        # Calculate actual checksum
        hash_func = {
            "sha256": hashlib.sha256,
            "sha512": hashlib.sha512,
            "md5": hashlib.md5
        }.get(checksum_type)
        
        if not hash_func:
            return ValidationResult(
                name=f"checksum_verify_{checksum_type}",
                valid=False,
                message=f"Unknown checksum type: {checksum_type}"
            )
        
        hasher = hash_func()
        with open(binary_path, 'rb') as f:
            for chunk in iter(lambda: f.read(8192), b''):
                hasher.update(chunk)
        
        actual_checksum = hasher.hexdigest()
        
        # Read expected checksum
        with open(checksum_file, 'r') as f:
            line = f.readline().strip()
            expected_checksum = line.split()[0]
        
        if actual_checksum != expected_checksum:
            return ValidationResult(
                name=f"checksum_verify_{checksum_type}",
                valid=False,
                message=f"Checksum mismatch: expected {expected_checksum}, got {actual_checksum}",
                details={
                    "expected": expected_checksum,
                    "actual": actual_checksum
                }
            )
        
        return ValidationResult(
            name=f"checksum_verify_{checksum_type}",
            valid=True,
            message=f"{checksum_type.upper()} checksum verified",
            details={"checksum": actual_checksum}
        )
    
    def validate_deployment_package(self, version: str) -> ValidationResult:
        """Validate deployment tarball"""
        package_path = self.deploy_dir / f"oms-engine-{version}.tar.gz"
        
        if not package_path.exists():
            return ValidationResult(
                name="deployment_package",
                valid=False,
                message=f"Deployment package not found: {package_path}"
            )
        
        # Check if it's a valid tar.gz
        try:
            with tarfile.open(package_path, 'r:gz') as tar:
                members = tar.getmembers()
                
                # Check for required files
                has_binary = any('oms-engine' in m.name for m in members)
                has_version = any('version.txt' in m.name for m in members)
                
                if not has_binary:
                    return ValidationResult(
                        name="deployment_package",
                        valid=False,
                        message="Package missing binary"
                    )
                
                if not has_version:
                    return ValidationResult(
                        name="deployment_package",
                        valid=False,
                        message="Package missing version.txt"
                    )
        except tarfile.TarError as e:
            return ValidationResult(
                name="deployment_package",
                valid=False,
                message=f"Invalid tar.gz file: {e}"
            )
        
        return ValidationResult(
            name="deployment_package",
            valid=True,
            message=f"Deployment package valid: {package_path.stat().st_size} bytes"
        )
    
    def validate_kubernetes_manifest(self, version: str) -> ValidationResult:
        """Validate Kubernetes manifest"""
        manifest_path = self.deploy_dir / f"oms-engine-{version}.yaml"
        
        if not manifest_path.exists():
            return ValidationResult(
                name="k8s_manifest",
                valid=False,
                message=f"Kubernetes manifest not found: {manifest_path}"
            )
        
        # Basic YAML validation
        try:
            with open(manifest_path, 'r') as f:
                content = f.read()
                
                # Check for required K8s fields
                required_fields = [
                    'apiVersion',
                    'kind',
                    'metadata',
                ]
                
                missing = [field for field in required_fields if field not in content]
                
                if missing:
                    return ValidationResult(
                        name="k8s_manifest",
                        valid=False,
                        message=f"Missing required K8s fields: {', '.join(missing)}"
                    )
                
                # Check for version placeholder replacement
                if 'VERSION_PLACEHOLDER' in content:
                    return ValidationResult(
                        name="k8s_manifest",
                        valid=False,
                        message="Version placeholder not replaced"
                    )
                
        except Exception as e:
            return ValidationResult(
                name="k8s_manifest_read",
                valid=False,
                message=f"Error reading manifest: {e}"
            )
        
        # Try kubeval if available
        try:
            result = subprocess.run(
                ['kubeval', str(manifest_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode != 0:
                return ValidationResult(
                    name="k8s_manifest_kubeval",
                    valid=False,
                    message=f"kubeval validation failed: {result.stderr}"
                )
        except (subprocess.SubprocessError, FileNotFoundError):
            # kubeval not available, skip
            pass
        
        return ValidationResult(
            name="k8s_manifest",
            valid=True,
            message="Kubernetes manifest valid"
        )
    
    def validate_sbom(self, version: str) -> ValidationResult:
        """Validate SBOM exists"""
        sbom_paths = [
            self.artifact_dir / version / "sbom.spdx.json",
            self.artifact_dir / version / "sbom-cargo.txt"
        ]
        
        for sbom_path in sbom_paths:
            if sbom_path.exists():
                return ValidationResult(
                    name="sbom",
                    valid=True,
                    message=f"SBOM found: {sbom_path.name}"
                )
        
        return ValidationResult(
            name="sbom",
            valid=False,
            message="SBOM not found"
        )
    
    def validate_version_consistency(self, version: str) -> ValidationResult:
        """Validate version consistency across all artifacts"""
        versions = []
        
        # Check version.txt
        version_file = self.artifact_dir / version / "version.txt"
        if version_file.exists():
            with open(version_file, 'r') as f:
                versions.append(("version.txt", f.read().strip()))
        
        # Check tarball version in path
        versions.append(("artifact_path", version))
        
        # Check all versions match
        unique_versions = set(v[1] for v in versions)
        
        if len(unique_versions) > 1:
            return ValidationResult(
                name="version_consistency",
                valid=False,
                message=f"Version mismatch: {versions}",
                details={"versions": versions}
            )
        
        return ValidationResult(
            name="version_consistency",
            valid=True,
            message=f"Version consistent: {list(unique_versions)[0]}"
        )
    
    def run_all_validations(self, version: str) -> Tuple[bool, List[ValidationResult]]:
        """Run all validation checks"""
        validations = [
            self.validate_artifact_exists(version),
            self.validate_binary_exists(version),
            self.validate_checksums(version),
            self.verify_checksum(version, "sha256"),
            self.verify_checksum(version, "sha512"),
            self.verify_checksum(version, "md5"),
            self.validate_deployment_package(version),
            self.validate_kubernetes_manifest(version),
            self.validate_sbom(version),
            self.validate_version_consistency(version),
        ]
        
        self.results = validations
        
        all_valid = all(r.valid for r in validations)
        return all_valid, validations
    
    def generate_report(self) -> Dict:
        """Generate validation report"""
        return {
            "validations": [
                {
                    "name": r.name,
                    "valid": r.valid,
                    "message": r.message,
                    "details": r.details
                }
                for r in self.results
            ],
            "summary": {
                "total": len(self.results),
                "passed": sum(1 for r in self.results if r.valid),
                "failed": sum(1 for r in self.results if not r.valid),
                "valid": all(r.valid for r in self.results)
            }
        }
    
    def save_report(self, output_path: str):
        """Save report to file"""
        report = self.generate_report()
        with open(output_path, 'w') as f:
            json.dump(report, f, indent=2)
        print(f"Report saved to {output_path}")


def main():
    if len(sys.argv) < 2:
        print("Usage: python validate-deployment.py <version> [artifact_dir] [deploy_dir]")
        sys.exit(1)
    
    version = sys.argv[1]
    artifact_dir = sys.argv[2] if len(sys.argv) > 2 else "artifacts"
    deploy_dir = sys.argv[3] if len(sys.argv) > 3 else "deployment"
    
    print(f"Validating deployment for version: {version}")
    print(f"Artifact directory: {artifact_dir}")
    print(f"Deployment directory: {deploy_dir}")
    print()
    
    validator = DeploymentValidator(artifact_dir, deploy_dir)
    is_valid, results = validator.run_all_validations(version)
    
    # Print results
    print("=" * 80)
    print("DEPLOYMENT VALIDATION RESULTS")
    print("=" * 80)
    
    for result in results:
        status = "✅" if result.valid else "❌"
        print(f"{status} {result.name}: {result.message}")
        if result.details:
            for key, value in result.details.items():
                print(f"   {key}: {value}")
    
    print()
    print("=" * 80)
    
    summary = validator.generate_report()["summary"]
    print(f"Total: {summary['total']} | Passed: {summary['passed']} | Failed: {summary['failed']}")
    
    if is_valid:
        print("✅ ALL VALIDATIONS PASSED - Ready for deployment")
    else:
        print("❌ VALIDATION FAILED - Fix issues before deploying")
    
    # Save report
    report_path = f"deployment-validation-{version}.json"
    validator.save_report(report_path)
    
    sys.exit(0 if is_valid else 1)


if __name__ == "__main__":
    main()

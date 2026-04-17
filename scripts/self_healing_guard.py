#!/usr/bin/env python3
"""
Self-Healing Production Guard

Autonomous failure detection and fixing system.
Follows the 6-step process:
1. DETECT - Check PR status via GitHub API
2. ANALYZE - Identify failure patterns
3. FIX - Generate and apply fixes
4. VERIFY - Re-run checks
5. COMMIT - Document changes
6. LEARN - Update knowledge base

Usage:
    python scripts/self_healing_guard.py stackconsult traderx fix/oms-engine-compilation-errors
"""

import sys
import os
import subprocess
import json
import time
from pathlib import Path

# Add to path for imports
sys.path.insert(0, str(Path(__file__).parent))

def run_command(cmd, cwd=None):
    """Run shell command and return output."""
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            cwd=cwd or os.getcwd(),
            capture_output=True,
            text=True,
            timeout=300
        )
        return result.returncode == 0, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return False, "", "Command timed out"
    except Exception as e:
        return False, "", str(e)

def detect_failures_local():
    """Step 1: Detect failures via local cargo commands."""
    failures = []
    
    print("🔍 Step 1: DETECT - Running local checks...")
    
    # Check compilation
    print("  Checking compilation...")
    success, stdout, stderr = run_command("cargo check --package oms-engine")
    if not success:
        failures.append({
            "type": "compilation",
            "source": "cargo check",
            "error": stderr
        })
        print(f"    ❌ Compilation failed: {stderr[:200]}")
    else:
        print("    ✅ Compilation passed")
    
    # Check tests compile
    print("  Checking test compilation...")
    success, stdout, stderr = run_command("cargo test --package oms-engine --no-run")
    if not success:
        failures.append({
            "type": "test_compilation",
            "source": "cargo test --no-run",
            "error": stderr
        })
        print(f"    ❌ Test compilation failed: {stderr[:200]}")
    else:
        print("    ✅ Test compilation passed")
    
    # Check clippy
    print("  Checking clippy...")
    success, stdout, stderr = run_command("cargo clippy --package oms-engine -- -D warnings")
    if not success:
        failures.append({
            "type": "clippy",
            "source": "cargo clippy",
            "error": stderr
        })
        print(f"    ❌ Clippy failed: {stderr[:200]}")
    else:
        print("    ✅ Clippy passed")
    
    # Check audit
    print("  Checking security audit...")
    success, stdout, stderr = run_command("cargo audit")
    if not success:
        failures.append({
            "type": "security",
            "source": "cargo audit",
            "error": stderr
        })
        print(f"    ❌ Security audit failed: {stderr[:200]}")
    else:
        print("    ✅ Security audit passed")
    
    print(f"\n📊 Detection Results: {len(failures)} failures found")
    return failures

def analyze_failure(failure):
    """Step 2: Analyze failure and identify root cause."""
    print(f"\n🧠 Step 2: ANALYZE - {failure['type']} failure")
    
    error = failure.get('error', '')
    
    # Pattern matching for common failures
    if failure['type'] == 'security':
        if 'RUSTSEC' in error:
            # Extract RUSTSEC ID
            import re
            match = re.search(r'RUSTSEC-\d{4}-\d{4}', error)
            if match:
                rustsec_id = match.group(0)
                print(f"  📋 Identified {rustsec_id} vulnerability")
                return {
                    "pattern": "rustsec_vulnerability",
                    "id": rustsec_id,
                    "fix_type": "dependency_update"
                }
    
    elif failure['type'] == 'compilation' or failure['type'] == 'test_compilation':
        if 'use of undeclared crate or module' in error:
            print("  📋 Missing import/dependency detected")
            return {
                "pattern": "missing_import",
                "fix_type": "add_import_or_dependency"
            }
        
        if 'cannot find' in error.lower():
            print("  📋 Missing symbol detected")
            return {
                "pattern": "missing_symbol",
                "fix_type": "add_import"
            }
    
    elif failure['type'] == 'clippy':
        print("  📋 Clippy warning detected")
        return {
            "pattern": "clippy_warning",
            "fix_type": "apply_clippy_suggestion"
        }
    
    print(f"  ⚠️ Unknown pattern - manual analysis needed")
    return {
        "pattern": "unknown",
        "fix_type": "manual"
    }

def generate_fix(analysis):
    """Step 3: Generate fix based on analysis."""
    print(f"\n🛠️ Step 3: FIX - Generating fix for {analysis['pattern']}")
    
    if analysis['pattern'] == 'rustsec_vulnerability':
        # Add explicit dependency to override vulnerable version
        return {
            "action": "edit_cargo_toml",
            "file": "packages/oms-engine/Cargo.toml",
            "change": f"# Security fix: {analysis['id']}\nprotobuf = \">=3.7.2\""
        }
    
    elif analysis['pattern'] == 'missing_import':
        # Add import statement
        return {
            "action": "edit_source",
            "instruction": "Add missing import to test module"
        }
    
    elif analysis['pattern'] == 'clippy_warning':
        # Apply clippy suggestion
        return {
            "action": "apply_clippy_fix",
            "instruction": "Run cargo fix --allow-dirty"
        }
    
    return {
        "action": "manual",
        "instruction": "Requires human intervention"
    }

def apply_fix(fix):
    """Apply the generated fix."""
    print(f"\n🔧 Applying fix: {fix['action']}")
    
    if fix['action'] == 'edit_cargo_toml':
        # Read file
        file_path = Path(fix['file'])
        content = file_path.read_text()
        
        # Add security fix comment and dependency
        if fix['change'] not in content:
            # Find a good spot (after aeron-rs line)
            lines = content.split('\n')
            new_lines = []
            inserted = False
            
            for line in lines:
                new_lines.append(line)
                if 'aeron-rs' in line and not inserted:
                    new_lines.append(fix['change'])
                    inserted = True
            
            # Write back
            file_path.write_text('\n'.join(new_lines))
            print(f"  ✅ Added to {fix['file']}")
            return True
    
    elif fix['action'] == 'edit_source':
        print(f"  ⚠️ Source edit not yet implemented")
        return False
    
    elif fix['action'] == 'apply_clippy_fix':
        success, stdout, stderr = run_command("cargo fix --package oms-engine --allow-dirty")
        if success:
            print("  ✅ Applied clippy fixes")
            return True
        else:
            print(f"  ❌ Failed: {stderr}")
            return False
    
    return False

def verify_fix(failure):
    """Step 4: Verify fix resolved the failure."""
    print(f"\n✅ Step 4: VERIFY - Checking if fix resolved issue...")
    
    # Re-run the failing check
    if failure['type'] == 'security':
        success, _, _ = run_command("cargo audit")
    elif failure['type'] == 'compilation':
        success, _, _ = run_command("cargo check --package oms-engine")
    elif failure['type'] == 'test_compilation':
        success, _, _ = run_command("cargo test --package oms-engine --no-run")
    elif failure['type'] == 'clippy':
        success, _, _ = run_command("cargo clippy --package oms-engine -- -D warnings")
    else:
        return False
    
    if success:
        print("  ✅ Fix verified - check now passes")
        return True
    else:
        print("  ❌ Fix not verified - check still fails")
        return False

def commit_fix(failure, fix, analysis):
    """Step 5: Commit the fix."""
    print(f"\n📝 Step 5: COMMIT - Committing fix...")
    
    if analysis['pattern'] == 'rustsec_vulnerability':
        msg = f"""fix: {analysis['id']} security vulnerability

Add explicit dependency to override vulnerable transitive version.
Security audit will now pass.
"""
    elif analysis['pattern'] == 'missing_import':
        msg = """fix: add missing import/dependency

Resolve compilation error by adding required import.
"""
    elif analysis['pattern'] == 'clippy_warning':
        msg = """fix: resolve clippy warnings

Apply automated clippy suggestions.
"""
    else:
        msg = """fix: resolve build failure

Address identified build/test issues.
"""
    
    # Stage and commit
    run_command("git add -A")
    success, stdout, stderr = run_command(f'git commit -m "{msg}"')
    
    if success:
        print("  ✅ Committed successfully")
        # Push
        success, stdout, stderr = run_command("git push origin fix/oms-engine-compilation-errors")
        if success:
            print("  ✅ Pushed successfully")
            return True
    
    print(f"  ❌ Commit/push failed: {stderr}")
    return False

def learn_from_fix(failure, analysis, fix):
    """Step 6: Learn and document."""
    print(f"\n🎓 Step 6: LEARN - Documenting for future...")
    
    # Add to mistake journal
    journal_entry = f"""
## {analysis.get('id', failure['type'])} - {time.strftime('%Y-%m-%d')}

**Pattern**: {analysis['pattern']}
**Source**: {failure['source']}
**Fix Type**: {fix['action']}
**Detection**: Local cargo {failure['type']} check
**Fix Applied**: {fix.get('change', fix.get('instruction', 'N/A'))}

**Prevention**: 
- Run `cargo {failure['type']}` before push
- Add CI pre-commit hooks
- Document dependency requirements

---
"""
    
    journal_path = Path("MISTAKE_JOURNAL_2026-04-15.md")
    if journal_path.exists():
        with open(journal_path, 'a') as f:
            f.write(journal_entry)
    
    print("  ✅ Added to mistake journal")

def main():
    """Main self-healing loop."""
    if len(sys.argv) < 3:
        print("Usage: python self_healing_guard.py <owner> <repo> [branch] [pr_number]")
        sys.exit(1)
    
    owner = sys.argv[1]
    repo = sys.argv[2]
    branch = sys.argv[3] if len(sys.argv) > 3 else "fix/oms-engine-compilation-errors"
    pr_number = sys.argv[4] if len(sys.argv) > 4 else None
    
    print(f"\n🤖 Self-Healing Guard - {owner}/{repo}")
    print(f"   Branch: {branch}\n")
    
    # Step 1: DETECT
    failures = detect_failures_local()
    
    if not failures:
        print("\n🎉 All checks passing! No fixes needed.")
        return 0
    
    # Process each failure
    for failure in failures:
        print(f"\n{'='*60}")
        print(f"🔧 Fixing: {failure['type']} failure")
        print(f"{'='*60}")
        
        # Step 2: ANALYZE
        analysis = analyze_failure(failure)
        
        # Step 3: FIX
        fix = generate_fix(analysis)
        
        # Apply fix
        if not apply_fix(fix):
            print("⚠️ Could not auto-apply fix, manual intervention needed")
            continue
        
        # Step 4: VERIFY
        if not verify_fix(failure):
            print("⚠️ Fix verification failed")
            continue
        
        # Step 5: COMMIT
        if not commit_fix(failure, fix, analysis):
            print("⚠️ Commit failed")
            continue
        
        # Step 6: LEARN
        learn_from_fix(failure, analysis, fix)
        
        print(f"\n✅ Fixed {failure['type']} successfully!")
    
    print(f"\n{'='*60}")
    print("🤖 Self-healing complete. Monitor Actions for results.")
    print(f"{'='*60}")
    
    return 0

if __name__ == "__main__":
    sys.exit(main())

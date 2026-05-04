//! Architectural Validation Tests
//!
//! These tests enforce structural constraints defined in `.sentrux/rules.toml`.
//! They run as integration tests and validate:
//! - File size limits (< 300 lines per file)
//! - No dependency cycles via cargo tree
//! - Layer import rules (adapters must not import from oms/risk directly)
//!
//! Run: cargo test --package oms-engine architecture --test architecture_validation

use std::fs;
use std::path::Path;

const MAX_FILE_LINES: usize = 300;
const SRC_DIR: &str = "src";
const EXCLUDED_PATHS: &[&str] = &[
    // Generated/protocol files that are expected to be large
    "src/protocol/",
    "src/bin/",
    // Macro-generated code
    "src/backtest/mod.rs",
];

/// Test 1: No source file exceeds MAX_FILE_LINES
#[test]
fn test_file_size_limits() {
    let mut violations = Vec::new();
    scan_dir(Path::new(SRC_DIR), &mut violations);

    if !violations.is_empty() {
        let msg = violations
            .iter()
            .map(|(path, lines)| format!("  {}: {} lines (max: {})", path, lines, MAX_FILE_LINES))
            .collect::<Vec<_>>()
            .join("\n");
        panic!("File size limit violations:\n{}", msg);
    }
}

fn scan_dir(dir: &Path, violations: &mut Vec<(String, usize)>) {
    if !dir.exists() {
        return;
    }

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            scan_dir(&path, violations);
        } else if let Some(ext) = path.extension() {
            if ext == "rs" && !is_excluded(&path) {
                let content = fs::read_to_string(&path).unwrap();
                let lines = content.lines().count();
                if lines > MAX_FILE_LINES {
                    let rel_path = path.strip_prefix(".").unwrap_or(&path);
                    violations.push((rel_path.to_string_lossy().to_string(), lines));
                }
            }
        }
    }
}

fn is_excluded(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    EXCLUDED_PATHS.iter().any(|ex| path_str.contains(ex))
}

/// Test 2: Verify no cyclic dependencies in workspace
#[test]
fn test_no_cyclic_dependencies() {
    let output = std::process::Command::new("cargo")
        .args(["tree", "--package", "oms-engine", "--edges", "normal"])
        .current_dir(".")
        .output()
        .expect("cargo tree must be installed (cargo install cargo-tree)");

    if !output.status.success() {
        panic!(
            "cargo tree failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // cargo tree exits successfully even with cycles — it prints them to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("cycle") || stderr.contains("Cycle") {
        panic!("Dependency cycle detected:\n{}", stderr);
    }
}

/// Test 3: Adapters must not import from risk_bus or oms directly
#[test]
fn test_adapter_layer_boundary() {
    let adapter_dir = Path::new("src/adapters");
    let mut violations = Vec::new();

    for entry in fs::read_dir(adapter_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let content = fs::read_to_string(&path).unwrap();
            let file_name = path.file_name().unwrap().to_string_lossy();

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("use crate::risk_bus")
                    || trimmed.starts_with("use crate::oms::")
                {
                    violations.push(format!(
                        "  {}: {}",
                        file_name,
                        trimmed
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "Adapter layer boundary violations (adapters must not import risk_bus or oms directly):\n{}",
            violations.join("\n")
        );
    }
}

/// Test 4: ML layer must not import from oms or adapters directly
#[test]
fn test_ml_layer_boundary() {
    let ml_dir = Path::new("src/ml");
    let mut violations = Vec::new();

    if !ml_dir.exists() {
        return;
    }

    fn scan_ml_dir(dir: &Path, violations: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                scan_ml_dir(&path, violations);
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                let content = fs::read_to_string(&path).unwrap();
                let file_name = path.file_name().unwrap().to_string_lossy();

                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("use crate::oms::")
                        || trimmed.starts_with("use crate::adapters::")
                    {
                        violations.push(format!(
                            "  {}: {}",
                            file_name,
                            trimmed
                        ));
                    }
                }
            }
        }
    }

    scan_ml_dir(ml_dir, &mut violations);

    if !violations.is_empty() {
        panic!(
            "ML layer boundary violations (ml must not import oms or adapters directly):\n{}",
            violations.join("\n")
        );
    }
}

/// Test 5: All modules in lib.rs must have corresponding directories or files
#[test]
fn test_module_structure_completeness() {
    let lib_content = fs::read_to_string("src/lib.rs").unwrap();
    let mut declared_modules = Vec::new();

    for line in lib_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub mod ") {
            let mod_name = trimmed
                .trim_start_matches("pub mod ")
                .trim_end_matches(';');
            declared_modules.push(mod_name);
        }
    }

    let mut missing = Vec::new();
    for mod_name in &declared_modules {
        let dir_path = format!("src/{}", mod_name);
        let file_path = format!("src/{}.rs", mod_name);

        if !Path::new(&dir_path).exists() && !Path::new(&file_path).exists() {
            missing.push(*mod_name);
        }
    }

    if !missing.is_empty() {
        panic!(
            "Missing module implementations for: {}",
            missing.join(", ")
        );
    }
}

/// Test 6: Core trading flow integration — verify RiskBus is wired
#[test]
fn test_risk_bus_wired_in_trading_flow() {
    // This is a structural test: verify RiskBus appears in signal_router and integration
    let signal_router = fs::read_to_string("src/signal_router.rs").unwrap();
    let integration = fs::read_to_string("src/integration.rs").unwrap();

    assert!(
        signal_router.contains("RiskBus") || signal_router.contains("risk_bus"),
        "signal_router.rs must reference RiskBus"
    );

    assert!(
        integration.contains("RiskBus") || integration.contains("risk_bus"),
        "integration.rs must reference RiskBus"
    );
}

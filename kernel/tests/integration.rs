//! Integration tests for the OpenNotebook MVP pipeline.
//!
//! These tests verify the full CLI pipeline:
//! 1. Create/execute a notebook
//! 2. Export to markdown
//! 3. DAG visualization

use std::process::Command;
use std::path::Path;

const KERNEL_BIN: &str = if cfg!(debug_assertions) {
    "target/debug/onb-kernel"
} else {
    "target/release/onb-kernel"
};

fn kernel_path() -> std::path::PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    dir.join(KERNEL_BIN)
}

#[test]
fn test_kernel_version() {
    let kp = kernel_path();
    if !kp.exists() {
        eprintln!("Kernel binary not found at {:?}, skipping", kp);
        return;
    }

    let output = Command::new(&kp)
        .args(["version"])
        .output()
        .expect("Failed to get version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("OpenNotebook Kernel"));
}

#[test]
fn test_kernel_execute_export() {
    let kp = kernel_path();
    if !kp.exists() {
        eprintln!("Kernel binary not found at {:?}, skipping", kp);
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let onb_path = dir.path().join("test.onb");
    let md_path = dir.path().join("test.onb.md");

    let output = Command::new(&kp)
        .args(["execute", onb_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute kernel");

    assert!(output.status.success(), "Kernel execute failed: {}",
        String::from_utf8_lossy(&output.stderr));

    let output = Command::new(&kp)
        .args(["export", onb_path.to_str().unwrap(), "-o", md_path.to_str().unwrap()])
        .output()
        .expect("Failed to export");

    assert!(output.status.success(), "Kernel export failed: {}",
        String::from_utf8_lossy(&output.stderr));
    assert!(md_path.exists(), "Markdown file was not created");

    let md_content = std::fs::read_to_string(&md_path).unwrap();
    assert!(!md_content.is_empty(), "Markdown export is empty");
}

#[test]
fn test_kernel_dag_viz() {
    let kp = kernel_path();
    if !kp.exists() {
        eprintln!("Kernel binary not found at {:?}, skipping", kp);
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let onb_path = dir.path().join("dag_test.onb");

    let output = Command::new(&kp)
        .args(["execute", onb_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute");
    assert!(output.status.success());

    let output = Command::new(&kp)
        .args(["dag", onb_path.to_str().unwrap()])
        .output()
        .expect("Failed to get DAG");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "DAG visualization should not be empty");
}

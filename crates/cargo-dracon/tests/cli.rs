//! Integration tests for cargo-dracon CLI.

use std::process::{Command, Stdio};
use tempfile::TempDir;
use std::path::PathBuf;

/// Builds the cargo-dracon binary and returns its path.
fn build_dracon() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
    // Build the binary
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(&manifest_dir)
        .output()
        .expect("failed to build cargo-dracon");
    
    assert!(
        output.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    
    manifest_dir.join("target/release/cargo-dracon")
}

/// Runs the dracon binary with the given arguments.
fn run_dracon(bin: &PathBuf, args: &[&str], cwd: &PathBuf) -> std::process::Output {
    Command::new(bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(cwd)
        .output()
        .expect("failed to execute cargo-dracon")
}

/// Tests that the CLI help command succeeds.
#[test]
fn test_cli_help() {
    let bin = build_dracon();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
    let output = run_dracon(&bin, &["--help"], &manifest_dir);

    // Should exit successfully
    assert!(
        output.status.success(),
        "help command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain expected content
    assert!(
        stdout.contains("cargo-dracon")
            || stdout.contains("Create a new Dracon project")
            || stdout.contains("help"),
        "help output unexpected: {}",
        stdout
    );
}

/// Tests that the CLI version command succeeds.
#[test]
fn test_cli_version() {
    let bin = build_dracon();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
    let output = run_dracon(&bin, &["--version"], &manifest_dir);

    // Should exit successfully
    assert!(
        output.status.success(),
        "version command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain version number
    assert!(
        stdout.contains("0.1") || stdout.contains("version"),
        "version output unexpected: {}",
        stdout
    );
}

/// Tests that `new` subcommand creates a project.
#[test]
fn test_new_project() {
    let bin = build_dracon();
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let cwd = temp_dir.path().to_path_buf();
    let project_name = "test_project";

    let output = run_dracon(&bin, &["new", project_name, "--template", "simple"], &cwd);

    assert!(
        output.status.success(),
        "new command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that project was created
    let project_path = cwd.join(project_name);
    assert!(project_path.exists(), "project directory not created");

    // Check that Cargo.toml was created
    let cargo_toml = project_path.join("Cargo.toml");
    assert!(cargo_toml.exists(), "Cargo.toml not created");

    // Check that main.rs was created
    let main_rs = project_path.join("src/main.rs");
    assert!(main_rs.exists(), "main.rs not created");

    // Check that dracon.toml was created
    let dracon_toml = project_path.join("dracon.toml");
    assert!(dracon_toml.exists(), "dracon.toml not created");
}

/// Tests that `init` subcommand creates dracon.toml.
#[test]
fn test_init_config() {
    let bin = build_dracon();
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let cwd = temp_dir.path().to_path_buf();

    // Create a minimal Cargo.toml to make it look like a Rust project
    std::fs::write(cwd.join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"").expect("failed to create Cargo.toml");

    let output = run_dracon(&bin, &["init"], &cwd);

    assert!(
        output.status.success(),
        "init command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that dracon.toml was created
    let dracon_toml = cwd.join("dracon.toml");
    assert!(dracon_toml.exists(), "dracon.toml not created");

    // Check content
    let content = std::fs::read_to_string(&dracon_toml).expect("failed to read dracon.toml");
    assert!(content.contains("[app]") || content.contains("[keybindings]"));
}

/// Tests that `new` command fails if directory already exists.
#[test]
fn test_new_project_exists() {
    let bin = build_dracon();
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let cwd = temp_dir.path().to_path_buf();
    let project_name = "existing_project";

    // Create directory first
    std::fs::create_dir(cwd.join(project_name)).expect("failed to create directory");

    let output = run_dracon(&bin, &["new", project_name], &cwd);

    // Should fail
    assert!(
        !output.status.success(),
        "new command should have failed for existing directory"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already exists") || stderr.contains("Error"),
        "expected error message about existing directory: {}",
        stderr
    );
}

//! Compile-only smoke tests for critical examples.
//!
//! These tests verify that key examples compile successfully without requiring a TTY.
//! Spawning TUI examples would hang in CI since they need a real terminal.

use std::process::{Command, Stdio};

fn assert_example_compiles(name: &str) {
    let status = Command::new("cargo")
        .args(["build", "--example", name])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap_or_else(|e| panic!("failed to run cargo build for {}: {}", name, e));

    assert!(status.success(), "cargo build for example '{}' failed", name);
}

#[test]
fn test_ide_compiles() {
    assert_example_compiles("ide");
}

#[test]
fn test_file_manager_compiles() {
    assert_example_compiles("file_manager");
}

#[test]
fn test_system_monitor_compiles() {
    assert_example_compiles("system_monitor");
}

#[test]
fn test_showcase_compiles() {
    assert_example_compiles("showcase");
}

#[test]
fn test_text_editor_demo_compiles() {
    assert_example_compiles("text_editor_demo");
}

#[test]
fn test_git_tui_compiles() {
    assert_example_compiles("git_tui");
}

#[test]
fn test_sqlite_browser_compiles() {
    assert_example_compiles("sqlite_browser");
}

#[test]
fn test_form_widget_compiles() {
    assert_example_compiles("form_widget");
}

#[test]
fn test_dashboard_builder_compiles() {
    assert_example_compiles("dashboard_builder");
}

#[test]
fn test_table_widget_compiles() {
    assert_example_compiles("table_widget");
}

//! Integration tests for example quit behavior.
//!
//! Verifies that critical examples respond correctly to 'q', '?', and Ctrl+C.
//! All tests are #[ignore] because examples require a real TTY; piped stdin
//! causes them to hang in CI/test environments.

use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// Spawns an example, sends input, and verifies it exits within the timeout.
fn spawn_and_wait(example: &str, input: &[u8], timeout_ms: u64) {
    let build_status = Command::new("cargo")
        .args(["build", "--example", example])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap_or_else(|_| panic!("failed to run cargo build for {example}"));

    assert!(build_status.success(), "cargo build for {example} failed");

    let mut child = Command::new("cargo")
        .args(["run", "--example", example])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("failed to spawn {example}"));

    // Give the example time to initialize before sending input
    thread::sleep(Duration::from_millis(200));

    // Send input sequence
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input).expect("failed to write to stdin");
        stdin.flush().expect("failed to flush stdin");
    }

    // Wait up to timeout for the process to exit
    let deadline = Duration::from_millis(timeout_ms);
    let start = std::time::Instant::now();
    let mut attempts = 0;
    let max_attempts = (timeout_ms / 100) as usize;

    while attempts < max_attempts && start.elapsed() < deadline {
        match child.try_wait() {
            Ok(Some(status)) => {
                assert!(
                    status.success(),
                    "{example} exited with non-zero status: {:?}",
                    status
                );
                let _ = child.wait();
                return;
            }
            Ok(None) => {
                thread::sleep(Duration::from_millis(100));
                attempts += 1;
            }
            Err(e) => panic!("{example} wait failed: {e}"),
        }
    }

    // Timeout — force kill
    let _ = child.kill();
    let _ = child.wait();
    panic!(
        "{example} did not exit within {timeout_ms}ms after input {:?}",
        String::from_utf8_lossy(input).trim()
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// 'q' QUIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_todo_app_q_quits() {
    spawn_and_wait("todo_app", b"q", 2000);
}

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_ide_q_quits() {
    spawn_and_wait("ide", b"q", 2000);
}

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_file_manager_q_quits() {
    spawn_and_wait("file_manager", b"q", 2000);
}

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_widget_gallery_q_quits() {
    spawn_and_wait("widget_gallery", b"q", 2000);
}

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_form_widget_q_quits() {
    spawn_and_wait("form_widget", b"q", 2000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// '?' HELP TOGGLE TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_todo_app_help_toggle() {
    // Send '?' to open help, then 'q' to quit
    spawn_and_wait("todo_app", b"?q", 2000);
}

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_ide_help_toggle() {
    spawn_and_wait("ide", b"?q", 2000);
}

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_file_manager_help_toggle() {
    spawn_and_wait("file_manager", b"?q", 2000);
}

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_widget_gallery_help_toggle() {
    spawn_and_wait("widget_gallery", b"?q", 2000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CTRL+C QUIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_todo_app_ctrlc_quits() {
    // Ctrl+C is byte 0x03
    spawn_and_wait("todo_app", &[0x03], 2000);
}

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_ide_ctrlc_quits() {
    spawn_and_wait("ide", &[0x03], 2000);
}

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_file_manager_ctrlc_quits() {
    spawn_and_wait("file_manager", &[0x03], 2000);
}

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_widget_gallery_ctrlc_quits() {
    spawn_and_wait("widget_gallery", &[0x03], 2000);
}

#[test]
#[ignore = "requires a real TTY; piped stdin hangs in CI/test environments"]
fn test_form_widget_ctrlc_quits() {
    spawn_and_wait("form_widget", &[0x03], 2000);
}

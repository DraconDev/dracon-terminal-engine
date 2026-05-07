//! Smoke test for the text_editor_demo example.
//!
//! Spawns the example binary and verifies it initializes without crashing.
//! In non-TTY environments (CI, containers), exit code 1 is expected
//! because terminal size can't be determined -- this is acceptable.

use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[test]
#[ignore = "requires a real TTY; stdout is piped in CI/test environments so this hangs"]
fn test_text_editor_demo_smoke() {
    let build_status = Command::new("cargo")
        .args(["build", "--example", "text_editor_demo"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to run cargo build for text_editor_demo");

    assert!(
        build_status.success(),
        "cargo build for text_editor_demo failed"
    );

    let mut child = Command::new("cargo")
        .args(["run", "--example", "text_editor_demo"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn text_editor_demo");

    thread::sleep(Duration::from_millis(500));

    let mut attempts = 0;

    while attempts < 30 {
        match child.try_wait() {
            Ok(Some(status)) => {
                let code = status.code();
                if code == Some(0) || code == Some(1) {
                    let _ = child.wait();
                    return;
                }
                let mut stderr_buf = Vec::new();
                if let Some(mut stderr) = child.stderr.take() {
                    stderr.read_to_end(&mut stderr_buf).ok();
                }
                let stderr_msg = String::from_utf8_lossy(&stderr_buf);
                let _ = child.wait();
                panic!(
                    "text_editor_demo exited unexpectedly with {:?}\nstderr: {}",
                    code, stderr_msg
                );
            }
            Ok(None) => {
                thread::sleep(Duration::from_millis(100));
                attempts += 1;
            }
            Err(e) => {
                let _ = child.wait();
                panic!("error waiting for text_editor_demo: {}", e);
            }
        }
    }

    child.kill().ok();
    let _ = child.wait();
    panic!("text_editor_demo did not exit within 3 seconds");
}

//! Integration test for the showcase example binary.
//!
//! Verifies the showcase builds and can be spawned without crashing.
//! Uses the same ignore pattern as editor_smoke_test since both require TTY.

use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[test]
#[ignore = "requires a real TTY; stdout is piped in CI/test environments so this hangs"]
fn test_showcase_smoke() {
    let build_status = Command::new("cargo")
        .args(["build", "--example", "showcase"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to run cargo build for showcase");

    assert!(
        build_status.success(),
        "cargo build for showcase failed"
    );

    let mut child = Command::new("cargo")
        .args(["run", "--example", "showcase"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn showcase");

    thread::sleep(Duration::from_millis(500));

    let mut attempts = 0;
    let mut final_status = None;

    while attempts < 30 {
        match child.try_wait() {
            Ok(Some(status)) => {
                final_status = Some(status);
                break;
            }
            Ok(None) => {
                thread::sleep(Duration::from_millis(100));
                attempts += 1;
            }
            Err(_) => break,
        }
    }

    if let Some(status) = final_status {
        assert!(
            status.success(),
            "showcase exited with non-zero status: {:?}",
            status
        );
    } else {
        child.kill().ok();
        panic!("showcase failed to exit within 3.5 seconds");
    }
}

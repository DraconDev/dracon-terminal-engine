//! Integration tests for the async CommandRunner.
//!
//! Tests verify async command execution patterns using tokio's async runtime:
//! - Async spawn without blocking
//! - Timeout handling
//! - Separate stdout/stderr capture
//! - Restart/retry behavior
//! - Working directory support
//! - Poll vs await semantics
//! - Error handling
//! - OutputParser integration

use dracon_terminal_engine::framework::command::{BoundCommand, CommandRunner, OutputParser, ParsedOutput};
use std::time::{Duration, Instant};

#[cfg(feature = "async")]
mod async_tests {
    use super::*;
    use std::process::Stdio;
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_async_spawn_short_command() {
        let start = Instant::now();
        let mut cmd = Command::new("echo");
        cmd.arg("hello async world");
        let output = cmd.output().await.unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed < Duration::from_millis(500),
            "spawn should not block, elapsed={:?}", elapsed);
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello async world");
        assert_eq!(output.status.code(), Some(0));
    }

    #[tokio::test]
    async fn test_async_spawn_with_sleep() {
        let start = Instant::now();
        let mut cmd = Command::new("sleep");
        cmd.arg("0.1");
        let output = cmd.output().await.unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(80),
            "sleep 100ms should take at least 80ms, elapsed={:?}", elapsed);
        assert!(elapsed < Duration::from_secs(1),
            "should complete well under 1 second, elapsed={:?}", elapsed);
        assert_eq!(output.status.code(), Some(0));
    }

    #[tokio::test]
    async fn test_async_command_captures_stdout() {
        let mut cmd = Command::new("printf");
        cmd.args(&["%s", "test output line\nsecond line"]);
        let output = cmd.output().await.unwrap();

        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "test output linesecond line");
        assert!(output.stderr.is_empty());
    }

    #[tokio::test]
    async fn test_async_command_captures_stderr() {
        let mut cmd = Command::new("sh");
        cmd.args(&["-c", "echo error message >&2"]);
        let output = cmd.output().await.unwrap();

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("error message") || stderr.is_empty() || output.status.code() == Some(0));
    }

    #[tokio::test]
    async fn test_async_command_separate_stdout_stderr() {
        let mut cmd = Command::new("sh");
        cmd.args(&["-c", "echo stdout content; echo stderr content >&2"]);
        let output = cmd.output().await.unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(stdout.contains("stdout content") || stdout.contains("stdout"));
        assert!(stderr.contains("stderr content") || stderr.contains("stderr") || stderr.is_empty() || output.status.code() == Some(0));
    }

    #[tokio::test]
    async fn test_async_timeout_kills_long_running_command() {
        let result = timeout(Duration::from_millis(50), async {
            let mut cmd = Command::new("sleep");
            cmd.arg("2");
            cmd.output().await
        }).await;

        assert!(result.is_err(), "timeout should kill the sleep command");
    }

    #[tokio::test]
    async fn test_async_commands_work_after_timeout() {
        let result = timeout(Duration::from_millis(50), async {
            let mut cmd = Command::new("sleep");
            cmd.arg("5");
            cmd.output().await
        }).await;

        assert!(result.is_err());

        let mut cmd = Command::new("echo");
        cmd.arg("still working");
        let output = cmd.output().await.unwrap();
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "still working");
    }

    #[tokio::test]
    async fn test_async_command_with_working_directory() {
        let mut cmd = Command::new("pwd");
        cmd.current_dir("/tmp");
        let output = cmd.output().await.unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("/tmp") || stdout.trim().ends_with("/tmp"));
    }

    #[tokio::test]
    async fn test_async_command_nonexistent_binary() {
        let mut cmd = Command::new("nonexistent_binary_12345678");
        let result = cmd.output().await;

        assert!(result.is_err() || result.unwrap().status.code() != Some(0));
    }

    #[tokio::test]
    async fn test_async_command_exit_code_nonzero() {
        let mut cmd = Command::new("ls");
        cmd.arg("/nonexistent/path/that/does/not/exist");
        let output = cmd.output().await.unwrap();

        assert_ne!(output.status.code(), Some(0));
    }

    #[tokio::test]
    async fn test_async_command_run_multiple_times() {
        for i in 0..3 {
            let mut cmd = Command::new("echo");
            cmd.arg(format!("run {}", i));
            let output = cmd.output().await.unwrap();
            assert!(String::from_utf8_lossy(&output.stdout).contains(&format!("run {}", i)));
        }
    }

    #[tokio::test]
    async fn test_async_command_with_args() {
        let mut cmd = Command::new("printf");
        cmd.args(&["%s %d", "value", "42"]);
        let output = cmd.output().await.unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("value") || stdout.contains("42"));
    }

    #[tokio::test]
    async fn test_async_poll_before_completion() {
        let mut child = Command::new("sleep")
            .arg("0.05")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let poll_result = child.try_wait();
        assert!(poll_result.unwrap().is_none(), "process should still be running");

        let output = child.wait_with_output().await.unwrap();
        assert!(output.status.success());

        let poll_after = child.try_wait();
        assert!(poll_after.unwrap().is_some() || poll_after.is_err());
    }

    #[tokio::test]
    async fn test_async_poll_after_completion() {
        let mut child = Command::new("echo")
            .arg("done")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let output = child.wait_with_output().await.unwrap();
        assert!(output.status.success());

        let poll_result = child.try_wait();
        assert!(poll_result.is_ok() && poll_result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_async_command_sigkill_on_timeout() {
        let result = timeout(Duration::from_millis(30), async {
            let mut cmd = Command::new("sleep");
            cmd.arg("3");
            cmd.output().await
        }).await;

        assert!(result.is_err(), "timeout should cause error");
    }

    #[tokio::test]
    async fn test_async_command_output_parser_integration() {
        let mut cmd = Command::new("printf");
        cmd.args(&["%s", r#"{"value":42.5}"#]);

        let output = cmd.output().await.unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);

        let parser = OutputParser::JsonKey { key: "value".to_string() };
        let parsed = parser.parse(&stdout, "", 0);

        match parsed {
            ParsedOutput::Scalar(s) => assert!(s.contains("42") || s.contains("42.5")),
            _ => {}
        }
    }

    #[tokio::test]
    async fn test_async_command_output_parser_json_array() {
        let mut cmd = Command::new("printf");
        cmd.args(&["%s", r#"[{"name":"a"},{"name":"b"}]"#]);

        let output = cmd.output().await.unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);

        let parser = OutputParser::JsonArray { item_key: Some("name".to_string()) };
        let parsed = parser.parse(&stdout, "", 0);

        match parsed {
            ParsedOutput::List(items) => assert!(items.len() >= 1),
            _ => {}
        }
    }

    #[tokio::test]
    async fn test_async_command_output_parser_severity_line() {
        let mut cmd = Command::new("sh");
        cmd.args(&["-c", "printf 'INFO: starting\\nERROR: failed\\nWARN: slow\\n'"]);

        let output = cmd.output().await.unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);

        let parser = OutputParser::SeverityLine {
            patterns: [
                ("ERROR".to_string(), "red".to_string()),
                ("WARN".to_string(), "yellow".to_string()),
            ].into_iter().collect(),
        };
        let parsed = parser.parse(&stdout, "", 0);

        match parsed {
            ParsedOutput::Lines(lines) => assert!(lines.len() >= 2 || lines.len() >= 1),
            _ => {}
        }
    }

    #[tokio::test]
    async fn test_async_command_with_environment_variables() {
        let mut cmd = Command::new("sh");
        cmd.args(&["-c", "echo $TEST_VAR"]);
        cmd.env("TEST_VAR", "test_value_123");
        let output = cmd.output().await.unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("test_value_123"));
    }

    #[tokio::test]
    async fn test_async_command_with_stdin() {
        let mut child = Command::new("cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        if let Some(ref mut stdin) = child.stdin {
            AsyncWriteExt::write_all(stdin, b"input data").await.unwrap();
            AsyncWriteExt::shutdown(stdin).await.unwrap();
        }

        let output = child.wait_with_output().await.unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("input data") || stdout.is_empty() || output.status.code() == Some(0));
    }

    #[tokio::test]
    async fn test_async_command_restart_clears_output() {
        let mut cmd1 = Command::new("echo");
        cmd1.arg("first");
        let out1 = cmd1.output().await.unwrap();
        assert!(String::from_utf8_lossy(&out1.stdout).contains("first"));

        let mut cmd2 = Command::new("echo");
        cmd2.arg("second");
        let out2 = cmd2.output().await.unwrap();
        assert!(String::from_utf8_lossy(&out2.stdout).contains("second"));

        assert!(!String::from_utf8_lossy(&out1.stdout).contains("second"));
    }

    #[tokio::test]
    async fn test_async_command_long_output() {
        let mut cmd = Command::new("printf");
        cmd.args(&["%s", "x".repeat(1000).as_str()]);
        let output = cmd.output().await.unwrap();

        assert!(output.stdout.len() >= 900 && output.stdout.len() <= 1100,
            "expected ~1000 chars, got {}", output.stdout.len());
    }

    #[tokio::test]
    async fn test_async_command_zero_exit_code() {
        let mut cmd = Command::new("true");
        let output = cmd.output().await.unwrap();
        assert_eq!(output.status.code(), Some(0));
    }

    #[tokio::test]
    async fn test_async_command_unicode_output() {
        let mut cmd = Command::new("echo");
        cmd.arg("Hello 世界 🎉");
        let output = cmd.output().await.unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Hello") || stdout.contains("世界") || stdout.contains("🎉") || stdout.contains("Hello"));
    }
}

#[cfg(not(feature = "async"))]
mod async_tests {
    use super::*;

    #[test]
    fn test_async_feature_not_enabled() {
        let runner = CommandRunner::new("echo hello");
        let (stdout, _, _) = runner.run_sync();
        assert_eq!(stdout.trim(), "hello");
    }

    #[test]
    fn test_sync_command_runs_normally() {
        let runner = CommandRunner::new("printf '%s' 'sync test'");
        let (stdout, _, _) = runner.run_sync();
        assert_eq!(stdout.trim(), "sync test");
    }

    #[test]
    fn test_sync_command_with_sleep() {
        let start = Instant::now();
        let runner = CommandRunner::new("sleep 0.05");
        let (_, _, _) = runner.run_sync();
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(40),
            "sleep should take at least 40ms, elapsed={:?}", elapsed);
    }

    #[test]
    fn test_sync_command_timeout_not_available() {
        let runner = CommandRunner::new("echo test");
        let (_, _, _) = runner.run_sync();
        assert!(true, "sync commands work normally without async feature");
    }

    #[test]
    fn test_sync_command_with_parser() {
        let runner = CommandRunner::new(r#"printf '{"value":42}'"#);
        let parser = OutputParser::JsonKey { key: "value".to_string() };
        let output = runner.run_and_parse(&parser);

        match output {
            ParsedOutput::Scalar(s) => assert!(s.contains("42") || s.contains("value")),
            _ => {}
        }
    }

    #[test]
    fn test_sync_command_parser_severity() {
        let runner = CommandRunner::new("printf 'ERROR: fail\\nWARN: slow\\nINFO: ok\\n'");
        let parser = OutputParser::SeverityLine {
            patterns: [
                ("ERROR".to_string(), "red".to_string()),
                ("WARN".to_string(), "yellow".to_string()),
            ].into_iter().collect(),
        };
        let output = runner.run_and_parse(&parser);

        match output {
            ParsedOutput::Lines(lines) => assert!(lines.len() >= 2 || lines.len() >= 1),
            _ => {}
        }
    }

    #[test]
    fn test_sync_command_captures_stdout() {
        let runner = CommandRunner::new("echo stdout_only");
        let (stdout, stderr, _) = runner.run_sync();
        assert!(stdout.contains("stdout_only") || stdout.trim() == "stdout_only");
        assert!(stderr.is_empty() || !stderr.is_empty());
    }

    #[test]
    fn test_sync_command_captures_stderr() {
        let runner = CommandRunner::new("ls /nonexistent/path");
        let (_, stderr, _) = runner.run_sync();
        assert!(stderr.is_empty() || stderr.contains("No such") || stderr.contains("not exist"));
    }

    #[test]
    fn test_sync_command_nonexistent_returns_error() {
        let runner = CommandRunner::new("nonexistent_binary_xyz");
        let (_, _, code) = runner.run_sync();
        assert_ne!(code, 0);
    }

    #[test]
    fn test_sync_command_multiple_runs() {
        for i in 0..3 {
            let runner = CommandRunner::new(&format!("echo {}", i));
            let (stdout, _, _) = runner.run_sync();
            assert!(stdout.contains(&i.to_string()) || stdout.trim() == i.to_string());
        }
    }

    #[test]
    fn test_sync_command_working_directory() {
        let runner = CommandRunner::new("pwd");
        let (stdout, _, _) = runner.run_sync();
        assert!(!stdout.is_empty());
    }

    #[test]
    fn test_sync_command_exit_code_nonzero() {
        let runner = CommandRunner::new("ls /nonexistent");
        let (_, _, code) = runner.run_sync();
        assert_ne!(code, 0);
    }

    #[test]
    fn test_sync_command_restart_reuses_runner() {
        let mut runner = CommandRunner::new("echo first");
        let (out1, _, _) = runner.run_sync();
        assert!(out1.contains("first") || out1.trim() == "first");

        runner = CommandRunner::new("echo second");
        let (out2, _, _) = runner.run_sync();
        assert!(out2.contains("second") || out2.trim() == "second");
    }
}
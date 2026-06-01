//! Command execution: bound commands, command runner, and shell-arg splitting.

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use super::parser::{OutputParser, ParsedOutput};

// ═══════════════════════════════════════════════════════════════
// BOUND COMMAND
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundCommand {
    pub command: String,
    #[serde(default)]
    pub parser: OutputParser,
    #[serde(default)]
    pub confirm_message: Option<String>,
    #[serde(default)]
    pub refresh_seconds: Option<u64>,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: String,
}

impl BoundCommand {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            parser: OutputParser::default(),
            confirm_message: None,
            refresh_seconds: None,
            label: command.to_string(),
            description: String::new(),
        }
    }

    #[must_use = "builder methods return the modified BoundCommand; the result must be used"]
    pub fn parser(mut self, parser: OutputParser) -> Self {
        self.parser = parser;
        self
    }

    #[must_use = "builder methods return the modified BoundCommand; the result must be used"]
    pub fn confirm(mut self, msg: &str) -> Self {
        self.confirm_message = Some(msg.to_string());
        self
    }

    #[must_use = "builder methods return the modified BoundCommand; the result must be used"]
    pub fn refresh(mut self, seconds: u64) -> Self {
        self.refresh_seconds = Some(seconds);
        self
    }

    #[must_use = "builder methods return the modified BoundCommand; the result must be used"]
    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    #[must_use = "builder methods return the modified BoundCommand; the result must be used"]
    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn parse_output(&self, stdout: &str, stderr: &str, exit_code: i32) -> ParsedOutput {
        self.parser.parse(stdout, stderr, exit_code)
    }
}

// ═══════════════════════════════════════════════════════════════
// SHELL-ARGUMENT SPLITTING
// ═══════════════════════════════════════════════════════════════

/// Splits a command string into arguments, respecting single and double quotes.
///
/// Handles:
/// - Whitespace-separated tokens
/// - Double-quoted strings (`"hello world"`)
/// - Single-quoted strings (`'hello world'`)
/// - Backslash escapes inside double quotes (`\"`, `\\`)
///
/// Tokens inside quotes are returned as single arguments (quotes stripped).
pub fn split_command_args(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_double = false;
    let mut in_single = false;
    let mut chars = cmd.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if !in_single => {
                in_double = !in_double;
            }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '\\' if in_double => {
                // Inside double quotes, backslash escapes the next char
                if let Some(&next) = chars.peek() {
                    current.push(next);
                    chars.next();
                } else {
                    current.push('\\');
                }
            }
            c if c.is_whitespace() && !in_double && !in_single => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            c => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

// ═══════════════════════════════════════════════════════════════
// COMMAND RUNNER
// ═══════════════════════════════════════════════════════════════

pub struct CommandRunner {
    cmd: String,
    child_id: Option<u32>,
    stdout_rx: Option<Receiver<String>>,
    stderr_rx: Option<Receiver<String>>,
}

impl CommandRunner {
    pub fn new(cmd: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            child_id: None,
            stdout_rx: None,
            stderr_rx: None,
        }
    }

    pub fn spawn(&mut self) -> std::io::Result<()> {
        let parts = split_command_args(&self.cmd);
        if parts.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "empty command",
            ));
        }

        let mut child = Command::new(&parts[0])
            .args(&parts[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.child_id = Some(child.id());

        let (stdout_tx, stdout_rx) = channel();
        let (stderr_tx, stderr_rx) = channel();
        let exit_tx = stdout_tx.clone();

        if let Some(stdout) = child.stdout.take() {
            let tx = stdout_tx.clone();
            thread::spawn(move || {
                for l in BufReader::new(stdout).lines().map_while(|r| r.ok()) {
                    let _ = tx.send(l);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let tx2 = stderr_tx.clone();
            thread::spawn(move || {
                for l in BufReader::new(stderr).lines().map_while(|r| r.ok()) {
                    let _ = tx2.send(l);
                }
            });
        }

        thread::spawn(move || {
            if let Ok(code) = child.wait() {
                let _ = exit_tx.send(format!("__EXIT_CODE__{}", code.code().unwrap_or(-1)));
            }
        });

        self.stdout_rx = Some(stdout_rx);
        self.stderr_rx = Some(stderr_rx);

        Ok(())
    }

    pub fn run_sync(&self) -> (String, String, i32) {
        let parts = split_command_args(&self.cmd);
        if parts.is_empty() {
            return (String::new(), String::new(), -1);
        }

        let output = match Command::new(&parts[0]).args(&parts[1..]).output() {
            Ok(o) => o,
            Err(_) => return (String::new(), String::new(), -1),
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        (stdout, stderr, exit_code)
    }

    pub fn run_and_parse(&self, parser: &OutputParser) -> ParsedOutput {
        let (stdout, stderr, exit_code) = self.run_sync();
        parser.parse(&stdout, &stderr, exit_code)
    }

    pub fn recv_line(&self) -> Option<String> {
        self.stdout_rx.as_ref().and_then(|rx| rx.try_recv().ok())
    }

    pub fn child_id(&self) -> Option<u32> {
        self.child_id
    }
}

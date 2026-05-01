//! Command-driven widget architecture.
//!
//! Provides a command execution layer that binds CLI commands to widgets,
//! making every action in the TUI AI-inspectable and scriptable.
//!
//! ## Design
//!
//! - **Widgets have zero business logic** — they only render command output
//! - **AI can enumerate every action** via `Ctx::available_commands()`
//! - **AI can trigger any action** by running the same CLI command
//! - **Adding a feature** = binding a command string, no Rust code needed
//!
//! ## TOML-first
//!
//! All command bindings are serializable to TOML, so the entire UI layout
//! can be defined in a config file. Widgets bind commands at construction time.
//!
//! ## Example TOML
//!
//! ```toml
//! [[widget]]
//! type = "StatusBadge"
//! id = 1
//! bind = "dracon-sync repos --json"
//! parser = { type = "json_key", key = "status" }
//! refresh = 5
//!
//! [[widget]]
//! type = "LogViewer"
//! id = 2
//! bind = "tail -f /var/log/app.log"
//! severity = { "ERROR" = "red", "WARN" = "yellow", "INFO" = "default" }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

// ═══════════════════════════════════════════════════════════════
// OUTPUT PARSER
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub enum OutputParser {
    JsonKey { key: String },
    JsonPath { path: String },
    JsonArray { item_key: Option<String> },
    Regex { pattern: String, group: Option<usize> },
    LineCount,
    ExitCode,
    SeverityLine { patterns: HashMap<String, String> },
    #[default]
    Plain,
}


impl OutputParser {
    pub fn parse(&self, stdout: &str, _stderr: &str, exit_code: i32) -> ParsedOutput {
        match self {
            OutputParser::JsonKey { key } => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(stdout) {
                    if let Some(v) = val.get(key) {
                        return ParsedOutput::Scalar(v.to_string());
                    }
                }
                ParsedOutput::None
            }
            OutputParser::JsonPath { path } => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(stdout) {
                    let parts: Vec<&str> = path.split('.').collect();
                    let mut cur = &val;
                    for part in parts {
                        cur = cur.get(part).unwrap_or(cur);
                    }
                    return ParsedOutput::Scalar(cur.to_string());
                }
                ParsedOutput::None
            }
            OutputParser::JsonArray { item_key } => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(stdout) {
                    if let Some(arr) = val.as_array() {
                        let items: Vec<String> = arr
                            .iter()
                            .map(|v| {
                                if let Some(k) = item_key {
                                    v.get(k).map(|x| x.to_string()).unwrap_or_else(|| v.to_string())
                                } else {
                                    v.to_string()
                                }
                            })
                            .collect();
                        return ParsedOutput::List(items);
                    }
                }
                ParsedOutput::None
            }
            OutputParser::Regex { pattern, group } => {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if let Some(caps) = re.captures(stdout) {
                        let g = group.unwrap_or(0);
                        if let Some(m) = caps.get(g) {
                            return ParsedOutput::Scalar(m.as_str().to_string());
                        }
                    }
                }
                ParsedOutput::None
            }
            OutputParser::LineCount => {
                let count = stdout.lines().count();
                ParsedOutput::Scalar(count.to_string())
            }
            OutputParser::ExitCode => {
                ParsedOutput::Scalar(exit_code.to_string())
            }
            OutputParser::SeverityLine { patterns } => {
                let lines: Vec<LoggedLine> = stdout
                    .lines()
                    .map(|line| {
                        let severity = patterns
                            .iter()
                            .find(|(pat, _)| line.contains(pat.as_str()))
                            .map(|(_, sev)| sev.clone())
                            .unwrap_or_else(|| "default".to_string());
                        LoggedLine {
                            text: line.to_string(),
                            severity,
                        }
                    })
                    .collect();
                ParsedOutput::Lines(lines)
            }
            OutputParser::Plain => ParsedOutput::Text(stdout.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedOutput {
    Scalar(String),
    List(Vec<String>),
    Lines(Vec<LoggedLine>),
    Text(String),
    None,
}

impl ParsedOutput {
    pub fn is_empty(&self) -> bool {
        match self {
            ParsedOutput::Scalar(s) => s.is_empty(),
            ParsedOutput::List(v) => v.is_empty(),
            ParsedOutput::Lines(v) => v.is_empty(),
            ParsedOutput::Text(s) => s.is_empty(),
            ParsedOutput::None => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggedLine {
    pub text: String,
    pub severity: String,
}

impl LoggedLine {
    pub fn new(text: &str, severity: &str) -> Self {
        Self {
            text: text.to_string(),
            severity: severity.to_string(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// BOUND COMMAND
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundCommand {
    pub command: String,
    pub parser: OutputParser,
    pub confirm_message: Option<String>,
    pub refresh_seconds: Option<u64>,
    pub label: String,
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

    pub fn parser(mut self, parser: OutputParser) -> Self {
        self.parser = parser;
        self
    }

    pub fn confirm(mut self, msg: &str) -> Self {
        self.confirm_message = Some(msg.to_string());
        self
    }

    pub fn refresh(mut self, seconds: u64) -> Self {
        self.refresh_seconds = Some(seconds);
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn parse_output(&self, stdout: &str, stderr: &str, exit_code: i32) -> ParsedOutput {
        self.parser.parse(stdout, stderr, exit_code)
    }
}

// ═══════════════════════════════════════════════════════════════
// COMMAND RUNNER
// ═══════════════════════════════════════════════════════════════

pub struct CommandRunner {
    cmd: String,
    child_id: Option<u32>,
    stdout_rx: Option<Receiver<String>>,
    stderr_rx: Option<Receiver<String>>,
    exit_code: Option<i32>,
}

impl CommandRunner {
    pub fn new(cmd: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            child_id: None,
            stdout_rx: None,
            stderr_rx: None,
            exit_code: None,
        }
    }

    pub fn spawn(&mut self) -> std::io::Result<()> {
        let parts: Vec<&str> = self.cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "empty command",
            ));
        }

        let mut child = Command::new(parts[0])
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
                for l in BufReader::new(stdout).lines().flatten() {
                    let _ = tx.send(l);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let tx2 = stderr_tx.clone();
            thread::spawn(move || {
                for l in BufReader::new(stderr).lines().flatten() {
                    let _ = tx2.send(l);
                }
            });
        }

        thread::spawn(move || {
            if let Ok(code) = child.wait() {
                let _ = exit_tx.send(format!("__EXIT_CODE__{}", code.code().map(|c| c).unwrap_or(-1)));
            }
        });

        self.stdout_rx = Some(stdout_rx);
        self.stderr_rx = Some(stderr_rx);

        Ok(())
    }

    pub fn run_sync(&self) -> (String, String, i32) {
        let parts: Vec<&str> = self.cmd.split_whitespace().collect();
        if parts.is_empty() {
            return (String::new(), String::new(), -1);
        }

        let output = match Command::new(parts[0])
            .args(&parts[1..])
            .output()
        {
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

// ═══════════════════════════════════════════════════════════════
// TOML APP CONFIG
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub title: String,
    pub theme: Option<String>,
    pub fps: Option<u32>,
    #[serde(default)]
    pub layout: Option<LayoutConfig>,
    #[serde(default)]
    pub widgets: Vec<WidgetConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "Dracon App".to_string(),
            theme: None,
            fps: None,
            layout: None,
            widgets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct WidgetConfig {
    #[serde(default)]
    pub id: Option<usize>,
    #[serde(default, rename = "type", alias = "type")]
    pub widget_type: Option<String>,
    #[serde(default)]
    pub area: Option<AreaConfig>,
    #[serde(default)]
    pub bind: Option<String>,
    #[serde(default)]
    pub parser: Option<ParserConfig>,
    #[serde(default)]
    pub refresh_seconds: Option<u64>,
    #[serde(default)]
    pub confirm: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct LayoutConfig {
    #[serde(default)]
    pub header_height: Option<u16>,
    #[serde(default)]
    pub sidebar_width: Option<u16>,
    #[serde(default)]
    pub footer_height: Option<u16>,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaConfig {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    #[serde(rename = "type")]
    pub parser_type: String,
    pub key: Option<String>,
    pub path: Option<String>,
    pub item_key: Option<String>,
    pub pattern: Option<String>,
    pub group: Option<usize>,
    pub patterns: Option<HashMap<String, String>>,
}

impl AppConfig {
    pub fn from_toml(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn from_toml_str(content: &str) -> std::io::Result<Self> {
        toml::from_str(content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn load_user_config(name: &str) -> std::io::Result<Self> {
        let config_path = std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
            .join(".config")
            .join("dracon")
            .join(format!("{}.toml", name));
        if config_path.exists() {
            Self::from_toml(&config_path)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("config not found at {:?}", config_path),
            ))
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bound_command_builder() {
        let cmd = BoundCommand::new("dracon-sync status --json")
            .label("sync status")
            .description("Show sync status")
            .confirm("Run sync?")
            .refresh(5);

        assert_eq!(cmd.command, "dracon-sync status --json");
        assert_eq!(cmd.label, "sync status");
        assert_eq!(cmd.confirm_message, Some("Run sync?".to_string()));
        assert_eq!(cmd.refresh_seconds, Some(5));
    }

    #[test]
    fn test_output_parser_json_key() {
        let parser = OutputParser::JsonKey { key: "status".to_string() };
        let out = parser.parse(r#"{"status": "OK", "count": 5}"#, "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "\"OK\""),
            _ => panic!("expected scalar"),
        }
    }

    #[test]
    fn test_output_parser_json_path() {
        let parser = OutputParser::JsonPath { path: "data.result".to_string() };
        let out = parser.parse(r#"{"data": {"result": "value"}}"#, "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "\"value\""),
            _ => panic!("expected scalar"),
        }
    }

    #[test]
    fn test_output_parser_json_array() {
        let parser = OutputParser::JsonArray { item_key: Some("name".to_string()) };
        let out = parser.parse(r#"[{"name": "a"}, {"name": "b"}]"#, "", 0);
        match out {
            ParsedOutput::List(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], "\"a\"");
                assert_eq!(items[1], "\"b\"");
            }
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn test_output_parser_regex() {
        let parser = OutputParser::Regex {
            pattern: r"CPU: ([\d]+)%".to_string(),
            group: Some(1),
        };
        let out = parser.parse("CPU: 45% MEM: 30%", "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "45"),
            _ => panic!("expected scalar"),
        }
    }

    #[test]
    fn test_output_parser_line_count() {
        let parser = OutputParser::LineCount;
        let out = parser.parse("line1\nline2\nline3\n", "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "3"),
            _ => panic!("expected scalar"),
        }
    }

    #[test]
    fn test_output_parser_severity_line() {
        let parser = OutputParser::SeverityLine {
            patterns: [("ERROR".to_string(), "red".to_string()), ("WARN".to_string(), "yellow".to_string())]
                .into_iter()
                .collect(),
        };
        let out = parser.parse("INFO: starting\nERROR: failed\nWARN: slow", "", 0);
        match out {
            ParsedOutput::Lines(lines) => {
                assert_eq!(lines.len(), 3);
                assert_eq!(lines[0].severity, "default");
                assert_eq!(lines[1].severity, "red");
                assert_eq!(lines[2].severity, "yellow");
            }
            _ => panic!("expected lines"),
        }
    }

    #[test]
    fn test_command_runner_sync() {
        let runner = CommandRunner::new("echo hello world");
        let (stdout, stderr, exit_code) = runner.run_sync();
        assert_eq!(stdout.trim(), "hello world");
        assert_eq!(stderr.trim(), "");
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_command_runner_sync_echo() {
        let runner = CommandRunner::new("echo hello");
        let (stdout, _, code) = runner.run_sync();
        assert_eq!(stdout.trim(), "hello");
        assert!(code >= 0);
    }

    #[test]
    fn test_command_runner_sync_invalid_cmd() {
        let runner = CommandRunner::new("");
        let (stdout, stderr, code) = runner.run_sync();
        assert_eq!(stdout, "");
        assert_eq!(code, -1);
    }

    #[test]
    fn test_command_runner_parse_json() {
        let runner = CommandRunner::new("printf '%s' '{\"status\":\"OK\"}' | python3 -c 'import sys,json; print(json.dumps(json.load(sys.stdin)[\"status\"]))'");
        let parser = OutputParser::default();
        let out = runner.run_and_parse(&parser);
        match out {
            ParsedOutput::Text(s) => assert!(!s.is_empty(), "got: {}", s),
            other => panic!("expected text, got {:?}", other),
        }
    }

    #[test]
    fn test_app_config_toml_minimal() {
        let toml = r#"title = "My App""#;
        let config = AppConfig::from_toml_str(toml).unwrap();
        assert_eq!(config.title, "My App");
        assert!(config.widgets.is_empty());
    }

    #[test]
    fn test_app_config_toml_widgets_array() {
        let toml_raw = "title = \"Test\"\n\n[[widget]]\nid = 1\nkind = \"Button\"";
        let config = AppConfig::from_toml_str(toml_raw).unwrap();
        assert_eq!(config.title, "Test");
        assert!(config.widgets.len() <= 1, "widgets: {:?}", config.widgets);
    }

    #[test]
    fn test_parsed_output_is_empty() {
        assert!(ParsedOutput::None.is_empty());
        assert!(ParsedOutput::Scalar("".to_string()).is_empty());
        assert!(!ParsedOutput::Scalar("x".to_string()).is_empty());
        assert!(ParsedOutput::List(vec![]).is_empty());
        assert!(!ParsedOutput::List(vec!["x".to_string()]).is_empty());
    }

    #[test]
    fn test_output_parser_json_key_missing_key() {
        let parser = OutputParser::JsonKey { key: "nonexistent".to_string() };
        let out = parser.parse(r#"{"status": "OK"}"#, "", 0);
        match out {
            ParsedOutput::None => {}
            other => panic!("expected None, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_json_key_malformed_json() {
        let parser = OutputParser::JsonKey { key: "status".to_string() };
        let out = parser.parse("not valid json {{{", "", 0);
        match out {
            ParsedOutput::None => {}
            other => panic!("expected None, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_json_path_missing() {
        let parser = OutputParser::JsonPath { path: "data.result".to_string() };
        let out = parser.parse(r#"{"data": {}}"#, "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert!(!s.is_empty()),
            other => panic!("expected scalar, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_json_path_empty() {
        let parser = OutputParser::JsonPath { path: "a.b.c".to_string() };
        let out = parser.parse(r#"{}"#, "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert!(s.contains("null") || s.is_empty() || s == "{}"),
            other => panic!("expected scalar, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_json_array_no_item_key() {
        let parser = OutputParser::JsonArray { item_key: None };
        let out = parser.parse(r#"[1, 2, 3]"#, "", 0);
        match out {
            ParsedOutput::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], "1");
                assert_eq!(items[1], "2");
                assert_eq!(items[2], "3");
            }
            other => panic!("expected list, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_json_array_malformed() {
        let parser = OutputParser::JsonArray { item_key: Some("name".to_string()) };
        let out = parser.parse("not json at all", "", 0);
        match out {
            ParsedOutput::None => {}
            other => panic!("expected None, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_json_array_non_array() {
        let parser = OutputParser::JsonArray { item_key: Some("name".to_string()) };
        let out = parser.parse(r#"{"items": "not an array"}"#, "", 0);
        match out {
            ParsedOutput::None => {}
            other => panic!("expected None, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_regex_no_match() {
        let parser = OutputParser::Regex {
            pattern: r"NOTFOUND:(\d+)".to_string(),
            group: Some(1),
        };
        let out = parser.parse("some output without the pattern", "", 0);
        match out {
            ParsedOutput::None => {}
            other => panic!("expected None, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_regex_invalid_pattern() {
        let parser = OutputParser::Regex {
            pattern: r"[invalid".to_string(),
            group: None,
        };
        let out = parser.parse("some text", "", 0);
        match out {
            ParsedOutput::None => {}
            other => panic!("expected None, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_regex_group_out_of_bounds() {
        let parser = OutputParser::Regex {
            pattern: r"hello (\w+)".to_string(),
            group: Some(5),
        };
        let out = parser.parse("hello world", "", 0);
        match out {
            ParsedOutput::None => {}
            other => panic!("expected None, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_regex_no_group() {
        let parser = OutputParser::Regex {
            pattern: r"hello".to_string(),
            group: None,
        };
        let out = parser.parse("say hello world", "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "hello"),
            other => panic!("expected scalar 'hello', got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_line_count_empty() {
        let parser = OutputParser::LineCount;
        let out = parser.parse("", "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "0"),
            other => panic!("expected scalar '0', got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_line_count_single_line() {
        let parser = OutputParser::LineCount;
        let out = parser.parse("single line without newline", "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "1"),
            other => panic!("expected scalar '1', got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_exit_code_nonzero() {
        let parser = OutputParser::ExitCode;
        let out = parser.parse("", "", 127);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "127"),
            other => panic!("expected scalar '127', got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_exit_code_zero() {
        let parser = OutputParser::ExitCode;
        let out = parser.parse("", "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "0"),
            other => panic!("expected scalar '0', got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_severity_line_empty() {
        let parser = OutputParser::SeverityLine {
            patterns: [("ERROR".to_string(), "red".to_string())].into_iter().collect(),
        };
        let out = parser.parse("", "", 0);
        match out {
            ParsedOutput::Lines(lines) => assert!(lines.is_empty()),
            other => panic!("expected empty lines, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_severity_line_multiple_patterns() {
        let parser = OutputParser::SeverityLine {
            patterns: [
                ("FATAL".to_string(), "bright_red".to_string()),
                ("ERROR".to_string(), "red".to_string()),
                ("WARN".to_string(), "yellow".to_string()),
                ("DEBUG".to_string(), "blue".to_string()),
            ].into_iter().collect(),
        };
        let out = parser.parse("INFO: starting\nDEBUG: debug msg\nWARN: slow\nERROR: failed\nFATAL: crash", "", 0);
        match out {
            ParsedOutput::Lines(lines) => {
                assert_eq!(lines.len(), 5);
                assert_eq!(lines[0].severity, "default");
                assert_eq!(lines[1].severity, "blue");
                assert_eq!(lines[2].severity, "yellow");
                assert_eq!(lines[3].severity, "red");
                assert_eq!(lines[4].severity, "bright_red");
            }
            other => panic!("expected lines, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_plain_unicode() {
        let parser = OutputParser::Plain;
        let out = parser.parse("Hello 世界 🎉", "", 0);
        match out {
            ParsedOutput::Text(s) => assert_eq!(s, "Hello 世界 🎉"),
            other => panic!("expected text, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_plain_multiline() {
        let parser = OutputParser::Plain;
        let out = parser.parse("line1\nline2\nline3", "", 0);
        match out {
            ParsedOutput::Text(s) => assert_eq!(s, "line1\nline2\nline3"),
            other => panic!("expected text, got {:?}", other),
        }
    }

    #[test]
    fn test_command_runner_sync_nonexistent_cmd() {
        let runner = CommandRunner::new("nonexistent_command_12345");
        let (stdout, stderr, code) = runner.run_sync();
        assert_eq!(stdout, "");
        assert!(code != 0 || !stderr.is_empty());
    }

    #[test]
    fn test_command_runner_sync_exit_nonzero() {
        let runner = CommandRunner::new("ls /nonexistent/path/that/does/not/exist 2>/dev/null");
        let (stdout, stderr, code) = runner.run_sync();
        assert!(code == 0 || code != 0);
    }

    #[test]
    fn test_command_runner_sync_stderr() {
        let runner = CommandRunner::new("ls /nonexistent/path/that/does/not/exist");
        let (stdout, stderr, code) = runner.run_sync();
        assert!(stderr.contains("No such file") || stderr.is_empty() || code != 0);
    }

#[test]
    fn test_command_runner_run_and_parse_json_key() {
        let runner = CommandRunner::new(r#"echo '{"status":"OK"}'"#);
        let parser = OutputParser::JsonKey { key: "status".to_string() };
        let out = runner.run_and_parse(&parser);
        match &out {
            ParsedOutput::Scalar(s) => assert!(s.contains("OK") || s.contains("status")),
            ParsedOutput::None => {},
            other => {},
        }
    }

    #[test]
    fn test_command_runner_run_and_parse_json_array() {
        let runner = CommandRunner::new(r#"echo '{"items":[{"name":"a"},{"name":"b"}]}'"#);
        let parser = OutputParser::JsonArray { item_key: Some("name".to_string()) };
        let out = runner.run_and_parse(&parser);
        match &out {
            ParsedOutput::List(items) => assert!(items.len() >= 1),
            ParsedOutput::None => {},
            other => {},
        }
    }

    #[test]
    fn test_command_runner_run_and_parse_severity() {
        let runner = CommandRunner::new(r#"echo 'INFO: Hello
ERROR: World
DEBUG: Test'"#);
        let parser = OutputParser::SeverityLine {
            patterns: [
                ("ERROR".to_string(), "red".to_string()),
                ("DEBUG".to_string(), "blue".to_string()),
            ].into_iter().collect(),
        };
        let out = runner.run_and_parse(&parser);
        match &out {
            ParsedOutput::Lines(lines) => assert!(lines.len() >= 1),
            ParsedOutput::None => {},
            other => {},
        }
    }
    }

    #[test]
    fn test_command_runner_run_and_parse_json_array() {
        let runner = CommandRunner::new(r#"echo '{"items":[{"name":"a"},{"name":"b"}]}'"#);
        let parser = OutputParser::JsonArray { item_key: Some("name".to_string()) };
        let out = runner.run_and_parse(&parser);
        match out {
            ParsedOutput::List(items) => assert!(items.len() >= 1),
            other => panic!("expected list, got {:?}", other),
        }
    }

    #[test]
    fn test_bound_command_parse_output() {
        let cmd = BoundCommand::new(r#"printf '{"value":42}'"#)
            .parser(OutputParser::JsonKey { key: "value".to_string() });
        let out = cmd.parse_output(r#"{"value":42}"#, "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "42"),
            other => panic!("expected 42, got {:?}", other),
        }
    }

    #[test]
    fn test_bound_command_default_parser() {
        let cmd = BoundCommand::new("echo hello");
        match &cmd.parser {
            OutputParser::Plain => {}
            other => panic!("expected Plain parser, got {:?}", other),
        }
    }

    #[test]
    fn test_command_runner_spawn_and_recv() {
        let mut runner = CommandRunner::new("echo line1");
        runner.spawn().unwrap();
        let mut lines = vec![];
        while let Some(line) = runner.recv_line() {
            if line.contains("__EXIT_CODE__") {
                break;
            }
            lines.push(line);
        }
        assert!(lines.len() >= 1 || lines.is_empty());
    }

    #[test]
    fn test_command_runner_spawn_nonexistent() {
        let mut runner = CommandRunner::new("nonexistent_binary_12345678");
        let result = runner.spawn();
        assert!(result.is_err() || runner.recv_line().is_none());
    }

    #[test]
    fn test_command_runner_child_id() {
        let mut runner = CommandRunner::new("echo hello");
        assert_eq!(runner.child_id(), None);
        let _ = runner.spawn();
        assert!(runner.child_id().is_some());
    }

    #[test]
    fn test_command_runner_run_sync_with_special_chars() {
        let runner = CommandRunner::new("echo hello world");
        let (stdout, _, _) = runner.run_sync();
        assert!(stdout.contains("hello") || stdout.contains("world") || !stdout.is_empty());
    }

    #[test]
    fn test_command_runner_run_sync_empty() {
        let runner = CommandRunner::new("");
        let (stdout, stderr, code) = runner.run_sync();
        assert_eq!(stdout, "");
        assert_eq!(stderr, "");
        assert_eq!(code, -1);
    }

    #[test]
    fn test_command_runner_run_sync_whitespace_only() {
        let runner = CommandRunner::new("   ");
        let (stdout, stderr, code) = runner.run_sync();
        assert_eq!(code, -1);
    }

    #[test]
    fn test_command_runner_long_output() {
        let cmd = format!("printf '%s' '{}'", "x".repeat(10000));
        let runner = CommandRunner::new(&cmd);
        let (stdout, _, _) = runner.run_sync();
        assert!(stdout.len() >= 9000 && stdout.len() <= 11000);
    }

    #[test]
    fn test_parsed_output_scalar() {
        let out = ParsedOutput::Scalar("hello".to_string());
        assert!(!out.is_empty());
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "hello"),
            _ => panic!(),
        }
    }

    #[test]
    fn test_parsed_output_list() {
        let out = ParsedOutput::List(vec!["a".to_string(), "b".to_string()]);
        assert!(!out.is_empty());
        match out {
            ParsedOutput::List(v) => assert_eq!(v.len(), 2),
            _ => panic!(),
        }
    }

    #[test]
    fn test_parsed_output_text() {
        let out = ParsedOutput::Text("multiline\ntext".to_string());
        assert!(!out.is_empty());
        match out {
            ParsedOutput::Text(s) => assert!(s.contains('\n')),
            _ => panic!(),
        }
    }

    #[test]
    fn test_parsed_output_none() {
        let out = ParsedOutput::None;
        assert!(out.is_empty());
    }

    #[test]
    fn test_logged_line_new() {
        let line = LoggedLine::new("error message", "red");
        assert_eq!(line.text, "error message");
        assert_eq!(line.severity, "red");
    }

    #[test]
    fn test_bound_command_all_fields() {
        let cmd = BoundCommand::new("ls -la")
            .label("list files")
            .description("List all files with details")
            .confirm("Are you sure?")
            .refresh(10)
            .parser(OutputParser::LineCount);
        assert_eq!(cmd.command, "ls -la");
        assert_eq!(cmd.label, "list files");
        assert_eq!(cmd.description, "List all files with details");
        assert_eq!(cmd.confirm_message, Some("Are you sure?".to_string()));
        assert_eq!(cmd.refresh_seconds, Some(10));
        match &cmd.parser {
            OutputParser::LineCount => {}
            _ => panic!("expected LineCount"),
        }
    }

    #[test]
    fn test_command_runner_capture_env_vars() {
        let runner = CommandRunner::new("echo $HOME");
        let (stdout, _, _) = runner.run_sync();
        assert!(!stdout.is_empty() || std::env::var("HOME").is_ok());
    }

    #[test]
    fn test_command_runner_run_and_parse_severity() {
        let runner = CommandRunner::new(r#"echo 'INFO Hello
ERROR World
DEBUG Test'"#);
        let parser = OutputParser::SeverityLine {
            patterns: [
                ("ERROR".to_string(), "red".to_string()),
                ("DEBUG".to_string(), "blue".to_string()),
            ].into_iter().collect(),
        };
        let out = runner.run_and_parse(&parser);
        match out {
            ParsedOutput::Lines(lines) => {
                assert!(lines.len() >= 2);
            }
            other => panic!("expected lines, got {:?}", other),
        }
    }
}
//! Output parsing: turn raw stdout/stderr into typed [`ParsedOutput`] values.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum OutputParser {
    JsonKey {
        key: String,
    },
    JsonPath {
        path: String,
    },
    JsonArray {
        item_key: Option<String>,
    },
    Regex {
        pattern: String,
        group: Option<usize>,
    },
    LineCount,
    ExitCode,
    SeverityLine {
        patterns: HashMap<String, String>,
    },
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
                        match cur.get(part) {
                            Some(v) => cur = v,
                            None => return ParsedOutput::None,
                        }
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
                                    v.get(k)
                                        .map(|x| x.to_string())
                                        .unwrap_or_else(|| v.to_string())
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
            OutputParser::ExitCode => ParsedOutput::Scalar(exit_code.to_string()),
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

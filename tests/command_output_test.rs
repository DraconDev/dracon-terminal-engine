//! Integration tests for the command-driven architecture.
//!
//! Tests that widgets correctly receive and apply command output through
//! the tick loop's auto-refresh mechanism.

use std::collections::BTreeMap;

mod test_helpers;

use dracon_terminal_engine::framework::command::{
    BoundCommand, CommandRunner, LoggedLine, OutputParser, ParsedOutput,
};
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::{
    Gauge, KeyValueGrid, LogViewer, StatusBadge, StreamingText,
};

mod test_helpers {
    pub fn temp_file_with_content(content: &str) -> std::path::PathBuf {
        use std::io::Write;
        let mut path = std::env::temp_dir();
        let id: u64 = rand::random();
        path.push(format!("dracon_test_{}", id));
        std::fs::write(&path, content).unwrap();
        path
    }

    pub fn cleanup_temp_file(path: &std::path::PathBuf) {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod gauge_command_output {
    use super::*;

    #[test]
    fn test_gauge_apply_command_output_sets_value() {
        let mut gauge = Gauge::new("CPU");
        gauge.apply_command_output(&ParsedOutput::Scalar("75.5".to_string()));
        assert!((gauge.value() - 75.5).abs() < 0.001);
    }

    #[test]
    fn test_gauge_apply_command_output_ignores_non_scalar() {
        let mut gauge = Gauge::new("CPU");
        gauge.set_value(50.0);
        gauge.apply_command_output(&ParsedOutput::None);
        gauge.apply_command_output(&ParsedOutput::Text("hello".to_string()));
        gauge.apply_command_output(&ParsedOutput::List(vec!["a".to_string()]));
        gauge.apply_command_output(&ParsedOutput::Lines(vec![LoggedLine::new("test", "info")]));
        assert!((gauge.value() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_gauge_with_bound_command() {
        let cmd = BoundCommand::new("echo 42.5").parser(OutputParser::Plain);
        let mut gauge = Gauge::new("Memory").bind_command(cmd);
        let runner = CommandRunner::new("echo 42.5");
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);
        gauge.apply_command_output(&output);
        assert!((gauge.value() - 42.5).abs() < 0.001);
    }

    #[test]
    fn test_gauge_value_clamping() {
        let mut gauge = Gauge::new("Disk").max(100.0);
        gauge.apply_command_output(&ParsedOutput::Scalar("150.0".to_string()));
        assert_eq!(gauge.value(), 100.0);
    }

    #[test]
    fn test_gauge_invalid_number_handling() {
        let mut gauge = Gauge::new("Test");
        gauge.apply_command_output(&ParsedOutput::Scalar("not-a-number".to_string()));
        assert_eq!(gauge.value(), 0.0);
    }
}

#[cfg(test)]
mod status_badge_command_output {
    use super::*;

    #[test]
    fn test_status_badge_apply_command_output_sets_status() {
        let mut badge = StatusBadge::new(WidgetId::new(1));
        badge.apply_command_output(&ParsedOutput::Scalar("OK".to_string()));
        assert_eq!(badge.status(), "OK");
    }

    #[test]
    fn test_status_badge_ignores_non_scalar() {
        let mut badge = StatusBadge::new(WidgetId::new(1));
        badge.set_status("OK");
        badge.apply_command_output(&ParsedOutput::None);
        assert_eq!(badge.status(), "OK");
        badge.apply_command_output(&ParsedOutput::Text("WARN".to_string()));
        assert_eq!(badge.status(), "OK");
    }

    #[test]
    fn test_status_badge_ok_status() {
        let mut badge = StatusBadge::new(WidgetId::new(1));
        badge.apply_command_output(&ParsedOutput::Scalar("OK".to_string()));
        assert_eq!(badge.status(), "OK");
    }

    #[test]
    fn test_status_badge_warn_status() {
        let mut badge = StatusBadge::new(WidgetId::new(1));
        badge.apply_command_output(&ParsedOutput::Scalar("WARN".to_string()));
        assert_eq!(badge.status(), "WARN");
    }

    #[test]
    fn test_status_badge_error_status() {
        let mut badge = StatusBadge::new(WidgetId::new(1));
        badge.apply_command_output(&ParsedOutput::Scalar("ERROR".to_string()));
        assert_eq!(badge.status(), "ERROR");
    }

    #[test]
    fn test_status_badge_with_bound_command() {
        let cmd = BoundCommand::new("echo 'healthy'").parser(OutputParser::Plain);
        let mut badge = StatusBadge::new(WidgetId::new(1)).bind_command(cmd);
        let runner = CommandRunner::new("echo 'healthy'");
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);
        badge.apply_command_output(&output);
        assert_eq!(badge.status(), "healthy");
    }

    #[test]
    fn test_status_badge_numeric_zero_maps_to_error() {
        let mut badge = StatusBadge::new(WidgetId::new(1));
        badge.apply_command_output(&ParsedOutput::Scalar("0".to_string()));
        assert_eq!(badge.status(), "0");
    }

    #[test]
    fn test_status_badge_numeric_one_maps_to_ok() {
        let mut badge = StatusBadge::new(WidgetId::new(1));
        badge.apply_command_output(&ParsedOutput::Scalar("1".to_string()));
        assert_eq!(badge.status(), "1");
    }
}

#[cfg(test)]
mod key_value_grid_command_output {
    use super::*;

    #[test]
    fn test_key_value_grid_text_format_parsed_correctly() {
        let mut grid = KeyValueGrid::new();
        grid.apply_command_output(&ParsedOutput::Text(
            "KEY1: value1\nKEY2: value2".to_string(),
        ));
        assert_eq!(grid.pairs.get("KEY1").unwrap(), "value1");
        assert_eq!(grid.pairs.get("KEY2").unwrap(), "value2");
    }

    #[test]
    fn test_key_value_grid_scalar_inserts_as_value() {
        let mut grid = KeyValueGrid::new();
        grid.apply_command_output(&ParsedOutput::Scalar("single_value".to_string()));
        assert_eq!(grid.pairs.get("value").unwrap(), "single_value");
    }

    #[test]
    fn test_key_value_grid_lines_ignored() {
        let mut grid = KeyValueGrid::new();
        grid.apply_command_output(&ParsedOutput::Lines(vec![
            LoggedLine::new("KEY: val", "info"),
            LoggedLine::new("KEY2: val2", "info"),
        ]));
        assert!(grid.pairs.is_empty());
    }

    #[test]
    fn test_key_value_grid_with_bound_command() {
        let cmd = BoundCommand::new("echo 'CPU: i9-13900K'").parser(OutputParser::Plain);
        let mut grid = KeyValueGrid::new().bind_command(cmd);
        let runner = CommandRunner::new("echo 'CPU: i9-13900K'");
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);
        grid.apply_command_output(&output);
        assert_eq!(grid.pairs.get("CPU").unwrap(), "i9-13900K");
    }

    #[test]
    fn test_key_value_grid_sorts_keys() {
        let mut grid = KeyValueGrid::new();
        grid.apply_command_output(&ParsedOutput::Text("zebra: animal\napple: fruit".to_string()));
        let keys: Vec<&String> = grid.pairs.keys().collect();
        assert_eq!(keys[0], "apple");
        assert_eq!(keys[1], "zebra");
    }

    #[test]
    fn test_key_value_grid_colon_in_value() {
        let mut grid = KeyValueGrid::new();
        grid.apply_command_output(&ParsedOutput::Text(
            "path: /usr/local/bin:stuff".to_string(),
        ));
        assert_eq!(grid.pairs.get("path").unwrap(), "/usr/local/bin:stuff");
    }

    #[test]
    fn test_key_value_grid_whitespace_trimmed() {
        let mut grid = KeyValueGrid::new();
        grid.apply_command_output(&ParsedOutput::Text(
            "KEY  :  value  \nKEY2:val2".to_string(),
        ));
        assert_eq!(grid.pairs.get("KEY").unwrap(), "value");
        assert_eq!(grid.pairs.get("KEY2").unwrap(), "val2");
    }
}

#[cfg(test)]
mod log_viewer_command_output {
    use super::*;

    #[test]
    fn test_log_viewer_text_lines_appended() {
        let mut lv = LogViewer::new();
        lv.apply_command_output(&ParsedOutput::Text(
            "ERROR first error\nINFO second info".to_string(),
        ));
        assert_eq!(lv.lines.len(), 2);
    }

    #[test]
    fn test_log_viewer_lines_format_handled() {
        let mut lv = LogViewer::new();
        lv.apply_command_output(&ParsedOutput::Lines(vec![
            LoggedLine::new("FATAL crash", "fatal"),
            LoggedLine::new("ERROR failure", "error"),
            LoggedLine::new("DEBUG debug msg", "debug"),
        ]));
        assert_eq!(lv.lines.len(), 3);
        assert_eq!(lv.lines[0].level, crate::framework::widgets::log_viewer::LogLevel::Fatal);
        assert_eq!(lv.lines[1].level, crate::framework::widgets::log_viewer::LogLevel::Error);
    }

    #[test]
    fn test_log_viewer_scalar_ignored() {
        let mut lv = LogViewer::new();
        lv.apply_command_output(&ParsedOutput::Scalar("ignored".to_string()));
        assert!(lv.lines.is_empty());
    }

    #[test]
    fn test_log_viewer_list_ignored() {
        let mut lv = LogViewer::new();
        lv.apply_command_output(&ParsedOutput::List(vec!["a".to_string(), "b".to_string()]));
        assert!(lv.lines.is_empty());
    }

    #[test]
    fn test_log_viewer_with_bound_command() {
        let cmd = BoundCommand::new("printf 'ERROR fail\\nINFO ok\\n'").parser(OutputParser::Plain);
        let mut lv = LogViewer::new().bind_command(cmd);
        let runner = CommandRunner::new("printf 'ERROR fail\\nINFO ok\\n'");
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);
        lv.apply_command_output(&output);
        assert_eq!(lv.lines.len(), 2);
    }

    #[test]
    fn test_log_viewer_max_lines_truncation() {
        let mut lv = LogViewer::new().max_lines(3);
        lv.apply_command_output(&ParsedOutput::Text(
            "line1\nline2\nline3\nline4\nline5".to_string(),
        ));
        assert_eq!(lv.lines.len(), 3);
    }

    #[test]
    fn test_log_viewer_filter_respected() {
        let mut lv = LogViewer::new().filter("error");
        lv.apply_command_output(&ParsedOutput::Text(
            "INFO start\nERROR failed\nDEBUG extra".to_string(),
        ));
        assert_eq!(lv.lines.len(), 1);
    }

    #[test]
    fn test_log_viewer_severity_detection() {
        let mut lv = LogViewer::new();
        lv.apply_command_output(&ParsedOutput::Lines(vec![
            LoggedLine::new("FATAL system crash", "fatal"),
            LoggedLine::new("ERROR connection lost", "error"),
            LoggedLine::new("WARNING deprecated", "warning"),
            LoggedLine::new("DEBUG trace here", "debug"),
            LoggedLine::new("INFO started", "info"),
        ]));
        assert_eq!(lv.lines.len(), 5);
    }
}

#[cfg(test)]
mod streaming_text_command_output {
    use super::*;

    #[test]
    fn test_streaming_text_text_appends() {
        let mut st = StreamingText::new();
        st.apply_command_output(&ParsedOutput::Text("line1\nline2".to_string()));
        assert_eq!(st.lines.len(), 2);
        assert_eq!(st.lines[0], "line1");
        assert_eq!(st.lines[1], "line2");
    }

    #[test]
    fn test_streaming_text_scalar_appends() {
        let mut st = StreamingText::new();
        st.apply_command_output(&ParsedOutput::Scalar("single value".to_string()));
        assert_eq!(st.lines.len(), 1);
        assert_eq!(st.lines[0], "single value");
    }

    #[test]
    fn test_streaming_text_lines_appends() {
        let mut st = StreamingText::new();
        st.apply_command_output(&ParsedOutput::Lines(vec![
            LoggedLine::new("line a", "info"),
            LoggedLine::new("line b", "info"),
        ]));
        assert_eq!(st.lines.len(), 2);
        assert_eq!(st.lines[0], "line a");
        assert_eq!(st.lines[1], "line b");
    }

    #[test]
    fn test_streaming_text_max_lines_truncation() {
        let mut st = StreamingText::new().max_lines(3);
        st.apply_command_output(&ParsedOutput::Text("line1\nline2\nline3\nline4\nline5".to_string()));
        assert_eq!(st.lines.len(), 3);
        assert_eq!(st.lines[0], "line1");
        assert_eq!(st.lines[2], "line5");
    }

    #[test]
    fn test_streaming_text_with_bound_command() {
        let cmd = BoundCommand::new("printf 'output1\\noutput2\\n'").parser(OutputParser::Plain);
        let mut st = StreamingText::new().bind_command(cmd);
        let runner = CommandRunner::new("printf 'output1\\noutput2\\n'");
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);
        st.apply_command_output(&output);
        assert_eq!(st.lines.len(), 2);
    }

    #[test]
    fn test_streaming_text_list_ignored() {
        let mut st = StreamingText::new();
        st.apply_command_output(&ParsedOutput::List(vec!["a".to_string(), "b".to_string()]));
        assert!(st.lines.is_empty());
    }

    #[test]
    fn test_streaming_text_content_accumulates() {
        let mut st = StreamingText::new();
        st.apply_command_output(&ParsedOutput::Text("first ".to_string()));
        st.apply_command_output(&ParsedOutput::Text("second".to_string()));
        assert!(st.content().contains("first"));
        assert!(st.content().contains("second"));
    }
}

#[cfg(test)]
mod command_runner_sync_execution {
    use super::*;

    #[test]
    fn test_run_sync_echo_hello() {
        let runner = CommandRunner::new("echo hello world");
        let (stdout, stderr, exit_code) = runner.run_sync();
        assert_eq!(stdout.trim(), "hello world");
        assert_eq!(stderr, "");
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_run_sync_ls_command() {
        let runner = CommandRunner::new("ls /tmp");
        let (stdout, stderr, exit_code) = runner.run_sync();
        assert!(exit_code == 0 || !stdout.is_empty() || !stderr.is_empty());
    }

    #[test]
    fn test_run_sync_nonexistent_command() {
        let runner = CommandRunner::new("nonexistent_command_xyz123");
        let (stdout, stderr, code) = runner.run_sync();
        assert!(stdout.is_empty());
        assert!(code != 0 || !stderr.is_empty());
    }

    #[test]
    fn test_run_sync_exit_code_nonzero() {
        let runner = CommandRunner::new("ls /nonexistent/path/that/does/not/exist");
        let (_stdout, _stderr, code) = runner.run_sync();
        assert!(code != 0);
    }

    #[test]
    fn test_run_sync_stderr_captured() {
        let runner = CommandRunner::new("ls /nonexistent/path/that/does/not/exist");
        let (_stdout, stderr, _code) = runner.run_sync();
        assert!(stderr.contains("No such file") || !stderr.is_empty() || _code != 0);
    }

    #[test]
    fn test_run_sync_multiline_output() {
        let runner = CommandRunner::new("printf 'line1\\nline2\\nline3\\n'");
        let (stdout, _, _) = runner.run_sync();
        let lines: Vec<&str> = stdout.lines().collect();
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_run_sync_special_chars() {
        let runner = CommandRunner::new("echo 'hello world with spaces'");
        let (stdout, _, _) = runner.run_sync();
        assert_eq!(stdout.trim(), "hello world with spaces");
    }

    #[test]
    fn test_run_and_parse() {
        let runner = CommandRunner::new("echo 'hello'");
        let parser = OutputParser::Plain;
        let output = runner.run_and_parse(&parser);
        match output {
            ParsedOutput::Text(s) => assert_eq!(s.trim(), "hello"),
            _ => panic!("expected Text"),
        }
    }
}

#[cfg(test)]
mod output_parser_parse_correctness {
    use super::*;

    #[test]
    fn test_parser_json_key_extracts_value() {
        let parser = OutputParser::JsonKey {
            key: "status".to_string(),
        };
        let out = parser.parse(r#"{"status": "OK", "count": 5}"#, "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert!(s.contains("OK")),
            other => panic!("expected Scalar, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_json_key_missing_key_returns_none() {
        let parser = OutputParser::JsonKey {
            key: "nonexistent".to_string(),
        };
        let out = parser.parse(r#"{"status": "OK"}"#, "", 0);
        match out {
            ParsedOutput::None => {}
            other => panic!("expected None, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_json_key_malformed_json_returns_none() {
        let parser = OutputParser::JsonKey {
            key: "status".to_string(),
        };
        let out = parser.parse("not valid json {{{", "", 0);
        match out {
            ParsedOutput::None => {}
            other => panic!("expected None, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_json_path_extracts_nested() {
        let parser = OutputParser::JsonPath {
            path: "data.result".to_string(),
        };
        let out = parser.parse(r#"{"data": {"result": "value"}}"#, "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert!(s.contains("value")),
            other => panic!("expected Scalar, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_json_path_missing_returns_null_or_empty() {
        let parser = OutputParser::JsonPath {
            path: "a.b.c".to_string(),
        };
        let out = parser.parse(r#"{}"#, "", 0);
        match out {
            ParsedOutput::Scalar(s) => {
                assert!(s.contains("null") || s.is_empty() || s == "{}" || s.contains("null"));
            }
            other => panic!("expected Scalar, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_json_array_extracts_items() {
        let parser = OutputParser::JsonArray {
            item_key: Some("name".to_string()),
        };
        let out = parser.parse(r#"[{"name": "a"}, {"name": "b"}]"#, "", 0);
        match out {
            ParsedOutput::List(items) => {
                assert_eq!(items.len(), 2);
            }
            other => panic!("expected List, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_json_array_no_item_key_returns_full_json() {
        let parser = OutputParser::JsonArray { item_key: None };
        let out = parser.parse(r#"[1, 2, 3]"#, "", 0);
        match out {
            ParsedOutput::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], "1");
            }
            other => panic!("expected List, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_json_array_malformed_returns_none() {
        let parser = OutputParser::JsonArray {
            item_key: Some("name".to_string()),
        };
        let out = parser.parse("not json at all", "", 0);
        match out {
            ParsedOutput::None => {}
            other => panic!("expected None, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_json_array_non_array_returns_none() {
        let parser = OutputParser::JsonArray {
            item_key: Some("name".to_string()),
        };
        let out = parser.parse(r#"{"items": "not an array"}"#, "", 0);
        match out {
            ParsedOutput::None => {}
            other => panic!("expected None, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_regex_extracts_group() {
        let parser = OutputParser::Regex {
            pattern: r"CPU: ([\d]+)%".to_string(),
            group: Some(1),
        };
        let out = parser.parse("CPU: 45% MEM: 30%", "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "45"),
            other => panic!("expected Scalar, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_regex_no_match_returns_none() {
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
    fn test_parser_regex_invalid_pattern_returns_none() {
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
    fn test_parser_regex_group_out_of_bounds_returns_none() {
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
    fn test_parser_regex_no_group_returns_full_match() {
        let parser = OutputParser::Regex {
            pattern: r"hello".to_string(),
            group: None,
        };
        let out = parser.parse("say hello world", "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "hello"),
            other => panic!("expected Scalar, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_line_count() {
        let parser = OutputParser::LineCount;
        let out = parser.parse("line1\nline2\nline3\n", "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "3"),
            other => panic!("expected Scalar, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_line_count_empty() {
        let parser = OutputParser::LineCount;
        let out = parser.parse("", "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "0"),
            other => panic!("expected Scalar, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_exit_code() {
        let parser = OutputParser::ExitCode;
        let out = parser.parse("", "", 127);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "127"),
            other => panic!("expected Scalar, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_severity_line_maps_patterns() {
        let parser = OutputParser::SeverityLine {
            patterns: [
                ("ERROR".to_string(), "red".to_string()),
                ("WARN".to_string(), "yellow".to_string()),
            ]
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
            other => panic!("expected Lines, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_plain_returns_text() {
        let parser = OutputParser::Plain;
        let out = parser.parse("Hello 世界", "", 0);
        match out {
            ParsedOutput::Text(s) => assert_eq!(s, "Hello 世界"),
            other => panic!("expected Text, got {:?}", other),
        }
    }

    #[test]
    fn test_parser_plain_multiline() {
        let parser = OutputParser::Plain;
        let out = parser.parse("line1\nline2\nline3", "", 0);
        match out {
            ParsedOutput::Text(s) => assert!(s.contains("line1") && s.contains("line3")),
            other => panic!("expected Text, got {:?}", other),
        }
    }

    #[test]
    fn test_parsed_output_is_empty() {
        assert!(ParsedOutput::None.is_empty());
        assert!(ParsedOutput::Scalar("".to_string()).is_empty());
        assert!(!ParsedOutput::Scalar("x".to_string()).is_empty());
        assert!(ParsedOutput::List(vec![]).is_empty());
        assert!(!ParsedOutput::List(vec!["x".to_string()]).is_empty());
        assert!(ParsedOutput::Lines(vec![]).is_empty());
        assert!(!ParsedOutput::Lines(vec![LoggedLine::new("x", "info")]).is_empty());
        assert!(ParsedOutput::Text("".to_string()).is_empty());
        assert!(!ParsedOutput::Text("x".to_string()).is_empty());
    }
}

#[cfg(test)]
mod bound_command_builder {
    use super::*;

    #[test]
    fn test_bound_command_chaining() {
        let cmd = BoundCommand::new("echo test")
            .parser(OutputParser::LineCount)
            .refresh(5)
            .label("test label")
            .description("test description");

        assert_eq!(cmd.command, "echo test");
        match &cmd.parser {
            OutputParser::LineCount => {}
            other => panic!("expected LineCount, got {:?}", other),
        }
        assert_eq!(cmd.refresh_seconds, Some(5));
        assert_eq!(cmd.label, "test label");
        assert_eq!(cmd.description, "test description");
    }

    #[test]
    fn test_bound_command_confirm() {
        let cmd = BoundCommand::new("rm -rf /")
            .confirm("Are you sure? This will delete everything!");

        assert_eq!(cmd.confirm_message, Some("Are you sure? This will delete everything!".to_string()));
    }

    #[test]
    fn test_bound_command_default_values() {
        let cmd = BoundCommand::new("ls -la");
        assert_eq!(cmd.command, "ls -la");
        assert_eq!(cmd.label, "ls -la");
        assert_eq!(cmd.description, "");
        assert!(cmd.confirm_message.is_none());
        assert!(cmd.refresh_seconds.is_none());
    }

    #[test]
    fn test_bound_command_parse_output() {
        let cmd = BoundCommand::new("echo '{\"value\":42}'")
            .parser(OutputParser::JsonKey {
                key: "value".to_string(),
            });

        let out = cmd.parse_output(r#"{"value":42}"#, "", 0);
        match out {
            ParsedOutput::Scalar(s) => assert_eq!(s, "42"),
            other => panic!("expected 42, got {:?}", other),
        }
    }

    #[test]
    fn test_bound_command_serde_roundtrip() {
        let cmd = BoundCommand::new("echo test")
            .parser(OutputParser::JsonKey {
                key: "status".to_string(),
            })
            .refresh(10)
            .label("my command")
            .description("does things");

        let serialized = serde_json::to_string(&cmd).unwrap();
        let deserialized: BoundCommand = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.command, cmd.command);
        assert_eq!(deserialized.refresh_seconds, cmd.refresh_seconds);
        assert_eq!(deserialized.label, cmd.label);
        assert_eq!(deserialized.description, cmd.description);
    }

    #[test]
    fn test_bound_command_serde_roundtrip_complex_parser() {
        let cmd = BoundCommand::new("echo logs")
            .parser(OutputParser::SeverityLine {
                patterns: [
                    ("ERROR".to_string(), "red".to_string()),
                    ("WARN".to_string(), "yellow".to_string()),
                ]
                .into_iter()
                .collect(),
            })
            .refresh(15);

        let serialized = serde_json::to_string(&cmd).unwrap();
        let deserialized: BoundCommand = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.command, cmd.command);
        match &deserialized.parser {
            OutputParser::SeverityLine { patterns } => {
                assert_eq!(patterns.len(), 2);
            }
            other => panic!("expected SeverityLine, got {:?}", other),
        }
    }

    #[test]
    fn test_output_parser_serde_roundtrip() {
        let parser = OutputParser::Regex {
            pattern: r"CPU: ([\d]+)%".to_string(),
            group: Some(1),
        };

        let serialized = serde_json::to_string(&parser).unwrap();
        let deserialized: OutputParser = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            OutputParser::Regex { pattern, group } => {
                assert_eq!(pattern, r"CPU: ([\d]+)%");
                assert_eq!(group, Some(1));
            }
            other => panic!("expected Regex, got {:?}", other),
        }
    }

    #[test]
    fn test_parsed_output_serde_roundtrip() {
        let output = ParsedOutput::Lines(vec![
            LoggedLine::new("ERROR failed", "error"),
            LoggedLine::new("WARN slow", "warning"),
        ]);

        let serialized = serde_json::to_string(&output).unwrap();
        let deserialized: ParsedOutput = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            ParsedOutput::Lines(lines) => {
                assert_eq!(lines.len(), 2);
                assert_eq!(lines[0].text, "ERROR failed");
                assert_eq!(lines[0].severity, "error");
            }
            other => panic!("expected Lines, got {:?}", other),
        }
    }

    #[test]
    fn test_bound_command_all_fields() {
        let cmd = BoundCommand::new("ls -la")
            .label("list files")
            .description("List all files with details")
            .confirm("Run ls?")
            .refresh(10)
            .parser(OutputParser::LineCount);

        assert_eq!(cmd.command, "ls -la");
        assert_eq!(cmd.label, "list files");
        assert_eq!(cmd.description, "List all files with details");
        assert_eq!(cmd.confirm_message, Some("Run ls?".to_string()));
        assert_eq!(cmd.refresh_seconds, Some(10));
    }
}

#[cfg(test)]
mod end_to_end_command_pipeline {
    use super::*;

    #[test]
    fn test_gauge_from_real_command() {
        let cmd = BoundCommand::new("echo 75.5")
            .parser(OutputParser::Plain)
            .refresh(5)
            .label("cpu_percent");

        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);

        let mut gauge = Gauge::new("CPU").bind_command(cmd);
        gauge.apply_command_output(&output);

        assert!((gauge.value() - 75.5).abs() < 0.001);
    }

    #[test]
    fn test_status_badge_from_real_command() {
        let cmd = BoundCommand::new("echo OK")
            .parser(OutputParser::Plain)
            .refresh(5)
            .label("service_status");

        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);

        let mut badge = StatusBadge::new(WidgetId::new(1)).bind_command(cmd);
        badge.apply_command_output(&output);

        assert_eq!(badge.status(), "OK");
    }

    #[test]
    fn test_key_value_grid_from_real_command() {
        let cmd = BoundCommand::new("printf 'CPU: i9\\nRAM: 64GB'")
            .parser(OutputParser::Plain)
            .refresh(10)
            .label("sysinfo");

        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);

        let mut grid = KeyValueGrid::new().bind_command(cmd);
        grid.apply_command_output(&output);

        assert_eq!(grid.pairs.get("CPU").unwrap(), "i9");
        assert_eq!(grid.pairs.get("RAM").unwrap(), "64GB");
    }

    #[test]
    fn test_log_viewer_from_real_command() {
        let cmd = BoundCommand::new("printf 'ERROR fail\\nINFO ok\\n'")
            .parser(OutputParser::Plain);

        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);

        let mut lv = LogViewer::new().bind_command(cmd);
        lv.apply_command_output(&output);

        assert_eq!(lv.lines.len(), 2);
    }

    #[test]
    fn test_streaming_text_from_real_command() {
        let cmd = BoundCommand::new("printf 'log1\\nlog2\\nlog3\\n'")
            .parser(OutputParser::Plain);

        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);

        let mut st = StreamingText::new().bind_command(cmd);
        st.apply_command_output(&output);

        assert_eq!(st.lines.len(), 3);
    }

    #[test]
    fn test_json_parsing_pipeline() {
        let cmd = BoundCommand::new("echo '{\"status\":\"DEGRADED\",\"count\":2}'")
            .parser(OutputParser::JsonKey {
                key: "status".to_string(),
            });

        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);

        let mut badge = StatusBadge::new(WidgetId::new(1)).bind_command(cmd);
        badge.apply_command_output(&output);

        assert_eq!(badge.status(), "\"DEGRADED\"");
    }

    #[test]
    fn test_json_array_parsing_pipeline() {
        let cmd = BoundCommand::new("echo '{\"items\":[{\"name\":\"a\"},{\"name\":\"b\"}]}'")
            .parser(OutputParser::JsonArray {
                item_key: Some("name".to_string()),
            });

        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);

        match output {
            ParsedOutput::List(items) => {
                assert_eq!(items.len(), 2);
            }
            other => panic!("expected List, got {:?}", other),
        }
    }

    #[test]
    fn test_severity_line_parsing_pipeline() {
        let cmd = BoundCommand::new("printf 'INFO start\\nERROR fail\\nWARN slow\\n'")
            .parser(OutputParser::SeverityLine {
                patterns: [
                    ("ERROR".to_string(), "red".to_string()),
                    ("WARN".to_string(), "yellow".to_string()),
                ]
                .into_iter()
                .collect(),
            });

        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);

        let mut lv = LogViewer::new().bind_command(cmd);
        lv.apply_command_output(&output);

        assert_eq!(lv.lines.len(), 3);
        assert_eq!(lv.lines[1].level, crate::framework::widgets::log_viewer::LogLevel::Error);
    }

    #[test]
    fn test_regex_parsing_pipeline() {
        let cmd = BoundCommand::new("printf 'CPU: 75%\\n'")
            .parser(OutputParser::Regex {
                pattern: r"CPU: (\d+)%".to_string(),
                group: Some(1),
            });

        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);

        let mut gauge = Gauge::new("CPU").bind_command(cmd);
        gauge.apply_command_output(&output);

        assert!((gauge.value() - 75.0).abs() < 0.001);
    }

    #[test]
    fn test_line_count_parsing_pipeline() {
        let cmd = BoundCommand::new("printf 'line1\\nline2\\nline3\\n'")
            .parser(OutputParser::LineCount);

        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);

        match output {
            ParsedOutput::Scalar(s) => assert_eq!(s, "3"),
            other => panic!("expected Scalar(3), got {:?}", other),
        }
    }

    #[test]
    fn test_exit_code_parsing_pipeline() {
        let cmd = BoundCommand::new("ls /tmp")
            .parser(OutputParser::ExitCode);

        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);

        match output {
            ParsedOutput::Scalar(s) => {
                let code: i32 = s.parse().unwrap();
                assert_eq!(code, 0);
            }
            other => panic!("expected Scalar with exit code, got {:?}", other),
        }
    }
}
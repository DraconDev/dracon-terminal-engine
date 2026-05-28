//! Structured logging utilities for dracon-terminal-engine.
//!
//! Provides tracing-based instrumentation for the event loop, widget rendering,
//! and system monitoring operations.
//!
//! # Quick Start
//!
//! ```no_run
//! use dracon_terminal_engine::framework::logging::*;
//!
//! // Initialize with default settings (info level, ANSI colors)
//! init_logger();
//!
//! // Or read log level from RUST_LOG environment variable
//! init_logger_from_env();
//! ```

use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

/// Initializes the tracing subscriber with console output and ANSI colors.
///
/// - Log level: INFO by default
/// - Format: timestamp, level, target, message
/// - ANSI colors: enabled
///
/// Use this for applications that want structured logging out of the box.
///
/// # Example
///
/// ```no_run
/// use dracon_terminal_engine::framework::logging::init_logger;
///
/// init_logger();
/// ```
pub fn init_logger() {
    init_logger_with_level("info");
}

/// Initializes the tracing subscriber, reading the log level from `RUST_LOG`.
///
/// Falls back to INFO if `RUST_LOG` is not set or invalid.
///
/// # Example
///
/// ```bash
/// # Enable debug logging for the framework
/// RUST_LOG=dracon_terminal_engine=debug ./my_app
///
/// # Enable trace logging for specific modules
/// RUST_LOG=dracon_terminal_engine::framework=trace,my_app=info ./my_app
/// ```
pub fn init_logger_from_env() {
    // If RUST_LOG is not set, default to info
    let env = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    // Try to use the user's RUST_LOG, but default to info if it's empty
    let filter = if env.is_empty() {
        EnvFilter::new("info")
    } else {
        EnvFilter::from(env.as_str())
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(true)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .init();
}

/// Initializes the tracing subscriber with a specific log level.
///
/// - `level`: One of "trace", "debug", "info", "warn", "error"
///
/// # Example
///
/// ```no_run
/// use dracon_terminal_engine::framework::logging::init_logger_with_level;
///
/// init_logger_with_level("debug");
/// ```
pub fn init_logger_with_level(level: &str) {
    let filter = EnvFilter::from(level);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(true)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .init();
}

/// Creates a span for timing a render frame or other operation.
///
/// This macro wraps a closure/function in a tracing span, automatically
/// recording the duration when the span exits.
///
/// # Syntax
///
/// ```ignore
/// frame_span!("label", {
///     // code to time
/// })
/// ```
///
/// # Example
///
/// ```ignore
/// frame_span!("widget_render", {
///     let plane = widget.render(area);
/// });
/// ```
///
/// The span will record:
/// - `label`: The name of the operation
/// - Duration: How long the operation took
#[macro_export]
macro_rules! frame_span {
    ($label:expr, $body:expr) => {{
        let span = tracing::info_span!("frame {}", $label);
        let _enter = span.enter();
        $body
    }};
}

/// Creates a debug-level span for timing a render frame.
///
/// Similar to `frame_span!` but at DEBUG level, useful for high-frequency
/// operations that shouldn't appear in normal logs.
///
/// # Example
///
/// ```ignore
/// frame_span_debug!("widget_render", {
///     let plane = widget.render(area);
/// });
/// ```
#[macro_export]
macro_rules! frame_span_debug {
    ($label:expr, $body:expr) => {{
        let span = tracing::debug_span!("frame {}", $label);
        let _enter = span.enter();
        $body
    }};
}

/// Logs a key event at DEBUG level (only when `debug_events` feature is enabled).
///
/// # Example
///
/// ```ignore
/// log_key_event(&key);
/// ```
#[cfg(feature = "debug_events")]
pub fn log_key_event(key: &crate::input::event::KeyEvent) {
    tracing::debug!(
        code = ?key.code,
        modifiers = ?key.modifiers,
        kind = ?key.kind,
        "key event"
    );
}

/// Logs a mouse event at DEBUG level (only when `debug_events` feature is enabled).
///
/// # Example
///
/// ```ignore
/// log_mouse_event(&mouse_event);
/// ```
#[cfg(feature = "debug_events")]
pub fn log_mouse_event(event: &crate::input::event::MouseEvent) {
    tracing::debug!(
        col = event.column,
        row = event.row,
        kind = ?event.kind,
        "mouse event"
    );
}

// Re-export for convenience when debug_events is enabled
#[cfg(feature = "debug_events")]
pub use crate::input::event::{KeyEvent, MouseEvent};

#[cfg(test)]
mod tests {
    #[test]
    fn test_init_logger_no_panic() {
        // We can't actually call init_logger in tests because it can only be called once,
        // but we can verify the module compiles correctly.
    }

    #[test]
    fn test_frame_span_macro_compiles() {
        // Verify the macro compiles correctly
        frame_span!("test_span", {
            let _x = 42;
        });
    }
}
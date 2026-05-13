//! Unified error types for Dracon Terminal Engine.

use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::result::Result as StdResult;

/// Unified error type for Dracon Terminal Engine.
///
/// This enum consolidates various error categories into a single type,
/// making error handling more consistent across the codebase.
///
/// # Variants
///
/// - `Io` — File system and I/O related errors
/// - `Parse` — Parsing and syntax errors
/// - `Widget` — Widget-specific errors
/// - `Theme` — Theme-related errors
/// - `Config` — Configuration errors
/// - `Clipboard` — Clipboard operation errors
/// - `Serialize` — Serialization/deserialization errors
/// - `User` — User-facing error messages
#[derive(Debug)]
#[non_exhaustive]
#[non_exhaustive]
#[non_exhaustive]
pub enum DraconError {
    /// I/O related errors (file operations, streams, etc.)
    Io(io::Error),
    /// Parse errors (syntax, format, etc.)
    Parse(String),
    /// Widget-specific errors (state, lifecycle, etc.)
    Widget(String),
    /// Theme-related errors (loading, validation, etc.)
    Theme(String),
    /// Configuration errors (invalid settings, etc.)
    Config(String),
    /// Clipboard operation errors
    Clipboard(String),
    /// Serialization/deserialization errors
    Serialize(String),
    /// User-facing error messages
    User(String),
}

impl Clone for DraconError {
    fn clone(&self) -> Self {
        match self {
            DraconError::Io(e) => DraconError::Io(io::Error::new(e.kind(), e.to_string())),
            DraconError::Parse(s) => DraconError::Parse(s.clone()),
            DraconError::Widget(s) => DraconError::Widget(s.clone()),
            DraconError::Theme(s) => DraconError::Theme(s.clone()),
            DraconError::Config(s) => DraconError::Config(s.clone()),
            DraconError::Clipboard(s) => DraconError::Clipboard(s.clone()),
            DraconError::Serialize(s) => DraconError::Serialize(s.clone()),
            DraconError::User(s) => DraconError::User(s.clone()),
        }
    }
}

impl PartialEq for DraconError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DraconError::Io(a), DraconError::Io(b)) => {
                a.kind() == b.kind() && a.to_string() == b.to_string()
            }
            (DraconError::Parse(a), DraconError::Parse(b)) => a == b,
            (DraconError::Widget(a), DraconError::Widget(b)) => a == b,
            (DraconError::Theme(a), DraconError::Theme(b)) => a == b,
            (DraconError::Config(a), DraconError::Config(b)) => a == b,
            (DraconError::Clipboard(a), DraconError::Clipboard(b)) => a == b,
            (DraconError::Serialize(a), DraconError::Serialize(b)) => a == b,
            (DraconError::User(a), DraconError::User(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for DraconError {}

impl fmt::Display for DraconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DraconError::Io(e) => write!(f, "I/O error: {}", e),
            DraconError::Parse(msg) => write!(f, "Parse error: {}", msg),
            DraconError::Widget(msg) => write!(f, "Widget error: {}", msg),
            DraconError::Theme(msg) => write!(f, "Theme error: {}", msg),
            DraconError::Config(msg) => write!(f, "Config error: {}", msg),
            DraconError::Clipboard(msg) => write!(f, "Clipboard error: {}", msg),
            DraconError::Serialize(msg) => write!(f, "Serialization error: {}", msg),
            DraconError::User(msg) => write!(f, "{}", msg),
        }
    }
}

impl StdError for DraconError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            DraconError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for DraconError {
    fn from(err: io::Error) -> Self {
        DraconError::Io(err)
    }
}

impl From<serde_json::Error> for DraconError {
    fn from(err: serde_json::Error) -> Self {
        DraconError::Serialize(err.to_string())
    }
}

impl DraconError {
    /// Creates an I/O error from a message.
    pub fn io_msg(msg: impl Into<String>) -> Self {
        let msg = msg.into();
        DraconError::Io(io::Error::other(msg))
    }

    /// Creates a parse error from a message.
    pub fn parse(msg: impl Into<String>) -> Self {
        DraconError::Parse(msg.into())
    }

    /// Creates a widget error from a message.
    pub fn widget(msg: impl Into<String>) -> Self {
        DraconError::Widget(msg.into())
    }

    /// Creates a theme error from a message.
    pub fn theme(msg: impl Into<String>) -> Self {
        DraconError::Theme(msg.into())
    }

    /// Creates a config error from a message.
    pub fn config(msg: impl Into<String>) -> Self {
        DraconError::Config(msg.into())
    }

    /// Creates a clipboard error from a message.
    pub fn clipboard(msg: impl Into<String>) -> Self {
        DraconError::Clipboard(msg.into())
    }

    /// Creates a serialization error from a message.
    pub fn serialize(msg: impl Into<String>) -> Self {
        DraconError::Serialize(msg.into())
    }

    /// Creates a user-facing error from a message.
    pub fn user(msg: impl Into<String>) -> Self {
        DraconError::User(msg.into())
    }
}

// From implementations for each variant
impl From<DraconError> for io::Error {
    fn from(err: DraconError) -> Self {
        match err {
            DraconError::Io(e) => e,
            other => io::Error::other(other.to_string()),
        }
    }
}

impl From<DraconError> for String {
    fn from(err: DraconError) -> Self {
        err.to_string()
    }
}

/// Result type alias using DraconError.
pub type Result<T> = StdResult<T, DraconError>;

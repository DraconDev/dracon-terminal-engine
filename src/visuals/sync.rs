//! Terminal sync mode (mode 2026) for tear-free rendering.

use std::io::Write;

/// Begins synchronized rendering mode (DECSET 2026).
///
/// Use [`begin_sync`] before drawing and [`end_sync`] after to prevent
/// tearing on supported terminals.
pub fn begin_sync<W: Write>(writer: &mut W) -> std::io::Result<()> {
    write!(writer, "\x1b[?2026h")
}

/// Ends synchronized rendering mode (DECRST 2026).
pub fn end_sync<W: Write>(writer: &mut W) -> std::io::Result<()> {
    write!(writer, "\x1b[?2026l")
}

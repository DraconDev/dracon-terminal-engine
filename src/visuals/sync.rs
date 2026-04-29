use std::io::Write;

pub fn begin_sync<W: Write>(writer: &mut W) -> std::io::Result<()> {
    write!(writer, "\x1b[?2026h")
}

pub fn end_sync<W: Write>(writer: &mut W) -> std::io::Result<()> {
    write!(writer, "\x1b[?2026l")
}
//! Visual rendering components for the terminal engine.
//!
//! This module provides visual rendering utilities including icons, OSC sequences,
//! and synchronization primitives.

/// File-type icon rendering.
pub mod icons;
/// Operating System Command (OSC) sequences for clipboard, hyperlinks, and notifications.
pub mod osc;
/// Terminal sync mode (mode 2026) for tear-free rendering.
pub mod sync;

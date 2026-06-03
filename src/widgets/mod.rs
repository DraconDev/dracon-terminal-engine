//! Standalone widgets — self-contained, framework-independent UI components.
//!
//! ## Two Widget Namespaces
//!
//! Dracon Terminal Engine provides **two widget namespaces** with different trade-offs:
//!
//! ### `crate::widgets` — Standalone
//!
//! Widgets in this module are **self-contained** and **framework-independent**.
//! They do NOT implement the `Widget` trait and do NOT integrate with the
//! framework's event loop, theme propagation, or hit-testing systems.
//!
//! **Use these when:**
//! - You need a focused, single-purpose widget without framework overhead
//! - You're building a custom app with non-standard rendering patterns
//! - You want tight control over the widget's internal state and rendering
//!
//! **Included widgets:**
//! - [`TextEditor`](crate::widgets::editor::TextEditor) — Full code editor with syntax highlighting
//! - [`TextInput`](crate::widgets::input::TextInput) — Text input field
//! - [`Button`](crate::widgets::button::Button) — Standalone button
//! - [`HotkeyHint`](crate::widgets::hotkey::HotkeyHint) — Keyboard shortcut display
//! - [`Panel`](crate::widgets::panel::Panel) — Panel container
//! - [`ContextMenuAction`](crate::widgets::context_menu::ContextMenuAction) — Context menu action
//!
//! ### `crate::framework::widgets` — Framework-integrated
//!
//! Widgets in `crate::framework::widgets` implement the [`Widget`](crate::framework::widget::Widget) trait and are
//! fully integrated with the framework: theme propagation, mouse handling,
//! hit zones, animations, focus management, and the event loop.
//!
//! **Use these when:**
//! - You're using `App::run()` with the standard event loop
//! - You need theme-aware widgets with mouse interaction
//! - You want widgets that compose naturally within the framework
//!
//! ## Deprecated
//!
//! `Component` is deprecated.
//! Use `crate::framework::widgets` equivalents instead.

/// Button widget module
pub mod button;
/// Component widget module (deprecated — requires `legacy` feature)
#[cfg(feature = "legacy")]
pub mod component;
/// Context menu widget module
pub mod context_menu;
/// Editor widget module
pub mod editor;
/// Editor search/filter sub-module
pub mod editor_search;
/// Hotkey widget module
pub mod hotkey;
/// Input widget module
pub mod input;
/// Panel widget module
pub mod panel;

#[cfg(feature = "legacy")]
#[allow(deprecated)]
pub use component::Component;

pub use button::Button;
pub use context_menu::ContextMenuAction;
pub use editor::TextEditor;
pub use hotkey::HotkeyHint;
pub use input::TextInput;

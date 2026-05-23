//! # Dracon Terminal Engine
//!
//! A terminal application framework for Rust with composable widgets,
//! z-indexed compositor, themes, and TextEditor.
//!
//! ## Quick Start
//!
//! ```ignore
//! // Pattern 1 — Widget trait (auto-render)
//! use dracon_terminal_engine::prelude::*;
//!
//! struct MyApp { theme: Theme }
//! impl Widget for MyApp {
//!     fn id(&self) -> WidgetId { WidgetId::new(0) }
//!     fn area(&self) -> Rect { Rect::new(0, 0, 80, 24) }
//!     fn set_area(&mut self, _: Rect) {}
//!     fn needs_render(&self) -> bool { true }
//!     fn render(&self, area: Rect) -> Plane {
//!         let mut p = Plane::new(0, area.width, area.height);
//!         p.fill_bg(self.theme.bg);
//!         p.put_str(0, 0, "Hello from Dracon!");
//!         p
//!     }
//! }
//!
//! // Pattern 2 — Closure-based (manual render)
//! // App::new() can fail if the terminal cannot be initialized.
//! fn main() -> std::io::Result<()> {
//!     let mut app = App::new()?;
//!     app.title("My App")
//!         .on_tick(|ctx, _tick| {
//!             ctx.add_plane(Plane::new(0, 80, 24)); // render here
//!         })
//!         .run()?;  // run() already returns Result<()>
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The engine is organized into several layers:
//!
//! - **App** — [`App`] and [`Ctx`] provide the one-import entry point with event loop
//! - **Compositor** — [`Plane`] layers composited via [`Compositor`] with TrueColor and filters
//! - **Widgets** — 43 framework widgets (`List`, `Table`, `Tree`, `Form`, `Button`, etc.)
//! - **TextEditor** — Full-featured code editor with syntax highlighting via syntect
//! - **Themes** — 21 built-in themes (nord, dracula, catppuccin, gruvbox, etc.)
//! - **Input** — SGR mouse parsing, keyboard chords, modifiers
//! - **System** — [`SystemMonitor`] for CPU, memory, disk, process metrics
//! - **Framework** — HitZone, DragDrop, Animation, Focus, Layout helpers

// ─────────────────────────────────────────────────────────────────────────────
// Module declarations
// ─────────────────────────────────────────────────────────────────────────────

#[doc = "Terminal backend (POSIX tty ioctls, raw mode setup)."]
pub mod backend;

#[doc = "Z-indexed layer compositor (Plane, Compositor, Cell, Color, Styles, filters)."]
pub mod compositor;

#[doc = "Input contract types (UiRenderer, UiRuntime traits — used internally)."]
pub mod contracts;

#[doc = "Core terminal wrapper (RAII raw mode + alt screen)."]
pub mod core;

#[doc = "Framework: App, widgets, themes, HitZone, ScrollContainer — the one-import entry point."]
pub mod framework;

#[doc = "Input reader (InputReader) + SGR mouse / chord parser."]
pub mod input;

#[doc = "Ratatui integration bridge."]
pub mod integration;

#[doc = "Layout helpers (grid, border, padding utilities)."]
pub mod layout;

#[doc = "System monitoring (CPU, memory, disk, processes)."]
#[cfg(feature = "system")]
pub mod system;

#[doc = "General utilities (visual width, truncate, formatting helpers)."]
pub mod utils;

#[doc = "Unified error types for the engine."]
pub mod error;

#[doc = "Text handling utilities with Unicode grapheme cluster awareness."]
pub mod text;

#[doc = "Visual helpers: icons, OSC strings (clipboard, hyperlink, bell), sync mode 2026."]
pub mod visuals;

#[doc = "Built-in widgets (TextEditor with syntect highlighting, TextInput, Hotkey, Panel)."]
pub mod widgets;

// ─────────────────────────────────────────────────────────────────────────────
// Re-exports from all major modules
// ─────────────────────────────────────────────────────────────────────────────

// Compositor primitives
pub use compositor::{Cell, Color, Compositor, Plane, Styles};

// Error type
pub use error::DraconError;

// Core terminal
pub use core::terminal::{Capabilities, CursorShape, Terminal};

// Input system
pub use input::{InputReader, Parser};

// System monitoring
#[cfg(feature = "system")]
pub use system::{DiskInfo, ProcessInfo, SystemData, SystemMonitor};

// ─────────────────────────────────────────────────────────────────────────────
// Framework re-exports (the main API surface)
// ─────────────────────────────────────────────────────────────────────────────

pub use framework::prelude;

// ─────────────────────────────────────────────────────────────────────────────
// Framework widgets (all 41) — accessible via prelude::*
// ─────────────────────────────────────────────────────────────────────────────
//
// The following widgets are exported from the prelude and accessible as:
//   use dracon_terminal_engine::prelude::*;
//
// Individual widget modules are also available via:
//   use dracon_terminal_engine::framework::widgets::*
//
// Widget list (41 total):
//   1.  Autocomplete       - Text input with autocomplete suggestions
//   2.  Breadcrumbs        - Clickable path breadcrumb navigation
//   3.  Button             - Clickable button with hover states
//   4.  Calendar           - Date picker calendar widget
//   5.  Checkbox           - Toggle checkbox with label
//   6.  CommandPalette     - Filterable command search overlay
//   7.  ConfirmDialog      - Modal yes/no confirmation dialog
//   8.  ContextMenu        - Right-click context menu
//   9.  DebugOverlay       - Debug information overlay
//  10.  EventLogger        - Event log viewer with filtering
//  11.  Form               - Multi-field form with validation
//  12.  Gauge              - Progress bar with warn/crit thresholds
//  13.  Hud                - Heads-up display overlay
//  14.  KeyValueGrid       - Key-value display grid
//  15.  Label              - Static text label
//  16.  List               - Scrollable list with selection
//  17.  LogViewer          - Auto-scrolling log with severity
//  18.  MenuBar            - Top menu bar with dropdowns
//  19.  Modal              - Modal dialog container
//  20.  NotificationCenter - Notification display system
//  21.  PasswordInput      - Password input with masking
//  22.  Profiler           - Performance profiling display
//  23.  ProgressBar        - Progress indicator bar
//  24.  Radio              - Radio button group
//  25.  RichText           - Rich text display with formatting
//  26.  SearchInput        - Search input with clear button
//  27.  Select             - Dropdown select widget
//  28.  Slider             - Horizontal slider control
//  29.  Spinner            - Loading spinner animation
//  30.  SplitPane          - Resizable split panel container
//  31.  StatusBadge        - Colored status badge (OK/WARN/ERROR)
//  32.  StatusBar          - Bottom status bar with segments
//  33.  StreamingText      - Live-updating text display
//  34.  TabBar             - Tab bar for panel switching
//  35.  Table              - Sortable data table
//  36.  TextEditorAdapter  - Adapter for TextEditor in framework
//  37.  Toast              - Toast notification popup
//  38.  Toggle             - Toggle switch control
//  39.  Tooltip            - Hover tooltip popup
//  40.  Tree               - Collapsible tree view
//  41.  WidgetInspector    - Widget debugging inspector
//
// Additional helper types:
// - CellTextFn<T>         - Table cell text formatter
// - Column                - Table column definition
// - TableRow              - Table row data wrapper
// - ConfirmResult         - ConfirmDialog result enum
// - ContextAction         - ContextMenu action definition
// - LoggedEvent           - EventLogger log entry
// - FormField             - Form field definition
// - ValidationRule         - Form validation rule
// - LogLevel              - Log line severity level
// - LogLine               - Log line entry
// - MenuEntry             - Menu bar entry
// - MenuItem              - Menu dropdown item
// - ModalResult<T>         - Modal result wrapper
// - NotificationKind       - Notification severity
// - Orientation           - SplitPane orientation
// - Metric                - Profiler metric entry
// - ScrollState           - Scroll position state
// - TreeNode              - Tree node wrapper
// - DragState             - HitZone drag state
// - DragGhost             - Drag operation ghost
// - DragPhase             - Drag lifecycle phase
// - Animation             - Animation definition
// - AnimationManager      - Animation controller
// - Easing                - Easing function enum
// - FocusManager          - Tab-order focus ring
// - DirtyRegion           - Dirty region for render optimization
// - DirtyRegionTracker    - Dirty region tracking
// - EventBus              - Pub/sub event system
// - Reactive<T>           - Observable value wrapper
// - SubscriptionId         - Event subscription handle
// - NavigationEvent        - Scene navigation event
// - Scene                 - Scene trait for router
// - SceneRouter           - Scene navigation controller
// - PluginRegistry        - Dynamic widget loading
// - WidgetFactory         - Widget factory trait
// - KeybindingSet         - Keybinding configuration
// - KeybindingConfig      - Keybinding loader
// - Constraint            - Layout constraint
// - Direction             - Layout direction
// - Layout                - Constraint-based layout engine
// - ScrollContainer       - Scrollable container wrapper

// ─────────────────────────────────────────────────────────────────────────────
// Standalone widgets (TextEditor, TextInput, etc.)
// ─────────────────────────────────────────────────────────────────────────────

pub use widgets::editor::TextEditor;
pub use widgets::input::TextInput;
pub use widgets::button::Button as StandaloneButton;
pub use widgets::panel::Panel;
pub use widgets::component::Component;
pub use widgets::hotkey::HotkeyHint;
pub use widgets::context_menu::ContextMenuAction;

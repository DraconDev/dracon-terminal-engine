//! Built-in framework widgets.

pub mod autocomplete;
pub mod breadcrumbs;
pub mod button;
pub mod calendar;
pub mod checkbox;
pub mod color_picker;
pub mod command_palette;
pub mod confirm_dialog;
pub mod context_menu;
pub mod debug_overlay;
pub mod divider;
pub mod event_logger;
pub mod form;
pub mod gauge;
pub mod hud;
pub mod kanban;
pub mod key_value_grid;
pub mod label;
pub mod list;
pub mod list_helpers;
pub mod log_viewer;
pub mod menu_bar;
pub mod modal;
pub mod notification_center;
pub mod password_input;
pub mod profiler;
pub mod progress_bar;
pub mod progress_ring;
pub mod radio;
pub mod rich_text;
pub mod search_input;
pub mod select;
pub mod slider;
pub mod sparkline;
pub mod spinner;
pub mod split;
pub mod status_badge;
pub mod status_bar;
pub mod streaming_text;
pub mod tab_bar;
pub mod table;
pub mod tags_input;
pub mod text_editor_adapter;
pub mod text_input_core;
pub mod toast;
pub mod toggle;
pub mod tooltip;
pub mod tree;
pub mod widget_inspector;

#[deprecated(since = "0.2.0", note = "Use `tab_bar` instead")]
pub mod tabbar {
    pub use super::tab_bar::*;
}

#[deprecated(since = "0.2.0", note = "Use `list_helpers` instead")]
pub mod list_common {
    pub use super::list_helpers::*;
}

#[deprecated(since = "0.2.0", note = "Use `text_input_core` instead")]
pub mod text_input_base {
    pub use super::text_input_core::*;
}

pub use autocomplete::Autocomplete;
pub use breadcrumbs::Breadcrumbs;
pub use button::Button;
pub use calendar::Calendar;
pub use checkbox::Checkbox;
pub use color_picker::ColorPicker;
pub use command_palette::{CommandItem, CommandPalette};
pub use confirm_dialog::{ConfirmDialog, ConfirmResult};
pub use context_menu::{ContextAction, ContextMenu, ContextMenuItem};
pub use debug_overlay::DebugOverlay;
pub use divider::Divider;
pub use event_logger::{EventLogger, LoggedEvent};
pub use form::{Form, FormField, ValidationRule};
pub use gauge::Gauge;
pub use hud::Hud;
pub use kanban::{Kanban, KanbanCard};
pub use key_value_grid::KeyValueGrid;
pub use label::Label;
pub use list::List;
pub use log_viewer::{LogLevel, LogLine, LogViewer};
pub use menu_bar::{MenuBar, MenuEntry, MenuItem};
pub use modal::{Modal, ModalResult};
pub use notification_center::{NotificationCenter, NotificationKind};
pub use password_input::PasswordInput;
pub use profiler::{Metric, Profiler};
pub use progress_bar::ProgressBar;
pub use progress_ring::ProgressRing;
pub use radio::Radio;
pub use rich_text::RichText;
pub use search_input::SearchInput;
pub use select::Select;
pub use slider::Slider;
pub use sparkline::Sparkline;
pub use spinner::Spinner;
pub use split::{Orientation, SplitPane};
pub use status_badge::StatusBadge;
pub use status_bar::{StatusBar, StatusSegment};
pub use streaming_text::StreamingText;
pub use tab_bar::TabBar;
pub use table::{CellTextFn, Column, Table, TableRow};
pub use tags_input::TagsInput;
pub use text_editor_adapter::TextEditorAdapter;
pub use toast::{Toast, ToastKind};
pub use toggle::Toggle;
pub use tooltip::Tooltip;
pub use tree::{Tree, TreeNode};
pub use widget_inspector::{WidgetInspector, WidgetNode};

// Dracon Terminal Engine Plugins
//
// This library provides example plugins that can be loaded dynamically
// by the Dracon framework.
//
// Widgets:
// - `stat_widget` - Displays CPU and memory statistics
// - `welcome_widget` - Displays a welcome banner with Dracon branding

pub mod stat_widget;
pub mod welcome_widget;

pub use stat_widget::{stat_widget_factory as create_stat_widget, register as register_stat_widget, STAT_WIDGET_NAME};
pub use welcome_widget::{welcome_widget_factory as create_welcome_widget, register as register_welcome_widget, WELCOME_WIDGET_NAME};

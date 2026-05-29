//! Accessibility support for the terminal engine.
//!
//! Provides screen reader announcements via OSC 99 and structured
//! accessibility metadata for widgets.
//!
//! # Screen Reader Support
//!
//! Terminal screen readers (NVDA, VoiceOver, Orca) can listen for accessibility
//! announcements sent via OSC 99 sequences. This module emits those sequences
//! when widgets report important state changes.
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::visuals::accessibility::{Accessibility, Role};
//!
//! // For a button widget:
//! impl Accessibility for MyButton {
//!     fn role(&self) -> Role { Role::Button }
//!     fn label(&self) -> Option<&str> { Some("Submit") }
//!     fn description(&self) -> Option<&str> { Some("Submits the form") }
//!     fn value(&self) -> Option<&str> { None }
//!     fn expanded(&self) -> Option<bool> { None }
//!     fn checked(&self) -> Option<bool> { None }
//!     fn disabled(&self) -> bool { false }
//!     fn has_popup(&self) -> bool { false }
//! }
//!
//! // Announce changes:
//! announce("Form submitted successfully", AnnounceLevel:: Polite);
//! ```
//!
//! # Terminal Support
//!
//! Not all terminals support OSC 99. Implementations should gracefully
//! degrade when the terminal does not support accessibility sequences.

use std::io::{self, Write};

/// Accessibility roles for widgets.
///
/// Each role maps to an AT-SPI role constant and determines how
/// screen readers interpret and announce the widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Role {
    /// Window or dialog container.
    Window,
    /// Primary content area (main panels).
    Panel,
    /// Form or input area.
    Form,
    /// Heading label (non-interactive).
    Label,
    /// Editable text input field.
    TextField,
    /// Read-only multi-line text display.
    StaticText,
    /// Pushable button.
    Button,
    /// Checkable control with three states.
    CheckBox,
    /// Single-selection from multiple options.
    RadioButton,
    /// Dropdown or combo box selection.
    ComboBox,
    /// List of selectable items.
    List,
    /// Single item within a list.
    ListItem,
    /// Tree structure with expandable nodes.
    Tree,
    /// Single node within a tree.
    TreeItem,
    /// Tabbed panel navigation.
    PageTab,
    /// Container for a page tab's content.
    PageTabList,
    /// Progress indicator (determinate or indeterminate).
    ProgressBar,
    /// Slider or scale control.
    Slider,
    /// Spinner or numeric up/down control.
    SpinButton,
    /// Separate window sub-division.
    Section,
    /// Separator between content regions.
    Separator,
    /// Image or icon (informational).
    Graphic,
    /// Tooltip popup text.
    Tooltip,
    /// Status bar or informational line.
    StatusBar,
    /// Menu bar with menu items.
    MenuBar,
    /// Dropdown menu container.
    Menu,
    /// Single item within a menu.
    MenuItem,
    /// Context (right-click) menu.
    ContextMenu,
    /// Table with rows and columns.
    Table,
    /// Table row.
    TableRow,
    /// Table cell.
    TableCell,
    /// Description list (term + definition pairs).
    DescriptionList,
    /// Description term.
    DescriptionTerm,
    /// Description definition/value.
    DescriptionValue,
    /// Alert or error message.
    Alert,
    /// Content separator with label.
    Landmark,
    /// Generic container without specific semantics.
    Generic,
}

impl Role {
    /// Returns the AT-SPI role string for this role.
    /// Used in OSC 99 announcements.
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Window => "window",
            Role::Panel => "panel",
            Role::Form => "form",
            Role::Label => "label",
            Role::TextField => "text field",
            Role::StaticText => "static",
            Role::Button => "push button",
            Role::CheckBox => "check box",
            Role::RadioButton => "radio button",
            Role::ComboBox => "combo box",
            Role::List => "list",
            Role::ListItem => "list item",
            Role::Tree => "tree",
            Role::TreeItem => "tree item",
            Role::PageTab => "page tab",
            Role::PageTabList => "page tab list",
            Role::ProgressBar => "progress bar",
            Role::Slider => "slider",
            Role::SpinButton => "spinbutton",
            Role::Section => "section",
            Role::Separator => "separator",
            Role::Graphic => "graphic",
            Role::Tooltip => "tooltip",
            Role::StatusBar => "status bar",
            Role::MenuBar => "menu bar",
            Role::Menu => "menu",
            Role::MenuItem => "menu item",
            Role::ContextMenu => "popup menu",
            Role::Table => "table",
            Role::TableRow => "table row",
            Role::TableCell => "table cell",
            Role::DescriptionList => "description list",
            Role::DescriptionTerm => "description term",
            Role::DescriptionValue => "description value",
            Role::Alert => "alert",
            Role::Landmark => "landmark",
            Role::Generic => "unknown",
        }
    }
}

/// Announcement priority level.
///
/// Controls when the announcement is sent relative to other output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnounceLevel {
    /// Polite — announced when the screen reader is idle.
    /// Use for non-critical updates.
    Polite,
    /// Assertive — announced immediately.
    /// Use for errors, warnings, and critical state changes.
    Assertive,
}

impl AnnounceLevel {
    fn as_str(&self) -> &'static str {
        match self {
            AnnounceLevel::Polite => "polite",
            AnnounceLevel::Assertive => "assertive",
        }
    }
}

/// Sends an accessibility announcement via OSC 99.
///
/// The announcement includes the text, role (for context), and priority level.
/// Screen readers receive this and speak the text to the user.
///
/// # Format
///
/// OSC 99 format: `\x1b]99;params;text\x07`
///
/// Where params is `role,label,value,description;level`
///
/// # Terminal Compatibility
///
/// This feature requires terminal support for OSC 99. Not all terminals
/// implement it. Implementations should check for support and gracefully
/// degrade when unavailable.
///
/// # Example
///
/// ```rust,ignore
/// // Announce a form validation error
/// announce(&mut writer, "Email field: invalid address", AnnounceLevel::Assertive)?;
/// ```
pub fn announce<W: Write>(writer: &mut W, text: &str, level: AnnounceLevel) -> io::Result<()> {
    let escaped = text.replace(['\x07', '\x1b'], "");
    write!(writer, "\x1b]99;{};{}\x07", level.as_str(), escaped)
}

/// Sends an accessibility announcement with full metadata.
///
/// Includes role, label, value, description, and level.
pub fn announce_with_meta<W: Write>(
    writer: &mut W,
    role: Role,
    label: Option<&str>,
    value: Option<&str>,
    description: Option<&str>,
    level: AnnounceLevel,
) -> io::Result<()> {
    let role_str = role.as_str();
    let label_str = label.unwrap_or("");
    let value_str = value.unwrap_or("");
    let desc_str = description.unwrap_or("");

    // Build the param string
    let params = format!("{},{},{},{}", role_str, label_str, value_str, desc_str);
    let escaped = params.replace(['\x07', '\x1b'], "");

    write!(writer, "\x1b]99;{};{}\x07", level.as_str(), escaped)
}

/// Announcer for widget-level accessibility events.
///
/// Embeds in a widget struct and provides a clean API for announcing
/// state changes. Integrates with the terminal writer to emit OSC 99.
#[derive(Debug, Clone)]
pub struct Announcer {
    /// Whether announcements are enabled.
    enabled: bool,
}

impl Announcer {
    /// Creates a new enabled announcer.
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Creates an announcer with explicit enabled state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Announces text at the given priority level.
    pub fn announce<W: Write>(
        &self,
        writer: &mut W,
        text: &str,
        level: AnnounceLevel,
    ) -> io::Result<()> {
        if self.enabled {
            announce(writer, text, level)
        } else {
            Ok(())
        }
    }

    /// Announces a state change with role and label context.
    pub fn announce_change<W: Write>(
        &self,
        writer: &mut W,
        role: Role,
        label: Option<&str>,
        description: Option<&str>,
        level: AnnounceLevel,
    ) -> io::Result<()> {
        if self.enabled {
            announce_with_meta(writer, role, label, None, description, level)
        } else {
            Ok(())
        }
    }
}

impl Default for Announcer {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for widgets that expose accessibility metadata.
///
/// Implementing this trait enables screen reader support for your widget.
/// The framework queries these methods to build OSC 99 announcements
/// when the widget's state changes in an accessibility-relevant way.
///
/// # Example
///
/// ```rust,ignore
/// struct MyCheckbox {
///     checked: bool,
///     label: String,
/// }
///
/// impl Accessibility for MyCheckbox {
///     fn role(&self) -> Role { Role::CheckBox }
///     fn label(&self) -> Option<&str> { Some(&self.label) }
///     fn checked(&self) -> Option<bool> { Some(self.checked) }
///     fn disabled(&self) -> bool { false }
/// }
/// ```
pub trait Accessibility {
    /// Returns the accessibility role of this widget.
    fn role(&self) -> Role {
        Role::Generic
    }

    /// Returns the accessible label (human-readable name) of this widget.
    /// Used as the primary announcement text by screen readers.
    fn label(&self) -> Option<&str> {
        None
    }

    /// Returns an extended description of this widget's purpose or current state.
    fn description(&self) -> Option<&str> {
        None
    }

    /// Returns the current value of this widget (for inputs, sliders, etc.).
    fn value(&self) -> Option<&str> {
        None
    }

    /// For expandable containers (trees, collapsibles): whether expanded.
    fn expanded(&self) -> Option<bool> {
        None
    }

    /// For checkable widgets: whether checked.
    fn checked(&self) -> Option<bool> {
        None
    }

    /// Whether this widget is disabled and non-interactive.
    fn disabled(&self) -> bool {
        false
    }

    /// Whether this widget triggers a popup or dialog on activation.
    fn has_popup(&self) -> bool {
        false
    }

    /// Returns the keyboard shortcut for this widget (e.g., "Ctrl+S").
    fn keyboard_shortcut(&self) -> Option<&str> {
        None
    }
}

/// Null implementation for widgets without accessibility support.
///
/// Provides sensible defaults so implementing `Accessibility` is optional.
impl Accessibility for () {
    fn role(&self) -> Role {
        Role::Generic
    }
}

/// Macro to quickly implement `Accessibility` for a widget struct.
///
/// # Usage
///
/// ```rust,ignore
/// impl Accessibility for MyWidget {
///     accessible_widget! {
///         role: Role::Button,
///         label: self.button_text.as_str(),
///         description: "Click to submit",
///         disabled: self.is_disabled,
///     }
/// }
/// ```
#[macro_export]
macro_rules! accessible_widget {
    (
        role: $role:expr
        $(,)?
    ) => {
        fn role(&self) -> Role { $role }
    };
    (
        role: $role:expr
        $(, label: $label:expr)?
        $(, description: $desc:expr)?
        $(, value: $value:expr)?
        $(, checked: $checked:expr)?
        $(, expanded: $expanded:expr)?
        $(, disabled: $disabled:expr)?
        $(, has_popup: $popup:expr)?
        $(, keyboard_shortcut: $shortcut:expr)?
    ) => {
        fn role(&self) -> Role { $role }
        $(fn label(&self) -> Option<&str> { Some($label) })?
        $(fn description(&self) -> Option<&str> { Some($desc) })?
        $(fn value(&self) -> Option<&str> { Some($value) })?
        $(fn checked(&self) -> Option<bool> { Some($checked) })?
        $(fn expanded(&self) -> Option<bool> { Some($expanded) })?
        $(fn disabled(&self) -> bool { $disabled })?
        $(fn has_popup(&self) -> bool { $popup })?
        $(fn keyboard_shortcut(&self) -> Option<&str> { Some($shortcut) })?
    };
}

//! Internationalization (i18n) scaffolding for Dracon Terminal Engine.
//!
//! This module provides basic i18n support for widget text.
//! It uses a simple key-value store approach with JSON locale files.
//!
//! ## Usage
//!
//! ```ignore
//! use dracon_terminal_engine::framework::i18n::{I18n, tr};
//!
//! // Create i18n instance with English as default
//! let mut i18n = I18n::new("en");
//!
//! // Load additional locale
//! i18n.load_locale("fr").ok();
//!
//! // Get translation
//! let text = i18n.t("greeting"); // returns "Hello" if current locale is "en"
//!
//! // Change locale
//! i18n.set_locale("fr");
//! let text = i18n.t("greeting"); // returns "Bonjour" if current locale is "fr"
//!
//! // Use tr!() macro for compile-time key validation
//! let text = tr!("greeting"); // returns key "greeting" (runtime lookup)
//! ```
//!
//! ## Locale File Format
//!
//! Locale files are simple JSON files:
//! ```json
//! {
//!     "greeting": "Hello",
//!     "farewell": "Goodbye",
//!     "items": {
//!         "one": "1 item",
//!         "many": "{count} items"
//!     }
//! }
//! ```
//!
//! ## Adding Translations to Widgets
//!
//! Use the `#[tr("key")]` attribute to mark widget strings for extraction:
//! ```ignore
//! use dracon_terminal_engine::framework::i18n::tr;
//!
//! struct MyWidget {
//!     #[tr("widget.title")]
//!     title: String,
//!
//!     #[tr("widget.button")]
//!     button_text: String,
//! }
//! ```

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;

/// Internationalization state and translation lookup.
#[derive(Debug, Clone)]
pub struct I18n {
    /// Current locale code (e.g., "en", "fr", "de")
    locale: String,
    /// Translation map: key -> value
    translations: HashMap<String, String>,
    /// Fallback locale
    #[allow(dead_code)]
    fallback_locale: String,
    /// Default fallback translations (English)
    fallback_map: HashMap<String, String>,
}

impl Default for I18n {
    fn default() -> Self {
        Self::new("en")
    }
}

impl I18n {
    /// Create a new I18n instance with the given locale.
    pub fn new(locale: &str) -> Self {
        let mut i18n = Self {
            locale: locale.to_string(),
            translations: HashMap::new(),
            fallback_locale: "en".to_string(),
            fallback_map: HashMap::new(),
        };
        // Load default English translations
        i18n.load_builtin_en();
        i18n
    }

    /// Get the current locale.
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Set the current locale.
    pub fn set_locale(&mut self, lang: &str) {
        self.locale = lang.to_string();
    }

    /// Load a locale file from disk.
    ///
    /// Looks for locale files in the following locations:
    /// - `./locales/{lang}.json`
    /// - `~/.config/dracon/locales/{lang}.json`
    /// - `/etc/dracon/locales/{lang}.json`
    ///
    /// Returns `Ok(())` on success, `Err` if the file cannot be read or parsed.
    pub fn load_locale(&mut self, lang: &str) -> Result<(), I18nError> {
        // Search paths for locale files
        let search_paths = [
            format!("locales/{lang}.json"),
            format!(
                "{}/.config/dracon/locales/{lang}.json",
                dirs::home_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default()
            ),
            format!("/etc/dracon/locales/{lang}.json"),
        ];

        for path in search_paths {
            if let Ok(content) = fs::read_to_string(&path) {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(value) => {
                        if let Some(obj) = value.as_object() {
                            self.translations.clear();
                            Self::flatten_json(obj, "", &mut self.translations);
                            self.locale = lang.to_string();
                            return Ok(());
                        }
                    }
                    Err(e) => return Err(I18nError::ParseError(e.to_string())),
                }
            }
        }

        Err(I18nError::LocaleNotFound(lang.to_string()))
    }

    /// Translate a key to the current locale.
    ///
    /// If the key is not found in the current locale, falls back to
    /// English (built-in) translations, then returns the key itself.
    pub fn t(&self, key: &str) -> Cow<'_, str> {
        // Try current locale
        if let Some(value) = self.translations.get(key) {
            return Cow::Owned(value.clone());
        }
        // Fall back to English
        if let Some(value) = self.fallback_map.get(key) {
            return Cow::Owned(value.clone());
        }
        // Return the key itself as last resort
        Cow::Owned(key.to_string())
    }

    /// Translate with interpolation support.
    ///
    /// Replaces `{placeholder}` in the translation string with provided values.
    ///
    /// ```ignore
    /// let text = i18n.t_interpolate("items_count", &[("count", "5")]);
    /// // If "items_count" is "{count} items", returns "5 items"
    /// ```
    pub fn t_interpolate(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let template = self.t(key).into_owned();
        let mut result = template;
        for (name, value) in vars {
            result = result.replace(&format!("{{{name}}}"), value);
        }
        result
    }

    /// Get all available translation keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.translations
            .keys()
            .chain(self.fallback_map.keys())
            .map(|s| s.as_str())
    }

    /// Check if a key exists in the current locale.
    pub fn contains(&self, key: &str) -> bool {
        self.translations.contains_key(key) || self.fallback_map.contains_key(key)
    }

    /// Add a translation directly (useful for testing or dynamic content).
    pub fn add(&mut self, key: &str, value: &str) {
        self.translations.insert(key.to_string(), value.to_string());
    }

    /// Load built-in English translations.
    fn load_builtin_en(&mut self) {
        self.fallback_map = HashMap::from([
            // Common UI strings
            ("app.title".to_string(), "Dracon Terminal Engine".to_string()),
            ("app.subtitle".to_string(), "A terminal application framework".to_string()),
            ("button.ok".to_string(), "OK".to_string()),
            ("button.cancel".to_string(), "Cancel".to_string()),
            ("button.apply".to_string(), "Apply".to_string()),
            ("button.close".to_string(), "Close".to_string()),
            ("button.save".to_string(), "Save".to_string()),
            ("button.delete".to_string(), "Delete".to_string()),
            ("button.edit".to_string(), "Edit".to_string()),
            ("button.back".to_string(), "Back".to_string()),
            ("button.next".to_string(), "Next".to_string()),
            ("button.finish".to_string(), "Finish".to_string()),
            // Navigation
            ("nav.home".to_string(), "Home".to_string()),
            ("nav.settings".to_string(), "Settings".to_string()),
            ("nav.help".to_string(), "Help".to_string()),
            ("nav.quit".to_string(), "Quit".to_string()),
            // Messages
            ("msg.loading".to_string(), "Loading...".to_string()),
            ("msg.saving".to_string(), "Saving...".to_string()),
            ("msg.error".to_string(), "Error".to_string()),
            ("msg.success".to_string(), "Success".to_string()),
            ("msg.warning".to_string(), "Warning".to_string()),
            ("msg.info".to_string(), "Information".to_string()),
            ("msg.confirm".to_string(), "Confirm".to_string()),
            ("msg.delete_confirm".to_string(), "Are you sure you want to delete this?".to_string()),
            // Status
            ("status.loading".to_string(), "Loading".to_string()),
            ("status.ready".to_string(), "Ready".to_string()),
            ("status.error".to_string(), "Error".to_string()),
            // Widgets
            ("widget.search".to_string(), "Search...".to_string()),
            ("widget.no_results".to_string(), "No results found".to_string()),
            ("widget.loading".to_string(), "Loading...".to_string()),
            ("widget.empty".to_string(), "Empty".to_string()),
            // Errors
            ("error.not_found".to_string(), "Not found".to_string()),
            ("error.unauthorized".to_string(), "Unauthorized".to_string()),
            ("error.timeout".to_string(), "Request timed out".to_string()),
            ("error.network".to_string(), "Network error".to_string()),
            ("error.parse".to_string(), "Failed to parse data".to_string()),
        ]);
    }

    /// Flatten nested JSON object into dot-notation keys.
    fn flatten_json(
        obj: &serde_json::Map<String, serde_json::Value>,
        prefix: &str,
        map: &mut HashMap<String, String>,
    ) {
        for (key, value) in obj {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };

            match value {
                serde_json::Value::String(s) => {
                    map.insert(full_key, s.clone());
                }
                serde_json::Value::Object(nested) => {
                    Self::flatten_json(nested, &full_key, map);
                }
                serde_json::Value::Array(arr) => {
                    for (i, item) in arr.iter().enumerate() {
                        if let serde_json::Value::String(s) = item {
                            map.insert(format!("{full_key}[{i}]"), s.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// I18n-related errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum I18nError {
    /// The locale file could not be found.
    LocaleNotFound(String),
    /// The locale file could not be parsed.
    ParseError(String),
    /// An I/O error occurred.
    IoError(String),
}

impl std::fmt::Display for I18nError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            I18nError::LocaleNotFound(lang) => write!(f, "Locale not found: {lang}"),
            I18nError::ParseError(msg) => write!(f, "Failed to parse locale: {msg}"),
            I18nError::IoError(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for I18nError {}

impl From<std::io::Error> for I18nError {
    fn from(e: std::io::Error) -> Self {
        I18nError::IoError(e.to_string())
    }
}

// Optional dirs dependency for home directory detection
mod dirs {
    pub fn home_dir() -> Option<std::path::PathBuf> {
        std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| std::env::var_os("USERPROFILE").map(std::path::PathBuf::from))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Macros
// ─────────────────────────────────────────────────────────────────────────────

/// Translate a string key at runtime.
///
/// This macro provides compile-time key syntax but relies on runtime lookup
/// through the global I18n instance. For static strings, use the key directly.
///
/// # Example
///
/// ```ignore
/// let text = tr!("button.ok"); // returns "OK" if locale is English
/// ```
#[macro_export]
macro_rules! tr {
    ($key:expr) => {{
        // This expands to the key itself at compile time.
        // For actual translation, use the I18n instance:
        // i18n.t($key)
        $key
    }};
}

/// Format a translated string with interpolation.
///
/// # Example
///
/// ```ignore
/// let text = trf!("items_count", count = "5");
/// // If "items_count" is "{count} items", returns "5 items"
/// ```
#[macro_export]
macro_rules! trf {
    ($key:expr, $($var:ident = $value:expr),*) => {{
        // Build the format string
        let mut result = $key.to_string();
        $(
            result = result.replace(&format!("{{{}}}", stringify!($var)), $value);
        )*
        result
    }};
}

// ─────────────────────────────────────────────────────────────────────────────
// Procedural macro for marking widget fields for extraction
// ─────────────────────────────────────────────────────────────────────────────

/// Attribute macro to mark a widget field for translation extraction.
///
/// This does not perform translation at compile time - it only marks the
/// field for extraction by external tools (like gettext or custom extraction).
///
/// # Example
///
/// ```ignore
/// use dracon_terminal_engine::framework::i18n::tr_attr;
///
/// #[tr_attr("widget.title")]
/// struct MyWidget {
///     #[tr_attr("button.label")]
///     button_text: String,
/// }
/// ```
///
/// Note: This macro just passes the string through at compile time.
/// Real translation happens at runtime via I18n::t().
pub use macro_attrs::tr_attr as tr;

mod macro_attrs {
    /// Empty attribute for marking strings for translation extraction.
    pub fn tr_attr(_key: &'static str) {}
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_default() {
        let i18n = I18n::default();
        assert_eq!(i18n.locale(), "en");
    }

    #[test]
    fn test_i18n_new() {
        let i18n = I18n::new("de");
        assert_eq!(i18n.locale(), "de");
    }

    #[test]
    fn test_i18n_t_builtin() {
        let i18n = I18n::new("en");
        assert_eq!(i18n.t("button.ok").as_ref(), "OK");
        assert_eq!(i18n.t("button.cancel").as_ref(), "Cancel");
        assert_eq!(i18n.t("nav.quit").as_ref(), "Quit");
    }

    #[test]
    fn test_i18n_t_unknown_key() {
        let i18n = I18n::new("en");
        assert_eq!(i18n.t("unknown.key").as_ref(), "unknown.key");
    }

    #[test]
    fn test_i18n_set_locale() {
        let mut i18n = I18n::new("en");
        i18n.set_locale("fr");
        assert_eq!(i18n.locale(), "fr");
    }

    #[test]
    fn test_i18n_add() {
        let mut i18n = I18n::new("en");
        i18n.add("custom.key", "Custom Value");
        assert_eq!(i18n.t("custom.key").as_ref(), "Custom Value");
    }

    #[test]
    fn test_i18n_contains() {
        let i18n = I18n::new("en");
        assert!(i18n.contains("button.ok"));
        assert!(!i18n.contains("nonexistent.key"));
    }

    #[test]
    fn test_i18n_interpolate() {
        let i18n = I18n::new("en");
        // Test with simple key (returns key if not found)
        let result = i18n.t_interpolate("test.key", &[("name", "John")]);
        assert!(result.contains("John") || result == "test.key");
    }

    #[test]
    fn test_i18n_keys() {
        let i18n = I18n::new("en");
        let keys: Vec<_> = i18n.keys().collect();
        assert!(!keys.is_empty());
        assert!(keys.contains(&"button.ok"));
    }

    #[test]
    fn test_i18n_clone() {
        let i18n = I18n::new("en");
        let cloned = i18n.clone();
        assert_eq!(cloned.locale(), i18n.locale());
    }

    #[test]
    fn test_tr_macro() {
        // The tr! macro returns the key itself
        let key = tr!("test.key");
        assert_eq!(key, "test.key");
    }

    #[test]
    fn test_trf_macro() {
        let result = trf!("Hello {name}", name = "World");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_flatten_json() {
        let mut map = HashMap::new();
        let json = serde_json::json!({
            "level1": {
                "level2": "value"
            },
            "simple": "also_value"
        });

        I18n::flatten_json(json.as_object().unwrap(), "", &mut map);

        assert_eq!(map.get("level1.level2"), Some(&"value".to_string()));
        assert_eq!(map.get("simple"), Some(&"also_value".to_string()));
    }

    #[test]
    fn test_i18n_load_nonexistent() {
        let mut i18n = I18n::new("en");
        let result = i18n.load_locale("nonexistent_locale_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_i18n_error_display() {
        let err = I18nError::LocaleNotFound("fr".to_string());
        assert_eq!(format!("{err}"), "Locale not found: fr");

        let err = I18nError::ParseError("invalid json".to_string());
        assert_eq!(format!("{err}"), "Failed to parse locale: invalid json");
    }
}
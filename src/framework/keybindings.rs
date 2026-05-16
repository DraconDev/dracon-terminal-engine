//! Keybinding configuration with TOML support.
//!
//! Provides a centralized, configurable keybinding system that all examples
//! and apps can use. Supports tiered resolution:
//!
//! 1. Project-local `./dracon.toml`
//! 2. User-global `~/.config/dracon/dracon.toml`
//! 3. Engine defaults (compiled-in)
//!
//! ## Keybinding String Format
//!
//! - Simple keys: `"q"`, `"?"`, `"esc"`, `"enter"`, `"tab"`, `"backspace"`, `"up"`, `"down"`, `"left"`, `"right"`
//! - With modifiers: `"ctrl+q"`, `"ctrl+t"`, `"alt+f4"`, `"shift+tab"`
//! - Multiple modifiers: `"ctrl+shift+t"`
//!
//! ## Example TOML
//!
//! ```toml
//! [keybindings]
//! quit = "q"
//! help = "?"
//! theme = "f2"
//! back = "esc"
//! submit = "enter"
//! new_tab = "ctrl+t"
//! close_tab = "ctrl+w"
//! save = "ctrl+s"
//! ```

use crate::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

static RESOLVED_CONFIG: RwLock<Option<KeybindingConfig>> = RwLock::new(None);

/// Resolves the effective keybinding config using tiered resolution.
///
/// Resolution order (later overrides earlier):
/// 1. Engine defaults
/// 2. User global `~/.config/dracon/dracon.toml`
/// 3. Project-local `./dracon.toml`
///
/// The result is cached after the first call. Call [`invalidate_keybinding_cache`]
/// to force re-resolution on the next call (e.g., after config file changes).
pub fn resolve_keybindings() -> KeybindingConfig {
    {
        let guard = match RESOLVED_CONFIG.read() {
            Ok(g) => g,
            Err(_) => return do_resolve_keybindings(),
        };
        if let Some(ref config) = *guard {
            return config.clone();
        }
    }
    let config = do_resolve_keybindings();
    if let Ok(mut guard) = RESOLVED_CONFIG.write() {
        *guard = Some(config.clone());
    }
    config
}

/// Invalidates the cached keybinding config, so the next call to
/// [`resolve_keybindings`] will re-read config files from disk.
pub fn invalidate_keybinding_cache() {
    if let Ok(mut guard) = RESOLVED_CONFIG.write() {
        *guard = None;
    }
}

/// Standard action names used across all examples.
///
/// These are the canonical names for actions. Apps can query
/// `KeybindingSet::matches(action, key_event)` to check if a key
/// event triggers a given action.
pub mod actions {
    pub const QUIT: &str = "quit";
    pub const HELP: &str = "help";
    pub const THEME: &str = "theme";
    pub const BACK: &str = "back";
    pub const SUBMIT: &str = "submit";
    pub const TAB_NEXT: &str = "tab_next";
    pub const TAB_PREV: &str = "tab_prev";
    pub const NEW_TAB: &str = "new_tab";
    pub const CLOSE_TAB: &str = "close_tab";
    pub const SAVE: &str = "save";
    pub const DELETE: &str = "delete";
    pub const NEW_ITEM: &str = "new_item";
    pub const EDIT: &str = "edit";
    pub const REFRESH: &str = "refresh";
    pub const SEARCH: &str = "search";
    pub const CANCEL: &str = "cancel";
    pub const DISMISS: &str = "dismiss";
    pub const TREE_MODE: &str = "tree_mode";
    pub const PAUSE: &str = "pause";
    // Navigation
    pub const UP: &str = "up";
    pub const DOWN: &str = "down";
    pub const LEFT: &str = "left";
    pub const RIGHT: &str = "right";
    pub const PAGE_UP: &str = "page_up";
    pub const PAGE_DOWN: &str = "page_down";
    pub const HOME: &str = "home";
    pub const END: &str = "end";
}

/// A mapping from action names to keybinding strings.
///
/// Example TOML:
/// ```toml
/// [keybindings]
/// quit = "q"
/// help = "?"
/// theme = "t"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeybindingConfig {
    /// Action name -> keybinding string (e.g. "quit" -> "q")
    #[serde(flatten)]
    pub bindings: HashMap<String, String>,
}

impl KeybindingConfig {
    /// Create with conservative defaults — no customization needed.
    /// Uses modifier keys to avoid conflicts with text input.
    pub fn defaults() -> Self {
        let mut bindings = HashMap::new();
        // Core (always safe)
        bindings.insert(actions::QUIT.to_string(), "ctrl+q".to_string());
        bindings.insert(actions::BACK.to_string(), "esc".to_string());
        bindings.insert(actions::SUBMIT.to_string(), "enter".to_string());
        bindings.insert(actions::TAB_NEXT.to_string(), "tab".to_string());
        bindings.insert(actions::TAB_PREV.to_string(), "shift+tab".to_string());
        // Help (F1 avoids ? conflict)
        bindings.insert(actions::HELP.to_string(), "f1".to_string());
        // Tabs (browser/IDE standard)
        bindings.insert(actions::NEW_TAB.to_string(), "ctrl+t".to_string());
        bindings.insert(actions::CLOSE_TAB.to_string(), "ctrl+w".to_string());
        // File operations (universal standard)
        bindings.insert(actions::SAVE.to_string(), "ctrl+s".to_string());
        bindings.insert(actions::NEW_ITEM.to_string(), "ctrl+n".to_string());
        bindings.insert(actions::DELETE.to_string(), "ctrl+d".to_string());
        bindings.insert(actions::EDIT.to_string(), "ctrl+e".to_string());
        // Search
        bindings.insert(actions::SEARCH.to_string(), "ctrl+f".to_string());
        bindings.insert(actions::THEME.to_string(), "f2".to_string());
        bindings.insert(actions::REFRESH.to_string(), "f5".to_string());
        bindings.insert(actions::PAUSE.to_string(), "ctrl+p".to_string());
        bindings.insert(actions::DISMISS.to_string(), "escape".to_string());
        Self { bindings }
    }

    /// Merge another config on top of this one (overrides).
    pub fn merge(&mut self, other: KeybindingConfig) {
        for (k, v) in other.bindings {
            self.bindings.insert(k, v);
        }
    }

    /// Get the keybinding string for an action.
    pub fn get(&self, action: &str) -> Option<&str> {
        self.bindings.get(action).map(|s| s.as_str())
    }

    /// Parse a keybinding string into a KeyEvent.
    ///
    /// Returns `None` if the string is malformed.
    ///
    /// # Examples
    ///
    /// - `"q"` -> KeyEvent { code: Char('q'), modifiers: empty }
    /// - `"ctrl+t"` -> KeyEvent { code: Char('t'), modifiers: CTRL }
    /// - `"esc"` -> KeyEvent { code: Esc, modifiers: empty }
    /// - `"shift+tab"` -> KeyEvent { code: Tab, modifiers: SHIFT }
    pub fn parse_keybinding(binding: &str) -> Option<KeyEvent> {
        let parts: Vec<&str> = binding.split('+').collect();
        if parts.is_empty() {
            return None;
        }

        let mut modifiers = KeyModifiers::empty();
        let key_part = parts[parts.len() - 1];

        for part in &parts[..parts.len() - 1] {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                "alt" => modifiers |= KeyModifiers::ALT,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                "super" | "meta" | "cmd" | "win" => modifiers |= KeyModifiers::SUPER,
                _ => return None, // unknown modifier
            }
        }

        // If the last part looks like a modifier too, it's invalid
        // (e.g. "ctrl+shift+" without a key)
        let code = match key_part.to_lowercase().as_str() {
            "esc" | "escape" => KeyCode::Esc,
            "enter" | "return" => KeyCode::Enter,
            "tab" => KeyCode::Tab,
            "backspace" => KeyCode::Backspace,
            "space" => KeyCode::Char(' '),
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            "delete" | "del" => KeyCode::Delete,
            "insert" | "ins" => KeyCode::Insert,
            "f1" => KeyCode::F(1),
            "f2" => KeyCode::F(2),
            "f3" => KeyCode::F(3),
            "f4" => KeyCode::F(4),
            "f5" => KeyCode::F(5),
            "f6" => KeyCode::F(6),
            "f7" => KeyCode::F(7),
            "f8" => KeyCode::F(8),
            "f9" => KeyCode::F(9),
            "f10" => KeyCode::F(10),
            "f11" => KeyCode::F(11),
            "f12" => KeyCode::F(12),
            "f13" => KeyCode::F(13),
            "f14" => KeyCode::F(14),
            "f15" => KeyCode::F(15),
            "f16" => KeyCode::F(16),
            "f17" => KeyCode::F(17),
            "f18" => KeyCode::F(18),
            "f19" => KeyCode::F(19),
            "f20" => KeyCode::F(20),
            // Keypad digits (no dedicated Keypad variant; map to Char)
            "kp0" => KeyCode::Char('0'),
            "kp1" => KeyCode::Char('1'),
            "kp2" => KeyCode::Char('2'),
            "kp3" => KeyCode::Char('3'),
            "kp4" => KeyCode::Char('4'),
            "kp5" => KeyCode::Char('5'),
            "kp6" => KeyCode::Char('6'),
            "kp7" => KeyCode::Char('7'),
            "kp8" => KeyCode::Char('8'),
            "kp9" => KeyCode::Char('9'),
            // Media keys — placeholder Char mappings until dedicated KeyCode variants are added
            "media_play" => KeyCode::Char('\u{10A5F}'),  // TODO: replace with KeyCode::MediaPlay
            "media_pause" => KeyCode::Char('\u{23F8}'),  // TODO: replace with KeyCode::MediaPause
            "media_stop" => KeyCode::Char('\u{23F9}'),   // TODO: replace with KeyCode::MediaStop
            "media_next" => KeyCode::Char('\u{23ED}'),   // TODO: replace with KeyCode::MediaNext
            "media_prev" => KeyCode::Char('\u{23EE}'),   // TODO: replace with KeyCode::MediaPrev
            s if s.len() == 1 => {
                let ch = s.chars().next()?;
                KeyCode::Char(ch)
            }
            _ => return None,
        };

        Some(KeyEvent {
            kind: KeyEventKind::Press,
            code,
            modifiers,
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// KEYBINDING SET (RESOLVED + VALIDATED)
// ═══════════════════════════════════════════════════════════════

/// A resolved set of keybindings with pre-parsed KeyEvents.
///
/// Created from a `KeybindingConfig` for fast lookup at runtime.
/// Also validates that there are no conflicting bindings.
#[derive(Debug, Clone)]
pub struct KeybindingSet {
    /// action name -> (parsed KeyEvent, original display string)
    bindings: HashMap<String, (KeyEvent, String)>,
}

impl KeybindingSet {
    /// Create from a config, validating for conflicts.
    ///
    /// Logs warnings for conflicting bindings but does not fail.
    pub fn from_config(config: &KeybindingConfig) -> Self {
        let mut bindings = HashMap::new();
        let mut seen = HashMap::new(); // KeyEvent -> action name

        for (action, binding_str) in &config.bindings {
            if let Some(event) = KeybindingConfig::parse_keybinding(binding_str) {
                // Check for conflicts
                if let Some(existing_action) = seen.get(&event) {
                    eprintln!(
                        "[dracon] keybinding conflict: '{}' and '{}' both bound to '{}'",
                        existing_action, action, binding_str
                    );
                }
                seen.insert(event, action.clone());
                bindings.insert(action.clone(), (event, binding_str.clone()));
            } else {
                eprintln!(
                    "[dracon] invalid keybinding for '{}': '{}'",
                    action, binding_str
                );
            }
        }

        Self { bindings }
    }

    /// Reloads keybindings from disk (re-reads dracon.toml).
    ///
    /// Call this when you want to pick up config changes at runtime
    /// without restarting the application.
    pub fn reload(&mut self) {
        invalidate_keybinding_cache();
        let config = resolve_keybindings();
        *self = Self::from_config(&config);
    }

    /// Check if a key event matches an action.
    ///
    /// # Example
    /// ```ignore
    /// if keybindings.matches("quit", key) {
    ///     // quit the app
    /// }
    /// ```
    pub fn matches(&self, action: &str, event: &KeyEvent) -> bool {
        if let Some((expected, _)) = self.bindings.get(action) {
            if expected.kind == KeyEventKind::Press
                && event.kind != KeyEventKind::Press
                && event.kind != KeyEventKind::Repeat
            {
                return false;
            }
            expected.code == event.code && expected.modifiers == event.modifiers
        } else {
            false
        }
    }

    /// Get the display string for an action (e.g. "quit" -> "ctrl+q").
    pub fn display(&self, action: &str) -> Option<&str> {
        self.bindings.get(action).map(|(_, s)| s.as_str())
    }

    /// Get all bound actions.
    pub fn actions(&self) -> impl Iterator<Item = &String> {
        self.bindings.keys()
    }

    /// Format a status bar hint from action/label pairs.
    ///
    /// # Example
    /// ```ignore
    /// let hint = keybindings.format_hint(&[
    ///     ("quit", "quit"),
    ///     ("help", "help"),
    ///     ("theme", "theme"),
    /// ]);
    /// // -> "ctrl+q: quit | ?: help | t: theme"
    /// ```
    pub fn format_hint(&self, pairs: &[(&str, &str)]) -> String {
        pairs
            .iter()
            .filter_map(|(action, label)| {
                self.display(action)
                    .map(|key| format!("{}: {}", key, label))
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

impl Default for KeybindingSet {
    fn default() -> Self {
        Self::from_config(&KeybindingConfig::defaults())
    }
}

// ═══════════════════════════════════════════════════════════════
// TIERED CONFIG RESOLUTION
// ═══════════════════════════════════════════════════════════════
// (Implementation in RwLock block above)

fn do_resolve_keybindings() -> KeybindingConfig {
    let mut config = KeybindingConfig::defaults();

    // Tier 2: User global
    if let Some(xdg_config) = xdg_config_path() {
        if xdg_config.exists() {
            match load_keybindings_from_toml(&xdg_config) {
                Ok(user_config) => config.merge(user_config),
                Err(e) => eprintln!("[dracon] warning: failed to load XDG config: {}", e),
            }
        }
    }

    // Tier 3: Project-local
    let local_path = std::path::Path::new("dracon.toml");
    if local_path.exists() {
        match load_keybindings_from_toml(local_path) {
            Ok(local_config) => config.merge(local_config),
            Err(e) => eprintln!("[dracon] warning: failed to load local dracon.toml: {}", e),
        }
    }

    config
}

/// Load just the `[keybindings]` section from a TOML file.
fn load_keybindings_from_toml(path: &std::path::Path) -> Result<KeybindingConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("read error: {}", e))?;

    // Parse the full TOML to extract just the keybindings section
    let doc: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("parse error: {}", e))?;

    if let Some(kb_table) = doc.get("keybindings") {
        kb_table
            .clone()
            .try_into::<KeybindingConfig>()
            .map_err(|e| format!("keybindings parse error: {}", e))
    } else {
        Ok(KeybindingConfig::default())
    }
}

/// Get the XDG config directory path: `~/.config/dracon/dracon.toml`
fn xdg_config_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok()?;
    Some(std::path::Path::new(&home).join(".config").join("dracon").join("dracon.toml"))
}

// ═══════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_keys() {
        assert!(KeybindingConfig::parse_keybinding("q").is_some());
        assert!(KeybindingConfig::parse_keybinding("?").is_some());
        assert!(KeybindingConfig::parse_keybinding("esc").is_some());
        assert!(KeybindingConfig::parse_keybinding("enter").is_some());
        assert!(KeybindingConfig::parse_keybinding("tab").is_some());
    }

    #[test]
    fn test_parse_with_modifiers() {
        let evt = KeybindingConfig::parse_keybinding("ctrl+q").unwrap();
        assert!(matches!(evt.code, KeyCode::Char('q')));
        assert!(evt.modifiers.contains(KeyModifiers::CONTROL));

        let evt = KeybindingConfig::parse_keybinding("ctrl+t").unwrap();
        assert!(matches!(evt.code, KeyCode::Char('t')));
        assert!(evt.modifiers.contains(KeyModifiers::CONTROL));

        let evt = KeybindingConfig::parse_keybinding("shift+tab").unwrap();
        assert!(matches!(evt.code, KeyCode::Tab));
        assert!(evt.modifiers.contains(KeyModifiers::SHIFT));

        let evt = KeybindingConfig::parse_keybinding("alt+f4").unwrap();
        assert!(matches!(evt.code, KeyCode::F(4)));
        assert!(evt.modifiers.contains(KeyModifiers::ALT));
    }

    #[test]
    fn test_parse_case_insensitive() {
        let evt1 = KeybindingConfig::parse_keybinding("ESC").unwrap();
        let evt2 = KeybindingConfig::parse_keybinding("esc").unwrap();
        assert_eq!(evt1.code, evt2.code);
    }

    #[test]
    fn test_parse_invalid() {
        assert!(KeybindingConfig::parse_keybinding("").is_none());
        assert!(KeybindingConfig::parse_keybinding("ctrl+").is_none());
        assert!(KeybindingConfig::parse_keybinding("unknownkey").is_none());
    }

    #[test]
    fn test_keybinding_set_matches() {
        let config = KeybindingConfig::defaults();
        let set = KeybindingSet::from_config(&config);

        let ctrl_q = KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::CONTROL,
        };
        assert!(set.matches("quit", &ctrl_q));

        let plain_q = KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::empty(),
        };
        assert!(!set.matches("quit", &plain_q));
    }

    #[test]
    fn test_keybinding_set_conflict_detection() {
        let mut config = KeybindingConfig::default();
        config.bindings.insert("action1".to_string(), "q".to_string());
        config.bindings.insert("action2".to_string(), "q".to_string());

        // Should not panic, just log warning
        let _set = KeybindingSet::from_config(&config);
    }

    #[test]
    fn test_config_merge() {
        let mut base = KeybindingConfig::defaults();
        let mut override_config = KeybindingConfig::default();
        override_config.bindings.insert("quit".to_string(), "ctrl+x".to_string());

        assert_eq!(base.get("quit"), Some("ctrl+q"));
        base.merge(override_config);
        assert_eq!(base.get("quit"), Some("ctrl+x"));
    }

    #[test]
    fn test_defaults_use_modifier_keys() {
        let config = KeybindingConfig::defaults();
        let bare_char_actions: Vec<&str> = config
            .bindings
            .iter()
            .filter(|(_, binding)| {
                let binding_lower = binding.to_lowercase();
                !binding_lower.contains('+')
                    && binding_lower.len() == 1
                    && binding_lower.chars().next().unwrap().is_ascii_lowercase()
            })
            .map(|(action, _)| action.as_str())
            .collect::<Vec<_>>();
        assert!(
            bare_char_actions.is_empty(),
            "These actions use bare char keys (conflict with text input): {:?}",
            bare_char_actions
        );
    }

    #[test]
    fn test_pause_binding_uses_modifier() {
        let config = KeybindingConfig::defaults();
        let pause_binding = config.get(super::actions::PAUSE).expect("PAUSE should have a default binding");
        assert!(
            pause_binding.contains('+'),
            "PAUSE binding '{}' should use a modifier key to avoid text input conflicts",
            pause_binding
        );
    }

    #[test]
    fn test_dismiss_not_ctrl_c() {
        let config = KeybindingConfig::defaults();
        let dismiss_binding = config.get(super::actions::DISMISS).expect("DISMISS should have a default binding");
        assert_ne!(
            dismiss_binding, "ctrl+c",
            "DISMISS should not be ctrl+c (conflicts with Copy/SIGINT)"
        );
    }

    #[test]
    fn test_invalidate_cache_allows_reload() {
        let _ = resolve_keybindings();
        invalidate_keybinding_cache();
        let config1 = resolve_keybindings();
        invalidate_keybinding_cache();
        let config2 = resolve_keybindings();
        assert_eq!(config1.bindings.len(), config2.bindings.len());
    }

    #[test]
    fn test_keybinding_set_reload_works() {
        let config = KeybindingConfig::defaults();
        let mut set = KeybindingSet::from_config(&config);
        let quit_display_before = set.display("quit").map(|s| s.to_string());
        set.reload();
        let quit_display_after = set.display("quit").map(|s| s.to_string());
        assert_eq!(quit_display_before, quit_display_after);
    }
}

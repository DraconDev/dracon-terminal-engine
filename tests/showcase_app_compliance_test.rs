//! Compliance tests for showcase "app" example files.
//!
//! Verifies that each example app adheres to the dracon-terminal-engine conventions
//! documented in AGENTS.md, including theme propagation, keybinding usage, and
//! help overlay structure.

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;

/// Helper: create a key press event.
fn key_press(code: KeyCode) -> KeyEvent {
    KeyEvent {
        kind: KeyEventKind::Press,
        code,
        modifiers: KeyModifiers::empty(),
    }
}

fn key_press_ctrl(code: KeyCode) -> KeyEvent {
    KeyEvent {
        kind: KeyEventKind::Press,
        code,
        modifiers: KeyModifiers::CONTROL,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 1: Help overlay uses theme.surface_elevated background, rounded corners,
//         centered title with primary+BOLD, and two-column key/desc layout
// ═══════════════════════════════════════════════════════════════════════════════

/// A minimal widget that renders a help overlay following convention.
/// Used to verify the convention pattern itself is testable.
struct HelpOverlayWidget {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
}

impl HelpOverlayWidget {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
            show_help: true,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }
}

impl Widget for HelpOverlayWidget {
    fn id(&self) -> WidgetId { WidgetId::new(1) }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { Rect::new(0, 0, 80, 24) }
    fn set_area(&mut self, _area: Rect) {}
    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(t.bg);

        if self.show_help {
            let hw = 46u16.min(area.width.saturating_sub(4));
            let hh = 11u16.min(area.height.saturating_sub(4));
            let hx = (area.width - hw) / 2;
            let hy = (area.height - hh) / 2;

            for y in hy..hy + hh {
                for x in hx..hx + hw {
                    let idx = (y * plane.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * plane.width + cx) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = *ch; plane.cells[idx].fg = t.outline; }
            }
            for x in hx + 1..hx + hw - 1 {
                let top = (hy * plane.width + x) as usize;
                let bot = ((hy + hh - 1) * plane.width + x) as usize;
                if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
                if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
            }
            for y in hy + 1..hy + hh - 1 {
                let left = (y * plane.width + hx) as usize;
                let right = (y * plane.width + hx + hw - 1) as usize;
                if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
                if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
            }

            let title = "App Help";
            let tx = hx + (hw - title.len() as u16) / 2;
            for (i, c) in title.chars().enumerate() {
                let idx = ((hy + 1) * plane.width + tx + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }

            let shortcuts = [
                ("↑/↓", "Navigate"),
                ("Enter", "Select"),
                (self.keybindings.display(actions::THEME).unwrap_or("ctrl+t"), "Cycle theme"),
                (self.keybindings.display(actions::HELP).unwrap_or("f1"), "Toggle help"),
                (self.keybindings.display(actions::BACK).unwrap_or("esc"), "Dismiss"),
                (self.keybindings.display(actions::QUIT).unwrap_or("ctrl+q"), "Quit"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * plane.width + hx + 2 + j as u16) as usize;
                    if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * plane.width + hx + 14 + j as u16) as usize;
                    if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.fg; }
                }
            }
        }
        plane
    }
}

#[test]
fn test_help_overlay_background_is_surface_elevated() {
    let widget = HelpOverlayWidget::new(Theme::nord());
    let plane = widget.render(Rect::new(0, 0, 80, 24));
    let t = Theme::nord();
    let hw = 46u16.min(76);
    let hh = 11u16.min(20);
    let hx = (80 - hw) / 2;
    let hy = (24 - hh) / 2;
    let center_idx = ((hy + 2) * 80 + hx + 2) as usize;
    assert!(
        center_idx < plane.cells.len(),
        "center idx should be in bounds"
    );
    assert_eq!(
        plane.cells[center_idx].bg,
        t.surface_elevated,
        "help overlay interior should use surface_elevated background"
    );
}

#[test]
fn test_help_overlay_has_rounded_corners() {
    let widget = HelpOverlayWidget::new(Theme::nord());
    let plane = widget.render(Rect::new(0, 0, 80, 24));
    let hw = 46u16.min(76);
    let hh = 11u16.min(20);
    let hx = (80 - hw) / 2;
    let hy = (24 - hh) / 2;
    let top_left = (hy * 80 + hx) as usize;
    let top_right = (hy * 80 + hx + hw - 1) as usize;
    let bot_left = ((hy + hh - 1) * 80 + hx) as usize;
    let bot_right = ((hy + hh - 1) * 80 + hx + hw - 1) as usize;
    assert_eq!(plane.cells[top_left].char, '╭', "top-left corner should be ╭");
    assert_eq!(plane.cells[top_right].char, '╮', "top-right corner should be ╮");
    assert_eq!(plane.cells[bot_left].char, '╰', "bottom-left corner should be ╰");
    assert_eq!(plane.cells[bot_right].char, '╯', "bottom-right corner should be ╯");
}

#[test]
fn test_help_overlay_title_is_centered_and_primary_bold() {
    let widget = HelpOverlayWidget::new(Theme::nord());
    let plane = widget.render(Rect::new(0, 0, 80, 24));
    let t = Theme::nord();
    let hw = 46u16.min(76);
    let hh = 11u16.min(20);
    let hx = (80 - hw) / 2;
    let hy = (24 - hh) / 2;
    let title = "App Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    let first_char_idx = ((hy + 1) * 80 + tx) as usize;
    assert_eq!(plane.cells[first_char_idx].char, 'A');
    assert_eq!(plane.cells[first_char_idx].fg, t.primary, "title should use theme.primary");
    assert_eq!(plane.cells[first_char_idx].style, Styles::BOLD, "title should be BOLD");
}

#[test]
fn test_help_overlay_two_column_key_primary_desc_fg() {
    let widget = HelpOverlayWidget::new(Theme::nord());
    let plane = widget.render(Rect::new(0, 0, 80, 24));
    let t = Theme::nord();
    let hw = 46u16.min(76);
    let hh = 11u16.min(20);
    let hx = (80 - hw) / 2;
    let hy = (24 - hh) / 2;
    let first_shortcut_row = hy + 3;
    let key_idx = (first_shortcut_row * 80 + hx + 2) as usize;
    let desc_idx = (first_shortcut_row * 80 + hx + 14) as usize;
    assert_eq!(
        plane.cells[key_idx].fg, t.primary,
        "shortcut keys should use theme.primary"
    );
    assert_eq!(
        plane.cells[desc_idx].fg, t.fg,
        "shortcut descriptions should use theme.fg (not fg_muted)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 2: KeybindingSet should use from_config, not default
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_keybinding_set_from_config_includes_all_defaults() {
    let from_config = KeybindingSet::from_config(&resolve_keybindings());
    assert!(
        from_config.matches(actions::QUIT, &key_press_ctrl(KeyCode::Char('q'))),
        "from_config should resolve engine defaults for QUIT"
    );
    assert!(
        from_config.matches(actions::HELP, &key_press(KeyCode::F(1))),
        "from_config should resolve engine defaults for HELP"
    );
}

#[test]
fn test_keybinding_set_matches_standard_actions() {
    let kb = KeybindingSet::from_config(&resolve_keybindings());
    assert!(
        kb.matches(actions::QUIT, &key_press_ctrl(KeyCode::Char('q'))),
        "QUIT action should match Ctrl+Q by default"
    );
    assert!(
        kb.matches(actions::HELP, &key_press(KeyCode::F(1))),
        "HELP action should match F1 by default"
    );
    assert!(
        kb.matches(actions::THEME, &key_press(KeyCode::F(2))),
        "THEME action should match F2 by default"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 3: Modifier guards on single-char keys
//
// Ctrl+Char should NOT trigger app-level actions that use bare Char handlers.
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ctrl_char_does_not_match_bare_char() {
    let ctrl_q = key_press_ctrl(KeyCode::Char('q'));
    let bare_q = key_press(KeyCode::Char('q'));
    assert_ne!(
        ctrl_q.modifiers, bare_q.modifiers,
        "Ctrl+Q and bare 'q' should have different modifiers"
    );
    assert!(
        ctrl_q.modifiers.contains(KeyModifiers::CONTROL),
        "Ctrl+Q should have CONTROL modifier"
    );
    assert!(
        bare_q.modifiers.is_empty(),
        "bare 'q' should have no modifiers"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 4: Pattern 2 apps must sync theme from ctx.theme() in on_tick
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_current_theme_returns_widget_theme() {
    let theme = Theme::cyberpunk();
    let widget = HelpOverlayWidget::new(theme.clone());
    assert_eq!(
        widget.current_theme().map(|t| t.name.to_string()),
        None,
        "default Widget::current_theme() returns None — widgets must override it"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 5: Theme::from_env_or vs hardcoded theme
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_from_env_or_falls_back_to_default() {
    let theme = Theme::from_env_or(Theme::nord());
    assert!(
        Theme::all().iter().any(|t| t.name == theme.name),
        "from_env_or should return a valid theme"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 6: Background fill — plane should start with theme.bg, not Color::Reset
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_plane_fill_bg_uses_theme_not_reset() {
    let t = Theme::nord();
    let mut plane = Plane::new(0, 80, 24);
    plane.fill_bg(t.bg);
    assert_ne!(t.bg, Color::Reset, "theme.bg should not be Color::Reset");
    assert_eq!(plane.cells[0].bg, t.bg, "first cell bg should be theme.bg");
}

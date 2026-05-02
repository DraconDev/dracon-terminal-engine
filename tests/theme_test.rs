//! Tests for the Theme system.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::theme::Theme;

fn assert_theme_name(t: &Theme, expected_name: &str) {
    assert_eq!(t.name, expected_name);
}

fn assert_rgb(t: &Theme, field: &str, r: u8, g: u8, b: u8) {
    let expected = Color::Rgb(r, g, b);
    let actual = match field {
        "bg" => t.bg,
        "fg" => t.fg,
        "primary" => t.primary,
        "secondary" => t.secondary,
        "surface" => t.surface,
        "fg_muted" => t.fg_muted,
        "outline" => t.outline,
        "error" => t.error,
        "success" => t.success,
        "warning" => t.warning,
        "selection_bg" => t.selection_bg,
        "selection_fg" => t.selection_fg,
        "scrollbar_track" => t.scrollbar_track,
        "scrollbar_thumb" => t.scrollbar_thumb,
        "input_bg" => t.input_bg,
        "input_fg" => t.input_fg,
        "disabled_fg" => t.disabled_fg,
        _ => panic!("unknown field: {}", field),
    };
    assert_eq!(actual, expected, "Theme.{} mismatch", field);
}

// === dark theme ===

#[test]
fn test_theme_dark_name() {
    assert_theme_name(&Theme::dark(), "dark");
}

#[test]
fn test_theme_dark_bg() {
    let t = Theme::dark();
    assert_eq!(t.bg, Color::Rgb(16, 16, 24));
}

#[test]
fn test_theme_dark_fg() {
    assert_rgb(&Theme::dark(), "fg", 200, 200, 220);
}

#[test]
fn test_theme_dark_primary() {
    assert_rgb(&Theme::dark(), "primary", 0, 200, 120);
}

#[test]
fn test_theme_dark_surface() {
    assert_rgb(&Theme::dark(), "surface", 24, 24, 36);
}

// === light theme ===

#[test]
fn test_theme_light_name() {
    assert_theme_name(&Theme::light(), "light");
}

#[test]
fn test_theme_light_bg() {
    assert_eq!(Theme::light().bg, Color::Rgb(250, 250, 250));
}

#[test]
fn test_theme_light_fg() {
    assert_rgb(&Theme::light(), "fg", 30, 30, 40);
}

#[test]
fn test_theme_light_primary() {
    assert_rgb(&Theme::light(), "primary", 0, 100, 180);
}

// === cyberpunk theme ===

#[test]
fn test_theme_cyberpunk_name() {
    assert_theme_name(&Theme::cyberpunk(), "cyberpunk");
}

#[test]
fn test_theme_cyberpunk_bg_is_black() {
    assert_eq!(Theme::cyberpunk().bg, Color::Rgb(0, 0, 0));
}

#[test]
fn test_theme_cyberpunk_fg() {
    assert_rgb(&Theme::cyberpunk(), "fg", 0, 255, 136);
}

#[test]
fn test_theme_cyberpunk_primary() {
    assert_rgb(&Theme::cyberpunk(), "primary", 255, 0, 100);
}

#[test]
fn test_theme_cyberpunk_success() {
    assert_rgb(&Theme::cyberpunk(), "success", 0, 255, 180);
}

#[test]
fn test_theme_cyberpunk_error() {
    assert_rgb(&Theme::cyberpunk(), "error", 255, 0, 80);
}

// === dracula theme ===

#[test]
fn test_theme_dracula_name() {
    assert_theme_name(&Theme::dracula(), "dracula");
}

#[test]
fn test_theme_dracula_bg() {
    assert_eq!(Theme::dracula().bg, Color::Rgb(40, 42, 54));
}

#[test]
fn test_theme_dracula_fg() {
    assert_rgb(&Theme::dracula(), "fg", 248, 248, 242);
}

#[test]
fn test_theme_dracula_primary() {
    assert_rgb(&Theme::dracula(), "primary", 98, 114, 164);
}

#[test]
fn test_theme_dracula_success() {
    assert_rgb(&Theme::dracula(), "success", 80, 250, 123);
}

#[test]
fn test_theme_dracula_error() {
    assert_rgb(&Theme::dracula(), "error", 255, 85, 85);
}

// === nord theme ===

#[test]
fn test_theme_nord_name() {
    assert_theme_name(&Theme::nord(), "nord");
}

#[test]
fn test_theme_nord_bg() {
    assert_eq!(Theme::nord().bg, Color::Rgb(46, 52, 64));
}

#[test]
fn test_theme_nord_fg() {
    assert_rgb(&Theme::nord(), "fg", 216, 222, 233);
}

#[test]
fn test_theme_nord_primary() {
    assert_rgb(&Theme::nord(), "primary", 136, 192, 208);
}

#[test]
fn test_theme_nord_error() {
    assert_rgb(&Theme::nord(), "error", 191, 97, 106);
}

#[test]
fn test_theme_nord_success() {
    assert_rgb(&Theme::nord(), "success", 163, 190, 140);
}

// === catppuccin_mocha theme ===

#[test]
fn test_theme_catppuccin_mocha_name() {
    assert_theme_name(&Theme::catppuccin_mocha(), "catppuccin-mocha");
}

#[test]
fn test_theme_catppuccin_mocha_bg() {
    assert_eq!(Theme::catppuccin_mocha().bg, Color::Rgb(30, 30, 46));
}

#[test]
fn test_theme_catppuccin_mocha_fg() {
    assert_rgb(&Theme::catppuccin_mocha(), "fg", 205, 214, 244);
}

#[test]
fn test_theme_catppuccin_mocha_primary() {
    assert_rgb(&Theme::catppuccin_mocha(), "primary", 137, 180, 250);
}

#[test]
fn test_theme_catppuccin_mocha_error() {
    assert_rgb(&Theme::catppuccin_mocha(), "error", 243, 139, 168);
}

// === gruvbox_dark theme ===

#[test]
fn test_theme_gruvbox_dark_name() {
    assert_theme_name(&Theme::gruvbox_dark(), "gruvbox-dark");
}

#[test]
fn test_theme_gruvbox_dark_bg() {
    assert_eq!(Theme::gruvbox_dark().bg, Color::Rgb(40, 40, 40));
}

#[test]
fn test_theme_gruvbox_dark_fg() {
    assert_rgb(&Theme::gruvbox_dark(), "fg", 213, 196, 161);
}

#[test]
fn test_theme_gruvbox_dark_primary() {
    assert_rgb(&Theme::gruvbox_dark(), "primary", 214, 93, 14);
}

#[test]
fn test_theme_gruvbox_dark_error() {
    assert_rgb(&Theme::gruvbox_dark(), "error", 204, 36, 36);
}

// === tokyo_night theme ===

#[test]
fn test_theme_tokyo_night_name() {
    assert_theme_name(&Theme::tokyo_night(), "tokyo-night");
}

#[test]
fn test_theme_tokyo_night_bg() {
    assert_eq!(Theme::tokyo_night().bg, Color::Rgb(32, 34, 44));
}

#[test]
fn test_theme_tokyo_night_fg() {
    assert_rgb(&Theme::tokyo_night(), "fg", 192, 202, 245);
}

#[test]
fn test_theme_tokyo_night_primary() {
    assert_rgb(&Theme::tokyo_night(), "primary", 98, 130, 234);
}

#[test]
fn test_theme_tokyo_night_error() {
    assert_rgb(&Theme::tokyo_night(), "error", 255, 85, 85);
}

// === solarized_dark theme ===

#[test]
fn test_theme_solarized_dark_name() {
    assert_theme_name(&Theme::solarized_dark(), "solarized-dark");
}

#[test]
fn test_theme_solarized_dark_bg() {
    assert_eq!(Theme::solarized_dark().bg, Color::Rgb(0, 43, 54));
}

#[test]
fn test_theme_solarized_dark_fg() {
    assert_rgb(&Theme::solarized_dark(), "fg", 131, 148, 150);
}

#[test]
fn test_theme_solarized_dark_primary() {
    assert_rgb(&Theme::solarized_dark(), "primary", 38, 139, 210);
}

// === solarized_light theme ===

#[test]
fn test_theme_solarized_light_name() {
    assert_theme_name(&Theme::solarized_light(), "solarized-light");
}

#[test]
fn test_theme_solarized_light_bg() {
    assert_eq!(Theme::solarized_light().bg, Color::Rgb(253, 246, 227));
}

#[test]
fn test_theme_solarized_light_fg() {
    assert_rgb(&Theme::solarized_light(), "fg", 101, 123, 131);
}

#[test]
fn test_theme_solarized_light_primary() {
    assert_rgb(&Theme::solarized_light(), "primary", 38, 139, 210);
}

// === one_dark theme ===

#[test]
fn test_theme_one_dark_name() {
    assert_theme_name(&Theme::one_dark(), "one-dark");
}

#[test]
fn test_theme_one_dark_bg() {
    assert_eq!(Theme::one_dark().bg, Color::Rgb(40, 44, 52));
}

#[test]
fn test_theme_one_dark_fg() {
    assert_rgb(&Theme::one_dark(), "fg", 220, 223, 228);
}

#[test]
fn test_theme_one_dark_primary() {
    assert_rgb(&Theme::one_dark(), "primary", 97, 175, 239);
}

#[test]
fn test_theme_one_dark_error() {
    assert_rgb(&Theme::one_dark(), "error", 224, 108, 108);
}

// === rose_pine theme ===

#[test]
fn test_theme_rose_pine_name() {
    assert_theme_name(&Theme::rose_pine(), "rose-pine");
}

#[test]
fn test_theme_rose_pine_bg() {
    assert_eq!(Theme::rose_pine().bg, Color::Rgb(30, 30, 46));
}

#[test]
fn test_theme_rose_pine_fg() {
    assert_rgb(&Theme::rose_pine(), "fg", 220, 200, 200);
}

#[test]
fn test_theme_rose_pine_primary() {
    assert_rgb(&Theme::rose_pine(), "primary", 210, 160, 160);
}

// === kanagawa theme ===

#[test]
fn test_theme_kanagawa_name() {
    assert_theme_name(&Theme::kanagawa(), "kanagawa");
}

#[test]
fn test_theme_kanagawa_bg() {
    assert_eq!(Theme::kanagawa().bg, Color::Rgb(38, 40, 64));
}

#[test]
fn test_theme_kanagawa_fg() {
    assert_rgb(&Theme::kanagawa(), "fg", 220, 217, 201);
}

#[test]
fn test_theme_kanagawa_primary() {
    assert_rgb(&Theme::kanagawa(), "primary", 166, 122, 102);
}

// === everforest theme ===

#[test]
fn test_theme_everforest_name() {
    assert_theme_name(&Theme::everforest(), "everforest");
}

#[test]
fn test_theme_everforest_bg() {
    assert_eq!(Theme::everforest().bg, Color::Rgb(43, 48, 40));
}

#[test]
fn test_theme_everforest_fg() {
    assert_rgb(&Theme::everforest(), "fg", 210, 191, 163);
}

#[test]
fn test_theme_everforest_primary() {
    assert_rgb(&Theme::everforest(), "primary", 148, 181, 97);
}

// === monokai theme ===

#[test]
fn test_theme_monokai_name() {
    assert_theme_name(&Theme::monokai(), "monokai");
}

#[test]
fn test_theme_monokai_bg() {
    assert_eq!(Theme::monokai().bg, Color::Rgb(39, 40, 34));
}

#[test]
fn test_theme_monokai_fg() {
    assert_rgb(&Theme::monokai(), "fg", 248, 248, 242);
}

#[test]
fn test_theme_monokai_primary() {
    assert_rgb(&Theme::monokai(), "primary", 102, 217, 239);
}

// === Default ===

#[test]
fn test_theme_default_is_dark() {
    let default = Theme::default();
    assert_theme_name(&default, "dark");
    assert_eq!(default.bg, Theme::dark().bg);
    assert_eq!(default.fg, Theme::dark().fg);
    assert_eq!(default.primary, Theme::dark().primary);
}

// === Trait derives ===

#[test]
fn test_theme_clone() {
    let a = Theme::cyberpunk();
    let b = a;
    assert_eq!(a.name, b.name);
    assert_eq!(a.bg, b.bg);
    assert_eq!(a.fg, b.fg);
}

#[test]
fn test_theme_copy() {
    let a = Theme::dracula();
    let b = a;
    assert_eq!(a.name, b.name);
}

#[test]
fn test_theme_partial_eq() {
    let a = Theme::nord();
    let b = Theme::nord();
    let c = Theme::catppuccin_mocha();
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_theme_debug() {
    let t = Theme::rose_pine();
    let debug = format!("{:?}", t);
    assert!(debug.contains("rose-pine"));
}

// === Scrollbar width invariant ===

#[test]
fn test_all_themes_have_scrollbar_width_1() {
    for theme in [
        Theme::dark(),
        Theme::light(),
        Theme::cyberpunk(),
        Theme::dracula(),
        Theme::nord(),
        Theme::catppuccin_mocha(),
        Theme::gruvbox_dark(),
        Theme::tokyo_night(),
        Theme::solarized_dark(),
        Theme::solarized_light(),
        Theme::one_dark(),
        Theme::rose_pine(),
        Theme::kanagawa(),
        Theme::everforest(),
        Theme::monokai(),
    ] {
        assert_eq!(
            theme.scrollbar_width, 1,
            "scrollbar_width for {} should be 1",
            theme.name
        );
    }
}

// === No color is Color::Reset in built-in themes ===

#[test]
fn test_theme_bg_is_never_reset() {
    for theme in [
        Theme::dark(),
        Theme::light(),
        Theme::cyberpunk(),
        Theme::dracula(),
        Theme::nord(),
        Theme::catppuccin_mocha(),
        Theme::gruvbox_dark(),
        Theme::tokyo_night(),
        Theme::solarized_dark(),
        Theme::solarized_light(),
        Theme::one_dark(),
        Theme::rose_pine(),
        Theme::kanagawa(),
        Theme::everforest(),
        Theme::monokai(),
    ] {
        assert!(
            !matches!(theme.bg, Color::Reset),
            "theme {} has bg=Reset",
            theme.name
        );
    }
}

// === Semantic colors differ from bg/fg (sanity check) ===

#[test]
fn test_theme_primary_differs_from_fg() {
    let t = Theme::dark();
    assert_ne!(t.primary, t.fg, "primary should differ from fg in dark theme");
}

#[test]
fn test_theme_error_differs_from_fg() {
    let t = Theme::dark();
    assert_ne!(
        t.error, t.fg,
        "error should differ from fg in dark theme"
    );
}

#[test]
fn test_theme_success_differs_from_fg() {
    let t = Theme::light();
    assert_ne!(
        t.success, t.fg,
        "success should differ from fg in light theme"
    );
}

// === ThemeKind tests ===

#[test]
fn test_theme_dark_is_dark_kind() {
    assert_eq!(Theme::dark().kind, dracon_terminal_engine::framework::theme::ThemeKind::Dark);
}

#[test]
fn test_theme_light_is_light_kind() {
    assert_eq!(Theme::light().kind, dracon_terminal_engine::framework::theme::ThemeKind::Light);
}

#[test]
fn test_cyberpunk_is_dark_kind() {
    assert_eq!(Theme::cyberpunk().kind, dracon_terminal_engine::framework::theme::ThemeKind::Dark);
}
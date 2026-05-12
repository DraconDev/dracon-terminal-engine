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
    for theme in Theme::all() {
        assert_eq!(
            theme.scrollbar_width, 1,
            "scrollbar_width for {} should be 1",
            theme.name
        );
    }
}

#[test]
fn test_theme_all_returns_21_themes() {
    let themes = Theme::all();
    assert_eq!(themes.len(), 21, "Theme::all() should return 21 themes");
    // Verify no duplicate names
    let mut names: Vec<&str> = themes.iter().map(|t| t.name).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 21, "all 21 themes should have unique names");
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
    assert_ne!(
        t.primary, t.fg,
        "primary should differ from fg in dark theme"
    );
}

#[test]
fn test_theme_error_differs_from_fg() {
    let t = Theme::dark();
    assert_ne!(t.error, t.fg, "error should differ from fg in dark theme");
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
    assert_eq!(
        Theme::dark().kind,
        dracon_terminal_engine::framework::theme::ThemeKind::Dark
    );
}

#[test]
fn test_theme_light_is_light_kind() {
    assert_eq!(
        Theme::light().kind,
        dracon_terminal_engine::framework::theme::ThemeKind::Light
    );
}

#[test]
fn test_cyberpunk_is_dark_kind() {
    assert_eq!(
        Theme::cyberpunk().kind,
        dracon_terminal_engine::framework::theme::ThemeKind::Dark
    );
}

// === warm theme ===

#[test]
fn test_theme_warm_name() {
    assert_theme_name(&Theme::warm(), "warm");
}

#[test]
fn test_theme_warm_bg() {
    assert_eq!(Theme::warm().bg, Color::Rgb(28, 26, 24));
}

#[test]
fn test_theme_warm_primary() {
    assert_rgb(&Theme::warm(), "primary", 224, 164, 90);
}

#[test]
fn test_theme_warm_secondary() {
    assert_rgb(&Theme::warm(), "secondary", 94, 199, 178);
}

// === cool theme ===

#[test]
fn test_theme_cool_name() {
    assert_theme_name(&Theme::cool(), "cool");
}

#[test]
fn test_theme_cool_bg() {
    assert_eq!(Theme::cool().bg, Color::Rgb(24, 26, 32));
}

#[test]
fn test_theme_cool_primary() {
    assert_rgb(&Theme::cool(), "primary", 160, 118, 255);
}

#[test]
fn test_theme_cool_secondary() {
    assert_rgb(&Theme::cool(), "secondary", 116, 184, 255);
}

// === forest theme ===

#[test]
fn test_theme_forest_name() {
    assert_theme_name(&Theme::forest(), "forest");
}

#[test]
fn test_theme_forest_bg() {
    assert_eq!(Theme::forest().bg, Color::Rgb(24, 30, 26));
}

#[test]
fn test_theme_forest_primary() {
    assert_rgb(&Theme::forest(), "primary", 126, 196, 102);
}

#[test]
fn test_theme_forest_secondary() {
    assert_rgb(&Theme::forest(), "secondary", 86, 168, 142);
}

// === sunset theme ===

#[test]
fn test_theme_sunset_name() {
    assert_theme_name(&Theme::sunset(), "sunset");
}

#[test]
fn test_theme_sunset_bg() {
    assert_eq!(Theme::sunset().bg, Color::Rgb(32, 24, 26));
}

#[test]
fn test_theme_sunset_primary() {
    assert_rgb(&Theme::sunset(), "primary", 236, 146, 98);
}

#[test]
fn test_theme_sunset_secondary() {
    assert_rgb(&Theme::sunset(), "secondary", 236, 99, 141);
}

// === mono theme ===

#[test]
fn test_theme_mono_name() {
    assert_theme_name(&Theme::mono(), "mono");
}

#[test]
fn test_theme_mono_bg() {
    assert_eq!(Theme::mono().bg, Color::Rgb(26, 28, 32));
}

#[test]
fn test_theme_mono_primary() {
    assert_rgb(&Theme::mono(), "primary", 210, 214, 224);
}

#[test]
fn test_theme_mono_secondary() {
    assert_rgb(&Theme::mono(), "secondary", 162, 172, 188);
}

// === Theme::from_name() tests ===

#[test]
fn test_from_name_all_themes_by_exact_name() {
    let themes = [
        ("dark", Theme::dark()),
        ("light", Theme::light()),
        ("high_contrast", Theme::high_contrast()),
        ("cyberpunk", Theme::cyberpunk()),
        ("dracula", Theme::dracula()),
        ("nord", Theme::nord()),
        ("catppuccin-mocha", Theme::catppuccin_mocha()),
        ("gruvbox-dark", Theme::gruvbox_dark()),
        ("tokyo-night", Theme::tokyo_night()),
        ("solarized-dark", Theme::solarized_dark()),
        ("solarized-light", Theme::solarized_light()),
        ("one-dark", Theme::one_dark()),
        ("rose-pine", Theme::rose_pine()),
        ("kanagawa", Theme::kanagawa()),
        ("everforest", Theme::everforest()),
        ("monokai", Theme::monokai()),
        ("warm", Theme::warm()),
        ("cool", Theme::cool()),
        ("forest", Theme::forest()),
        ("sunset", Theme::sunset()),
        ("mono", Theme::mono()),
    ];
    for (name, expected) in themes.iter() {
        let resolved = Theme::from_name(name).expect(&format!("should resolve theme: {}", name));
        assert_eq!(resolved.name, expected.name, "name mismatch for {}", name);
        assert_eq!(resolved.bg, expected.bg, "bg mismatch for {}", name);
    }
}

#[test]
fn test_from_name_hyphenated_aliases() {
    assert_eq!(Theme::from_name("catppuccin-mocha").unwrap().name, "catppuccin-mocha");
    assert_eq!(Theme::from_name("gruvbox-dark").unwrap().name, "gruvbox-dark");
    assert_eq!(Theme::from_name("solarized-dark").unwrap().name, "solarized-dark");
    assert_eq!(Theme::from_name("solarized-light").unwrap().name, "solarized-light");
    assert_eq!(Theme::from_name("tokyo-night").unwrap().name, "tokyo-night");
    assert_eq!(Theme::from_name("one-dark").unwrap().name, "one-dark");
    assert_eq!(Theme::from_name("rose-pine").unwrap().name, "rose-pine");
    assert_eq!(Theme::from_name("high-contrast").unwrap().name, "high_contrast");
}

#[test]
fn test_from_name_underscore_aliases() {
    assert_eq!(Theme::from_name("catppuccin_mocha").unwrap().name, "catppuccin-mocha");
    assert_eq!(Theme::from_name("gruvbox_dark").unwrap().name, "gruvbox-dark");
    assert_eq!(Theme::from_name("solarized_dark").unwrap().name, "solarized-dark");
    assert_eq!(Theme::from_name("solarized_light").unwrap().name, "solarized-light");
    assert_eq!(Theme::from_name("tokyo_night").unwrap().name, "tokyo-night");
    assert_eq!(Theme::from_name("one_dark").unwrap().name, "one-dark");
    assert_eq!(Theme::from_name("rose_pine").unwrap().name, "rose-pine");
    assert_eq!(Theme::from_name("high_contrast").unwrap().name, "high_contrast");
}

#[test]
fn test_from_name_case_insensitive() {
    assert_eq!(Theme::from_name("DARK").unwrap().name, "dark");
    assert_eq!(Theme::from_name("Dark").unwrap().name, "dark");
    assert_eq!(Theme::from_name("NORD").unwrap().name, "nord");
    assert_eq!(Theme::from_name("Catppuccin-Mocha").unwrap().name, "catppuccin-mocha");
}

#[test]
fn test_from_name_unknown_returns_none() {
    assert!(Theme::from_name("nonexistent").is_none());
    assert!(Theme::from_name("").is_none());
}

#[test]
fn test_from_name_short_aliases() {
    assert_eq!(Theme::from_name("catppuccin").unwrap().name, "catppuccin-mocha");
    assert_eq!(Theme::from_name("gruvbox").unwrap().name, "gruvbox-dark");
}

// === Theme::from_env_or() tests ===

#[test]
fn test_from_env_or_uses_env_var() {
    // Save original value to restore later
    let original = std::env::var("DTRON_THEME").ok();
    std::env::set_var("DTRON_THEME", "nord");
    let theme = Theme::from_env_or(Theme::dark());
    assert_eq!(theme.name, "nord");
    // Restore
    match original {
        Some(v) => std::env::set_var("DTRON_THEME", v),
        None => std::env::remove_var("DTRON_THEME"),
    }
}

#[test]
fn test_from_env_or_falls_back_on_invalid_name() {
    let original = std::env::var("DTRON_THEME").ok();
    std::env::set_var("DTRON_THEME", "nonexistent_theme");
    let theme = Theme::from_env_or(Theme::dark());
    assert_eq!(theme.name, "dark");
    match original {
        Some(v) => std::env::set_var("DTRON_THEME", v),
        None => std::env::remove_var("DTRON_THEME"),
    }
}

#[test]
fn test_from_env_or_falls_back_when_unset() {
    let original = std::env::var("DTRON_THEME").ok();
    std::env::remove_var("DTRON_THEME");
    let theme = Theme::from_env_or(Theme::light());
    assert_eq!(theme.name, "light");
    match original {
        Some(v) => std::env::set_var("DTRON_THEME", v),
        None => {}
    }
}

#[test]
fn test_from_env_or_hyphenated_theme_name() {
    let original = std::env::var("DTRON_THEME").ok();
    std::env::set_var("DTRON_THEME", "catppuccin-mocha");
    let theme = Theme::from_env_or(Theme::dark());
    assert_eq!(theme.name, "catppuccin-mocha");
    match original {
        Some(v) => std::env::set_var("DTRON_THEME", v),
        None => std::env::remove_var("DTRON_THEME"),
    }
}

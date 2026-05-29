//! File-type icon rendering for file managers and explorers.
//!
//! This module provides:
//!
//! - [`Icon`] enum — Icon types for files, folders, and UI elements
//! - [`IconMode`] — Rendering mode (Nerd Font, emoji, ASCII, box-drawing)
//! - [`render_icon()`] — Get icon string for a file category
//!
//! Icons are rendered based on file type detection and the selected icon mode.

use crate::utils::{FileCategory, IconMode};
use serde::{Deserialize, Serialize};

/// Represents an icon type for files, folders, and UI elements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Icon {
    /// Folder icon for directories.
    Folder,
    /// Generic file icon.
    File,
    /// Star/favorite icon.
    Star,
    /// Storage device icon.
    Storage,
    /// Remote/network icon.
    Remote,
    /// Git version control icon.
    Git,
    /// Archive/compressed file icon.
    Archive,
    /// Image file icon.
    Image,
    /// Audio file icon.
    Audio,
    /// Video file icon.
    Video,
    /// Script/executable file icon.
    Script,
    /// Document file icon.
    Document,
    /// Search icon.
    Search,
    /// Split/pane icon.
    Split,
    /// Single/pane icon.
    Single,
    /// Back/navigation icon.
    Back,
    /// Forward/navigation icon.
    Forward,
    /// Burger/hamburger menu icon.
    Burger,
    /// Refresh icon.
    Refresh,
    /// Monitor/display icon.
    Monitor,
    /// Settings/gear icon.
    Settings,
    /// Trash/delete icon.
    Trash,
    /// Home directory icon.
    Home,
    /// Downloads directory icon.
    Downloads,
    /// Documents directory icon.
    Documents,
    /// Pictures directory icon.
    Pictures,
    /// Music directory icon.
    Music,
    /// Videos directory icon.
    Videos,
    /// Rust source file icon.
    Rust,
    /// C source file icon.
    C,
    /// C++ source file icon.
    Cpp,
    /// Go source file icon.
    Go,
    /// Java source file icon.
    Java,
    /// Kotlin source file icon.
    Kotlin,
    /// Python source file icon.
    Python,
    /// Ruby source file icon.
    Ruby,
    /// PHP source file icon.
    Php,
    /// HTML file icon.
    Html,
    /// CSS file icon.
    Css,
    /// YAML file icon.
    Yaml,
    /// XML file icon.
    Xml,
    /// SQL file icon.
    Sql,
    /// Shell script icon.
    Shell,
    /// LaTeX/typeset document icon.
    Tex,
    /// Lock file icon.
    Lock,
    /// Docker/containers icon.
    Docker,
    /// Build system icon (Make, CMake).
    Build,
    /// Config/dotfile icon.
    Config,
    /// JSON file icon.
    Json,
    /// TOML file icon.
    Toml,
    /// Markdown file icon.
    Markdown,
    /// Select all icon.
    SelectAll,
    /// Toggle hidden files icon.
    ToggleHidden,
    /// Set wallpaper icon.
    SetWallpaper,
    /// Paste icon.
    Paste,
    /// Cut icon.
    Cut,
    /// Copy icon.
    Copy,
    /// Duplicate icon.
    Duplicate,
    /// Delete icon.
    Delete,
    /// Rename icon.
    Rename,
}

impl Icon {
    /// Returns the appropriate icon string for a file path based on its category and type.
    pub fn get_for_path(
        path: &std::path::Path,
        category: FileCategory,
        is_dir: bool,
        icon_mode: IconMode,
    ) -> &'static str {
        if is_dir {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            let icon = match name.as_str() {
                "home" => Icon::Home,
                "downloads" => Icon::Downloads,
                "documents" => Icon::Documents,
                "pictures" => Icon::Pictures,
                "music" => Icon::Music,
                "videos" => Icon::Videos,
                "desktop" => Icon::Monitor,
                ".local" | ".config" | ".cache" => Icon::Settings,
                ".trash" | "trash" => Icon::Trash,
                _ => Icon::Folder,
            };
            return icon.get(icon_mode);
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let name_lower = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        let icon = match ext.as_str() {
            "rs" => Icon::Rust,
            "c" | "h" => Icon::C,
            "cpp" | "cc" | "cxx" | "hpp" => Icon::Cpp,
            "go" => Icon::Go,
            "java" => Icon::Java,
            "kt" => Icon::Kotlin,
            "py" => Icon::Python,
            "rb" => Icon::Ruby,
            "php" => Icon::Php,
            "html" => Icon::Html,
            "css" | "scss" => Icon::Css,
            "yaml" | "yml" => Icon::Yaml,
            "xml" => Icon::Xml,
            "json" => Icon::Json,
            "sql" => Icon::Sql,
            "toml" => Icon::Toml,
            "md" => Icon::Markdown,
            "sh" | "bash" | "zsh" | "fish" => Icon::Shell,
            "tex" | "rst" => Icon::Tex,
            "lock" => Icon::Lock,
            "env" | "ini" | "cfg" | "conf" => Icon::Config,
            _ => match name_lower.as_str() {
                "dockerfile" => Icon::Docker,
                "makefile" | "cmake" => Icon::Build,
                _ => match category {
                    FileCategory::Archive => Icon::Archive,
                    FileCategory::Image => Icon::Image,
                    FileCategory::Audio => Icon::Audio,
                    FileCategory::Video => Icon::Videos,
                    FileCategory::Script => Icon::Script,
                    FileCategory::Document => Icon::Document,
                    _ => Icon::File,
                },
            },
        };
        icon.get(icon_mode)
    }

    /// Returns the icon string for this icon in the specified display mode.
    pub fn get(&self, mode: IconMode) -> &'static str {
        match mode {
            IconMode::Nerd => match self {
                Icon::Folder => "󰉋 ",
                Icon::File => "󰈔 ",
                Icon::Star => "󰓎 ",
                Icon::Storage => "󰋊 ",
                Icon::Remote => "󰒍 ",
                Icon::Git => "󰊢 ",
                Icon::Archive => "󰛫 ",
                Icon::Image => "󰸉 ",
                Icon::Audio => "󰝚 ",
                Icon::Video => "󰐊 ",
                Icon::Script => "󰞷 ",
                Icon::Document => "󰈙 ",
                Icon::Search => "󰍉 ",
                Icon::Split => "󰙀 ",
                Icon::Single => "󰇄 ",
                Icon::Back => "󰁍 ",
                Icon::Forward => "󰁔 ",
                Icon::Burger => "󰍜 ",
                Icon::Refresh => "󰑓 ",
                Icon::Monitor => "󰐠 ",
                Icon::Settings => "󰒓 ",
                Icon::Trash => "󰆴 ",
                Icon::Home => "󰋜 ",
                Icon::Downloads => "󰇚 ",
                Icon::Documents => "󰈙 ",
                Icon::Pictures => "󰸉 ",
                Icon::Music => "󰝚 ",
                Icon::Videos => "󰐊 ",
                Icon::Rust => "󱘗 ",
                Icon::C => "󰅴 ",
                Icon::Cpp => "󰙲 ",
                Icon::Go => "󰟓 ",
                Icon::Java => "󰬇 ",
                Icon::Kotlin => "󰲙 ",
                Icon::Python => "󰌠 ",
                Icon::Ruby => "󰴍 ",
                Icon::Php => "󰌞 ",
                Icon::Html => "󰌝 ",
                Icon::Css => "󰌜 ",
                Icon::Yaml => "󰂷 ",
                Icon::Xml => "󰒝 ",
                Icon::Sql => "󰆼 ",
                Icon::Shell => "󰞷 ",
                Icon::Tex => "󰊠 ",
                Icon::Lock => "󰌾 ",
                Icon::Docker => "󰡨 ",
                Icon::Build => "󱁤 ",
                Icon::Config => "󰒓 ",
                Icon::Json => "󬭦 ",
                Icon::Toml => "󱘗 ",
                Icon::Markdown => "󰍔 ",
                Icon::SelectAll => "󰒆 ",
                Icon::ToggleHidden => "󰈈 ",
                Icon::SetWallpaper => "󰸉 ",
                Icon::Paste => "󰆒 ",
                Icon::Cut => "󰆐 ",
                Icon::Copy => "󰆏 ",
                Icon::Duplicate => "󰆏 ",
                Icon::Delete => "󰆴 ",
                Icon::Rename => "󰏫 ",
            },
            IconMode::Unicode => match self {
                Icon::Folder => "> ",
                Icon::File => "- ",
                Icon::Star => "* ",
                Icon::Storage => "D ",
                Icon::Remote => "~ ",
                Icon::Git => "@ ",
                Icon::Archive => "# ",
                Icon::Image => "I ",
                Icon::Audio => "~ ",
                Icon::Video => "> ",
                Icon::Script => "$ ",
                Icon::Document => "= ",
                Icon::Search => "/ ",
                Icon::Split => "|| ",
                Icon::Single => "[] ",
                Icon::Back => "< ",
                Icon::Forward => "> ",
                Icon::Burger => "= ",
                Icon::Refresh => "R ",
                Icon::Monitor => "M ",
                Icon::Settings => "S ",
                Icon::Trash => "X ",
                Icon::Home => "H ",
                Icon::Downloads => "D ",
                Icon::Documents => "T ",
                Icon::Pictures => "P ",
                Icon::Music => "~ ",
                Icon::Videos => "> ",
                Icon::Rust => "R ",
                Icon::C => "C ",
                Icon::Cpp => "C+ ",
                Icon::Go => "G ",
                Icon::Java => "J ",
                Icon::Kotlin => "K ",
                Icon::Python => "Py ",
                Icon::Ruby => "Rb ",
                Icon::Php => "P ",
                Icon::Html => "<> ",
                Icon::Css => "# ",
                Icon::Yaml => "Y ",
                Icon::Xml => "<> ",
                Icon::Sql => "S ",
                Icon::Shell => "$ ",
                Icon::Tex => "T ",
                Icon::Lock => "L ",
                Icon::Docker => "D ",
                Icon::Build => "B ",
                Icon::Config => "C ",
                Icon::Json => "{ ",
                Icon::Toml => "T ",
                Icon::Markdown => "M ",
                Icon::SelectAll => "A ",
                Icon::ToggleHidden => "H ",
                Icon::SetWallpaper => "W ",
                Icon::Paste => "P ",
                Icon::Cut => "X ",
                Icon::Copy => "C ",
                Icon::Duplicate => "C ",
                Icon::Delete => "X ",
                Icon::Rename => "R ",
            },
            IconMode::ASCII => match self {
                Icon::Folder => "[D] ",
                Icon::File => "[F] ",
                Icon::Star => "[*] ",
                Icon::Storage => "[S] ",
                Icon::Remote => "[R] ",
                Icon::Git => "[G] ",
                Icon::Archive => "[Z] ",
                Icon::Image => "[I] ",
                Icon::Audio => "[A] ",
                Icon::Video => "[V] ",
                Icon::Script => "[X] ",
                Icon::Document => "[T] ",
                Icon::Search => "/ ",
                Icon::Split => "[S] ",
                Icon::Single => "[1] ",
                Icon::Back => "< ",
                Icon::Forward => "> ",
                Icon::Burger => "[=] ",
                Icon::Refresh => "[R] ",
                Icon::Monitor => "[M] ",
                Icon::Settings => "[S] ",
                Icon::Trash => "[X] ",
                Icon::Home => "[H] ",
                Icon::Downloads => "[v] ",
                Icon::Documents => "[D] ",
                Icon::Pictures => "[P] ",
                Icon::Music => "[M] ",
                Icon::Videos => "[V] ",
                Icon::Rust => "[R] ",
                Icon::C => "[C] ",
                Icon::Cpp => "[C+] ",
                Icon::Go => "[G] ",
                Icon::Java => "[J] ",
                Icon::Kotlin => "[K] ",
                Icon::Python => "[Py] ",
                Icon::Ruby => "[Rb] ",
                Icon::Php => "[P] ",
                Icon::Html => "[<] ",
                Icon::Css => "[#] ",
                Icon::Yaml => "[Y] ",
                Icon::Xml => "[X] ",
                Icon::Sql => "[S] ",
                Icon::Shell => "[X] ",
                Icon::Tex => "[T] ",
                Icon::Lock => "[L] ",
                Icon::Docker => "[D] ",
                Icon::Build => "[B] ",
                Icon::Config => "[C] ",
                Icon::Json => "[J] ",
                Icon::Toml => "[T] ",
                Icon::Markdown => "[M] ",
                Icon::SelectAll => "[A] ",
                Icon::ToggleHidden => "[H] ",
                Icon::SetWallpaper => "[W] ",
                Icon::Paste => "[P] ",
                Icon::Cut => "[X] ",
                Icon::Copy => "[C] ",
                Icon::Duplicate => "[D] ",
                Icon::Delete => "[D] ",
                Icon::Rename => "[R] ",
            },
        }
    }

    /// Returns the display width (in terminal cells) of this icon in the given mode.
    ///
    /// - ASCII mode: always 1 (single-width characters)
    /// - Nerd Font / Unicode mode: 2 (icons may occupy 2 cells due to
    ///   wide CJK/emoji characters or private-use-area glyphs)
    ///
    /// Use this for correct padding in fixed-width layouts.
    pub fn width(&self, mode: IconMode) -> usize {
        match mode {
            IconMode::ASCII => 1,
            IconMode::Nerd | IconMode::Unicode => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_get_nerd() {
        let icon = Icon::File;
        let s = icon.get(IconMode::Nerd);
        assert_eq!(s, "󰈔 ");
    }

    #[test]
    fn test_icon_get_ascii() {
        let icon = Icon::File;
        let s = icon.get(IconMode::ASCII);
        assert_eq!(s, "[F] ");
    }

    #[test]
    fn test_icon_get_unicode() {
        let icon = Icon::File;
        let s = icon.get(IconMode::Unicode);
        assert_eq!(s, "- ");
    }

    #[test]
    fn test_icon_get_unicode_mode() {
        let icon = Icon::File;
        let s = icon.get(IconMode::Unicode);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_icon_get_all_variants() {
        // Test that all icon variants have a string for each mode
        let modes = [IconMode::Nerd, IconMode::ASCII, IconMode::Unicode];
        let variants = [
            Icon::File,
            Icon::Folder,
            Icon::Git,
            Icon::Config,
            Icon::Markdown,
        ];
        for icon in variants {
            for mode in modes {
                let s = icon.get(mode);
                assert!(
                    !s.is_empty(),
                    "Icon {:?} should have content for mode {:?}",
                    icon,
                    mode
                );
            }
        }
    }

    #[test]
    fn test_icon_width_nerd() {
        let icon = Icon::File;
        assert_eq!(icon.width(IconMode::Nerd), 2);
    }

    #[test]
    fn test_icon_width_ascii() {
        let icon = Icon::File;
        assert_eq!(icon.width(IconMode::ASCII), 1);
    }

    #[test]
    fn test_icon_width_unicode() {
        let icon = Icon::File;
        assert_eq!(icon.width(IconMode::Unicode), 2);
    }

    #[test]
    fn test_icon_mode_nerd() {
        // Test that Nerd mode is a valid variant
        let mode = IconMode::Nerd;
        let icon = Icon::File;
        let s = icon.get(mode);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_get_for_path_rust_file() {
        let icon = Icon::get_for_path(
            "test.rs".as_ref(),
            FileCategory::Script,
            false,
            IconMode::Nerd,
        );
        assert_eq!(icon, "󱘗 ");
    }

    #[test]
    fn test_get_for_path_image() {
        let icon = Icon::get_for_path(
            "photo.jpg".as_ref(),
            FileCategory::Image,
            false,
            IconMode::Nerd,
        );
        assert_eq!(icon, "󰸉 ");
    }

    #[test]
    fn test_get_for_path_archive() {
        let icon = Icon::get_for_path(
            "archive.tar.gz".as_ref(),
            FileCategory::Archive,
            false,
            IconMode::ASCII,
        );
        assert_eq!(icon, "[Z] ");
    }

    #[test]
    fn test_get_for_path_rust_ascii() {
        let icon = Icon::get_for_path(
            "lib.rs".as_ref(),
            FileCategory::Script,
            false,
            IconMode::ASCII,
        );
        assert_eq!(icon, "[R] ");
    }

    #[test]
    fn test_get_for_path_json() {
        let icon = Icon::get_for_path(
            "config.json".as_ref(),
            FileCategory::Other,
            false,
            IconMode::ASCII,
        );
        assert_eq!(icon, "[J] ");
    }

    #[test]
    fn test_get_for_path_lock() {
        let icon = Icon::get_for_path(
            "secrets.env".as_ref(),
            FileCategory::Text,
            false,
            IconMode::ASCII,
        );
        // .env files are detected as Config icon
        assert_eq!(icon, "[C] ");
    }

    #[test]
    fn test_get_for_path_markdown() {
        let icon = Icon::get_for_path(
            "README.md".as_ref(),
            FileCategory::Document,
            false,
            IconMode::ASCII,
        );
        assert_eq!(icon, "[M] ");
    }

    #[test]
    fn test_get_for_path_dockerfile() {
        let icon = Icon::get_for_path(
            "Dockerfile".as_ref(),
            FileCategory::Text,
            false,
            IconMode::Nerd,
        );
        assert_eq!(icon, "󰡨 ");
    }

    #[test]
    fn test_get_for_path_makefile() {
        let icon = Icon::get_for_path(
            "Makefile".as_ref(),
            FileCategory::Script,
            false,
            IconMode::Nerd,
        );
        assert_eq!(icon, "󱁤 ");
    }

    #[test]
    fn test_get_for_path_go() {
        let icon = Icon::get_for_path(
            "main.go".as_ref(),
            FileCategory::Script,
            false,
            IconMode::Nerd,
        );
        assert_eq!(icon, "󰟓 ");
    }

    #[test]
    fn test_get_for_path_python() {
        let icon = Icon::get_for_path(
            "script.py".as_ref(),
            FileCategory::Script,
            false,
            IconMode::ASCII,
        );
        assert_eq!(icon, "[Py] ");
    }

    #[test]
    fn test_get_for_path_directory() {
        let icon = Icon::get_for_path("src".as_ref(), FileCategory::Other, true, IconMode::Nerd);
        assert_eq!(icon, "󰉋 ");
    }

    #[test]
    fn test_get_for_path_directory_home() {
        let icon = Icon::get_for_path("home".as_ref(), FileCategory::Other, true, IconMode::Nerd);
        assert_eq!(icon, "󰋜 ");
    }

    #[test]
    fn test_get_for_path_directory_downloads() {
        let icon = Icon::get_for_path(
            "Downloads".as_ref(),
            FileCategory::Other,
            true,
            IconMode::Nerd,
        );
        assert_eq!(icon, "󰇚 ");
    }
}

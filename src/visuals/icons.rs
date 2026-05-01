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
        let icon = match ext.as_str() {
            "rs" => Icon::Rust,
            "json" => Icon::Json,
            "toml" => Icon::Toml,
            "md" => Icon::Markdown,
            "sh" | "bash" | "py" | "js" | "ts" => Icon::Script,
            _ => match category {
                FileCategory::Archive => Icon::Archive,
                FileCategory::Image => Icon::Image,
                FileCategory::Audio => Icon::Audio,
                FileCategory::Video => Icon::Videos,
                FileCategory::Script => Icon::Script,
                FileCategory::Document => Icon::Document,
                _ => Icon::File,
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
                Icon::Folder => "▸ ",
                Icon::File => "▪ ",
                Icon::Star => "★ ",
                Icon::Storage => "⛁ ",
                Icon::Remote => "☁ ",
                Icon::Git => "± ",
                Icon::Archive => "⚑ ",
                Icon::Image => "画像 ",
                Icon::Audio => "♪ ",
                Icon::Video => "► ",
                Icon::Script => "$ ",
                Icon::Document => "≡ ",
                Icon::Search => "🔍 ",
                Icon::Split => "|| ",
                Icon::Single => "[] ",
                Icon::Back => "← ",
                Icon::Forward => "→ ",
                Icon::Burger => "≡ ",
                Icon::Refresh => "↻ ",
                Icon::Monitor => "📈 ",
                Icon::Settings => "⚙ ",
                Icon::Trash => "⌫ ",
                Icon::Home => "⌂ ",
                Icon::Downloads => "⇓ ",
                Icon::Documents => "≡ ",
                Icon::Pictures => "▨ ",
                Icon::Music => "♪ ",
                Icon::Videos => "► ",
                Icon::Rust => "R ",
                Icon::Json => "{ ",
                Icon::Toml => "T ",
                Icon::Markdown => "M ",
                Icon::SelectAll => "∀ ",
                Icon::ToggleHidden => "👁 ",
                Icon::SetWallpaper => "🖼 ",
                Icon::Paste => "📋 ",
                Icon::Cut => "✂ ",
                Icon::Copy => "⎘ ",
                Icon::Duplicate => "⎘ ",
                Icon::Delete => "⚔ ",
                Icon::Rename => "✎ ",
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
}

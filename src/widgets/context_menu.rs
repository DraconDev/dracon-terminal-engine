use crate::utils::FileColumn;
use serde::{Deserialize, Serialize};

/// Actions available in a context menu for files, folders, and other items.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ContextMenuAction {
    /// Open the selected file or directory
    Open,
    /// Open the selected file or directory in a new tab
    OpenNewTab,
    /// Open the file with a specific application
    OpenWith,
    /// Edit the file
    Edit,
    /// Run the file as an executable
    Run,
    /// Run the file in a new terminal window
    RunTerminal,
    /// Extract compressed archive in the current directory
    ExtractHere,
    /// Create a new folder
    NewFolder,
    /// Create a new file
    NewFile,
    /// Cut the selected item(s) to clipboard
    Cut,
    /// Copy the selected item(s) to clipboard
    Copy,
    /// Copy the full path of the item
    CopyPath,
    /// Copy only the name of the item
    CopyName,
    /// Paste from clipboard
    Paste,
    /// Rename the selected item
    Rename,
    /// Duplicate the selected item
    Duplicate,
    /// Compress the selected item(s) into an archive
    Compress,
    /// Delete the selected item(s)
    Delete,
    /// Open a new terminal window
    TerminalWindow,
    /// Open a new terminal tab
    TerminalTab,
    /// Set terminal background color
    SetColor(Option<u8>),
    /// Show properties/details of the item
    Properties,
    /// Show Git status for the item
    GitStatus,
    /// Add the item to favorites
    AddToFavorites,
    /// Remove the item from favorites
    RemoveFromFavorites,
    /// Refresh the current view
    Refresh,
    /// Select all items
    SelectAll,
    /// Toggle visibility of hidden files
    ToggleHidden,
    /// Connect to a remote filesystem
    ConnectRemote,
    /// Delete a remote connection
    DeleteRemote,
    /// Mount a remote filesystem
    Mount,
    /// Unmount a mounted filesystem
    Unmount,
    /// Set terminal wallpaper
    SetWallpaper,
    /// Initialize a Git repository
    GitInit,
    /// Open system monitor
    SystemMonitor,
    /// Initiate drag operation
    Drag,
    /// Sort entries by a specific column
    SortBy(FileColumn),
    /// Visual separator between menu sections
    Separator,
}

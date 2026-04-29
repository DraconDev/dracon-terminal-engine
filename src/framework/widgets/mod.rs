//! Built-in framework widgets.

pub mod breadcrumbs;
pub mod context_menu;
pub mod hud;
pub mod list;
pub mod modal;
pub mod split;
pub mod tabbar;
pub mod table;

pub use breadcrumbs::Breadcrumbs;
pub use context_menu::{ContextAction, ContextMenu};
pub use hud::Hud;
pub use list::List;
pub use modal::{Modal, ModalResult};
pub use split::{Orientation, SplitPane};
pub use tabbar::TabBar;
pub use table::Table;
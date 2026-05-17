//! The terminal application framework.
//!
//! Provides `App` — a one-import entry point that owns the terminal,
//! compositor, input parsing, and event loop.
//!
//! # Example
//!
//! ```no_run
//! use dracon_terminal_engine::framework::prelude::*;
//! use dracon_terminal_engine::framework::widget::Widget;
//! use ratatui::layout::Rect;
//!
//! App::new().unwrap()
//!     .title("My App")
//!     .fps(30)
//!     .run(|ctx| {
//!         let (w, h) = ctx.compositor().size();
//!         let area = Rect::new(0, 0, w, h);
//!         let list = List::new(vec!["Item 1", "Item 2", "Item 3"]);
//!         ctx.add_plane(list.render(area));
//!     });
//! ```

pub mod animation;
pub mod app;
pub mod command;
pub mod ctx;
pub mod dirty_regions;
pub mod dragdrop;
pub mod event_bus;
pub mod event_dispatcher;
pub mod focus;
pub mod hitzone;
pub mod i18n;
pub mod keybindings;
pub mod layout;
#[cfg(feature = "tracing")]
pub mod logging;
pub mod plugin;
pub mod scroll;
pub mod scene_router;
pub mod theme;
pub mod widget;
pub mod widget_container;
pub mod widgets;

/// The prelude module.
pub mod prelude {
    pub use crate::compositor::{Cell, CellPool, Color, Compositor, Plane, PoolConfig, Styles};
    pub use crate::error::DraconError;
    pub use crate::framework::widget::{Commandable, Focusable, InputHandler, Renderable, Themable, Widget, WidgetId, WidgetState};
    #[cfg(feature = "tracing")]
    pub use crate::frame_span;
    #[cfg(feature = "tracing")]
    pub use crate::frame_span_debug;
    pub use crate::framework::{
        animation::{Animation, AnimationManager, Easing},
        app::{App, Ctx, WidgetRef, WidgetRefMut},
        command::{
            AppConfig, AreaConfig, BoundCommand, CommandRunner, LayoutConfig, LoggedLine,
            OutputParser, ParsedOutput, ParserConfig, WidgetConfig,
        },
        dirty_regions::{DirtyRegion, DirtyRegionTracker},
        dragdrop::{DragGhost, DragManager, DragPhase},
        event_bus::{EventBus, Reactive, SubscriptionId},
        focus::FocusManager,
        hitzone::{DragState, HitZone, HitZoneGroup, ScopedZone, ScopedZoneRegistry},
        i18n::{tr, I18n, I18nError},
        keybindings::{KeybindingConfig, KeybindingSet, actions, resolve_keybindings},
        layout::{Constraint, Direction, Layout},
        plugin::{PluginRegistry, WidgetFactory},
        scroll::{ScrollContainer, ScrollState},
        scene_router::{NavigationEvent, Scene, SceneRouter},
        theme::Theme,
        widgets::*,
    };
    pub use crate::input::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
        MouseEventKind,
    };
    #[cfg(feature = "tracing")]
    pub use tracing::instrument;
    pub use crate::Terminal;
    pub use ratatui::layout::Rect;
}

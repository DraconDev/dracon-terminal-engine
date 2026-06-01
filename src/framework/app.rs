//! The main application entry point.
//!
//! [`App`] owns the terminal, compositor, input parser, widget registry,
//! and event loop. Build with the builder pattern, then call [`run`](App::run).
//!
//! ## Widget lifecycle
//!
//! - [`add_widget`](App::add_widget) registers a widget and calls [`on_mount`](crate::framework::widget::Widget::on_mount)
//! - [`remove_widget`](App::remove_widget) calls [`on_unmount`](crate::framework::widget::Widget::on_unmount) and removes it
//! - Focus changes trigger [`on_focus`](crate::framework::widget::Widget::on_focus) / [`on_blur`](crate::framework::widget::Widget::on_blur)
//! - Theme changes propagate via [`set_theme`](App::set_theme) → [`on_theme_change`](crate::framework::widget::Widget::on_theme_change)
//!
//! ## Dirty rendering
//!
//! Widgets that return `false` from [`needs_render`](crate::framework::widget::Widget::needs_render)
//! are skipped during the render pass. Call [`mark_dirty`](Ctx::mark_dirty) on `Ctx` to
//! invalidate a screen region, or call [`mark_dirty`](crate::framework::widget::Widget::mark_dirty) on a
//! widget to force it to re-render next frame.

use crate::backend::tty;
use crate::compositor::{Compositor, Plane};
use crate::core::terminal::RESTORE_SEQ;
use crate::framework::animation::AnimationManager;
use crate::framework::command::{AppConfig, BoundCommand, CommandRunner};
use crate::framework::dirty_regions::DirtyRegionTracker;
use crate::framework::event_bus::EventBus;
use crate::framework::focus::FocusManager;
use crate::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
#[cfg(feature = "debug_events")]
use crate::framework::logging::{log_key_event, log_mouse_event};
use crate::framework::scene_router::SceneRouter;
use crate::framework::theme::Theme;
use crate::framework::widget::{Widget, WidgetId};
use crate::input::event::{Event, KeyEvent};
use crate::input::parser::Parser;
use crate::Terminal;
use ratatui::layout::Rect;
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use std::cell::Cell;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::ops::Deref;
use std::ops::DerefMut;
use std::os::fd::AsFd;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The main application entry point.
///
/// Manages the terminal, compositor, input parsing, and event loop.
/// Build an `App` with the builder pattern, then call [`App::run`] to start it.
///
/// # Example
///
/// ```no_run
/// use dracon_terminal_engine::prelude::*;
/// use ratatui::layout::Rect;
///
/// struct MyApp { theme: Theme }
///
/// impl Widget for MyApp {
///     fn id(&self) -> WidgetId { WidgetId::new(0) }
///     fn area(&self) -> Rect { Rect::new(0, 0, 80, 24) }
///     fn set_area(&mut self, _: Rect) {}
///     fn needs_render(&self) -> bool { true }
///     fn render(&self, area: Rect) -> Plane {
///         let mut p = Plane::new(0, area.width, area.height);
///         p.fill_bg(self.theme.bg);
///         p
///     }
/// }
///
/// fn main() -> std::io::Result<()> {
///     App::new()?
///         .title("My App")
///         .fps(60)
///         .on_tick(|ctx, _tick| {
///             let (w, h) = ctx.compositor().size();
///             let app = MyApp { theme: Theme::default() };
///             ctx.add_plane(app.render(Rect::new(0, 0, w, h)));
///         })
///         .run(|_ctx| {})
/// }
/// ```
/// Tick callback type alias for cleaner signatures.
pub(crate) type TickCallback = Box<dyn FnMut(&mut Ctx, u64) + 'static>;

/// Opaque wrapper around `Ref<'_, Box<dyn Widget>>` that hides the `RefCell`
/// borrow guard from the public API.
///
/// Derefs to `Box<dyn Widget>`, so callers can use widget methods directly.
pub struct WidgetRef<'a> {
    inner: Ref<'a, Box<dyn Widget>>,
}

impl<'a> Deref for WidgetRef<'a> {
    type Target = Box<dyn Widget>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Opaque wrapper around `RefMut<'_, Box<dyn Widget>>` that hides the `RefCell`
/// borrow guard from the public API.
///
/// Derefs to `Box<dyn Widget>`, so callers can use widget methods directly.
pub struct WidgetRefMut<'a> {
    inner: RefMut<'a, Box<dyn Widget>>,
}

impl<'a> Deref for WidgetRefMut<'a> {
    type Target = Box<dyn Widget>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> DerefMut for WidgetRefMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// The main application struct.
///
/// Owns the terminal, compositor, input parser, and all registered widgets.
/// Runs the event loop and manages rendering, focus, and theme propagation.
///
/// ## Borrow Safety
///
/// The `widgets` field uses `RefCell<Vec<Box<dyn Widget>>>` to allow
/// immutable iteration during render (which takes `&self`) and mutable
/// iteration during event dispatch (which also takes `&self` via RefCell).
///
/// **The framework guarantees borrow safety** by never nesting mutable borrows:
/// - `borrow()` (immutable) is used during render, z-order queries, and widget lookups
/// - `borrow_mut()` (mutable) is used during event dispatch, tick callbacks, and add/remove
/// - The event loop processes one phase at a time: input → tick → render
/// - `on_tick` and `on_input` callbacks receive a `Ctx` that does NOT expose widgets directly
///
/// **Panic scenario** (currently impossible but worth documenting): if a user's
/// `on_tick` or `on_input` closure somehow obtained a `WidgetRef`/`WidgetRefMut`
/// and held it across a framework borrow, the RefCell would panic. The wrapper
/// types prevent this by tying borrows to the call scope.
pub struct App {
    terminal: Terminal<io::Stdout>,
    compositor: Compositor,
    parser: Parser,
    title: String,
    fps: u32,
    theme: Theme,
    running: Arc<AtomicBool>,
    frame_count: Arc<AtomicU64>,
    last_frame_time: Instant,
    last_tick_time: Instant,
    tick_interval: Duration,
    tick_count: u64,
    on_tick: RefCell<Option<TickCallback>>,
    widgets: RefCell<Vec<Box<dyn Widget>>>,
    z_order_cache: RefCell<Vec<WidgetId>>,
    z_order_dirty: RefCell<bool>,
    focus_manager: FocusManager,
    dirty_tracker: DirtyRegionTracker,
    animations: AnimationManager,
    next_widget_id: usize,
    commands: RefCell<Vec<BoundCommand>>,
    command_tracking: RefCell<HashMap<WidgetId, (Instant, BoundCommand)>>,
    event_bus: EventBus,
    scene_router: crate::framework::scene_router::SceneRouter,
    keybindings: KeybindingSet,
    /// Input shield: swallows all key/mouse input until this instant.
    /// Used after mode transitions (modal open/close, view switch) to prevent
    /// stale keypresses from leaking into the new state.
    /// Set via `app.shield_input(duration)`.
    input_shield_until: Cell<Option<Instant>>,
}

impl App {
    fn rebuild_z_order_cache(&self) {
        if !*self.z_order_dirty.borrow() {
            return;
        }
        let widgets = self.widgets.borrow();
        let mut ids: Vec<(WidgetId, u16)> = widgets.iter().map(|w| (w.id(), w.z_index())).collect();
        ids.sort_by_key(|(_, z)| *z);
        *self.z_order_cache.borrow_mut() = ids.into_iter().map(|(id, _)| id).collect();
        *self.z_order_dirty.borrow_mut() = false;
    }

    fn invalidate_z_order_cache(&self) {
        *self.z_order_dirty.borrow_mut() = true;
    }

    fn dispatch_key(
        &mut self,
        k: &crate::input::event::KeyEvent,
        running: &std::sync::atomic::AtomicBool,
    ) {
        if self.keybindings.matches(actions::QUIT, k) {
            running.store(false, Ordering::SeqCst);
            return;
        }

        if self.keybindings.matches(actions::BACK, k) {
            let pre_scene_depth = self.scene_router.stack_depth();
            let consumed = self
                .focus_manager
                .focused()
                .and_then(|id| self.widget_mut(id))
                .map(|mut w| w.handle_key(*k))
                .unwrap_or(false);
            if !consumed {
                if self.scene_router.can_go_back() {
                    self.scene_router.pop();
                    self.dirty_tracker.mark_all_dirty();
                } else {
                    running.store(false, Ordering::SeqCst);
                }
            }
            if self.scene_router.stack_depth() != pre_scene_depth {
                self.dirty_tracker.mark_all_dirty();
            }
            return;
        }

        if k.code == crate::input::event::KeyCode::Tab {
            // Let the focused widget try to consume Tab first (e.g., for
            // in-widget navigation like contact switching in ChatApp).
            // If the widget doesn't consume it, use Tab for focus cycling.
            let consumed = self
                .focus_manager
                .focused()
                .and_then(|id| self.widget_mut(id))
                .map(|mut w| w.handle_key(*k))
                .unwrap_or(false);
            if !consumed {
                let old = self.focus_manager.focused();
                if k.modifiers
                    .contains(crate::input::event::KeyModifiers::SHIFT)
                {
                    let _ = self.focus_manager.tab_prev();
                } else {
                    let _ = self.focus_manager.tab_next();
                }
                let new = self.focus_manager.focused();
                if new != old {
                    if let Some(old_id) = old {
                        if let Some(mut w) = self.widget_mut(old_id) {
                            w.on_blur();
                        }
                    }
                    if let Some(new_id) = new {
                        if let Some(mut w) = self.widget_mut(new_id) {
                            w.on_focus();
                        }
                    }
                }
            }
        } else if let Some(focused) = self.focus_manager.focused() {
            let pre_scene_depth = self.scene_router.stack_depth();
            let new_theme = if let Some(mut widget) = self.widget_mut(focused) {
                let _ = widget.handle_key(*k);
                widget.current_theme()
            } else {
                None
            };
            // If a scene was popped, the compositor's final_buffer still has
            // the old scene content. Mark all dirty to force a full clear.
            if self.scene_router.stack_depth() != pre_scene_depth {
                self.dirty_tracker.mark_all_dirty();
            }
            if let Some(theme) = new_theme {
                if theme.name != self.theme.name {
                    App::apply_theme(self, theme);
                }
            }
        }
    }

    fn dispatch_resize(&mut self, new_w: u16, new_h: u16) {
        self.compositor.resize(new_w, new_h);
        self.dirty_tracker.mark_all_dirty();
        let rect = Rect::new(0, 0, new_w, new_h);
        for widget in self.widgets.borrow_mut().iter_mut() {
            widget.set_area(rect);
            widget.mark_dirty();
        }
    }

    fn dispatch_mouse(
        &mut self,
        col: u16,
        row: u16,
        mouse_event: &crate::input::event::MouseEvent,
    ) {
        self.rebuild_z_order_cache();
        let target_id = {
            let widgets = self.widgets.borrow();
            let cache = self.z_order_cache.borrow();
            cache
                .iter()
                .find(|id| {
                    if let Some(w) = widgets.iter().find(|w| w.id() == **id) {
                        let a = w.area();
                        col >= a.x && col < a.x + a.width && row >= a.y && row < a.y + a.height
                    } else {
                        false
                    }
                })
                .copied()
        };
        if let Some(id) = target_id {
            let old = self.focus_manager.focused();
            if old != Some(id) {
                if let Some(old_id) = old {
                    if let Some(mut w) = self.widget_mut(old_id) {
                        w.on_blur();
                    }
                }
                self.focus_manager.set_focus(id);
                if let Some(mut w) = self.widget_mut(id) {
                    w.on_focus();
                }
            }
            let pre_scene_depth = self.scene_router.stack_depth();
            if let Some(mut widget) = self.widget_mut(id) {
                let a = widget.area();
                let local_col = col.saturating_sub(a.x);
                let local_row = row.saturating_sub(a.y);
                let _ = widget.handle_mouse(mouse_event.kind, local_col, local_row);
            }
            if self.scene_router.stack_depth() != pre_scene_depth {
                self.dirty_tracker.mark_all_dirty();
            }
        }
    }

    fn handle_event(&mut self, event: &Event, running: &std::sync::atomic::AtomicBool) {
        // Input shield: swallow all key/mouse input during cooldown period
        if let Some(until) = self.input_shield_until.get() {
            if Instant::now() < until {
                return; // Swallow event during shield
            }
            self.input_shield_until.set(None);
        }

        match event {
            Event::Resize(w, h) => self.dispatch_resize(*w, *h),
            Event::Key(k) => self.dispatch_key(k, running),
            Event::Mouse(mouse_event) => {
                self.dispatch_mouse(mouse_event.column, mouse_event.row, mouse_event)
            }
            Event::Paste(text) => self.dispatch_paste(text),
            _ => {}
        }
    }
    /// Creates a new `App` with a linked terminal.
    /// Returns an error if the terminal cannot be initialized.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(title = "Dracon App"))
    )]
    pub fn new() -> io::Result<Self> {
        Self::new_impl()
    }

    fn new_impl() -> io::Result<Self> {
        let terminal = Terminal::new(io::stdout())?;
        let (w, h) = tty::get_window_size(io::stdout().as_fd()).unwrap_or((80, 24));

        let mut compositor = Compositor::new(w, h);
        compositor.set_clear_color(Theme::default().bg);

        Ok(Self::build(
            terminal,
            compositor,
            String::from("Dracon App"),
            30,
            Vec::new(),
        ))
    }

    fn build(
        terminal: Terminal<io::Stdout>,
        compositor: Compositor,
        title: String,
        fps: u32,
        commands: Vec<BoundCommand>,
    ) -> Self {
        Self {
            terminal,
            compositor,
            parser: Parser::new(),
            title,
            fps: fps.clamp(1, 120),
            theme: Theme::default(),
            running: Arc::new(AtomicBool::new(true)),
            frame_count: Arc::new(AtomicU64::new(0)),
            last_frame_time: Instant::now(),
            last_tick_time: Instant::now(),
            tick_interval: Duration::from_millis(250),
            tick_count: 0,
            on_tick: RefCell::new(None),
            widgets: RefCell::new(Vec::new()),
            z_order_cache: RefCell::new(Vec::new()),
            z_order_dirty: RefCell::new(true),
            focus_manager: FocusManager::new(),
            dirty_tracker: DirtyRegionTracker::new(),
            animations: AnimationManager::new(),
            next_widget_id: 0,
            commands: RefCell::new(commands),
            command_tracking: RefCell::new(HashMap::new()),
            event_bus: EventBus::new(),
            scene_router: SceneRouter::new(),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            input_shield_until: Cell::new(None),
        }
    }

    /// Creates an App from a TOML configuration file.
    ///
    /// This is the primary entry point for command-driven apps.
    /// The TOML file defines the layout, widgets, and their command bindings.
    ///
    /// # Example
    ///
    /// ```ignore
    /// App::from_toml("/home/user/.config/dracon/myapp.toml")?
    ///     .title("My Dashboard")
    ///     .run(|ctx| { /* render */ });
    /// ```
    pub fn from_toml(path: &std::path::Path) -> io::Result<Self> {
        let config = AppConfig::from_toml(path)?;
        let terminal = Terminal::new(io::stdout())?;
        let (w, h) = tty::get_window_size(io::stdout().as_fd()).unwrap_or((80, 24));

        let mut app = Self::build(
            terminal,
            Compositor::new(w, h),
            config.title.clone(),
            config.fps.unwrap_or(30),
            config.commands,
        );

        write!(app.terminal, "\x1b]0;{}\x07", app.title).ok();
        Ok(app)
    }

    /// Adds a command to the global command registry (for AI enumeration).
    pub fn add_command(&mut self, cmd: BoundCommand) {
        self.commands.borrow_mut().push(cmd);
    }

    /// Returns all registered commands across all widgets.
    pub fn available_commands(&self) -> Vec<BoundCommand> {
        let mut cmds = self.commands.borrow().clone();
        for widget in self.widgets.borrow().iter() {
            cmds.extend(widget.commands());
        }
        cmds
    }

    /// Sets the terminal window title (via OSC escape sequence).
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        write!(self.terminal, "\x1b]0;{title}\x07").ok();
        self
    }

    /// Sets the target frames per second (clamped to 1–120).
    pub fn fps(mut self, fps: u32) -> Self {
        self.fps = fps.clamp(1, 120);
        self
    }

    /// Sets the UI theme and propagates it to all registered widgets.
    ///
    /// This calls `on_theme_change()` on every widget, allowing them to
    /// update internal theme-dependent state without requiring manual
    /// configuration of each widget.
    ///
    /// Returns `Self` to support builder-style chaining:
    ///
    /// ```no_run
    /// use dracon_terminal_engine::prelude::*;
    ///
    /// fn main() -> Result<(), dracon_terminal_engine::DraconError> {
    ///     App::new()?
    ///         .title("My App")
    ///         .fps(60)
    ///         .set_theme(Theme::nord())
    ///         .run(|_ctx| {})?;
    ///     Ok(())
    /// }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(theme_name = %theme.name)))]
    pub fn set_theme(mut self, theme: Theme) -> Self {
        self.compositor.set_clear_color(theme.bg);
        self.theme = theme;
        self.dirty_tracker.mark_all_dirty();
        for widget in self.widgets.borrow_mut().iter_mut() {
            widget.on_theme_change(&self.theme);
            widget.mark_dirty();
        }
        self
    }

    /// Applies theme to `&mut self` without consuming. Used internally by framework.
    fn apply_theme(&mut self, theme: Theme) {
        self.compositor.set_clear_color(theme.bg);
        self.theme = theme;
        self.dirty_tracker.mark_all_dirty();
        for widget in self.widgets.borrow_mut().iter_mut() {
            widget.on_theme_change(&self.theme);
            widget.mark_dirty();
        }
    }

    /// Sets the UI theme (builder-style, equivalent to `set_theme`).
    #[deprecated(since = "0.2.0", note = "Use `set_theme()` instead for consistent API")]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.compositor.set_clear_color(theme.bg);
        self.theme = theme;
        self.dirty_tracker.mark_all_dirty();
        for widget in self.widgets.borrow_mut().iter_mut() {
            widget.on_theme_change(&self.theme);
            widget.mark_dirty();
        }
        self
    }

    /// Activate the input shield for the given duration.
    ///
    /// During the shield period, all key and mouse events are silently
    /// swallowed. This prevents stale keypresses from mode transitions
    /// (e.g., closing a modal with Esc) from leaking into the new state.
    ///
    /// # Example
    /// ```ignore
    /// // After closing a modal, shield for 100ms to prevent Esc leak
    /// app.shield_input(Duration::from_millis(100));
    /// ```
    pub fn shield_input(&self, duration: Duration) {
        self.input_shield_until.set(Some(Instant::now() + duration));
    }

    /// Check if the input shield is currently active.
    pub fn is_input_shielded(&self) -> bool {
        self.input_shield_until
            .get()
            .is_some_and(|until| Instant::now() < until)
    }

    /// Dispatches a bracketed-paste text string as synthetic key events to the focused widget.
    /// Converts newlines to Enter, tabs to Tab, and other chars to KeyCode::Char.
    fn dispatch_paste(&mut self, text: &str) {
        if let Some(focused) = self.focus_manager.focused() {
            for ch in text.chars() {
                let code = match ch {
                    '\r' | '\n' => crate::input::event::KeyCode::Enter,
                    '\t' => crate::input::event::KeyCode::Tab,
                    c => crate::input::event::KeyCode::Char(c),
                };
                let key = crate::input::event::KeyEvent {
                    code,
                    modifiers: crate::input::event::KeyModifiers::empty(),
                    kind: crate::input::event::KeyEventKind::Press,
                };
                if let Some(mut widget) = self.widget_mut(focused) {
                    let _ = widget.handle_key(key);
                }
            }
        }
    }

    /// Registers a callback that fires every `tick_interval` milliseconds.
    /// The callback receives the context and the tick count.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use dracon_terminal_engine::prelude::*;
    /// use ratatui::layout::Rect;
    ///
    /// struct Counter { count: u64 }
    ///
    /// impl Widget for Counter {
    ///     fn id(&self) -> WidgetId { WidgetId::new(0) }
    ///     fn area(&self) -> Rect { Rect::new(0, 0, 80, 24) }
    ///     fn set_area(&mut self, _: Rect) {}
    ///     fn needs_render(&self) -> bool { true }
    ///     fn render(&self, area: Rect) -> Plane {
    ///         let mut p = Plane::new(0, area.width, area.height);
    ///         p.fill_bg(Theme::default().bg);
    ///         p.put_str(0, 0, &format!("Count: {}", self.count));
    ///         p
    ///     }
    /// }
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let app = std::rc::Rc::new(std::cell::RefCell::new(Counter { count: 0 }));
    ///     let app_clone = app.clone();
    ///     App::new()?
    ///         .on_tick(move |ctx, tick| {
    ///             app_clone.borrow_mut().count = tick;
    ///             let (w, h) = ctx.compositor().size();
    ///             ctx.add_plane(app_clone.borrow().render(Rect::new(0, 0, w, h)));
    ///         })
    ///         .run(|_| {});
    ///     Ok(())
    /// }
    /// ```
    pub fn on_tick<F>(self, f: F) -> Self
    where
        F: FnMut(&mut Ctx, u64) + 'static,
    {
        *self.on_tick.borrow_mut() = Some(Box::new(f));
        self
    }

    /// Sets the tick interval in milliseconds (default: 250ms).
    pub fn tick_interval(mut self, ms: u64) -> Self {
        self.tick_interval = Duration::from_millis(ms);
        self
    }

    /// Registers a keyboard input handler for the on_tick + add_plane pattern.
    ///
    /// Creates a hidden full-screen widget that receives keyboard focus and
    /// delegates KeyEvent to the given closure. This eliminates the need for
    /// manual `InputRouter` boilerplate when using `on_tick` + `ctx.add_plane()`.
    ///
    /// The closure should return `true` if the event was handled.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use dracon_terminal_engine::prelude::*;
    /// use ratatui::layout::Rect;
    ///
    /// struct Counter { count: u64 }
    ///
    /// impl Widget for Counter {
    ///     fn id(&self) -> WidgetId { WidgetId::new(0) }
    ///     fn area(&self) -> Rect { Rect::new(0, 0, 80, 24) }
    ///     fn set_area(&mut self, _: Rect) {}
    ///     fn needs_render(&self) -> bool { true }
    ///     fn render(&self, area: Rect) -> Plane {
    ///         let mut p = Plane::new(0, area.width, area.height);
    ///         p.fill_bg(Theme::default().bg);
    ///         p.put_str(0, 0, &format!("Count: {}", self.count));
    ///         p
    ///     }
    /// }
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let app = std::rc::Rc::new(std::cell::RefCell::new(Counter { count: 0 }));
    ///     let app_for_input = app.clone();
    ///     let app_for_tick = app.clone();
    ///     App::new()?
    ///         .on_tick(move |ctx, _tick| {
    ///             let (w, h) = ctx.compositor().size();
    ///             ctx.add_plane(app_for_tick.borrow().render(Rect::new(0, 0, w, h)));
    ///         })
    ///         .on_input(move |key| {
    ///             app_for_input.borrow_mut().count += 1;
    ///             true  // event handled
    ///         })
    ///         .run(|_| {});
    ///     Ok(())
    /// }
    /// ```
    pub fn on_input<F>(mut self, handler: F) -> Self
    where
        F: FnMut(KeyEvent) -> bool + 'static,
    {
        let (w, h) = tty::get_window_size(io::stdout().as_fd()).unwrap_or((80, 24));
        let input_widget = InputHandler {
            handler: Box::new(handler),
            id: WidgetId::new(self.next_widget_id),
            area: Rect::new(0, 0, w, h),
            theme: Some(self.theme.clone()),
        };
        self.add_widget(Box::new(input_widget), Rect::new(0, 0, w, h));
        self
    }

    /// Adds a widget to the application with the given area.
    /// Returns the assigned `WidgetId`.
    pub fn add_widget(&mut self, mut widget: Box<dyn Widget>, area: Rect) -> WidgetId {
        let id = WidgetId(self.next_widget_id);
        widget.set_id(id);
        widget.set_area(area);
        widget.on_mount();
        widget.on_theme_change(&self.theme);
        let focusable = widget.focusable();
        let cmds = widget.commands();
        self.widgets.borrow_mut().push(widget);
        self.compositor
            .set_widget_count(self.widgets.borrow().len());
        self.focus_manager.register(id, focusable);
        self.invalidate_z_order_cache();

        // Auto-focus first widget if nothing is focused yet
        if self.focus_manager.focused().is_none() && focusable {
            self.focus_manager.set_focus(id);
            if let Some(mut w) = self.widget_mut(id) {
                w.on_focus();
            }
        }

        self.next_widget_id += 1;

        for cmd in cmds {
            if cmd.refresh_seconds.is_some() {
                self.command_tracking
                    .borrow_mut()
                    .insert(id, (Instant::now(), cmd));
            }
        }

        id
    }

    /// Removes a widget by its ID.
    pub fn remove_widget(&mut self, id: WidgetId) {
        if let Some(w) = self.widgets.borrow_mut().iter_mut().find(|w| w.id() == id) {
            w.on_unmount();
        }
        self.widgets.borrow_mut().retain(|w| w.id() != id);
        self.compositor
            .set_widget_count(self.widgets.borrow().len());
        self.focus_manager.unregister(id);
        self.command_tracking.borrow_mut().remove(&id);
        self.invalidate_z_order_cache();
    }

    /// Returns an immutable reference to a widget by ID.
    pub fn widget(&self, id: WidgetId) -> Option<WidgetRef<'_>> {
        let widgets = self.widgets.borrow();
        let idx = widgets.iter().position(|w| w.id() == id)?;
        Some(WidgetRef {
            inner: Ref::map(widgets, |w| &w[idx]),
        })
    }

    /// Returns a mutable reference to a widget by ID.
    pub fn widget_mut(&mut self, id: WidgetId) -> Option<WidgetRefMut<'_>> {
        let widgets = self.widgets.borrow_mut();
        let idx = widgets.iter().position(|w| w.id() == id)?;
        Some(WidgetRefMut {
            inner: RefMut::map(widgets, |w| &mut w[idx]),
        })
    }

    /// Returns the number of registered widgets.
    pub fn widget_count(&self) -> usize {
        self.widgets.borrow().len()
    }

    /// Returns the number of planes in the compositor.
    pub fn plane_count(&self) -> usize {
        self.compositor.planes.len()
    }

    /// Returns the last frame duration in milliseconds.
    pub fn frame_time_ms(&self) -> f64 {
        self.compositor.last_frame_duration_ms()
    }

    fn poll_and_dispatch_input(&mut self, stdin: &mut io::Stdin) {
        let running = self.running.clone();
        const INPUT_BUF_SIZE: usize = 1024;
        match tty::poll_input(stdin.as_fd(), 1) {
            Ok(true) => {
                let mut chunk_buf = [0u8; INPUT_BUF_SIZE];
                if let Ok(n) = stdin.read(&mut chunk_buf) {
                    if n == 0 {
                        // EOF — stdin closed (e.g., pipe broken), trigger graceful shutdown
                        self.running.store(false, Ordering::SeqCst);
                        return;
                    }
                    for byte in chunk_buf.iter().take(n) {
                        if let Some(event) = self.parser.advance(*byte) {
                            #[cfg(feature = "debug_events")]
                            match &event {
                                Event::Key(k) => log_key_event(k),
                                Event::Mouse(m) => log_mouse_event(m),
                                _ => {}
                            }
                            self.handle_event(&event, &running);
                        }
                    }
                }
                for _ in 0..64 {
                    match tty::poll_input(stdin.as_fd(), 0) {
                        Ok(true) => {
                            let mut drain_buf = [0u8; INPUT_BUF_SIZE];
                            if let Ok(dn) = stdin.read(&mut drain_buf) {
                                if dn == 0 {
                                    break;
                                }
                                for byte in drain_buf.iter().take(dn) {
                                    if let Some(event) = self.parser.advance(*byte) {
                                        self.handle_event(&event, &running);
                                    }
                                }
                            }
                        }
                        _ => break,
                    }
                }
            }
            Ok(false) => {
                if let Some(evt) = self.parser.check_timeout() {
                    #[cfg(feature = "debug_events")]
                    match &evt {
                        Event::Key(k) => log_key_event(k),
                        Event::Mouse(m) => log_mouse_event(m),
                        _ => {}
                    }
                    self.handle_event(&evt, &running);
                }
            }
            Err(_) => {}
        }
    }

    fn focused_cursor_position(&self) -> Option<(u16, u16)> {
        let focused_id = self.focus_manager.focused()?;
        let widgets = self.widgets.borrow();
        widgets
            .iter()
            .find(|w| w.id() == focused_id)
            .and_then(|w| w.cursor_position())
    }

    fn render_dirty_widgets(&mut self) {
        #[cfg(feature = "tracing")]
        let _widget_span = tracing::debug_span!("widget_dispatch").entered();
        self.rebuild_z_order_cache();
        let cache = self.z_order_cache.borrow().clone();
        let mut widgets = self.widgets.borrow_mut();
        for id in cache {
            let w = match widgets.iter_mut().find(|w| w.id() == id) {
                Some(w) => w,
                None => continue,
            };
            if !w.needs_render() {
                continue;
            }
            let area = w.area();
            #[cfg(feature = "tracing")]
            let _render_span = tracing::debug_span!(
                "widget_render",
                widget_id = w.id().0,
                width = area.width,
                height = area.height
            )
            .entered();
            let plane = w.render(area);
            w.clear_dirty();
            self.compositor.add_plane(plane);
        }
    }

    fn run_tick_callback(&mut self, frame_count: &Arc<AtomicU64>) {
        if self.last_tick_time.elapsed() < self.tick_interval {
            return;
        }
        if let Some(ref mut tick_fn) = *self.on_tick.borrow_mut() {
            let prev_theme_name = self.theme.name.clone();
            tick_fn(
                &mut Ctx {
                    compositor: &mut self.compositor,
                    theme: &mut self.theme,
                    frame_count: frame_count.load(Ordering::SeqCst),
                    last_frame: &self.last_frame_time,
                    terminal: &mut self.terminal,
                    focus_manager: &mut self.focus_manager,
                    animations: &mut self.animations,
                    dirty_tracker: &mut self.dirty_tracker,
                    commands: &self.commands,
                    running: &self.running,
                    event_bus: &self.event_bus,
                    scene_router: &mut self.scene_router,
                },
                self.tick_count,
            );
            if self.theme.name != prev_theme_name {
                self.compositor.set_clear_color(self.theme.bg);
                self.dirty_tracker.mark_all_dirty();
                for widget in self.widgets.borrow_mut().iter_mut() {
                    widget.on_theme_change(&self.theme);
                    widget.mark_dirty();
                }
            }
            self.tick_count += 1;
            self.last_tick_time = Instant::now();
        }
    }

    fn run_periodic_commands(&mut self) {
        let now = Instant::now();
        let mut to_reschedule: Vec<(WidgetId, BoundCommand)> = Vec::new();
        let mut expired: Vec<WidgetId> = Vec::new();
        {
            let tracked = self.command_tracking.borrow();
            for (&wid, (last_run, cmd)) in tracked.iter() {
                let interval = Duration::from_secs(cmd.refresh_seconds.unwrap_or(0));
                if interval.is_zero() || now.duration_since(*last_run) < interval {
                    continue;
                }
                expired.push(wid);
            }
        }
        for wid in expired {
            let cmd = match self.command_tracking.borrow().get(&wid) {
                Some((_, c)) => c.clone(),
                None => continue,
            };
            if let Some(mut w) = self.widget_mut(wid) {
                let runner = CommandRunner::new(&cmd.command);
                let (stdout, stderr, exit_code) = runner.run_sync();
                let output = cmd.parse_output(&stdout, &stderr, exit_code);
                w.apply_command_output(&output);
                w.mark_dirty();
                to_reschedule.push((wid, cmd));
            }
        }
        for (wid, cmd) in to_reschedule {
            self.command_tracking
                .borrow_mut()
                .insert(wid, (Instant::now(), cmd));
        }
    }

    /// Starts the application event loop.
    ///
    /// Reads input, fires tick callbacks, and invokes the render callback
    /// each frame until the user presses Ctrl+C or [`App::stop`] is called.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(title = %self.title, fps = %self.fps)))]
    pub fn run<F>(mut self, mut f: F) -> io::Result<()>
    where
        F: FnMut(&mut Ctx),
    {
        let running = self.running.clone();
        let frame_count = self.frame_count.clone();

        let title = self.title.clone();
        write!(self.terminal, "\x1b]0;{title}\x07").ok();

        let previous_hook = Arc::new(Mutex::new(std::panic::take_hook()));
        let prev_hook_clone = previous_hook.clone();
        std::panic::set_hook(Box::new(move |info| {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            let _ = handle.write_all(RESTORE_SEQ.as_bytes());
            let _ = handle.flush();
            if let Ok(mut hook) = prev_hook_clone.lock() {
                let original = std::mem::replace(&mut *hook, Box::new(|_| {}));
                original(info);
            }
        }));

        let running_for_signal = running.clone();
        // SAFETY: signal_hook::low_level::register() is unsafe because it installs
        // a signal handler that must be async-signal-safe. The closures only call
        // AtomicBool::store(SeqCst), which is async-signal-safe (no allocation,
        // no locks, no syscalls). The `.ok()` discards registration errors silently
        // (e.g., if the signal is already handled), which is acceptable — the app
        // will still terminate on the next Ctrl-C via the default handler.
        unsafe {
            let running_int = running_for_signal.clone();
            signal_hook::low_level::register(SIGINT, move || {
                running_int.store(false, Ordering::SeqCst);
            })
            .ok();
            let running_term = running_for_signal.clone();
            signal_hook::low_level::register(SIGTERM, move || {
                running_term.store(false, Ordering::SeqCst);
            })
            .ok();
        }

        let mut stdin = io::stdin();
        let frame_duration = Duration::from_secs_f64(1.0 / self.fps as f64);

        self.last_frame_time = Instant::now();

        let (term_w, term_h) = self.compositor.size();
        let full_rect = Rect::new(0, 0, term_w, term_h);
        for widget in self.widgets.borrow_mut().iter_mut() {
            widget.set_area(full_rect);
            widget.mark_dirty();
        }

        while running.load(Ordering::SeqCst) {
            let frame_start = Instant::now();
            #[cfg(feature = "tracing")]
            let _frame_span = tracing::debug_span!("frame").entered();

            #[cfg(feature = "tracing")]
            let _input_span = tracing::debug_span!("input_poll").entered();

            self.poll_and_dispatch_input(&mut stdin);

            #[cfg(feature = "tracing")]
            drop(_input_span);

            self.render_dirty_widgets();

            self.run_tick_callback(&frame_count);

            self.run_periodic_commands();

            f(&mut Ctx {
                compositor: &mut self.compositor,
                theme: &mut self.theme,
                frame_count: frame_count.load(Ordering::SeqCst),
                last_frame: &self.last_frame_time,
                terminal: &mut self.terminal,
                focus_manager: &mut self.focus_manager,
                animations: &mut self.animations,
                dirty_tracker: &mut self.dirty_tracker,
                commands: &self.commands,
                running: &self.running,
                event_bus: &self.event_bus,
                scene_router: &mut self.scene_router,
            });

            if !self.compositor.planes.is_empty() {
                self.compositor.set_dirty_regions(&self.dirty_tracker);
                self.compositor.render(&mut self.terminal)?;
            }

            if let Some((col, row)) = self.focused_cursor_position() {
                let _ = self.terminal.set_cursor(col, row);
                let _ = self.terminal.show_cursor();
            } else {
                let _ = self.terminal.hide_cursor();
            }

            self.animations.tick();

            frame_count.fetch_add(1, Ordering::SeqCst);
            let frame_elapsed = self.last_frame_time.elapsed().as_secs_f64() * 1000.0;
            self.compositor.set_last_frame_duration(frame_elapsed);
            self.last_frame_time = Instant::now();

            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }

        let _our_hook = std::panic::take_hook();
        if let Ok(mut guard) = previous_hook.lock() {
            let original = std::mem::replace(&mut *guard, Box::new(|_| {}));
            std::panic::set_hook(original);
        }

        if let Ok(path) = std::env::var("DTRON_THEME_FILE") {
            if let Err(e) = std::fs::write(&path, self.theme.name.as_bytes()) {
                eprintln!("warning: failed to write theme to {}: {}", path, e);
            }
        }

        Ok(())
    }

    /// Stops the event loop on the next iteration.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Default for App {
    /// Returns a default App.
    ///
    /// **Note**: This constructor calls `expect()` and panics if the terminal
    /// cannot be initialized (e.g., not a TTY). Prefer [`App::from_defaults()`]
    /// which returns `io::Result<Self>` and allows graceful error handling.
    fn default() -> Self {
        Self::new().expect("failed to initialize terminal")
    }
}

impl App {
    /// Creates a new App with default settings.
    ///
    /// This is a fallible constructor that returns `Result` instead of panicking.
    /// Prefer this over `App::default()` which will panic if the terminal
    /// cannot be initialized.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if the terminal cannot be initialized (e.g., not a TTY).
    pub fn from_defaults() -> io::Result<Self> {
        Self::new()
    }
}

/// Application context, passed to every render and tick callback.
///
/// Provides access to the compositor, theme, animation manager, focus manager,
/// and dirty region tracker. Use it to add planes, manage focus, and mark
/// screen regions as dirty for the next render pass.
///
/// ## Example
///
/// ```ignore
/// app.run(|ctx| {
///     ctx.mark_dirty(0, 0, 80, 24); // mark entire screen dirty
///     let plane = my_widget.render(ctx.compositor().size().into());
///     ctx.add_plane(plane);
/// });
/// ```
pub use crate::framework::ctx::Ctx;

/// Hidden widget that routes keyboard events to a closure.
/// Created by [`App::on_input`] to enable input for the `on_tick` + `add_plane` pattern.
struct InputHandler {
    handler: Box<dyn FnMut(KeyEvent) -> bool>,
    id: WidgetId,
    area: Rect,
    theme: Option<Theme>,
}

impl Widget for InputHandler {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn area(&self) -> Rect {
        self.area
    }
    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        false
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }
    fn render(&self, _area: Rect) -> Plane {
        Plane::new(0, 0, 0)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        (self.handler)(key)
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = Some(theme.clone());
    }

    fn current_theme(&self) -> Option<Theme> {
        self.theme.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static FAKE_RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
    use crate::framework::command::{
        AppConfig, AreaConfig, LayoutConfig, ParserConfig, WidgetConfig,
    };

    fn make_test_terminal() -> io::Result<crate::Terminal<io::Stdout>> {
        crate::Terminal::new(io::stdout())
    }

    macro_rules! with_ctx {
        (mut $ctx:ident, $body:expr) => {{
            let mut compositor = Compositor::new(80, 24);
            let mut focus_manager = FocusManager::new();
            let mut dirty_tracker = DirtyRegionTracker::new();
            let mut animations = AnimationManager::new();
            let mut theme = Theme::default();
            let last_frame = Instant::now();
            let commands = RefCell::new(Vec::new());
            let event_bus = EventBus::new();
            let mut scene_router = SceneRouter::new();
            let mut terminal = make_test_terminal().unwrap();
            let mut $ctx = Ctx {
                compositor: &mut compositor,
                theme: &mut theme,
                frame_count: 0,
                last_frame: &last_frame,
                running: &FAKE_RUNNING,
                terminal: &mut terminal,
                focus_manager: &mut focus_manager,
                animations: &mut animations,
                dirty_tracker: &mut dirty_tracker,
                commands: &commands,
                event_bus: &event_bus,
                scene_router: &mut scene_router,
            };
            $body
        }};
        ($ctx:ident, $body:expr) => {{
            let mut compositor = Compositor::new(80, 24);
            let mut focus_manager = FocusManager::new();
            let mut dirty_tracker = DirtyRegionTracker::new();
            let mut animations = AnimationManager::new();
            let mut theme = Theme::default();
            let last_frame = Instant::now();
            let commands = RefCell::new(Vec::new());
            let event_bus = EventBus::new();
            let mut scene_router = SceneRouter::new();
            let mut terminal = make_test_terminal().unwrap();
            let $ctx = Ctx {
                compositor: &mut compositor,
                theme: &mut theme,
                frame_count: 0,
                last_frame: &last_frame,
                running: &FAKE_RUNNING,
                terminal: &mut terminal,
                focus_manager: &mut focus_manager,
                animations: &mut animations,
                dirty_tracker: &mut dirty_tracker,
                commands: &commands,
                event_bus: &event_bus,
                scene_router: &mut scene_router,
            };
            $body
        }};
    }

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert!(app.is_ok());
        let app = app.unwrap();
        assert_eq!(app.widget_count(), 0);
        assert_eq!(app.title, "Dracon App");
        assert_eq!(app.fps, 30);
    }

    #[test]
    fn test_app_default() {
        let app = App::default();
        assert_eq!(app.widget_count(), 0);
        assert_eq!(app.title, "Dracon App");
    }

    #[test]
    fn test_app_title_fps_builder() {
        let app = App::new().unwrap().title("My Dashboard").fps(60);
        assert_eq!(app.title, "My Dashboard");
        assert_eq!(app.fps, 60);
    }

    #[test]
    fn test_app_fps_clamped() {
        let app = App::new().unwrap().fps(0);
        assert_eq!(app.fps, 1);
        let app = App::new().unwrap().fps(200);
        assert_eq!(app.fps, 120);
    }

    #[test]
    fn test_app_add_widget() {
        use crate::framework::widgets::Label;
        let mut app = App::new().unwrap();
        let label = Label::new("test");
        let id = app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));
        assert_eq!(app.widget_count(), 1);
        assert!(app.widget(id).is_some());
    }

    #[test]
    fn test_app_widget_mut() {
        use crate::framework::widgets::Label;
        let mut app = App::new().unwrap();
        let label = Label::new("test");
        let id = app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));
        let w = app.widget_mut(id);
        assert!(w.is_some());
    }

    #[test]
    fn test_app_remove_widget() {
        use crate::framework::widgets::Label;
        let mut app = App::new().unwrap();
        let label = Label::new("test");
        let id = app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));
        assert_eq!(app.widget_count(), 1);
        app.remove_widget(id);
        assert_eq!(app.widget_count(), 0);
    }

    #[test]
    fn test_app_widget_not_found() {
        let mut app = App::new().unwrap();
        let id = WidgetId(99999);
        assert!(app.widget(id).is_none());
        assert!(app.widget_mut(id).is_none());
    }

    #[test]
    fn test_app_add_command() {
        let mut app = App::new().unwrap();
        let cmd = BoundCommand::new("ls -la");
        app.add_command(cmd.clone());
        let cmds = app.available_commands();
        assert!(!cmds.is_empty());
    }

    #[test]
    fn test_app_available_commands_includes_widget_commands() {
        use crate::framework::widgets::Label;
        let mut app = App::new().unwrap();
        let label = Label::new("test");
        let _id = app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));
        let cmds = app.available_commands();
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_app_set_theme() {
        use crate::framework::widgets::Label;
        let mut app = App::new().unwrap();
        let label = Label::new("test");
        app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));
        let _app = app.set_theme(Theme::cyberpunk());
    }

    #[test]
    fn test_app_tick_interval() {
        let app = App::new().unwrap().tick_interval(500);
        assert_eq!(app.tick_interval, Duration::from_millis(500));
    }

    #[test]
    fn test_app_stop() {
        let app = App::new().unwrap();
        app.stop();
    }

    #[test]
    fn test_ctx_available_commands_empty() {
        with_ctx!(ctx, {
            let cmds = ctx.available_commands();
            assert!(cmds.is_empty());
        });
    }

    #[test]
    fn test_ctx_add_plane() {
        with_ctx!(mut ctx, {
            let plane = Plane::new(0, 20, 10);
            ctx.add_plane(plane);
            assert_eq!(ctx.compositor().planes.len(), 1);
        });
    }

    #[test]
    fn test_ctx_mark_dirty() {
        with_ctx!(mut ctx, {
            ctx.mark_dirty(0, 0, 80, 24);
        });
    }

    #[test]
    fn test_ctx_set_focus() {
        with_ctx!(mut ctx, {
            let id = WidgetId(42);
            ctx.set_focus(id);
            let _ = ctx.focused();
        });
    }

    #[test]
    fn test_ctx_theme_access() {
        with_ctx!(ctx, {
            assert!(
                ctx.theme().name == Arc::from("default") || ctx.theme().name == Arc::from("dark")
            );
        });
    }

    #[test]
    fn test_ctx_mark_all_dirty() {
        with_ctx!(mut ctx, {
            ctx.mark_all_dirty();
            assert!(ctx.dirty_tracker.needs_full_refresh());
        });
    }

    #[test]
    fn test_ctx_clear() {
        with_ctx!(mut ctx, {
            ctx.clear();
            assert!(ctx.dirty_tracker.needs_full_refresh());
        });
    }

    #[test]
    fn test_ctx_compositor_access() {
        with_ctx!(ctx, {
            let (w, h) = ctx.compositor().size();
            assert_eq!(w, 80);
            assert_eq!(h, 24);
        });
    }

    #[test]
    fn test_ctx_fps_zero_elapsed() {
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let mut theme = Theme::default();
        let last_frame = std::time::Instant::now();
        let commands = RefCell::new(Vec::new());
        let event_bus = EventBus::new();
        let mut scene_router = SceneRouter::new();
        let mut terminal = make_test_terminal().unwrap();

        let ctx = Ctx {
            compositor: &mut compositor,
            theme: &mut theme,
            frame_count: 100,
            last_frame: &last_frame,
            running: &FAKE_RUNNING,
            terminal: &mut terminal,
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
            event_bus: &event_bus,
            scene_router: &mut scene_router,
        };

        let _fps = ctx.fps();
    }

    #[test]
    fn test_ctx_split_h() {
        with_ctx!(mut ctx, {
            ctx.split_h(|left, right| {
                let a = left.area();
                let b = right.area();
                assert!(a.width > 0);
                assert!(b.width > 0);
            });
        });
    }

    #[test]
    fn test_ctx_split_v() {
        with_ctx!(mut ctx, {
            ctx.split_v(|_top, _bottom| {});
        });
    }

    #[test]
    fn test_ctx_layout() {
        with_ctx!(ctx, {
            use crate::framework::layout::Constraint;
            let rects = ctx.layout(vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
            assert_eq!(rects.len(), 2);
        });
    }

    #[test]
    fn test_ctx_run_command() {
        with_ctx!(ctx, {
            let (stdout, _stderr, code) = ctx.run_command("echo test_run_command");
            assert!(stdout.contains("test_run_command"));
            assert_eq!(code, 0);
        });
    }

    #[test]
    fn test_ctx_available_commands() {
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let mut theme = Theme::default();
        let last_frame = Instant::now();
        let commands = RefCell::new(vec![
            BoundCommand::new("test cmd 1"),
            BoundCommand::new("test cmd 2"),
        ]);
        let event_bus = EventBus::new();
        let mut scene_router = SceneRouter::new();
        let mut terminal = make_test_terminal().unwrap();

        let ctx = Ctx {
            compositor: &mut compositor,
            theme: &mut theme,
            frame_count: 0,
            last_frame: &last_frame,
            running: &FAKE_RUNNING,
            terminal: &mut terminal,
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
            event_bus: &event_bus,
            scene_router: &mut scene_router,
        };

        let cmds = ctx.available_commands();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].command, "test cmd 1");
        assert_eq!(cmds[1].command, "test cmd 2");
    }

    #[test]
    fn test_app_config_from_toml_str() {
        let toml = r#"
            title = "Test App"
            fps = 60
        "#;
        let config = AppConfig::from_toml_str(toml).unwrap();
        assert_eq!(config.title, "Test App");
        assert_eq!(config.fps, Some(60));
    }

    #[test]
    fn test_app_config_from_toml_str_widgets() {
        let toml = r#"
            title = "Widget Test"
            [[widget]]
            id = 1
            type = "Label"
            label = "Test Label"
        "#;
        let config = AppConfig::from_toml_str(toml).unwrap();
        assert_eq!(config.title, "Widget Test");
    }

    #[test]
    fn test_widget_config_all_fields() {
        let toml = r#"
            id = 5
            type = "CustomWidget"
            bind = "mycommand --arg"
            refresh_seconds = 15
            confirm = "Confirm?"
            label = "My Label"
            description = "Widget description"
        "#;
        let config: WidgetConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.id, Some(5));
        assert_eq!(config.widget_type, Some("CustomWidget".to_string()));
        assert_eq!(config.bind, Some("mycommand --arg".to_string()));
        assert_eq!(config.refresh_seconds, Some(15));
        assert_eq!(config.confirm, Some("Confirm?".to_string()));
        assert_eq!(config.label, Some("My Label".to_string()));
        assert_eq!(config.description, Some("Widget description".to_string()));
    }

    #[test]
    fn test_widget_config_type_alias() {
        let toml = r#"
            type = "StatusBadge"
        "#;
        let config: WidgetConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.widget_type, Some("StatusBadge".to_string()));
    }

    #[test]
    fn test_widget_config_default() {
        let toml = "";
        let config: WidgetConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.id, None);
        assert_eq!(config.widget_type, None);
        assert!(config.bind.is_none());
    }

    #[test]
    fn test_area_config() {
        let toml = r#"
            x = 10
            y = 20
            width = 80
            height = 24
        "#;
        let config: AreaConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.x, 10);
        assert_eq!(config.y, 20);
        assert_eq!(config.width, 80);
        assert_eq!(config.height, 24);
    }

    #[test]
    fn test_parser_config() {
        let toml = r#"
            type = "json_key"
            key = "status"
        "#;
        let config: ParserConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.parser_type, "json_key");
        assert_eq!(config.key, Some("status".to_string()));
    }

    #[test]
    fn test_parser_config_json_path() {
        let toml = r#"
            type = "json_path"
            path = "data.result"
        "#;
        let config: ParserConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.parser_type, "json_path");
        assert_eq!(config.path, Some("data.result".to_string()));
    }

    #[test]
    fn test_parser_config_regex() {
        let toml = r#"
            type = "regex"
            pattern = "CPU: (\\d+)"
            group = 1
        "#;
        let config: ParserConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.parser_type, "regex");
        assert_eq!(config.pattern, Some("CPU: (\\d+)".to_string()));
        assert_eq!(config.group, Some(1));
    }

    #[test]
    fn test_layout_config() {
        let toml = r#"
            header_height = 3
            sidebar_width = 25
            footer_height = 2
        "#;
        let config: LayoutConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.header_height, Some(3));
        assert_eq!(config.sidebar_width, Some(25));
        assert_eq!(config.footer_height, Some(2));
    }

    #[test]
    fn test_app_config_layout_only() {
        let toml = r#"
            title = "Layout Test"
            [layout]
            header_height = 5
        "#;
        let config = AppConfig::from_toml_str(toml).unwrap();
        assert_eq!(config.title, "Layout Test");
        assert!(config.layout.is_some());
    }

    #[test]
    fn test_app_config_widgets_multiple() {
        let toml = r#"
            title = "Multi Widget"

            [[widget]]
            id = 1
            type = "Label"
            label = "First"

            [[widget]]
            id = 2
            type = "Button"
            label = "Second"
        "#;
        let config = AppConfig::from_toml_str(toml).unwrap();
        assert_eq!(config.title, "Multi Widget");
    }

    #[test]
    fn test_widget_config_options() {
        let toml = r#"
            type = "Custom"
            [options]
            width = 100
            height = 50
            enabled = true
        "#;
        let config: WidgetConfig = toml::from_str(toml).unwrap();
        assert!(config.options.contains_key("width"));
        assert!(config.options.contains_key("height"));
    }

    #[test]
    fn test_app_config_commands() {
        let toml = r#"
            title = "Command Test"

            [[commands]]
            command = "dracon-system status --json"
            label = "system status"
            description = "Get system status"
            refresh_seconds = 5

            [[commands]]
            command = "dracon-sync repos --json"
            label = "sync repos"
            description = "Get repo status"
            refresh_seconds = 10
        "#;
        let config = AppConfig::from_toml_str(toml).unwrap();
        assert_eq!(config.title, "Command Test");
        assert_eq!(config.commands.len(), 2);
        assert_eq!(config.commands[0].command, "dracon-system status --json");
        assert_eq!(config.commands[0].label, "system status");
        assert_eq!(config.commands[0].refresh_seconds, Some(5));
        assert_eq!(config.commands[1].command, "dracon-sync repos --json");
        assert_eq!(config.commands[1].refresh_seconds, Some(10));
    }

    #[test]
    fn test_app_command_tracking_on_add_widget() {
        use crate::framework::command::BoundCommand;
        let mut app = App::new().unwrap();
        let _cmd = BoundCommand::new("echo test").refresh(5);
        let label = crate::framework::widgets::Label::new("test");
        let _id = app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));
        let tracking = app.command_tracking.borrow();
        assert!(tracking.is_empty());
    }

    #[test]
    fn test_app_command_tracking_removed_on_widget_remove() {
        use crate::framework::command::BoundCommand;
        let mut app = App::new().unwrap();
        let _cmd = BoundCommand::new("echo test").refresh(5);
        let label = crate::framework::widgets::Label::new("test");
        let id = app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));
        app.remove_widget(id);
        assert!(app.command_tracking.borrow().is_empty());
    }

    #[test]
    fn test_focused_cursor_position_none_when_no_focus() {
        let app = App::new().unwrap();
        assert!(app.focused_cursor_position().is_none());
    }

    #[test]
    fn test_focused_cursor_position_none_for_non_cursor_widget() {
        use crate::framework::widgets::Label;
        let mut app = App::new().unwrap();
        let label = Label::new("test");
        let id = app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));
        app.focus_manager.set_focus(id);
        assert!(app.focused_cursor_position().is_none());
    }

    #[test]
    fn test_focused_cursor_position_some_for_cursor_widget() {
        use crate::framework::widgets::SearchInput;
        let mut app = App::new().unwrap();
        let search = SearchInput::new(WidgetId::next());
        let id = app.add_widget(Box::new(search), Rect::new(0, 0, 20, 1));
        app.focus_manager.set_focus(id);
        assert!(app.focused_cursor_position().is_some());
    }
}

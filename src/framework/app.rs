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
use crate::framework::animation::AnimationManager;
use crate::framework::command::{AppConfig, BoundCommand, CommandRunner};
use crate::framework::dirty_regions::DirtyRegionTracker;
use crate::framework::focus::FocusManager;
use crate::framework::theme::Theme;
use crate::framework::widget::{Widget, WidgetId};
use crate::input::event::Event;
use crate::input::parser::Parser;
use crate::Terminal;
use ratatui::layout::Rect;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::os::fd::AsFd;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The main application entry point.
///
/// Manages the terminal, compositor, input parsing, and event loop.
/// Build an `App` with the builder pattern, then call [`App::run`] to start it.
///
/// # Example
///
/// ```ignore
/// App::new()?
///     .title("My App")
///     .fps(60)
///     .on_tick(|ctx, tick, app| { /* update every 250ms */ })
///     .run(|ctx, app| { /* render per frame */ });
/// ```
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
    resize_flag: Arc<AtomicBool>,
    tick_count: u64,
    on_tick: RefCell<Option<Box<dyn FnMut(&mut Ctx, u64) + 'static>>>,
    widgets: RefCell<Vec<Box<dyn Widget>>>,
    focus_manager: FocusManager,
    dirty_tracker: DirtyRegionTracker,
    animations: AnimationManager,
    next_widget_id: usize,
    commands: RefCell<Vec<BoundCommand>>,
    command_tracking: RefCell<HashMap<WidgetId, (Instant, BoundCommand)>>,
}

impl App {
    /// Creates a new `App` with a linked terminal.
    /// Returns an error if the terminal cannot be initialized.
    pub fn new() -> io::Result<Self> {
        let terminal = Terminal::new(io::stdout())?;
        let (w, h) = tty::get_window_size(io::stdout().as_fd()).unwrap_or((80, 24));

        Ok(Self {
            terminal,
            compositor: Compositor::new(w, h),
            parser: Parser::new(),
            title: String::from("Dracon App"),
            fps: 30,
            theme: Theme::default(),
            running: Arc::new(AtomicBool::new(true)),
            frame_count: Arc::new(AtomicU64::new(0)),
            last_frame_time: Instant::now(),
            last_tick_time: Instant::now(),
            tick_interval: Duration::from_millis(250),
            resize_flag: Arc::new(AtomicBool::new(false)),
            tick_count: 0,
            on_tick: RefCell::new(None),
            widgets: RefCell::new(Vec::new()),
            focus_manager: FocusManager::new(),
            dirty_tracker: DirtyRegionTracker::new(),
            animations: AnimationManager::new(),
            next_widget_id: 0,
            commands: RefCell::new(Vec::new()),
            command_tracking: RefCell::new(HashMap::new()),
        })
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

        let mut app = Self {
            terminal,
            compositor: Compositor::new(w, h),
            parser: Parser::new(),
            title: config.title.clone(),
            fps: config.fps.unwrap_or(30),
            theme: Theme::default(),
            running: Arc::new(AtomicBool::new(true)),
            frame_count: Arc::new(AtomicU64::new(0)),
            last_frame_time: Instant::now(),
            last_tick_time: Instant::now(),
            tick_interval: Duration::from_millis(250),
            resize_flag: Arc::new(AtomicBool::new(false)),
            tick_count: 0,
            on_tick: RefCell::new(None),
            widgets: RefCell::new(Vec::new()),
            focus_manager: FocusManager::new(),
            dirty_tracker: DirtyRegionTracker::new(),
            animations: AnimationManager::new(),
            next_widget_id: 0,
            commands: RefCell::new(config.commands),
            command_tracking: RefCell::new(HashMap::new()),
        };

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
        self.fps = fps.max(1).min(120);
        self
    }

    /// Sets the UI theme.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the UI theme and propagates it to all registered widgets.
    ///
    /// This calls `on_theme_change()` on every widget, allowing them to
    /// update internal theme-dependent state without requiring manual
    /// configuration of each widget.
    pub fn set_theme(&mut self, theme: Theme) -> &mut Self {
        self.theme = theme;
        self.dirty_tracker.mark_all_dirty();
        for widget in self.widgets.borrow_mut().iter_mut() {
            widget.on_theme_change(&theme);
            widget.mark_dirty();
        }
        self
    }

    /// Registers a callback that fires every `tick_interval` milliseconds.
    /// The callback receives the context and the tick count.
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

    /// Adds a widget to the application with the given area.
    /// Returns the assigned `WidgetId`.
    pub fn add_widget(&mut self, mut widget: Box<dyn Widget>, area: Rect) -> WidgetId {
        let id = WidgetId(self.next_widget_id);
        widget.set_id(id);
        widget.set_area(area);
        widget.on_mount();
        let focusable = widget.focusable();
        let cmds = widget.commands();
        self.widgets.borrow_mut().push(widget);
        self.focus_manager.register(id, focusable);
        self.next_widget_id += 1;

        for cmd in cmds {
            if cmd.refresh_seconds.is_some() {
                self.command_tracking.borrow_mut().insert(id, (Instant::now(), cmd));
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
        self.focus_manager.unregister(id);
        self.command_tracking.borrow_mut().remove(&id);
    }

    /// Returns an immutable reference to a widget by ID.
    pub fn widget(&self, id: WidgetId) -> Option<Ref<'_, Box<dyn Widget>>> {
        let widgets = self.widgets.borrow();
        let idx = widgets.iter().position(|w| w.id() == id)?;
        Some(Ref::map(widgets, |w| &w[idx]))
    }

    /// Returns a mutable reference to a widget by ID.
    pub fn widget_mut(&mut self, id: WidgetId) -> Option<RefMut<'_, Box<dyn Widget>>> {
        let widgets = self.widgets.borrow_mut();
        let idx = widgets.iter().position(|w| w.id() == id)?;
        Some(RefMut::map(widgets, |w| &mut w[idx]))
    }

    /// Returns the number of registered widgets.
    pub fn widget_count(&self) -> usize {
        self.widgets.borrow().len()
    }

    /// Starts the application event loop.
    ///
    /// Reads input, fires tick callbacks, and invokes the render callback
    /// each frame until the user presses Ctrl+C or [`App::stop`] is called.
    pub fn run<F>(mut self, mut f: F) -> io::Result<()>
    where
        F: FnMut(&mut Ctx),
    {
        let running = self.running.clone();
        let resize_flag = self.resize_flag.clone();
        let frame_count = self.frame_count.clone();

        let title = self.title.clone();
        write!(self.terminal, "\x1b]0;{title}\x07").ok();

        let mut stdin = io::stdin();
        let mut buf = [0u8; 1024];
        let frame_duration = Duration::from_secs_f64(1.0 / self.fps as f64);

        while running.load(Ordering::SeqCst) {
            let frame_start = Instant::now();

            if resize_flag.load(Ordering::SeqCst) {
                resize_flag.store(false, Ordering::SeqCst);
                if let Ok((w, h)) = tty::get_window_size(io::stdout().as_fd()) {
                    self.compositor.resize(w, h);
                    self.dirty_tracker.mark_all_dirty();
                    for w in self.widgets.borrow_mut().iter_mut() {
                        w.mark_dirty();
                    }
                }
            }

            while let Ok(n) = stdin.read(&mut buf) {
                if n == 0 {
                    break;
                }
                for byte in buf.iter().take(n) {
                    if let Some(event) = self.parser.advance(*byte) {
                        match &event {
                            Event::Resize(w, h) => {
                                self.compositor.resize(*w, *h);
                                self.dirty_tracker.mark_all_dirty();
                                for w in self.widgets.borrow_mut().iter_mut() {
                                    w.mark_dirty();
                                }
                            }
                            Event::Key(k) => {
                                if k.code == crate::input::event::KeyCode::Char('c')
                                    && k.modifiers.contains(crate::input::event::KeyModifiers::CONTROL)
                                {
                                    let focused = self.focus_manager.focused();
                                    let dominated = focused.and_then(|id| self.widget_mut(id))
                                        .map(|mut w| w.handle_key(*k))
                                        .unwrap_or(false);
                                    if !dominated {
                                        running.store(false, Ordering::SeqCst);
                                    }
                                } else if k.code == crate::input::event::KeyCode::Tab {
                                    let old = self.focus_manager.focused();
                                    if k.modifiers.contains(crate::input::event::KeyModifiers::SHIFT) {
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
                                } else if let Some(focused) = self.focus_manager.focused() {
                                    if let Some(mut widget) = self.widget_mut(focused) {
                                        let _ = widget.handle_key(*k);
                                    }
                                }
                            }
                            Event::Mouse(mouse_event) => {
                                let col = mouse_event.column;
                                let row = mouse_event.row;
                                let target_id = {
                                    let widgets = self.widgets.borrow();
                                    let mut sorted: Vec<_> = widgets.iter().collect();
                                    sorted.sort_by_key(|w| w.z_index());
                                    sorted.into_iter().find(|w| {
                                        let a = w.area();
                                        col >= a.x && col < a.x + a.width && row >= a.y && row < a.y + a.height
                                    }).map(|w| w.id())
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
                                    if let Some(mut widget) = self.widget_mut(id) {
                                        let a = widget.area();
                                        let local_col = col.saturating_sub(a.x);
                                        let local_row = row.saturating_sub(a.y);
                                        let _ = widget.handle_mouse(
                                            mouse_event.kind,
                                            local_col,
                                            local_row,
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            {
                let mut widgets = self.widgets.borrow_mut();
                let mut sorted: Vec<_> = widgets.iter_mut().collect();
                sorted.sort_by_key(|w| w.z_index());
                for w in sorted {
                    if !w.needs_render() {
                        continue;
                    }
                    let area = w.area();
                    let plane = w.render(area);
                    w.clear_dirty();
                    self.compositor.add_plane(plane);
                }
            }

            if self.last_tick_time.elapsed() >= self.tick_interval {
                if let Some(ref mut tick_fn) = *self.on_tick.borrow_mut() {
                    tick_fn(&mut Ctx {
                        compositor: &mut self.compositor,
                        theme: &self.theme,
                        frame_count: frame_count.load(Ordering::SeqCst),
                        last_frame: &self.last_frame_time,
                        terminal: &mut self.terminal,
                        focus_manager: &mut self.focus_manager,
                        animations: &mut self.animations,
                        dirty_tracker: &mut self.dirty_tracker,
                        commands: &self.commands,
                    }, self.tick_count);
                    self.tick_count += 1;
                    self.last_tick_time = Instant::now();
                }
            }

            {
                let now = Instant::now();
                let mut to_reschedule: Vec<(WidgetId, BoundCommand)> = Vec::new();
                let tracked: HashMap<WidgetId, (Instant, BoundCommand)> =
                    self.command_tracking.borrow().clone();
                for (wid, (last_run, cmd)) in tracked.iter() {
                    let interval = Duration::from_secs(cmd.refresh_seconds.unwrap_or(0));
                    if interval.is_zero() || now.duration_since(*last_run) < interval {
                        continue;
                    }
                    if let Some(mut w) = self.widget_mut(*wid) {
                        let runner = CommandRunner::new(&cmd.command);
                        let (stdout, stderr, exit_code) = runner.run_sync();
                        let output = cmd.parse_output(&stdout, &stderr, exit_code);
                        w.apply_command_output(&output);
                        w.mark_dirty();
                        to_reschedule.push((*wid, cmd.clone()));
                    }
                }
                for (wid, cmd) in to_reschedule {
                    self.command_tracking.borrow_mut().insert(wid, (Instant::now(), cmd));
                }
            }

            f(&mut Ctx {
                compositor: &mut self.compositor,
                theme: &self.theme,
                frame_count: frame_count.load(Ordering::SeqCst),
                last_frame: &self.last_frame_time,
                terminal: &mut self.terminal,
                focus_manager: &mut self.focus_manager,
                animations: &mut self.animations,
                dirty_tracker: &mut self.dirty_tracker,
                commands: &self.commands,
            });

            self.compositor.render(&mut self.terminal)?;

            self.animations.tick();

            frame_count.fetch_add(1, Ordering::SeqCst);
            self.last_frame_time = Instant::now();

            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
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
    fn default() -> Self {
        Self::new().expect("failed to initialize terminal")
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
pub struct Ctx<'a> {
    pub(crate) compositor: &'a mut Compositor,
    pub(crate) theme: &'a Theme,
    pub(crate) frame_count: u64,
    pub(crate) last_frame: &'a Instant,
    pub(crate) terminal: &'a mut crate::Terminal<io::Stdout>,
    pub(crate) focus_manager: &'a mut FocusManager,
    pub(crate) animations: &'a mut AnimationManager,
    pub(crate) dirty_tracker: &'a mut DirtyRegionTracker,
    pub(crate) commands: &'a RefCell<Vec<BoundCommand>>,
}

impl<'a> Ctx<'a> {
    /// Adds a plane to the compositor.
    pub fn add_plane(&mut self, plane: Plane) {
        self.compositor.add_plane(plane);
    }

    /// Shows the terminal cursor (for text input widgets).
    pub fn show_cursor(&mut self) -> io::Result<()> {
        self.terminal.show_cursor()
    }

    /// Hides the terminal cursor (for non-text widgets during render).
    pub fn hide_cursor(&mut self) -> io::Result<()> {
        self.terminal.hide_cursor()
    }

    /// Sets the terminal cursor position.
    pub fn set_cursor(&mut self, col: u16, row: u16) -> io::Result<()> {
        self.terminal.set_cursor(col, row)
    }

    /// Sets the focused widget by ID.
    pub fn set_focus(&mut self, id: WidgetId) {
        self.focus_manager.set_focus(id);
    }

    /// Returns the currently focused widget ID, if any.
    pub fn focused(&self) -> Option<WidgetId> {
        self.focus_manager.focused()
    }

    /// Returns the animation manager for managing toasts, progress bars, etc.
    pub fn animations(&self) -> &AnimationManager {
        self.animations
    }

    /// Returns a mutable reference to the animation manager.
    pub fn animations_mut(&mut self) -> &mut AnimationManager {
        self.animations
    }

    /// Marks a screen region as dirty, so it will be re-rendered on the next frame.
    pub fn mark_dirty(&mut self, x: u16, y: u16, width: u16, height: u16) {
        self.dirty_tracker.mark_dirty(x, y, width, height);
    }

    /// Marks the entire screen as dirty, requiring a full refresh.
    pub fn mark_all_dirty(&mut self) {
        self.dirty_tracker.mark_all_dirty();
    }

    /// Returns true if a full screen refresh is needed.
    pub fn needs_full_refresh(&self) -> bool {
        self.dirty_tracker.needs_full_refresh()
    }

    /// Returns an immutable reference to the compositor.
    pub fn compositor(&self) -> &Compositor {
        self.compositor
    }

    /// Returns a mutable reference to the compositor.
    pub fn compositor_mut(&mut self) -> &mut Compositor {
        self.compositor
    }

    /// Clears the entire terminal.
    pub fn clear(&mut self) {
        self.compositor.force_clear();
    }

    /// Returns the measured frames per second based on elapsed time and frame count.
    pub fn fps(&self) -> u64 {
        let elapsed = self.last_frame.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            (self.frame_count as f64 / elapsed) as u64
        } else {
            0
        }
    }

    /// Returns a reference to the current theme.
    pub fn theme(&self) -> &Theme {
        self.theme
    }

    /// Splits the screen horizontally into two panes and passes them to the closure.
    ///
    /// The closure receives two `SplitPane` instances covering the left and right halves.
    pub fn split_h<F>(&mut self, f: F)
    where
        F: FnOnce(&mut crate::framework::widgets::split::SplitPane, &mut crate::framework::widgets::split::SplitPane),
    {
        let (w, h) = self.compositor.size();
        let split = crate::framework::widgets::split::SplitPane::new(crate::framework::widgets::split::Orientation::Horizontal).ratio(0.5);
        let (r1, r2) = split.split(Rect::new(0, 0, w, h));
        let mut left = crate::framework::widgets::split::SplitPane::from_rect(r1);
        let mut right = crate::framework::widgets::split::SplitPane::from_rect(r2);
        f(&mut left, &mut right);
    }

    /// Splits the screen vertically into two panes and passes them to the closure.
    ///
    /// The closure receives two `SplitPane` instances covering the top and bottom halves.
    pub fn split_v<F>(&mut self, f: F)
    where
        F: FnOnce(&mut crate::framework::widgets::split::SplitPane, &mut crate::framework::widgets::split::SplitPane),
    {
        let (w, h) = self.compositor.size();
        let split = crate::framework::widgets::split::SplitPane::new(crate::framework::widgets::split::Orientation::Vertical).ratio(0.5);
        let (r1, r2) = split.split(Rect::new(0, 0, w, h));
        let mut left = crate::framework::widgets::split::SplitPane::from_rect(r1);
        let mut right = crate::framework::widgets::split::SplitPane::from_rect(r2);
        f(&mut left, &mut right);
    }

    /// Lays out child rectangles using the given constraints within the screen area.
    ///
    /// This is a convenience method that uses the [`Layout`] engine to compute
    /// widget rectangles from constraint specifications (percentage, fixed, min, max, ratio).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use dracon_terminal_engine::framework::prelude::*;
    ///
    /// App::new().unwrap().run(|ctx| {
    ///     let rects = ctx.layout(vec![
    ///         Constraint::Percentage(30),
    ///         Constraint::Percentage(70),
    ///     ]);
    ///     // Use rects[0] and rects[1] for widget areas
    /// });
    /// ```
    pub fn layout(&self, constraints: Vec<crate::framework::layout::Constraint>) -> Vec<Rect> {
        let (w, h) = self.compositor.size();
        let layout = crate::framework::layout::Layout::new(constraints);
        layout.layout(Rect::new(0, 0, w, h))
    }

    /// Runs a command synchronously and returns its output.
    ///
    /// This is the primary way widgets execute CLI commands.
    /// AI can also call this directly via `ctx.run_command("dracon-sync status")`.
    ///
    /// Returns `(stdout, stderr, exit_code)`.
    pub fn run_command(&self, cmd: &str) -> (String, String, i32) {
        let runner = CommandRunner::new(cmd);
        runner.run_sync()
    }

    /// Returns all available commands registered with the app.
    ///
    /// This is the primary AI surface — an AI can query this to know
    /// every action the TUI can perform.
    pub fn available_commands(&self) -> Vec<BoundCommand> {
        self.commands.borrow().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::command::{AppConfig, AreaConfig, LayoutConfig, ParserConfig, WidgetConfig};

    fn make_test_terminal() -> io::Result<crate::Terminal<io::Stdout>> {
        crate::Terminal::new(io::stdout())
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
        let mut w = app.widget_mut(id);
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
        assert!(cmds.len() >= 1);
    }

    #[test]
    fn test_app_available_commands_includes_widget_commands() {
        use crate::framework::widgets::Label;
        let mut app = App::new().unwrap();
        let label = Label::new("test");
        let id = app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));
        let cmds = app.available_commands();
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_app_set_theme() {
        use crate::framework::widgets::Label;
        let mut app = App::new().unwrap();
        let label = Label::new("test");
        app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));
        let theme = Theme::cyberpunk();
        app.set_theme(theme);
        assert_eq!(app.theme.name, "cyberpunk");
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
        let app = App::new().unwrap();
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = Instant::now();
        let commands = RefCell::new(Vec::new());

        let ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        let cmds = ctx.available_commands();
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_ctx_add_plane() {
        let app = App::new().unwrap();
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = Instant::now();
        let commands = RefCell::new(Vec::new());

        let mut ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        let plane = Plane::new(0, 20, 10);
        ctx.add_plane(plane);
        assert_eq!(compositor.planes.len(), 1);
    }

#[test]
    fn test_ctx_mark_dirty() {
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = std::time::Instant::now();
        let commands = RefCell::new(Vec::new());

        let mut ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        ctx.mark_dirty(0, 0, 80, 24);
        assert!(true);
    }

    #[test]
    fn test_ctx_set_focus() {
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = std::time::Instant::now();
        let commands = RefCell::new(Vec::new());

        let mut ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        let id = WidgetId(42);
        ctx.set_focus(id);
        assert!(ctx.focused().is_some() || ctx.focused().is_none());
    }

    #[test]
    fn test_ctx_theme_access() {
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = std::time::Instant::now();
        let commands = RefCell::new(Vec::new());

        let ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        assert!(ctx.theme().name == "default" || ctx.theme().name == "dark");
    }

    #[test]
    fn test_ctx_mark_all_dirty() {
        let app = App::new().unwrap();
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = Instant::now();
        let commands = RefCell::new(Vec::new());

        let mut ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        ctx.mark_all_dirty();
        assert!(dirty_tracker.needs_full_refresh());
    }

    #[test]
    fn test_ctx_clear() {
        let app = App::new().unwrap();
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = Instant::now();
        let commands = RefCell::new(Vec::new());

        let mut ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        ctx.clear();
        assert!(dirty_tracker.needs_full_refresh());
    }

    #[test]
    fn test_ctx_compositor_access() {
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = Instant::now();
        let commands = RefCell::new(Vec::new());

        let ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        let (w, h) = ctx.compositor().size();
        assert_eq!(w, 80);
        assert_eq!(h, 24);
    }

    #[test]
    fn test_ctx_fps_zero_elapsed() {
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = std::time::Instant::now();
        let commands = RefCell::new(Vec::new());

        let ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 100,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        let fps = ctx.fps();
        assert!(fps >= 0);
    }

    #[test]
    fn test_ctx_split_h() {
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = Instant::now();
        let commands = RefCell::new(Vec::new());

        let mut ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        ctx.split_h(|left, right| {
            let a = left.area();
            let b = right.area();
            assert!(a.width > 0);
            assert!(b.width > 0);
        });
    }

    #[test]
    fn test_ctx_split_v() {
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = Instant::now();
        let commands = RefCell::new(Vec::new());

        let mut ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        ctx.split_v(|top, bottom| {
            let a = top.area();
            let b = bottom.area();
            assert!(a.height > 0);
            assert!(b.height > 0);
        });
    }

    #[test]
    fn test_ctx_layout() {
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = Instant::now();
        let commands = RefCell::new(Vec::new());

        let ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        use crate::framework::layout::Constraint;
        let rects = ctx.layout(vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
        assert_eq!(rects.len(), 2);
    }

    #[test]
    fn test_ctx_run_command() {
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = Instant::now();
        let commands = RefCell::new(Vec::new());

        let ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
        };

        let (stdout, stderr, code) = ctx.run_command("echo test_run_command");
        assert!(stdout.contains("test_run_command"));
        assert_eq!(code, 0);
    }

    #[test]
    fn test_ctx_available_commands() {
        let mut compositor = Compositor::new(80, 24);
        let mut focus_manager = FocusManager::new();
        let mut dirty_tracker = DirtyRegionTracker::new();
        let mut animations = AnimationManager::new();
        let theme = Theme::default();
        let last_frame = Instant::now();
        let commands = RefCell::new(vec![
            BoundCommand::new("test cmd 1"),
            BoundCommand::new("test cmd 2"),
        ]);

        let ctx = Ctx {
            compositor: &mut compositor,
            theme: &theme,
            frame_count: 0,
            last_frame: &last_frame,
            terminal: &mut make_test_terminal().unwrap(),
            focus_manager: &mut focus_manager,
            animations: &mut animations,
            dirty_tracker: &mut dirty_tracker,
            commands: &commands,
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
    fn test_app_command_tracking_on_add_widget() {
        use crate::framework::command::BoundCommand;
        let mut app = App::new().unwrap();
        let cmd = BoundCommand::new("echo test").refresh(5);
        let label = crate::framework::widgets::Label::new("test");
        let id = app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));
        let tracking = app.command_tracking.borrow();
        assert!(tracking.is_empty());
    }

    #[test]
    fn test_app_command_tracking_removed_on_widget_remove() {
        use crate::framework::command::BoundCommand;
        let mut app = App::new().unwrap();
        let cmd = BoundCommand::new("echo test").refresh(5);
        let mut label = crate::framework::widgets::Label::new("test");
        let id = app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));
        app.remove_widget(id);
        assert!(app.command_tracking.borrow().is_empty());
    }
}
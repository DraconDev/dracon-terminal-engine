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
use crate::framework::dirty_regions::DirtyRegionTracker;
use crate::framework::event_dispatcher::EventDispatcher;
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
    #[allow(unused)]
    event_dispatcher: EventDispatcher,
    dirty_tracker: DirtyRegionTracker,
    animations: AnimationManager,
    next_widget_id: usize,
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
            event_dispatcher: EventDispatcher::new(),
            dirty_tracker: DirtyRegionTracker::new(),
            animations: AnimationManager::new(),
            next_widget_id: 0,
        })
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
        widget.set_area(area);
        widget.on_mount();
        let focusable = widget.focusable();
        self.widgets.borrow_mut().push(widget);
        self.focus_manager.register(id, focusable);
        self.next_widget_id += 1;
        id
    }

    /// Removes a widget by its ID.
    pub fn remove_widget(&mut self, id: WidgetId) {
        if let Some(w) = self.widgets.borrow_mut().iter_mut().find(|w| w.id() == id) {
            w.on_unmount();
        }
        self.widgets.borrow_mut().retain(|w| w.id() != id);
        self.focus_manager.unregister(id);
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
                    w.mark_dirty();
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
                    }, self.tick_count);
                    self.tick_count += 1;
                    self.last_tick_time = Instant::now();
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
        &self.animations
    }

    /// Returns a mutable reference to the animation manager.
    pub fn animations_mut(&mut self) -> &mut AnimationManager {
        &mut self.animations
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
}
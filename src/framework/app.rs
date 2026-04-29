//! The main application entry point.

use crate::backend::tty;
use crate::compositor::{Compositor, Plane};
use crate::framework::theme::Theme;
use crate::input::event::Event;
use crate::input::parser::Parser;
use crate::Terminal;
use ratatui::layout::Rect;
use std::io::{self, Read, Write};
use std::os::fd::AsFd;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::cell::RefCell;

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
    on_tick: RefCell<Option<Box<dyn FnMut(&mut Ctx, u64)>>>,
}

impl App {
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
        })
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        write!(self.terminal, "\x1b]0;{title}\x07").ok();
        self
    }

    pub fn fps(mut self, fps: u32) -> Self {
        self.fps = fps.max(1).min(120);
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn on_tick<F>(self, f: F) -> Self
    where
        F: FnMut(&mut Ctx, u64) + 'static,
    {
        *self.on_tick.borrow_mut() = Some(Box::new(f));
        self
    }

    pub fn tick_interval(mut self, ms: u64) -> Self {
        self.tick_interval = Duration::from_millis(ms);
        self
    }

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
                            }
                            Event::Key(k) => {
                                if k.code == crate::input::event::KeyCode::Char('c')
                                    && k.modifiers.contains(crate::input::event::KeyModifiers::CONTROL)
                                {
                                    running.store(false, Ordering::SeqCst);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            let mut ctx = Ctx {
                compositor: &mut self.compositor,
                theme: &self.theme,
                frame_count: frame_count.load(Ordering::SeqCst),
                last_frame: &self.last_frame_time,
            };

            if self.last_tick_time.elapsed() >= self.tick_interval {
                if let Some(ref mut tick_fn) = *self.on_tick.borrow_mut() {
                    tick_fn(&mut ctx, self.tick_count);
                }
                self.tick_count += 1;
                self.last_tick_time = Instant::now();
            }

            f(&mut ctx);

            self.compositor.render(&mut self.terminal)?;

            frame_count.fetch_add(1, Ordering::SeqCst);
            self.last_frame_time = Instant::now();

            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("failed to initialize terminal")
    }
}

pub struct Ctx<'a> {
    pub(crate) compositor: &'a mut Compositor,
    pub(crate) theme: &'a Theme,
    pub(crate) frame_count: u64,
    pub(crate) last_frame: &'a Instant,
}

impl<'a> Ctx<'a> {
    pub fn add_plane(&mut self, plane: Plane) {
        self.compositor.add_plane(plane);
    }

    pub fn compositor(&self) -> &Compositor {
        self.compositor
    }

    pub fn compositor_mut(&mut self) -> &mut Compositor {
        self.compositor
    }

    pub fn clear(&mut self) {
        self.compositor.force_clear();
    }

    pub fn fps(&self) -> u64 {
        let elapsed = self.last_frame.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            (self.frame_count as f64 / elapsed) as u64
        } else {
            0
        }
    }

    pub fn theme(&self) -> &Theme {
        self.theme
    }

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
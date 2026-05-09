#![allow(missing_docs)]
//! Network Client — HTTP API consumer example.
//!
//! Fetches posts from JSONPlaceholder API and displays them in a list.
//! Demonstrates: async data fetching, loading states, error handling,
//! detail view navigation, and JSON parsing.
//!
//! Controls:
//!   ↑/↓      — navigate posts
//!   Enter    — view post details
//!   r        — refresh data
//!   t        — cycle theme
//!   ?        — toggle help
//!   q        — quit

use dracon_terminal_engine::compositor::{Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════════════════
// DATA MODEL
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
struct Post {
    id: u32,
    title: String,
    body: String,
}

impl Post {
    fn from_json(value: &serde_json::Value) -> Option<Self> {
        Some(Self {
            id: value.get("id")?.as_u64()? as u32,
            title: value.get("title")?.as_str()?.to_string(),
            body: value.get("body")?.as_str()?.to_string(),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API CLIENT
// ═══════════════════════════════════════════════════════════════════════════════

fn fetch_posts() -> Result<Vec<Post>, String> {
    let output = Command::new("curl")
        .args(["-s", "-m", "5", "https://jsonplaceholder.typicode.com/posts?_limit=10"])
        .output()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !output.status.success() {
        return Err(format!("curl exited with code: {:?}", output.status.code()));
    }

    let json_str = String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8: {}", e))?;
    let json: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| format!("JSON parse error: {}", e))?;

    let posts = json
        .as_array()
        .ok_or("Expected JSON array")?
        .iter()
        .filter_map(Post::from_json)
        .collect();

    Ok(posts)
}

// ═══════════════════════════════════════════════════════════════════════════════
// APP STATE
// ═══════════════════════════════════════════════════════════════════════════════

struct NetworkApp {
    posts: Vec<Post>,
    selected: usize,
    loading: bool,
    error: Option<String>,
    view_detail: bool,
    theme: Theme,
    show_help: bool,
    should_quit: Arc<AtomicBool>,
}

impl NetworkApp {
    fn new(should_quit: Arc<AtomicBool>) -> Self {
        Self {
            posts: vec![],
            selected: 0,
            loading: true,
            error: None,
            view_detail: false,
            theme: Theme::nord(),
            show_help: false,
            should_quit,
        }
    }

    fn refresh(&mut self) {
        self.loading = true;
        self.error = None;
        match fetch_posts() {
            Ok(posts) => {
                self.posts = posts;
                self.selected = 0;
            }
            Err(e) => self.error = Some(e),
        }
        self.loading = false;
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::nord(),
            Theme::dracula(),
            Theme::cyberpunk(),
            Theme::gruvbox_dark(),
            Theme::tokyo_night(),
        ];
        let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];
    }

    fn render_list(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = self.theme.bg;
        }

        let t = &self.theme;

        // Title bar
        let title = "🌐 Network Client — JSONPlaceholder API";
        let tx = (area.width as usize).saturating_sub(title.len()) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = tx + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Status
        let status = if self.loading {
            "Loading...".to_string()
        } else if let Some(ref e) = self.error {
            format!("Error: {}", e)
        } else {
            format!("{} posts loaded", self.posts.len())
        };

        let sx = 2usize;
        for (i, c) in status.chars().enumerate() {
            let idx = (area.width as usize) + sx + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if self.error.is_some() {
                    t.error
                } else if self.loading {
                    t.warning
                } else {
                    t.success
                };
            }
        }

        // Status hint
        let hint = "↑↓:nav | Enter:detail | r:refresh | t:theme | ?:help | q:quit";
        let hint_x = area.width as usize - hint.len() - 2;
        for (i, c) in hint.chars().enumerate() {
            let idx = (area.width as usize) + hint_x + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
            }
        }

        // List items
        let start_y = 3;
        for (i, post) in self.posts.iter().enumerate() {
            let y = start_y + i * 2;
            if y >= area.height as usize {
                break;
            }

            let is_selected = i == self.selected;
            let prefix = if is_selected { "▶ " } else { "  " };
            let line = format!("{}{}. {}", prefix, post.id, &post.title[..post.title.len().min(40)]);

            let bg = if is_selected { t.selection_bg } else { t.bg };
            let fg = if is_selected { t.primary } else { t.fg };

            for (j, c) in line.chars().enumerate() {
                let idx = y * area.width as usize + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        plane
    }

    fn render_help(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = self.theme.bg;
        }

        let t = &self.theme;
        let shortcuts = [
            ("↑/↓", "Navigate posts"),
            ("Enter", "View post details"),
            ("r", "Refresh data"),
            ("t", "Cycle theme"),
            ("?", "Toggle this help"),
            ("q", "Quit"),
        ];

        let hw = 42u16.min(area.width.saturating_sub(4));
        let hh = (shortcuts.len() as u16 + 4).min(area.height.saturating_sub(4));
        let hx = (area.width - hw) / 2;
        let hy = (area.height - hh) / 2;

        for y in hy..hy + hh {
            for x in hx..hx + hw {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Rounded border
        let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
        for (ch, cx, cy) in corners.iter() {
            let idx = (cy * area.width + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = *ch;
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }
        for x in hx + 1..hx + hw - 1 {
            let top = (hy * area.width + x) as usize;
            let bot = ((hy + hh - 1) * area.width + x) as usize;
            if top < plane.cells.len() {
                plane.cells[top].char = '─';
                plane.cells[top].fg = t.outline;
                plane.cells[top].transparent = false;
            }
            if bot < plane.cells.len() {
                plane.cells[bot].char = '─';
                plane.cells[bot].fg = t.outline;
                plane.cells[bot].transparent = false;
            }
        }
        for y in hy + 1..hy + hh - 1 {
            let left = (y * area.width + hx) as usize;
            let right = (y * area.width + hx + hw - 1) as usize;
            if left < plane.cells.len() {
                plane.cells[left].char = '│';
                plane.cells[left].fg = t.outline;
                plane.cells[left].transparent = false;
            }
            if right < plane.cells.len() {
                plane.cells[right].char = '│';
                plane.cells[right].fg = t.outline;
                plane.cells[right].transparent = false;
            }
        }

        // Title
        let title = "Network Client — Help";
        let tx = hx + (hw - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
                plane.cells[idx].transparent = false;
            }
        }

        // Two-column layout
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            for (j, c) in key.chars().enumerate() {
                let idx = (row * area.width + hx + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                    plane.cells[idx].transparent = false;
                }
            }
            for (j, c) in desc.chars().enumerate() {
                let idx = (row * area.width + hx + 14 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        plane
    }

    fn render_detail(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = self.theme.bg;
        }

        let t = &self.theme;
        let post = &self.posts[self.selected];

        // Header
        let header = format!("Post #{} — {}", post.id, &post.title[..post.title.len().min(50)]);
        for (i, c) in header.chars().enumerate() {
            if i < area.width as usize {
                plane.cells[i].char = c;
                plane.cells[i].fg = t.primary;
                plane.cells[i].style = Styles::BOLD;
            }
        }

        // Body (word wrap)
        let mut y = 2usize;
        let mut x = 2usize;
        for word in post.body.split_whitespace() {
            if x + word.len() + 1 > area.width as usize - 2 {
                y += 1;
                x = 2;
                if y >= area.height as usize {
                    break;
                }
            }
            for c in word.chars() {
                let idx = y * area.width as usize + x;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                }
                x += 1;
            }
            x += 1; // space
        }

        // Footer hint
        let hint = "Press Enter or Backspace to return";
        let hy = area.height as usize - 1;
        let hx = (area.width as usize).saturating_sub(hint.len()) / 2;
        for (i, c) in hint.chars().enumerate() {
            let idx = hy * area.width as usize + hx + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
            }
        }

        plane
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            KeyCode::Char('r') if !self.view_detail => {
                self.refresh();
                true
            }
            KeyCode::Char('t') => {
                self.cycle_theme();
                true
            }
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                true
            }
            KeyCode::Up if !self.view_detail && !self.posts.is_empty() => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                true
            }
            KeyCode::Down if !self.view_detail && !self.posts.is_empty() => {
                if self.selected + 1 < self.posts.len() {
                    self.selected += 1;
                }
                true
            }
            KeyCode::Enter if !self.posts.is_empty() => {
                self.view_detail = !self.view_detail;
                true
            }
            KeyCode::Backspace if self.view_detail => {
                self.view_detail = false;
                true
            }
            KeyCode::Esc if self.show_help => {
                self.show_help = false;
                true
            }
            _ => false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET WRAPPER
// ═══════════════════════════════════════════════════════════════════════════════

struct NetworkWidget {
    app: Rc<RefCell<NetworkApp>>,
    id: WidgetId,
    area: std::cell::Cell<Rect>,
}

impl Widget for NetworkWidget {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn area(&self) -> Rect {
        self.area.get()
    }
    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }
    fn needs_render(&self) -> bool {
        true
    }
    fn render(&self, area: Rect) -> Plane {
        let app = self.app.borrow();
        if app.show_help {
            app.render_help(area)
        } else if app.view_detail {
            app.render_detail(area)
        } else {
            app.render_list(area)
        }
    }
    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        self.app.borrow_mut().handle_key(key)
    }
    fn on_theme_change(&mut self, theme: &Theme) {
        self.app.borrow_mut().theme = *theme;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("Network Client — fetching data from JSONPlaceholder API...");
    println!("Requires: curl, internet connection");
    std::thread::sleep(std::time::Duration::from_millis(500));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let app = NetworkApp::new(should_quit);
    let app_for_widget = Rc::new(RefCell::new(app));
    let _app_for_tick = Rc::clone(&app_for_widget);

    // Initial fetch
    app_for_widget.borrow_mut().refresh();

    let mut framework = App::new()?
        .title("Network Client")
        .fps(30)
        .theme(Theme::nord());

    let widget = NetworkWidget {
        app: app_for_widget,
        id: WidgetId::new(1),
        area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
    };
    framework.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    framework
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
            }
        })
        .run(|_| {})
}

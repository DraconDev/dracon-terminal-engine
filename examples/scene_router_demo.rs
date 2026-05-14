//! Scene Router Demo — Multi-screen navigation with EventBus communication.
//!
//! Demonstrates the Dracon app architecture:
//! - SceneRouter for push/pop navigation between screens
//! - EventBus for decoupled communication (theme changes broadcast to all screens)
//! - Centralized AppState shared across scenes
//!
//! Screens:
//!   Home      — Main menu with navigation options
//!   Settings  — Theme toggle (broadcasts via EventBus)
//!   Profile   — User info display
//!
//! Controls:
//!   ↑/↓       — navigate menu
//!   Enter     — select / go forward
//!   Esc — go back
//!   t         — cycle theme (global)
//!   ?         — toggle help
//!   q         — quit

use dracon_terminal_engine::compositor::{Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::scene_router::{Scene, SceneRouter};
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════════════════
// APP STATE (shared across all scenes)
// ═══════════════════════════════════════════════════════════════════════════════

// AppEvent and AppState are part of the architecture pattern demonstrated
// by this example, showing how a real app would structure state.
#[allow(dead_code)]
#[derive(Clone, Debug)]
enum AppEvent {
    ThemeChanged(String),
    NavigationRequested(String),
    UserLoggedIn(String),
}

#[allow(dead_code)]
struct AppState {
    username: String,
    login_count: u32,
    theme_index: usize,
}

#[allow(dead_code)]
impl AppState {
    fn new() -> Self {
        Self {
            username: "Guest".into(),
            login_count: 1,
            theme_index: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HOME SCREEN
// ═══════════════════════════════════════════════════════════════════════════════

struct HomeScreen {
    theme: Theme,
    selected: usize,
    items: Vec<&'static str>,
    dirty: bool,
}

impl HomeScreen {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
            selected: 0,
            items: vec!["Profile", "Settings", "Help"],
            dirty: true,
        }
    }
}

impl Scene for HomeScreen {
    fn scene_id(&self) -> &str { "home" }

    fn on_enter(&mut self) {
        self.dirty = true;
    }

    fn on_resume(&mut self) {
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = self.theme.clone();

        // Background
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.transparent = false;
        }

        // Title
        let title = "🏠 Home";
        let tx = 2u16;
        for (i, c) in title.chars().enumerate() {
            let idx = (tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Menu items
        for (i, item) in self.items.iter().enumerate() {
            let y = 3 + i as u16;
            let is_selected = i == self.selected;
            let prefix = if is_selected { "▸ " } else { "  " };
            let text = format!("{}{}", prefix, item);

            for (j, c) in text.chars().enumerate() {
                let idx = (y * area.width + 4 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if is_selected { t.selection_fg } else { t.fg };
                    plane.cells[idx].bg = if is_selected { t.selection_bg } else { t.bg };
                    plane.cells[idx].style = if is_selected { Styles::BOLD } else { Styles::empty() };
                }
            }
        }

        // Hint
        let hint = "Enter: select | t: theme | ?: help | Esc: dismiss | q: quit";
        let hy = area.height - 1;
        for (i, c) in hint.chars().enumerate() {
            let idx = (hy * area.width + i as u16) as usize;
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
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Down => {
                if self.selected < self.items.len() - 1 {
                    self.selected += 1;
                    self.dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, row: u16) -> bool {
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let first_item_y = 3u16;
                let last_item_y = first_item_y + self.items.len() as u16 - 1;
                if row >= first_item_y && row <= last_item_y {
                    let idx = (row - first_item_y) as usize;
                    if idx < self.items.len() {
                        self.selected = idx;
                        self.dirty = true;
                        return true;
                    }
                }
                false
            }
            MouseEventKind::ScrollDown => {
                if self.selected < self.items.len().saturating_sub(1) {
                    self.selected += 1;
                    self.dirty = true;
                }
                true
            }
            MouseEventKind::ScrollUp => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SETTINGS SCREEN
// ═══════════════════════════════════════════════════════════════════════════════

struct SettingsScreen {
    theme: Theme,
    dirty: bool,
    option_enabled: bool,
}

impl SettingsScreen {
    fn new(theme: Theme) -> Self {
        Self { theme, dirty: true, option_enabled: false }
    }
}

impl Scene for SettingsScreen {
    fn scene_id(&self) -> &str { "settings" }

    fn on_enter(&mut self) {
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = self.theme.clone();

        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.transparent = false;
        }

        // Title
        let title = "⚙️  Settings";
        for (i, c) in title.chars().enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Theme info
        let theme_text = format!("Current theme: {}", t.name);
        let y = 3;
        for (i, c) in theme_text.chars().enumerate() {
            let idx = (y * area.width + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg;
            }
        }

        // Toggle option
        let toggle_label = "Toggle Option";
        let toggle_y = 6;
        for (i, c) in toggle_label.chars().enumerate() {
            let idx = (toggle_y * area.width + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg;
            }
        }
        let toggle_status = if self.option_enabled { " [ON] " } else { " [OFF]" };
        let status_x = 2 + toggle_label.len() as u16 + 1;
        for (i, c) in toggle_status.chars().enumerate() {
            let idx = (toggle_y * area.width + status_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if self.option_enabled { t.success } else { t.error };
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Instructions
        let instructions = "Press 't' to cycle theme | Esc to go back";
        let iy = 8;
        for (i, c) in instructions.chars().enumerate() {
            let idx = (iy * area.width + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
            }
        }

        plane
    }

    fn handle_key(&mut self, _key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, row: u16) -> bool {
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let toggle_y = 6u16;
                if row == toggle_y {
                    self.option_enabled = !self.option_enabled;
                    self.dirty = true;
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PROFILE SCREEN
// ═══════════════════════════════════════════════════════════════════════════════

struct ProfileScreen {
    theme: Theme,
    dirty: bool,
}

impl ProfileScreen {
    fn new(theme: Theme) -> Self {
        Self { theme, dirty: true }
    }
}

impl Scene for ProfileScreen {
    fn scene_id(&self) -> &str { "profile" }

    fn on_enter(&mut self) {
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = self.theme.clone();

        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.transparent = false;
        }

        // Title
        let title = "👤 Profile";
        for (i, c) in title.chars().enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // User info
        let info = [
            ("Name:", "Alice"),
            ("Role:", "Developer"),
            ("Theme:", &*t.name),
        ];
        for (i, (label, value)) in info.iter().enumerate() {
            let y = 3 + i as u16;
            for (j, c) in label.chars().enumerate() {
                let idx = (y * area.width + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg_muted;
                }
            }
            for (j, c) in value.chars().enumerate() {
                let idx = (y * area.width + 12 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, _key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        matches!(kind, MouseEventKind::Down(MouseButton::Left))
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

// ═══════════════════════════════════════════════════════════════════════════════
// INPUT ROUTER
// ═══════════════════════════════════════════════════════════════════════════════

struct AppRouter {
    router: Rc<RefCell<SceneRouter>>,
    theme: Rc<RefCell<Theme>>,
    should_quit: Arc<AtomicBool>,
    show_help: Rc<RefCell<bool>>,
    keybindings: KeybindingSet,
    id: WidgetId,
    area: std::cell::Cell<Rect>,
}

impl AppRouter {
    fn new(
        router: Rc<RefCell<SceneRouter>>,
        theme: Rc<RefCell<Theme>>,
        should_quit: Arc<AtomicBool>,
        show_help: Rc<RefCell<bool>>,
        keybindings: KeybindingSet,
    ) -> Self {
        Self {
            router,
            theme,
            should_quit,
            show_help,
            keybindings,
            id: WidgetId::new(100),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn cycle_theme(&self) {
        let themes = Theme::all();
        let mut theme = self.theme.borrow_mut();
        let idx = themes.iter().position(|t| t.name == theme.name).unwrap_or(0);
        *theme = themes[(idx + 1) % themes.len()].clone();
    }
}

impl dracon_terminal_engine::framework::widget::Widget for AppRouter {
    fn id(&self) -> WidgetId { self.id }
    fn area(&self) -> Rect { self.area.get() }
    fn set_area(&mut self, area: Rect) { self.area.set(area); }
    fn needs_render(&self) -> bool { false }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn z_index(&self) -> u16 { 0 }
    fn render(&self, _area: Rect) -> Plane { Plane::new(0, 0, 0) }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        let kb = &self.keybindings;

        if *self.show_help.borrow() {
            if kb.matches(actions::DISMISS, &key) || kb.matches(actions::HELP, &key) {
                *self.show_help.borrow_mut() = false;
                return true;
            }
            return false;
        }

        if kb.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if kb.matches(actions::HELP, &key) {
            *self.show_help.borrow_mut() = true;
            return true;
        }
        if kb.matches(actions::THEME, &key) {
            self.cycle_theme();
            let theme = self.theme.borrow().clone();
            self.router.borrow_mut().on_theme_change(&theme);
            return true;
        }
        if kb.matches(actions::BACK, &key) {
            self.router.borrow_mut().pop();
            return true;
        }
        if key.code == KeyCode::Enter {
            let current = self.router.borrow().current().map(|s| s.to_string());
            if let Some(current) = current {
                if current.as_str() == "home" {
                    let router = self.router.borrow();
                    // Check selected item on home screen - we'd need to query it
                    // For simplicity, just go to profile
                    drop(router);
                    self.router.borrow_mut().push("profile");
                }
            }
            return true;
        }
        // Delegate to current scene
        self.router.borrow_mut().handle_key(key)
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let handled = self.router.borrow_mut().handle_mouse(kind, col, row);
        if handled {
            let current = self.router.borrow().current().map(|s| s.to_string());
            if let Some(ref id) = current {
                if id == "profile" {
                    self.router.borrow_mut().pop();
                }
            }
        }
        handled
    }
    fn current_theme(&self) -> Option<Theme> {
        Some(self.theme.borrow().clone())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("Scene Router Demo | Multi-screen navigation | ?: help | Esc: dismiss | q: quit");
    std::thread::sleep(std::time::Duration::from_millis(300));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);
    let show_help = Rc::new(RefCell::new(false));
    let show_help_for_tick = Rc::clone(&show_help);

    let env_theme = Theme::from_env_or(Theme::nord());
    let theme = Rc::new(RefCell::new(env_theme.clone()));
    let theme_for_tick = Rc::clone(&theme);

    let mut router = SceneRouter::new();
    let initial_theme = theme.borrow().clone();
    router.register("home", Box::new(HomeScreen::new(initial_theme)));
    router.register("settings", Box::new(SettingsScreen::new(initial_theme)));
    router.register("profile", Box::new(ProfileScreen::new(initial_theme)));
    router.push("home");

    let router = Rc::new(RefCell::new(router));
    let router_for_input = Rc::clone(&router);

    let show_help_for_input = Rc::clone(&show_help);

    let keybindings = KeybindingSet::from_config(&resolve_keybindings());
    let keybindings_for_tick = keybindings.clone();
    let app_router = AppRouter::new(router_for_input, theme, should_quit, show_help_for_input, keybindings);

    let mut app = App::new()?
        .title("Scene Router Demo")
        .fps(30)
        .theme(Theme::from_env_or(Theme::nord()));

    app.add_widget(Box::new(app_router), Rect::new(0, 0, 80, 24));

    let _ = app
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
                return;
            }

            let mut router = router.borrow_mut();
            let theme = theme_for_tick.borrow().clone();

            if router.needs_render() {
                let (w, h) = ctx.compositor().size();
                let plane = router.render(Rect::new(0, 0, w, h));
                ctx.add_plane(plane);
                router.clear_dirty();
            }

            // Render help overlay on top of everything
            if *show_help_for_tick.borrow() {
                let (w, h) = ctx.compositor().size();
                let mut help_plane = dracon_terminal_engine::compositor::Plane::new(0, w, h);
                help_plane.z_index = 200;

                let kb = &keybindings_for_tick;
                let shortcuts = [
                    ("↑/↓", "Navigate menu"),
                    ("Enter", "Select / go forward"),
                    (kb.display(actions::BACK).unwrap_or("esc"), "Go back"),
                    (kb.display(actions::THEME).unwrap_or("t"), "Cycle theme"),
                    (kb.display(actions::HELP).unwrap_or("?"), "Toggle this help"),
                    (kb.display(actions::QUIT).unwrap_or("q"), "Quit"),
                ];

                let hw = 40u16.min(w.saturating_sub(4));
                let hh = (shortcuts.len() as u16 + 4).min(h.saturating_sub(4));
                let hx = (w - hw) / 2;
                let hy = (h - hh) / 2;

                for y in hy..hy + hh {
                    for x in hx..hx + hw {
                        let idx = (y * w + x) as usize;
                        if idx < help_plane.cells.len() {
                            help_plane.cells[idx].bg = theme.surface_elevated;
                            help_plane.cells[idx].transparent = false;
                        }
                    }
                }

                let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
                for (ch, cx, cy) in corners.iter() {
                    let idx = (cy * w + cx) as usize;
                    if idx < help_plane.cells.len() {
                        help_plane.cells[idx].char = *ch;
                        help_plane.cells[idx].fg = theme.outline;
                        help_plane.cells[idx].transparent = false;
                    }
                }
                for x in hx + 1..hx + hw - 1 {
                    let top = (hy * w + x) as usize;
                    let bot = ((hy + hh - 1) * w + x) as usize;
                    if top < help_plane.cells.len() {
                        help_plane.cells[top].char = '─';
                        help_plane.cells[top].fg = theme.outline;
                        help_plane.cells[top].transparent = false;
                    }
                    if bot < help_plane.cells.len() {
                        help_plane.cells[bot].char = '─';
                        help_plane.cells[bot].fg = theme.outline;
                        help_plane.cells[bot].transparent = false;
                    }
                }
                for y in hy + 1..hy + hh - 1 {
                    let left = (y * w + hx) as usize;
                    let right = (y * w + hx + hw - 1) as usize;
                    if left < help_plane.cells.len() {
                        help_plane.cells[left].char = '│';
                        help_plane.cells[left].fg = theme.outline;
                        help_plane.cells[left].transparent = false;
                    }
                    if right < help_plane.cells.len() {
                        help_plane.cells[right].char = '│';
                        help_plane.cells[right].fg = theme.outline;
                        help_plane.cells[right].transparent = false;
                    }
                }

                let title = "Scene Router Demo — Help";
                let tx = hx + (hw - title.len() as u16) / 2;
                for (i, c) in title.chars().enumerate() {
                    let idx = ((hy + 1) * w + tx + i as u16) as usize;
                    if idx < help_plane.cells.len() {
                        help_plane.cells[idx].char = c;
                        help_plane.cells[idx].fg = theme.primary;
                        help_plane.cells[idx].style = dracon_terminal_engine::compositor::Styles::BOLD;
                        help_plane.cells[idx].transparent = false;
                    }
                }

                for (i, (key, desc)) in shortcuts.iter().enumerate() {
                    let row = hy + 3 + i as u16;
                    for (j, c) in key.chars().enumerate() {
                        let idx = (row * w + hx + 2 + j as u16) as usize;
                        if idx < help_plane.cells.len() {
                            help_plane.cells[idx].char = c;
                            help_plane.cells[idx].fg = theme.primary;
                            help_plane.cells[idx].transparent = false;
                        }
                    }
                    for (j, c) in desc.chars().enumerate() {
                        let idx = (row * w + hx + 14 + j as u16) as usize;
                        if idx < help_plane.cells.len() {
                            help_plane.cells[idx].char = c;
                            help_plane.cells[idx].fg = theme.fg;
                            help_plane.cells[idx].transparent = false;
                        }
                    }
                }

                ctx.add_plane(help_plane);
            }
        })
        .run(|_| {});

    Ok(())
}

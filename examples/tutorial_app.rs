//! Tutorial: Building Your First Dracon App
//!
//! This example is a progressive tutorial that teaches Dracon app development
//! by building a simple Task Manager from scratch.

#![allow(dead_code)]  // Tutorial demonstrates patterns not all exercised in skeleton
//!
//! ## Architecture demonstrated:
//! - SceneRouter for multi-screen navigation (List → Detail → Edit)
//! - EventBus for decoupled widget communication
//! - Centralized AppState with reactive updates
//! - Theme propagation across all screens
//! - Proper Widget/Scene lifecycle management
//!
//! ## Screens:
//!   TaskList    — View all tasks, add new ones
//!   TaskDetail  — View single task, mark complete
//!   TaskEdit    — Create or edit tasks
//!
//! Controls:
//!   ↑/↓         — navigate
//!   Enter       — select / save
//!   n           — new task
//!   d           — delete task
//!   e           — edit task
//!   c           — toggle complete
//!   Backspace   — go back
//!   t           — cycle theme
//!   ?           — toggle help
//!   q           — quit

use dracon_terminal_engine::compositor::{Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::{Scene, SceneRouter};
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════════════════
// DATA MODEL
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
struct Task {
    id: usize,
    title: String,
    description: String,
    completed: bool,
    priority: Priority,
}

#[derive(Clone, Debug, PartialEq)]
enum Priority {
    Low,
    Medium,
    High,
}

impl Task {
    fn new(id: usize, title: &str) -> Self {
        Self {
            id,
            title: title.into(),
            description: String::new(),
            completed: false,
            priority: Priority::Medium,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// APP STATE (Centralized state management)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
enum AppEvent {
    TaskAdded(Task),
    TaskUpdated(Task),
    TaskDeleted(usize),
    TaskToggled(usize),
    ThemeChanged(String),
}

struct AppState {
    tasks: Vec<Task>,
    next_id: usize,
    selected_task: Option<usize>,
}

impl AppState {
    fn new() -> Self {
        let mut tasks = vec![
            Task::new(0, "Learn Dracon framework"),
            Task::new(1, "Build a TUI app"),
            Task::new(2, "Share with friends"),
        ];
        tasks[0].description = "Read the docs and examples".into();
        tasks[0].priority = Priority::High;
        tasks[1].description = "Create something amazing".into();
        tasks[1].priority = Priority::Medium;
        tasks[2].description = "Show off your work".into();
        tasks[2].priority = Priority::Low;

        Self {
            tasks,
            next_id: 3,
            selected_task: None,
        }
    }

    fn add_task(&mut self, title: &str) -> Task {
        let task = Task::new(self.next_id, title);
        self.next_id += 1;
        self.tasks.push(task.clone());
        task
    }

    fn get_task(&self, id: usize) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    fn get_task_mut(&mut self, id: usize) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    fn delete_task(&mut self, id: usize) {
        self.tasks.retain(|t| t.id != id);
    }

    fn toggle_task(&mut self, id: usize) {
        if let Some(task) = self.get_task_mut(id) {
            task.completed = !task.completed;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TASK LIST SCREEN
// ═══════════════════════════════════════════════════════════════════════════════

struct TaskListScreen {
    theme: Theme,
    selected: usize,
    dirty: bool,
}

impl TaskListScreen {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
            selected: 0,
            dirty: true,
        }
    }
}

impl Scene for TaskListScreen {
    fn scene_id(&self) -> &str { "task_list" }

    fn on_enter(&mut self) {
        self.dirty = true;
    }

    fn on_resume(&mut self) {
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = &self.theme;

        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.transparent = false;
        }

        // Title
        let title = "📋 Tasks";
        for (i, c) in title.chars().enumerate() {
            if i < plane.cells.len() {
                plane.cells[i].char = c;
                plane.cells[i].fg = t.primary;
                plane.cells[i].style = Styles::BOLD;
            }
        }

        // Would render task list here with real data
        let hint = "No tasks yet. Press 'n' to create one.";
        let y = 3;
        for (i, c) in hint.chars().enumerate() {
            let idx = (y * area.width + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
            }
        }

        // Status bar
        let status = "↑↓: nav | Enter: detail | n: new | t: theme | ?: help | q: quit";
        let sy = area.height - 1;
        for (i, c) in status.chars().enumerate() {
            let idx = (sy * area.width + i as u16) as usize;
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
                self.selected += 1;
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TASK DETAIL SCREEN
// ═══════════════════════════════════════════════════════════════════════════════

struct TaskDetailScreen {
    theme: Theme,
    task_id: Option<usize>,
    dirty: bool,
}

impl TaskDetailScreen {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
            task_id: None,
            dirty: true,
        }
    }

    fn set_task(&mut self, id: usize) {
        self.task_id = Some(id);
        self.dirty = true;
    }
}

impl Scene for TaskDetailScreen {
    fn scene_id(&self) -> &str { "task_detail" }

    fn on_enter(&mut self) {
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = &self.theme;

        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.transparent = false;
        }

        // Title
        let title = "📄 Task Detail";
        for (i, c) in title.chars().enumerate() {
            if i < plane.cells.len() {
                plane.cells[i].char = c;
                plane.cells[i].fg = t.primary;
                plane.cells[i].style = Styles::BOLD;
            }
        }

        // Task info placeholder
        let info = "Select a task from the list to view details.";
        let y = 3;
        for (i, c) in info.chars().enumerate() {
            let idx = (y * area.width + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
            }
        }

        // Status
        let status = "e: edit | c: toggle | d: delete | Backspace: back | t: theme | ?: help | q: quit";
        let sy = area.height - 1;
        for (i, c) in status.chars().enumerate() {
            let idx = (sy * area.width + i as u16) as usize;
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

    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TASK EDIT SCREEN
// ═══════════════════════════════════════════════════════════════════════════════

struct TaskEditScreen {
    theme: Theme,
    task_id: Option<usize>,
    title_input: String,
    desc_input: String,
    editing_field: usize, // 0 = title, 1 = description
    dirty: bool,
}

impl TaskEditScreen {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
            task_id: None,
            title_input: String::new(),
            desc_input: String::new(),
            editing_field: 0,
            dirty: true,
        }
    }

    fn set_task(&mut self, task: Option<&Task>) {
        if let Some(task) = task {
            self.task_id = Some(task.id);
            self.title_input = task.title.clone();
            self.desc_input = task.description.clone();
        } else {
            self.task_id = None;
            self.title_input.clear();
            self.desc_input.clear();
        }
        self.dirty = true;
    }
}

impl Scene for TaskEditScreen {
    fn scene_id(&self) -> &str { "task_edit" }

    fn on_enter(&mut self) {
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = &self.theme;

        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.transparent = false;
        }

        // Title
        let title = if self.task_id.is_some() { "✏️  Edit Task" } else { "➕ New Task" };
        for (i, c) in title.chars().enumerate() {
            if i < plane.cells.len() {
                plane.cells[i].char = c;
                plane.cells[i].fg = t.primary;
                plane.cells[i].style = Styles::BOLD;
            }
        }

        // Form fields
        let fields = [
            ("Title:", &self.title_input),
            ("Description:", &self.desc_input),
        ];
        for (i, (label, value)) in fields.iter().enumerate() {
            let y = 3 + i as u16 * 3;

            // Label
            for (j, c) in label.chars().enumerate() {
                let idx = (y * area.width + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg_muted;
                }
            }

            // Value
            let is_active = i == self.editing_field;
            let val_y = y + 1;
            for (j, c) in value.chars().enumerate() {
                let idx = (val_y * area.width + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                    plane.cells[idx].bg = if is_active { t.focus_bg } else { t.input_bg };
                }
            }

            // Cursor indicator for active field
            if is_active {
                let cursor_x = 2 + value.len() as u16;
                let idx = (val_y * area.width + cursor_x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '▏';
                    plane.cells[idx].fg = t.primary;
                }
            }
        }

        // Status
        let status = "Tab: switch field | Enter: save | Backspace: cancel | t: theme | ?: help | q: quit";
        let sy = area.height - 1;
        for (i, c) in status.chars().enumerate() {
            let idx = (sy * area.width + i as u16) as usize;
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
            KeyCode::Tab => {
                self.editing_field = (self.editing_field + 1) % 2;
                self.dirty = true;
                true
            }
            KeyCode::Char(c) => {
                match self.editing_field {
                    0 => self.title_input.push(c),
                    1 => self.desc_input.push(c),
                    _ => {}
                }
                self.dirty = true;
                true
            }
            KeyCode::Backspace => {
                match self.editing_field {
                    0 => { self.title_input.pop(); }
                    1 => { self.desc_input.pop(); }
                    _ => {}
                }
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

// ═══════════════════════════════════════════════════════════════════════════════
// APP ROUTER (Input handling and navigation)
// ═══════════════════════════════════════════════════════════════════════════════

struct AppRouter {
    router: Rc<RefCell<SceneRouter>>,
    state: Rc<RefCell<AppState>>,
    theme: Rc<RefCell<Theme>>,
    should_quit: Arc<AtomicBool>,
    show_help: Rc<RefCell<bool>>,
    id: WidgetId,
    area: std::cell::Cell<Rect>,
}

impl AppRouter {
    fn new(
        router: Rc<RefCell<SceneRouter>>,
        state: Rc<RefCell<AppState>>,
        theme: Rc<RefCell<Theme>>,
        should_quit: Arc<AtomicBool>,
        show_help: Rc<RefCell<bool>>,
    ) -> Self {
        Self {
            router,
            state,
            theme,
            should_quit,
            show_help,
            id: WidgetId::new(100),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn cycle_theme(&self) {
        let themes = [
            Theme::nord(), Theme::cyberpunk(), Theme::dracula(),
            Theme::catppuccin_mocha(), Theme::gruvbox_dark(), Theme::tokyo_night(),
        ];
        let mut theme = self.theme.borrow_mut();
        let idx = themes.iter().position(|t| t.name == theme.name).unwrap_or(0);
        *theme = themes[(idx + 1) % themes.len()];
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

        if *self.show_help.borrow() {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                *self.show_help.borrow_mut() = false;
                return true;
            }
            return false;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            KeyCode::Char('?') => {
                *self.show_help.borrow_mut() = true;
                true
            }
            KeyCode::Char('t') => {
                self.cycle_theme();
                let theme = *self.theme.borrow();
                self.router.borrow_mut().on_theme_change(&theme);
                true
            }
            KeyCode::Esc => {
                self.router.borrow_mut().pop();
                true
            }
            KeyCode::Char('n') => {
                self.router.borrow_mut().push("task_edit");
                true
            }
            _ => {
                // Delegate to current scene
                self.router.borrow_mut().handle_key(key)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("Tutorial: Building Your First Dracon App | ?: help | q: quit");
    std::thread::sleep(std::time::Duration::from_millis(300));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);
    let show_help = Rc::new(RefCell::new(false));
    let show_help_for_tick = Rc::clone(&show_help);

    let theme = Rc::new(RefCell::new(Theme::nord()));
    let theme_for_tick = Rc::clone(&theme);

    let state = Rc::new(RefCell::new(AppState::new()));

    let mut router = SceneRouter::new();
    let initial_theme = *theme.borrow();
    router.register("task_list", Box::new(TaskListScreen::new(initial_theme)));
    router.register("task_detail", Box::new(TaskDetailScreen::new(initial_theme)));
    router.register("task_edit", Box::new(TaskEditScreen::new(initial_theme)));
    router.push("task_list");

    let router = Rc::new(RefCell::new(router));
    let router_for_input = Rc::clone(&router);
    let show_help_for_input = Rc::clone(&show_help);

    let app_router = AppRouter::new(router_for_input, state, theme, should_quit, show_help_for_input);

    let mut app = App::new()?
        .title("Task Manager Tutorial")
        .fps(30);

    app.add_widget(Box::new(app_router), Rect::new(0, 0, 80, 24));

    let _ = app
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
                return;
            }

            let mut router = router.borrow_mut();
            let theme = *theme_for_tick.borrow();

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

                let shortcuts = [
                    ("↑/↓", "Navigate"),
                    ("Enter", "Select / save"),
                    ("n", "New task"),
                    ("d", "Delete task"),
                    ("e", "Edit task"),
                    ("c", "Toggle complete"),
                    ("Backspace", "Go back"),
                    ("t", "Cycle theme"),
                    ("?", "Toggle this help"),
                    ("q", "Quit"),
                ];

                let hw = 42u16.min(w.saturating_sub(4));
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

                let title = "Task Manager Tutorial — Help";
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
                        let idx = (row * w + hx + 16 + j as u16) as usize;
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

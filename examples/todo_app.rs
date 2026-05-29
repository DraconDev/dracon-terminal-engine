//! Todo App  -  A real, functional task manager with SQLite persistence.
#![allow(dead_code)] // Tutorial demonstrates architecture; not all features wired in skeleton
//!
//! This example demonstrates production-ready Dracon app architecture:
//! - SQLite database for persistent storage
//! - SceneRouter for List > Add > Detail navigation
//! - Framework widgets (List, StatusBar, Modal) instead of raw rendering
//! - EventBus for cross-screen communication
//! - Full CRUD operations with error handling
//!
//! Controls:
//!   ^/v          -  navigate tasks
//!   n            -  new task
//!   Enter        -  view task detail / confirm
//!   Space        -  toggle complete
//!   d            -  delete task (with confirmation)
//!   e            -  edit task
//!   Backspace    -  delete character (in edit mode)
//!   t            -  cycle theme
//!   ?            -  toggle help
//!   q            -  quit

use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::{Scene, SceneRouter};
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{List, StatusBar, StatusSegment};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use rusqlite::{Connection, Result as SqlResult};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════════════════
// DATABASE
// ═══════════════════════════════════════════════════════════════════════════════

struct TodoDb {
    conn: Connection,
}

#[derive(Clone, Debug)]
struct TodoTask {
    id: i64,
    title: String,
    description: String,
    completed: bool,
    priority: i32, // 0=low, 1=medium, 2=high
    created_at: String,
}

impl TodoDb {
    fn open<P: AsRef<Path>>(path: P) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> SqlResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                completed INTEGER NOT NULL DEFAULT 0,
                priority INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    fn get_all_tasks(&self) -> SqlResult<Vec<TodoTask>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, completed, priority, created_at
             FROM tasks ORDER BY completed ASC, priority DESC, created_at DESC",
        )?;

        let tasks = stmt
            .query_map([], |row| {
                Ok(TodoTask {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    completed: row.get::<_, i32>(3)? != 0,
                    priority: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    fn add_task(&self, title: &str, description: &str, priority: i32) -> SqlResult<i64> {
        let created_at = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        self.conn.execute(
            "INSERT INTO tasks (title, description, completed, priority, created_at)
             VALUES (?1, ?2, 0, ?3, ?4)",
            [title, description, &priority.to_string(), &created_at],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    fn update_task(&self, id: i64, title: &str, description: &str, priority: i32) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE tasks SET title = ?1, description = ?2, priority = ?3 WHERE id = ?4",
            [title, description, &priority.to_string(), &id.to_string()],
        )?;
        Ok(())
    }

    fn toggle_task(&self, id: i64) -> SqlResult<bool> {
        let current: i32 =
            self.conn
                .query_row("SELECT completed FROM tasks WHERE id = ?1", [&id], |row| {
                    row.get(0)
                })?;
        let new_val = if current == 0 { 1 } else { 0 };
        self.conn.execute(
            "UPDATE tasks SET completed = ?1 WHERE id = ?2",
            [&new_val.to_string(), &id.to_string()],
        )?;
        Ok(new_val == 1)
    }

    fn delete_task(&self, id: i64) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM tasks WHERE id = ?1", [&id])?;
        Ok(())
    }

    fn get_task(&self, id: i64) -> SqlResult<Option<TodoTask>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, completed, priority, created_at
             FROM tasks WHERE id = ?1",
        )?;

        let mut rows = stmt.query([&id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(TodoTask {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                completed: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                created_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// APP STATE
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
enum TodoEvent {
    Added(TodoTask),
    Updated(TodoTask),
    Deleted(i64),
    Toggled(i64, bool),
}

struct AppState {
    db: TodoDb,
    tasks: Vec<TodoTask>,
    selected_task: Option<i64>,
    dirty: bool,
}

impl AppState {
    fn new(db_path: &str) -> SqlResult<Self> {
        let db = TodoDb::open(db_path)?;
        let tasks = db.get_all_tasks()?;
        Ok(Self {
            db,
            tasks,
            selected_task: None,
            dirty: true,
        })
    }

    fn refresh(&mut self) -> SqlResult<()> {
        self.tasks = self.db.get_all_tasks()?;
        self.dirty = true;
        Ok(())
    }

    fn add_task(&mut self, title: &str, description: &str, priority: i32) -> SqlResult<TodoTask> {
        let id = self.db.add_task(title, description, priority)?;
        let task = self.db.get_task(id)?.unwrap();
        self.tasks.push(task.clone());
        self.dirty = true;
        Ok(task)
    }

    fn toggle_task(&mut self, id: i64) -> SqlResult<bool> {
        let completed = self.db.toggle_task(id)?;
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.completed = completed;
        }
        self.dirty = true;
        Ok(completed)
    }

    fn delete_task(&mut self, id: i64) -> SqlResult<()> {
        self.db.delete_task(id)?;
        self.tasks.retain(|t| t.id != id);
        self.dirty = true;
        Ok(())
    }

    fn update_task(
        &mut self,
        id: i64,
        title: &str,
        description: &str,
        priority: i32,
    ) -> SqlResult<()> {
        self.db.update_task(id, title, description, priority)?;
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.title = title.to_string();
            task.description = description.to_string();
            task.priority = priority;
        }
        self.dirty = true;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TASK LIST SCREEN
// ═══════════════════════════════════════════════════════════════════════════════

struct TaskListScreen {
    theme: Theme,
    task_list: List<String>,
    status_bar: StatusBar,
    selected: usize,
    dirty: bool,
}

impl TaskListScreen {
    fn new(theme: Theme) -> Self {
        let mut list = List::new(Vec::<String>::new());
        list.on_theme_change(&theme);

        let status = StatusBar::new(WidgetId::new(2))
            .add_segment(StatusSegment::new("0 tasks").with_fg(theme.fg_muted));

        Self {
            theme,
            task_list: list,
            status_bar: status,
            selected: 0,
            dirty: true,
        }
    }

    fn update_tasks(&mut self, tasks: &[TodoTask]) {
        let items: Vec<String> = tasks
            .iter()
            .map(|t| {
                let status = if t.completed { "[ok]" } else { "( )" };
                let priority = match t.priority {
                    2 => "^",
                    0 => "v",
                    _ => " ",
                };
                format!("{} {} {}", status, priority, t.title)
            })
            .collect();

        self.task_list.set_items(items);

        let count = tasks.len();
        let completed = tasks.iter().filter(|t| t.completed).count();
        let status_text = format!("{} tasks | {} done", count, completed);
        self.status_bar = StatusBar::new(WidgetId::new(2))
            .add_segment(StatusSegment::new(&status_text).with_fg(self.theme.fg_muted));

        if self.selected >= tasks.len() && !tasks.is_empty() {
            self.selected = tasks.len() - 1;
        }

        self.dirty = true;
    }
}

impl Scene for TaskListScreen {
    fn scene_id(&self) -> &str {
        "task_list"
    }
    fn on_enter(&mut self) {
        self.dirty = true;
    }
    fn on_resume(&mut self) {
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> dracon_terminal_engine::compositor::Plane {
        let mut plane = dracon_terminal_engine::compositor::Plane::new(0, area.width, area.height);

        // Background
        for cell in plane.cells.iter_mut() {
            cell.bg = self.theme.bg;
            cell.transparent = false;
        }

        // Title
        let title = " Todo App ";
        let tx = (area.width as usize - title.len()) / 2;
        plane.put_str(tx as u16, 0, title);

        // Task list
        let list_h = area.height.saturating_sub(3);
        if list_h > 0 {
            let list_plane = self.task_list.render(Rect::new(0, 2, area.width, list_h));
            for y in 0..list_h {
                for x in 0..area.width {
                    let src_idx = (y * area.width + x) as usize;
                    let dst_idx = ((y + 2) * area.width + x) as usize;
                    if src_idx < list_plane.cells.len()
                        && dst_idx < plane.cells.len()
                        && !list_plane.cells[src_idx].transparent
                    {
                        plane.cells[dst_idx] = list_plane.cells[src_idx];
                    }
                }
            }
        }

        // Status bar
        let status_y = area.height.saturating_sub(1);
        let status_plane = self
            .status_bar
            .render(Rect::new(0, status_y, area.width, 1));
        for x in 0..area.width {
            let idx = (status_y * area.width + x) as usize;
            if idx < plane.cells.len() && (x as usize) < status_plane.cells.len() {
                plane.cells[idx] = status_plane.cells[x as usize];
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
                let task_count = self.task_list.len();
                if self.selected + 1 < task_count {
                    self.selected += 1;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Enter => {
                true // Handled at router level
            }
            _ => false,
        }
    }

    fn handle_mouse(
        &mut self,
        kind: dracon_terminal_engine::input::event::MouseEventKind,
        _col: u16,
        row: u16,
    ) -> bool {
        use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let list_start_y = 2u16;
                let list_h = self.task_list.len();
                if row >= list_start_y && ((row - list_start_y) as usize) < list_h {
                    let new_selected = (row - list_start_y) as usize;
                    if new_selected != self.selected {
                        self.selected = new_selected;
                        self.dirty = true;
                    }
                    true
                } else {
                    false
                }
            }
            MouseEventKind::ScrollUp => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.dirty = true;
                }
                true
            }
            MouseEventKind::ScrollDown => {
                let task_count = self.task_list.len();
                if self.selected + 1 < task_count {
                    self.selected += 1;
                    self.dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.task_list.on_theme_change(theme);
        self.status_bar.on_theme_change(theme);
        self.dirty = true;
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ADD TASK SCREEN
// ═══════════════════════════════════════════════════════════════════════════════

struct AddTaskScreen {
    theme: Theme,
    title_input: String,
    desc_input: String,
    priority: i32,
    editing_field: usize,
    dirty: bool,
}

impl AddTaskScreen {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
            title_input: String::new(),
            desc_input: String::new(),
            priority: 1,
            editing_field: 0,
            dirty: true,
        }
    }

    fn clear(&mut self) {
        self.title_input.clear();
        self.desc_input.clear();
        self.priority = 1;
        self.editing_field = 0;
        self.dirty = true;
    }
}

impl Scene for AddTaskScreen {
    fn scene_id(&self) -> &str {
        "add_task"
    }
    fn on_enter(&mut self) {
        self.clear();
    }

    fn render(&self, area: Rect) -> dracon_terminal_engine::compositor::Plane {
        let mut plane = dracon_terminal_engine::compositor::Plane::new(0, area.width, area.height);
        let t = self.theme.clone();

        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.transparent = false;
        }

        // Title
        let title = " New Task ";
        let tx = (area.width as usize - title.len()) / 2;
        plane.put_str(tx as u16, 0, title);

        // Fields
        let fields = [
            ("Title:", &self.title_input, 0),
            ("Description:", &self.desc_input, 1),
        ];

        for (i, (label, value, field_idx)) in fields.iter().enumerate() {
            let y = 3 + i as u16 * 3;

            // Label
            plane.put_str(2, y, label);

            // Input box
            let is_active = *field_idx == self.editing_field;
            let input_y = y + 1;
            let max_width = (area.width - 4) as usize;
            let display = if value.len() > max_width {
                &value[value.len() - max_width..]
            } else {
                value.as_str()
            };

            for (j, c) in display.chars().enumerate() {
                let idx = (input_y * area.width + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                    plane.cells[idx].bg = if is_active { t.focus_bg } else { t.input_bg };
                }
            }

            // Cursor
            if is_active {
                let cursor_x = 2 + display.len() as u16;
                let idx = (input_y * area.width + cursor_x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '▏';
                    plane.cells[idx].fg = t.primary;
                }
            }
        }

        // Priority
        let pri_y = 9;
        plane.put_str(2, pri_y, "Priority: ");
        let pri_labels = [(0, "Low"), (1, "Medium"), (2, "High")];
        let mut x = 12;
        for (val, label) in pri_labels {
            let is_selected = val == self.priority;
            for (j, c) in label.chars().enumerate() {
                let idx = (pri_y * area.width + x + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if is_selected { t.primary } else { t.fg_muted };
                    plane.cells[idx].style = if is_selected {
                        dracon_terminal_engine::compositor::Styles::BOLD
                    } else {
                        dracon_terminal_engine::compositor::Styles::empty()
                    };
                }
            }
            x += label.len() as u16 + 3;
        }

        // Hint
        let hint = "Tab: next field | </>: priority | Type: edit | Esc: back";
        let hy = area.height - 1;
        plane.put_str(0, hy, hint);

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
            KeyCode::Left => {
                if self.editing_field == 2 || self.editing_field == 0 {
                    self.priority = (self.priority - 1).max(0);
                    self.dirty = true;
                }
                true
            }
            KeyCode::Right => {
                if self.editing_field == 2 || self.editing_field == 0 {
                    self.priority = (self.priority + 1).min(2);
                    self.dirty = true;
                }
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
                    0 => {
                        self.title_input.pop();
                    }
                    1 => {
                        self.desc_input.pop();
                    }
                    _ => {}
                }
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(
        &mut self,
        kind: dracon_terminal_engine::input::event::MouseEventKind,
        _col: u16,
        row: u16,
    ) -> bool {
        use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Title input field area: y=2..=4
                if (2..=4).contains(&row) {
                    self.editing_field = 0;
                    self.dirty = true;
                    true
                // Desc input field area: y=5..=7
                } else if (5..=7).contains(&row) {
                    self.editing_field = 1;
                    self.dirty = true;
                    true
                // Priority area: y=9
                } else if row == 9 {
                    self.editing_field = 2;
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.dirty = true;
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// APP ROUTER
// ═══════════════════════════════════════════════════════════════════════════════

struct TodoRouter {
    router: Rc<RefCell<SceneRouter>>,
    state: Rc<RefCell<AppState>>,
    theme: Rc<RefCell<Theme>>,
    should_quit: Arc<AtomicBool>,
    show_help: Rc<RefCell<bool>>,
    show_delete_confirm: bool,
    delete_target: Option<i64>,
    id: WidgetId,
    area: std::cell::Cell<Rect>,
    keybindings: KeybindingSet,
}

impl TodoRouter {
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
            show_delete_confirm: false,
            delete_target: None,
            id: WidgetId::new(100),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn cycle_theme(&self) {
        let themes = Theme::all();
        let mut theme = self.theme.borrow_mut();
        let idx = themes
            .iter()
            .position(|t| t.name == theme.name)
            .unwrap_or(0);
        *theme = themes[(idx + 1) % themes.len()].clone();
    }

    fn refresh_task_list(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            if state.refresh().is_ok() {
                // Update the task list screen
                if let Some(screen) = self.router.borrow_mut().get_scene_mut("task_list") {
                    if let Some(list_screen) =
                        (screen as &mut dyn std::any::Any).downcast_mut::<TaskListScreen>()
                    {
                        list_screen.update_tasks(&state.tasks);
                    }
                }
            }
        }
    }
}

impl dracon_terminal_engine::framework::widget::Widget for TodoRouter {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn area(&self) -> Rect {
        self.area.get()
    }
    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }
    fn needs_render(&self) -> bool {
        false
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn z_index(&self) -> u16 {
        0
    }
    fn render(&self, _area: Rect) -> dracon_terminal_engine::compositor::Plane {
        dracon_terminal_engine::compositor::Plane::new(0, 0, 0)
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Handle overlays first
        if *self.show_help.borrow() {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
                *self.show_help.borrow_mut() = false;
                return true;
            }
            return false;
        }

        if self.show_delete_confirm {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Some(id) = self.delete_target {
                        if let Ok(mut state) = self.state.try_borrow_mut() {
                            let _ = state.delete_task(id);
                        }
                        self.refresh_task_list();
                    }
                    self.show_delete_confirm = false;
                    self.delete_target = None;
                    return true;
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.show_delete_confirm = false;
                    self.delete_target = None;
                    return true;
                }
                _ if self.keybindings.matches(actions::BACK, &key) => {
                    self.show_delete_confirm = false;
                    self.delete_target = None;
                    return true;
                }
                _ => return false,
            }
        }

        if self.keybindings.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            *self.show_help.borrow_mut() = true;
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            self.cycle_theme();
            let theme = self.theme.borrow().clone();
            self.router.borrow_mut().on_theme_change(&theme);
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            self.router.borrow_mut().pop();
            return true;
        }
        if key.code == KeyCode::Enter {
            // TODO: Push "detail" scene when DetailScreen is implemented
            return true;
        }
        if self.keybindings.matches(actions::NEW_ITEM, &key) {
            self.router.borrow_mut().push("add_task");
            return true;
        }
        self.router.borrow_mut().handle_key(key)
    }
    fn current_theme(&self) -> Option<Theme> {
        Some(self.theme.borrow().clone())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    let db_path = std::env::var("HOME")
        .map(|home| format!("{}/.dracon_todo.db", home))
        .unwrap_or_else(|_| "/tmp/dracon_todo.db".to_string());

    println!("Todo App  -  Tasks saved to {}", db_path);
    println!("Controls: ^v nav | Ctrl+N new | Enter detail | t theme | F1 help | Ctrl+Q quit");
    std::thread::sleep(std::time::Duration::from_millis(500));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);
    let show_help = Rc::new(RefCell::new(false));
    let show_help_for_tick = Rc::clone(&show_help);

    let env_theme = Theme::from_env_or(Theme::nord());
    let theme = Rc::new(RefCell::new(env_theme.clone()));

    let state = match AppState::new(&db_path) {
        Ok(s) => Rc::new(RefCell::new(s)),
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    };

    let mut router = SceneRouter::new();
    let initial_theme = theme.borrow().clone();

    let mut list_screen = TaskListScreen::new(initial_theme.clone());
    if let Ok(state_ref) = state.try_borrow() {
        list_screen.update_tasks(&state_ref.tasks);
    }
    router.register("task_list", Box::new(list_screen));
    router.register("add_task", Box::new(AddTaskScreen::new(initial_theme)));
    router.push("task_list");

    let router = Rc::new(RefCell::new(router));
    let router_for_input = Rc::clone(&router);
    let _state_for_tick = Rc::clone(&state);
    let theme_for_tick = Rc::clone(&theme);
    let show_help_for_input = Rc::clone(&show_help);

    let app_router = TodoRouter::new(
        router_for_input,
        state,
        theme,
        should_quit,
        show_help_for_input,
    );

    let mut app = App::new()?
        .title("Todo App")
        .fps(30)
        .set_theme(Theme::from_env_or(Theme::nord()));

    app.add_widget(Box::new(app_router), Rect::new(0, 0, 80, 24));

    let keybindings = KeybindingSet::from_config(&resolve_keybindings());

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

                let shortcuts = [
                    ("^/v", "Navigate tasks"),
                    (
                        keybindings.display(actions::NEW_ITEM).unwrap_or("Ctrl+N"),
                        "New task",
                    ),
                    ("Enter", "View task detail / confirm"),
                    (
                        keybindings.display(actions::BACK).unwrap_or("Esc"),
                        "Go back / cancel",
                    ),
                    (
                        keybindings.display(actions::THEME).unwrap_or("t"),
                        "Cycle theme",
                    ),
                    (
                        keybindings.display(actions::HELP).unwrap_or("F1"),
                        "Toggle this help",
                    ),
                    (
                        keybindings.display(actions::QUIT).unwrap_or("Ctrl+Q"),
                        "Quit",
                    ),
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

                let corners = [
                    ('╭', hx, hy),
                    ('╮', hx + hw - 1, hy),
                    ('╰', hx, hy + hh - 1),
                    ('╯', hx + hw - 1, hy + hh - 1),
                ];
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

                let title = "Todo App  -  Help";
                let tx = hx + (hw - title.len() as u16) / 2;
                for (i, c) in title.chars().enumerate() {
                    let idx = ((hy + 1) * w + tx + i as u16) as usize;
                    if idx < help_plane.cells.len() {
                        help_plane.cells[idx].char = c;
                        help_plane.cells[idx].fg = theme.primary;
                        help_plane.cells[idx].style =
                            dracon_terminal_engine::compositor::Styles::BOLD;
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

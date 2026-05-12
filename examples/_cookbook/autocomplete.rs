use dracon_terminal_engine::framework::prelude::*;
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

struct AutocompleteDemo {
    id: WidgetId,
    area: Rect,
    should_quit: Rc<AtomicBool>,
    theme: Theme,
    autocomplete: Autocomplete,
    selected: RefCell<Option<String>>,
    dirty: bool,
}

impl AutocompleteDemo {
    fn new(should_quit: Rc<AtomicBool>, theme: Theme) -> Self {
        let suggestions = vec![
            "apple".to_string(),
            "apricot".to_string(),
            "avocado".to_string(),
            "banana".to_string(),
            "blueberry".to_string(),
            "cherry".to_string(),
            "coconut".to_string(),
            "cranberry".to_string(),
            "date".to_string(),
            "dragonfruit".to_string(),
            "elderberry".to_string(),
            "fig".to_string(),
            "grape".to_string(),
            "grapefruit".to_string(),
            "guava".to_string(),
            "kiwi".to_string(),
            "lemon".to_string(),
            "lime".to_string(),
            "lychee".to_string(),
            "mango".to_string(),
            "melon".to_string(),
            "orange".to_string(),
            "papaya".to_string(),
            "peach".to_string(),
            "pear".to_string(),
            "pineapple".to_string(),
            "plum".to_string(),
            "pomegranate".to_string(),
            "raspberry".to_string(),
            "strawberry".to_string(),
            "tangerine".to_string(),
            "watermelon".to_string(),
        ];

        let selected = RefCell::new(None);
        let sel = selected.clone();
        let mut autocomplete = Autocomplete::new(WidgetId::new(2), suggestions);
        autocomplete = autocomplete
            .with_theme(theme)
            .with_max_visible(8)
            .on_select(move |s| {
                *sel.borrow_mut() = Some(s.to_string());
            });

        Self {
            id: WidgetId::new(1),
            area: Rect::default(),
            should_quit,
            theme,
            autocomplete,
            selected,
            dirty: true,
        }
    }
}

impl Widget for AutocompleteDemo {
    fn needs_render(&self) -> bool {
        self.dirty || self.autocomplete.needs_render()
    }

    fn id(&self) -> WidgetId { self.id }

    fn area(&self) -> Rect { self.area }

    fn set_area(&mut self, area: Rect) { self.area = area; }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        // Title
        let title = "Autocomplete Demo — Type to filter fruits";
        let tx = (area.width - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (1 * area.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Selected value display
        if let Some(ref val) = *self.selected.borrow() {
            let label = format!("Selected: {}", val);
            let lx = 4;
            let ly = area.height.saturating_sub(3);
            for (i, c) in label.chars().enumerate() {
                let idx = (ly * area.width + lx + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = self.theme.success;
                }
            }
        }

        // Status bar
        let status = "Ctrl+Q: quit | Type to filter | ↑/↓ navigate | Enter select | Tab insert top";
        let sy = area.height.saturating_sub(1);
        for (i, c) in status.chars().enumerate() {
            let idx = (sy * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
                plane.cells[idx].bg = self.theme.surface;
                plane.cells[idx].transparent = false;
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        use dracon_terminal_engine::input::event::KeyCode;
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            _ => {
                let handled = self.autocomplete.handle_key(key);
                if handled && self.autocomplete.selected().is_some() {
                    self.dirty = true;
                }
                handled
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.autocomplete.handle_mouse(kind, col, row)
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.autocomplete.on_theme_change(theme);
        self.dirty = true;
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Rc::new(AtomicBool::new(false));
    let theme = Theme::from_env_or(Theme::nord());
    let demo = AutocompleteDemo::new(Rc::clone(&should_quit), theme);

    let mut app = App::new()?;
    app.add_widget(Box::new(demo), Rect::new(0, 0, 80, 24));

    let q = should_quit;
    app.title("Autocomplete Demo")
        .fps(30)
        .theme(theme)
        .run(move |ctx| {
            if q.load(Ordering::SeqCst) {
                ctx.stop();
            }
        })
}

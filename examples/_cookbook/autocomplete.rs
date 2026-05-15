use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

struct AutocompleteDemo {
    id: WidgetId,
    area: Rect,
    should_quit: Rc<AtomicBool>,
    theme: Theme,
    keybindings: KeybindingSet,
    autocomplete: Autocomplete,
    selected: RefCell<Option<String>>,
    show_help: bool,
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
            .with_theme(theme.clone())
            .with_max_visible(8)
            .on_select(move |s| {
                *sel.borrow_mut() = Some(s.to_string());
            });

        Self {
            id: WidgetId::new(1),
            area: Rect::default(),
            should_quit,
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            autocomplete,
            selected,
            show_help: false,
            dirty: true,
        }
    }
    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.autocomplete.on_theme_change(&self.theme);
        self.dirty = true;
    }
}

impl Widget for AutocompleteDemo {
    fn needs_render(&self) -> bool {
        self.dirty || self.autocomplete.needs_render()
    }

    fn id(&self) -> WidgetId { self.id }

    fn area(&self) -> Rect { self.area }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
        self.autocomplete.set_area(Rect::new(4, 4, area.width.saturating_sub(8), 3));
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        // Title
        let title = "Autocomplete Demo — Type to filter fruits";
        let tx = (area.width - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (area.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Render the autocomplete widget
        let auto_area = Rect::new(4, 4, area.width.saturating_sub(8), 3);
        let auto_plane = self.autocomplete.render(auto_area);
        for y in 0..auto_plane.height {
            for x in 0..auto_plane.width {
                let src_idx = (y * auto_plane.width + x) as usize;
                let dst_idx = ((auto_area.y + y) * area.width + (auto_area.x + x)) as usize;
                if src_idx < auto_plane.cells.len() && dst_idx < plane.cells.len() {
                    let src = &auto_plane.cells[src_idx];
                    if !src.transparent {
                        plane.cells[dst_idx] = auto_plane.cells[src_idx];
                    }
                }
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
        let status = self.keybindings.format_hint(&[
            (actions::QUIT, "quit"),
            (actions::HELP, "help"),
            (actions::THEME, "theme"),
        ]);
        let extra = " | Type to filter | ^/v navigate | Enter select | Tab insert top";
        let full_status = format!("{}{}", status, extra);
        let sy = area.height.saturating_sub(1);
        for (i, c) in full_status.chars().enumerate() {
            let idx = (sy * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
                plane.cells[idx].bg = self.theme.surface;
                plane.cells[idx].transparent = false;
            }
        }

        // Help overlay
        if self.show_help {
            let t = &self.theme;
            let hw = 44u16.min(area.width.saturating_sub(4));
            let hh = 10u16.min(area.height.saturating_sub(4));
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

            let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = *ch; plane.cells[idx].fg = t.outline; }
            }
            for x in hx + 1..hx + hw - 1 {
                let top = (hy * area.width + x) as usize;
                let bot = ((hy + hh - 1) * area.width + x) as usize;
                if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
                if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
            }
            for y in hy + 1..hy + hh - 1 {
                let left = (y * area.width + hx) as usize;
                let right = (y * area.width + hx + hw - 1) as usize;
                if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
                if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
            }

            let title = "Autocomplete Help";
            let tx = hx + (hw - title.len() as u16) / 2;
            for (i, c) in title.chars().enumerate() {
                let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }

            let quit_key = self.keybindings.display(actions::QUIT).unwrap_or("ctrl+q");
            let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            let theme_key = self.keybindings.display(actions::THEME).unwrap_or("ctrl+t");
            let shortcuts = [
                ("^/v", "Navigate suggestions"),
                ("Enter", "Select suggestion"),
                ("Tab", "Insert top suggestion"),
                (quit_key, "Quit"),
                (help_key, "Toggle help"),
                (back_key, "Dismiss help"),
                (theme_key, "Cycle theme"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * area.width + hx + 2 + j as u16) as usize;
                    if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * area.width + hx + 14 + j as u16) as usize;
                    if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.fg; }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.keybindings.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.show_help && self.keybindings.matches(actions::BACK, &key) {
            self.show_help = false;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            self.cycle_theme();
            return true;
        }
        if self.show_help {
            return true;
        }
        let handled = self.autocomplete.handle_key(key);
        if handled && self.autocomplete.selected().is_some() {
            self.dirty = true;
        }
        handled
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.autocomplete.handle_mouse(kind, col, row)
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.autocomplete.on_theme_change(theme);
        self.dirty = true;
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Rc::new(AtomicBool::new(false));
    let theme = Theme::from_env_or(Theme::nord());
    let demo = AutocompleteDemo::new(Rc::clone(&should_quit), theme.clone());

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

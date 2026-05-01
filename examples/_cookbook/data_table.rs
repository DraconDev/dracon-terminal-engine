//! Data Table Demo — sortable table with SearchInput filter.
//!
//! Demonstrates: Table widget with column sorting, row selection, and real-time
//! filtering via SearchInput.
//!
//! Layout:
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │ 🔍 Filter: [________________]     Sort: [Name ▼]        │
//! ├─────────────────────────────────────────────────────────┤
//! │ Name          │ Age │ City        │ Profession          │
//! ├───────────────┼─────┼─────────────┼─────────────────────┤
//! │ > Alice       │  28 │ New York    │ Software Engineer   │  ← selected
//! │   Bob         │  34 │ London      │ Data Scientist      │
//! │   Carol       │  22 │ Tokyo       │ Product Manager     │
//! │   David       │  31 │ Berlin      │ DevOps Engineer     │
//! │   Eve         │  29 │ Sydney      │ UX Designer         │
//! ├─────────────────────────────────────────────────────────┤
//! │ Selected: Alice | Age: 28 | City: New York | 5 rows      │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Behavior
//! - Table shows all columns: Name, Age, City, Profession
//! - Click column header to sort (cycles: none → asc → desc → none)
//! - Type in SearchInput to filter rows (filters by any column)
//! - Up/Down arrows navigate rows
//! - Enter selects a row (shows details in status bar)
//! - Selected row is highlighted
//! - Status bar shows: "Selected: [name] | Age: [age] | City: [city] | [n] rows"
//!
//! ## Key Patterns
//! 1. Table widget with column definitions
//! 2. Column header click for sorting
//! 3. SearchInput with debounced filter
//! 4. Row selection and highlight
//! 5. Status bar showing selection details

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{List, SearchInput};
use ratatui::layout::Rect;

const MOCK_DATA: &[(&str, u32, &str, &str)] = &[
    ("Alice", 28, "New York", "Software Engineer"),
    ("Bob", 34, "London", "Data Scientist"),
    ("Carol", 22, "Tokyo", "Product Manager"),
    ("David", 31, "Berlin", "DevOps Engineer"),
    ("Eve", 29, "Sydney", "UX Designer"),
    ("Frank", 45, "Toronto", "Engineering Manager"),
    ("Grace", 27, "Singapore", "Frontend Developer"),
    ("Heidi", 33, "Paris", "Backend Developer"),
    ("Ivan", 41, "Amsterdam", "CTO"),
    ("Judy", 26, "Seoul", "Mobile Developer"),
];

#[derive(Clone)]
struct Person {
    name: String,
    age: u32,
    city: String,
    profession: String,
}

impl Person {
    fn new(name: &str, age: u32, city: &str, profession: &str) -> Self {
        Self {
            name: name.to_string(),
            age,
            city: city.to_string(),
            profession: profession.to_string(),
        }
    }
}

impl std::fmt::Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:>3} {:<12} {}",
            self.name,
            self.age,
            self.city,
            self.profession
        )
    }
}

enum SortState {
    None,
    Asc,
    Desc,
}

impl SortState {
    fn next(&self) -> Self {
        match self {
            SortState::None => SortState::Asc,
            SortState::Asc => SortState::Desc,
            SortState::Desc => SortState::None,
        }
    }
    fn symbol(&self) -> &'static str {
        match self {
            SortState::None => "  ",
            SortState::Asc => " ▲",
            SortState::Desc => " ▼",
        }
    }
}

struct DataTable {
    id: WidgetId,
    people: Vec<Person>,
    filtered: Vec<Person>,
    selected: usize,
    offset: usize,
    visible_count: usize,
    filter_text: String,
    sort_state: SortState,
    search_widget: SearchInput,
    theme: Theme,
    area: Rect,
    dirty: bool,
}

impl DataTable {
    fn new(people: Vec<Person>) -> Self {
        let mut search = SearchInput::new(WidgetId::new(100));
        search.base.text = String::new();
        Self {
            id: WidgetId::default_id(),
            people: people.clone(),
            filtered: people,
            selected: 0,
            offset: 0,
            visible_count: 10,
            filter_text: String::new(),
            sort_state: SortState::None,
            search_widget: search,
            theme: Theme::cyberpunk(),
            area: Rect::new(0, 0, 80, 20),
            dirty: true,
        }
    }

    fn apply_filter(&mut self) {
        let query = self.filter_text.to_lowercase();
        self.filtered = if query.is_empty() {
            self.people.clone()
        } else {
            self.people
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&query)
                        || p.city.to_lowercase().contains(&query)
                        || p.profession.to_lowercase().contains(&query)
                        || p.age.to_string().contains(&query)
                })
                .cloned()
                .collect()
        };
        self.apply_sort();
        self.selected = 0;
        self.offset = 0;
        self.dirty = true;
    }

    fn apply_sort(&mut self) {
        match self.sort_state {
            SortState::None => {}
            SortState::Asc => {
                self.filtered.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortState::Desc => {
                self.filtered.sort_by(|a, b| b.name.cmp(&a.name));
            }
        }
    }

    fn scroll_to(&mut self, index: usize) {
        if index >= self.filtered.len() {
            return;
        }
        self.selected = index;
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + self.visible_count {
            self.offset = self.selected.saturating_sub(self.visible_count) + 1;
        }
    }
}

impl Widget for DataTable {
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
        self.dirty = true;
    }

    fn z_index(&self) -> u16 {
        10
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

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let header_height = 2u16;
        let status_height = 1u16;
        let table_height = area.height.saturating_sub(header_height + status_height);

        for y in 0..area.height {
            for x in 0..area.width {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = self.theme.bg;
                    plane.cells[idx].fg = self.theme.fg;
                }
            }
        }

        let search_area = Rect::new(0, 0, 30, 1);
        let search_plane = self.search_widget.render(search_area);
        plane.merge_plane(search_plane, 0, 0);

        let sort_x = area.width.saturating_sub(25);
        let sort_label = format!("Sort: Name{}", self.sort_state.symbol());
        for (i, c) in sort_label.chars().enumerate() {
            let idx = (0 * area.width + sort_x as u16 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg;
            }
        }

        for x in 0..area.width {
            let idx = (1 * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = Color::Ansi(240);
            }
        }

        let col_widths = [14u16, 5, 12, 20];
        let col_headers = ["Name", "Age", "City", "Profession"];
        let mut x = 0u16;
        for (i, (header, width)) in col_headers.iter().zip(col_widths.iter()).enumerate() {
            let w = *width.min(&area.width.saturating_sub(x));
            for (j, c) in header.chars().take(w as usize).enumerate() {
                let idx = (header_height * area.width + x + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = self.theme.active_fg;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }
            x += w + 1;
        }

        let visible_rows = self
            .filtered
            .iter()
            .skip(self.offset)
            .take(self.visible_count)
            .collect::<Vec<_>>();

        for (i, person) in visible_rows.iter().enumerate() {
            let row = i as u16;
            let is_selected = self.offset + i == self.selected;
            let bg = if is_selected {
                self.theme.selection_bg
            } else {
                self.theme.bg
            };
            let fg = if is_selected {
                self.theme.selection_fg
            } else {
                self.theme.fg
            };
            let style = if is_selected {
                Styles::BOLD
            } else {
                Styles::empty()
            };

            let y = header_height + row;
            for x in 0..area.width {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].fg = fg;
                }
            }

            let values: Vec<String> = vec![
                person.name.clone(),
                person.age.to_string(),
                person.city.clone(),
                person.profession.clone(),
            ];
            let mut x = 0u16;
            for (j, (val, width)) in values.iter().zip(col_widths.iter()).enumerate() {
                let w = *width.min(&area.width.saturating_sub(x));
                let cell_text = if j == 1 {
                    format!("{:>3}", val)
                } else {
                    val.chars().take(w as usize - 1).collect()
                };
                let prefix = if is_selected && j == 0 { "> " } else { "  " };
                for (k, c) in prefix.chars().chain(cell_text.chars()).take(w as usize).enumerate() {
                    let idx = (y * area.width + x + k as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = fg;
                        plane.cells[idx].style = style;
                    }
                }
                x += w + 1;
            }
        }

        let status_y = area.height - status_height;
        let selected_text = if let Some(p) = self.filtered.get(self.selected) {
            format!(
                "Selected: {} | Age: {} | City: {} | {} rows",
                p.name,
                p.age,
                p.city,
                self.filtered.len()
            )
        } else {
            format!("No results | {} rows", self.filtered.len())
        };
        for (i, c) in selected_text.chars().take(area.width as usize).enumerate() {
            let idx = (status_y * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = Color::Rgb(0, 255, 136);
            }
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.search_widget.handle_key(key.clone()) {
            let new_query = self.search_widget.query().to_string();
            if new_query != self.filter_text {
                self.filter_text = new_query;
                self.apply_filter();
            }
            return true;
        }

        match key.code {
            KeyCode::Down => {
                if self.selected + 1 < self.filtered.len() {
                    self.selected += 1;
                    if self.selected >= self.offset + self.visible_count {
                        self.offset = self.selected.saturating_sub(self.visible_count) + 1;
                    }
                    self.dirty = true;
                }
                true
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    if self.selected < self.offset {
                        self.offset = self.selected;
                    }
                    self.dirty = true;
                }
                true
            }
            KeyCode::Home => {
                self.selected = 0;
                self.offset = 0;
                self.dirty = true;
                true
            }
            KeyCode::End => {
                self.selected = self.filtered.len().saturating_sub(1);
                self.offset = self.filtered.len().saturating_sub(self.visible_count);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        let header_height = 2u16;
        let status_height = 1u16;
        let table_height = self
            .area
            .height
            .saturating_sub(header_height + status_height);

        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                if row == header_height - 1 && col >= self.area.width.saturating_sub(25) {
                    self.sort_state = self.sort_state.next();
                    self.apply_sort();
                    self.dirty = true;
                    return true;
                }

                if row >= header_height && row < self.area.height - status_height {
                    let rel_row = row - header_height;
                    if rel_row as usize >= self.offset
                        && rel_row as usize < self.offset + self.visible_count
                    {
                        let idx = self.offset + rel_row as usize;
                        if idx < self.filtered.len() {
                            self.selected = idx;
                            self.dirty = true;
                            return true;
                        }
                    }
                }
                false
            }
            crate::input::event::MouseEventKind::ScrollDown => {
                self.offset = (self.offset + 1)
                    .min(self.filtered.len().saturating_sub(self.visible_count));
                self.dirty = true;
                true
            }
            crate::input::event::MouseEventKind::ScrollUp => {
                self.offset = self.offset.saturating_sub(1);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();

    let people: Vec<Person> = MOCK_DATA
        .iter()
        .map(|(name, age, city, prof)| Person::new(name, *age, city, prof))
        .collect();

    App::new()?
        .title("Data Table Demo")
        .fps(30)
        .theme(theme)
        .run(move |ctx| {
            let (w, h) = ctx.compositor().size();
            let table_area = Rect::new(0, 0, w, h);

            let mut table = DataTable::new(people.clone());
            table.set_area(table_area);
            table.visible_count = (h as usize).saturating_sub(4).max(1);

            let plane = table.render(table_area);
            ctx.add_plane(plane);
        })
}
use crate::framework::hitzone::HitZone;
use crate::framework::theme::Theme;
use crate::compositor::{Cell, Color, Plane, Styles};
use ratatui::layout::Rect;

pub struct Column {
    pub header: String,
    pub width: u16,
}

#[derive(Clone)]
pub struct TableRow<T> {
    pub data: T,
    cells: Vec<String>,
}

pub struct Table<T> {
    columns: Vec<Column>,
    rows: Vec<TableRow<T>>,
    selected: usize,
    sort_col: Option<usize>,
    sort_asc: bool,
    offset: usize,
    visible_count: usize,
    theme: Theme,
    on_select: Option<Box<dyn FnMut(&T)>>,
    on_sort: Option<Box<dyn FnMut(&T, &T) -> std::cmp::Ordering>>,
}

impl<T: Clone + ToString> Table<T> {
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            selected: 0,
            sort_col: None,
            sort_asc: true,
            offset: 0,
            visible_count: 10,
            theme: Theme::default(),
            on_select: None,
            on_sort: None,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_rows(mut self, rows: Vec<T>) -> Self
    where
        T: 'static,
    {
        self.rows = rows
            .into_iter()
            .map(|data| {
                let cells = vec![];
                TableRow { data, cells }
            })
            .collect();
        self
    }

    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn on_sort<F>(mut self, f: F) -> Self
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering + 'static,
    {
        self.on_sort = Some(Box::new(f));
        self
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn get_selected(&self) -> Option<&T> {
        self.rows.get(self.selected).map(|r| &r.data)
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn viewport(&self) -> (usize, usize) {
        let start = self.offset;
        let end = (self.offset + self.visible_count).min(self.rows.len());
        (start, end)
    }

    pub fn scroll_to(&mut self, index: usize) {
        if index >= self.rows.len() {
            return;
        }
        self.selected = index;
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + self.visible_count {
            self.offset = self.selected.saturating_sub(self.visible_count) + 1;
        }
    }

    pub fn set_visible_count(&mut self, count: usize) {
        self.visible_count = count;
    }

    fn cell_text(&self, row: &TableRow<T>, _col: usize) -> String {
        row.data.to_string()
    }

    pub fn render(&self, area: Rect) -> (Plane, Vec<HitZone<usize>>, Vec<HitZone<usize>>) {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let mut header_zones = Vec::new();
        let mut row_zones = Vec::new();
        let row_height: u16 = 1;

        let mut x: u16 = 0;
        for (i, col) in self.columns.iter().enumerate() {
            let w = col.width.min(area.width.saturating_sub(x));
            let rect = Rect::new(x, area.y, w, 1);
            header_zones.push(HitZone::new(i, x, area.y, w, 1));

            for col_idx in 0..w {
                let idx = col_idx as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = self.theme.active_bg;
                    plane.cells[idx].fg = self.theme.fg;
                    plane.cells[idx].char = ' ';
                }
            }

            let label_len = col.header.len().min(w as usize - 2);
            let start = (w.saturating_sub(label_len as u16)) / 2;
            for (j, ch) in col.header.chars().take(label_len).enumerate() {
                let idx = start as usize + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = if self.sort_col == Some(i) {
                        self.theme.accent
                    } else {
                        self.theme.fg
                    };
                    plane.cells[idx].style = if self.sort_col == Some(i) {
                        Styles::BOLD
                    } else {
                        Styles::empty()
                    };
                }
            }

            x += w;
        }

        let visible_rows: Vec<_> = self.rows.iter()
            .skip(self.offset)
            .take(self.visible_count)
            .collect();

        for (i, row) in visible_rows.iter().enumerate() {
            let y = 1 + i;
            let is_selected = self.offset + i == self.selected;
            let bg = if is_selected { self.theme.selection_bg } else { self.theme.bg };
            let fg = if is_selected { self.theme.selection_fg } else { self.theme.fg };

            let y_off = 1u16 + i as u16;
                x = 0;
            for (j, col) in self.columns.iter().enumerate() {
                let w = col.width.min(area.width.saturating_sub(x));
                row_zones.push(HitZone::new(self.offset + i, x, area.y + y_off, w, row_height));

                for col_idx in 0..w {
                    let idx = y as usize * area.width as usize + x as usize + col_idx as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = bg;
                        plane.cells[idx].fg = fg;
                        plane.cells[idx].char = ' ';
                    }
                }

                let text = self.cell_text(row, j);
                let label_len = text.len().min(w as usize - 1).saturating_sub(1);
                for (k, ch) in text.chars().take(label_len).enumerate() {
                    let idx = y as usize * area.width as usize + x as usize + 1 + k;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = fg;
                        plane.cells[idx].style = if is_selected { Styles::BOLD } else { Styles::empty() };
                    }
                }

                x += w;
            }
        }

        (plane, header_zones, row_zones)
    }

    pub fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, _col: u16, row: u16) -> bool {
        if row == 0 {
            for zone in self.columns.iter().enumerate() {
                let _z = HitZone::new(zone.0, 0, 0, 0, 0);
                // find header hit
            }
            return false;
        }

        let rel_row = row.saturating_sub(1);
        if rel_row >= self.visible_count as u16 {
            return false;
        }

        let idx = self.offset + rel_row as usize;
        if idx >= self.rows.len() {
            return false;
        }

        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                self.selected = idx;
                if let Some(f) = self.on_select.as_mut() {
                    f(&self.rows[idx].data);
                }
                true
            }
            crate::input::event::MouseEventKind::ScrollDown => {
                self.offset = (self.offset + 1).min(self.rows.len().saturating_sub(self.visible_count));
                true
            }
            crate::input::event::MouseEventKind::ScrollUp => {
                self.offset = self.offset.saturating_sub(1);
                true
            }
            _ => false,
        }
    }

    pub fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Down => {
                if self.selected + 1 < self.rows.len() {
                    self.selected += 1;
                    if self.selected >= self.offset + self.visible_count {
                        self.offset = self.selected.saturating_sub(self.visible_count) + 1;
                    }
                }
                true
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    if self.selected < self.offset {
                        self.offset = self.selected;
                    }
                }
                true
            }
            KeyCode::Home => {
                self.selected = 0;
                self.offset = 0;
                true
            }
            KeyCode::End => {
                self.selected = self.rows.len().saturating_sub(1);
                self.offset = self.rows.len().saturating_sub(self.visible_count);
                true
            }
            KeyCode::Enter => {
                if let Some(f) = self.on_select.as_mut() {
                    f(&self.rows[self.selected].data);
                }
                true
            }
            _ => false,
        }
    }
}
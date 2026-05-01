//! Widget Gallery — interactive showcase of all framework widgets.
//!
//! Demonstrates every interactive widget in a single, runnable App.
//! Navigate with arrow keys, activate with Enter/Space.
//!
//! # Widgets Shown
//!
//! | Widget | Interaction | State API |
//! |--------|-------------|-----------|
//! | Checkbox | Click/Enter to toggle | `is_checked()` |
//! | Radio | Click/Enter to select | `is_selected()` |
//! | Slider | Drag to change value | `value()` |
//! | Spinner | Auto-animates | `current_frame()` |
//! | Toggle | Click/Enter to toggle | `is_on()` |
//! | Select | Click/Enter to expand | `selected_label()` |
//! | SearchInput | Type to input text | `query()` |
//! | ProgressBar | Changes with slider | `progress()` |
//! | Button | Click/Enter to press | `on_click()` |
//!
//! # Run
//!
//! ```sh
//! cargo run --example widget_gallery
//! ```

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, Checkbox, List, Orientation, ProgressBar, Radio, Select, SearchInput, Slider,
    Spinner, SplitPane, Toggle,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;

const WIDGET_NAMES: &[&str] = &[
    "Checkbox",
    "Radio",
    "Slider",
    "Spinner",
    "Toggle",
    "Select",
    "SearchInput",
    "ProgressBar",
    "Button",
];

struct WidgetGallery {
    id: WidgetId,
    selected: usize,
    checkbox: Checkbox,
    radio: Radio,
    slider: Slider,
    spinner: Spinner,
    toggle: Toggle,
    select: Select,
    search: SearchInput,
    progress: ProgressBar,
    button: Button,
    area: Rect,
    dirty: bool,
}

impl WidgetGallery {
    fn new() -> Self {
        let id = WidgetId::new(1);
        Self {
            id,
            selected: 0,
            checkbox: Checkbox::new(WidgetId::new(10), "Enable Feature"),
            radio: Radio::new(WidgetId::new(11), "Option A"),
            slider: Slider::new(WidgetId::new(12)).with_range(0.0, 100.0),
            spinner: Spinner::new(WidgetId::new(13)),
            toggle: Toggle::new(WidgetId::new(14), "Dark Mode"),
            select: Select::new(WidgetId::new(15))
                .with_options(vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()]),
            search: SearchInput::new(WidgetId::new(16)),
            progress: ProgressBar::new(WidgetId::new(17)),
            button: Button::with_id(WidgetId::new(18), "Click Me!"),
            area: Rect::new(0, 0, 80, 24),
            dirty: true,
        }
    }

    fn nav_list(&self) -> List<String> {
        let items: Vec<String> = WIDGET_NAMES.iter().map(|s| s.to_string()).collect();
        let mut list = List::new_with_id(WidgetId::new(100), items);
        list.set_visible_count(WIDGET_NAMES.len());
        list.with_theme(self.current_theme())
    }

    fn current_theme(&self) -> Theme {
        Theme::nord()
    }

    fn update_progress_from_slider(&mut self) {
        let progress = self.slider.value() / 100.0;
        self.progress.set_progress(progress);
    }

    fn render_nav(&self, area: Rect) -> Plane {
        let mut list = self.nav_list();
        let mut list = list;
        list.set_area(area);
        list.scroll_to(self.selected);
        list.render(area)
    }

    fn render_content(&self, area: Rect) -> Plane {
        let theme = self.current_theme();
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let widget_name = WIDGET_NAMES[self.selected];
        let title = format!("═══ {} ═══", widget_name);
        for (i, c) in title.chars().take(area.width as usize).enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: theme.accent,
                    bg: theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        let content_top = 2;
        let content_height = (area.height as usize).saturating_sub(content_top + 4);

        match self.selected {
            0 => {
                let cb_area = Rect::new(area.x + 2, area.y + content_top as u16, 30, 3);
                let cb_plane = self.checkbox.render(cb_area);
                self.copy_plane(&mut plane, &cb_plane, 2, content_top);
            }
            1 => {
                let radio_area = Rect::new(area.x + 2, area.y + content_top as u16, 30, 3);
                let radio_plane = self.radio.render(radio_area);
                self.copy_plane(&mut plane, &radio_plane, 2, content_top);
            }
            2 => {
                let slider_area = Rect::new(area.x + 2, area.y + content_top as u16, area.width - 4, 3);
                let slider_plane = self.slider.render(slider_area);
                self.copy_plane(&mut plane, &slider_plane, 2, content_top);
                let val_text = format!("Value: {:.0}", self.slider.value());
                for (i, c) in val_text.chars().take(area.width as usize - 4).enumerate() {
                    let idx = ((content_top + 3) as u16 * plane.width + 2 + i as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = Cell {
                            char: c,
                            fg: theme.fg,
                            bg: theme.bg,
                            style: Styles::empty(),
                            transparent: false,
                            skip: false,
                        };
                    }
                }
            }
            3 => {
                let spinner_area = Rect::new(area.x + 2, area.y + content_top as u16, 10, 3);
                let spinner_plane = self.spinner.render(spinner_area);
                self.copy_plane(&mut plane, &spinner_plane, 2, content_top);
                let frame_text = format!("Frame: '{}'", self.spinner.current_frame());
                for (i, c) in frame_text.chars().take(20).enumerate() {
                    let idx = ((content_top + 3) as u16 * plane.width + 2 + i as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = Cell {
                            char: c,
                            fg: theme.fg,
                            bg: theme.bg,
                            style: Styles::empty(),
                            transparent: false,
                            skip: false,
                        };
                    }
                }
            }
            4 => {
                let toggle_area = Rect::new(area.x + 2, area.y + content_top as u16, 30, 3);
                let toggle_plane = self.toggle.render(toggle_area);
                self.copy_plane(&mut plane, &toggle_plane, 2, content_top);
            }
            5 => {
                let select_area = Rect::new(area.x + 2, area.y + content_top as u16, 20, 5);
                let select_plane = self.select.render(select_area);
                self.copy_plane(&mut plane, &select_plane, 2, content_top);
            }
            6 => {
                let search_area = Rect::new(area.x + 2, area.y + content_top as u16, area.width - 4, 3);
                let search_plane = self.search.render(search_area);
                self.copy_plane(&mut plane, &search_plane, 2, content_top);
            }
            7 => {
                let pb_area = Rect::new(area.x + 2, area.y + content_top as u16, area.width - 4, 3);
                let pb_plane = self.progress.render(pb_area);
                self.copy_plane(&mut plane, &pb_plane, 2, content_top);
                let pct = (self.progress.progress() * 100.0).round() as u16;
                let pct_text = format!("{}%", pct);
                for (i, c) in pct_text.chars().take(area.width as usize - 4).enumerate() {
                    let idx = ((content_top + 3) as u16 * plane.width + 2 + i as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = Cell {
                            char: c,
                            fg: theme.fg,
                            bg: theme.bg,
                            style: Styles::empty(),
                            transparent: false,
                            skip: false,
                        };
                    }
                }
            }
            8 => {
                let btn_area = Rect::new(area.x + 2, area.y + content_top as u16, 20, 3);
                let btn_plane = self.button.render(btn_area);
                self.copy_plane(&mut plane, &btn_plane, 2, content_top);
            }
            _ => {}
        }

        let status_y = (area.height as usize).saturating_sub(2);
        let status_text = format!(
            "State: checkbox={}, toggle={}, slider={:.0}, progress={:.0}%",
            self.checkbox.is_checked(),
            self.toggle.is_on(),
            self.slider.value(),
            self.progress.progress() * 100.0
        );
        for (i, c) in status_text.chars().take(area.width as usize).enumerate() {
            let idx = (status_y as u16 * plane.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: theme.inactive_fg,
                    bg: theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        plane
    }

    fn copy_plane(&self, dest: &mut Plane, src: &Plane, offset_x: usize, offset_y: usize) {
        for (i, cell) in src.cells.iter().enumerate() {
            if cell.char == '\0' || cell.transparent {
                continue;
            }
            let src_width = src.width as usize;
            let row = i / src_width;
            let col = i % src_width;
            let dest_row = offset_y + row;
            let dest_col = offset_x + col;
            if dest_row >= dest.height as usize || dest_col >= dest.width as usize {
                continue;
            }
            let dest_idx = dest_row * dest.width as usize + dest_col;
            if dest_idx < dest.cells.len() {
                dest.cells[dest_idx] = cell.clone();
            }
        }
    }

    fn get_widget_mut(&mut self) -> Option<&mut dyn Widget> {
        match self.selected {
            0 => Some(&mut self.checkbox),
            1 => Some(&mut self.radio),
            2 => Some(&mut self.slider),
            3 => Some(&mut self.spinner),
            4 => Some(&mut self.toggle),
            5 => Some(&mut self.select),
            6 => Some(&mut self.search),
            7 => Some(&mut self.progress),
            8 => Some(&mut self.button),
            _ => None,
        }
    }
}

impl Widget for WidgetGallery {
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

    fn needs_render(&self) -> bool {
        self.dirty
            || self.checkbox.needs_render()
            || self.radio.needs_render()
            || self.slider.needs_render()
            || self.spinner.needs_render()
            || self.toggle.needs_render()
            || self.select.needs_render()
            || self.search.needs_render()
            || self.progress.needs_render()
            || self.button.needs_render()
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
        self.checkbox.clear_dirty();
        self.radio.clear_dirty();
        self.slider.clear_dirty();
        self.spinner.clear_dirty();
        self.toggle.clear_dirty();
        self.select.clear_dirty();
        self.search.clear_dirty();
        self.progress.clear_dirty();
        self.button.clear_dirty();
    }

    fn render(&self, area: Rect) -> Plane {
        let split = SplitPane::new(Orientation::Horizontal).ratio(0.3);
        let (nav_rect, content_rect) = split.split(area);

        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 0;

        let nav_plane = self.render_nav(nav_rect);
        self.copy_plane(&mut plane, &nav_plane, nav_rect.x as usize, nav_rect.y as usize);

        let divider_plane = split.render_divider(area);
        self.copy_plane(&mut plane, &divider_plane, divider_plane.x as usize, divider_plane.y as usize);

        let content_plane = self.render_content(content_rect);
        self.copy_plane(&mut plane, &content_plane, content_rect.x as usize, content_rect.y as usize);

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        use KeyCode::{Down, Enter, Up};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            Down => {
                if self.selected + 1 < WIDGET_NAMES.len() {
                    self.selected += 1;
                    self.dirty = true;
                }
                true
            }
            Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.dirty = true;
                }
                true
            }
            Enter => {
                if let Some(widget) = self.get_widget_mut() {
                    widget.handle_key(key)
                } else {
                    false
                }
            }
            _ => {
                if let Some(widget) = self.get_widget_mut() {
                    widget.handle_key(key)
                } else {
                    false
                }
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let split = SplitPane::new(Orientation::Horizontal).ratio(0.3);
        let (nav_rect, content_rect) = split.split(self.area);

        if col >= nav_rect.x && col < nav_rect.x + nav_rect.width
            && row >= nav_rect.y && row < nav_rect.y + nav_rect.height
        {
            let rel_row = (row - nav_rect.y) as usize;
            if rel_row < WIDGET_NAMES.len() {
                self.selected = rel_row;
                self.dirty = true;
                return true;
            }
        }

        if col >= content_rect.x && col < content_rect.x + content_rect.width
            && row >= content_rect.y && row < content_rect.y + content_rect.height
        {
            let rel_col = col - content_rect.x;
            let rel_row = row - content_rect.y;
            if let Some(widget) = self.get_widget_mut() {
                let handled = widget.handle_mouse(kind, rel_col, rel_row);
                if handled && self.selected == 2 {
                    self.update_progress_from_slider();
                }
                if handled {
                    self.dirty = true;
                }
                return handled;
            }
        }
        false
    }

    fn focusable(&self) -> bool {
        true
    }

    fn z_index(&self) -> u16 {
        10
    }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::nord();

    App::new()?
        .title("Widget Gallery")
        .fps(30)
        .theme(theme)
        .on_tick(|_ctx, _tick| {})
        .run(|ctx| {
            let (w, h) = ctx.compositor().size();
            let area = Rect::new(0, 0, w, h);

            let mut gallery = WidgetGallery::new();
            gallery.set_area(area);

            let plane = gallery.render(area);
            ctx.add_plane(plane);
        })
}
#![allow(missing_docs)]
//! Theme Switcher Demo — demonstrates live theme switching with visual feedback.
//!
//! ## Features
//!
//! 1. **Theme cycling**: Press `t` to cycle through all 15 built-in themes
//! 2. **Theme preview panels**: Same widget rendered under each theme
//! 3. **Theme-aware widgets**: StatusBadge, Gauge, Breadcrumbs, and List update on theme change
//! 4. **TrackingWidget**: Verifies `on_theme_change` fires and widgets re-render
//!
//! ## Key Patterns Shown
//!
//! - `App::set_theme()` propagates theme to all widgets via `on_theme_change()`
//! - Widgets that store their own `Theme` can be updated imperatively
//! - TrackingWidget pattern for debugging theme change callbacks
//! - Theme colors (`success_fg`, `warning_fg`, `error_fg`) vary per theme

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::app::App;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use ratatui::layout::Rect;
use std::io::Result;
use std::sync::atomic::{AtomicUsize, Ordering};

const ALL_THEMES: &[(&str, fn() -> Theme)] = &[
    ("Dark", Theme::dark),
    ("Light", Theme::light),
    ("Dracula", Theme::dracula),
    ("Monokai", Theme::monokai),
    ("Nord", Theme::nord),
    ("Gruvbox", Theme::gruvbox_dark),
    ("OneDark", Theme::one_dark),
    ("SolarizedDark", Theme::solarized_dark),
    ("SolarizedLight", Theme::solarized_light),
    ("Catppuccin", Theme::catppuccin_mocha),
    ("TokyoNight", Theme::tokyo_night),
    ("VSCodeDark", || vscode_dark()),
    ("VSCodeLight", || vscode_light()),
    ("Cyberpunk", Theme::cyberpunk),
    ("Autumn", || autumn()),
];

static CURRENT_THEME_INDEX: AtomicUsize = AtomicUsize::new(0);

fn vscode_dark() -> Theme {
    Theme {
        name: "vscode-dark",
        bg: Color::Rgb(30, 30, 30),
        fg: Color::Rgb(204, 204, 204),
        accent: Color::Rgb(0, 122, 204),
        selection_bg: Color::Rgb(62, 62, 62),
        selection_fg: Color::Rgb(255, 255, 255),
        border: Color::Rgb(63, 63, 63),
        scrollbar_track: Color::Rgb(40, 40, 40),
        scrollbar_thumb: Color::Rgb(90, 90, 90),
        hover_bg: Color::Rgb(50, 50, 50),
        active_bg: Color::Rgb(60, 60, 60),
        inactive_fg: Color::Rgb(127, 127, 127),
        input_bg: Color::Rgb(35, 35, 35),
        input_fg: Color::Rgb(204, 204, 204),
        scrollbar_width: 1,
        error_fg: Color::Rgb(235, 75, 75),
        success_fg: Color::Rgb(73, 201, 73),
        warning_fg: Color::Rgb(227, 180, 60),
        disabled_fg: Color::Rgb(90, 90, 90),
    }
}

fn vscode_light() -> Theme {
    Theme {
        name: "vscode-light",
        bg: Color::Rgb(255, 255, 255),
        fg: Color::Rgb(30, 30, 30),
        accent: Color::Rgb(0, 122, 204),
        selection_bg: Color::Rgb(173, 214, 255),
        selection_fg: Color::Rgb(0, 0, 0),
        border: Color::Rgb(200, 200, 200),
        scrollbar_track: Color::Rgb(245, 245, 245),
        scrollbar_thumb: Color::Rgb(160, 160, 160),
        hover_bg: Color::Rgb(230, 230, 230),
        active_bg: Color::Rgb(200, 200, 200),
        inactive_fg: Color::Rgb(150, 150, 150),
        input_bg: Color::Rgb(255, 255, 255),
        input_fg: Color::Rgb(30, 30, 30),
        scrollbar_width: 1,
        error_fg: Color::Rgb(229, 62, 62),
        success_fg: Color::Rgb(73, 201, 73),
        warning_fg: Color::Rgb(227, 180, 60),
        disabled_fg: Color::Rgb(150, 150, 150),
    }
}

fn autumn() -> Theme {
    Theme {
        name: "autumn",
        bg: Color::Rgb(40, 35, 30),
        fg: Color::Rgb(220, 200, 180),
        accent: Color::Rgb(200, 140, 80),
        selection_bg: Color::Rgb(80, 65, 50),
        selection_fg: Color::Rgb(240, 225, 200),
        border: Color::Rgb(120, 100, 80),
        scrollbar_track: Color::Rgb(35, 30, 25),
        scrollbar_thumb: Color::Rgb(100, 85, 70),
        hover_bg: Color::Rgb(55, 45, 35),
        active_bg: Color::Rgb(70, 60, 50),
        inactive_fg: Color::Rgb(150, 130, 110),
        input_bg: Color::Rgb(30, 25, 20),
        input_fg: Color::Rgb(220, 200, 180),
        scrollbar_width: 1,
        error_fg: Color::Rgb(200, 100, 80),
        success_fg: Color::Rgb(140, 180, 100),
        warning_fg: Color::Rgb(200, 160, 80),
        disabled_fg: Color::Rgb(100, 90, 75),
    }
}

fn get_current_theme() -> Theme {
    let idx = CURRENT_THEME_INDEX.load(Ordering::SeqCst);
    ALL_THEMES[idx % ALL_THEMES.len()].1()
}

fn cycle_theme() {
    let idx = CURRENT_THEME_INDEX.load(Ordering::SeqCst);
    CURRENT_THEME_INDEX.store((idx + 1) % ALL_THEMES.len(), Ordering::SeqCst);
}

struct ThemeHeader {
    id: WidgetId,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl ThemeHeader {
    fn new(id: WidgetId) -> Self {
        Self {
            id,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 3)),
            dirty: true,
        }
    }
}

impl Widget for ThemeHeader {
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
        self.dirty = true;
    }

    fn z_index(&self) -> u16 {
        100
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
        plane.z_index = 100;

        let theme = get_current_theme();
        let theme_idx = CURRENT_THEME_INDEX.load(Ordering::SeqCst);
        let title = format!(" Theme: {} (press t to cycle) ", ALL_THEMES[theme_idx].0);

        for y in 0..area.height {
            for x in 0..area.width {
                let idx = y as usize * area.width as usize + x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: ' ',
                        fg: if y == 1 { theme.accent } else { theme.fg },
                        bg: theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        for (i, c) in title.chars().enumerate() {
            let x = (area.width as usize / 2 - title.len() / 2 + i) as u16;
            let idx = 1 * area.width as usize + x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        plane
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        if let KeyCode::Char('t') = key.code {
            cycle_theme();
            self.dirty = true;
            true
        } else {
            false
        }
    }

    fn on_theme_change(&mut self, _theme: &Theme) {
        self.dirty = true;
    }
}

struct TrackingWidget {
    id: WidgetId,
    theme_change_count: u32,
    last_theme_name: String,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl TrackingWidget {
    fn new(id: WidgetId) -> Self {
        Self {
            id,
            theme_change_count: 0,
            last_theme_name: String::from("none"),
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 5)),
            dirty: true,
        }
    }
}

impl Widget for TrackingWidget {
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

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme_change_count += 1;
        self.last_theme_name = theme.name.to_string();
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 50;

        let theme = get_current_theme();
        let lines = [
            format!("on_theme_change calls: {}", self.theme_change_count),
            format!("Last theme: {}", self.last_theme_name),
            "TrackingWidget (verifies callback)".to_string(),
        ];

        for (y, line) in lines.iter().enumerate().take(area.height as usize) {
            for (x, c) in line.chars().take(area.width as usize - 1).enumerate() {
                let idx = y * area.width as usize + x;
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

        plane
    }
}

struct ThemePreviewPanel {
    id: WidgetId,
    preview_index: usize,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl ThemePreviewPanel {
    fn new(id: WidgetId, preview_index: usize) -> Self {
        Self {
            id,
            preview_index,
            area: std::cell::Cell::new(Rect::new(0, 0, 30, 8)),
            dirty: true,
        }
    }

    fn theme(&self) -> Theme {
        let current = CURRENT_THEME_INDEX.load(Ordering::SeqCst);
        let idx = (current + self.preview_index) % ALL_THEMES.len();
        ALL_THEMES[idx].1()
    }
}

impl Widget for ThemePreviewPanel {
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

        let theme = self.theme();

        for y in 0..area.height {
            for x in 0..area.width {
                let idx = y as usize * area.width as usize + x as usize;
                if idx < plane.cells.len() {
                    let is_border = x == 0 || x == area.width - 1 || y == 0 || y == area.height - 1;
                    plane.cells[idx] = Cell {
                        char: if is_border { '#' } else { ' ' },
                        fg: theme.border,
                        bg: theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        let current = CURRENT_THEME_INDEX.load(Ordering::SeqCst);
        let preview_idx = (current + self.preview_index) % ALL_THEMES.len();
        let theme_name = ALL_THEMES[preview_idx].0;
        let label = format!("[{}]", theme_name);
        for (i, c) in label.chars().enumerate() {
            let x = 1 + i as u16;
            let idx = 1 * area.width as usize + x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = theme.accent;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let badge_row = 3;
        let badge_texts = ["OK", "WARN", "ERROR", "OK"];
        let badge_fgs = [theme.success_fg, theme.warning_fg, theme.error_fg, theme.success_fg];

        for (i, (text, fg)) in badge_texts.iter().zip(badge_fgs.iter()).enumerate() {
            let x = 2 + (i as u16 * 7);
            let content = format!("[{}]", text);
            for (j, c) in content.chars().enumerate() {
                let idx = badge_row as usize * area.width as usize + x as usize + j as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = *fg;
                    plane.cells[idx].style = Styles::BOLD;
                    plane.cells[idx].bg = theme.bg;
                }
            }
        }

        let gauge_row = 5u16;
        let gauge_width = (area.width - 4) as usize;
        let fill = (gauge_width * 65) / 100;

        if 1 < area.height {
            let left_bracket_idx = gauge_row as usize * area.width as usize + 1;
            let right_bracket_idx = gauge_row as usize * area.width as usize + area.width as usize - 2;
            if left_bracket_idx < plane.cells.len() {
                plane.cells[left_bracket_idx] = Cell {
                    char: '[',
                    fg: theme.fg,
                    bg: theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
            if right_bracket_idx < plane.cells.len() {
                plane.cells[right_bracket_idx] = Cell {
                    char: ']',
                    fg: theme.fg,
                    bg: theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }

            for i in 0..gauge_width {
                let idx = gauge_row as usize * area.width as usize + 2 + i;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: if i < fill { '█' } else { '░' },
                        fg: if i < fill { theme.success_fg } else { theme.inactive_fg },
                        bg: theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        plane
    }

    fn on_theme_change(&mut self, _theme: &Theme) {
        self.dirty = true;
    }
}

struct WidgetDemoPanel {
    id: WidgetId,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl WidgetDemoPanel {
    fn new(id: WidgetId) -> Self {
        Self {
            id,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 12)),
            dirty: true,
        }
    }
}

impl Widget for WidgetDemoPanel {
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
        self.dirty = true;
    }

    fn z_index(&self) -> u16 {
        20
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
        plane.z_index = 20;

        let theme = get_current_theme();
        let title = " Widget Preview ";
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = theme.bg;
            }
        }
        for (i, c) in title.chars().enumerate() {
            let x = (area.width as usize / 2 - title.len() / 2 + i) as u16;
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = theme.accent;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let row1 = 2;
        let badges = [
            ("OK", theme.success_fg),
            ("WARNING", theme.warning_fg),
            ("ERROR", theme.error_fg),
            ("OK", theme.success_fg),
        ];
        for (i, (text, fg)) in badges.iter().enumerate() {
            let x = 2 + (i as u16 * 14);
            let content = format!("[{}]", text);
            for (j, c) in content.chars().enumerate() {
                let idx = row1 as usize * area.width as usize + x as usize + j as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = *fg;
                    plane.cells[idx].style = Styles::BOLD;
                    plane.cells[idx].bg = theme.bg;
                }
            }
        }

        let gauge_row = 4;
        let label = "CPU: ";
        for (i, c) in label.chars().enumerate() {
            let idx = gauge_row as usize * area.width as usize + 2 + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = theme.fg;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let gauge_start = 2 + label.len();
        let gauge_end = area.width as usize - 2;
        let gauge_len = gauge_end - gauge_start;
        let fill = (gauge_len * 65) / 100;

        let left_bracket_idx = gauge_row as usize * area.width as usize + gauge_start;
        let right_bracket_idx = gauge_row as usize * area.width as usize + gauge_end;
        if left_bracket_idx < plane.cells.len() {
            plane.cells[left_bracket_idx] = Cell {
                char: '[',
                fg: theme.fg,
                bg: theme.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
        if right_bracket_idx < plane.cells.len() {
            plane.cells[right_bracket_idx] = Cell {
                char: ']',
                fg: theme.fg,
                bg: theme.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }

        for i in 0..gauge_len {
            let idx = gauge_row as usize * area.width as usize + gauge_start + 1 + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: if i < fill { '█' } else { '░' },
                    fg: if i < fill { theme.success_fg } else { theme.inactive_fg },
                    bg: theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        let list_row = 6;
        let items = ["item-1", "item-2", "item-3", "selected", "item-5"];
        let selected_idx = 3;
        for (i, item) in items.iter().enumerate() {
            let is_selected = i == selected_idx;
            let bg = if is_selected {
                theme.selection_bg
            } else {
                theme.bg
            };
            let fg = if is_selected {
                theme.selection_fg
            } else {
                theme.fg
            };
            let style = if is_selected {
                Styles::BOLD
            } else {
                Styles::empty()
            };

            for (j, c) in item.chars().enumerate() {
                let idx = (list_row as usize + i) * area.width as usize + 2 + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].style = style;
                }
            }
        }

        let breadcrumb_row = 11;
        let crumbs = ["home", "projects", "demo"];
        let total_len: usize = crumbs.iter().map(|s| s.len()).sum::<usize>() + crumbs.len() - 1;
        let start_x = (area.width as usize / 2 - total_len / 2) as u16;

        let mut x = start_x;
        for (i, crumb) in crumbs.iter().enumerate() {
            let is_last = i == crumbs.len() - 1;
            if i > 0 {
                let idx = breadcrumb_row as usize * area.width as usize + x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '/';
                    plane.cells[idx].fg = theme.inactive_fg;
                }
                x += 1;
            }

            for c in crumb.chars() {
                let idx = breadcrumb_row as usize * area.width as usize + x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if is_last {
                        theme.accent
                    } else {
                        theme.fg
                    };
                    plane.cells[idx].style = if is_last {
                        Styles::BOLD
                    } else {
                        Styles::empty()
                    };
                    plane.cells[idx].bg = if is_last {
                        theme.active_bg
                    } else {
                        theme.bg
                    };
                }
                x += 1;
            }
        }

        plane
    }

    fn on_theme_change(&mut self, _theme: &Theme) {
        self.dirty = true;
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?.title("Theme Switcher Demo").fps(30);

    let header = ThemeHeader::new(WidgetId::new(1));
    let _header_id = app.add_widget(Box::new(header), Rect::new(0, 0, 80, 3));

    let tracking = TrackingWidget::new(WidgetId::new(2));
    let _tracking_id = app.add_widget(Box::new(tracking), Rect::new(60, 3, 20, 5));

    let preview = ThemePreviewPanel::new(WidgetId::new(3), 0);
    let _preview_id = app.add_widget(Box::new(preview), Rect::new(0, 8, 30, 8));

    let preview2 = ThemePreviewPanel::new(WidgetId::new(4), 1);
    let _preview2_id = app.add_widget(Box::new(preview2), Rect::new(30, 8, 30, 8));

    let preview3 = ThemePreviewPanel::new(WidgetId::new(5), 2);
    let _preview3_id = app.add_widget(Box::new(preview3), Rect::new(60, 8, 20, 8));

    let demo = WidgetDemoPanel::new(WidgetId::new(6));
    let _demo_id = app.add_widget(Box::new(demo), Rect::new(0, 17, 80, 12));

    let _ = app.run(|_ctx| {});

    println!("\nTheme Switcher Demo Ended");
    println!("All 15 themes demonstrated:");
    for (name, _) in ALL_THEMES {
        println!("  - {}", name);
    }

    Ok(())
}
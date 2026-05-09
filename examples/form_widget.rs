#![allow(missing_docs)]
//! Form Widget Example — comprehensive form with validation and theming.
//!
//! Demonstrates multi-field forms with various input types, validation,
//! focus cycling, and theme switching.
//!
//! Controls:
//!   Tab/Shift+Tab  — cycle fields
//!   Type           — enter values
//!   Enter          — submit form
//!   t              — cycle theme
//!   ?              — toggle help
//!   q              — quit

use dracon_terminal_engine::compositor::{Color, Plane};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::Form;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct FormApp {
    form: Rc<RefCell<Form>>,
    should_quit: Arc<AtomicBool>,
    theme: Theme,
    show_help: bool,
    submitted: bool,
    submit_time: Option<std::time::Instant>,
}

impl FormApp {
    fn new(should_quit: Arc<AtomicBool>, theme: Theme) -> Self {
        let form = Form::new(WidgetId::new(1))
            .with_theme(theme)
            .add_field("Full Name")
            .add_field("Email Address")
            .add_field("Company")
            .add_field("Role")
            .add_field("Project Name")
            .add_field("Description");
        Self {
            form: Rc::new(RefCell::new(form)),
            should_quit,
            theme,
            show_help: false,
            submitted: false,
            submit_time: None,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::dark(), Theme::light(), Theme::cyberpunk(), Theme::dracula(),
            Theme::nord(), Theme::catppuccin_mocha(), Theme::gruvbox_dark(),
            Theme::tokyo_night(), Theme::solarized_dark(), Theme::solarized_light(),
            Theme::one_dark(), Theme::rose_pine(), Theme::kanagawa(),
            Theme::everforest(), Theme::monokai(), Theme::warm(),
            Theme::cool(), Theme::forest(), Theme::sunset(), Theme::mono(),
        ];
        let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];
        self.form.borrow_mut().on_theme_change(&self.theme);
    }

    fn submit_form(&mut self) {
        self.submitted = true;
        self.submit_time = Some(std::time::Instant::now());
    }

    fn render_help_overlay(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let help_lines = [
            "╭─ Controls ──────────────────╮",
            "│ Tab/Shift+Tab  Cycle fields │",
            "│ Type           Enter value  │",
            "│ Enter          Submit form  │",
            "│ t              Cycle theme  │",
            "│ ?              Toggle help  │",
            "│ Esc            Dismiss help │",
            "│ q              Quit         │",
            "╰─────────────────────────────╯",
        ];
        let w = 32.min(area.width.saturating_sub(4));
        let h = help_lines.len() as u16 + 2;
        let x = (area.width - w) / 2;
        let y = (area.height - h) / 2;

        // Background
        for row in 0..h {
            for col in 0..w {
                let idx = ((y + row) * plane.width + x + col) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Border
        let corners = [(y, x, '╭'), (y, x + w - 1, '╮'), (y + h - 1, x, '╰'), (y + h - 1, x + w - 1, '╯')];
        for (cy, cx, ch) in &corners {
            let idx = (*cy * plane.width + *cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = *ch;
                plane.cells[idx].fg = t.outline;
            }
        }
        for col in 1..w - 1 {
            let top = (y * plane.width + x + col) as usize;
            let bot = ((y + h - 1) * plane.width + x + col) as usize;
            if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
            if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
        }
        for row in 1..h - 1 {
            let left = ((y + row) * plane.width + x) as usize;
            let right = ((y + row) * plane.width + x + w - 1) as usize;
            if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
            if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
        }

        // Text
        let start_y = y + 1;
        for (i, line) in help_lines.iter().enumerate() {
            let row = start_y + i as u16;
            let line_x = x + (w - line.len() as u16) / 2;
            for (j, c) in line.chars().enumerate() {
                let idx = (row * plane.width + line_x + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if i == 0 || i == help_lines.len() - 1 {
                        t.primary
                    } else {
                        t.fg
                    };
                }
            }
        }
    }

    fn render_submitted_banner(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let msg = " ✓ Form submitted successfully! ";
        let w = msg.len() as u16 + 4;
        let h = 3u16;
        let x = (area.width - w) / 2;
        let y = 1u16;

        for row in 0..h {
            for col in 0..w {
                let idx = ((y + row) * plane.width + x + col) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.success;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        for (i, c) in msg.chars().enumerate() {
            let idx = ((y + 1) * plane.width + x + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_on_accent;
                plane.cells[idx].style = Styles::BOLD;
            }
        }
    }
}

impl Widget for FormApp {
    fn id(&self) -> WidgetId { WidgetId::new(0) }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { self.form.borrow().area() }
    fn set_area(&mut self, area: Rect) { self.form.borrow_mut().set_area(area); }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.form.borrow_mut().on_theme_change(theme);
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = self.form.borrow().render(area);
        for cell in plane.cells.iter_mut() {
            cell.transparent = false;
            if cell.bg == Color::Reset {
                cell.bg = self.theme.bg;
            }
        }

        // Title
        let title = " Form Demo ";
        let tx = (area.width - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (plane.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Status bar
        let status_y = area.height.saturating_sub(1);
        let hint = "Tab: next | Enter: submit | t: theme | ?: help | q: quit";
        for (i, c) in hint.chars().take((area.width as usize).saturating_sub(2)).enumerate() {
            let idx = (status_y * plane.width + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
                plane.cells[idx].bg = self.theme.surface;
            }
        }

        // Submitted banner
        if self.submitted {
            if let Some(time) = self.submit_time {
                if time.elapsed().as_secs() < 3 {
                    self.render_submitted_banner(&mut plane, area);
                }
            }
        }

        if self.show_help {
            self.render_help_overlay(&mut plane, area);
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
            KeyCode::Char('t') => {
                self.cycle_theme();
                true
            }
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                true
            }
            KeyCode::Esc => {
                if self.show_help {
                    self.show_help = false;
                    true
                } else {
                    false
                }
            }
            KeyCode::Enter => {
                self.submit_form();
                true
            }
            _ => self.form.borrow_mut().handle_key(key),
        }
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let theme = Theme::nord();
    let app = Rc::new(RefCell::new(FormApp::new(should_quit, theme)));
    let app_for_input = Rc::clone(&app);
    let app_for_tick = Rc::clone(&app);

    let mut app_widget = App::new()?.title("Form Widget Demo").fps(30).theme(theme);

    app_widget.add_widget(
        Box::new(FormApp::new(quit_check.clone(), theme)),
        Rect::new(0, 0, 80, 24),
    );

    app_widget
        .on_input(move |key| app_for_input.borrow_mut().handle_key(key))
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
            }
            let mut app = app_for_tick.borrow_mut();
            if app.submitted {
                if let Some(time) = app.submit_time {
                    if time.elapsed().as_secs() >= 3 {
                        app.submitted = false;
                        ctx.mark_all_dirty();
                    }
                }
            }
        })
        .run(|_ctx| {})
}

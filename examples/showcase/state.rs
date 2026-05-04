use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::hitzone::ScopedZoneRegistry;
use ratatui::layout::Rect;

use crate::data::ExampleMeta;

// ═══════════════════════════════════════════════════════════════════════════════
// SHOWCASE STATE
// ═══════════════════════════════════════════════════════════════════════════════

pub struct Showcase {
    pub examples: Vec<ExampleMeta>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub category_filter: Option<&'static str>,
    pub search_query: String,
    pub search_active: bool,
    pub theme: Theme,
    pub pending_theme: Option<usize>,
    pub should_quit: Arc<AtomicBool>,
    pub pending_binary: Arc<Mutex<Option<String>>>,
    pub status_message: Option<(String, Instant)>,
    pub area: Rect,
    pub cols: std::cell::Cell<usize>,
    pub last_click_time: Option<Instant>,
    pub last_click_idx: Option<usize>,
    pub fps: Arc<AtomicU64>,
    pub show_fps: bool,
    pub card_start: Instant,
    pub primitive_toggle: bool,
    pub primitive_slider: f32,
    pub primitive_checkbox: bool,
    pub primitive_radio: usize,
    pub primitive_button: bool,
    pub show_help: bool,
    pub modal_preview: bool,
    pub show_debug: bool,
    pub primitive_button_time: Option<Instant>,
    pub zones: RefCell<ScopedZoneRegistry<usize>>,
    pub tooltip_text: Option<String>,
    pub tooltip_timer: Option<Instant>,
    pub tooltip_pos: Option<(u16, u16)>,
    pub context_menu: Option<(usize, u16, u16)>,
    pub context_menu_selected: usize,
    pub hovered_card: Option<usize>,
    pub mouse_pos: Option<(u16, u16)>,
}

impl Showcase {
    fn new(
        should_quit: Arc<AtomicBool>,
        pending: Arc<Mutex<Option<String>>>,
        fps: Arc<AtomicU64>,
    ) -> Self {
        let examples = ExampleMeta::all();
        let filtered: Vec<usize> = (0..examples.len()).collect();
        Self {
            examples,
            filtered,
            selected: 0,
            category_filter: None,
            search_query: String::new(),
            search_active: false,
            theme: Theme::nord(),
            pending_theme: None,
            should_quit,
            pending_binary: pending,
            status_message: None,
            area: Rect::new(0, 0, 80, 24),
            cols: std::cell::Cell::new(3),
            last_click_time: None,
            last_click_idx: None,
            fps,
            hovered_card: None,
            mouse_pos: None,
            context_menu: None,
            context_menu_selected: 0,
            tooltip_text: None,
            tooltip_timer: None,
            tooltip_pos: None,
            show_help: false,
            modal_preview: false,
            show_fps: true,
            card_start: Instant::now(),
            primitive_toggle: false,
            primitive_slider: 0.5,
            primitive_checkbox: true,
            primitive_radio: 0,
            primitive_button: false,
            show_debug: false,
            primitive_button_time: None,
            zones: RefCell::new(ScopedZoneRegistry::new()),
        }
    }

    pub fn themes() -> Vec<(&'static str, Theme)> {
        vec![
            ("dark", Theme::dark()),
            ("light", Theme::light()),
            ("cyberpunk", Theme::cyberpunk()),
            ("dracula", Theme::dracula()),
            ("nord", Theme::nord()),
            ("catppuccin", Theme::catppuccin_mocha()),
            ("gruvbox", Theme::gruvbox_dark()),
            ("tokyo-night", Theme::tokyo_night()),
            ("solarized-dark", Theme::solarized_dark()),
            ("solarized-light", Theme::solarized_light()),
            ("one-dark", Theme::one_dark()),
            ("rose-pine", Theme::rose_pine()),
            ("kanagawa", Theme::kanagawa()),
            ("everforest", Theme::everforest()),
            ("monokai", Theme::monokai()),
        ]
    }

    pub fn apply_filter(&mut self) {
        if let Some(idx) = self.pending_theme.take() {
            self.theme = Self::themes()[idx % Self::themes().len()].1;
        }
        self.filtered = self
            .examples
            .iter()
            .enumerate()
            .filter(|(_, ex)| {
                let matches_category = self.category_filter.is_none_or(|cat| ex.category == cat);
                let matches_search = if self.search_query.is_empty() {
                    true
                } else {
                    let q = self.search_query.to_lowercase();
                    ex.name.to_lowercase().contains(&q)
                        || ex.description.to_lowercase().contains(&q)
                        || ex.category.to_lowercase().contains(&q)
                };
                matches_category && matches_search
            })
            .map(|(i, _)| i)
            .collect();
        self.selected = self.selected.min(self.filtered.len().saturating_sub(1));
    }

    fn selected_example(&self) -> Option<&ExampleMeta> {
        self.filtered
            .get(self.selected)
            .and_then(|&idx| self.examples.get(idx))
    }

    fn launch_selected(&mut self) {
        if let Some(ex) = self.selected_example() {
            *self.pending_binary.lock().unwrap() = Some(ex.binary_name.to_string());
            self.status_message = Some((format!("Launching {}...", ex.name), Instant::now()));
        }
    }
}

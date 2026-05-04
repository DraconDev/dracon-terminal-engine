use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use dracon_terminal_engine::framework::hitzone::ScopedZoneRegistry;
use dracon_terminal_engine::framework::prelude::*;
use ratatui::layout::Rect;

use crate::data::ExampleMeta;

// ═══════════════════════════════════════════════════════════════════════════════
// SHOWCASE STATE
// ═══════════════════════════════════════════════════════════════════════════════

pub struct Showcase {
    pub(crate) examples: Vec<ExampleMeta>,
    pub(crate) filtered: Vec<usize>,
    pub(crate) selected: usize,
    pub(crate) category_filter: Option<&'static str>,
    pub(crate) search_query: String,
    pub(crate) search_active: bool,
    pub(crate) theme: Theme,
    pub(crate) pending_theme: Option<usize>,
    pub(crate) should_quit: Arc<AtomicBool>,
    pub(crate) pending_binary: Arc<Mutex<Option<String>>>,
    pub(crate) status_message: Option<(String, Instant)>,
    pub(crate) area: Rect,
    pub(crate) cols: std::cell::Cell<usize>,
    pub(crate) last_click_time: Option<Instant>,
    pub(crate) last_click_idx: Option<usize>,
    pub(crate) fps: Arc<AtomicU64>,
    pub(crate) show_fps: bool,
    pub(crate) card_start: Instant,
    pub(crate) primitive_toggle: bool,
    pub(crate) primitive_slider: f32,
    pub(crate) primitive_checkbox: bool,
    pub(crate) primitive_radio: usize,
    pub(crate) primitive_button: bool,
    pub(crate) show_help: bool,
    pub(crate) modal_preview: bool,
    pub(crate) show_debug: bool,
    pub(crate) primitive_button_time: Option<Instant>,
    pub(crate) zones: RefCell<ScopedZoneRegistry<usize>>,
    pub(crate) tooltip_text: Option<String>,
    pub(crate) tooltip_timer: Option<Instant>,
    pub(crate) tooltip_pos: Option<(u16, u16)>,
    pub(crate) context_menu: Option<(usize, u16, u16)>,
    pub(crate) context_menu_selected: usize,
    pub(crate) hovered_card: Option<usize>,
    pub(crate) mouse_pos: Option<(u16, u16)>,
    pub(crate) recently_launched: Vec<String>,
}

impl Showcase {
    pub fn new(
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
            recently_launched: Vec::new(),
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

    pub fn selected_example(&self) -> Option<&ExampleMeta> {
        self.filtered
            .get(self.selected)
            .and_then(|&idx| self.examples.get(idx))
    }

    pub fn launch_selected(&mut self) {
        if let Some(ex) = self.selected_example() {
            *self.pending_binary.lock().unwrap() = Some(ex.binary_name.to_string());
            self.status_message = Some((format!("Launching {}...", ex.name), Instant::now()));
            self.recently_launched.retain(|n| n != ex.binary_name);
            self.recently_launched.insert(0, ex.binary_name.to_string());
            if self.recently_launched.len() > 5 {
                self.recently_launched.truncate(5);
            }
        }
    }
}

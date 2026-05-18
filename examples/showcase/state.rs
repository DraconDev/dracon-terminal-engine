use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dracon_terminal_engine::framework::animation::AnimationManager;
use dracon_terminal_engine::framework::hitzone::ScopedZoneRegistry;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::{SceneRouter, SceneTransition};
use ratatui::layout::Rect;

use crate::data::ExampleMeta;

// ═══════════════════════════════════════════════════════════════════════════════
// SORT
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SortField {
    Name,
    Category,
    RunCount,
}

impl SortField {
    pub fn all() -> [SortField; 3] {
        [SortField::Name, SortField::Category, SortField::RunCount]
    }
    pub fn label(&self) -> &'static str {
        match self {
            SortField::Name => "name",
            SortField::Category => "cat",
            SortField::RunCount => "runs",
        }
    }
}

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
    pub(crate) show_profiler: bool,
    pub(crate) primitive_button_time: Option<Instant>,
    pub(crate) zones: RefCell<ScopedZoneRegistry<usize>>,
    pub(crate) tooltip_text: Option<String>,
    pub(crate) tooltip_timer: Option<Instant>,
    pub(crate) tooltip_pos: Option<(u16, u16)>,
    pub(crate) context_menu: Option<(usize, u16, u16)>,
    pub(crate) context_menu_selected: usize,
    pub(crate) hovered_card: Option<usize>,
    pub(crate) mouse_pos: Option<(u16, u16)>,
    pub(crate) scroll_offset: std::cell::Cell<usize>,
    pub(crate) recently_launched: Vec<String>,
    pub(crate) show_input_debug: bool,
    pub(crate) event_log: RefCell<VecDeque<(Instant, String)>>,
    pub(crate) animations: AnimationManager,
    pub(crate) card_hover_anim: Vec<Option<usize>>, // animation id per card
    pub(crate) toast_anim: Option<usize>,
    pub(crate) returned_from: Arc<Mutex<Option<(String, Instant)>>>,
    pub(crate) scene_router: SceneRouter,
    pub(crate) keybindings: KeybindingSet,
    pub(crate) pending_app_theme: Arc<Mutex<Option<Theme>>>,
    pub(crate) run_counts: Vec<u32>,
    pub(crate) sort_field: SortField,
    pub(crate) sort_ascending: bool,
    pub(crate) cached_themes: Vec<Theme>,

    pub(crate) search_query_lower: String,
    pub(crate) dirty: bool,
    pub(crate) last_render_second: u32,
    /// Cached stats bar text ("N Examples | M Widgets | K Themes")
    /// Only recomputed when filter changes.
    pub(crate) cached_stats_text: String,
    /// Cached category counts, recomputed only when examples list changes.
    pub(crate) cached_cat_counts: [usize; 7],
    /// Cached clock text ("HH:MM:SS"), updated once per second.
    pub(crate) cached_clock_text: RefCell<String>,
}

impl Showcase {
    pub fn new(
        should_quit: Arc<AtomicBool>,
        pending: Arc<Mutex<Option<String>>>,
        fps: Arc<AtomicU64>,
        returned_from: Arc<Mutex<Option<(String, Instant)>>>,
        pending_app_theme: Arc<Mutex<Option<Theme>>>,
        theme: Theme,
    ) -> Self {
        let examples = ExampleMeta::all();
        let filtered: Vec<usize> = (0..examples.len()).collect();
        let mut scene_router = SceneRouter::new()
            .with_default_transition(SceneTransition::Fade);
        scene_router.register("widget_gallery", Box::new(crate::scenes::widget_gallery::WidgetGalleryScene::new(theme.clone())));
        scene_router.register("theme_switcher", Box::new(crate::scenes::theme_switcher::ThemeSwitcherScene::new(theme.clone())));
        scene_router.register("form_demo", Box::new(crate::scenes::form_demo::FormDemoScene::new(theme.clone())));
        scene_router.register("tree_navigator", Box::new(crate::scenes::tree_navigator::TreeNavigatorScene::new(theme.clone())));
        scene_router.register("modal_demo", Box::new(crate::scenes::modal_demo::ModalDemoScene::new(theme.clone())));
        scene_router.register("calendar", Box::new(crate::scenes::calendar_scene::CalendarScene::new(theme.clone())));
        scene_router.register("rich_text", Box::new(crate::scenes::rich_text_scene::RichTextScene::new(theme.clone())));
        scene_router.register("autocomplete", Box::new(crate::scenes::autocomplete_scene::AutocompleteScene::new(theme.clone())));
        scene_router.register("notification_center", Box::new(crate::scenes::notification_center_scene::NotificationCenterScene::new(theme.clone())));
        scene_router.register("accessibility", Box::new(crate::scenes::accessibility_scene::AccessibilityScene::new(theme.clone())));
        scene_router.register("cell_pool", Box::new(crate::scenes::cell_pool_scene::CellPoolScene::new(theme.clone())));
        scene_router.register("kanban", Box::new(crate::scenes::kanban_scene::KanbanScene::new(theme.clone())));
        scene_router.register("animation", Box::new(crate::scenes::animation_scene::AnimationScene::new(theme.clone())));
        scene_router.register("color_picker", Box::new(crate::scenes::color_picker_scene::ColorPickerScene::new(theme.clone())));
        scene_router.register("tags_input", Box::new(crate::scenes::tags_input_scene::TagsInputScene::new(theme.clone())));
        scene_router.register("tooltip", Box::new(crate::scenes::tooltip_scene::TooltipScene::new(theme.clone())));
        scene_router.register("progress", Box::new(crate::scenes::progress_scene::ProgressScene::new(theme.clone())));
        scene_router.register("password_input", Box::new(crate::scenes::password_input_scene::PasswordInputScene::new(theme.clone())));
        scene_router.register("radio", Box::new(crate::scenes::radio_scene::RadioScene::new(theme.clone())));
        scene_router.register("debug_overlay", Box::new(crate::scenes::debug_overlay_scene::DebugOverlayScene::new(theme.clone())));
        scene_router.register("raycaster", Box::new(crate::scenes::raycaster_scene::RaycasterScene::new(theme.clone())));
        scene_router.register("paint", Box::new(crate::scenes::paint_scene::PaintScene::new(theme.clone())));
        scene_router.register("workshop", Box::new(crate::scenes::workshop_scene::WorkshopScene::new(theme.clone())));
        scene_router.register("command_palette", Box::new(crate::scenes::command_palette_scene::CommandPaletteScene::new(theme.clone())));
        scene_router.register("table_list", Box::new(crate::scenes::table_list_scene::TableListScene::new(theme.clone())));
        scene_router.register("settings_panel", Box::new(crate::scenes::settings_scene::SettingsScene::new(theme.clone())));
        scene_router.register("live_feed", Box::new(crate::scenes::live_feed_scene::LiveFeedScene::new(theme.clone())));
        scene_router.register("action_center", Box::new(crate::scenes::action_center_scene::ActionCenterScene::new(theme.clone())));
        scene_router.register("dev_console", Box::new(crate::scenes::dev_console_scene::DevConsoleScene::new(theme.clone())));
        scene_router.register("metrics_hub", Box::new(crate::scenes::metrics_hub_scene::MetricsHubScene::new(theme.clone())));
        scene_router.register("navigator", Box::new(crate::scenes::navigator_scene::NavigatorScene::new(theme.clone())));
        scene_router.register("control_panel", Box::new(crate::scenes::control_panel_scene::ControlPanelScene::new(theme.clone())));
        scene_router.register("hud_demo", Box::new(crate::scenes::hud_demo_scene::HudDemoScene::new(theme.clone())));

        let run_counts = vec![0u32; examples.len()];

        let cached_themes: Vec<Theme> = Theme::all().iter().filter(|t| &*t.name != "high_contrast").cloned().collect();

        let mut showcase = Self {
            examples,
            filtered,
            selected: 0,
            category_filter: None,
            search_query: String::new(),
            search_active: false,
            theme,
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
            scroll_offset: std::cell::Cell::new(0),
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
            show_profiler: false,
            primitive_button_time: None,
            zones: RefCell::new(ScopedZoneRegistry::new()),
            recently_launched: Vec::new(),
            show_input_debug: false,
            event_log: RefCell::new(VecDeque::with_capacity(16)),
            animations: AnimationManager::new(),
            card_hover_anim: Vec::new(),
            toast_anim: None,
            returned_from,
            scene_router,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            pending_app_theme,
            run_counts,
            sort_field: SortField::Name,
            sort_ascending: true,
            cached_themes,

            search_query_lower: String::new(),
            dirty: true,
            last_render_second: 0,
            cached_stats_text: String::new(),
            cached_cat_counts: [0usize; 7],
            cached_clock_text: RefCell::new(String::new()),
        };
        // Initialize cached stats text and category counts
        showcase.apply_filter();
        showcase
    }

    pub fn themes(&self) -> &[Theme] {
        &self.cached_themes
    }

    pub fn apply_filter(&mut self) {
        if let Some(idx) = self.pending_theme.take() {
            let themes = &self.cached_themes;
            self.theme = themes[idx % themes.len()].clone();
        }
        self.search_query_lower = self.search_query.to_lowercase();
        self.filtered = self
            .examples
            .iter()
            .enumerate()
            .filter(|(_, ex)| {
                let matches_category = self.category_filter.is_none_or(|cat| ex.category == cat);
                let matches_search = if self.search_query.is_empty() {
                    true
                } else {
                    let q = &self.search_query_lower;
                    ex.name.to_lowercase().contains(q)
                        || ex.description.to_lowercase().contains(q)
                        || ex.category.to_lowercase().contains(q)
                };
                matches_category && matches_search
            })
            .map(|(i, _)| i)
            .collect();

        // Sort filtered results
        match self.sort_field {
            SortField::Name => {
                if self.sort_ascending {
                    self.filtered.sort_by_key(|&i| self.examples[i].name);
                } else {
                    self.filtered.sort_by(|&a, &b| self.examples[b].name.cmp(self.examples[a].name));
                }
            }
            SortField::Category => {
                if self.sort_ascending {
                    self.filtered.sort_by_key(|&i| self.examples[i].category);
                } else {
                    self.filtered.sort_by(|&a, &b| self.examples[b].category.cmp(self.examples[a].category));
                }
            }
            SortField::RunCount => {
                if self.sort_ascending {
                    self.filtered.sort_by_key(|&i| self.run_counts[i]);
                } else {
                    self.filtered.sort_by(|&a, &b| self.run_counts[b].cmp(&self.run_counts[a]));
                }
            }
        }

        self.selected = self.selected.min(self.filtered.len().saturating_sub(1));
        self.scroll_offset.set(0);

        // Recompute cached stats text
        self.cached_stats_text = format!(
            "  {} Examples  │  {} Widgets  │  {} Themes ",
            self.examples.len(),
            43,
            self.cached_themes.len()
        );

        // Recompute cached category counts
        let categories = ["all", "apps", "input", "data", "cookbook", "tools", "accessibility"];
        for (i, cat) in categories.iter().enumerate() {
            self.cached_cat_counts[i] = if *cat == "all" {
                self.examples.len()
            } else {
                self.examples.iter().filter(|e| e.category == *cat).count()
            };
        }
    }

    pub fn selected_example(&self) -> Option<&ExampleMeta> {
        self.filtered
            .get(self.selected)
            .and_then(|&idx| self.examples.get(idx))
    }

    /// Number of visible card rows based on current area and card size
    pub fn visible_rows(&self) -> usize {
        let area = self.area;
        let sidebar_w = 18;
        let grid_start_y = 3; // sidebar_start_y + 1
        let available_h = (area.height as usize).saturating_sub(grid_start_y).saturating_sub(2);
        let card_h = if (area.width as usize).saturating_sub(sidebar_w + 2) >= 90 { 16 }
                     else if (area.width as usize).saturating_sub(sidebar_w + 2) >= 60 { 14 }
                     else { 12 };
        (available_h / (card_h + 1)).max(1)
    }

    /// Auto-scroll so that the selected card is visible
    pub fn ensure_selected_visible(&self) {
        let cols = self.cols.get().max(1);
        let selected_row = self.selected / cols;
        let scroll = self.scroll_offset.get();
        let visible = self.visible_rows();

        if selected_row < scroll {
            // Selected is above visible area — scroll up
            self.scroll_offset.set(selected_row);
        } else if selected_row >= scroll + visible {
            // Selected is below visible area — scroll down
            self.scroll_offset.set(selected_row.saturating_sub(visible - 1));
        }
    }

    /// Total rows of cards
    pub fn total_rows(&self) -> usize {
        let cols = self.cols.get().max(1);
        self.filtered.len().div_ceil(cols)
    }

    pub fn is_embedded(&self, name: &str) -> bool {
        matches!(name, "widget_gallery" | "theme_switcher" | "form_demo" | "tree_navigator" | "modal_demo" | "calendar" | "rich_text" | "autocomplete" | "notification_center" | "accessibility" | "cell_pool" | "kanban" | "animation" | "color_picker" | "tags_input" | "tooltip" | "progress" | "password_input" | "radio" | "debug_overlay" | "raycaster" | "paint" | "workshop")
    }

    pub fn launch_selected(&mut self) {
        let example = self.selected_example().cloned();
        if let Some(ex) = example {
            // Increment run count
            if let Some(ex_idx) = self.filtered.get(self.selected) {
                if *ex_idx < self.run_counts.len() {
                    self.run_counts[*ex_idx] += 1;
                }
            }

            if self.is_embedded(ex.name) {
                // Launch embedded scene
                self.scene_router.push(ex.name);
                self.scene_router.on_theme_change(&self.theme);
                return;
            }

            // Launch external binary
            let name = ex.binary_name.to_string();
            *self.pending_binary.lock().unwrap() = Some(name.clone());
            self.status_message = Some((format!("Launching {}...", ex.name), Instant::now()));
            self.toast_anim = Some(self.animations.start(-3.0, 0.0, Duration::from_millis(300)));
            self.recently_launched.retain(|n| n != &name);
            self.recently_launched.insert(0, name);
            if self.recently_launched.len() > 5 {
                self.recently_launched.truncate(5);
            }
        }
    }
}

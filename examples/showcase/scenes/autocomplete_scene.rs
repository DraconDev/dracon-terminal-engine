//! Embedded Autocomplete scene for the showcase.
//!
//! Demonstrates the Autocomplete widget with search suggestions,
//! rich info panel, match count, and recent selections history.

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::{Color, Plane};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::Autocomplete;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;

struct PackageInfo {
    name: &'static str,
    version: &'static str,
    downloads: &'static str,
    category: &'static str,
    description: &'static str,
}

const PACKAGES: &[PackageInfo] = &[
    PackageInfo { name: "rustacean", version: "1.4.0", downloads: "2.1M", category: "tooling", description: "Rust IDE support & analysis" },
    PackageInfo { name: "rust-analyzer", version: "2026.5", downloads: "8.3M", category: "tooling", description: "Next-gen Rust compiler frontend" },
    PackageInfo { name: "rustdoc", version: "1.78", downloads: "5.6M", category: "docs", description: "Documentation generator for Rust" },
    PackageInfo { name: "rustfmt", version: "1.7", downloads: "9.1M", category: "tooling", description: "Rust code formatter" },
    PackageInfo { name: "rustc", version: "1.78", downloads: "12M", category: "compiler", description: "The Rust compiler" },
    PackageInfo { name: "cargo", version: "1.78", downloads: "14M", category: "tooling", description: "Rust package manager" },
    PackageInfo { name: "clippy", version: "0.1.78", downloads: "7.2M", category: "linting", description: "Rust linter & code checker" },
    PackageInfo { name: "miri", version: "0.1", downloads: "890K", category: "testing", description: "Undefined behavior detector" },
    PackageInfo { name: "rls", version: "1.42", downloads: "3.4M", category: "tooling", description: "Rust Language Server (legacy)" },
    PackageInfo { name: "rustlings", version: "5.6", downloads: "1.8M", category: "learning", description: "Interactive Rust exercises" },
    PackageInfo { name: "rustup", version: "1.27", downloads: "11M", category: "tooling", description: "Rust toolchain installer" },
    PackageInfo { name: "crates.io", version: "-", downloads: "-", category: "registry", description: "Rust package registry" },
];

fn category_color(cat: &str, theme: &Theme) -> Color {
    match cat {
        "tooling" => theme.primary,
        "compiler" => theme.error,
        "docs" => theme.success,
        "linting" => theme.warning,
        "testing" => theme.info,
        "learning" => theme.secondary,
        "registry" => theme.fg_muted,
        _ => theme.fg,
    }
}

pub struct AutocompleteScene {
    autocomplete: Autocomplete,
    theme: Theme,
    show_help: bool,
    selected_item: Option<String>,
    recent_selections: Vec<String>,
    keybindings: KeybindingSet,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    selection_bridge: Rc<RefCell<Option<String>>>,
}

impl AutocompleteScene {
    pub fn new(theme: Theme) -> Self {
        let suggestions: Vec<String> = PACKAGES.iter().map(|p| p.name.to_string()).collect();
        let bridge = Rc::new(RefCell::new(None));
        let bridge_cb = Rc::clone(&bridge);
        let mut autocomplete = Autocomplete::new(WidgetId::new(100), suggestions)
            .with_theme(theme.clone())
            .with_max_visible(6)
            .on_select(move |s| { *bridge_cb.borrow_mut() = Some(s.to_string()); });
        autocomplete.set_area(Rect::new(2, 3, 28, 12));
        autocomplete.on_focus();
        autocomplete.open_dropdown();
        Self {
            autocomplete,
            theme,
            show_help: false,
            selected_item: None,
            recent_selections: Vec::new(),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
            selection_bridge: bridge,
        }
    }

    fn sync_bridge(&mut self) {
        if let Some(sel) = self.selection_bridge.borrow_mut().take() {
            self.selected_item = Some(sel.clone());
            // Add to recent selections (dedup, max 5)
            self.recent_selections.retain(|s| s != &sel);
            self.recent_selections.insert(0, sel);
            self.recent_selections.truncate(5);
            self.dirty = true;
        }
    }

    fn get_package_info(&self, name: &str) -> Option<&PackageInfo> {
        PACKAGES.iter().find(|p| p.name == name)
    }

    fn render_info_panel(&self, plane: &mut Plane, x: u16, y: u16, w: u16) {
        let t = &self.theme;

        if let Some(ref name) = self.selected_item {
            draw_text(plane, x, y, "Package Details", t.primary, t.bg, true);

            // Divider
            for dx in 0..w {
                let idx = ((y + 1) * plane.width + x + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }

            if let Some(pkg) = self.get_package_info(name) {
                // Category badge
                let cat_color = category_color(pkg.category, t);
                let badge = format!(" {} ", pkg.category);
                draw_text(plane, x, y + 2, &badge, cat_color, t.bg, true);

                // Name (large)
                draw_text(plane, x, y + 3, pkg.name, t.primary, t.bg, true);

                // Version + downloads
                let ver = format!("v{}", pkg.version);
                let dl = format!("{} downloads", pkg.downloads);
                draw_text(plane, x, y + 4, &ver, t.fg, t.bg, false);
                draw_text(plane, x + ver.len() as u16 + 2, y + 4, &dl, t.fg_muted, t.bg, false);

                // Description
                draw_text(plane, x, y + 6, pkg.description, t.fg, t.bg, false);

                // Visual install bar (decorative)
                draw_text(plane, x, y + 8, "Popularity:", t.fg_muted, t.bg, false);
                let bar_x = x + 12;
                let bar_w = (w as usize).saturating_sub(14);
                let fill = match pkg.downloads {
                    d if d.contains('M') => d.trim_end_matches('M').parse::<f32>().unwrap_or(0.0) / 15.0,
                    d if d.contains('K') => d.trim_end_matches('K').parse::<f32>().unwrap_or(0.0) / 15000.0,
                    _ => 0.1,
                };
                let filled = (fill * bar_w as f32) as usize;
                for bx in 0..bar_w {
                    let idx = ((y + 8) * plane.width + bar_x + bx as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = if bx < filled { '█' } else { '░' };
                        plane.cells[idx].fg = if bx < filled { cat_color } else { t.fg_muted };
                        plane.cells[idx].transparent = false;
                    }
                }
            } else {
                draw_text(plane, x, y + 2, name, t.primary, t.bg, true);
                draw_text(plane, x, y + 3, "Custom package", t.fg_muted, t.bg, false);
            }
        } else {
            draw_text(plane, x, y, "Package Details", t.primary, t.bg, true);
            for dx in 0..w {
                let idx = ((y + 1) * plane.width + x + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }
            draw_text(plane, x, y + 2, "Select a package", t.fg_muted, t.bg, false);
            draw_text(plane, x, y + 3, "to see details", t.fg_muted, t.bg, false);
        }
    }

    fn render_recent_panel(&self, plane: &mut Plane, x: u16, y: u16) {
        let t = &self.theme;

        draw_text(plane, x, y, "Recent", t.secondary, t.bg, true);

        if self.recent_selections.is_empty() {
            draw_text(plane, x, y + 1, "No selections yet", t.fg_muted, t.bg, false);
        } else {
            for (i, sel) in self.recent_selections.iter().enumerate() {
                let ry = y + 1 + i as u16;
                let num = format!("{}.", i + 1);
                draw_text(plane, x, ry, &num, t.fg_muted, t.bg, false);

                let cat_color = self.get_package_info(sel)
                    .map(|p| category_color(p.category, t))
                    .unwrap_or(t.fg);
                draw_text(plane, x + 3, ry, sel, cat_color, t.bg, false);
            }
        }
    }

    fn render_stats_bar(&self, plane: &mut Plane, x: u16, y: u16) {
        let t = &self.theme;
        let total = PACKAGES.len();
        let categories: Vec<&str> = PACKAGES.iter().map(|p| p.category).collect();
        let unique_cats: Vec<&&str> = categories.iter().collect::<std::collections::HashSet<_>>().into_iter().collect();

        let stats = format!("{} packages · {} categories", total, unique_cats.len());
        draw_text(plane, x, y, &stats, t.fg_muted, t.bg, false);
    }
}

impl Scene for AutocompleteScene {
    fn scene_id(&self) -> &str { "autocomplete" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        draw_text(&mut plane, 2, 0, " Autocomplete ", t.primary, t.bg, true);
        self.render_stats_bar(&mut plane, 18, 0);

        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 2), 0,
                  &theme_label, t.secondary, t.bg, false);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Left: Search input + Autocomplete dropdown
        draw_text(&mut plane, 2, 2, "Search packages:", t.fg_muted, t.bg, false);
        let ac_area = Rect::new(2, 3, 28, 12);
        let ac_plane = self.autocomplete.render(ac_area);
        blit_to(&mut plane, &ac_plane, ac_area.x as usize, ac_area.y as usize);

        // Category legend under the dropdown
        let legend_y = 3 + 8;
        let categories = [("tooling", t.primary), ("compiler", t.error), ("docs", t.success),
                         ("linting", t.warning), ("testing", t.info), ("learning", t.secondary)];
        let mut lx = 2u16;
        for (cat, color) in &categories {
            let pill = format!(" {} ", cat);
            draw_text(&mut plane, lx, legend_y, &pill, *color, t.bg, true);
            lx += pill.len() as u16 + 1;
        }

        // Vertical divider
        let div_x = 32u16;
        for y in 2..area.height.saturating_sub(2) {
            let idx = (y * area.width + div_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Right: Info panel + Recent selections
        let panel_x = div_x + 2;
        let panel_w = area.width.saturating_sub(panel_x + 2);
        self.render_info_panel(&mut plane, panel_x, 2, panel_w);
        self.render_recent_panel(&mut plane, panel_x, 12);

        // Footer
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(" Type:search | ↑↓:nav | Enter:select | Tab:complete | {}:help | {}:back ", help_key, back_key);
        let fy = area.height.saturating_sub(1);
        for (i, c) in footer.chars().enumerate() {
            let idx = (fy * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        if self.show_help {
            render_help_overlay(&mut plane, area, t, "Autocomplete Help", &[("Up/Dn", "Navigate suggestions"), ("Enter", "Select item"), ("Tab", "Auto-complete"), ("B/Esc", "Back to showcase")]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        if self.autocomplete.handle_key(key) {
            self.sync_bridge();
            // Refresh set_area after key handling
            let ac_area = Rect::new(2, 3, 28, 12);
            self.autocomplete.set_area(ac_area);
            if self.selected_item.is_none() {
                if let Some(selected) = self.autocomplete.selected() {
                    self.selected_item = Some(selected.to_string());
                }
            }
            self.dirty = true;
            return true;
        }
        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();
        let ac_area = Rect::new(area.x + 2, area.y + 3, 28, 12);
        self.autocomplete.set_area(ac_area);
        let rel_col = col.saturating_sub(ac_area.x);
        let rel_row = row.saturating_sub(ac_area.y);
        if (ac_area.x..ac_area.x + ac_area.width).contains(&col) &&
           (ac_area.y..ac_area.y + ac_area.height).contains(&row) &&
           self.autocomplete.handle_mouse(kind, rel_col, rel_row)
        {
            self.sync_bridge();
            if self.selected_item.is_none() {
                if let Some(selected) = self.autocomplete.selected() {
                    self.selected_item = Some(selected.to_string());
                    self.dirty = true;
                }
            }
            return true;
        }

        // Category pills (row 11): click to filter
        if let MouseEventKind::Down(_) = kind {
            if row == 11 && col >= 2 {
                let categories = ["tooling", "compiler", "docs", "linting", "testing", "learning"];
                let mut pill_x = 2usize;
                for cat in &categories {
                    let pill_w = cat.len() + 2; // " cat "
                    if (pill_x..pill_x + pill_w).contains(&(col as usize)) {
                        // Clear and type the category name
                        self.autocomplete.clear();
                        for c in cat.chars() {
                            self.autocomplete.handle_key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::empty(), kind: KeyEventKind::Press });
                        }
                        self.sync_bridge();
                        self.dirty = true;
                        return true;
                    }
                    pill_x += pill_w + 1;
                }
            }

            // Recent selections list (right panel, col 34+, rows 12+)
            let info_x = area.width * 40 / 100;
            if col >= info_x && row >= 12 {
                let idx = (row - 12) as usize;
                if idx < self.recent_selections.len() {
                    let name = self.recent_selections[idx].clone();
                    self.autocomplete.clear();
                    for c in name.chars() {
                        self.autocomplete.handle_key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::empty(), kind: KeyEventKind::Press });
                    }
                    self.sync_bridge();
                    self.selected_item = Some(name);
                    self.dirty = true;
                    return true;
                }
            }
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.autocomplete.on_theme_change(theme);
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}


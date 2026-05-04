use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use chrono::Local;
use dracon_terminal_engine::compositor::Plane;
use crate::render::CardConfig;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

use crate::render::{category_color, draw_rounded_border, draw_text, render_card, set_cell};
use crate::state::Showcase;

impl Widget for Showcase {
    fn id(&self) -> WidgetId {
        WidgetId::new(0)
    }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect {
        self.area
    }
    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = self.theme;

        // Background fill
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Clear and rebuild zone registry for this frame
        self.zones.borrow_mut().clear();

        // Title bar with decorative border
        let title_text = " Dracon Terminal Engine ";
        let title_x = 2usize;
        let title_y = 0usize;

        for (i, ch) in title_text.chars().enumerate() {
            let px = title_x + i;
            if px < area.width as usize {
                set_cell(&mut plane, px, title_y, ch, t.primary, t.bg);
            }
        }

        // Live clock
        let now = Local::now();
        let clock_text = now.format("%H:%M:%S").to_string();
        let clock_x = title_x + title_text.len() + 2;
        if clock_x + clock_text.len() < area.width as usize - 10 {
            draw_text(
                &mut plane,
                clock_x,
                title_y,
                &clock_text,
                t.fg_muted,
                t.bg,
                false,
            );
        }

        // FPS counter (right-aligned)
        if self.show_fps {
            let fps_val = self.fps.load(Ordering::Relaxed);
            let fps_text = format!("{} FPS", fps_val);
            let fps_x = area.width as usize - fps_text.len() - 2;
            if fps_x > title_x + title_text.len() {
                draw_text(
                    &mut plane, fps_x, title_y, &fps_text, t.success, t.bg, false,
                );
            }
        }

        // FPS toggle checkbox
        let fps_toggle = if self.show_fps { "[x] FPS" } else { "[ ] FPS" };
        let toggle_x = area.width as usize - fps_toggle.len() - 2;
        draw_text(
            &mut plane, toggle_x, title_y, fps_toggle, t.fg_muted, t.bg, false,
        );
        // Zone: FPS toggle
        let mut zones = self.zones.borrow_mut();
        zones.register(
            400,
            toggle_x as u16,
            title_y as u16,
            fps_toggle.len() as u16,
            1,
        );
        drop(zones);

        // Theme palette bar
        let palette_y = 1usize;
        let themes = Self::themes();
        let square_w = 2usize;
        let gap = 1usize;
        let max_visible = (area.width as usize).saturating_sub(4) / (square_w + gap);
        let visible_themes = max_visible.min(themes.len());
        let palette_start_x =
            ((area.width as usize).saturating_sub(visible_themes * (square_w + gap))) / 2;
        const PALETTE_BASE: usize = 200;
        // Determine hovered palette swatch
        let hovered_palette = self
            .mouse_pos
            .filter(|(_, my)| *my as usize == palette_y)
            .and_then(|(mx, _)| {
                let x = mx as usize;
                if x >= palette_start_x && x < palette_start_x + visible_themes * (square_w + gap) {
                    let rel = x - palette_start_x;
                    let idx = rel / (square_w + gap);
                    if rel % (square_w + gap) < square_w {
                        Some(idx)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .filter(|idx| *idx < themes.len());
        for (i, (_name, theme)) in themes.iter().enumerate() {
            if i >= visible_themes {
                break;
            }
            let x = palette_start_x + i * (square_w + gap);
            let is_active = theme.name == self.theme.name;
            let is_hovered = hovered_palette == Some(i);
            let bg = if is_hovered {
                theme.primary_hover
            } else if is_active {
                theme.primary_active
            } else {
                theme.primary
            };
            let fg = if is_hovered || is_active {
                theme.fg_on_accent
            } else {
                theme.fg_muted
            };
            // Draw 2-char wide colored square
            for dx in 0..square_w {
                if x + dx < area.width as usize {
                    let ch = if dx == 0 && is_active && !is_hovered {
                        '▶'
                    } else {
                        ' '
                    };
                    set_cell(&mut plane, x + dx, palette_y, ch, fg, bg);
                }
            }
            // Register zone for this palette swatch
            let mut zones = self.zones.borrow_mut();
            zones.register(
                PALETTE_BASE + i,
                x as u16,
                palette_y as u16,
                square_w as u16,
                1,
            );
            drop(zones);
        }

        // Stats bar
        let stats_y = 2usize;
        let stats_text = format!(
            " {} Examples  │  {} Widgets  │  {} Themes ",
            self.examples.len(),
            35,
            themes.len()
        );
        let stats_start = 2usize;
        draw_text(
            &mut plane,
            stats_start,
            stats_y,
            &stats_text,
            t.fg_muted,
            t.bg,
            false,
        );
        for x in stats_start + stats_text.len()..area.width as usize - 2 {
            set_cell(&mut plane, x, stats_y, '─', t.outline, t.bg);
        }

        // Search bar with icon
        let search_y = 3usize;
        let search_icon = if self.search_active { ">" } else { ":" };
        let search_prompt = if self.search_active { ">" } else { " " };
        let search_text = format!("{} {} [{}]", search_icon, search_prompt, self.search_query);
        let search_fg = if self.search_active {
            t.primary
        } else {
            t.fg_muted
        };
        let search_text_chars = search_text.chars().count() + 1;
        draw_text(
            &mut plane,
            2,
            search_y,
            &search_text,
            search_fg,
            t.surface,
            false,
        );
        // Fill rest of search bar
        for x in search_text_chars + 2..area.width as usize - 2 {
            set_cell(&mut plane, x, search_y, ' ', search_fg, t.surface);
        }
        // Draw cursor if active
        if self.search_active && !self.search_query.is_empty() {
            let cursor_x = 2 + search_text_chars - 1;
            if cursor_x < area.width as usize - 2 {
                set_cell(&mut plane, cursor_x, search_y, '_', t.primary, t.surface);
            }
        }

        // Search feedback: match count or no results
        if self.search_active && !self.search_query.is_empty() {
            let feedback_text = if self.filtered.is_empty() {
                "  No results!".to_string()
            } else if self.filtered.len() == 1 {
                "  1 result".to_string()
            } else {
                format!("  {} results", self.filtered.len())
            };
            let feedback_color = if self.filtered.is_empty() {
                t.error
            } else {
                t.fg_muted
            };
            draw_text(
                &mut plane,
                2,
                5,
                &feedback_text,
                feedback_color,
                t.bg,
                false,
            );
        }

        // Primitives bar
        let prim_y = 4usize;
        let state_0 = if self.primitive_toggle {
            "[*] Toggle"
        } else {
            "[ ] Toggle"
        };
        let state_1 = {
            let pos = ((self.primitive_slider * 10.0).round() as usize).min(10);
            let filled: String = (0..pos).map(|_| '=').collect();
            let empty: String = (pos..10).map(|_| "-").collect();
            format!("[{}{}]", filled, empty)
        };
        let state_2 = if self.primitive_checkbox {
            "[x] Check"
        } else {
            "[ ] Check"
        };
        let state_3 = {
            let sel = self.primitive_radio;
            let opts = ["(1)", "(2)", "(3)"];
            let mut s = String::new();
            for (j, _o) in opts.iter().enumerate() {
                s.push_str(if j == sel { "(*)" } else { "( )" });
            }
            s
        };
        let state_4 = if self.primitive_button {
            "[CLICKED!]"
        } else {
            "[ Button ]"
        };
        let prim_controls: [(&str, &str); 5] = [
            ("[1]", state_0),
            ("[2]", &state_1),
            ("[3]", state_2),
            ("[4]", &state_3),
            ("[5]", state_4),
        ];
        // Compute positions and register zones
        let mut prim_x = 2usize;
        let mut zones = self.zones.borrow_mut();
        const PRIM_BASE: usize = 100;
        for (i, (key, state)) in prim_controls.iter().enumerate() {
            let total_w = key.len() + 1 + state.len();
            zones.register(
                PRIM_BASE + i,
                prim_x as u16,
                prim_y as u16,
                total_w as u16,
                1,
            );
            prim_x += total_w + 3;
        }
        // Determine hover from zones
        let hovered_prim = self
            .mouse_pos
            .and_then(|(mx, my)| zones.dispatch(mx, my))
            .filter(|id| *id >= PRIM_BASE && *id < PRIM_BASE + 5)
            .map(|id| id - PRIM_BASE);
        drop(zones); // release borrow before using plane
                     // Draw primitives with hover highlight
        prim_x = 2usize;
        for (i, (key, state)) in prim_controls.iter().enumerate() {
            let hovered = hovered_prim == Some(i);
            let key_fg = if hovered { t.primary } else { t.fg_muted };
            let state_fg = if hovered { t.primary } else { t.fg };
            draw_text(&mut plane, prim_x, prim_y, key, key_fg, t.bg, false);
            prim_x += key.len();
            draw_text(&mut plane, prim_x, prim_y, " ", t.fg_muted, t.bg, false);
            prim_x += 1;
            draw_text(&mut plane, prim_x, prim_y, state, state_fg, t.bg, false);
            prim_x += state.len() + 3;
        }

        // Category sidebar
        let sidebar_w = 14usize;
        let sidebar_start_y = 6usize;
        let categories = ["all", "apps", "cookbook", "tools"];
        const CAT_BASE: usize = 300;
        // Determine hovered sidebar category
        let hovered_cat = self
            .mouse_pos
            .filter(|(mx, my)| {
                let y = *my as usize;
                (*mx as usize) < sidebar_w && y >= sidebar_start_y && y < sidebar_start_y + 8
            })
            .map(|(_, my)| (my as usize - sidebar_start_y) / 2)
            .filter(|idx| *idx < categories.len());
        for (i, cat) in categories.iter().enumerate() {
            let cat_y = sidebar_start_y + i * 2;
            let is_active = self.category_filter.map_or(*cat == "all", |f| f == *cat);
            let is_hovered = hovered_cat == Some(i);
            let (fg, bg_cat) = if is_hovered {
                (t.fg, t.surface_elevated)
            } else if is_active {
                (t.fg_on_accent, t.primary_active)
            } else {
                (t.fg_muted, t.bg)
            };
            let (icon, label) = match *cat {
                "all" => ("◈", " ALL "),
                "apps" => ("▣", " APPS "),
                "cookbook" => ("◉", " COOKBOOK "),
                "tools" => ("◦", " TOOLS "),
                _ => ("•", *cat),
            };
            let icon_fg = if is_hovered || is_active {
                t.primary
            } else {
                t.fg_muted
            };
            draw_text(
                &mut plane,
                1,
                cat_y,
                icon,
                icon_fg,
                bg_cat,
                is_active || is_hovered,
            );
            draw_text(
                &mut plane,
                3,
                cat_y,
                label,
                fg,
                bg_cat,
                is_active || is_hovered,
            );
            // Register zone for this category
            let mut zones = self.zones.borrow_mut();
            zones.register(CAT_BASE + i, 0, cat_y as u16, sidebar_w as u16, 1);
            drop(zones);

            // Count badge
            let count = if *cat == "all" {
                self.examples.len()
            } else {
                self.examples.iter().filter(|e| e.category == *cat).count()
            };
            let count_str = format!("{:>2}", count);
            draw_text(&mut plane, 13, cat_y, &count_str, t.fg_muted, bg_cat, false);
        }

        // Grid of cards — responsive sizing
        let grid_start_x = sidebar_w + 2;
        let grid_start_y = sidebar_start_y + 1;
        let available_w = area.width as usize - grid_start_x;
        let available_h = area.height as usize - grid_start_y - 2;

        // Responsive card sizing
        let (card_w, card_h) = if available_w >= 90 {
            (32usize, 16usize) // Large terminal
        } else if available_w >= 60 {
            (28usize, 14usize) // Medium terminal
        } else {
            (24usize, 12usize) // Small terminal
        };

        self.cols.set((available_w / (card_w + 2)).max(1));
        let cols = self.cols.get();

        for (grid_idx, &ex_idx) in self.filtered.iter().enumerate() {
            if let Some(ex) = self.examples.get(ex_idx) {
                let col = grid_idx % cols;
                let row = grid_idx / cols;
                let x = grid_start_x + col * (card_w + 2);
                let y = grid_start_y + row * (card_h + 1);

                if x + card_w > area.width as usize || y + card_h > area.height as usize - 2 {
                    continue;
                }

                let card_config = CardConfig {
                    ex,
                    idx: grid_idx,
                    selected_idx: self.selected,
                    hovered_idx: self.hovered_card,
                    theme: t,
                    phase: self.card_start.elapsed().as_secs_f64(),
                    width: card_w as u16,
                    height: card_h as u16,
                };

                let card = render_card(&card_config);

                for cy in 0..card_h {
                    for cx in 0..card_w {
                        let src_idx = cy * card_w + cx;
                        let dst_idx = (y + cy) * area.width as usize + x + cx;
                        if src_idx < card.cells.len()
                            && dst_idx < plane.cells.len()
                            && !card.cells[src_idx].transparent
                        {
                            plane.cells[dst_idx] = card.cells[src_idx].clone();
                        }
                    }
                }

                const CARD_BASE: usize = 500;
                let mut zones = self.zones.borrow_mut();
                zones.register(
                    CARD_BASE + grid_idx,
                    x as u16,
                    y as u16,
                    card_w as u16,
                    card_h as u16,
                );
                drop(zones);
            }
        }

        // Scroll indicator with styled container
        let total_cards = self.filtered.len();
        let visible_cards = cols * (available_h / (card_h + 1)).max(1);
        if total_cards > visible_cards {
            let scroll_text = format!("↓ {} more", total_cards - visible_cards);
            let sx = area.width as usize - scroll_text.len() - 4;
            let sy = area.height as usize - 3;
            // Draw scroll indicator background
            for i in 0..scroll_text.len() + 4 {
                set_cell(&mut plane, sx + i, sy, ' ', t.fg, t.surface);
            }
            draw_text(&mut plane, sx + 1, sy, "▼", t.primary, t.surface, true);
            let rest: String = scroll_text.chars().skip(2).collect();
            draw_text(&mut plane, sx + 3, sy, &rest, t.fg_muted, t.surface, false);
        }

        // Status bar
        let status_y = area.height as usize - 1;
        for x in 0..area.width as usize {
            set_cell(&mut plane, x, status_y, ' ', t.fg, t.surface_elevated);
        }

        let hints = [
            "↑↓←→ nav",
            "Enter launch",
            "/ search",
            "Tab category",
            "t theme",
            "q quit",
        ];
        let mut hint_x = 2usize;
        for hint in hints.iter() {
            draw_text(
                &mut plane,
                hint_x,
                status_y,
                hint,
                t.primary,
                t.surface_elevated,
                false,
            );
            hint_x += hint.len() + 3;
        }

        // Mouse coordinates (right side)
        if let Some((mx, my)) = self.mouse_pos {
            let coords = format!("{}:{}", mx, my);
            let coords_x = area.width as usize - coords.len() - 2;
            if coords_x > hint_x {
                draw_text(
                    &mut plane,
                    coords_x,
                    status_y,
                    &coords,
                    t.fg_muted,
                    t.surface_elevated,
                    false,
                );
            }
        }

        // Status message (temporary) - toast style
        if let Some((ref msg, time)) = self.status_message {
            if time.elapsed() < Duration::from_secs(2) {
                let msg_y = area.height as usize / 2;
                let msg_x = ((area.width as usize).saturating_sub(msg.len() + 6)) / 2;
                let msg_w = msg.len() + 6;

                // Toast background
                for cx in 0..msg_w {
                    if msg_x + cx < area.width as usize {
                        set_cell(&mut plane, msg_x + cx, msg_y - 1, ' ', t.fg, t.warning);
                        set_cell(&mut plane, msg_x + cx, msg_y, ' ', t.fg, t.warning);
                        set_cell(&mut plane, msg_x + cx, msg_y + 1, ' ', t.fg, t.warning);
                    }
                }

                // Toast border
                for cx in 0..msg_w {
                    if msg_x + cx < area.width as usize {
                        set_cell(&mut plane, msg_x + cx, msg_y - 1, '─', t.warning, t.warning);
                        set_cell(&mut plane, msg_x + cx, msg_y + 1, '─', t.warning, t.warning);
                    }
                }
                set_cell(&mut plane, msg_x, msg_y - 1, '┌', t.warning, t.warning);
                set_cell(
                    &mut plane,
                    msg_x + msg_w - 1,
                    msg_y - 1,
                    '┐',
                    t.warning,
                    t.warning,
                );
                set_cell(&mut plane, msg_x, msg_y + 1, '└', t.warning, t.warning);
                set_cell(
                    &mut plane,
                    msg_x + msg_w - 1,
                    msg_y + 1,
                    '┘',
                    t.warning,
                    t.warning,
                );
                set_cell(&mut plane, msg_x, msg_y, '│', t.warning, t.warning);
                set_cell(
                    &mut plane,
                    msg_x + msg_w - 1,
                    msg_y,
                    '│',
                    t.warning,
                    t.warning,
                );

                // Message text
                draw_text(&mut plane, msg_x + 3, msg_y, msg, t.bg, t.warning, true);
            }
        }

        // Context menu
        if let Some((card_idx, mx, my)) = self.context_menu {
            if let Some(&ex_idx) = self.filtered.get(card_idx) {
                if let Some(ex) = self.examples.get(ex_idx) {
                    let menu_x = (mx as usize).min(area.width as usize - 20);
                    let menu_y = (my as usize).min(area.height as usize - 6);
                    let menu_w = 18usize;
                    let menu_h = 6usize;
                    let menu_items: [String; 4] = [
                        "▶ Launch".to_string(),
                        format!("  Copy: {}", ex.binary_name),
                        format!("  Filter: {}", ex.category),
                        "  Cancel".to_string(),
                    ];

                    // Menu background
                    for cy in 0..menu_h {
                        for cx in 0..menu_w {
                            set_cell(
                                &mut plane,
                                menu_x + cx,
                                menu_y + cy,
                                ' ',
                                t.fg,
                                t.surface_elevated,
                            );
                        }
                    }

                    // Border
                    for cx in 0..menu_w {
                        set_cell(
                            &mut plane,
                            menu_x + cx,
                            menu_y,
                            '─',
                            t.outline,
                            t.surface_elevated,
                        );
                        set_cell(
                            &mut plane,
                            menu_x + cx,
                            menu_y + menu_h - 1,
                            '─',
                            t.outline,
                            t.surface_elevated,
                        );
                    }
                    for cy in 0..menu_h {
                        set_cell(
                            &mut plane,
                            menu_x,
                            menu_y + cy,
                            '│',
                            t.outline,
                            t.surface_elevated,
                        );
                        set_cell(
                            &mut plane,
                            menu_x + menu_w - 1,
                            menu_y + cy,
                            '│',
                            t.outline,
                            t.surface_elevated,
                        );
                    }
                    set_cell(
                        &mut plane,
                        menu_x,
                        menu_y,
                        '┌',
                        t.outline,
                        t.surface_elevated,
                    );
                    set_cell(
                        &mut plane,
                        menu_x + menu_w - 1,
                        menu_y,
                        '┐',
                        t.outline,
                        t.surface_elevated,
                    );
                    set_cell(
                        &mut plane,
                        menu_x,
                        menu_y + menu_h - 1,
                        '└',
                        t.outline,
                        t.surface_elevated,
                    );
                    set_cell(
                        &mut plane,
                        menu_x + menu_w - 1,
                        menu_y + menu_h - 1,
                        '┘',
                        t.outline,
                        t.surface_elevated,
                    );

                    // Menu items
                    for (i, item) in menu_items.iter().enumerate() {
                        let selected = i == self.context_menu_selected;
                        let fg = if selected {
                            t.bg
                        } else if i == 0 {
                            t.primary
                        } else {
                            t.fg
                        };
                        let bg = if selected {
                            t.primary
                        } else {
                            t.surface_elevated
                        };
                        draw_text(&mut plane, menu_x + 2, menu_y + 1 + i, item, fg, bg, false);
                    }
                }
            }
        }

        // Tooltip on hover
        if let Some(ref text) = self.tooltip_text {
            if let Some((tx, ty)) = self.tooltip_pos {
                let tooltip_x = (tx as usize).min(area.width as usize - text.len() - 4);
                let tooltip_y = (ty as usize).saturating_sub(2);
                let tooltip_w = text.len() + 4;
                let tooltip_h = 3usize;

                // Background
                for cy in 0..tooltip_h {
                    for cx in 0..tooltip_w {
                        if tooltip_x + cx < area.width as usize
                            && tooltip_y + cy < area.height as usize
                        {
                            set_cell(
                                &mut plane,
                                tooltip_x + cx,
                                tooltip_y + cy,
                                ' ',
                                t.fg,
                                t.surface_elevated,
                            );
                        }
                    }
                }

                // Border
                for cx in 0..tooltip_w {
                    if tooltip_x + cx < area.width as usize && tooltip_y < area.height as usize {
                        set_cell(
                            &mut plane,
                            tooltip_x + cx,
                            tooltip_y,
                            '─',
                            t.outline,
                            t.surface_elevated,
                        );
                    }
                    if tooltip_x + cx < area.width as usize
                        && tooltip_y + tooltip_h - 1 < area.height as usize
                    {
                        set_cell(
                            &mut plane,
                            tooltip_x + cx,
                            tooltip_y + tooltip_h - 1,
                            '─',
                            t.outline,
                            t.surface_elevated,
                        );
                    }
                }
                for cy in 0..tooltip_h {
                    if tooltip_x < area.width as usize && tooltip_y + cy < area.height as usize {
                        set_cell(
                            &mut plane,
                            tooltip_x,
                            tooltip_y + cy,
                            '│',
                            t.outline,
                            t.surface_elevated,
                        );
                    }
                    if tooltip_x + tooltip_w - 1 < area.width as usize
                        && tooltip_y + cy < area.height as usize
                    {
                        set_cell(
                            &mut plane,
                            tooltip_x + tooltip_w - 1,
                            tooltip_y + cy,
                            '│',
                            t.outline,
                            t.surface_elevated,
                        );
                    }
                }

                // Text
                if tooltip_y + 1 < area.height as usize {
                    draw_text(
                        &mut plane,
                        tooltip_x + 2,
                        tooltip_y + 1,
                        text,
                        t.fg,
                        t.surface_elevated,
                        false,
                    );
                }
            }
        }

        // Help overlay
        if self.show_help {
            let help_w = 50usize;
            let help_h = 16usize;
            let help_x = ((area.width as usize).saturating_sub(help_w)) / 2;
            let help_y = ((area.height as usize).saturating_sub(help_h)) / 2;

            // Background
            for cy in 0..help_h {
                for cx in 0..help_w {
                    if help_x + cx < area.width as usize && help_y + cy < area.height as usize {
                        set_cell(
                            &mut plane,
                            help_x + cx,
                            help_y + cy,
                            ' ',
                            t.fg,
                            t.surface_elevated,
                        );
                    }
                }
            }

            // Border
            draw_rounded_border(
                &mut plane,
                Rect::new(help_x as u16, help_y as u16, help_w as u16, help_h as u16),
                t.primary,
                t.surface_elevated,
                true,
            );

            // Title
            let title = " Keyboard Shortcuts ";
            let title_x = help_x + (help_w - title.len()) / 2;
            draw_text(
                &mut plane,
                title_x,
                help_y + 1,
                title,
                t.primary,
                t.surface_elevated,
                true,
            );

            // Content
            let lines = [
                ("↑↓←→", "Navigate cards"),
                ("Enter", "Launch selected"),
                ("Space", "Show details"),
                ("/", "Focus search"),
                ("Tab", "Cycle categories"),
                ("t", "Cycle theme"),
                ("d", "Debug overlay"),
                ("i", "Input debug"),
                ("q / Esc", "Quit"),
                ("?", "Toggle this help"),
                ("", ""),
                ("Mouse", ""),
                ("Click", "Select card"),
                ("Double-click", "Launch example"),
                ("Right-click", "Context menu"),
                ("Scroll", "Navigate grid"),
            ];
            for (i, (key_text, desc)) in lines.iter().enumerate() {
                let y = help_y + 3 + i;
                if y < area.height as usize - 1 && !key_text.is_empty() {
                    draw_text(
                        &mut plane,
                        help_x + 3,
                        y,
                        key_text,
                        t.primary,
                        t.surface_elevated,
                        false,
                    );
                    draw_text(
                        &mut plane,
                        help_x + 18,
                        y,
                        desc,
                        t.fg,
                        t.surface_elevated,
                        false,
                    );
                }
            }
        }

        // Modal preview
        if self.modal_preview {
            if let Some(&ex_idx) = self.filtered.get(self.selected) {
                if let Some(ex) = self.examples.get(ex_idx) {
                    let modal_w = 50usize;
                    let modal_h = 12usize;
                    let modal_x = ((area.width as usize).saturating_sub(modal_w)) / 2;
                    let modal_y = ((area.height as usize).saturating_sub(modal_h)) / 2;

                    // Background
                    for cy in 0..modal_h {
                        for cx in 0..modal_w {
                            if modal_x + cx < area.width as usize
                                && modal_y + cy < area.height as usize
                            {
                                set_cell(
                                    &mut plane,
                                    modal_x + cx,
                                    modal_y + cy,
                                    ' ',
                                    t.fg,
                                    t.surface_elevated,
                                );
                            }
                        }
                    }

                    // Border
                    draw_rounded_border(
                        &mut plane,
                        Rect::new(
                            modal_x as u16,
                            modal_y as u16,
                            modal_w as u16,
                            modal_h as u16,
                        ),
                        t.primary,
                        t.surface_elevated,
                        true,
                    );

                    // Title
                    let title = format!(" {} ", ex.name);
                    let title_x = modal_x + (modal_w - title.len()) / 2;
                    draw_text(
                        &mut plane,
                        title_x,
                        modal_y + 1,
                        &title,
                        t.primary,
                        t.surface_elevated,
                        true,
                    );

                    // Category badge
                    let badge = format!(" {} ", ex.category.to_uppercase());
                    draw_text(
                        &mut plane,
                        modal_x + 2,
                        modal_y + 3,
                        &badge,
                        t.fg_on_accent,
                        category_color(t, ex.category),
                        false,
                    );

                    // Description
                    let desc: String = ex.description.chars().take(modal_w - 4).collect();
                    draw_text(
                        &mut plane,
                        modal_x + 2,
                        modal_y + 5,
                        &desc,
                        t.fg,
                        t.surface_elevated,
                        false,
                    );

                    // Preview lines
                    for (i, line) in ex.preview.iter().enumerate() {
                        let py = modal_y + 7 + i;
                        if py < modal_y + modal_h - 2 && py < area.height as usize {
                            let preview_line: String = line.chars().take(modal_w - 4).collect();
                            draw_text(
                                &mut plane,
                                modal_x + 2,
                                py,
                                &preview_line,
                                t.fg_subtle,
                                t.surface_elevated,
                                false,
                            );
                        }
                    }

                    // Footer
                    draw_text(
                        &mut plane,
                        modal_x + 2,
                        modal_y + modal_h - 2,
                        "Press Space or Esc to close",
                        t.fg_muted,
                        t.surface_elevated,
                        false,
                    );
                }
            }
        }

        // Debug overlay: show dirty region highlights
        if self.show_debug {
            let dbg_text = " DEBUG MODE [D] ";
            let dbg_w = dbg_text.len() + 4;
            let dbg_x = (area.width as usize).saturating_sub(dbg_w) / 2;
            let dbg_y = 2;

            for cx in 0..dbg_w {
                if dbg_x + cx < area.width as usize {
                    set_cell(&mut plane, dbg_x + cx, dbg_y, ' ', t.bg, t.error);
                }
            }
            for cx in 0..dbg_w {
                if dbg_x + cx < area.width as usize {
                    set_cell(&mut plane, dbg_x + cx, dbg_y, '─', t.bg, t.error);
                }
            }
            draw_text(&mut plane, dbg_x + 2, dbg_y, dbg_text, t.bg, t.error, true);

            let dbg_info = format!(
                "FPS:{:>3} | Cards:{:>2} | Selected:{:>2} | Hover:{:>2?} | Search:{:>5}",
                self.fps.load(Ordering::Relaxed),
                self.filtered.len(),
                self.selected,
                self.hovered_card,
                if self.search_active { "active" } else { "idle" }
            );
            let dbg_info_y = dbg_y + 2;
            draw_text(&mut plane, 2, dbg_info_y, &dbg_info, t.error, t.bg, false);
        }

        // Input debug overlay
        if self.show_input_debug {
            let log = self.event_log.borrow();
            if !log.is_empty() {
                let panel_w = 42usize;
                let panel_h = (log.len() + 2).min(18);
                let panel_x = 2usize;
                let panel_y = (area.height as usize).saturating_sub(panel_h + 2);

                // Background
                for cy in 0..panel_h {
                    for cx in 0..panel_w {
                        let px = panel_x + cx;
                        let py = panel_y + cy;
                        if px < area.width as usize && py < area.height as usize {
                            set_cell(&mut plane, px, py, ' ', t.fg, t.surface_elevated);
                        }
                    }
                }

                // Border
                draw_rounded_border(
                    &mut plane,
                    Rect::new(panel_x as u16, panel_y as u16, panel_w as u16, panel_h as u16),
                    t.warning,
                    t.surface_elevated,
                    true,
                );

                // Title
                draw_text(
                    &mut plane,
                    panel_x + 2,
                    panel_y + 1,
                    " Input Debug [i] ",
                    t.warning,
                    t.surface_elevated,
                    true,
                );

                // Events (newest first)
                for (i, (_, entry)) in log.iter().rev().take(panel_h.saturating_sub(3)).enumerate() {
                    let y = panel_y + 2 + i;
                    if y < area.height as usize {
                        let truncated: String = entry.chars().take(panel_w - 4).collect();
                        let fg = if entry.contains("CONSUMED") {
                            t.success
                        } else if entry.contains("ignored") {
                            t.error
                        } else {
                            t.fg_muted
                        };
                        draw_text(&mut plane, panel_x + 2, y, &truncated, fg, t.surface_elevated, false);
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Log input event for debug
        let key_desc = format!("{:?}", key.code);
        let mods = format!("{:?}", key.modifiers);
        let consumed = self.dispatch_key(key);
        let status = if consumed { "CONSUMED" } else { "ignored" };
        let log_entry = format!("{} {} {}", key_desc, mods, status);
        self.event_log.borrow_mut().push_back((std::time::Instant::now(), log_entry));
        while self.event_log.borrow().len() > 16 {
            self.event_log.borrow_mut().pop_front();
        }
        consumed
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.mouse_pos = Some((col, row));
        let consumed = self.dispatch_mouse(kind, col, row);
        if self.show_input_debug {
            let mouse_desc = format!("{:?} at ({}, {})", kind, col, row);
            let status = if consumed { "CONSUMED" } else { "ignored" };
            let log_entry = format!("{} {}", mouse_desc, status);
            self.event_log.borrow_mut().push_back((std::time::Instant::now(), log_entry));
            while self.event_log.borrow().len() > 16 {
                self.event_log.borrow_mut().pop_front();
            }
        }
        consumed
    }
}

    fn dispatch_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let sidebar_w = 14usize;
        let sidebar_start_y = 6usize;
        let _grid_start_x = sidebar_w + 2;
        let _grid_start_y = sidebar_start_y + 1;
        let _card_w = 28usize;
        let _card_h = 14usize;

        const PRIM_BASE: usize = 100;
        const PALETTE_BASE: usize = 200;
        const CAT_BASE: usize = 300;
        const FPS_ZONE: usize = 400;
        const CARD_BASE: usize = 500;

        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let y = row as usize;
                let x = col as usize;

                // Zone-based dispatch — query the registry populated during render
                let clicked_zone = self.zones.borrow().dispatch(col, row);
                if let Some(zone_id) = clicked_zone {
                    match zone_id {
                        // Theme palette swatches (PALETTE_BASE + i)
                        id if (PALETTE_BASE..PALETTE_BASE + 20).contains(&id) => {
                            let idx = id - PALETTE_BASE;
                            self.pending_theme = Some(idx);
                            self.apply_filter();
                            return true;
                        }
                        // FPS toggle
                        FPS_ZONE => {
                            self.show_fps = !self.show_fps;
                            return true;
                        }
                        // Primitives bar controls (PRIM_BASE + i)
                        id if (PRIM_BASE..PRIM_BASE + 5).contains(&id) => match id - PRIM_BASE {
                            0 => {
                                self.primitive_toggle = !self.primitive_toggle;
                                return true;
                            }
                            1 => {
                                self.primitive_slider = (self.primitive_slider + 0.1).min(1.0);
                                return true;
                            }
                            2 => {
                                self.primitive_checkbox = !self.primitive_checkbox;
                                return true;
                            }
                            3 => {
                                self.primitive_radio = (self.primitive_radio + 1) % 3;
                                return true;
                            }
                            4 => {
                                self.primitive_button = true;
                                self.primitive_button_time = Some(Instant::now());
                                return true;
                            }
                            _ => {}
                        },
                        // Sidebar categories (CAT_BASE + i)
                        id if (CAT_BASE..CAT_BASE + 4).contains(&id) => {
                            let cats: [Option<&str>; 4] =
                                [None, Some("apps"), Some("cookbook"), Some("tools")];
                            self.category_filter = cats[id - CAT_BASE];
                            self.apply_filter();
                            return true;
                        }
                        // Cards (CARD_BASE + grid_idx)
                        id if id >= CARD_BASE => {
                            let card_idx = id - CARD_BASE;
                            if card_idx < self.filtered.len() {
                                let now = Instant::now();
                                let is_double_click = self
                                    .last_click_time
                                    .zip(self.last_click_idx)
                                    .map(|(time, idx)| {
                                        idx == card_idx
                                            && now.duration_since(time).as_millis() < 300
                                    })
                                    .unwrap_or(false);
                                if is_double_click {
                                    self.selected = card_idx;
                                    self.launch_selected();
                                } else {
                                    self.selected = card_idx;
                                }
                                self.last_click_time = Some(now);
                                self.last_click_idx = Some(card_idx);
                                return true;
                            }
                        }
                        _ => {}
                    }
                }

                // Search bar click (no zone registered for this)
                if y == 3 && (2..30).contains(&x) {
                    self.search_active = true;
                    return true;
                }

                false
            }
            MouseEventKind::Down(MouseButton::Right) => {
                // Use zone dispatch for right-click on cards
                let clicked_zone = self.zones.borrow().dispatch(col, row);
                if let Some(zone_id) = clicked_zone {
                    if zone_id >= CARD_BASE {
                        let card_idx = zone_id - CARD_BASE;
                        if card_idx < self.filtered.len() {
                            self.selected = card_idx;
                            self.context_menu = Some((card_idx, col, row));
                            self.context_menu_selected = 0;
                            return true;
                        }
                    }
                }
                self.context_menu = None;
                false
            }
            MouseEventKind::ScrollDown if self.selected + 1 < self.filtered.len() => {
                self.selected += 1;
                true
            }
            MouseEventKind::ScrollUp if self.selected > 0 => {
                self.selected -= 1;
                true
            }
            MouseEventKind::Moved => {
                if let Some(btn_time) = self.primitive_button_time {
                    if btn_time.elapsed() >= Duration::from_secs(1) {
                        self.primitive_button = false;
                        self.primitive_button_time = None;
                    }
                }
                // Use zone dispatch for hover detection
                let hovered_zone = self.zones.borrow().dispatch(col, row);
                if let Some(zone_id) = hovered_zone {
                    if zone_id >= CARD_BASE {
                        let card_idx = zone_id - CARD_BASE;
                        if card_idx < self.filtered.len() {
                            self.hovered_card = Some(card_idx);
                            // Start or update tooltip timer
                            match self.tooltip_timer {
                                None => {
                                    self.tooltip_timer = Some(Instant::now());
                                    self.tooltip_pos = Some((col, row));
                                }
                                Some(time) => {
                                    if time.elapsed().as_millis() >= 500 {
                                        if let Some(&ex_idx) = self.filtered.get(card_idx) {
                                            if let Some(ex) = self.examples.get(ex_idx) {
                                                self.tooltip_text =
                                                    Some(ex.description.to_string());
                                                self.tooltip_pos = Some((col, row));
                                            }
                                        }
                                    }
                                }
                            }
                            return true;
                        }
                    }
                }
                self.hovered_card = None;
                self.tooltip_text = None;
                self.tooltip_timer = None;
                self.tooltip_pos = None;
                false
            }
            _ => false,
        }
    }
}

impl Showcase {
    fn dispatch_key(&mut self, key: KeyEvent) -> bool {
        // Help overlay takes priority
        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => {
                    self.show_help = false;
                    return true;
                }
                _ => return true,
            }
        }

        // Context menu takes priority
        if self.context_menu.is_some() {
            let menu_len = 4;
            match key.code {
                KeyCode::Esc => {
                    self.context_menu = None;
                    return true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.context_menu_selected = self.context_menu_selected.saturating_sub(1);
                    return true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.context_menu_selected = (self.context_menu_selected + 1).min(menu_len - 1);
                    return true;
                }
                KeyCode::Enter => {
                    let selected = self.context_menu_selected;
                    self.context_menu = None;
                    if selected == 0 {
                        self.launch_selected();
                    } else if selected == 1 {
                        if let Some(ex) = self.selected_example() {
                            println!("{}", ex.binary_name);
                            self.status_message =
                                Some((format!("Copied: {}", ex.binary_name), Instant::now()));
                        }
                    } else if selected == 2 {
                        let category = self.selected_example().map(|ex| ex.category);
                        if let Some(cat) = category {
                            self.category_filter = Some(cat);
                            self.apply_filter();
                            self.status_message =
                                Some((format!("Filtered: {}", cat), Instant::now()));
                        }
                    }
                    return true;
                }
                _ => return true,
            }
        }

        // Modal preview takes priority
        if self.modal_preview {
            match key.code {
                KeyCode::Esc | KeyCode::Char(' ') => {
                    self.modal_preview = false;
                    return true;
                }
                _ => return true,
            }
        }

        // Search mode
        if self.search_active {
            match key.code {
                KeyCode::Esc => {
                    self.search_active = false;
                    true
                }
                KeyCode::Enter => {
                    self.search_active = false;
                    self.launch_selected();
                    true
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.apply_filter();
                    true
                }
                KeyCode::Char(ch) => {
                    self.search_query.push(ch);
                    self.apply_filter();
                    true
                }
                _ => false,
            }
        } else {
            match key.code {
                KeyCode::Char('q') => {
                    self.should_quit.store(true, Ordering::SeqCst);
                    true
                }
                KeyCode::Char('?') => {
                    self.show_help = true;
                    true
                }
                KeyCode::Char(' ') => {
                    self.modal_preview = true;
                    true
                }
                KeyCode::Char('t') => {
                    let themes = Self::themes();
                    let current = themes
                        .iter()
                        .position(|(_, t)| t.name == self.theme.name)
                        .unwrap_or(0);
                    self.pending_theme = Some((current + 1) % themes.len());
                    self.apply_filter();
                    true
                }
                KeyCode::Char('d') => {
                    self.show_debug = !self.show_debug;
                    true
                }
                KeyCode::Char('i') => {
                    self.show_input_debug = !self.show_input_debug;
                    true
                }
                KeyCode::Char('/') => {
                    self.search_active = true;
                    true
                }
                KeyCode::Tab => {
                    let categories = [None, Some("apps"), Some("cookbook"), Some("tools")];
                    let current = categories
                        .iter()
                        .position(|&c| c == self.category_filter)
                        .unwrap_or(0);
                    self.category_filter = categories[(current + 1) % categories.len()];
                    self.apply_filter();
                    true
                }
                KeyCode::Char('1') => {
                    self.primitive_toggle = !self.primitive_toggle;
                    true
                }
                KeyCode::Char('2') => {
                    self.primitive_slider = (self.primitive_slider + 0.1).min(1.0);
                    true
                }
                KeyCode::Char('3') => {
                    self.primitive_checkbox = !self.primitive_checkbox;
                    true
                }
                KeyCode::Char('4') => {
                    self.primitive_radio = (self.primitive_radio + 1) % 3;
                    true
                }
                KeyCode::Char('5') => {
                    self.primitive_button = true;
                    self.primitive_button_time = Some(Instant::now());
                    true
                }
                KeyCode::Down => {
                    if self.selected + 1 < self.filtered.len() {
                        self.selected += 1;
                    } else if !self.filtered.is_empty() {
                        self.selected = 0;
                    }
                    true
                }
                KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    } else if !self.filtered.is_empty() {
                        self.selected = self.filtered.len() - 1;
                    }
                    true
                }
                KeyCode::Right => {
                    let cols = self.cols.get().max(1);
                    if !self.filtered.is_empty() {
                        self.selected = (self.selected + cols) % self.filtered.len();
                    }
                    true
                }
                KeyCode::Left => {
                    let cols = self.cols.get().max(1);
                    if !self.filtered.is_empty() {
                        self.selected = (self.selected + self.filtered.len()
                            - cols % self.filtered.len())
                            % self.filtered.len();
                    }
                    true
                }
                KeyCode::Enter => {
                    self.launch_selected();
                    true
                }
                _ => false,
            }
        }
    }
}

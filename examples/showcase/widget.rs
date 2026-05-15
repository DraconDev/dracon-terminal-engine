use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::render::CardConfig;
use chrono::{Local, Timelike};
use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::keybindings::actions;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

use crate::render::{category_color, draw_rounded_border, draw_text, render_card, set_cell};
use crate::state::{Showcase, SortField};

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
        // Re-render when dirty, or when the clock second changes,
        // or when animations are actively running (not just completed-but-uncleaned)
        if self.dirty {
            return true;
        }
        let now = Local::now();
        let current_second = now.num_seconds_from_midnight();
        if current_second != self.last_render_second {
            // Cache clock text when second changes to avoid formatting in render()
            *self.cached_clock_text.borrow_mut() = now.format("%H:%M:%S").to_string();
            return true;
        }
        // Only re-render if animations are actually still running
        if self.animations.has_active() {
            return true;
        }
        // Active scene always renders
        if self.scene_router.current().is_some() {
            return true;
        }
        false
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
        let now = Local::now();
        self.last_render_second = now.num_seconds_from_midnight();
    }
    fn focusable(&self) -> bool {
        true
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.scene_router.on_theme_change(theme);
    }

    fn render(&self, area: Rect) -> Plane {
        // If a scene is active, delegate to it with title bar
        if let Some(scene_name) = self.scene_router.current() {
            let mut plane = self.scene_router.render(area);
            let t = &self.theme;
            // Draw scene title bar at top
            let title = format!(" {} ", scene_name);
            for (i, c) in title.chars().enumerate() {
                if i < area.width as usize {
                    let idx = i;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].fg = t.primary;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
            // Fill rest of title bar
            for x in title.len()..area.width as usize {
                let idx = x;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ' ';
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].fg = t.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
            return plane;
        }

        let mut plane = Plane::new(0, area.width, area.height);
        let t = &self.theme;

        // Background fill
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Clear and rebuild zone registry for this frame
        self.zones.borrow_mut().clear();

        // Title bar with decorative border
        let title_text = " * Dracon Terminal Engine ";
        let title_x = 2usize;
        let title_y = 0usize;

        for (i, ch) in title_text.chars().enumerate() {
            let px = title_x + i;
            if px < area.width as usize {
                set_cell(&mut plane, px, title_y, ch, t.fg_on_accent, t.primary);
            }
        }

        // Fill rest of title bar
        for x in title_x + title_text.len()..area.width as usize {
            set_cell(&mut plane, x, title_y, ' ', t.primary, t.primary);
        }

        // Live clock (cached in needs_render, updated once per second)
        let clock_text = self.cached_clock_text.borrow();
        let clock_x = title_x + title_text.len() + 2;
        if clock_x + clock_text.len() < (area.width as usize).saturating_sub(10) {
            draw_text(
                &mut plane,
                clock_x,
                title_y,
                &clock_text,
                t.fg_on_accent,
                t.primary,
                Styles::empty(),
            );
        }
        drop(clock_text);

        // FPS counter (right-aligned)
        let mut right_x = area.width as usize;
        if self.show_fps {
            let fps_val = self.fps.load(Ordering::Relaxed);
            let fps_text = format!("{} FPS", fps_val);
            let fps_x = (area.width as usize).saturating_sub(fps_text.len()).saturating_sub(2);
            if fps_x > title_x + title_text.len() {
                draw_text(
                    &mut plane, fps_x, title_y, &fps_text, t.success, t.bg, Styles::empty(),
                );
                right_x = fps_x;
            }
        }

        // FPS toggle checkbox (left of FPS counter when visible)
        let fps_toggle = if self.show_fps { "[x] FPS" } else { "[ ] FPS" };
        let toggle_x = right_x.saturating_sub(fps_toggle.len() + 2);
        draw_text(
            &mut plane, toggle_x, title_y, fps_toggle, t.fg_muted, t.bg, Styles::empty(),
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
        let themes = self.themes();
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
        for (i, theme) in themes.iter().enumerate() {
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
                        '>'
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
        let stats_text = &self.cached_stats_text;
        let stats_start = 2usize;
        draw_text(
            &mut plane,
            stats_start,
            stats_y,
            stats_text,
            t.fg,
            t.bg,
            Styles::BOLD,
        );
        for x in stats_start + stats_text.len()..(area.width as usize).saturating_sub(2) {
            set_cell(&mut plane, x, stats_y, '─', t.outline, t.bg);
        }

        // Features highlight bar
        let features_y = 3usize;
        let phase = self.card_start.elapsed().as_secs_f64();
        crate::render::render_features_bar(&mut plane, t, features_y, phase);

        // Search bar with icon and better styling
        let search_y = 4usize;
        let search_active_bg = t.primary;
        let search_inactive_bg = t.surface;
        let search_bg = if self.search_active {
            search_active_bg
        } else {
            search_inactive_bg
        };

        // Draw search bar background
        for x in 2..(area.width as usize).saturating_sub(2) {
            set_cell(&mut plane, x, search_y, ' ', t.fg, search_bg);
        }

        // Search icon
        let search_icon = if self.search_active { ">" } else { "/" };
        draw_text(
            &mut plane,
            2,
            search_y,
            search_icon,
            t.fg_on_accent,
            search_bg,
            Styles::empty(),
        );

        // Search text or placeholder
        let content_x = 4usize;
        if self.search_active {
            if self.search_query.is_empty() {
                // Show blinking placeholder
                let blink = std::time::Instant::now()
                    .duration_since(self.card_start)
                    .as_secs_f64();
                let show_placeholder = (blink * 1.5).fract() < 0.85;
                if show_placeholder {
                    draw_text(
                        &mut plane,
                        content_x,
                        search_y,
                        "type to search...",
                        t.fg_muted,
                        search_bg,
                        Styles::empty(),
                    );
                }
                // Always show cursor when active, even if empty
                let cursor_x = content_x;
                if (blink * 2.0).fract() < 0.7 {
                    set_cell(
                        &mut plane,
                        cursor_x,
                        search_y,
                        '▋',
                        t.fg_on_accent,
                        search_bg,
                    );
                }
            } else {
                // Show actual query
                draw_text(
                    &mut plane,
                    content_x,
                    search_y,
                    &self.search_query,
                    t.fg_on_accent,
                    search_bg,
                    Styles::empty(),
                );
                // Cursor at end of query
                let cursor_x = content_x + self.search_query.len();
                let blink = std::time::Instant::now()
                    .duration_since(self.card_start)
                    .as_secs_f64();
                if (blink * 2.0).fract() < 0.7 {
                    set_cell(
                        &mut plane,
                        cursor_x,
                        search_y,
                        '▋',
                        t.fg_on_accent,
                        search_bg,
                    );
                }
            }
        } else {
            // Inactive - show hint
            draw_text(
                &mut plane,
                content_x,
                search_y,
                "press / to search",
                t.fg_muted,
                search_bg,
                Styles::empty(),
            );
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
                6,
                &feedback_text,
                feedback_color,
                t.bg,
                Styles::empty(),
            );
        }

        // Primitives bar - live widget demo
        let prim_y = 5usize;
        let state_0 = if self.primitive_toggle {
            " ✓ Toggle"
        } else {
            " ○ Toggle"
        };
        let state_1 = {
            let pos = ((self.primitive_slider * 10.0).round() as usize).min(10);
            let mut slider = String::with_capacity(20);
            slider.push_str(" Slider [");
            for i in 0..10 {
                slider.push(if i < pos { '▓' } else { '░' });
            }
            slider.push(']');
            slider
        };
        let state_2 = if self.primitive_checkbox {
            " ☑ Check"
        } else {
            " ☐ Check"
        };
        let state_3 = {
            let sel = self.primitive_radio;
            let opts = ["①", "②", "③"];
            let mut s = String::new();
            for (j, _o) in opts.iter().enumerate() {
                s.push_str(if j == sel { "●" } else { "○" });
            }
            format!(" Radio {}", s)
        };
        let state_4 = if self.primitive_button || self.primitive_button_time.is_some() {
            " ✦ Clicked!"
        } else {
            " ◇ Button"
        };
        let prim_controls: [(&str, &str); 5] = [
            ("", state_0),
            ("", &state_1),
            ("", state_2),
            ("", &state_3),
            ("", state_4),
        ];
        // Compute positions and register zones
        let mut prim_x = 2usize;
        let mut zones = self.zones.borrow_mut();
        const PRIM_BASE: usize = 100;
        for (i, (_key, state)) in prim_controls.iter().enumerate() {
            let total_w = state.len();
            zones.register(
                PRIM_BASE + i,
                prim_x as u16,
                prim_y as u16,
                total_w as u16,
                1,
            );
            prim_x += total_w + 4;
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
        for (i, (_key, state)) in prim_controls.iter().enumerate() {
            let hovered = hovered_prim == Some(i);
            let state_fg = if hovered { t.primary } else { t.fg };
            let state_bg = if hovered {
                t.surface_elevated
            } else {
                t.surface
            };
            draw_text(
                &mut plane, prim_x, prim_y, state, state_fg, state_bg, if hovered { Styles::BOLD } else { Styles::empty() },
            );
            prim_x += state.len() + 4;
        }

        // Category sidebar
        let sidebar_w = 14usize;
        let sidebar_start_y = 6usize;
        let categories = ["all", "apps", "input", "data", "cookbook", "tools", "accessibility"];
        const CAT_BASE: usize = 300;
        // Determine hovered sidebar category
        let hovered_cat = self
            .mouse_pos
            .filter(|(mx, my)| {
                let y = *my as usize;
                (*mx as usize) < sidebar_w && y >= sidebar_start_y && y < sidebar_start_y + categories.len() * 2
            })
            .map(|(_, my)| (my as usize).saturating_sub(sidebar_start_y) / 2)
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
                "all" => ("*", " ALL "),
                "apps" => ("#", " APPS "),
                "input" => ("=", " INPUT "),
                "data" => ("d", " DATA "),
                "cookbook" => ("o", " COOK "),
                "tools" => ("-", " TOOLS "),
                "accessibility" => ("+", " A11Y "),
                _ => (".", *cat),
            };
            let icon_fg = if is_hovered || is_active {
                t.primary
            } else {
                t.fg_muted
            };
            // Fill entire sidebar row with background first
            for x in 0..sidebar_w {
                set_cell(&mut plane, x, cat_y, ' ', fg, bg_cat);
            }
            draw_text(
                &mut plane,
                1,
                cat_y,
                icon,
                icon_fg,
                bg_cat,
                if is_active || is_hovered { Styles::BOLD } else { Styles::empty() },
            );
            draw_text(
                &mut plane,
                3,
                cat_y,
                label,
                fg,
                bg_cat,
                if is_active || is_hovered { Styles::BOLD } else { Styles::empty() },
            );
            // Register zone for this category
            let mut zones = self.zones.borrow_mut();
            zones.register(CAT_BASE + i, 0, cat_y as u16, sidebar_w as u16, 1);
            drop(zones);

            // Count badge (cached, no per-frame allocation)
            let count = self.cached_cat_counts[i];
            let count_str = format!("{:>2}", count);
            draw_text(&mut plane, 12, cat_y, &count_str, t.fg_muted, bg_cat, Styles::empty());
        }

        // Grid of cards — responsive sizing
        let grid_start_x = sidebar_w + 2;
        let grid_start_y = sidebar_start_y + 1;
        let available_w = (area.width as usize).saturating_sub(grid_start_x);
        let available_h = (area.height as usize).saturating_sub(grid_start_y).saturating_sub(2);

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

                if x + card_w > area.width as usize || y + card_h > (area.height as usize).saturating_sub(2) {
                    continue;
                }

                // Get hover animation offset
                let hover_offset =
                    if let Some(anim_id) = self.card_hover_anim.get(grid_idx).copied().flatten() {
                        self.animations.value(anim_id).unwrap_or(0.0)
                    } else {
                        0.0
                    };
                let offset_y = (hover_offset * -0.5) as i16; // Lift up by up to 0.5 rows
                let offset_x = (hover_offset * 0.5) as i16; // Slight right shift
                let draw_x = (x as i16 + offset_x).max(1) as usize;
                let draw_y = (y as i16 + offset_y).max(1) as usize;

                let run_count = self.run_counts.get(ex_idx).copied().unwrap_or(0);
                let card_config = CardConfig {
                    ex,
                    idx: grid_idx,
                    selected_idx: self.selected,
                    hovered_idx: self.hovered_card,
                    theme: t,
                    phase: self.card_start.elapsed().as_secs_f64(),
                    width: card_w as u16,
                    height: card_h as u16,
                    is_embedded: self.is_embedded(ex.name),
                    search_query: &self.search_query,
                    run_count,
                };

                render_card(&card_config, &mut plane, draw_x, draw_y);

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
            let sx = (area.width as usize).saturating_sub(scroll_text.len()).saturating_sub(4);
            let sy = (area.height as usize).saturating_sub(3);
            // Draw scroll indicator background
            for i in 0..scroll_text.len() + 4 {
                set_cell(&mut plane, sx + i, sy, ' ', t.fg, t.surface);
            }
            draw_text(&mut plane, sx + 1, sy, "▼", t.primary, t.surface, Styles::BOLD);
            let rest: String = scroll_text.chars().skip(2).collect();
            draw_text(&mut plane, sx + 3, sy, &rest, t.fg_muted, t.surface, Styles::empty());
        }

        // Status bar with gradient effect
        let status_y = (area.height as usize).saturating_sub(1);
        for x in 0..area.width as usize {
            let _gradient_ratio = x as f32 / area.width as f32;
            let bg = if x < area.width as usize / 2 {
                t.surface_elevated
            } else {
                t.surface
            };
            set_cell(&mut plane, x, status_y, ' ', t.fg, bg);
        }

        let hint = format!(
            "↑↓←→ nav | Enter launch | / search | Tab cat | {}",
            self.keybindings.format_hint(&[
                (actions::THEME, "theme"),
                (actions::HELP, "help"),
                (actions::BACK, "dismiss"),
                (actions::QUIT, "quit"),
            ])
        );
        let hint_x = 2usize;
        draw_text(
            &mut plane,
            hint_x,
            status_y,
            &hint,
            t.fg_muted,
            t.surface_elevated,
            Styles::empty(),
        );

        // Sort indicator
        let sort_arrow = if self.sort_ascending { "▲" } else { "▼" };
        let sort_text = format!(" {} {} ", sort_arrow, self.sort_field.label());
        let sort_x = hint_x + hint.len() + 2;
        draw_text(
            &mut plane,
            sort_x,
            status_y,
            &sort_text,
            t.primary,
            t.surface_elevated,
            Styles::empty(),
        );

        // Mouse coordinates (right side)
        if let Some((mx, my)) = self.mouse_pos {
            let coords = format!("{}:{}", mx, my);
            let coords_x = (area.width as usize).saturating_sub(coords.len()).saturating_sub(2);
            if coords_x > hint_x {
                draw_text(
                    &mut plane,
                    coords_x,
                    status_y,
                    &coords,
                    t.fg_muted,
                    t.surface_elevated,
                    Styles::empty(),
                );
            }
        }

        // Status message (temporary) - toast style with slide-in animation
        if let Some((ref msg, time)) = self.status_message {
            if time.elapsed() < Duration::from_secs(2) {
                let toast_offset = self
                    .toast_anim
                    .and_then(|id| self.animations.value(id))
                    .unwrap_or(0.0);

                let msg_y = (area.height as i16 / 2 + toast_offset as i16).max(1) as usize;
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
                draw_text(&mut plane, msg_x + 3, msg_y, msg, t.bg, t.warning, Styles::BOLD);
            }
        }

        // Returned-from toast (shown when coming back from an example)
        if let Ok(guard) = self.returned_from.lock() {
            if let Some((ref name, time)) = *guard {
                if time.elapsed() < Duration::from_secs(3) {
                    let help_key = self.keybindings.display(actions::HELP).unwrap_or("F1");
                    let msg = format!("Returned from {} — Press {} for help", name, help_key);
                    let msg_y = (area.height as usize).saturating_sub(3);
                    let msg_x = ((area.width as usize).saturating_sub(msg.len() + 4)) / 2;
                    let msg_w = msg.len() + 4;

                    // Toast background
                    for cx in 0..msg_w {
                        if msg_x + cx < area.width as usize {
                            set_cell(&mut plane, msg_x + cx, msg_y, ' ', t.fg, t.success);
                        }
                    }

                    // Toast border
                    for cx in 0..msg_w {
                        if msg_x + cx < area.width as usize {
                            set_cell(&mut plane, msg_x + cx, msg_y, '─', t.success, t.success);
                        }
                    }
                    set_cell(&mut plane, msg_x, msg_y, '┌', t.success, t.success);
                    set_cell(&mut plane, msg_x + msg_w - 1, msg_y, '┐', t.success, t.success);

                    // Message text
                    draw_text(&mut plane, msg_x + 2, msg_y, &msg, t.bg, t.success, Styles::BOLD);
                }
            }
        }

        // Context menu
        if let Some((card_idx, mx, my)) = self.context_menu {
            if let Some(&ex_idx) = self.filtered.get(card_idx) {
                if let Some(ex) = self.examples.get(ex_idx) {
                    let menu_x = (mx as usize).min((area.width as usize).saturating_sub(20));
                    let menu_y = (my as usize).min((area.height as usize).saturating_sub(6));
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
                        draw_text(&mut plane, menu_x + 2, menu_y + 1 + i, item, fg, bg, Styles::empty());
                    }
                }
            }
        }

        // Tooltip on hover
        if let Some(ref text) = self.tooltip_text {
            if let Some((tx, ty)) = self.tooltip_pos {
                let tooltip_x = (tx as usize).min((area.width as usize).saturating_sub(text.len()).saturating_sub(4));
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
                        Styles::empty(),
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
            let title = " ◊ Keyboard Shortcuts ";
            let title_x = help_x + (help_w - title.len()) / 2;
            draw_text(
                &mut plane,
                title_x,
                help_y + 1,
                title,
                t.primary,
                t.surface_elevated,
                Styles::BOLD,
            );

            // Content
            let kb_theme = self.keybindings.display(actions::THEME).unwrap_or("F2").to_string();
            let kb_help = self.keybindings.display(actions::HELP).unwrap_or("F1").to_string();
            let kb_quit = self.keybindings.display(actions::QUIT).unwrap_or("Ctrl+Q").to_string();
            let kb_back = self.keybindings.display(actions::BACK).unwrap_or("Esc").to_string();
            let kb_quit_back = format!("{} / {}", kb_quit, kb_back);
            let lines: Vec<(&str, &str)> = vec![
                ("↑↓←→", "Navigate cards"),
                ("Enter", "Launch selected"),
                ("Space", "Show details"),
                ("/", "Focus search"),
                ("Tab", "Cycle categories"),
                ("s", "Sort by field"),
                ("S", "Toggle sort order"),
                (&kb_theme, "Cycle theme"),
                ("d", "Debug overlay"),
                ("i", "Input debug"),
                ("F12", "Profiler"),
                (&kb_quit_back, "Quit / Dismiss"),
                (&kb_help, "Toggle this help"),
                ("", ""),
                ("Mouse", ""),
                ("Click", "Select card"),
                ("Double-click", "Launch example"),
                ("Right-click", "Context menu"),
                ("Scroll", "Navigate grid"),
            ];
            for (i, (key_text, desc)) in lines.iter().enumerate() {
                let y = help_y + 3 + i;
                if y < (area.height as usize).saturating_sub(1) && !key_text.is_empty() {
                    draw_text(
                        &mut plane,
                        help_x + 3,
                        y,
                        key_text,
                        t.primary,
                        t.surface_elevated,
                        Styles::empty(),
                    );
                    draw_text(
                        &mut plane,
                        help_x + 18,
                        y,
                        desc,
                        t.fg,
                        t.surface_elevated,
                        Styles::empty(),
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
                        Styles::BOLD,
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
                        Styles::empty(),
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
                        Styles::empty(),
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
                                Styles::empty(),
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
                        Styles::empty(),
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
            draw_text(&mut plane, dbg_x + 2, dbg_y, dbg_text, t.bg, t.error, Styles::BOLD);

            let dbg_info = format!(
                "FPS:{:>3} | Cards:{:>2} | Selected:{:>2} | Hover:{:>2?} | Search:{:>5}",
                self.fps.load(Ordering::Relaxed),
                self.filtered.len(),
                self.selected,
                self.hovered_card,
                if self.search_active { "active" } else { "idle" }
            );
            let dbg_info_y = dbg_y + 2;
            draw_text(&mut plane, 2, dbg_info_y, &dbg_info, t.error, t.bg, Styles::empty());
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
                    Rect::new(
                        panel_x as u16,
                        panel_y as u16,
                        panel_w as u16,
                        panel_h as u16,
                    ),
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
                    Styles::BOLD,
                );

                // Events (newest first)
                for (i, (_, entry)) in log.iter().rev().take(panel_h.saturating_sub(3)).enumerate()
                {
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
                        draw_text(
                            &mut plane,
                            panel_x + 2,
                            y,
                            &truncated,
                            fg,
                            t.surface_elevated,
                            Styles::empty(),
                        );
                    }
                }
            }
        }

        // Profiler overlay: shows performance metrics
        if self.show_profiler {
            let panel_w = 40usize;
            let panel_h = 12usize;
            let panel_x = (area.width as usize).saturating_sub(panel_w + 2);
            let panel_y = 2usize;

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
                t.primary,
                t.surface_elevated,
                true,
            );

            // Title
            let title = " PROFILER [F12] ";
            draw_text(&mut plane, panel_x + 2, panel_y + 1, title, t.primary, t.surface_elevated, Styles::BOLD);

            // Get stats from cached values
            let fps = self.fps.load(Ordering::Relaxed);
            let cards = self.filtered.len();
            let frame_elapsed = self.card_start.elapsed().as_secs_f64();
            let frame_ms = (frame_elapsed * 1000.0).min(999.0);

            // Profiler stats
            let stats = [
                ("FPS", format!("{}", fps)),
                ("Frame", format!("{:.1}ms", frame_ms)),
                ("Cards", format!("{}", cards)),
                ("Selected", format!("{}", self.selected)),
                ("Hover", format!("{:?}", self.hovered_card)),
                ("Search", if self.search_active { "active" } else { "idle" }.to_string()),
                ("Debug", if self.show_debug { "on" } else { "off" }.to_string()),
                ("Help", if self.show_help { "on" } else { "off" }.to_string()),
            ];

            for (i, (label, value)) in stats.iter().enumerate() {
                let row = panel_y + 3 + i / 2;
                let col_offset = if i % 2 == 0 { 0 } else { panel_w / 2 };
                if row < panel_y + panel_h - 1 {
                    let label_text = format!("{}:", label);
                    draw_text(
                        &mut plane,
                        panel_x + 2 + col_offset,
                        row,
                        &label_text,
                        t.fg_muted,
                        t.surface_elevated,
                        Styles::empty(),
                    );
                    draw_text(
                        &mut plane,
                        panel_x + 2 + col_offset + label_text.len(),
                        row,
                        value,
                        t.primary,
                        t.surface_elevated,
                        Styles::empty(),
                    );
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Any key press should trigger a re-render
        self.dirty = true;

        // Log input event for debug
        let key_desc = format!("{:?}", key.code);
        let mods = format!("{:?}", key.modifiers);
        let consumed = self.dispatch_key(key);
        let status = if consumed { "CONSUMED" } else { "ignored" };
        let log_entry = format!("{} {} {}", key_desc, mods, status);
        self.event_log
            .borrow_mut()
            .push_back((std::time::Instant::now(), log_entry));
        while self.event_log.borrow().len() > 16 {
            self.event_log.borrow_mut().pop_front();
        }
        consumed
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.mouse_pos = Some((col, row));
        let consumed = self.dispatch_mouse(kind, col, row);
        // Only re-render when a state change actually occurred (hover, click, scroll)
        // Mouse moves that don't change anything don't trigger re-renders
        if consumed {
            self.dirty = true;
        }
        if self.show_input_debug {
            let mouse_desc = format!("{:?} at ({}, {})", kind, col, row);
            let status = if consumed { "CONSUMED" } else { "ignored" };
            let log_entry = format!("{} {}", mouse_desc, status);
            self.event_log
                .borrow_mut()
                .push_back((std::time::Instant::now(), log_entry));
            while self.event_log.borrow().len() > 16 {
                self.event_log.borrow_mut().pop_front();
            }
        }
        consumed
    }
}

impl Showcase {
    fn dispatch_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // If a scene is active, delegate to it
        if self.scene_router.current().is_some() {
            return self.scene_router.handle_mouse(kind, col, row);
        }

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
                        id if (PALETTE_BASE..PALETTE_BASE + self.themes().len()).contains(&id) => {
                            let idx = id - PALETTE_BASE;
                            self.pending_theme = Some(idx);
                            self.apply_filter();
                            let cur_theme = self.theme.clone();
                            self.scene_router.on_theme_change(&cur_theme);
                            *self.pending_app_theme.lock().unwrap() = Some(cur_theme);
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
                                // Slider: click position determines direction
                                // Slider zone starts at prim_x=2, format: " Slider [▓▓▓░░░░░░░]"
                                // Thumb position is at ~2 + 9 + 1 + thumb_pos
                                let slider_zone_x = 2usize;
                                let slider_label_w = 9usize; // " Slider ["
                                let track_w = 10usize;
                                let thumb_pos = (self.primitive_slider * track_w as f32).round() as usize;
                                let thumb_x = slider_zone_x + slider_label_w + thumb_pos;
                                if (col as usize) < thumb_x {
                                    self.primitive_slider = (self.primitive_slider - 0.1).max(0.0);
                                } else {
                                    self.primitive_slider = (self.primitive_slider + 0.1).min(1.0);
                                }
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
                        id if (CAT_BASE..CAT_BASE + 7).contains(&id) => {
                            let cats: [Option<&str>; 7] =
                                [None, Some("apps"), Some("input"), Some("data"),
                                 Some("cookbook"), Some("tools"), Some("accessibility")];
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
                if y == 4 && (2..(self.area.width.saturating_sub(2)) as usize).contains(&x) {
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
                            // Start hover animation if entering new card
                            if self.hovered_card != Some(card_idx) {
                                let anim_id =
                                    self.animations.start(0.0, 1.0, Duration::from_millis(200));
                                if card_idx >= self.card_hover_anim.len() {
                                    self.card_hover_anim.resize(card_idx + 1, None);
                                }
                                self.card_hover_anim[card_idx] = Some(anim_id);
                            }
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
                // Clear hover and start exit animations
                if let Some(prev_hover) = self.hovered_card {
                    if prev_hover < self.card_hover_anim.len() {
                        // Start reverse animation
                        let anim_id = self.animations.start(1.0, 0.0, Duration::from_millis(150));
                        self.card_hover_anim[prev_hover] = Some(anim_id);
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
        // If a scene is active, handle global keys first, then delegate
        if self.scene_router.current().is_some() {
            if self.keybindings.matches(actions::QUIT, &key) {
                self.should_quit.store(true, Ordering::SeqCst);
                return true;
            }
            if self.keybindings.matches(actions::THEME, &key) {
                let themes = self.themes();
                let current = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
                self.pending_theme = Some((current + 1) % themes.len());
                self.apply_filter();
                self.scene_router.on_theme_change(&self.theme);
                *self.pending_app_theme.lock().unwrap() = Some(self.theme.clone());
                return true;
            }
            if self.keybindings.matches(actions::BACK, &key) {
                if !self.scene_router.handle_key(key) {
                    self.scene_router.pop();
                }
                return true;
            }
            return self.scene_router.handle_key(key);
        }

        // Help overlay takes priority
        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                return true;
            }
            return true;
        }

        // Context menu takes priority
        if self.context_menu.is_some() {
            let menu_len = 4;
            if self.keybindings.matches(actions::BACK, &key) {
                self.context_menu = None;
                return true;
            }
            match key.code {
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
            if self.keybindings.matches(actions::BACK, &key) || key.code == KeyCode::Char(' ') {
                self.modal_preview = false;
                return true;
            }
            return true;
        }

        // Search mode
        if self.search_active {
            if self.keybindings.matches(actions::BACK, &key) {
                self.search_active = false;
                return true;
            }
            if self.keybindings.matches(actions::SUBMIT, &key) {
                self.search_active = false;
                self.launch_selected();
                return true;
            }
            match key.code {
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
            // Esc is a no-op in the launcher to prevent accidental quit
            if key.code == KeyCode::Esc && key.modifiers.is_empty() {
                return true;
            }
            if self.keybindings.matches(actions::QUIT, &key) {
                self.should_quit.store(true, Ordering::SeqCst);
                return true;
            }
            if self.keybindings.matches(actions::HELP, &key) {
                self.show_help = true;
                return true;
            }
            if self.keybindings.matches(actions::THEME, &key) {
                let themes = self.themes();
                let current = themes
                    .iter()
                    .position(|t| t.name == self.theme.name)
                    .unwrap_or(0);
                self.pending_theme = Some((current + 1) % themes.len());
                self.apply_filter();
                self.scene_router.on_theme_change(&self.theme);
                *self.pending_app_theme.lock().unwrap() = Some(self.theme.clone());
                return true;
            }
            match key.code {
                KeyCode::Char(' ') => {
                    self.modal_preview = true;
                    true
                }
                KeyCode::Char('d') => {
                    self.show_debug = !self.show_debug;
                    true
                }
                KeyCode::F(12) => {
                    self.show_profiler = !self.show_profiler;
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
                    let categories = [None, Some("apps"), Some("input"), Some("data"), Some("cookbook"), Some("tools"), Some("accessibility")];
                    let current = categories
                        .iter()
                        .position(|&c| c == self.category_filter)
                        .unwrap_or(0);
                    self.category_filter = categories[(current + 1) % categories.len()];
                    self.apply_filter();
                    true
                }
                KeyCode::Char('s') if !self.search_active => {
                    let fields = SortField::all();
                    let cur = fields.iter().position(|f| *f == self.sort_field).unwrap_or(0);
                    self.sort_field = fields[(cur + 1) % fields.len()];
                    self.apply_filter();
                    true
                }
                KeyCode::Char('S') => {
                    self.sort_ascending = !self.sort_ascending;
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

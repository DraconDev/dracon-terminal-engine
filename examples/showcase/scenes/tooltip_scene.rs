//! Embedded Tooltip scene for the showcase.
//!
//! Demonstrates tooltips on diverse hover targets: toolbar icons,
//! sidebar menu items, status indicators, and action buttons.
//! Includes tooltip history sidebar.

use crate::scenes::shared_helpers::draw_text;
use dracon_terminal_engine::compositor::plane::{Color, Plane};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

#[derive(Clone)]
struct TooltipEntry {
    source: String,
    text: String,
    color: Color,
}

struct ToolbarItem {
    icon: char,
    label: &'static str,
    tooltip: &'static str,
    color: Color,
}

struct SidebarItem {
    icon: char,
    label: &'static str,
    tooltip: &'static str,
    badge: Option<&'static str>,
}

struct ActionButton {
    label: &'static str,
    tooltip: &'static str,
    color: Color,
}

pub struct TooltipScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    hovered: Option<String>,
    tooltip_history: Vec<TooltipEntry>,
    dirty: bool,
    area: std::cell::Cell<Rect>,
}

impl TooltipScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            hovered: None,
            tooltip_history: Vec::new(),
            dirty: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn toolbar_items(&self) -> Vec<ToolbarItem> {
        let t = &self.theme;
        vec![
            ToolbarItem { icon: '◈', label: "Files", tooltip: "Browse project files and folders", color: t.primary },
            ToolbarItem { icon: '◇', label: "Search", tooltip: "Search across files, symbols, commands", color: t.secondary },
            ToolbarItem { icon: '◈', label: "Git", tooltip: "Source control: commit, push, pull", color: t.success },
            ToolbarItem { icon: '◆', label: "Debug", tooltip: "Launch and manage debug sessions", color: t.warning },
            ToolbarItem { icon: '▣', label: "Ext", tooltip: "Install and manage editor extensions", color: t.info },
            ToolbarItem { icon: '☰', label: "Config", tooltip: "Configure editor preferences", color: t.fg_muted },
            ToolbarItem { icon: '◉', label: "Alerts", tooltip: "3 unread notifications", color: t.error },
            ToolbarItem { icon: '⊚', label: "User", tooltip: "Signed in as user@terminal", color: t.fg },
        ]
    }

    fn sidebar_items(&self) -> Vec<SidebarItem> {
        vec![
            SidebarItem { icon: '▸', label: "Explorer", tooltip: "File tree and workspace browser", badge: None },
            SidebarItem { icon: '▸', label: "Search", tooltip: "Find in files with regex support", badge: None },
            SidebarItem { icon: '▸', label: "Source Control", tooltip: "Git changes and history", badge: Some("3") },
            SidebarItem { icon: '▸', label: "Debug", tooltip: "Run and debug configurations", badge: None },
            SidebarItem { icon: '▸', label: "Extensions", tooltip: "Installed: 12 extensions", badge: Some("1") },
            SidebarItem { icon: '▸', label: "Output", tooltip: "View terminal and task output", badge: None },
            SidebarItem { icon: '▸', label: "Problems", tooltip: "2 errors, 5 warnings", badge: Some("7") },
            SidebarItem { icon: '▸', label: "Terminal", tooltip: "Integrated terminal (bash)", badge: None },
        ]
    }

    fn action_buttons(&self) -> Vec<ActionButton> {
        let t = &self.theme;
        vec![
            ActionButton { label: "New File", tooltip: "Create a new file (Ctrl+N)", color: t.primary },
            ActionButton { label: "Open Folder", tooltip: "Open a project folder", color: t.secondary },
            ActionButton { label: "Save All", tooltip: "Save all unsaved files (Ctrl+Shift+S)", color: t.success },
            ActionButton { label: "Format", tooltip: "Format document with formatter", color: t.info },
            ActionButton { label: "Refactor", tooltip: "Rename symbol across workspace", color: t.warning },
            ActionButton { label: "Run Task", tooltip: "Execute a configured task", color: t.primary },
        ]
    }

    fn add_history(&mut self, source: &str, text: String, color: Color) {
        self.tooltip_history.insert(0, TooltipEntry {
            source: source.to_string(),
            text,
            color,
        });
        if self.tooltip_history.len() > 8 {
            self.tooltip_history.pop();
        }
    }

    fn get_tooltip_for_key(&self, key: &str) -> (&str, Color) {
        let t = &self.theme;
        if let Some(idx) = key.strip_prefix("toolbar:").and_then(|s| s.parse::<usize>().ok()) {
            let items = self.toolbar_items();
            if idx < items.len() { return (items[idx].tooltip, items[idx].color); }
        } else if let Some(idx) = key.strip_prefix("sidebar:").and_then(|s| s.parse::<usize>().ok()) {
            let items = self.sidebar_items();
            if idx < items.len() { return (items[idx].tooltip, t.primary); }
        } else if let Some(idx) = key.strip_prefix("action:").and_then(|s| s.parse::<usize>().ok()) {
            let buttons = self.action_buttons();
            if idx < buttons.len() { return (buttons[idx].tooltip, buttons[idx].color); }
        }
        ("Unknown", t.fg_muted)
    }

    fn render_toolbar(&self, plane: &mut Plane, y: u16) {
        let t = &self.theme;

        for x in 0..plane.width {
            let idx = (y * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        let items = self.toolbar_items();
        let mut x = 2u16;
        for (i, item) in items.iter().enumerate() {
            let key = format!("toolbar:{}", i);
            let is_hovered = self.hovered.as_deref() == Some(key.as_str());

            let idx = (y * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = item.icon;
                plane.cells[idx].fg = if is_hovered { t.primary } else { item.color };
                plane.cells[idx].bg = if is_hovered { t.hover_bg } else { t.surface };
                plane.cells[idx].transparent = false;
            }

            let lx = x + 2;
            for (j, ch) in item.label.chars().enumerate() {
                let idx = (y * plane.width + lx + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = if is_hovered { t.primary } else { t.fg_muted };
                    plane.cells[idx].bg = if is_hovered { t.hover_bg } else { t.surface };
                    plane.cells[idx].transparent = false;
                }
            }

            x += item.label.len() as u16 + 4;
        }

        for x in 0..plane.width {
            let idx = ((y + 1) * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
    }

    fn render_sidebar(&self, plane: &mut Plane, y: u16, w: u16, h: u16) {
        let t = &self.theme;

        for sy in 0..h {
            for sx in 0..w {
                let idx = ((y + sy) * plane.width + sx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        draw_text(plane, 1, y, " Explorer", t.fg, t.surface, true);

        let items = self.sidebar_items();
        for (i, item) in items.iter().enumerate() {
            let iy = y + 2 + i as u16;
            if iy >= y + h { break; }

            let key = format!("sidebar:{}", i);
            let is_hovered = self.hovered.as_deref() == Some(key.as_str());
            let row_bg = if is_hovered { t.hover_bg } else { t.surface };

            for sx in 0..w {
                let idx = (iy * plane.width + sx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = row_bg;
                }
            }

            let fg = if is_hovered { t.primary } else { t.fg_muted };
            let icon_idx = (iy * plane.width + 2) as usize;
            if icon_idx < plane.cells.len() {
                plane.cells[icon_idx].char = item.icon;
                plane.cells[icon_idx].fg = fg;
            }

            draw_text(plane, 4, iy, item.label, if is_hovered { t.primary } else { t.fg }, row_bg, is_hovered);

            if let Some(badge) = item.badge {
                let bx = w.saturating_sub(badge.len() as u16 + 2);
                draw_text(plane, bx, iy, badge, t.error, row_bg, true);
            }
        }
    }

    fn render_actions(&self, plane: &mut Plane, x: u16, y: u16, w: u16) {
        let t = &self.theme;

        draw_text(plane, x, y, "Actions", t.primary, t.bg, true);
        for dx in 0..w {
            let idx = ((y + 1) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        let buttons = self.action_buttons();
        let cols = 2usize;
        let col_w = w / cols as u16;
        for (i, btn) in buttons.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let bx = x + (col * col_w as usize) as u16;
            let by = y + 2 + row as u16 * 3;
            if by + 2 >= plane.height { break; }

            let key = format!("action:{}", i);
            let is_hovered = self.hovered.as_deref() == Some(key.as_str());
            let btn_bg = if is_hovered { t.hover_bg } else { t.surface };
            let btn_fg = if is_hovered { t.primary } else { btn.color };

            let btn_w = (col_w - 2).min(btn.label.len() as u16 + 4);
            for dx in 0..btn_w {
                let idx = (by * plane.width + bx + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = btn_bg;
                    plane.cells[idx].transparent = false;
                }
            }

            let label = format!(" {} ", btn.label);
            draw_text(plane, bx + 1, by, &label, btn_fg, btn_bg, is_hovered);
        }
    }

    fn render_tooltip_popup(&self, plane: &mut Plane, target_x: u16, target_y: u16, text: &str, color: Color) {
        let t = &self.theme;
        let tw = text.len() as u16 + 4;
        let th = 3u16;

        let ty = if target_y + 1 + th < plane.height { target_y + 1 } else { target_y.saturating_sub(th) };
        let tx = target_x.min(plane.width.saturating_sub(tw + 1));

        // Shadow
        for dy in 0..th + 1 {
            for dx in 1..tw + 1 {
                let sy = ty + dy;
                let sx = tx + dx;
                if sy < plane.height && sx < plane.width {
                    let idx = (sy * plane.width + sx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = Color::Rgb(0, 0, 0);
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }

        // Background
        for dy in 0..th {
            for dx in 0..tw {
                let py = ty + dy;
                let px = tx + dx;
                if py < plane.height && px < plane.width {
                    let idx = (py * plane.width + px) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].fg = t.fg;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }

        // Border
        for dx in 0..tw {
            let px = tx + dx;
            if px < plane.width {
                let top = (ty * plane.width + px) as usize;
                let bot = ((ty + th - 1) * plane.width + px) as usize;
                if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = color; }
                if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = color; }
            }
        }
        for dy in 0..th {
            let py = ty + dy;
            if py < plane.height {
                let left = (py * plane.width + tx) as usize;
                let right = (py * plane.width + tx + tw - 1) as usize;
                if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = color; }
                if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = color; }
            }
        }
        for (ch, cx, cy) in [('╭', tx, ty), ('╮', tx + tw - 1, ty), ('╰', tx, ty + th - 1), ('╯', tx + tw - 1, ty + th - 1)] {
            if cy < plane.height && cx < plane.width {
                let idx = (cy * plane.width + cx) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = color; }
            }
        }

        // Text
        draw_text(plane, tx + 2, ty + 1, text, t.fg, t.surface_elevated, false);

        // Arrow
        if ty > 0 {
            let arrow_x = target_x.min(tx + tw - 2).max(tx);
            if arrow_x < plane.width && ty > 0 {
                let arrow_idx = ((ty - 1) * plane.width + arrow_x) as usize;
                if arrow_idx < plane.cells.len() {
                    plane.cells[arrow_idx].char = '▼';
                    plane.cells[arrow_idx].fg = color;
                }
            }
        }
    }

    fn render_history(&self, plane: &mut Plane, x: u16, y: u16, w: u16, h: u16) {
        let t = &self.theme;

        draw_text(plane, x, y, "Tooltip History", t.secondary, t.bg, true);
        for dx in 0..w {
            let idx = ((y + 1) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        if self.tooltip_history.is_empty() {
            draw_text(plane, x, y + 2, "Hover to start", t.fg_muted, t.bg, false);
        } else {
            for (i, entry) in self.tooltip_history.iter().take((h as usize).saturating_sub(2)).enumerate() {
                let hy = y + 2 + i as u16;
                if hy >= y + h { break; }

                draw_text(plane, x, hy, &entry.source, entry.color, t.bg, true);
                let max_text = (w as usize).saturating_sub(entry.source.len() + 3);
                let text: String = entry.text.chars().take(max_text).collect();
                draw_text(plane, x + entry.source.len() as u16 + 1, hy, &text, t.fg_muted, t.bg, false);
            }
        }
    }

    fn render_help(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 44u16.min(area.width.saturating_sub(4));
        let hh = 12u16.min(area.height.saturating_sub(4));
        let hx = (area.width - hw) / 2;
        let hy = (area.height - hh) / 2;

        for y in hy..hy + hh {
            for x in hx..hx + hw {
                let idx = (y * plane.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
        }
        for x in hx + 1..hx + hw - 1 {
            let top = (hy * plane.width + x) as usize;
            let bot = ((hy + hh - 1) * plane.width + x) as usize;
            if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
            if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
        }
        for y in hy + 1..hy + hh - 1 {
            let left = (y * plane.width + hx) as usize;
            let right = (y * plane.width + hx + hw - 1) as usize;
            if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
            if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
        }
        for (ch, cx, cy) in [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)] {
            let idx = (cy * plane.width + cx) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = t.outline; }
        }

        let title = "Tooltip Demo Help";
        let tx = hx + (hw - title.len() as u16) / 2;
        draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

        let shortcuts = [
            ("Mouse", "Hover any element for tooltip"),
            ("", "Toolbar icons, sidebar items, buttons"),
            ("?", "Toggle this help"),
            ("B/Esc", "Back to showcase"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            if !key.is_empty() {
                draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
            }
            draw_text(plane, hx + 14, row, desc, t.fg, t.surface_elevated, false);
        }
    }
}

impl Scene for TooltipScene {
    fn scene_id(&self) -> &str { "tooltip" }

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
        draw_text(&mut plane, 2, 0, " Tooltips ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 2), 0,
                  &theme_label, t.secondary, t.bg, false);

        // Toolbar (y=1)
        self.render_toolbar(&mut plane, 1);

        // Layout
        let sidebar_w = 16u16;
        let history_w = 20u16;

        // Sidebar
        self.render_sidebar(&mut plane, 2, sidebar_w, area.height.saturating_sub(5));

        // Sidebar right border
        for y in 2..area.height.saturating_sub(3) {
            let idx = (y * plane.width + sidebar_w) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Main content: action buttons
        let main_x = sidebar_w + 2;
        let main_w = area.width.saturating_sub(sidebar_w + history_w + 4);
        self.render_actions(&mut plane, main_x, 2, main_w);

        // History divider
        let hist_div = area.width.saturating_sub(history_w + 1);
        for y in 2..area.height.saturating_sub(3) {
            let idx = (y * plane.width + hist_div) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // History panel
        let hist_x = hist_div + 2;
        self.render_history(&mut plane, hist_x, 2, history_w - 2, area.height.saturating_sub(5));

        // Status indicators
        let status_y = area.height.saturating_sub(4);
        draw_text(&mut plane, main_x, status_y, "Status", t.primary, t.bg, true);
        for dx in 0..main_w {
            let idx = ((status_y + 1) * plane.width + main_x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
        let indicators = [
            ("●", "Connected", t.success),
            ("◐", "Syncing", t.warning),
            ("○", "Offline", t.fg_muted),
        ];
        let mut ix = main_x;
        for (icon, label, color) in indicators {
            draw_text(&mut plane, ix, status_y + 2, icon, color, t.bg, true);
            draw_text(&mut plane, ix + 2, status_y + 2, label, color, t.bg, false);
            ix += label.len() as u16 + 5;
        }

        // Tooltip popup
        if let Some(ref key) = self.hovered {
            let area = self.area.get();
            let sidebar_w = 16u16;
            let history_w = 20u16;
            let main_x = sidebar_w + 2;
            let main_w = area.width.saturating_sub(sidebar_w + history_w + 4);

            if let Some(idx) = key.strip_prefix("toolbar:").and_then(|s| s.parse::<usize>().ok()) {
                let items = self.toolbar_items();
                if idx < items.len() {
                    let mut x = 2u16;
                    for item in items.iter().take(idx) { x += item.label.len() as u16 + 4; }
                    self.render_tooltip_popup(&mut plane, x, 1, items[idx].tooltip, items[idx].color);
                }
            } else if let Some(idx) = key.strip_prefix("sidebar:").and_then(|s| s.parse::<usize>().ok()) {
                let items = self.sidebar_items();
                if idx < items.len() {
                    self.render_tooltip_popup(&mut plane, 2, 4 + idx as u16, items[idx].tooltip, t.primary);
                }
            } else if let Some(idx) = key.strip_prefix("action:").and_then(|s| s.parse::<usize>().ok()) {
                let buttons = self.action_buttons();
                if idx < buttons.len() {
                    let col = idx % 2;
                    let row = idx / 2;
                    let col_w = main_w / 2;
                    let bx = main_x + (col * col_w as usize) as u16;
                    let by = 4 + row as u16 * 3;
                    self.render_tooltip_popup(&mut plane, bx, by, buttons[idx].tooltip, buttons[idx].color);
                }
            }
        }

        // Footer
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(" Hover for tooltips | {}:help | {}:back ", help_key, back_key);
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
            self.render_help(&mut plane, area);
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
        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        match kind {
            MouseEventKind::Moved => {
                let new_hovered = self.hit_test(col, row);
                if new_hovered != self.hovered {
                    if let Some(ref key) = new_hovered {
                        let (text, color) = self.get_tooltip_for_key(key);
                        let source = key.split(':').next().unwrap_or("").to_string();
                        let text_owned = text.to_string();
                        self.add_history(&source, text_owned, color);
                    }
                    self.hovered = new_hovered;
                    self.dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl TooltipScene {
    fn hit_test(&self, col: u16, row: u16) -> Option<String> {
        let area = self.area.get();
        let sidebar_w = 16u16;
        let history_w = 20u16;
        let main_x = sidebar_w + 2;
        let main_w = area.width.saturating_sub(sidebar_w + history_w + 4);

        // Toolbar (row 1)
        if row == 1 {
            let items = self.toolbar_items();
            let mut x = 2u16;
            for (i, item) in items.iter().enumerate() {
                let item_w = item.label.len() as u16 + 4;
                if col >= x && col < x + item_w {
                    return Some(format!("toolbar:{}", i));
                }
                x += item_w;
            }
        }

        // Sidebar (rows 4..)
        if col < sidebar_w && row >= 4 {
            let idx = (row - 4) as usize;
            let items = self.sidebar_items();
            if idx < items.len() {
                return Some(format!("sidebar:{}", idx));
            }
        }

        // Action buttons
        if col >= main_x && col < main_x + main_w && row >= 4 {
            let buttons = self.action_buttons();
            let col_w = main_w / 2;
            for (i, _btn) in buttons.iter().enumerate() {
                let c = i % 2;
                let r = i / 2;
                let bx = main_x + (c * col_w as usize) as u16;
                let by = 4 + r as u16 * 3;
                let btn_w = (col_w - 2).min(16);
                if row == by && col >= bx && col < bx + btn_w {
                    return Some(format!("action:{}", i));
                }
            }
        }

        None
    }
}

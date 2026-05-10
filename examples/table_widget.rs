#![allow(missing_docs)]
//! Rich Table Widget Demo — user directory with badges, search, and detail panel.
//!
//! Features:
//! - 25 demo users with names, roles, departments, statuses, last active
//! - Per-column cell rendering via `with_cell_text_fn`
//! - Search/filter by name or department (`/` to search, Esc to clear)
//! - Theme cycling (`t`)
//! - Help overlay (`?`)
//! - Detail panel with colored status badge and user metadata
//! - Rounded border card layout
//! - Scroll indicator
//! - Mouse hover highlighting and click selection
//!
//! Controls:
//!   ↑/↓         — navigate rows
//!   Enter       — select row
//!   /           — search
//!   Esc         — clear search / dismiss help
//!   t           — cycle theme
//!   ?           — help overlay
//!   q           — quit

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Column, Table};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use std::os::fd::AsFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
struct User {
    name: String,
    role: String,
    department: String,
    status: UserStatus,
    last_active: String,
    email: String,
    join_date: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum UserStatus {
    Active,
    Away,
    Offline,
    Busy,
    OnLeave,
}

impl UserStatus {
    fn label(self) -> &'static str {
        match self {
            UserStatus::Active => "Active",
            UserStatus::Away => "Away",
            UserStatus::Offline => "Offline",
            UserStatus::Busy => "Busy",
            UserStatus::OnLeave => "On Leave",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            UserStatus::Active => "󰄬",
            UserStatus::Away => "󰒲",
            UserStatus::Offline => "󰤄",
            UserStatus::Busy => "󰀦",
            UserStatus::OnLeave => "󰒲",
        }
    }

    fn color(self, theme: Theme) -> Color {
        match self {
            UserStatus::Active => theme.success,
            UserStatus::Away => theme.warning,
            UserStatus::Offline => theme.fg_muted,
            UserStatus::Busy => theme.error,
            UserStatus::OnLeave => theme.info,
        }
    }
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn build_users() -> Vec<User> {
    vec![
        User {
            name: "Alice Chen".into(),
            role: "Engineering Lead".into(),
            department: "Engineering".into(),
            status: UserStatus::Active,
            last_active: "2m ago".into(),
            email: "alice@corp.io".into(),
            join_date: "2021-03-15".into(),
        },
        User {
            name: "Bob Martinez".into(),
            role: "Senior Dev".into(),
            department: "Engineering".into(),
            status: UserStatus::Active,
            last_active: "now".into(),
            email: "bob@corp.io".into(),
            join_date: "2020-07-22".into(),
        },
        User {
            name: "Carol White".into(),
            role: "Product Manager".into(),
            department: "Product".into(),
            status: UserStatus::Busy,
            last_active: "1h ago".into(),
            email: "carol@corp.io".into(),
            join_date: "2019-11-08".into(),
        },
        User {
            name: "David Kim".into(),
            role: "Designer".into(),
            department: "Design".into(),
            status: UserStatus::Away,
            last_active: "30m ago".into(),
            email: "david@corp.io".into(),
            join_date: "2022-01-10".into(),
        },
        User {
            name: "Eve Johnson".into(),
            role: "DevOps Engineer".into(),
            department: "Engineering".into(),
            status: UserStatus::Active,
            last_active: "5m ago".into(),
            email: "eve@corp.io".into(),
            join_date: "2021-09-01".into(),
        },
        User {
            name: "Frank Liu".into(),
            role: "QA Lead".into(),
            department: "Engineering".into(),
            status: UserStatus::Offline,
            last_active: "3h ago".into(),
            email: "frank@corp.io".into(),
            join_date: "2020-04-18".into(),
        },
        User {
            name: "Grace Park".into(),
            role: "Data Scientist".into(),
            department: "Data".into(),
            status: UserStatus::Active,
            last_active: "15m ago".into(),
            email: "grace@corp.io".into(),
            join_date: "2022-06-30".into(),
        },
        User {
            name: "Henry Wilson".into(),
            role: "Sales Director".into(),
            department: "Sales".into(),
            status: UserStatus::OnLeave,
            last_active: "2d ago".into(),
            email: "henry@corp.io".into(),
            join_date: "2018-02-14".into(),
        },
        User {
            name: "Ivy Thompson".into(),
            role: "Marketing Lead".into(),
            department: "Marketing".into(),
            status: UserStatus::Active,
            last_active: "1h ago".into(),
            email: "ivy@corp.io".into(),
            join_date: "2021-12-05".into(),
        },
        User {
            name: "Jack Brown".into(),
            role: "Backend Dev".into(),
            department: "Engineering".into(),
            status: UserStatus::Busy,
            last_active: "now".into(),
            email: "jack@corp.io".into(),
            join_date: "2023-01-20".into(),
        },
        User {
            name: "Karen Davis".into(),
            role: "HR Manager".into(),
            department: "HR".into(),
            status: UserStatus::Active,
            last_active: "45m ago".into(),
            email: "karen@corp.io".into(),
            join_date: "2019-08-12".into(),
        },
        User {
            name: "Leo Garcia".into(),
            role: "Frontend Dev".into(),
            department: "Engineering".into(),
            status: UserStatus::Away,
            last_active: "2h ago".into(),
            email: "leo@corp.io".into(),
            join_date: "2022-03-08".into(),
        },
        User {
            name: "Mia Rodriguez".into(),
            role: "UX Researcher".into(),
            department: "Design".into(),
            status: UserStatus::Active,
            last_active: "10m ago".into(),
            email: "mia@corp.io".into(),
            join_date: "2023-04-15".into(),
        },
        User {
            name: "Noah Taylor".into(),
            role: "Security Engineer".into(),
            department: "Engineering".into(),
            status: UserStatus::Offline,
            last_active: "5h ago".into(),
            email: "noah@corp.io".into(),
            join_date: "2020-10-01".into(),
        },
        User {
            name: "Olivia Lee".into(),
            role: "Finance Analyst".into(),
            department: "Finance".into(),
            status: UserStatus::Active,
            last_active: "20m ago".into(),
            email: "olivia@corp.io".into(),
            join_date: "2021-05-22".into(),
        },
        User {
            name: "Paul Walker".into(),
            role: "Support Lead".into(),
            department: "Support".into(),
            status: UserStatus::Busy,
            last_active: "now".into(),
            email: "paul@corp.io".into(),
            join_date: "2022-08-01".into(),
        },
        User {
            name: "Quinn Adams".into(),
            role: "Mobile Dev".into(),
            department: "Engineering".into(),
            status: UserStatus::Active,
            last_active: "1h ago".into(),
            email: "quinn@corp.io".into(),
            join_date: "2023-02-28".into(),
        },
        User {
            name: "Rachel Scott".into(),
            role: "Content Strategist".into(),
            department: "Marketing".into(),
            status: UserStatus::OnLeave,
            last_active: "1w ago".into(),
            email: "rachel@corp.io".into(),
            join_date: "2020-01-15".into(),
        },
        User {
            name: "Sam Clark".into(),
            role: "Platform Engineer".into(),
            department: "Engineering".into(),
            status: UserStatus::Active,
            last_active: "3m ago".into(),
            email: "sam@corp.io".into(),
            join_date: "2021-07-10".into(),
        },
        User {
            name: "Tina Hall".into(),
            role: "Accountant".into(),
            department: "Finance".into(),
            status: UserStatus::Away,
            last_active: "4h ago".into(),
            email: "tina@corp.io".into(),
            join_date: "2019-04-20".into(),
        },
        User {
            name: "Uma Patel".into(),
            role: "ML Engineer".into(),
            department: "Data".into(),
            status: UserStatus::Active,
            last_active: "8m ago".into(),
            email: "uma@corp.io".into(),
            join_date: "2022-11-11".into(),
        },
        User {
            name: "Victor King".into(),
            role: "Recruiter".into(),
            department: "HR".into(),
            status: UserStatus::Offline,
            last_active: "6h ago".into(),
            email: "victor@corp.io".into(),
            join_date: "2023-03-05".into(),
        },
        User {
            name: "Wendy Young".into(),
            role: "Tech Writer".into(),
            department: "Engineering".into(),
            status: UserStatus::Active,
            last_active: "25m ago".into(),
            email: "wendy@corp.io".into(),
            join_date: "2022-05-18".into(),
        },
        User {
            name: "Xavier Lopez".into(),
            role: "SRE".into(),
            department: "Engineering".into(),
            status: UserStatus::Busy,
            last_active: "now".into(),
            email: "xavier@corp.io".into(),
            join_date: "2021-01-30".into(),
        },
        User {
            name: "Yara Nguyen".into(),
            role: "Product Designer".into(),
            department: "Design".into(),
            status: UserStatus::Active,
            last_active: "12m ago".into(),
            email: "yara@corp.io".into(),
            join_date: "2023-06-01".into(),
        },
        User {
            name: "Zack Wright".into(),
            role: "Intern".into(),
            department: "Engineering".into(),
            status: UserStatus::Away,
            last_active: "1h ago".into(),
            email: "zack@corp.io".into(),
            join_date: "2024-01-15".into(),
        },
    ]
}

struct TableApp {
    id: WidgetId,
    all_users: Vec<User>,
    filtered_users: Vec<User>,
    table: Table<User>,
    theme: Theme,
    show_help: bool,
    show_search: bool,
    search_query: String,
    sort_column: Option<usize>,
    sort_ascending: bool,
    area: Rect,
    dirty: bool,
    should_quit: Arc<AtomicBool>,
    keybindings: KeybindingSet,
}

impl TableApp {
    fn new(should_quit: Arc<AtomicBool>, theme: Theme, keybindings: KeybindingSet) -> Self {
        let all_users = build_users();
        let filtered_users = all_users.clone();

        let columns = vec![
            Column {
                header: "󰣉 Name".into(),
                width: 18,
            },
            Column {
                header: "󰠨 Role".into(),
                width: 20,
            },
            Column {
                header: "󰉋 Dept".into(),
                width: 14,
            },
            Column {
                header: "󰄬 Status".into(),
                width: 12,
            },
            Column {
                header: "󰃰 Active".into(),
                width: 10,
            },
        ];

        let cell_fn = move |user: &User, col: usize| -> String {
            match col {
                0 => user.name.clone(),
                1 => user.role.clone(),
                2 => user.department.clone(),
                3 => format!("{} {}", user.status.icon(), user.status.label()),
                4 => user.last_active.clone(),
                _ => String::new(),
            }
        };

        let mut table = Table::new(columns)
            .with_theme(theme)
            .with_rows(filtered_users.clone())
            .with_cell_text_fn(cell_fn)
            .on_select(|_user| {});
        table.set_visible_count(15);

        Self {
            id: WidgetId::new(0),
            all_users,
            filtered_users,
            table,
            theme,
            show_help: false,
            show_search: false,
            search_query: String::new(),
            sort_column: None,
            sort_ascending: true,
            area: Rect::new(0, 0, 80, 24),
            dirty: true,
            should_quit,
            keybindings,
        }
    }

    fn sort_users(users: &mut [User], col: usize, ascending: bool) {
        users.sort_by(|a, b| {
            let ord = match col {
                0 => a.name.cmp(&b.name),
                1 => a.role.cmp(&b.role),
                2 => a.department.cmp(&b.department),
                3 => a.status.label().cmp(b.status.label()),
                4 => a.last_active.cmp(&b.last_active),
                _ => std::cmp::Ordering::Equal,
            };
            if ascending {
                ord
            } else {
                ord.reverse()
            }
        });
    }

    fn rebuild_table(&mut self) {
        // Apply filter
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.filtered_users = self.all_users.clone();
        } else {
            self.filtered_users = self
                .all_users
                .iter()
                .filter(|u| {
                    u.name.to_lowercase().contains(&query)
                        || u.department.to_lowercase().contains(&query)
                        || u.role.to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
        }

        // Apply sort
        if let Some(col) = self.sort_column {
            Self::sort_users(&mut self.filtered_users, col, self.sort_ascending);
        }

        let columns = vec![
            Column {
                header: "󰣉 Name".into(),
                width: 18,
            },
            Column {
                header: "󰠨 Role".into(),
                width: 20,
            },
            Column {
                header: "󰉋 Dept".into(),
                width: 14,
            },
            Column {
                header: "󰄬 Status".into(),
                width: 12,
            },
            Column {
                header: "󰃰 Active".into(),
                width: 10,
            },
        ];
        let cell_fn = move |user: &User, col: usize| -> String {
            match col {
                0 => user.name.clone(),
                1 => user.role.clone(),
                2 => user.department.clone(),
                3 => format!("{} {}", user.status.icon(), user.status.label()),
                4 => user.last_active.clone(),
                _ => String::new(),
            }
        };

        let sort_col = self.sort_column;
        let sort_asc = self.sort_ascending;
        let mut new_table = Table::new(columns)
            .with_theme(self.theme)
            .with_rows(self.filtered_users.clone())
            .with_cell_text_fn(cell_fn)
            .on_select(|_user| {});
        new_table.set_visible_count((self.area.height.saturating_sub(10)) as usize);
        new_table.set_sort(sort_col, sort_asc);
        self.table = new_table;
        self.dirty = true;
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::dark(),
            Theme::light(),
            Theme::cyberpunk(),
            Theme::dracula(),
            Theme::nord(),
            Theme::catppuccin_mocha(),
            Theme::gruvbox_dark(),
            Theme::tokyo_night(),
            Theme::solarized_dark(),
            Theme::solarized_light(),
            Theme::one_dark(),
            Theme::rose_pine(),
            Theme::kanagawa(),
            Theme::everforest(),
            Theme::monokai(),
            Theme::warm(),
            Theme::cool(),
            Theme::forest(),
            Theme::sunset(),
            Theme::mono(),
        ];
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];
        self.table.on_theme_change(&self.theme);
        self.rebuild_table();
    }

    fn selected_user(&self) -> Option<&User> {
        let idx = self.table.selected_index();
        self.filtered_users.get(idx)
    }

    fn toggle_sort(&mut self, col: usize) {
        if self.sort_column == Some(col) {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = Some(col);
            self.sort_ascending = true;
        }
        self.rebuild_table();
    }
}

impl Widget for TableApp {
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
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        self.dirty || self.table.needs_render()
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
        self.table.clear_dirty();
    }
    fn focusable(&self) -> bool {
        true
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.rebuild_table();
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = self.theme;

        // Background
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        let margin = 1u16;
        let _card_x = margin;
        let card_w = area.width.saturating_sub(margin * 2);
        let header_h = 3u16;
        let search_h = if self.show_search { 2u16 } else { 0u16 };
        let detail_h = 6u16;
        let table_h = area
            .height
            .saturating_sub(header_h + search_h + detail_h + margin + 1);

        // === HEADER BAR ===
        let title = " 󰓫 User Directory ";
        let count = format!("{} users ", self.filtered_users.len());
        let theme_name = format!(" {} ", t.name);

        for x in 0..card_w {
            let idx = (margin + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].fg = t.fg;
            }
            // Row 1 of header
            let idx2 = (area.width + margin + x) as usize;
            if idx2 < plane.cells.len() {
                plane.cells[idx2].bg = t.surface;
            }
            // Row 2 of header
            let idx3 = ((2) * area.width + margin + x) as usize;
            if idx3 < plane.cells.len() {
                plane.cells[idx3].bg = t.surface;
            }
        }

        // Title
        for (i, c) in title.chars().enumerate() {
            let idx = (area.width + margin + 1 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
                plane.cells[idx].bg = t.surface;
            }
        }

        // Count badge
        let count_x = card_w.saturating_sub(count.len() as u16 + theme_name.len() as u16 + 2);
        for (i, c) in count.chars().enumerate() {
            let idx = (area.width + margin + count_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
            }
        }

        // Theme badge
        let theme_x = card_w.saturating_sub(theme_name.len() as u16);
        for (i, c) in theme_name.chars().enumerate() {
            let idx = (area.width + margin + theme_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_on_accent;
                plane.cells[idx].bg = t.primary_active;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Subtitle
        let subtitle = "Employee directory with live status and search";
        for (i, c) in subtitle.chars().enumerate() {
            let idx = (2 * area.width + margin + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
            }
        }

        // === SEARCH BAR ===
        let mut table_y = header_h;
        if self.show_search {
            let search_y = header_h;
            for x in 0..card_w {
                let idx = (search_y * area.width + margin + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                }
                let idx2 = ((search_y + 1) * area.width + margin + x) as usize;
                if idx2 < plane.cells.len() {
                    plane.cells[idx2].bg = t.surface_elevated;
                }
            }
            let search_label = "󰼈 Search: ";
            for (i, c) in search_label.chars().enumerate() {
                let idx = (search_y * area.width + margin + 2 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                    plane.cells[idx].bg = t.surface_elevated;
                }
            }
            for (i, c) in self.search_query.chars().enumerate() {
                let idx = (search_y * area.width + margin + 12 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                    plane.cells[idx].bg = t.input_bg;
                }
            }
            // Cursor
            let cursor_x = margin + 12 + self.search_query.len() as u16;
            let cursor_idx = (search_y * area.width + cursor_x) as usize;
            if cursor_idx < plane.cells.len() {
                plane.cells[cursor_idx].bg = t.primary;
                plane.cells[cursor_idx].fg = t.fg_on_accent;
                plane.cells[cursor_idx].char = ' ';
            }
            table_y = header_h + search_h;
        }

        // === TABLE ===
        let table_rect = Rect::new(margin, table_y, card_w, table_h.max(5));
        let table_plane = self.table.render(table_rect);
        blit(&mut plane, &table_plane, margin, table_y);

        // === DETAIL PANEL ===
        let detail_y = table_y + table_h;
        if detail_y + detail_h <= area.height && self.selected_user().is_some() {
            // Rounded border for detail panel
            draw_rounded_border(&mut plane, margin, detail_y, card_w, detail_h, t);

            // Fill background inside border
            for dy in 1..detail_h.saturating_sub(1) {
                for dx in 1..card_w.saturating_sub(1) {
                    let idx = ((detail_y + dy) * area.width + margin + dx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface;
                    }
                }
            }

            if let Some(user) = self.selected_user() {
                let content_y = detail_y + 1;
                let content_x = margin + 2;

                // Name + status badge
                let name_text = format!("{}  ", user.name);
                for (i, c) in name_text.chars().enumerate() {
                    let idx = (content_y * area.width + content_x + i as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.fg;
                        plane.cells[idx].style = Styles::BOLD;
                        plane.cells[idx].bg = t.surface;
                    }
                }

                // Status badge
                let badge = format!(" {} {} ", user.status.icon(), user.status.label());
                let badge_x = content_x + name_text.len() as u16;
                let badge_color = user.status.color(t);
                for (i, c) in badge.chars().enumerate() {
                    let idx = (content_y * area.width + badge_x + i as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.fg_on_accent;
                        plane.cells[idx].bg = badge_color;
                        plane.cells[idx].style = Styles::BOLD;
                    }
                }

                // Row 2: Role + Department
                let row2 = format!("󰠨 {}    󰉋 {}", user.role, user.department);
                for (i, c) in row2.chars().enumerate() {
                    let idx = ((content_y + 1) * area.width + content_x + i as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.fg_muted;
                        plane.cells[idx].bg = t.surface;
                    }
                }

                // Row 3: Email + Join date
                let row3 = format!("󰇮 {}    󰃰 Joined: {}", user.email, user.join_date);
                for (i, c) in row3.chars().enumerate() {
                    let idx = ((content_y + 2) * area.width + content_x + i as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.fg_muted;
                        plane.cells[idx].bg = t.surface;
                    }
                }

                // Row 4: Hint
                let hint = "Press ? for help  |  / to search  |  t for theme  |  q to quit";
                for (i, c) in hint.chars().enumerate() {
                    let idx = ((content_y + 3) * area.width + content_x + i as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.fg_subtle;
                        plane.cells[idx].bg = t.surface;
                    }
                }
            }
        }

        // === STATUS BAR ===
        let status_y = area.height.saturating_sub(1);
        let hint = format!(
            "t: theme | ?: help | Esc: dismiss | ↑↓: nav | Enter: select | q: quit | {} users",
            self.filtered_users.len()
        );
        for (i, c) in hint
            .chars()
            .take((area.width as usize).saturating_sub(2))
            .enumerate()
        {
            let idx = (status_y * plane.width + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
            }
        }

        // === CARD BORDER ===
        draw_rounded_border(
            &mut plane,
            margin,
            0,
            card_w,
            area.height.saturating_sub(1),
            t,
        );

        // === HELP OVERLAY ===
        if self.show_help {
            render_help_overlay(&mut plane, area, t);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        let kb = &self.keybindings;

        if self.show_help {
            if kb.matches(actions::DISMISS, &key) || kb.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true;
        }

        if self.show_search {
            match key.code {
                KeyCode::Esc => {
                    self.show_search = false;
                    self.search_query.clear();
                    self.rebuild_table();
                    return true;
                }
                KeyCode::Enter => {
                    self.show_search = false;
                    self.rebuild_table();
                    return true;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.rebuild_table();
                    return true;
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.rebuild_table();
                    return true;
                }
                _ => {}
            }
        }

        if kb.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if kb.matches(actions::THEME, &key) {
            self.cycle_theme();
            return true;
        }
        if kb.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if key.code == KeyCode::Char('/') && key.modifiers.is_empty() {
            self.show_search = !self.show_search;
            if !self.show_search {
                self.search_query.clear();
                self.rebuild_table();
            }
            self.dirty = true;
            return true;
        }
        if kb.matches(actions::BACK, &key) {
            if !self.search_query.is_empty() {
                self.search_query.clear();
                self.rebuild_table();
                self.dirty = true;
            }
            return true;
        }

        let handled = self.table.handle_key(key);
        if handled {
            self.dirty = true;
        }
        handled
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let margin = 1u16;
        let card_w = self.area.width.saturating_sub(margin * 2);
        let header_h = 3u16;
        let search_h = if self.show_search { 2u16 } else { 0u16 };
        let table_y = header_h + search_h;
        let table_h = self.area.height.saturating_sub(header_h + search_h + 7);

        // Only handle mouse within table area
        if col >= margin && col < margin + card_w && row >= table_y && row < table_y + table_h {
            let local_col = col - margin;
            let local_row = row - table_y;

            // Header click for sorting
            if let MouseEventKind::Down(MouseButton::Left) = kind {
                if local_row == 0 {
                    let mut col_x: u16 = 0;
                    let column_widths = [18u16, 20, 14, 12, 10];
                    for (i, w) in column_widths.iter().enumerate() {
                        if local_col >= col_x && local_col < col_x + w {
                            self.toggle_sort(i);
                            return true;
                        }
                        col_x += w;
                    }
                }
            }

            let handled = self.table.handle_mouse(kind, local_col, local_row);
            if handled {
                self.dirty = true;
            }
            return handled;
        }
        false
    }
}

fn render_help_overlay(plane: &mut Plane, area: Rect, t: Theme) {
    let w = 52u16.min(area.width - 4);
    let h = 14u16.min(area.height - 4);
    let x = (area.width - w) / 2;
    let y = (area.height - h) / 2;

    // Background
    for py in 0..h {
        for px in 0..w {
            let idx = ((y + py) * area.width + x + px) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface_elevated;
                plane.cells[idx].transparent = false;
            }
        }
    }

    draw_rounded_border(plane, x, y, w, h, t);

    let title = "User Directory Help";
    let title_x = x + (w - title.len() as u16) / 2;
    draw_text(
        plane,
        title_x,
        y + 1,
        title,
        t.primary,
        t.surface_elevated,
        true,
    );

    let shortcuts = [
        ("↑/↓", "Navigate rows"),
        ("Enter", "Select row"),
        ("Click col", "Sort column"),
        ("/", "Search/filter"),
        ("Esc", "Clear search"),
        ("t", "Cycle theme"),
        ("?", "Toggle help"),
        ("q", "Quit"),
    ];

    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = y + 3 + i as u16;
        draw_text(plane, x + 3, row, key, t.primary, t.surface_elevated, true);
        draw_text(plane, x + 14, row, desc, t.fg, t.surface_elevated, false);
    }

    let hint = "Press ? or Esc to close";
    draw_text(
        plane,
        x + 3,
        y + h - 1,
        hint,
        t.fg_muted,
        t.surface_elevated,
        false,
    );
}

fn draw_rounded_border(plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, t: Theme) {
    if w < 3 || h < 2 {
        return;
    }
    for row in y..y + h {
        for col in x..x + w {
            let idx = (row * plane.width + col) as usize;
            if idx >= plane.cells.len() {
                continue;
            }
            let is_border = row == y || row == y + h - 1 || col == x || col == x + w - 1;
            if is_border {
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].char = if row == y && col == x {
                    '╭'
                } else if row == y && col == x + w - 1 {
                    '╮'
                } else if row == y + h - 1 && col == x {
                    '╰'
                } else if row == y + h - 1 && col == x + w - 1 {
                    '╯'
                } else if row == y || row == y + h - 1 {
                    '─'
                } else {
                    '│'
                };
                plane.cells[idx].transparent = false;
            }
        }
    }
}

fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = (y * plane.width + x + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch,
                fg,
                bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false,
                skip: false,
            };
        }
    }
}

fn blit(dst: &mut Plane, src: &Plane, dx: u16, dy: u16) {
    for (i, cell) in src.cells.iter().enumerate() {
        if cell.transparent {
            continue;
        }
        let x = (i % src.width as usize) as u16 + dx;
        let y = (i / src.width as usize) as u16 + dy;
        let idx = (y * dst.width + x) as usize;
        if idx < dst.cells.len() && x < dst.width && y < dst.height {
            dst.cells[idx] = cell.clone();
        }
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let keybindings = KeybindingSet::from_config(&resolve_keybindings());
    let kb_input = keybindings.clone();

    let (_w, _h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let app_widget = TableApp::new(should_quit.clone(), Theme::nord(), keybindings);

    App::new()?
        .title("Table Widget Demo")
        .fps(30)
        .theme(Theme::nord())
        .on_input(move |key| {
            if kb_input.matches(actions::QUIT, key)
                && key.kind == KeyEventKind::Press
            {
                should_quit.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        })
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
            }
        })
        .run(move |ctx| {
            let (w, h) = ctx.compositor().size();
            ctx.add_plane(app_widget.render(Rect::new(0, 0, w, h)));
        })
}

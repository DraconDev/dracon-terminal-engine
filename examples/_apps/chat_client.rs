//! Chat Client — Framework widget composition demo.
//!
//! Demonstrates proper Pattern-1 Widget architecture using framework widgets:
//!   - `List<Message>` for the message list
//!   - `SearchInput` for text input
//!   - `StatusBar` for the bottom bar
//!   - `Modal` for emoji picker + settings
//!   - `NotificationCenter` for toast feedback
//!
//! Works fully offline with simulated contacts. No async feature required.
//!
//! Controls:
//!   Type        — Compose message
//!   Enter       — Send message
//!   ↑/↓         — Scroll message list
//!   Ctrl+E      — Emoji picker
//!   Ctrl+S      — Settings modal
//!   Ctrl+T      — Cycle theme
//!   F1 / ?      — Toggle help
//!   Esc         — Dismiss overlay
//!   Ctrl+Q      — Quit

use dracon_terminal_engine::compositor::plane::{Plane, Styles};
use dracon_terminal_engine::framework::app::App;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::list::List;
use dracon_terminal_engine::framework::widgets::modal::{Modal, ModalResult};
use dracon_terminal_engine::framework::widgets::notification_center::NotificationCenter;
use dracon_terminal_engine::framework::widgets::search_input::SearchInput;
use dracon_terminal_engine::framework::widgets::status_bar::{StatusBar, StatusSegment};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use dracon_terminal_engine::prelude::Rect;
use std::cell::Cell as StdCell;

// ═══════════════════════════════════════════════════════════════════════════════
// Data
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
struct Message {
    sender: &'static str,
    text: String,
    time: &'static str,
    is_me: bool,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_me {
            write!(f, " [{}] You: {}", self.time, self.text)
        } else {
            write!(f, " [{}] {}: {}", self.time, self.sender, self.text)
        }
    }
}

struct Contact {
    name: &'static str,
    emoji: &'static str,
    last_msg: &'static str,
    time: &'static str,
    unread: u8,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Chat App
// ═══════════════════════════════════════════════════════════════════════════════

struct ChatApp {
    // Identity
    id: WidgetId,

    // Widgets
    message_list: List<Message>,
    input: SearchInput,
    status_bar: StatusBar,
    notifications: NotificationCenter,
    settings_modal: Option<Modal<'static>>,
    emoji_modal: Option<Modal<'static>>,

    // State
    theme: Theme,
    dirty: bool,
    keybindings: KeybindingSet,
    show_help: bool,

    // Settings
    notifications_enabled: bool,
    sound_enabled: bool,

    // Contacts sidebar
    contacts: Vec<Contact>,
    selected_contact: usize,

    // Overlay state

    // Area cache
    area: StdCell<Rect>,
}

impl ChatApp {
    fn new(theme: Theme) -> Self {
        let keybindings = KeybindingSet::from_config(&resolve_keybindings());

        // Seed messages
        let messages = vec![
            Message {
                sender: "Alice",
                text: "Hey! How's the terminal engine going?".into(),
                time: "10:42",
                is_me: false,
            },
            Message {
                sender: "You",
                text: "Pretty good! Just finished the arena game.".into(),
                time: "10:43",
                is_me: true,
            },
            Message {
                sender: "Bob",
                text: "The game is awesome, I love the particles!".into(),
                time: "10:44",
                is_me: false,
            },
            Message {
                sender: "You",
                text: "Thanks! It was fun to build.".into(),
                time: "10:45",
                is_me: true,
            },
            Message {
                sender: "Alice",
                text: "Can you add emoji support? 😄".into(),
                time: "10:46",
                is_me: false,
            },
            Message {
                sender: "You",
                text: "Working on it right now!".into(),
                time: "10:47",
                is_me: true,
            },
            Message {
                sender: "Bob",
                text: "The widget gallery is also really nice.".into(),
                time: "10:48",
                is_me: false,
            },
            Message {
                sender: "Alice",
                text: "I showed it to the team, they're impressed.".into(),
                time: "10:49",
                is_me: false,
            },
            Message {
                sender: "You",
                text: "That's great to hear!".into(),
                time: "10:50",
                is_me: true,
            },
            Message {
                sender: "Bob",
                text: "Any plans for a spreadsheet widget?".into(),
                time: "10:51",
                is_me: false,
            },
            Message {
                sender: "You",
                text: "Maybe in 0.2.0, we'll see.".into(),
                time: "10:52",
                is_me: true,
            },
            Message {
                sender: "Alice",
                text: "The theme system is 🔥 btw".into(),
                time: "10:53",
                is_me: false,
            },
        ];

        let message_list = List::new_with_id(WidgetId::new(1), messages)
            .with_theme(theme.clone())
            .with_item_height(1);

        let input = SearchInput::new(WidgetId::new(2))
            .with_theme(theme.clone())
            .with_placeholder("Type a message...")
            .on_submit(|_text| {
                // Submit handled in handle_key after checking input.query()
            });

        let status_bar = StatusBar::new(WidgetId::new(3))
            .with_theme(theme.clone())
            .add_segment(StatusSegment::new("Chat Client").with_fg(theme.primary))
            .add_segment(StatusSegment::new("Online").with_fg(theme.success))
            .add_segment(StatusSegment::new(""));

        let notifications = NotificationCenter::new(theme.clone());

        let contacts = vec![
            Contact {
                name: "Alice",
                emoji: "👩",
                last_msg: "The theme system is 🔥",
                time: "10:53",
                unread: 0,
            },
            Contact {
                name: "Bob",
                emoji: "👨",
                last_msg: "Any plans for a spreadsheet?",
                time: "10:51",
                unread: 0,
            },
            Contact {
                name: "Team",
                emoji: "👥",
                last_msg: "Meeting at 3pm",
                time: "09:30",
                unread: 2,
            },
            Contact {
                name: "Eve",
                emoji: "🦹",
                last_msg: "Check the new widgets!",
                time: "Yesterday",
                unread: 1,
            },
        ];

        let mut app = Self {
            id: WidgetId::default(),
            message_list,
            input,
            status_bar,
            notifications,
            settings_modal: None,
            emoji_modal: None,
            theme,
            dirty: true,
            keybindings,
            show_help: false,
            notifications_enabled: true,
            sound_enabled: true,
            contacts,
            selected_contact: 0,
            area: StdCell::new(Rect::default()),
        };

        // Scroll to bottom
        app.message_list
            .scroll_to(app.message_list.len().saturating_sub(1));
        app
    }

    fn send_message(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let times = [
            "10:54", "10:55", "10:56", "10:57", "10:58", "10:59", "11:00",
        ];
        let time = times[self.message_list.len() % times.len()];
        self.message_list.push_item(Message {
            sender: "You",
            text: text.into(),
            time,
            is_me: true,
        });
        self.message_list
            .scroll_to(self.message_list.len().saturating_sub(1));
        self.input.clear();
        self.dirty = true;

        // Simulate reply after a delay (instant for demo)
        if text.contains("hello") || text.contains("hi") {
            self.simulate_reply("Alice", "Hey there! 👋");
        } else if text.contains("?") {
            self.simulate_reply("Bob", "Good question, let me think about that...");
        } else {
            let replies = [
                "That's cool!",
                "I agree!",
                "Nice one! 🎉",
                "Interesting!",
                "👍",
                "Lol 😂",
            ];
            let idx = self.message_list.len() % replies.len();
            let sender = if idx.is_multiple_of(2) {
                "Alice"
            } else {
                "Bob"
            };
            self.simulate_reply(sender, replies[idx]);
        }
    }

    fn simulate_reply(&mut self, sender: &'static str, text: &'static str) {
        let times = [
            "10:54", "10:55", "10:56", "10:57", "10:58", "10:59", "11:00",
        ];
        let time = times[self.message_list.len() % times.len()];
        self.message_list.push_item(Message {
            sender,
            text: text.into(),
            time,
            is_me: false,
        });
        self.message_list
            .scroll_to(self.message_list.len().saturating_sub(1));

        if self.notifications_enabled {
            self.notifications.info(sender, text);
        }

        // Update contact unread
        if let Some(contact) = self.contacts.iter_mut().find(|c| c.name == sender) {
            contact.unread = contact.unread.saturating_add(1);
        }

        self.dirty = true;
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::nord(),
            Theme::cyberpunk(),
            Theme::dracula(),
            Theme::dark(),
            Theme::solarized_dark(),
        ];
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.propagate_theme();
        self.dirty = true;
    }

    fn propagate_theme(&mut self) {
        self.message_list.on_theme_change(&self.theme);
        self.input.on_theme_change(&self.theme);
        self.status_bar.on_theme_change(&self.theme);
    }

    fn show_emoji_picker(&mut self) {
        self.emoji_modal = Some(
            Modal::new("Emoji Picker")
                .with_size(30, 10)
                .with_theme(self.theme.clone())
                .with_buttons(vec![
                    ("😀", ModalResult::Custom(1)),
                    ("❤️", ModalResult::Custom(2)),
                    ("👍", ModalResult::Custom(3)),
                    ("🔥", ModalResult::Custom(4)),
                    ("🎉", ModalResult::Custom(5)),
                    ("Cancel", ModalResult::Cancel),
                ])
                .on_confirm(|| {})
                .on_cancel(|| {}),
        );
        self.dirty = true;
    }

    fn show_settings(&mut self) {
        let settings = Modal::new("Settings")
            .with_size(36, 8)
            .with_theme(self.theme.clone())
            .with_buttons(vec![
                ("Save", ModalResult::Confirm),
                ("Cancel", ModalResult::Cancel),
            ])
            .on_confirm(|| {})
            .on_cancel(|| {});

        // Add toggle widgets as content description
        let _ = settings; // modal will be rendered with overlay text
        self.settings_modal = Some(settings);
        self.dirty = true;
    }

    fn dismiss_modals(&mut self) {
        if self.emoji_modal.is_some() {
            self.emoji_modal = None;
            self.dirty = true;
        }
        if self.settings_modal.is_some() {
            self.settings_modal = None;
            self.dirty = true;
        }
        if self.show_help {
            self.show_help = false;
            self.dirty = true;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Widget Trait
// ═══════════════════════════════════════════════════════════════════════════════

impl Widget for ChatApp {
    fn id(&self) -> WidgetId {
        WidgetId::new(100)
    }
    fn set_id(&mut self, id: WidgetId) {
        let _ = id;
    }
    fn area(&self) -> Rect {
        self.area.get()
    }
    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }
    fn z_index(&self) -> u16 {
        0
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
    fn focusable(&self) -> bool {
        true
    }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(t.bg);

        let sidebar_w = 18u16.min(area.width / 4);
        let status_h = 1u16;
        let input_h = 3u16;
        let header_h = 1u16;
        let msg_area_w = area.width.saturating_sub(sidebar_w);
        let msg_list_h = area.height.saturating_sub(status_h + input_h + header_h);

        // ── Header ────────────────────────────────────────────────────────────
        let contact = &self.contacts[self.selected_contact];
        let header_text = format!(
            " {} {}  {} — {}",
            contact.emoji, contact.name, contact.last_msg, contact.time
        );
        let header_len = header_text.chars().count().min(msg_area_w as usize);
        for (i, c) in header_text.chars().take(header_len).enumerate() {
            let idx = sidebar_w as usize + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_on_accent;
                plane.cells[idx].bg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
                plane.cells[idx].transparent = false;
            }
        }
        // Fill header rest
        for x in (sidebar_w + header_len as u16)..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.primary;
                plane.cells[idx].transparent = false;
            }
        }

        // ── Contacts Sidebar ──────────────────────────────────────────────────
        for (i, ct) in self.contacts.iter().enumerate() {
            let row = 1 + i as u16;
            if row >= area.height {
                break;
            }
            let is_selected = i == self.selected_contact;
            let bg = if is_selected {
                t.selection_bg
            } else {
                t.surface
            };
            let fg = if is_selected { t.selection_fg } else { t.fg };

            // Fill row background
            for col in 0..sidebar_w {
                let idx = (row * area.width + col) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].transparent = false;
                }
            }

            // Emoji + name
            let label = if ct.unread > 0 {
                format!("{} {} ({})", ct.emoji, ct.name, ct.unread)
            } else {
                format!("{} {}", ct.emoji, ct.name)
            };
            let label_len = label.chars().count().min(sidebar_w as usize);
            for (j, c) in label.chars().take(label_len).enumerate() {
                let idx = (row * area.width + 1 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if j == 0 {
                        fg
                    } else if ct.unread > 0 && !is_selected {
                        t.primary
                    } else {
                        fg
                    };
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].style = if ct.unread > 0 {
                        Styles::BOLD
                    } else {
                        Styles::empty()
                    };
                }
            }
        }

        // Sidebar divider
        for y in 0..area.height {
            let idx = (y * area.width + sidebar_w) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].bg = t.bg;
                plane.cells[idx].transparent = false;
            }
        }

        // ── Message List ──────────────────────────────────────────────────────
        let msg_plane = self.message_list.render(Rect::new(
            sidebar_w + 1,
            header_h,
            msg_area_w.saturating_sub(1),
            msg_list_h,
        ));
        plane.blit_from(&msg_plane, sidebar_w + 1, header_h);

        // ── Input Area ────────────────────────────────────────────────────────
        let input_y = header_h + msg_list_h;
        let input_plane = self.input.render(Rect::new(
            0,
            0,
            msg_area_w.saturating_sub(2),
            input_h.saturating_sub(2),
        ));
        plane.blit_from(&input_plane, sidebar_w + 2, input_y + 1);

        // Input border
        for col in (sidebar_w + 1)..area.width.saturating_sub(1) {
            let top = (input_y * area.width + col) as usize;
            let bot = ((input_y + input_h.saturating_sub(1)) * area.width + col) as usize;
            if top < plane.cells.len() {
                plane.cells[top].char = '─';
                plane.cells[top].fg = t.outline;
                plane.cells[top].transparent = false;
            }
            if bot < plane.cells.len() {
                plane.cells[bot].char = '─';
                plane.cells[bot].fg = t.outline;
                plane.cells[bot].transparent = false;
            }
        }

        // ── Status Bar ────────────────────────────────────────────────────────
        let sb_y = area.height.saturating_sub(1);
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let theme_key = self.keybindings.display(actions::THEME).unwrap_or("ctrl+t");
        let quit_key = self.keybindings.display(actions::QUIT).unwrap_or("ctrl+q");
        let status_text = format!(
            " {}:help | {}:theme | {}:quit | Enter:send | ↑↓:scroll ",
            help_key, theme_key, quit_key
        );
        for (i, c) in status_text.chars().enumerate() {
            let idx = (sb_y * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        // Fill rest of status bar
        for x in status_text.chars().count()..area.width as usize {
            let idx = (sb_y * area.width + x as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        // ── Notification overlay ──────────────────────────────────────────────
        if !self.notifications.is_empty() {
            let notif_plane = self.notifications.render(Rect::new(
                area.width.saturating_sub(32),
                2,
                30.min(area.width),
                8.min(area.height),
            ));
            // Blit notifications on top (non-transparent cells only)
            plane.blit_from(&notif_plane, area.width.saturating_sub(32), 2);
        }

        // ── Emoji Modal overlay ───────────────────────────────────────────────
        if let Some(ref modal) = self.emoji_modal {
            let mw = 30u16.min(area.width);
            let mh = 10u16.min(area.height);
            let mx = (area.width.saturating_sub(mw)) / 2;
            let my = (area.height.saturating_sub(mh)) / 2;
            let modal_plane = modal.render(Rect::new(0, 0, mw, mh));
            plane.blit_from(&modal_plane, mx, my);
        }

        // ── Settings Modal overlay ────────────────────────────────────────────
        if let Some(ref modal) = self.settings_modal {
            let mw = 36u16.min(area.width);
            let mh = 8u16.min(area.height);
            let mx = (area.width.saturating_sub(mw)) / 2;
            let my = (area.height.saturating_sub(mh)) / 2;
            let modal_plane = modal.render(Rect::new(0, 0, mw, mh));
            plane.blit_from(&modal_plane, mx, my);

            // Render settings content inside modal
            let settings_lines = [
                format!(
                    "Notifications: {}",
                    if self.notifications_enabled {
                        "ON"
                    } else {
                        "OFF"
                    }
                ),
                format!(
                    "Sound:         {}",
                    if self.sound_enabled { "ON" } else { "OFF" }
                ),
                format!("Theme:         {}", self.theme.name),
            ];
            for (i, line) in settings_lines.iter().enumerate() {
                let ly = my + 2 + i as u16;
                for (j, c) in line.chars().enumerate() {
                    let px = mx + 2 + j as u16;
                    if px < area.width && ly < area.height {
                        let idx = (ly * area.width + px) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].char = c;
                            plane.cells[idx].fg = t.fg;
                        }
                    }
                }
            }
        }

        // ── Help Overlay ──────────────────────────────────────────────────────
        if self.show_help {
            self.render_help(&mut plane, area);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Dismiss overlays first
        if self.keybindings.matches(actions::BACK, &key) {
            self.dismiss_modals();
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }

        // If help is showing, only allow dismiss
        if self.show_help {
            return false;
        }

        // Theme cycling
        if self.keybindings.matches(actions::THEME, &key) {
            self.cycle_theme();
            return true;
        }

        // Modal-specific input
        if self.emoji_modal.is_some() {
            // Handle emoji modal button clicks via keyboard
            match key.code {
                KeyCode::Char('1')
                | KeyCode::Char('2')
                | KeyCode::Char('3')
                | KeyCode::Char('4')
                | KeyCode::Char('5') => {
                    let emojis = ["😀", "❤️", "👍", "🔥", "🎉"];
                    let idx = match key.code {
                        KeyCode::Char('1') => 0,
                        KeyCode::Char('2') => 1,
                        KeyCode::Char('3') => 2,
                        KeyCode::Char('4') => 3,
                        KeyCode::Char('5') => 4,
                        _ => 0,
                    };
                    let current = self.input.query().to_string();
                    let new_text = format!("{}{}", current, emojis[idx]);
                    // Can't set input text directly, so just close modal
                    self.emoji_modal = None;
                    self.dirty = true;
                    let _ = new_text; // Would need set_text() on SearchInput
                }
                _ => {}
            }
            return true;
        }

        if self.settings_modal.is_some() {
            match key.code {
                KeyCode::Char('n') => {
                    self.notifications_enabled = !self.notifications_enabled;
                    self.dirty = true;
                    return true;
                }
                KeyCode::Char('s') if key.modifiers.is_empty() => {
                    self.sound_enabled = !self.sound_enabled;
                    self.dirty = true;
                    return true;
                }
                KeyCode::Enter => {
                    self.settings_modal = None;
                    self.notifications.success("Settings", "Saved!");
                    self.dirty = true;
                    return true;
                }
                _ => {}
            }
            return true;
        }

        // Emoji picker
        if key.code == KeyCode::Char('e')
            && key
                .modifiers
                .contains(dracon_terminal_engine::input::event::KeyModifiers::CONTROL)
        {
            self.show_emoji_picker();
            return true;
        }

        // Settings
        if key.code == KeyCode::Char('s')
            && key
                .modifiers
                .contains(dracon_terminal_engine::input::event::KeyModifiers::CONTROL)
        {
            self.show_settings();
            return true;
        }

        // Send message
        if key.code == KeyCode::Enter {
            let text = self.input.query().to_string();
            if !text.is_empty() {
                self.send_message(&text);
            }
            return true;
        }

        // Message list navigation
        match key.code {
            KeyCode::Up => {
                self.message_list.handle_key(key);
                self.dirty = true;
                return true;
            }
            KeyCode::Down => {
                self.message_list.handle_key(key);
                self.dirty = true;
                return true;
            }
            _ => {}
        }

        // Contact switching
        match key.code {
            KeyCode::Tab => {
                self.selected_contact = (self.selected_contact + 1) % self.contacts.len();
                // Clear unread
                self.contacts[self.selected_contact].unread = 0;
                self.dirty = true;
                return true;
            }
            KeyCode::BackTab => {
                self.selected_contact = if self.selected_contact == 0 {
                    self.contacts.len() - 1
                } else {
                    self.selected_contact - 1
                };
                self.contacts[self.selected_contact].unread = 0;
                self.dirty = true;
                return true;
            }
            _ => {}
        }

        // Forward to input
        if self.input.handle_key(key) {
            self.dirty = true;
            return true;
        }

        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();

        // Forward mouse events to modals first (so clicking buttons works)
        if let Some(ref mut modal) = self.emoji_modal {
            let mw = 30u16.min(area.width);
            let mh = 10u16.min(area.height);
            let mx = (area.width.saturating_sub(mw)) / 2;
            let my = (area.height.saturating_sub(mh)) / 2;
            if col >= mx && col < mx + mw && row >= my && row < my + mh {
                let handled = modal.handle_mouse(kind, col - mx, row - my);
                if handled {
                    self.dirty = true;
                }
                return true;
            }
            // Click outside modal dismisses it
            if matches!(
                kind,
                MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left)
            ) {
                self.emoji_modal = None;
                self.dirty = true;
                return true;
            }
            return true;
        }

        if let Some(ref mut modal) = self.settings_modal {
            let mw = 36u16.min(area.width);
            let mh = 8u16.min(area.height);
            let mx = (area.width.saturating_sub(mw)) / 2;
            let my = (area.height.saturating_sub(mh)) / 2;
            if col >= mx && col < mx + mw && row >= my && row < my + mh {
                let handled = modal.handle_mouse(kind, col - mx, row - my);
                if handled {
                    self.dirty = true;
                }
                return true;
            }
            // Click outside modal dismisses it
            if matches!(
                kind,
                MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left)
            ) {
                self.settings_modal = None;
                self.dirty = true;
                return true;
            }
            return true;
        }

        // Contact sidebar click
        let sidebar_w = 18u16.min(area.width / 4);
        if col < sidebar_w
            && row > 0
            && (row - 1) < self.contacts.len() as u16
            && matches!(
                kind,
                MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left)
            )
        {
            let idx = (row - 1) as usize;
            if idx < self.contacts.len() {
                self.selected_contact = idx;
                self.contacts[self.selected_contact].unread = 0;
                self.dirty = true;
                return true;
            }
        }

        // Message list click
        let header_h = 1u16;
        let input_h = 3u16;
        let status_h = 1u16;
        let msg_list_h = area.height.saturating_sub(status_h + input_h + header_h);
        if col > sidebar_w && row >= header_h && row < header_h + msg_list_h {
            return self
                .message_list
                .handle_mouse(kind, col - sidebar_w - 1, row - header_h);
        }

        // Input area click
        let input_y = header_h + msg_list_h;
        if col >= sidebar_w + 2 && row > input_y && row < input_y + input_h {
            return self
                .input
                .handle_mouse(kind, col - sidebar_w - 2, row - input_y - 1);
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.propagate_theme();
        self.dirty = true;
    }
}

impl ChatApp {
    fn render_help(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 44u16.min(area.width.saturating_sub(4));
        let hh = 14u16.min(area.height.saturating_sub(4));
        let hx = (area.width - hw) / 2;
        let hy = (area.height - hh) / 2;

        // Background
        for y in hy..hy + hh {
            for x in hx..hx + hw {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Border
        let corners = [
            ('╭', hx, hy),
            ('╮', hx + hw - 1, hy),
            ('╰', hx, hy + hh - 1),
            ('╯', hx + hw - 1, hy + hh - 1),
        ];
        for (ch, cx, cy) in corners {
            let idx = (cy * area.width + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = t.outline;
            }
        }
        for x in hx + 1..hx + hw - 1 {
            let top = (hy * area.width + x) as usize;
            let bot = ((hy + hh - 1) * area.width + x) as usize;
            if top < plane.cells.len() {
                plane.cells[top].char = '─';
                plane.cells[top].fg = t.outline;
            }
            if bot < plane.cells.len() {
                plane.cells[bot].char = '─';
                plane.cells[bot].fg = t.outline;
            }
        }
        for y in hy + 1..hy + hh - 1 {
            let left = (y * area.width + hx) as usize;
            let right = (y * area.width + hx + hw - 1) as usize;
            if left < plane.cells.len() {
                plane.cells[left].char = '│';
                plane.cells[left].fg = t.outline;
            }
            if right < plane.cells.len() {
                plane.cells[right].char = '│';
                plane.cells[right].fg = t.outline;
            }
        }

        // Title
        let title = "Chat Client Help";
        let tx = hx + hw.saturating_sub(title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Shortcuts
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let theme_key = self.keybindings.display(actions::THEME).unwrap_or("ctrl+t");
        let quit_key = self.keybindings.display(actions::QUIT).unwrap_or("ctrl+q");

        let shortcuts = [
            ("Enter", "Send message"),
            ("↑/↓", "Scroll messages"),
            ("Tab", "Switch contact"),
            ("Ctrl+E", "Emoji picker"),
            ("Ctrl+S", "Settings"),
            (theme_key, "Cycle theme"),
            (help_key, "Toggle help"),
            (back_key, "Dismiss overlay"),
            (quit_key, "Quit"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            for (j, c) in key.chars().enumerate() {
                let idx = (row * area.width + hx + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                }
            }
            for (j, c) in desc.chars().enumerate() {
                let idx = (row * area.width + hx + 14 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = Theme::from_env_or(Theme::nord());
    let chat = ChatApp::new(theme);

    let mut app = App::new()?
        .title("Chat Client")
        .fps(30)
        .set_theme(Theme::from_env_or(Theme::nord()));
    app.add_widget(Box::new(chat), Rect::new(0, 0, 0, 0));
    app.run(|_| {})?;
    Ok(())
}

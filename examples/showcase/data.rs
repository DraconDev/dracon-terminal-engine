// ═══════════════════════════════════════════════════════════════════════════════
// DATA — Showcase example entries, ordered by impressiveness
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct ExampleMeta {
    pub(crate) name: &'static str,
    pub(crate) category: &'static str,
    pub(crate) description: &'static str,
    pub(crate) binary_name: &'static str,
    pub(crate) preview: &'static [&'static str],
}

impl ExampleMeta {
    pub fn all() -> Vec<Self> {
        vec![
            // ── Showcase Stars (most impressive, first) ────────────────────
            ExampleMeta {
                name: "raycaster",
                category: "apps",
                description: "Wolfenstein-style 3D raycaster engine",
                binary_name: "raycaster",
                preview: &[
                    "████████▓▓▒▒░░    ░░▒▒▓▓████",
                    "████▓▓▒▒░░          ░░▒▒▓▓██",
                    "██▓▒░                  ░▒▓██",
                    "░                 @        ░",
                ],
            },
            ExampleMeta {
                name: "paint",
                category: "apps",
                description: "Mouse-driven pixel art canvas with brushes",
                binary_name: "paint",
                preview: &[
                    "  B Brush  E Erase   F Fill",
                    "  1 Red  2 Orange  3 Yellow",
                    "  ┌─────────────────────────┐",
                    "  │████  ████  ░░░░  ░░░░  │",
                ],
            },
            ExampleMeta {
                name: "workshop",
                category: "apps",
                description: "Interactive widget playground (Storybook)",
                binary_name: "workshop",
                preview: &[
                    "  ► Button    | Label: Click Me  ",
                    "    Checkbox  | Pressed: false    ",
                    "    Toggle    | ┌──Preview──┐    ",
                    "    Slider    │ [Click Me] │    ",
                ],
            },
            ExampleMeta {
                name: "command_palette",
                category: "apps",
                description: "IDE Lite: CommandPalette + MenuBar",
                binary_name: "command_palette",
                preview: &[
                    "  File  Edit  View            ",
                    "  ┌────Command Palette────┐  ",
                    "  │ > New File       file │  ",
                    "  │   Save File      file │  ",
                ],
            },
            ExampleMeta {
                name: "live_feed",
                category: "apps",
                description: "Live feed: SplitPane + TabBar + StreamingText + Sparkline",
                binary_name: "live_feed",
                preview: &[
                    "  [Logs] [CPU] [Memory]      ",
                    "  ╷INFO╷ Request processed  ╷CPU╷▁▂▃▅▆▇▆▅▃▂▁",
                    "  │WARN│ Slow query: 450ms  │   45.2% avg      ",
                    "  │INFO│ Cache hit for /api ╵                   ",
                ],
            },
            ExampleMeta {
                name: "action_center",
                category: "apps",
                description: "ContextMenu + ConfirmDialog + Toast",
                binary_name: "action_center",
                preview: &[
                    "  src        dir   —       Interaction Patterns",
                    "  Cargo.toml  file  1.2K    ContextMenu  ConfirmDialog",
                    "  README.md  file  4.5K    Toast notifications",
                    "  Right-click for menu → Delete → Confirm",
                ],
            },
            ExampleMeta {
                name: "metrics_hub",
                category: "tools",
                description: "Metrics Hub: Slider + Gauge + ProgressRing + Spinner + StatusBadge",
                binary_name: "metrics_hub",
                preview: &[
                    "  ▸ CPU  ▓▓▓▓▓▓▓▓░░░  75%  CPU ████████░░ 75%",
                    "    MEM  ▓▓▓▓▓░░░░░  48%  MEM ████░░░░░░ 48%",
                    "    DSK  ▓▓▓▓▓▓▓▓▓░  92%  DSK █████████░ 92%",
                ],
            },
            ExampleMeta {
                name: "dev_console",
                category: "tools",
                description: "Dev Console: LogViewer + EventLogger + Label + Divider + Inspector",
                binary_name: "dev_console",
                preview: &[
                    "  [ALL] [DEBUG] [INFO] [WARN] [ERROR]",
                    "  ── Logs ──────────────────────────",
                    "  [INFO] Request processed: GET /api",
                    "  [WARN] Slow query detected: 450ms",
                ],
            },
            ExampleMeta {
                name: "navigator",
                category: "apps",
                description: "Navigator: Breadcrumbs + MenuBar + Divider + Label",
                binary_name: "navigator",
                preview: &[
                    "  File  Edit  View  Help",
                    "  home > user > Documents",
                    "  📁 work                        —",
                    "  📄 resume.pdf                 245K",
                ],
            },
            ExampleMeta {
                name: "hud_demo",
                category: "cookbook",
                description: "HUD Demo: HUD overlay + Gauge + Spinner (game HUD)",
                binary_name: "hud_demo",
                preview: &[
                    "  SCORE: 000500  LVL:2  WAVE:3",
                    "  HP ████████░░░░  SH ██████░░░░░",
                    "  AMMO: 27/30",
                ],
            },
            ExampleMeta {
                name: "animation",
                category: "data",
                description: "Animation & easing curves",
                binary_name: "animation",
                preview: &[
                    "  ●        ●",
                    "    ●    ●  ",
                    "  Linear EaseIn",
                    "  EaseOut EaseInOut",
                ],
            },
            // ── Apps ─────────────────────────────────────────────────────────
            ExampleMeta {
                name: "system_monitor",
                category: "apps",
                description: "Live system gauges with auto-refresh",
                binary_name: "system_monitor",
                preview: &[
                    "CPU [████████░░] 80%",
                    "MEM [██████░░░░] 60%",
                    "DISK [████░░░░░░] 40%",
                    "NET  [██████████] 100%",
                ],
            },
            ExampleMeta {
                name: "file_manager",
                category: "apps",
                description: "File browser with Tree + Table",
                binary_name: "file_manager",
                preview: &["v home/", "  v user/", "    v src/", "      > main.rs"],
            },
            ExampleMeta {
                name: "chat_client",
                category: "apps",
                description: "Rich chat UI with contacts & panels",
                binary_name: "chat_client",
                preview: &[
                    "[10:42] Alice: Hey!",
                    "[10:43] Bob: Hi",
                    "[10:44] Alice: Hi!",
                    "> _",
                ],
            },
            ExampleMeta {
                name: "dashboard_builder",
                category: "apps",
                description: "Build dashboards with drag & drop",
                binary_name: "dashboard_builder",
                preview: &[
                    "+---Gauge---+--List--+",
                    "| CPU ███░░ |Item 1  |",
                    "| MEM ██░░░ |Item 2  |",
                    "+-----------+--------+",
                ],
            },
            // ── Input ────────────────────────────────────────────────────────
            ExampleMeta {
                name: "form_demo",
                category: "input",
                description: "Form layout with validation",
                binary_name: "form_demo",
                preview: &[
                    " Name: [___________]",
                    " Email: [__________]",
                    " [Submit] [Cancel]",
                ],
            },
            ExampleMeta {
                name: "autocomplete",
                category: "input",
                description: "Search input with suggestions",
                binary_name: "autocomplete",
                preview: &[
                    "[rust       ]",
                    "  rustacean    ",
                    "> rust        ",
                    "  rust-analyzer",
                ],
            },
            ExampleMeta {
                name: "tags_input",
                category: "input",
                description: "Tag composition with autocomplete",
                binary_name: "tags_input",
                preview: &[
                    "[rust] [terminal] [___]",
                    "> rust-analyzer   ",
                    "  Tags: 2/8       ",
                ],
            },
            ExampleMeta {
                name: "tooltip",
                category: "input",
                description: "Hover tooltips on buttons",
                binary_name: "tooltip",
                preview: &[
                    "┌──────────┐",
                    "│ 📁 Files  │",
                    "└──────────┘",
                    "  ┌──────────────────────────┐",
                ],
            },
            ExampleMeta {
                name: "password_input",
                category: "input",
                description: "Login form with masked password input",
                binary_name: "password_input",
                preview: &[
                    "  Username: [admin____]",
                    "  Password: [••••••____]",
                    "  Strength: Strong",
                ],
            },
            ExampleMeta {
                name: "color_picker",
                category: "input",
                description: "Interactive color picker with preview",
                binary_name: "color_picker",
                preview: &[
                    "  ████████████",
                    "  Hex: #58a6ff",
                    "  RGB: 88, 166, 255",
                    "  Palette: ██ ██ ██ ██",
                ],
            },
            // ── Data ─────────────────────────────────────────────────────────
            ExampleMeta {
                name: "rich_text",
                category: "data",
                description: "Markdown rendering with tabbed docs",
                binary_name: "rich_text",
                preview: &[
                    "# RichText Widget",
                    "**Bold** and *italic*",
                    "`inline code`",
                    "- List item",
                ],
            },
            ExampleMeta {
                name: "notification_center",
                category: "data",
                description: "Toast notification queue with filters",
                binary_name: "notification_center",
                preview: &["  ╭───────╮", "  │ i Info│", "  │ Done! │", "  ╰───────╯"],
            },
            ExampleMeta {
                name: "cell_pool",
                category: "data",
                description: "Cell allocation recycling with gauges",
                binary_name: "cell_pool",
                preview: &[
                    "Cell Pool Stats:",
                    "  Active: 48",
                    "  Reuse rate: 97.5%",
                    "  Memory saved: 15KB",
                ],
            },
            ExampleMeta {
                name: "kanban",
                category: "data",
                description: "Drag-drop kanban board",
                binary_name: "kanban",
                preview: &[
                    "To Do  |In Prog|Done  ",
                    "┌────┐ ┌────┐ ┌────┐",
                    "│card│ │card│ │card│",
                    "└────┘ └────┘ └────┘",
                ],
            },
            ExampleMeta {
                name: "debug_overlay",
                category: "data",
                description: "Performance metrics, FPS, frame time, profiler",
                binary_name: "debug_overlay",
                preview: &[
                    "  60 FPS  16.7ms",
                    "  CPU [████░░░░░] 12%",
                    "  render  13.3ms  1x",
                ],
            },
            // ── Cookbook ──────────────────────────────────────────────────────
            ExampleMeta {
                name: "widget_gallery",
                category: "cookbook",
                description: "All interactive widgets demo",
                binary_name: "widget_gallery",
                preview: &[
                    "[x] Checkbox  [B1][B2]",
                    "(o) Radio    [====]",
                    "Table: | A | B | C |",
                    "Tree: v root > child",
                ],
            },
            ExampleMeta {
                name: "theme_switcher",
                category: "cookbook",
                description: "Live theme cycling (21 themes)",
                binary_name: "theme_switcher",
                preview: &["Theme: Nord", "+----------+", "| # # # #   |"],
            },
            // ── Tools ─────────────────────────────────────────────────────────
            ExampleMeta {
                name: "accessibility",
                category: "accessibility",
                description: "Screen reader support (OSC 99)",
                binary_name: "accessibility",
                preview: &[
                    "OSC 99 Announcements:",
                    "  Role: button",
                    "  Label: Submit",
                    "  Screen reader: NVDA",
                ],
            },
        ]
        .clone()
    }
}

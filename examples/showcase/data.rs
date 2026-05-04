// ═══════════════════════════════════════════════════════════════════════════════
// DATA
// ═══════════════════════════════════════════════════════════════════════════════

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
            // Apps
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
                name: "ide",
                category: "apps",
                description: "Full IDE with menus, tabs, tree, editor",
                binary_name: "ide",
                preview: &[
                    "[File][Edit][View]",
                    "+-src/ +--------+",
                    "| main |fn main|",
                    "| lib  |{      |",
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
                description: "Rich chat UI with panels",
                binary_name: "chat_client",
                preview: &[
                    "[10:42] Alice: Hey!",
                    "[10:43] Bob: Hi",
                    "[10:44] Alice: Hi!",
                    "> _",
                ],
            },
            ExampleMeta {
                name: "git_tui",
                category: "apps",
                description: "Real Git status/log/diff/branches",
                binary_name: "git_tui",
                preview: &[
                    "[Status][Log][Diff]",
                    " M src/main.rs",
                    " A Cargo.toml",
                    "?? README.md",
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
            // Cookbook
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
                name: "command_bindings",
                category: "cookbook",
                description: "Live CLI-bound widgets",
                binary_name: "command_bindings",
                preview: &[
                    "Load: 0.45 0.32",
                    "CPU:  [####--]",
                    "Mem:  [######]",
                    "Net:  [------]",
                ],
            },
            ExampleMeta {
                name: "split_resizer",
                category: "cookbook",
                description: "Drag-to-resize SplitPane",
                binary_name: "split_resizer",
                preview: &[
                    "+-----+-----+",
                    "|  A  |  B  |",
                    "+--+--+-----+",
                    "|  C  |  D  |",
                ],
            },
            ExampleMeta {
                name: "menu_system",
                category: "cookbook",
                description: "MenuBar + ContextMenu",
                binary_name: "menu_system",
                preview: &[
                    "[File][Edit][View]",
                    "+-----------+",
                    "| New        |",
                    "| Open       |",
                ],
            },
            ExampleMeta {
                name: "tabbed_panels",
                category: "cookbook",
                description: "Tab bar with panel switching",
                binary_name: "tabbed_panels",
                preview: &[
                    "[Tab1][Tab2][Tab3]+",
                    "+---------------+",
                    "|   Panel       |",
                    "|   Content     |",
                    "+---------------+",
                ],
            },
            ExampleMeta {
                name: "tree_navigator",
                category: "cookbook",
                description: "Expandable tree widget",
                binary_name: "tree_navigator",
                preview: &[
                    "v root/",
                    "| v src/",
                    "| | > main.rs",
                    "| | > lib.rs",
                    "| v target/",
                ],
            },
            ExampleMeta {
                name: "data_table",
                category: "cookbook",
                description: "Sortable table with selection",
                binary_name: "data_table",
                preview: &[
                    " Name     | Age | City ",
                    "----------|-----|------",
                    " Alice    |  28 | NYC  ",
                    " Bob      |  34 | LA   ",
                    "> Carol   |  22 | SEA  ",
                ],
            },
            ExampleMeta {
                name: "log_monitor",
                category: "cookbook",
                description: "Live log viewer with filters",
                binary_name: "log_monitor",
                preview: &[
                    "[ERROR][WARN][INFO]",
                    "10:42 ERROR connection",
                    "10:41 WARN retry...",
                    "10:40 INFO ready",
                    "10:39 INFO listening",
                ],
            },
            ExampleMeta {
                name: "debug_overlay",
                category: "cookbook",
                description: "FPS/performance overlay",
                binary_name: "debug_overlay",
                preview: &[
                    "FPS: 60 | CPU: 12%",
                    "Widgets: 24 | Z: 5",
                    "Mouse: (80, 24)",
                    "Frame: 12340",
                ],
            },
            // Tools
            ExampleMeta {
                name: "theme_switcher",
                category: "tools",
                description: "Live theme cycling (15 themes)",
                binary_name: "theme_switcher",
                preview: &[
                    "Theme: Nord",
                    "+----------+",
                    "| # # # #   |",
                    "| # # # #   |",
                ],
            },
            ExampleMeta {
                name: "modal_demo",
                category: "tools",
                description: "Modal dialogs + focus trapping",
                binary_name: "modal_demo",
                preview: &[
                    "+---------------+",
                    "| Confirm?     |",
                    "| [Yes] [No]   |",
                    "+---------------+",
                ],
            },
            ExampleMeta {
                name: "desktop",
                category: "tools",
                description: "Draggable windows + taskbar",
                binary_name: "desktop",
                preview: &[
                    "+------++------+",
                    "| Win1 || Win2  |",
                    "|      ||      |",
                    "+------++------+",
                ],
            },
            ExampleMeta {
                name: "input_debug",
                category: "tools",
                description: "Key/mouse event visualizer",
                binary_name: "input_debug",
                preview: &[
                    "Key: ArrowUp  0x2191",
                    "Mod: Ctrl+Shift",
                    "Mouse: 45, 12 [L-down]",
                    "Wheel: +1",
                ],
            },
            ExampleMeta {
                name: "text_editor_demo",
                category: "tools",
                description: "Syntax-highlighted editor",
                binary_name: "text_editor_demo",
                preview: &[
                    "1 | fn main() {",
                    "2 | >  println!();",
                    "3 | }",
                    "   [rust] UTF-8",
                ],
            },
            ExampleMeta {
                name: "table_widget",
                category: "tools",
                description: "Advanced table with sorting",
                binary_name: "table_widget",
                preview: &[
                    "ID | Name    | Score",
                    "---|---------|------",
                    " 1 | Alpha   |  98",
                    " 2 | Beta    |  85",
                    " 3 | Gamma   |  92",
                ],
            },
            ExampleMeta {
                name: "form_demo",
                category: "tools",
                description: "Form layout with validation",
                binary_name: "form_demo",
                preview: &[
                    " Name: [___________]",
                    " Email: [__________]",
                    " [Submit] [Cancel]",
                ],
            },
            ExampleMeta {
                name: "sqlite_browser",
                category: "tools",
                description: "Browse SQLite databases",
                binary_name: "sqlite_browser",
                preview: &[
                    "Tables: [users]",
                    "[schema] [data] [query]",
                    "+---------------+",
                    "| id | name     |",
                    "| 1  | Alice    |",
                    "+---------------+",
                ],
            },
            ExampleMeta {
                name: "widget_tutorial",
                category: "tools",
                description: "Step-by-step widget guide",
                binary_name: "widget_tutorial",
                preview: &[
                    "Step 1: Create widget",
                    "  let btn = Button::new()",
                    "Step 2: Add to app",
                    "  app.add_widget(btn)",
                ],
            },
            ExampleMeta {
                name: "game_loop",
                category: "tools",
                description: "Snake game with tick loop",
                binary_name: "game_loop",
                preview: &[
                    "    ████    ",
                    "    █  █    ",
                    "  ███  ███  ",
                    "    ████    ",
                    "  Score: 42 ",
                ],
            },
            ExampleMeta {
                name: "form_widget",
                category: "tools",
                description: "Form field components",
                binary_name: "form_widget",
                preview: &[
                    "Field: [________]",
                    "Select: (opt1|[v])",
                    "Check: [x] enabled",
                    "[Apply] [Reset]",
                ],
            },
            ExampleMeta {
                name: "framework_file_manager",
                category: "tools",
                description: "Framework-based file browser",
                binary_name: "framework_file_manager",
                preview: &[
                    "/ home/ user/",
                    "+--------+-------+",
                    "| Name   | Size  |",
                    "| src/   |  -    |",
                    "| main.rs| 1.2KB |",
                    "+--------+-------+",
                ],
            },
        ]
    }
}

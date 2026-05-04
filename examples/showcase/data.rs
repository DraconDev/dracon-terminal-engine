// ═══════════════════════════════════════════════════════════════════════════════
// DATA
// ═══════════════════════════════════════════════════════════════════════════════

pub struct ExampleMeta {
    name: &'static str,
    category: &'static str,
    description: &'static str,
    binary_name: &'static str,
    preview: &'static [&'static str],
}

impl ExampleMeta {
    fn all() -> Vec<Self> {
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
            // Cookbook
            ExampleMeta {
                name: "widget_gallery",
                category: "cookbook",
                description: "All interactive widgets demo",
                binary_name: "widget_gallery",
                preview: &[
                    "[x] Checkbox",
                    "(o) Radio",
                    "[----] Slider",
                    "Loading [####] ",
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
        ]
    }
}

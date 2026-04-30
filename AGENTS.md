# Dracon Terminal Engine — Agent Notes

## Vision

**GUI-grade terminal applications.** Persistent, visible, mouse-friendly, composable widgets.

Not "CLI+" (hotkey-centric like vim/Helix). Not an editor competitor.
More like a GUI that happens to run in a terminal.

## Why TUI Over GUI?

### Deployment Advantages
- **Universal**: Runs on VPS, SSH, containers, CI, embedded — anywhere with a terminal
- **Zero user dependencies**: No browser, no runtime, no permissions, no install
- **Single binary**: Ships as one executable, instant startup
- **Cross-platform without bridge hell**: No Tauri/Dioxus/egui platform issues, no browser bugs, no lag

### UX Advantages Over CLI
- **Persistent state**: Don't re-run commands to see output
- **Visible structure**: Panels, trees, forms — not just scrolling text
- **Mouse-friendly**: Click, drag, scroll — natural interactions
- **Composability**: Mix widgets (list + editor + form) freely

## TextEditor Scope

TextEditor is a **view/edit widget** for composing into larger applications:
- File managers: view/edit config files
- Chat UIs: edit messages
- Forms: text input fields
- Log viewers: search, filter, navigate

**NOT a vim/Helix competitor.** Not a modal editor. Not LSP-powered.

Single cursor + selection is sufficient. Framework integration is the priority.

## Architecture Principles

| Principle | Meaning |
|-----------|---------|
| Widgets own state | TextEditor manages its own lines, cursor, selection |
| App owns composition | App manages widgets via registry, z-index, focus |
| Mouse-first | Widgets respond to clicks, not just keys |
| Keyboard-enhanced | Navigation shortcuts exist but aren't required |
| Terminal as universal target | No platform-specific code, no external dependencies |

## TextEditor Public API

```rust
// Creation
TextEditor::new()                        // Empty editor
TextEditor::with_content("...")         // From string
TextEditor::open(&path)                 // From file (loads .undo too)

// File I/O
editor.save()                           // Save to current path
editor.save_as(&path)                   // Save to new path
editor.file_path()                       // Current path if any

// View options
editor.with_show_line_numbers(bool)
editor.with_word_wrap(bool)
editor.with_indent_guides(bool)
editor.with_status_bar(bool)
editor.with_language("rust")           // For syntax highlighting

// Navigation & Search
editor.goto_line(line, area)            // Jump to line
editor.set_filter("query")               // Filter/highlight mode
editor.replace_all(find, replace)        // Global replace
editor.replace_next(find, replace)      // Next occurrence

// Selection & Clipboard
editor.get_selected_text()               // Get selection
editor.select_all()
editor.select_word_at(row, col)

// Multi-cursor (basic)
editor.add_cursor(row, col)              // Add extra cursor
editor.clear_extra_cursors()

// Persistence
editor.load_undo_stack()                 // Load from .file.undo
editor.save_undo_stack()                 // Save to .file.undo
editor.load_config()                     // Load from .file.dte.json
editor.save_config()                     // Save to .file.dte.json
```

## Widget Trait Bridge

TextEditor implements **ratatui's `Widget`** trait (renders to `Buffer`).
The App framework uses its own `Widget` trait (returns `Plane`, has `id`, `handle_key`, `handle_mouse`).

To use TextEditor in App, wrap it in an adapter that implements the framework's Widget trait.

## Deferred / Out of Scope

These are interesting but NOT priorities for an engine:

| Feature | Why Deferred |
|---------|-------------|
| LSP integration | Requires async runtime, external processes, complex state management |
| Syntax-aware folding | Requires tree-sitter integration, per-language grammar |
| Multi-cursor enhancements | Basic multi-cursor sufficient for light editing |
| Modal editing | Kakoune-style is complex, not needed for view/edit use cases |
| Advanced text objects | vim-style text objects require deep editor integration |

## Contributing

When adding features to TextEditor or other widgets:
1. Is it a widget feature or an app feature?
2. Does it require external processes/services?
3. Does it change the widget's core purpose?

**If a feature requires LSP, complex state, or makes the widget into an editor — it's probably out of scope.**

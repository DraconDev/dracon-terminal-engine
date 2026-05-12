# Dracon TUI Preview

Live TUI preview extension for Dracon Terminal Engine in VSCode.

## Features

- **Live Preview**: Start a TUI preview directly from your Rust code
- **Theme Support**: Multiple themes (Nord, Dracula, Monokai, Solarized, Gruvbox, Cyberpunk)
- **Code Lens**: Run preview directly from `main()` and `#[test]` functions
- **Editor Integration**: Start preview button in editor title bar
- **Real-time Updates**: Auto-refresh on file changes (debounced)

## Requirements

- VSCode 1.80.0+
- Rust toolchain
- Dracon showcase binary (built from `cargo build --example showcase`)

## Installation

1. Build the showcase binary:
   ```bash
   cargo build --example showcase
   ```

2. Build the LSP server:
   ```bash
   cd extensions/lsp-server
   cargo build --release
   ```

3. Copy the LSP binary to your cargo bin:
   ```bash
   cp target/release/dracon-lsp ~/.cargo/bin/
   ```

4. In VSCode:
   - Open the `extensions/vscode` directory
   - Run `npm install`
   - Run `npm run compile`
   - Press F5 to launch extension development host

## Usage

### Commands

| Command | Description |
|---------|-------------|
| `Dracon: Start TUI Preview` | Start live preview of current file |
| `Dracon: Stop TUI Preview` | Stop active preview |
| `Dracon: Refresh TUI Preview` | Refresh preview |

### Keyboard Shortcuts

Use the keybindings defined in your VSCode settings or Dracon configuration file.

### Configuration

| Setting | Description | Default |
|---------|-------------|---------|
| `dracon.showcasePath` | Path to showcase binary | `${workspaceFolder}/target/debug/showcase` |
| `dracon.theme` | Preview theme | `nord` |
| `dracon.lspServerPath` | Path to LSP server | `${workspaceFolder}/target/debug/dracon-lsp` |

## Architecture

```
extensions/
├── vscode/           # VSCode extension
│   ├── src/
│   │   └── extension.ts   # Main extension code
│   ├── package.json       # Extension manifest
│   └── tsconfig.json      # TypeScript config
└── lsp-server/       # Companion LSP server
    ├── src/
    │   └── main.rs        # LSP server implementation
    └── Cargo.toml         # Rust dependencies
```

### Extension Components

- **DraconTUIContentProvider**: Virtual document provider for `dracon-tui://` URIs
- **DraconLspClient**: Communication layer with LSP server
- **PreviewManager**: Preview lifecycle management

### LSP Server

The LSP server handles:
- Compilation of examples
- Terminal output streaming
- File watching for auto-refresh

## License

MIT OR Apache-2.0

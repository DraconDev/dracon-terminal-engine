# Contributing to Dracon Terminal Engine

Thank you for your interest in contributing!

## License

All contributions are subject to the terms of the [AGPLv3 license](./LICENSE) and the [Contributor License Agreement (CLA)](./CLA.md).

**By submitting a Contribution (including via pull request, issue, comment, or any other method), you agree to be bound by both the AGPLv3 license and the CLA.**

## Before You Submit a Pull Request

1. **Read the CLA** — Make sure you understand and agree to the [Contributor License Agreement](./CLA.md) before submitting any Contribution.
2. **Fork and branch** — Create a feature branch from `main` for your changes.
3. **Write clean, idiomatic code** — Follow the existing style and conventions of the project.
4. **Test your changes** — Ensure all existing and new tests pass before opening a PR.
5. **Describe your changes** — Include a clear PR description explaining *what* changed and *why*.
6. **Keep scope small** — One PR per logical change. Don't bundle unrelated fixes.

## Prerequisites

- Rust 1.75 or later
- A terminal that supports 24-bit color and SGR mouse events

## Setup

```bash
# Clone the repository
git clone https://github.com/DraconDev/dracon-terminal-engine.git
cd dracon-terminal-engine

# Build the project
cargo build

# Run tests
cargo test

# Run examples
cargo run --example framework_demo
```

## Project Structure

```
src/
├── main.rs              # Entry point, event loop, CLI parsing
├── framework/           # Layout, rendering, event pipeline
├── input/              # Keyboard/mouse input handling
├── widgets/             # Reusable TUI widgets
├── theme/               # Color schemes and styles
└── utils.rs             # Shared utilities (clipboard, highlight_code)
```

## Code of Conduct

All contributors are expected to behave professionally and respectfully. We do not tolerate harassment, discrimination, or hostile behavior in any form.

## Getting Help

If you have questions or need guidance, open an issue or reach out to the maintainers directly.

---

*For details on commercial licensing, see [COMMERCIAL-LICENSE.md](./COMMERCIAL-LICENSE.md).*
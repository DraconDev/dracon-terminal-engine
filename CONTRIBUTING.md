# Contributing to Dracon Terminal Engine

Thank you for your interest in contributing!

## Getting Started

### Prerequisites

- Rust 1.75 or later
- A terminal that supports 24-bit color and SGR mouse events

### Setup

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

### Repository Structure

```
dracon-terminal-engine/
├── src/
│   ├── backend/      # Low-level POSIX terminal ioctls
│   ├── compositor/   # Plane layering, color blending, filters
│   ├── core/         # Terminal wrapper (raw mode + alt screen)
│   ├── framework/    # App runtime, widgets, theme, animation
│   ├── input/        # SGR mouse + chord parsing
│   ├── integration/  # Ratatui bridge
│   ├── layout/       # Constraint-based layout engine
│   ├── system/       # SystemMonitor (CPU, memory, disk)
│   ├── visuals/      # Icons, OSC strings, sync mode
│   └── widgets/      # TextEditor, TextInput
├── tests/            # Integration tests
├── examples/         # Runnable examples
└── .github/         # CI/CD workflows
```

## Making Changes

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy --all-targets --all-features -- -D warnings` and fix all warnings
- Add doc comments to public APIs
- Keep lines under 100 characters

### Testing

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --lib                # Unit tests
cargo test --test phase1_widget_test  # Widget integration tests
cargo test --test theme_test    # Theme tests
cargo test --test scroll_test   # Scroll behavior tests

# Run with output
cargo test -- --nocapture
```

### Commit Messages

- Use the imperative mood ("Add feature" not "Added feature")
- Keep the first line under 72 characters
- Add a body explaining the "why" if needed
- Reference issues: "Fixes #123" or "Closes #123"

Example:
```
fix: Checkbox::toggle now marks widget dirty

Previously, calling toggle() on a Checkbox would not trigger a re-render,
causing the UI to show stale state. This change calls mark_dirty() after
the checked state changes, ensuring the widget re-renders on the next frame.

Fixes #456
```

### Pull Request Process

1. Fork the repo and create your branch from `master`
2. Make your changes and add tests
3. Ensure all tests pass (`cargo test --all-features`)
4. Update documentation if adding public APIs
5. Open a PR with a clear description of the changes

## Adding a New Widget

1. Create `src/framework/widgets/my_widget.rs`
2. Implement the `Widget` trait
3. Add `pub mod my_widget;` to `src/framework/widgets/mod.rs`
4. Export in `src/framework/mod.rs` and `src/framework/prelude`
5. Add integration tests in `tests/`
6. Add an example if it demonstrates a non-obvious use case

## Reporting Issues

- Search existing issues before creating a new one
- Include your OS, Rust version, and dracon-terminal-engine version
- Include a minimal reproduction case if possible
- For bugs, include the output of `cargo test` and any relevant error messages

## License

By contributing, you agree that your contributions will be dual-licensed
under the MIT License and the Apache License 2.0.

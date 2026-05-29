# Dracon Terminal Engine - Improvement Tasks

Audit date: 2026-05-29
Repo: `/home/dracon/Dev/dracon-terminal-engine`

This backlog is based on the current working tree, existing audit notes, and live verification commands:

- `cargo check --lib --all-features` passes.
- `cargo check --all-targets --all-features` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
- `cargo fmt --all -- --check` passes.
- `cargo test --all-features` passes.

## P0 - Restore Build And CI Health

- [x] Fix stale renamed-module imports in tests so `cargo check --all-targets --all-features` passes.
  - Preserved compatibility by keeping deprecated aliases for `text_input_base`, `tabbar`, and `list_common`.

- [x] Remove duplicate `#[test]` attributes that still emit warnings.

- [x] Run `cargo fmt --all` and commit the formatting-only drift.

- [x] Fix current clippy warnings after the test imports compile.

- [x] Run the full verification suite after P0 fixes.
  - `cargo fmt --all -- --check`
  - `cargo check --all-targets --all-features`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-features`

## P1 - Release And Metadata Correctness

- [x] Fix release workflow packaging.
  - `.github/workflows/release.yml` uploads `LICENSE-MIT` and `LICENSE-APACHE`, but the repo currently has `LICENSE` and package metadata says `AGPL-3.0-only`.
  - Update release files to match the actual license files, or add the missing license files if dual licensing is intentional.

- [x] Reconcile README, changelog, and crate metadata.
  - `Cargo.toml` version is `0.1.10`.
  - `README.md` still references framework "v29", `v29.11.0`, 41 widgets, and 21 themes.
  - Existing audits claim 47 framework widgets and 55+ examples.
  - Decide the canonical public counts and version language, then update `README.md`, `CHANGELOG.md`, and any generated docs consistently.

- [x] Add a release dry-run gate before publishing tags.
  - Add `cargo publish --dry-run` to release workflow once package metadata and exclude/include lists are correct.
  - Keep crates.io publish manual or token-gated until release contents are verified.

- [x] Review package exclusions.
  - `Cargo.toml` excludes `extensions/**`, `rust_out/**`, `.sisyphus/**`, and `plans/**`, but local audit/task files and generated artifacts remain package candidates.
  - Run `cargo package --list` and exclude non-release audit files if they should not ship.

## P2 - API Cleanup And Compatibility

- [x] Remove or preserve compatibility aliases for renamed modules.
  - Renames from `tabbar` to `tab_bar`, `list_common` to `list_helpers`, and `text_input_base` to `text_input_core` broke tests.
  - Decide whether these were intended breaking changes. If not, add deprecated module aliases that re-export the new modules.

- [ ] Finish the deprecated `App::theme()` migration/removal plan.
  - Current live scan found no production call sites of `.theme(Theme...)`, but `App::theme()` still exists as deprecated API.
  - Decide whether removal belongs in `0.2.0`; document it in changelog and migration notes.

- [ ] Resolve duplicate I/O error variants in `DraconError`.
  - Existing audit notes track `IoError` and `Io` as API duplication.
  - Merge during a breaking release and update conversions/tests.

- [ ] Standardize builder method ownership.
  - Audit `App`, `Theme`, layout, and widget builder methods for mixed `self`, `&mut self`, and return conventions.
  - Document the convention in `CONTRIBUTING.md` or `AI_GUIDE.md`.

- [ ] Decide the fate of deprecated standalone widgets.
  - `src/widgets/component.rs` is deprecated but still public.
  - `src/widgets/hotkey.rs` overlaps conceptually with framework widgets.
  - Remove or feature-gate them in the next breaking release.

## P3 - Testing Gaps

- [x] Add regression tests for renamed module compatibility.
  - If deprecated aliases are kept, add compile-time import tests for old and new paths.
  - If aliases are removed, update tests and document the breaking change.

- [ ] Add integration coverage for `SceneRouter` transitions.
  - Cover push/pop/replace lifecycle calls, transition completion, and back navigation depth checks.
  - Include at least one test for non-overlapping z-index composition during transitions.

- [ ] Add plugin loading/unloading integration tests.
  - Use a minimal in-repo test plugin or mock `WidgetFactory`.
  - Verify registration, widget creation, unload behavior, and failure paths.

- [ ] Add `cargo-dracon` CLI integration tests.
  - `cargo run -p cargo-dracon -- --help` exits successfully.
  - `cargo run -p cargo-dracon -- validate dracon.toml` succeeds for the repo config.

- [ ] Add event bus benchmarks.
  - Publish 1000 events to 10 subscribers.
  - `subscribe_once` with 100 callbacks and cleanup.

- [ ] Expand widget interaction tests where coverage is still shallow.
  - Prioritize mouse hover clearing, focus styling, theme propagation, and bounded text clipping.
  - Focus first on high-complexity widgets: `TextEditorAdapter`, `CommandPalette`, `Kanban`, `Table`, `TagsInput`, `Calendar`, `Modal`, `ContextMenu`.

## P4 - Documentation And Examples

- [x] Update example count and widget count docs.
  - README and testing docs still mention older counts such as 29 scenes and 41 widgets.
  - Make counts either generated or deliberately approximate to avoid constant drift.

- [x] Update quick-start examples to current APIs.
  - README still shows `.theme(Theme::cyberpunk())`; use `.set_theme(Theme::from_env_or(Theme::cyberpunk()))`.
  - Make all new example snippets follow the AGENTS.md theme inheritance rule.

- [x] Document the `Widget::render(&self)` design decision.
  - Existing audit notes say this is intentional for compositor/render scheduling.
  - Add a short note to the `Widget` trait docs explaining why render is immutable while input handlers are mutable.

- [ ] Add public item docs in remaining high-use widget modules.
  - `src/framework/widgets/log_viewer.rs`
  - `src/framework/widgets/confirm_dialog.rs`
  - `src/framework/widgets/list_helpers.rs`
  - `src/framework/widget_container.rs`
  - `src/input/mapping.rs`

- [ ] Consolidate audit files.
  - The repo currently contains `AUDIT.md`, `TODO.md`, `audit.md`, `audit-tastlist.md`, `tasklist.md`, and this `tasks.md`.
  - Keep one current backlog and move old audits to an archive or remove them before release.

## P5 - Runtime Robustness

- [ ] Review `extensions/lsp-server/src/main.rs` unwrap-heavy JSON send paths.
  - Replace production `.unwrap()` calls around serialization, locking, and channel writes with contextual errors or logged failures.
  - Keep test-only unwraps where they express assertions.

- [ ] Revisit `App::default()`.
  - `src/framework/app.rs` still uses `Self::new().expect("failed to initialize terminal")` because `Default` cannot return `Result`.
  - Add a fallible constructor with explicit default config and consider deprecating `Default` in a breaking release.

- [ ] Implement or remove sixel decoding.
  - `sixel` is feature-gated but decoding remains a stub by design.
  - Either implement decoding with tests or make the feature's limitations explicit in docs and examples.

- [ ] Add `dracon.toml` validation.
  - Create `AppConfig::validate()`.
  - Warn on unknown fields rather than failing, preserving forward compatibility.
  - Add tests for valid config, unknown fields, and malformed config.

## P6 - Maintainability Refactors

- [ ] Split the largest functions only when touching nearby behavior.
  - `src/widgets/editor.rs` `render()` and `handle_event()`
  - `src/compositor/engine.rs` `render()`
  - `src/input/parser.rs` `try_parse()` and `parse_csi_normal()`
  - `src/utils.rs` `spawn_terminal_at()`
  - Complex widget renders: `TagsInput`, `Kanban`, `CommandPalette`, `Calendar`, `Sparkline`, `ConfirmDialog`, `ColorPicker`, `LogViewer`

- [ ] Split catch-all framework modules into focused files.
  - `src/framework/command.rs`: separate app config, command execution, output parsing, and layout config.
  - `src/framework/helpers.rs`: separate text drawing, borders, blitting, and scroll helpers.
  - Consider `src/framework/callbacks.rs` for shared callback type aliases.

- [ ] Resolve layout module duplication.
  - `src/layout.rs` and `src/framework/layout.rs` are both public.
  - Keep compatibility only where needed and document the preferred path.

## Verification Notes

Last verified locally on 2026-05-29:

```text
cargo check --lib --all-features
PASS

cargo check --all-targets --all-features
PASS

cargo clippy --all-targets --all-features -- -D warnings
PASS

cargo fmt --all -- --check
PASS

cargo test --all-features
PASS

cargo publish --dry-run --allow-dirty
PASS
```

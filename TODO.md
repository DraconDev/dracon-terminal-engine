# Dracon Terminal Engine — TODO

Audit date: 2026-05-22  
Total LOC: 41,488  
Framework widgets: 41 · Themes: 21 · Examples: 57  
Public API items: 1,244 · Test functions: ~1,436  
Dependencies (transitive): 310 · Rc/RefCell uses: 403 · `unwrap()`/`expect()` calls: 129

---

## 🔴 High Priority

### 1. Fix transitive `lru` unsoundness

| Crate | Version | Issue | Source |
|-------|---------|-------|--------|
| `lru` | 0.12.5 | **RUSTSEC-2026-0002** — `IterMut` violates Stacked Borrows | via `ratatui 0.29` |

- [ ] File issue with `ratatui` to update `lru`
- [ ] If no upstream fix, evaluate pinning or vendoring

### 2. No CI pipeline

No `.github/workflows/` or CI config files found. Zero automated gates.
- [ ] Add GitHub Actions workflow: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check`
- [ ] Add nightly/hygiene checks: `cargo audit`, `cargo outdated`, doc build
- [ ] Add markdown lint for `CHANGELOG.md` formatting

### 3. 4 security advisories

All transitive via `syntect` and `ratatui`:

| Crate | Advisory | Action |
|-------|----------|--------|
| `bincode 1.3.3` | RUSTSEC-2025-0141 (unmaintained) | Evaluate replacing syntect or pinning |
| `paste 1.0.15` | RUSTSEC-2024-0436 (unmaintained) | Upstream (ratatui) — monitor |
| `yaml-rust 0.4.5` | RUSTSEC-2024-0320 (unmaintained) | Upstream (syntect) — monitor |
| `lru 0.12.5` | RUSTSEC-2026-0002 (unsound) | Upstream (ratatui) — escalate |

- [ ] Add `cargo audit` to CI (or use `cargo deny`)
- [ ] Track each upstream fix

---

## 🟡 Medium Priority

### 4. Split `editor.rs` (3,025 LOC)

Largest single file in the project. Propose splitting:

```
src/widgets/
  editor.rs          →  3,025 LOC — main logic, public API, cursor movement
  editor/                    ← NEW directory
    mod.rs                   ← re-exports
    selection.rs             ← selection logic (~400 LOC moved)
    syntax.rs                ← syntect integration (~300 LOC moved)
    movement.rs              ← cursor/goto/clamp logic (~500 LOC moved)
    history.rs               ← undo/redo stack (~400 LOC moved)
```

- [ ] Split without changing public API surface
- [ ] Move inline `#[cfg(test)] mod tests` into `$module/tests.rs`
- [ ] Verify no circular imports created

### 5. Split `utils.rs` (1,217 LOC)

Second-largest file. Catch-all for misc utilities.

- [ ] Extract `visual_width`, `truncate`, `formatting` → `src/text.rs` (already exists)
- [ ] Extract `clamp`, `bounding_box` → `src/layout.rs` (already exists)
- [ ] Extract `parse_hex_color`, `darken`, `lighten` → `src/visuals/` or `theme.rs`
- [ ] Remaining helpers → `src/framework/layout.rs` or dedicated `helpers.rs`

### 6. Test coverage gaps: 4 widgets with zero test files

| Widget | LOC | Tests? |
|--------|-----|--------|
| `list_common` | shared | ❌ 0 |
| `progress_ring` | 383 | ❌ 0 |
| `sparkline` | 455 | ❌ 0 |
| `text_editor_adapter` | — | ❌ 0 |

- [ ] Add minimal smoke tests for all 4
- [ ] Target: every framework widget covered by ≥1 integration test

### 7. Add `cargo outdated` to health checks

- [ ] Schedule quarterly `cargo outdated` review
- [ ] Dev-deps with major gaps: `criterion 0.5.1` → `0.8.2`, `itertools 0.10` → `0.13`
- [ ] Add `cargo upgrade` to maintenance workflow

### 8. Evaluate `App::new().unwrap()` in public API

`lib.rs` and `framework/mod.rs` both show `App::new().unwrap()` in doc examples. `App::new()` returns `io::Result<Self>`.

- [ ] Add `io::Result`-returning run variants: `App::try_run()` or propagate error in docs
- [ ] Document when `new()` can fail (terminal init, capabilities detection)

---

## 🟢 Low Priority

### 9. Docs and examples

- [ ] Add doc comments for all `pub fn` in `app.rs` (~30 public methods, some undocumented)
- [ ] Add compile-tested doc-examples for `App::on_input`, `App::on_tick`, `App::run`
- [ ] Add example for `MarqueeState` usage
- [ ] Add example for `SceneRouter` + embedded scenes pattern
- [ ] Currently 25 of 30 doc-tests are ```` ```ignore ```` — convert some to compile-tested where feasible

### 10. `text_input_base_test.rs` — 26 unit tests only

The `BaseInput` widget has decent coverage (26 tests) but no integration tests in `tests/`.

- [ ] Add integration test: Tab between fields, focus styling, scroll behavior
- [ ] Test mask/unmask toggle for `PasswordInput`

### 11. `lsp-server` extension unwrap cleanup

`extensions/lsp-server/src/main.rs` has **22 unwrap calls** in production code.

- [ ] Audit and replace with proper `Result` propagation
- [ ] Add error messages for each fallible operation

### 12. `cargo-dracon` scaffolding tool untested

`crates/cargo-dracon/src/` generates project templates but has zero tests.

- [ ] Add test: template generation produces compilable output
- [ ] Add snapshot tests for generated file contents

### 13. Code organization

- [ ] Move `src/compositor/size_test.rs` into `tests/` — it's a standalone size check, not a module
- [ ] Remove `src/input/mapping.rs` — contains a deprecated identity function (`UiEvent` → `Event`)
- [ ] Consider extracting `src/framework/prelude` into a standalone `prelude.rs` file

### 14. Build optimization

- [ ] Profile `debug` build time — check for slow generics (especially in `Plane`, `Compositor`, `Table<T>`)
- [ ] Add `lto = "thin"` for release builds
- [ ] Evaluate `codegen-units = 1` for release size/speed tradeoff
- [ ] Check if `bitflags::serde` feature is actually needed (adds `serde` dep to `bitflags`)

### 15. `CHANGELOG.md` format drift

Current entries use inconsistent subsection names ("Fixed" / "Changed"). Keep a Changelog spec recommends:

```
## [Unreleased]
### Added
### Changed
### Fixed
### Removed
### Security
```

- [ ] Enforce `keepachangelog.com` format in CI (via `changelog-check` or manual review)
- [ ] Add `[Unreleased]` section at top for tracking WIP changes

### 16. `dracon.toml` — no validation

- [ ] Add TOML schema validation (serde deserialize + structural check)
- [ ] Add unit tests for `KeybindingConfig::parse_keybinding()` edge cases (uppercase, malformed chords)
- [ ] Test `DraconError::InvalidKeybinding` path in error recovery

### 17. `unsafe` blocks — add proper `// SAFETY:` comments

Current locations:

| File | Count | Has SAFETY? |
|------|-------|-------------|
| `src/compositor/plane.rs` | 5 | ❌ |
| `src/backend/tty.rs` | 4 | ❌ |
| `src/framework/app.rs` | 2 | ✅ |
| `examples/showcase/main.rs` | 1 | ❌ |
| `examples/{game_loop,input_debug,desktop,arena}` | 1 each | ❌ |

- [ ] Add `// SAFETY:` preamble for every `unsafe` block in `src/`
- [ ] Examples less critical, but document for clarity

### 18. Event bus — no benchmarks

`event_bus.rs` (1,200+ loc) is used by `EventBusDemo` but has no performance tests.

- [ ] Add micro-benchmark: publish/subscribe throughput at 1/10/100 subscribers
- [ ] Add benchmark: filter vs unfiltered dispatch
- [ ] Add to `criterion` benchmark suite

---

## 🧪 Ideas (Further Investigation)

- **Panic safety audit**: Search for other potential panics (index arithmetic, `[..]` slicing)
- **Thread safety**: The framework is single-threaded by design; document as explicit constraint
- **Plugin architecture**: `PluginRegistry` exists but only one sample plugin; evaluate real-world use
- **Tracing feature**: Currently optional behind `tracing` feature flag; verify no performance regression when disabled
- **macOS/Windows testing**: `libc` gated to non-Windows; no macOS-specific test coverage found
- **Snapshot tests**: `insta` in dev-dependencies but no snapshot test files visible; add first snapshot for `Plane` or `Theme` serialization

---

## 📋 Summary of Failures Found

| Category | Items |
|----------|-------|
| Security advisories | 4 (1 unsound, 3 unmaintained) |
| Untested framework widgets | 4 (progress_ring, sparkline, list_common, text_editor_adapter) |
| CI/CD | 0 — no automated pipeline |
| Large files (>1,000 LOC) | 2 (editor 3,025, utils 1,217) |
| Production unwraps (non-test) | ~50 in `src/`, 22 in `extensions/lsp-server` |
| Unsafe blocks with missing SAFETY | 11 of 12 |

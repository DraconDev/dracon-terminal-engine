# Dracon Terminal Engine — TODO

Audit date: 2026-05-22  
Total LOC: 41,488  
Framework widgets: 41 · Themes: 21 · Examples: 57  
Public API items: 1,244 · Test functions: ~1,436  
Dependencies (transitive): 310 · Rc/RefCell uses: 403 · `unwrap()`/`expect()` calls: 129

---

## 🔴 High Priority

### 1. Fix transitive `lru` unsoundness — ✅ FIXED

Updated `ratatui` from 0.29.0 → 0.30.0 which bumps `lru` 0.12.5 → 0.16.4 (resolves RUSTSEC-2026-0002).
Ratatui 0.30 also splits into `ratatui-core` and `ratatui-widgets`, removes `cassowary`, updates `compact_str` and `unicode-width`.
Required `Backend` trait update: added `type Error = io::Error` and `clear_region()` implementation.

- [x] Update ratatui → 0.30.0
- [x] Fix `Backend` impl for new trait signature
- [x] Verify build + tests + clippy pass

### 2. CI pipeline — ✅ COMPLETE

- [x] GitHub Actions workflow: `ci.yml` — `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check`
- [x] `bench.yml` — criterion benchmarks with comparison
- [x] `plugin-ci.yml` — plugin smoke tests
- [x] `release.yml` — GitHub release artifacts
- [x] `rustsec/audit-check` in CI for security advisories

- [x] Add `cargo outdated` to health checks
- [x] Add markdown lint for `CHANGELOG.md` formatting

### 3. Security advisories — ✅ UPDATED

Updated `ratatui` 0.30 removes `paste` and updates `lru` (the unsound one). Still monitoring unmaintained transitive deps via `rustsec/audit-check`:

| Crate | Advisory | Action |
|-------|----------|--------|
| `bincode 1.3.3` | RUSTSEC-2025-0141 (unmaintained) | Monitor — no action possible |
| `yaml-rust 0.4.5` | RUSTSEC-2024-0320 (unmaintained) | Upstream (syntect) — monitor |
| `lru 0.16.4` | ~~RUSTSEC-2026-0002~~ | ✅ **Fixed** via ratatui 0.30 |

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

### 6. Test coverage gaps — ✅ ALL FIXED

| Widget | LOC | Status |
|--------|-----|--------|
| `progress_ring` | 383 | ✅ Added `tests/widget_progress_ring_test.rs` (38 tests) |
| `sparkline` | 455 | ✅ Added `tests/widget_sparkline_test.rs` (37 tests) |
| `list_common` | 196 | ✅ Added `tests/widget_list_common_test.rs` (25 tests) |
| `text_editor_adapter` | — | ✅ Already had 2 test files (`text_editor_adapter_test.rs`, `text_editor_adapter_edge_test.rs`) |

### 7. Add `cargo outdated` to health checks

- [ ] Schedule quarterly `cargo outdated` review
- [ ] Dev-deps with major gaps: `criterion 0.5.1` → `0.8.2`, `itertools 0.10` → `0.13`
- [ ] Add `cargo upgrade` to maintenance workflow

### 8. `App::new().unwrap()` in public API

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

## 📋 Summary of Completed / Remaining

| Category | Items | Status |
|----------|-------|--------|
| Security advisories | 2 (unmaintained only) | ✅ **DONE** |
| Untested framework widgets | 4 (progress_ring, sparkline, list_common, text_editor_adapter) | ✅ **DONE** |
| CI/CD pipeline | 6 workflows (ci, bench, plugin-ci, release, outdated, changelog) | ✅ **DONE** |
| Large files (>1,000 LOC) | 2 (editor 3,025, utils 1,217) | 🟡 Open |
| Production unwraps (non-test) | ~50 in `src/`, 22 in `extensions/lsp-server` | 🟡 Open |
| Unsafe blocks with missing SAFETY | 11 of 12 | 🟡 Open |
| `cargo outdated` integration | not in CI | 🟡 Open |
| Docs/examples | 25/30 doc-tests ignored | 🟢 Open |

**Completed:** 4 items (lru unsoundness, test coverage, CI pipeline, security advisories)
**Remaining:** 4 items

# Dracon Terminal Engine — Tasklist

**Status**: Active issue tracking
**Last Updated**: 2026-05-30

---

## 🚨 URGENT: git-seal stalled commits

**Repo**: `/home/dracon/Dev/git-seal`
**Issue**: 3 commits ahead of `origin/master`, not pushed

```
0f90aff chore: update CONTRIBUTING.md to remove CLA references
77143ff chore: remove CLA files, using standard AGPLv3 only
aedb5d0 4 file(s) in .env,.gitattributes,.gitignore
```

**Action**: Push to origin
```bash
cd /home/dracon/Dev/git-seal && git push origin main
```

---

## Quick Summary

| Category | Done | Total | Status |
|----------|------|-------|--------|
| P0 — Build & CI | 6 | 6 | ✅ 100% |
| P1 — Release/Metadata | 4 | 4 | ✅ 100% |
| P2 — API Cleanup | 1 | 5 | ⚠️ 20% |
| P3 — Testing | 3 | 6 | ⚠️ 50% |
| P4 — Documentation | 5 | 5 | ✅ 100% |
| P5 — Runtime | 3 | 4 | ⚠️ 75% |
| P6 — Refactors | 0 | 3 | ⏸️ Deferred |
| **Total** | **22** | **31** | **71%** |

---

## ✅ P0 — Build & CI Health (6/6 Complete)

- [x] Fix stale renamed-module imports in tests
- [x] Remove duplicate `#[test]` attributes
- [x] Run `cargo fmt --all` and commit formatting drift
- [x] Fix clippy warnings after test imports compile
- [x] Run full verification suite after P0 fixes

## ✅ P1 — Release & Metadata Correctness (4/4 Complete)

- [x] Fix release workflow packaging (LICENSE files)
- [x] Reconcile README, changelog, and crate metadata
- [x] Add release dry-run gate before publishing tags
- [x] Review package exclusions

## ⚠️ P2 — API Cleanup & Compatibility (1/5 Complete)

- [x] Remove or preserve compatibility aliases for renamed modules
- [ ] Finish deprecated `App::theme()` migration/removal plan
- [ ] Resolve duplicate I/O error variants in `DraconError`
- [ ] Standardize builder method ownership
- [ ] Decide fate of deprecated standalone widgets

## ⚠️ P3 — Testing Gaps (3/6 Complete)

- [x] Add regression tests for renamed module compatibility
- [x] Add `cargo-dracon` CLI integration tests
- [x] Add event bus benchmarks
- [ ] Add integration coverage for `SceneRouter` transitions
- [ ] Add plugin loading/unloading integration tests
- [ ] Expand widget interaction tests

## ✅ P4 — Documentation & Examples (5/5 Complete)

- [x] Update example/widget count docs
- [x] Update quick-start examples to current APIs
- [x] Document `Widget::render(&self)` design decision
- [x] Add public item docs in high-use widget modules
- [x] Consolidate audit files

## ⚠️ P5 — Runtime Robustness (3/4 Complete)

- [x] Review lsp-server unwrap-heavy JSON send paths
- [x] Add `dracon.toml` validation
- [ ] Revisit `App::default()` — add fallible constructor
- [ ] Implement or remove sixel decoding

## ⏸️ P6 — Maintainability Refactors (0/3 — Deferred)

> Large refactoring — do incrementally when touching related code.

### Long Functions (top priority)

| File | Function | Lines |
|------|----------|-------|
| `src/widgets/editor.rs` | `render()` | 764 |
| `src/widgets/editor.rs` | `handle_event()` | 488 |
| `src/compositor/engine.rs` | `render()` | 355 |
| `src/input/parser.rs` | `try_parse()` | 248 |

### Module Splitting

- [ ] Split `src/framework/command.rs`
- [ ] Split `src/framework/helpers.rs`
- [ ] Resolve `src/layout.rs` vs `src/framework/layout.rs`

---

## Verification Commands

```bash
cargo check --lib --all-features
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo test --all-features
```

**Last Verified**: 2026-05-29 ✅
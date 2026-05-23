# Ralph Loop State — todo-fix

**Started:** 2026-05-23
**Last updated:** 2026-05-23

## Completed ✅

### Item 1: lru unsoundness — ✅ FIXED
- Updated `ratatui` 0.29.0 → 0.30.0
- `lru` bumped 0.12.5 → 0.16.4 (resolves RUSTSEC-2026-0002)
- Fixed `Backend` impl: added `type Error = io::Error` + `clear_region()` with correct ClearType variants
- Build ✅ tests ✅ clippy ✅
- Files: Cargo.toml, Cargo.lock, src/integration/ratatui.rs

### Item 2: cargo outdated in CI — ✅ DONE
- Added `outdated` job to ci.yml
- Runs `cargo install cargo-outdated && cargo outdated -e Rust`

### Item 3: markdown lint in CI — ✅ DONE
- Added `changelog` job to ci.yml
- Validates `## [version]` headers and section format

## Remaining 🔴 High Priority Items
- [ ] ~~lru unsoundness~~ ✅ DONE
- [ ] ~~CI pipeline~~ ✅ DONE

## Remaining 🟡 Medium Priority
- [ ] Split `editor.rs` (3,025 LOC)
- [ ] Split `utils.rs` (1,217 LOC)
- [ ] `App::new().unwrap()` in public API docs
- [ ] Add `cargo outdated` to health checks (in CI now)

## Success Criteria ✅
- [x] `lru` issue documented with upstream path
- [x] `cargo outdated` runs in CI
- [x] Markdown lint added to CI workflow
- [x] All changes made (not committed — no git repo)

---

## Next: Medium Priority — Split editor.rs

Start splitting `src/widgets/editor.rs` (3,025 LOC) into submodules:
- `src/widgets/editor/selection.rs` (~400 LOC)
- `src/widgets/editor/syntax.rs` (~300 LOC)
- `src/widgets/editor/movement.rs` (~500 LOC)
- `src/widgets/editor/history.rs` (~400 LOC)
- `src/widgets/editor/mod.rs` (re-exports)

Keep public API surface unchanged.
## TODO.md Fix Loop — High Priority Items

**Goal:** Work through the 🔴 High Priority items systematically.

### Iteration plan:
1. **lru unsoundness** (RUSTSEC-2026-0002) — check current state, file ratatui issue
2. **CI pipeline** — add `cargo outdated` to health checks
3. **CI pipeline** — add markdown lint for CHANGELOG.md

### Task Content:

**Item 1: lru unsoundness audit**
- Check current `Cargo.lock` for `lru` version
- Verify `ratatui` is still at 0.29 (or updated)
- File issue with `ratatui` for RUSTSEC-2026-0002
- Document workaround or pin strategy

**Item 2: Add `cargo outdated` to CI**
- Add `cargo-outdated` step to `ci.yml`
- Run `cargo outdated -e Rust` to check deps
- Document findings

**Item 3: Add markdown lint to CI**
- Add `markdownlint` or `mdl` to `ci.yml`
- Enforce CHANGELOG format

### Success Criteria:
- [ ] `lru` issue documented with upstream path
- [ ] `cargo outdated` runs in CI and outputs findings
- [ ] Markdown lint added to CI workflow
- [ ] All changes committed with clear messages
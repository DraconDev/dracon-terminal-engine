# Ralph Loop State — todo-fix

**Started:** 2026-05-23
**Last updated:** 2026-05-23

## Completed ✅

### High Priority (Items 1-3) — ALL DONE
1. ✅ **lru unsoundness** — Fixed via ratatui 0.30.0
2. ✅ **CI pipeline** — Added outdated + changelog jobs
3. ✅ **Security advisories** — Updated table

### Medium Priority

#### editor.rs split — ❌ ABANDONED
Analysis revealed that all 34 private fns form a single tightly-coupled call graph.
Every function calls every other function (move_cursor calls all 25 others, etc.).
Attempting to split would require restructuring the entire call graph and
replacing `&mut self` method calls with a shared state struct — too invasive.

**Practical limit reached:** The internal coupling makes a clean submodule split
without a significant refactor impossible. This would be a major rewrite,
not a simple split. Recommend documenting this as a future refactor goal with
a specific architecture (e.g., extract state into `EditorState` struct,
then split methods by concern).

#### utils.rs split — 🟡 IN PROGRESS
- [ ] Check if utils.rs is similarly coupled
- [ ] Identify extractable utility functions (text, layout, color helpers)
- [ ] Extract into `src/text.rs`, `src/layout.rs`, `src/visuals/`

#### App::new().unwrap() docs — 🟢 TODO
- [ ] Update lib.rs and framework/mod.rs doc examples
- [ ] Document when `new()` can fail

## Remaining Items Summary

| Item | Status | Notes |
|------|--------|-------|
| lru unsoundness | ✅ DONE | ratatui 0.30, lru 0.16.4 |
| CI pipeline | ✅ DONE | +outdated, +changelog |
| Security advisories | ✅ DONE | Updated table |
| editor.rs split | ❌ SKIP | Tightly coupled monolith |
| utils.rs split | 🟡 TODO | Check coupling first |
| App::new().unwrap() | 🟢 TODO | Simple doc fix |
| Low priority items | 🟢 TODO | 11 remaining |

## Success Criteria
- [x] lru issue documented with upstream path
- [x] cargo outdated runs in CI
- [x] Markdown lint added to CI workflow
- [x] All changes made (no git repo)
- [ ] utils.rs extractable chunks identified
- [ ] App::new().unwrap() doc fix
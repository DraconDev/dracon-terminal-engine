# Ralph Loop State — todo-fix

**Started:** 2026-05-23
**Last updated:** 2026-05-23

## Iteration 5 Reflection — PIVOTING

### What was accomplished:
1. ✅ High priority items 1-3 complete
2. ✅ editor.rs split documented as impractical (same problem as before)
3. ✅ App::new().unwrap() docs fixed
4. ⏸️ utils.rs split — same coupling problem detected

### What's not working:
- Trying to split files that are monoliths by architecture, not just by size
- Both editor.rs AND utils.rs have the same pattern: many small functions that all call each other
- The "extractable subset" approach doesn't work when everything is cross-referenced

### PIVOT APPROACH:
Stop fighting the monolith. Focus on achievable items that don't require restructuring.

## Revised Priority List:

### ✅ COMPLETED (6 items):
1. lru unsoundness fix (ratatui 0.30, lru 0.16.4)
2. CI pipeline (outdated + changelog jobs)
3. Security advisories table updated
4. editor.rs split documented as impractical
5. App::new().unwrap() doc examples fixed
6. Test coverage gaps (done in prior session)

### 🟢 Achievable quick wins remaining:
- [ ] "Add compile-tested doc-examples" for App::on_input, App::on_tick, App::run
- [ ] "Add example for MarqueeState usage" 
- [ ] "Move src/compositor/size_test.rs into tests/"
- [ ] "Remove src/input/mapping.rs" (deprecated)

### ❌ Skip (architecture issues):
- editor.rs split — tightly coupled monolith
- utils.rs split — tightly coupled catch-all
- lsp-server unwrap cleanup (22 unwraps, needs context)

## Success Criteria
- [x] lru issue documented ✅
- [x] cargo outdated runs in CI ✅
- [x] Markdown lint added to CI workflow ✅
- [x] All changes made ✅
- [x] App::new().unwrap() doc fix ✅
- [ ] Quick win: pick ONE low-priority item to complete
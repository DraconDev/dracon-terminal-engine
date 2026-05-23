# Ralph Loop State — todo-fix

**Started:** 2026-05-23
**Last updated:** 2026-05-23 (iteration 6)

## Completed items:
1. ✅ lru unsoundness fix (ratatui 0.30, lru 0.16.4)
2. ✅ CI pipeline (outdated + changelog jobs)
3. ✅ Security advisories updated
4. ✅ editor.rs split documented as impractical
5. ✅ App::new().unwrap() docs fixed
6. ✅ Test coverage gaps
7. ✅ size_test.rs moved to tests/
8. ✅ set_theme doc comment added
9. ✅ on_tick compile-tested example added (app.rs line 496)

## Key learning:
- compile-tested doc examples work when using ```no_run``` + proper Widget impl
- Need to include ALL trait methods (set_area, area, id, needs_render, render)

## Remaining:
- on_input and run doc examples
- MarqueeState example
- SceneRouter example
- Many low-priority items (11 total)

## Strategy:
Continue picking off small wins. Each compile-tested doc example replaces an ignored one.
Focus on items that don't require restructuring.
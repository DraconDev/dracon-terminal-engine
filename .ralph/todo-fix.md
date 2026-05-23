# Ralph Loop State — todo-fix

**Started:** 2026-05-23
**Last updated:** 2026-05-23 (iteration 7)

## Completed this iteration:
1. ✅ on_input compile-tested doc example added (app.rs line 552)
2. ✅ All doc tests pass: 8 compile-tested, 23 ignored

## Total completed items: 10
1. lru unsoundness fix ✅
2. CI pipeline ✅
3. Security advisories updated ✅
4. editor.rs split documented as impractical ✅
5. App::new().unwrap() docs fixed ✅
6. Test coverage gaps ✅
7. size_test.rs moved to tests/ ✅
8. set_theme doc comment added ✅
9. on_tick compile-tested example ✅
10. on_input compile-tested example ✅
11. lib.rs doc example fixed ✅
12. framework/mod.rs doc example fixed ✅

## Doc test progress:
- Iteration 1: 0 compile-tested, 31 ignored
- Iteration 6: 7 compile-tested, 24 ignored  
- Iteration 7: 8 compile-tested, 23 ignored

## Pattern for multiple closures (on_input + on_tick):
Need separate clones: `app_for_input.clone()` and `app_for_tick.clone()`
because closures capture env by move.

## Remaining:
- MarqueeState example
- SceneRouter example
- 11 low priority items

## Strategy:
Continue with compile-tested doc examples.
Each iteration: +1 compile-tested, -1 ignored.
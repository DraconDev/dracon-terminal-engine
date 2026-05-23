# Ralph Loop State — todo-fix

**Started:** 2026-05-23
**Last updated:** 2026-05-23 (iteration 6)

## Completed this iteration:
1. ✅ Fixed lib.rs doc example (compile-tested, proper Widget impl, correct formatting)
2. ✅ Fixed framework/mod.rs doc example (compile-tested, proper Widget impl, use on_tick)
3. ✅ All doc tests now pass (7 compile-tested, 24 ignored)

## Pattern discovered:
Doc examples need:
1. `fn main() -> std::io::Result<()>` for `?` operator
2. Proper Widget trait impl (id, area, set_area, needs_render, render)
3. All on one line: `app.on_tick(...).run(|_| {})` — NOT chained with `?`
4. ````no_run` not ````ignore`

## Total completed items: 9
1. lru unsoundness fix ✅
2. CI pipeline ✅
3. Security advisories updated ✅
4. editor.rs split documented as impractical ✅
5. App::new().unwrap() docs fixed ✅
6. Test coverage gaps ✅
7. size_test.rs moved to tests/ ✅
8. set_theme doc comment added ✅
9. on_tick compile-tested example + fixed lib.rs + framework/mod.rs ✅

## Remaining:
- on_input doc example
- MarqueeState example
- SceneRouter example
- 11 low priority items

## Strategy:
Continue with compile-tested doc examples — each replaces an ignored one.
Track progress: 7 compile-tested (up from 0), 24 ignored (down from 31)
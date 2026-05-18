# Scene Enrichment Loop

Work through the ENRICHMENT.md plan systematically, upgrading spartan showcase scenes.

## Tier 1 — Most Spartan (50%+ empty)
1. [x] Enrich `modal_demo` — Add rich settings base screen, dimmed backdrop, toast stack, modal stacking
2. [x] Enrich `tooltip` — Add hoverable toolbar row, tooltip history sidebar, fill gaps
3. [x] Enrich `tags_input` — Add colored tag pills, shortcut legend, tag stats visualization
4. [x] Enrich `password_input` — Add requirements checklist with checkmarks, show/hide toggle, side panel
5. [x] Enrich `tree_navigator` — Add file type icons, size bars, content preview, search

## Tier 2 — Needs Polish (20-35% empty)
6. [x] Enrich `progress` — Add multi-stage subtasks, step dots, elapsed time, stage labels
7. [x] Enrich `form_demo` — Add section headers, inline validation, profile area, reset button
8. [x] Enrich `theme_switcher` — Expand preview area, per-swatch color dots, scroll all themes

## Tier 3 — Minor Gaps
9. [x] Enrich `cell_pool` — Add mini pool grid visualization, comparison chart
10. [x] Enrich `color_picker` — Add recent colors row, contrast checker, complementary suggestions
11. [x] Enrich `rich_text` — Add TOC sidebar, scroll indicator, word count, code block borders

## Per-item checklist
- [ ] Read current scene code
- [ ] Implement enrichment
- [ ] `cargo clippy --lib --examples` — 0 warnings
- [ ] `cargo test` — all pass
- [ ] Update `todo.md` — mark item done

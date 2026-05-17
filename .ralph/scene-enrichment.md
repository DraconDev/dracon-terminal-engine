# Scene Enrichment Loop

Work through the ENRICHMENT.md plan systematically, upgrading spartan showcase scenes.

## Tier 1 — Most Spartan (50%+ empty)
1. [ ] Enrich `modal_demo` — Add rich settings base screen, dimmed backdrop, toast stack, modal stacking
2. [ ] Enrich `tooltip` — Add hoverable toolbar row, tooltip history sidebar, fill gaps
3. [ ] Enrich `tags_input` — Add colored tag pills, shortcut legend, tag stats visualization
4. [ ] Enrich `password_input` — Add requirements checklist with checkmarks, show/hide toggle, side panel
5. [ ] Enrich `tree_navigator` — Add file type icons, size bars, content preview, search

## Tier 2 — Needs Polish (20-35% empty)
6. [ ] Enrich `progress` — Add multi-stage subtasks, step dots, elapsed time, stage labels
7. [ ] Enrich `form_demo` — Add section headers, inline validation, profile area, reset button
8. [ ] Enrich `theme_switcher` — Expand preview area, per-swatch color dots, scroll all themes

## Tier 3 — Minor Gaps
9. [ ] Enrich `cell_pool` — Add mini pool grid visualization, comparison chart
10. [ ] Enrich `color_picker` — Add recent colors row, contrast checker, complementary suggestions
11. [ ] Enrich `rich_text` — Add TOC sidebar, scroll indicator, word count, code block borders

## Per-item checklist
- [ ] Read current scene code
- [ ] Implement enrichment
- [ ] `cargo clippy --lib --examples` — 0 warnings
- [ ] `cargo test` — all pass
- [ ] Update `todo.md` — mark item done

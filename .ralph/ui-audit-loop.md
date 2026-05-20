# UI/UX Audit Loop

## Goal
Full verification pass: build, clippy, test, then systematically improve all showcase scenes.

## Checklist

### Phase 1 — Build & Clippy (always first)
- [ ] `cargo clippy --lib --examples 2>&1 | head -50` — must be clean
- [ ] `cargo test 2>&1 | tail -20` — must pass

### Phase 2 — Scene Quality Audit
Audit all 34 embedded scenes for:

**Layout & Structure**
- [ ] Split pane / sidebar layout (not flat vertical)
- [ ] Panel borders or card backgrounds (not flat wall of text)
- [ ] Visual hierarchy (surface, surface_elevated, bg)
- [ ] Footer with key hints

**Content Quality**
- [ ] Realistic demo data (not placeholder "Item 1")
- [ ] Empty states with icons + CTA
- [ ] Rich formatting per item type

**Interaction**
- [ ] Hover feedback on lists
- [ ] Live state updates
- [ ] Smooth transitions (or at least immediate visual feedback)

**Widgets Used**
- [ ] Composition of 2+ widgets (not single widget in a box)
- [ ] Proper widget forwarding (SearchInput, Select, etc.)

### Phase 3 — Priority Fixes
Fix in this order:

1. **widget_gallery** → DONE ✓ (sidebar + demo panel + properties)
2. **theme_switcher** → Split list + multi-widget preview + palette
3. **password_input** → Centered login form with real widgets + error state
4. **notification_center** → Split feed + detail panel
5. **color_picker** → Split picker + palette + CSS output

### Phase 4 — Remaining Scenes
Enrich remaining Tier 2-3 scenes:
- tags_input, progress, tree_navigator, radio, cell_pool, rich_text, animation, debug_overlay, tooltip

## Execution
- Each iteration: fix 1-2 scenes → verify build
- Every 5 iterations: reflect on progress, adjust priorities
- Final iteration: full verification pass
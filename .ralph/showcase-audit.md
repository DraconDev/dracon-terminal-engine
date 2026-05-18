
# Full Showcase Audit

## Goal
Systematically audit ALL showcase items (embedded scenes + external binary examples) for:
1. **Crash bugs** — panics, OOB access, unwrap on fallible ops, u16 underflow
2. **Interaction bugs** — keys not working, mouse not working, Esc/BACK inconsistency
3. **Visual bugs** — rendering issues, empty screens, misaligned content
4. **Missing features** — help overlay, status bar, theme propagation
5. **Code quality** — dead code, unused imports, clippy warnings

## Approach
1. Catalog all showcase items (embedded scenes + external binaries)
2. Read every scene file, check for each audit category
3. Check external binary examples
4. Fix all issues found
5. Final build verification

## Checklist (per scene)
- [ ] Compiles clean (no errors)
- [ ] No production unwraps (expect is OK for hardcoded values)
- [ ] No OOB array access without bounds check
- [ ] No u16 underflow in mouse handlers
- [ ] handle_key handles BACK/Esc consistently
- [ ] handle_key handles HELP/F1
- [ ] handle_mouse handles clicks and hover
- [ ] help overlay uses shared render_help_overlay()
- [ ] Theme propagation to all child widgets
- [ ] Plane background filled (no black holes)
- [ ] dirty flag set after mutations
- [ ] Status/footer with key hints
- [ ] No dead code or unused imports

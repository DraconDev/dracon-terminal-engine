# Dracon Terminal Engine — Audit Continuation (Round 2)

## Goal
Continue adding tests for remaining widgets to achieve 100% widget test coverage.

## Current Status
- **21 widgets tested**: 908 tests total
- **32 widgets remaining**
- **31% widget coverage** (21 of 68 total)

### Remaining Priority 2 Widgets (<200 LOC)
- ~~**EventLogger** (156 LOC) — ✅ 26 tests~~ — DONE
- **StatusBar** (186 LOC) — 10 tests (needs more)
- **Profiler** (176 LOC) — 10 tests (needs more)
- **Breadcrumbs** (178 LOC) — 0 tests
- **WidgetInspector** (160 LOC) — 0 tests
- **DebugOverlay** (129 LOC) — 11 tests (needs more)

### Priority 3 Widgets (<100 LOC)
- Various small widgets

## Target per Iteration
- Add tests for 1-2 widgets
- Each widget: 30-50 tests

## Success Criteria
- Each widget file created in `tests/widget_*_test.rs`
- All tests compile and pass
- audit.md updated with progress
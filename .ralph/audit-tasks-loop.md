# Audit Tasks Loop

## Goal
Work through the remaining P1-P5 tasks from tasks.md, prioritizing items that provide the most value.

## Remaining Tasks (by priority)

### P1 — Code Quality (43 remaining)

**Duplicate Type Consolidation — DONE:**
- [x] SelectCallback in list_common.rs (imported by autocomplete, tree)
- [x] SelectionChangeCallback in list_common.rs (imported by table, list)
- [x] UndoRedoCallback in list_common.rs (imported by table, list)

**Magic Number Constants — DONE:**
- [x] KITTY_PUA_START/END in kitty_key.rs
- [x] SIZE_GB, SIZE_MB, SIZE_KB in utils.rs
- [x] BINARY_CHECK_SIZE in utils.rs (8192)
- [x] READ_BUFFER_SIZE in reader.rs (1024)
- [x] MAX_BUFFER_SIZE in parser.rs (2048)
- [x] MS_PER_SEC in ctx.rs (1000.0)
- [x] INPUT_BUF_SIZE in app.rs (1024)

**Long Functions — DEFERRED (complex, low-risk):**
- editor.rs render() (764 lines) — high complexity, low risk
- editor.rs handle_event() (488 lines) — high complexity, low risk
- compositor/engine.rs render() (355 lines) — performance-critical, defer
- input/parser.rs try_parse() (248 lines) — already has good tests
- utils.rs spawn_terminal_at() (239 lines) — well-structured, low value

### P2 — Documentation (DONE!)

**All module docs added:**
- [x] backend/mod.rs, backend/tty.rs
- [x] compositor/*.rs (engine, filter, plane)
- [x] input/*.rs (event, mapping, parser, reader)
- [x] core/terminal.rs
- [x] visuals/*.rs (icons, osc)
- [x] widgets/*.rs (all 9 files)
- [x] system.rs, contracts.rs, layout.rs

### P3 — Architecture (7 remaining)

- [ ] Resolve layout.rs duplication (src/layout.rs vs framework/layout.rs)
- [ ] Module consolidation suggestions
- [x] Naming consistency: tabbar.rs → tab_bar.rs (done earlier)
- [x] Naming consistency: list_common.rs → list_helpers.rs (done this iteration)

### P4 — Error Handling (DONE)

- [x] Audit expect() in scene_router.rs — Justified preconditions (len > 1 / is_empty check guards)
- [x] App::default() expect() — Appropriate for Default trait (no alternative)
- [x] Signal registration expect() in reader.rs — Already replaced in earlier iteration

### P5 — Testing (DONE!)

- [x] Add tests for compositor/engine.rs — added 9 tests
- [x] Add tests for compositor/plane.rs — added 20 tests
- [x] Add tests for input/parser.rs — added 21 tests
- [x] Add tests for visuals/icons.rs — added 23 tests
- [x] Add tests for core/terminal.rs — added 9 tests
- [x] Add tests for system.rs — added 8 tests

## Approach

1. Start with P4 Error Handling (smallest scope, high value)
2. Then P1 Code Quality - magic numbers and type consolidation
3. Then P3 Architecture fixes
4. Then P2 Documentation
5. Then P5 Testing

Pick 1-3 items per iteration.
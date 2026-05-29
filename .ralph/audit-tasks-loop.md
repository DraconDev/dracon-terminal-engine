# Audit Tasks Loop

## Goal
Work through the remaining P1-P5 tasks from tasks.md, prioritizing items that provide the most value.

## Remaining Tasks (by priority)

### P1 — Code Quality (47 remaining)

**Duplicate Type Consolidation:**
1. Consolidate `SelectCallback` — defined in autocomplete.rs and tree.rs
2. Consolidate `SelectionChangeCallback` — defined in table.rs and list.rs  
3. Consolidate `UndoRedoCallback` — defined in table.rs and list.rs
4. Remove duplicate `Target` enum in `src/framework/app.rs` (lines 102 and 117)

**Magic Number Constants:**
1. Define named constants for Kitty protocol PUA codepoints in kitty_key.rs
2. Define named constants for byte size thresholds in utils.rs (GI_B, ME_B, KI_B)
3. Define named constant for binary detection buffer size in utils.rs (8192)
4. Define named constant for read buffer size in reader.rs (1024)
5. Define named constant for parser overflow threshold in parser.rs (2048)
6. Replace `1000.0` FPS constant in ctx.rs with Duration constant
7. Define named constants for pipe buffer sizes in app.rs (1024)

**Long Functions (>100 lines) — pick 3-5 to refactor:**
- editor.rs render() (764 lines)
- editor.rs handle_event() (488 lines)
- compositor/engine.rs render() (355 lines)
- input/parser.rs try_parse() (248 lines)
- utils.rs spawn_terminal_at() (239 lines)

### P2 — Documentation (DONE!)

**All module docs added:**
- [x] backend/mod.rs, backend/tty.rs
- [x] compositor/*.rs (engine, filter, plane)
- [x] input/*.rs (event, mapping, parser, reader)
- [x] core/terminal.rs
- [x] visuals/*.rs (icons, osc)
- [x] widgets/*.rs (all 9 files)
- [x] system.rs, contracts.rs, layout.rs

### P3 — Architecture (9 remaining)

- Resolve layout.rs duplication
- Module consolidation suggestions
- Naming consistency (tabbar.rs → tab_bar.rs)

### P4 — Error Handling (3 remaining)

- Replace expect() in reader.rs signal registration
- Replace expect() in app.rs App::from_default()
- Audit expect() in scene_router.rs

### P5 — Testing (2 remaining)

- [x] Add tests for compositor/engine.rs — added 9 tests
- [x] Add tests for compositor/plane.rs — added 20 tests
- [x] Add tests for input/parser.rs — added 21 tests
- [x] Add tests for visuals/icons.rs — added 23 tests
- [ ] Add tests for core/terminal.rs
- [x] Add tests for system.rs — added 8 tests

## Approach

1. Start with P4 Error Handling (smallest scope, high value)
2. Then P1 Code Quality - magic numbers and type consolidation
3. Then P3 Architecture fixes
4. Then P2 Documentation
5. Then P5 Testing

Pick 1-3 items per iteration.
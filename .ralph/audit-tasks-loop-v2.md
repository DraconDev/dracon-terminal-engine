# Audit Tasks Loop — Round 2

## Context
Previous loop completed 91/137 tasks (66%). Starting round 2 with 46 remaining.

## Progress This Round
1. ✅ Renamed `text_input_base.rs` → `text_input_core.rs` + updated all imports

## Remaining Tasks

### P3 — Architecture (5 remaining)
- [ ] Resolve layout.rs duplication (src/layout.rs vs framework/layout.rs)
- [ ] Module consolidation (breaking changes)
- [ ] Split command.rs, helpers.rs
- [x] Naming: text_input_base → text_input_core (done)

### P4 — Error Handling (1 remaining)
- [ ] App::from_default() requires Result return type change — **API breaking**, deferred

### P5 — Testing (1 remaining)
- [ ] Integration tests for scene_router/plugin loading

### P1 — Long Functions (5 deferred)
- editor.rs render() (764 lines)
- editor.rs handle_event() (488 lines)
- compositor/engine.rs render() (355 lines)
- etc.

## Approach
1. Pick 1-3 items per iteration
2. Update tasks.md with progress
3. Call ralph_done when complete
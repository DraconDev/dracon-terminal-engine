# Audit Tasks Loop — Round 2

## Context
Previous loop completed 91/137 tasks (66%). Remaining tasks are complex refactoring.

## Goal
Continue working through remaining tasks, prioritizing quick wins.

## Remaining Tasks

### P3 — Architecture (7 remaining)
- [ ] Resolve layout.rs duplication
- [ ] Module consolidation suggestions
- [ ] Split command.rs, helpers.rs

### P4 — Error Handling (1 remaining)
- [ ] App::from_default() requires Result return type change

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
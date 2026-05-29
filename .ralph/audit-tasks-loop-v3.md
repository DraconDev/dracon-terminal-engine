# Audit Tasks Loop — Round 3

## Context
Previous rounds completed 92/137 tasks (67%). Remaining 45 tasks are complex refactoring.

## Goal
Work through remaining tasks, prioritizing lower-risk items first.

## Remaining Tasks (by risk)

### Lower Risk (start here):
1. P1 Duplicated Code (5 items) — Extract shared patterns
2. P1 Unsafe Code (3 items) — Add safety comments
3. P5 Integration Tests (1 item) — scene_router, plugin tests

### Medium Risk:
4. P3 Module Consolidation (4 items) — Breaking API changes
5. P4 API Change (1 item) — App::from_default() Result return
6. P7 Sixel Feature (1 item) — New functionality

### Higher Risk (defer):
7. P1 Long Functions (26 items) — 100-764 lines each

## Approach
1. Start with P1 Duplicated Code extraction
2. Then P1 Unsafe Code audit
3. Then P5 Integration tests
4. Then P3 Module consolidation
5. Then remaining items
6. Call ralph_done after each item
7. Update tasks.md with progress
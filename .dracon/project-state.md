# Project State

## Current Focus
Refactored the `App` struct's tick callback type to be more explicit and encapsulated.

## Context
The previous implementation exposed the `TickCallback` type directly, which could lead to accidental misuse. This change improves type safety and encapsulation by making the callback an internal field of the `App` struct.

## Completed
- [x] Changed `pub(crate) type TickCallback` to `on_tick: RefCell<Option<TickCallback>>` to encapsulate the callback
- [x] Maintained the same functionality while improving type safety

## In Progress
- [ ] No active work in progress related to this change

## Blockers
- None identified for this specific change

## Next Steps
1. Verify no downstream effects from this refactoring
2. Update any related documentation or tests if needed

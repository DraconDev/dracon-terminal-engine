# Project State

## Current Focus
Refactored the `Select` widget's callback type to use a dedicated `ChangeCallback` type.

## Context
This change aligns with recent work to standardize callback types across widgets, improving type safety and consistency in the framework.

## Completed
- [x] Replaced `Box<dyn FnMut(&str)>` with `ChangeCallback` in the `Select` widget
- [x] Maintained existing functionality while improving type safety

## In Progress
- [ ] No active work in progress

## Blockers
- None

## Next Steps
1. Verify no runtime behavior changes in the `Select` widget
2. Update other widgets using similar callback patterns
```

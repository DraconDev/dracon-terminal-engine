# Project State

## Current Focus
Added a callback type for text input submission in the framework.

## Context
This change follows a pattern of refactoring widget callbacks to use dedicated types, improving type safety and clarity. The `SubmitCallback` type provides a consistent way to handle text input submissions across the framework.

## Completed
- [x] Added `SubmitCallback` type for text input submission handling
- [x] Documented the new callback type with a doc comment

## In Progress
- [ ] None

## Blockers
- None

## Next Steps
1. Implement the callback in the `BaseInput` widget
2. Update documentation to reflect the new callback type usage

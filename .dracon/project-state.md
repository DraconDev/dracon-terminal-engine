# Project State

## Current Focus
Added a generic callback type for table row selection handling.

## Context
This change follows a pattern of refactoring widget callback types to use dedicated types for better type safety and clarity, as seen in recent commits for the `Select` widget and `CommandPalette`.

## Completed
- [x] Added `SelectCallback<T>` type alias for table row selection handling
- [x] The type uses a boxed trait object for mutable callbacks

## In Progress
- [x] Implementation of table row selection using this callback type

## Blockers
- Need to verify callback behavior with the table's selection logic

## Next Steps
1. Implement table row selection using the new callback type
2. Add unit tests for the selection behavior

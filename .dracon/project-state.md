# Project State

## Current Focus
Added callback type aliases to improve code organization and reduce Clippy warnings

## Context
The changes introduce standardized type aliases for callback functions across the framework, making signatures cleaner and reducing "very complex type" warnings from Clippy.

## Completed
- [x] Added `TickCallback` for framework app timing
- [x] Added `ExecuteCallback` for command palette
- [x] Added `SelectCallback<T>` for list and table widgets
- [x] Added `ChangeCallback` for select widgets
- [x] Added `SubmitCallback` for text input
- [x] Added `SelectCallback` for tree widgets

## In Progress
- [ ] None (documentation-only change)

## Blockers
- None (documentation update)

## Next Steps
1. Review if additional callback types are needed
2. Ensure consistent usage across the codebase

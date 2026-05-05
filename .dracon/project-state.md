# Project State

## Current Focus
Added help overlay toggle functionality to the Git TUI and widget gallery examples

## Context
This change implements a consistent help overlay system across examples, allowing users to toggle keyboard shortcuts display with '?' and close with Escape

## Completed
- [x] Added `show_help` flag to both `CommandBindings` and `SplitResizerApp` structs
- [x] Implemented help overlay rendering in `SplitResizerApp`
- [x] Added keyboard shortcuts for help toggle ('?') and theme cycling ('t')
- [x] Updated Cargo.lock with dependency version bump

## In Progress
- [x] Help overlay implementation across multiple examples

## Blockers
- None identified

## Next Steps
1. Verify help overlay consistency across all examples
2. Document the new keyboard shortcuts in the examples' READMEs

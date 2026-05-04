# Project State

## Current Focus
Refactored widget callback types to use dedicated types for better type safety and clarity.

## Context
The recent commits show a pattern of adding generic callback types to improve type safety and code organization. This change continues that effort by introducing dedicated callback types for widget interactions.

## Completed
- [x] Refactored `List` widget to use `SelectCallback<T>` type instead of raw `Box<dyn FnMut(&T)>`
- [x] Added `ChangeCallback` type to `Select` widget for consistent callback handling

## In Progress
- [ ] No active work in progress shown in this diff

## Blockers
- None identified in this diff

## Next Steps
1. Verify the new callback types work correctly with existing widget implementations
2. Consider applying similar pattern to other widgets that use callbacks

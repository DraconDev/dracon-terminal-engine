# Project State

## Current Focus
Refactored the `Select` widget to improve type safety and code organization by introducing a dedicated `ChangeCallback` type alias for the on-change handler.

## Completed
- [x] Introduced `ChangeCallback` type alias for better type safety in the `Select` widget's on-change handler
- [x] Updated the `Select` struct to use the new `ChangeCallback` type instead of the raw closure type

# Project State

## Current Focus
Refactored table widget's selection callback to use a dedicated type for better type safety.

## Context
This change aligns with ongoing efforts to standardize callback types across widgets for improved type safety and maintainability.

## Completed
- [x] Replaced `Box<dyn FnMut(&T)>` with `SelectCallback<T>` in the Table widget

## In Progress
- [x] Ongoing refactoring of other widgets to use dedicated callback types

## Blockers
- None identified

## Next Steps
1. Continue refactoring remaining widgets to use dedicated callback types
2. Update documentation to reflect the new callback type usage

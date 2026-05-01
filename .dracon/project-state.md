# Project State

## Current Focus
Refactored widget callback types to improve type safety and code organization

## Completed
- [x] Refactored `List` widget to use direct `Option<Box<dyn FnMut(&T)>>` for selection callbacks
- [x] Refactored `Select` widget to use direct `Option<Box<dyn FnMut(&str)>>` for change callbacks

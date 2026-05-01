# Project State

## Current Focus
Refactored the `fps` method in the `App` struct to use `clamp` instead of manual min/max operations for better readability and maintainability.

## Completed
- [x] Replaced `fps.max(1).min(120)` with `fps.clamp(1, 120)` in the `App::fps` method

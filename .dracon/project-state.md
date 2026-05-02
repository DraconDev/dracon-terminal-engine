# Project State

## Current Focus
Expose `Column` and `TableRow` types from the `Table` widget module.

## Context
The `Table` widget was previously only exposing the `Table` type, but now needs to expose its internal types (`Column` and `TableRow`) to allow for more flexible table construction and manipulation.

## Completed
- [x] Added `Column` and `TableRow` to the public re-exports from the `Table` module

## In Progress
- [ ] None

## Blockers
- None

## Next Steps
1. Update any code that directly uses `Table`'s internal types to use the newly exported types
2. Verify all table-related examples and tests still work with the updated API

# Project State

## Current Focus
Added header click handling to the table widget for column sorting functionality.

## Context
The table widget needs to support user-initiated sorting by column headers. This change enables click callbacks on headers and prepares the data structure for tracking sort state.

## Completed
- [x] Added `HeaderClickCallback` type for header click handling
- [x] Added `sort_column` and `sort_ascending` fields to track sort state

## In Progress
- [ ] Implementation of actual sorting logic when headers are clicked

## Blockers
- Need to implement the actual sorting behavior that will use these new fields

## Next Steps
1. Implement the sorting logic that uses the new callback and state fields
2. Add visual indicators for sorted columns (arrows, highlighting)

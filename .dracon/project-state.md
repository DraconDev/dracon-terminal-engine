# Project State

## Current Focus
Refactored table widget mouse handling to properly expose column index

## Context
This change was prompted by the ongoing work to add column sorting functionality to the table widget. The previous implementation was discarding the column index parameter in the mouse event handler.

## Completed
- [x] Removed unused `_col` parameter in table mouse handler
- [x] Exposed column index parameter for use in sorting functionality

## In Progress
- [x] Implementation of column sorting based on mouse clicks

## Blockers
- Need to implement the actual sorting logic after header clicks

## Next Steps
1. Implement column sorting logic when headers are clicked
2. Add visual indicators for sorted columns

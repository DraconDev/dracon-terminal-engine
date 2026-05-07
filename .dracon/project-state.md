# Project State

## Current Focus
Improved bounds checking in GitTui file selection logic

## Context
The change addresses potential integer underflow in file selection calculations when `row` is less than `content_y`. This prevents incorrect negative indices when calculating file positions in the UI.

## Completed
- [x] Replaced subtraction with `saturating_sub` to prevent negative indices
- [x] Maintained same functionality for valid cases

## In Progress
- [x] Testing edge cases where `row` is less than `content_y`

## Blockers
- None identified

## Next Steps
1. Verify no visual regression in file selection UI
2. Add unit tests for edge cases

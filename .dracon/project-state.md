# Project State

## Current Focus
Added mouse event logging to the showcase example for debugging purposes

## Context
This change enhances the showcase example's debugging capabilities by logging mouse events when input debugging is enabled. This helps developers track and verify mouse interactions during development.

## Completed
- [x] Added mouse event logging when `show_input_debug` is enabled
- [x] Limited event log to 16 most recent entries for performance
- [x] Included timestamp and event details in the log

## In Progress
- [x] Mouse event logging implementation

## Blockers
- None identified

## Next Steps
1. Verify the logging works correctly in the showcase example
2. Consider adding similar logging for other input events if needed

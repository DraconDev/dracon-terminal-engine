# Project State

## Current Focus
Added support for unsupported terminal events in the input debugger.

## Context
The input debugger was enhanced to handle all possible terminal events, including those marked as unsupported. This ensures comprehensive event logging and debugging capabilities.

## Completed
- [x] Added handling for `Event::Unsupported` in the input debugger
- [x] Updated Cargo.lock to reflect dependency changes

## In Progress
- [ ] None

## Blockers
- None

## Next Steps
1. Verify the new event handling works as expected in terminal environments
2. Consider expanding the debugger to include more detailed unsupported event information if needed

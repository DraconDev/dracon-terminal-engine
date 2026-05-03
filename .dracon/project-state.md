# Project State

## Current Focus
Added FPS counter to showcase example for performance monitoring

## Context
To help developers visualize and debug rendering performance in the showcase example, we need a way to display frames-per-second metrics.

## Completed
- [x] Added `fps` field to `Showcase` struct to track frame rate
- [x] Initialized `fps` with default value of 0

## In Progress
- [x] Implementation of actual FPS calculation and display

## Blockers
- Need to implement the actual FPS measurement logic that updates this value

## Next Steps
1. Implement FPS calculation logic that updates the cell value
2. Add UI display for the FPS counter in the showcase example

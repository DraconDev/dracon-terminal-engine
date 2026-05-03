# Project State

## Current Focus
Added FPS display toggle to the showcase example

## Context
This change enables developers to visualize performance metrics by adding a toggleable FPS counter to the showcase example. This supports debugging and optimization work by providing real-time performance feedback.

## Completed
- [x] Added `show_fps` boolean field to the Showcase struct to track FPS display state

## In Progress
- [x] Implementation of actual FPS calculation and display logic (not yet in this commit)

## Blockers
- Need to implement FPS calculation and rendering logic in the next commit

## Next Steps
1. Implement FPS calculation and display logic
2. Add keyboard shortcut to toggle FPS display (likely 'F' key)

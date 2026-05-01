# Project State

## Current Focus
Implement periodic automatic execution of widget-bound commands with configurable refresh intervals, enabling widgets to update their state at set time periods via a new command tracking system in the application framework.

## Completed
- [x] Import HashMap to support command tracking storage
- [x] Add `command_tracking` field to App struct storing widget IDs, last command run times, and refreshable bound commands
- [x] Initialize `command_tracking` in all App constructor paths
- [x] Register widget commands with valid `refresh_seconds` values in `command_tracking` when widgets are added to the app
- [x] Add periodic command execution logic to the main app loop: execute tracked commands after their refresh interval elapses, apply output to the target widget, mark the widget as dirty, and update the last run time for rescheduled commands

# Project State

## Current Focus
Fix child process terminal corruption in showcase example by adding terminal suspend/resume functionality

## Context
Showcase launches child TUI examples with `Command::spawn()`, which inherits the parent's raw terminal state (alternate screen, raw mode, mouse capture, hidden cursor), causing instant corruption/breakage.

## Completed
- [x] Added `suspend()` and `resume()` methods to `Terminal` to properly handle terminal state transitions
- [x] Modified showcase to use suspend/resume around child process execution
- [x] Implemented proper child process cleanup and terminal state restoration

## In Progress
- [x] Verification of terminal state transitions during child process execution

## Blockers
- None identified

## Next Steps
1. Verify child process terminal state transitions work correctly
2. Add error handling for terminal state operations
3. Consider adding timeout for child process execution

# Project State

## Current Focus
Refactored terminal window size detection and widget initialization in cookbook examples

## Context
These changes standardize how terminal window size is determined across examples by using the same file descriptor-based approach introduced in recent commits. This improves consistency and reduces code duplication.

## Completed
- [x] Replaced compositor-based size detection with direct terminal size detection using file descriptors
- [x] Simplified widget initialization by removing redundant context usage
- [x] Standardized widget area calculation across both examples

## In Progress
- [x] All cookbook examples now use consistent terminal size detection

## Blockers
- None identified

## Next Steps
1. Review other examples for similar refactoring opportunities
2. Consider adding window resize handling to these examples

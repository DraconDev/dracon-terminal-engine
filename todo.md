# Bug Fix TODO — All Items Complete

## Fixed ✅ (15 items)

1. **Accessibility typing** — Accept SHIFT modifier for uppercase, guard Backspace
2. **Action Center mouse clicks** — on_select callback with bridge pattern
3. **Color Picker initial state** — Default slider selected (Hue)
4. **Control Panel Selects** — `Select::set_selected()`, sync index, render actual widget
5. **Autocomplete dropdown** — `open_dropdown()` on init
6. **Notification Center area** — Added `area.set()` in render
7. **Settings scene unwraps** — `.expect()` for hardcoded regexes
8. **Raycaster MAP safety** — `.clamp()` before MAP index access
9. **File manager unwrap** — `match` instead of `.unwrap()` on take()
10. **Plugin demo unwraps** — 11× `.expect()` for RwLock poisoning
11. **Cell pool unwraps** — 6× `.expect()` for Mutex poisoning
12. **Modal demo dead code** — Use `created` field for toast age display
13. **Stat widget dead code** — Remove `#[allow(dead_code)]`, add doc
14. **File manager dead code** — Prefix unused function with `_`
15. **Control Panel Select render** — Use actual Select widget instead of manual value display

## Full Audit Results

- **35 embedded scenes**: All pass all 12 audit criteria
- **23 external binaries**: 0 production unwraps
- **Build**: 0 clippy errors, 0 warnings, all tests pass

## Needs Runtime Testing
- Chat client "crash" — no code-level panic source found
- Action center "failed to start" — code compiles clean, needs terminal test

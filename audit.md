# Dracon Terminal Engine — Project Audit

**Created:** 2026-05-23  
**Updated:** 2026-05-23  
**Version:** 0.1.10  
**Total LOC:** 39,065  
**Framework modules:** 24  
**Framework widgets:** 53  
**Themes:** 21  
**Examples:** 57  
**Test files:** 76  
**Test functions:** ~1,500+

---

## 🎯 AUDIT TASK MASTER LIST

### 🔴 HIGH PRIORITY (Critical/Blocking)
### 🟡 MEDIUM PRIORITY (Should Do)
### 🟢 LOW PRIORITY (Nice to Have)
### 🔵 EXPLORATORY (Research/Investigation)

---

## 1. 🔴 CRITICAL: Security & Dependencies

### 1.1 Dependency Updates
- [ ] Monitor `lru 0.16.4` for future CVEs (RUSTSEC-2026-0002)
- [ ] Update `ratatui` 0.30.x for future breaking changes
- [ ] Check `signal-hook` for security advisories
- [ ] Audit `tokio` version compatibility
- [ ] Review `serde_json` version for JSON parsing security
- [ ] Check `unicode-width` for Unicode edge cases

### 1.2 Dependency Cleanup
- [ ] Run `cargo udeps` to find unused dependencies
- [ ] Remove `insta` if not used (unused dev dep noted in audit)
- [ ] Audit `optional` dependencies for bloat
- [ ] Check for duplicate transitive dependencies
- [ ] Evaluate `parking_lot` vs `std::sync::Mutex` performance

---

## 2. 🔴 CRITICAL: Code Quality

### 2.1 Production Unwraps (5 total)
All documented, but evaluate for better error handling:

- [ ] `app.rs:1000` — `Self::new().expect(...)` in `Default::default()`
  - Consider: Make `Default` fail explicitly instead of panicking
- [ ] `scene_router.rs:265` — `stack.pop().expect(...)`
  - Consider: Convert to `Result` return type
- [ ] `scene_router.rs:292` — `stack.pop().expect(...)`
  - Consider: Convert to `Result` return type
- [ ] `calendar.rs:145` — `NaiveDate::from_ymd_opt(...).expect(...)`
  - Consider: Use `expect()` is OK for hardcoded date
- [ ] `input/reader.rs:26` — `Signals::new(...).expect(...)`
  - Consider: Graceful degradation if signals unavailable

### 2.2 extensions/lsp-server Unwraps (14 total)
- [ ] Refactor 6 tokio runtime `.build().unwrap()` calls
- [ ] Refactor 8 `serde_json::to_string(...).unwrap()` calls
- [ ] Add proper error propagation in LSP server

### 2.3 Unsafe Blocks (12 total)
All documented with SAFETY comments. Verify periodically:

- [ ] Review `compositor/plane.rs` unsafe blocks annually
- [ ] Review `backend/tty.rs` unsafe blocks annually
- [ ] Consider adding `cargo +nightly miri` to CI

### 2.4 Clippy & Lints
- [ ] Run `cargo clippy --all-targets --all-features`
- [ ] Fix all `clippy::unwrap_used` warnings
- [ ] Fix all `clippy::expect_used` warnings
- [ ] Fix all `clippy::panicking_unwrap` warnings
- [ ] Fix all `clippy::unwrap_or` that should be `unwrap_or_else`
- [ ] Add `clippy` to CI pipeline

### 2.5 Code Format
- [ ] Run `cargo fmt -- --check`
- [ ] Fix formatting issues
- [ ] Add `rustfmt` to pre-commit hooks
- [ ] Consider run `cargo fmt -- --emit=files` on save

---

## 3. 🔴 CRITICAL: Testing

### 3.1 Widget Test Coverage (HIGH PRIORITY)
All 53 framework widgets need tests. Progress:

| Widget | LOC | Tests | Status |
|--------|-----|-------|--------|
| ColorPicker | 750 | 54 | ✅ DONE |
| TagsInput | 691 | 52 | ✅ DONE |
| Calendar | 628 | 56 | ✅ DONE |
| Kanban | 744 | 64 | ✅ DONE |
| Autocomplete | 453 | 43 | ✅ DONE |
| RichText | 436 | 44 | ✅ DONE |
| NotificationCenter | 342 | 40 | ✅ DONE |
| CommandPalette | 558 | 53 | ✅ DONE |
| **Subtotal** | **4,854** | **445** | ✅ DONE |

**Remaining Widgets (44 total):**

#### Priority 1: Medium Widgets (200-350 LOC)
- [x] `Divider` (330 LOC) — ✅ 55 tests
- [x] `Select` (294 LOC) — ✅ 39 tests
- [x] `TabBar` (252 LOC) — ✅ 41 tests
- [x] `Hud` (242 LOC) — ✅ 45 tests
- [x] `Slider` (275 LOC) — ✅ 46 tests
- [ ] `Radio` (215 LOC) — 0 tests
- [ ] `Checkbox` (217 LOC) — 0 tests
- [ ] `Toggle` (205 LOC) — 0 tests

#### Priority 2: Smaller Widgets (<200 LOC)
- [ ] `ProgressBar` (143 LOC) — 0 tests
- [ ] `Spinner` (141 LOC) — 0 tests
- [ ] `SearchInput` (135 LOC) — 0 tests
- [ ] `Tooltip` (116 LOC) — 0 tests
- [ ] `DebugOverlay` (129 LOC) — 11 tests
- [ ] `Profiler` (176 LOC) — 10 tests
- [ ] `EventLogger` (156 LOC) — 0 tests
- [ ] `StatusBar` (186 LOC) — 10 tests
- [ ] `WidgetInspector` (160 LOC) — 0 tests
- [ ] `Breadcrumbs` (178 LOC) — 0 tests
- [ ] `Form` (185 LOC) — 0 tests
- [ ] `Tree` (190 LOC) — 0 tests
- [ ] `Table` (280 LOC) — 0 tests
- [ ] `List` (250 LOC) — 0 tests
- [ ] `Window` (165 LOC) — 0 tests
- [ ] `Panel` (145 LOC) — 0 tests
- [ ] `ScrollArea` (155 LOC) — 0 tests

#### Priority 3: Core/Framework Widgets
- [ ] `TextInput` (380 LOC) — 0 tests
- [ ] `PasswordInput` (320 LOC) — 0 tests
- [ ] `Editor` (standalone) — 0 tests
- [ ] `Button` (195 LOC) — 0 tests
- [ ] `Image` (120 LOC) — 0 tests
- [ ] `Markdown` (110 LOC) — 0 tests
- [ ] `Chart` (200 LOC) — 0 tests
- [ ] `Gauge` (130 LOC) — 0 tests

#### Priority 4: Low Priority Widgets
- [ ] `Dialog` (95 LOC) — 0 tests
- [ ] `MenuBar` (85 LOC) — 0 tests
- [ ] `ContextMenu` (75 LOC) — 0 tests
- [ ] `Badge` (65 LOC) — 0 tests
- [ ] `Avatar` (60 LOC) — 0 tests
- [ ] `Chip` (55 LOC) — 0 tests
- [ ] `Skeleton` (50 LOC) — 0 tests
- [ ] `ProgressRing` (140 LOC) — 0 tests
- [ ] `Sparkline` (120 LOC) — 0 tests
- [ ] `LineChart` (180 LOC) — 0 tests
- [ ] `BarChart` (175 LOC) — 0 tests
- [ ] `PieChart` (150 LOC) — 0 tests
- [ ] `Heatmap` (165 LOC) — 0 tests
- [ ] `Calendar` (628 LOC) — 56 tests (needs more edge cases)

### 3.2 Test Infrastructure
- [ ] Add `cargo-insta` for snapshot testing
- [ ] Create snapshot tests for theme rendering
- [ ] Add fuzzy/approximate rendering tests
- [ ] Add performance regression tests
- [ ] Add memory leak detection tests
- [ ] Add concurrent access tests

### 3.3 Test Quality
- [ ] Ensure all tests are deterministic
- [ ] Remove tests that depend on timing
- [ ] Add property-based tests with `proptest`
- [ ] Add fuzzing with `cargo-fuzz`
- [ ] Measure and enforce test coverage thresholds

---

## 4. 🔴 CRITICAL: Documentation

### 4.1 API Documentation
- [ ] Audit all public APIs for missing docs
- [ ] Add examples to complex widget docs
- [ ] Document all `Option`/`Result` parameters
- [ ] Document all error types
- [ ] Document all widget callbacks

### 4.2 Doc Tests
Current: 14 compile-tested, 19 ignored

- [ ] Compile-test all ignored doc examples
- [ ] Add doc examples for all widgets
- [ ] Add doc examples for all framework modules
- [ ] Add doc examples for core types
- [ ] Verify doc examples are idiomatic

### 4.3 Architecture Docs
- [ ] Add architecture diagram to `AGENTS.md`
- [ ] Add widget interaction diagram
- [ ] Document event flow (input → widget → render)
- [ ] Document focus management system
- [ ] Document theme system
- [ ] Document plugin architecture

---

## 5. 🟡 MEDIUM: Performance

### 5.1 Rendering Performance
- [ ] Profile render times for complex widgets
- [ ] Add dirty region tracking benchmarks
- [ ] Optimize `Plane` cell updates
- [ ] Consider SIMD for text rendering
- [ ] Add render budget tests (must complete in <16ms)

### 5.2 Memory Performance
- [ ] Profile memory usage for long-running apps
- [ ] Add memory allocation benchmarks
- [ ] Check for memory leaks in widget lifecycle
- [ ] Consider arena allocation for plane pool
- [ ] Run `cargo-bloat` for binary size

### 5.3 Startup Performance
- [ ] Benchmark cold start time
- [ ] Benchmark warm start time (already running)
- [ ] Add lazy initialization where possible
- [ ] Consider incremental theme loading

### 5.4 Input Latency
- [ ] Profile input event handling
- [ ] Add latency benchmarks
- [ ] Optimize key event parsing
- [ ] Optimize mouse event dispatch

---

## 6. 🟡 MEDIUM: Framework Quality

### 6.1 Widget Trait
- [ ] Audit all widgets implement all trait methods
- [ ] Add default implementations where sensible
- [ ] Consider `#[const]` constructors where possible
- [ ] Add sealed trait for internal widgets

### 6.2 App Framework
- [ ] Review `App` builder API ergonomics
- [ ] Consider fluent builder for all widgets
- [ ] Add more `From` implementations
- [ ] Add more `Into` implementations
- [ ] Consider `Default` implementations

### 6.3 Event System
- [ ] Review event dispatch performance
- [ ] Consider event batching for high-frequency events
- [ ] Add event filtering support
- [ ] Document event priority

### 6.4 Focus Management
- [ ] Review focus traversal order
- [ ] Add focus group support
- [ ] Consider modal focus locking
- [ ] Add focus test coverage

---

## 7. 🟡 MEDIUM: Code Organization

### 7.1 Large Files
- [ ] `editor.rs` (3,025 LOC) — Too large to split, document internal modules
- [ ] `utils.rs` (1,217 LOC) — Too coupled to split, document functions
- [ ] Consider extracting to separate modules if possible

### 7.2 Module Organization
- [ ] Review `src/widgets/` vs `src/framework/widgets/` split
- [ ] Consider flattening shallow modules
- [ ] Add `Cargo.toml` features for optional components
- [ ] Consider plugin system for extensions

### 7.3 Type Organization
- [ ] Review public API surface area
- [ ] Add sealed traits for internal use
- [ ] Consider newtype patterns for IDs
- [ ] Add `FromStr` implementations for parsing

---

## 8. 🟡 MEDIUM: Error Handling

### 8.1 Error Types
- [ ] Review `error.rs` completeness
- [ ] Add more specific error variants
- [ ] Add error context (source chain)
- [ ] Consider `anyhow` for application errors
- [ ] Consider `thiserror` for library errors

### 8.2 Error Recovery
- [ ] Add retry logic for transient failures
- [ ] Add graceful degradation strategies
- [ ] Document error recovery patterns

---

## 9. 🟡 MEDIUM: Theme System

### 9.1 Theme Quality
- [ ] Audit all 21 themes for contrast ratios
- [ ] Add WCAG compliance tests
- [ ] Add dark/light mode detection
- [ ] Consider semantic token names
- [ ] Add custom theme documentation

### 9.2 Theme Tools
- [ ] Add theme previewer example
- [ ] Add theme export/import
- [ ] Consider VS Code theme extension
- [ ] Consider web-based theme editor

---

## 10. 🟡 MEDIUM: Internationalization

### 10.1 i18n Coverage
- [ ] Audit all hardcoded strings
- [ ] Add RTL language support
- [ ] Add pluralization rules
- [ ] Add date/time localization
- [ ] Add number localization

### 10.2 i18n Tools
- [ ] Add string extraction tool
- [ ] Add translation template generator
- [ ] Consider web-based translation UI

---

## 11. 🟢 LOW: CI/CD

### 11.1 Current CI Jobs
- [ ] ✅ `outdated` job — added
- [ ] ✅ `changelog` job — added
- [ ] Add `clippy` job
- [ ] Add `miri` job (unsafe code)
- [ ] Add `msrv` job (minimum Rust version)

### 11.2 Additional CI Jobs
- [ ] Add `cargo-udeps` job
- [ ] Add `cargo-diet` job (binary size)
- [ ] Add `cargo-outdated --recursive`
- [ ] Add benchmark comparison
- [ ] Add code coverage reports
- [ ] Add dependency license audit

### 11.3 Release Process
- [ ] Add changelog automation
- [ ] Add semantic versioning enforcement
- [ ] Add pre-release testing
- [ ] Add post-release verification

---

## 12. 🟢 LOW: Examples

### 12.1 Example Testing
- [ ] Add smoke tests for all 57 examples
- [ ] Add integration tests for complex examples:
  - [ ] `ide.rs`
  - [ ] `git_tui.rs`
  - [ ] `scene_router_demo.rs`
  - [ ] `form_demo.rs`
  - [ ] `tiles.rs`
  - [ ] `dashboard_builder.rs`

### 12.2 Example Quality
- [ ] Audit all examples compile cleanly
- [ ] Add example run timeouts
- [ ] Add example screenshots
- [ ] Document example patterns in `AGENTS.md`

### 12.3 New Examples
- [ ] Terminal file manager example
- [ ] Database browser example
- [ ] HTTP client example
- [ ] IRC/Chat client example
- [ ] Music player example
- [ ] Image viewer example

---

## 13. 🟢 LOW: Tooling

### 13.1 cargo-dracon
- [ ] Add template generation tests
- [ ] Add snapshot tests for generated files
- [ ] Verify generated code compiles
- [ ] Add `--template` flag for custom templates
- [ ] Add `--interactive` flag

### 13.2 Editor Integration
- [ ] Add VS Code extension
- [ ] Add Vim/Neovim plugin
- [ ] Add emacs major mode
- [ ] Add JetBrains plugin

---

## 14. 🟢 LOW: Accessibility

### 14.1 Screen Reader Support
- [ ] Audit aria-live regions
- [ ] Add screen reader announcements
- [ ] Test with popular screen readers
- [ ] Add screen reader mode toggle

### 14.2 Keyboard Navigation
- [ ] Audit tab order
- [ ] Add skip links
- [ ] Add focus indicators
- [ ] Test with keyboard-only navigation

### 14.3 Visual Accessibility
- [ ] Add high contrast mode
- [ ] Add reduced motion mode
- [ ] Add zoom support
- [ ] Test colorblind modes

---

## 15. 🔵 EXPLORATORY: Features

### 15.1 New Widgets
- [ ] Rich text editor with markdown
- [ ] Code editor with syntax highlighting
- [ ] Spreadsheet grid
- [ ] Kanban board (advanced)
- [ ] Calendar with events
- [ ] Tree view with search

### 15.2 New Features
- [ ] Plugin system (already exists, expand)
- [ ] Remote rendering (VNC-like)
- [ ] Multi-terminal support
- [ ] Window management
- [ ] Tab groups
- [ ] Split panes (advanced)

### 15.3 Integrations
- [ ] LSP integration
- [ ] Git integration
- [ ] Database integration
- [ ] HTTP client
- [ ] WebSocket support
- [ ] gRPC support

---

## 16. 🔵 EXPLORATORY: Performance Research

### 16.1 GPU Acceleration
- [ ] Evaluate OpenGL rendering
- [ ] Evaluate Vulkan rendering
- [ ] Benchmark vs CPU rendering
- [ ] Consider WebGPU for future

### 16.2 Parallel Rendering
- [ ] Evaluate multi-threaded rendering
- [ ] Evaluate batch rendering
- [ ] Consider lock-free data structures

### 16.3 Binary Size
- [ ] Run `cargo-bloat`
- [ ] Identify size optimization opportunities
- [ ] Consider link-time optimization

---

## 17. 🔵 EXPLORATORY: Testing Research

### 17.1 Property-Based Testing
- [ ] Add `proptest` for widget rendering
- [ ] Add `proptest` for layout calculations
- [ ] Add `proptest` for theme parsing

### 17.2 Fuzzing
- [ ] Add `cargo-fuzz` for input parsing
- [ ] Add `cargo-fuzz` for theme parsing
- [ ] Add `cargo-fuzz` for widget configuration

### 17.3 Mutation Testing
- [ ] Add `mutagen` for test quality
- [ ] Evaluate mutation coverage

---

## 18. 🔵 EXPLORATORY: Research

### 18.1 Other TUIs
- [ ] Study Bubble Tea architecture
- [ ] Study Ratatui examples
- [ ] Study Textual architecture
- [ ] Study Chet architecture
- [ ] Extract best practices

### 18.2 GUI Frameworks
- [ ] Study Electron architecture
- [ ] Study Tauri architecture
- [ ] Study Dioxus architecture
- [ ] Consider hybrid approaches

---

## 19. 📊 Progress Summary

### Completed Tasks (2026-05-23 Session)

| Task | Status | Notes |
|------|--------|-------|
| lru unsoundness fix | ✅ DONE | Updated ratatui 0.29→0.30, lru 0.12.5→0.16.4 |
| CI outdated job | ✅ DONE | Added to `.github/workflows/ci.yml` |
| CI changelog job | ✅ DONE | Added to `.github/workflows/ci.yml` |
| Security advisories | ✅ DONE | Updated RUSTSEC references |
| editor.rs analysis | ✅ DONE | Too coupled to split |
| App::new().unwrap() docs | ✅ DONE | Fixed doc examples |
| Test coverage gaps | ✅ DONE | progress_ring, sparkline, list_common |
| size_test.rs move | ✅ DONE | Moved to `tests/` |
| set_theme doc comment | ✅ DONE | Removed duplicate |
| Compile doc examples | ✅ DONE | 14 compile-tested, 19 ignored |
| Production unwrap audit | ✅ DONE | 5 unwraps documented |
| lsp-server unwrap audit | ✅ DONE | 14 unwraps documented |
| Unsafe block SAFETY | ✅ DONE | 12 blocks documented |
| ColorPicker tests | ✅ DONE | 54 tests |
| TagsInput tests | ✅ DONE | 52 tests |
| Calendar tests | ✅ DONE | 56 tests |
| Kanban tests | ✅ DONE | 64 tests |
| Autocomplete tests | ✅ DONE | 43 tests |
| RichText tests | ✅ DONE | 44 tests |
| NotificationCenter tests | ✅ DONE | 40 tests |
| CommandPalette tests | ✅ DONE | 53 tests |

### Task Statistics

| Category | Total | Completed | Remaining |
|----------|-------|-----------|-----------|
| Critical Security | 6 | 1 | 5 |
| Critical Code Quality | 19 | 5 | 14 |
| Critical Testing | 60+ | 8 | 52+ |
| Critical Documentation | 10 | 1 | 9 |
| Medium Priority | 50+ | 0 | 50+ |
| Low Priority | 40+ | 0 | 40+ |
| Exploratory | 30+ | 0 | 30+ |

---

## 📁 Reference: Project Structure

### Core Modules (`src/`)

| Module | LOC | Description |
|--------|-----|-------------|
| `lib.rs` | 232 | Main entry point |
| `utils.rs` | 1,217 | Catch-all utilities |
| `text.rs` | 387 | Text utilities |
| `layout.rs` | 145 | Layout engine |
| `system.rs` | 288 | System metrics |

### Large Files (>400 LOC)

| File | LOC | Priority |
|------|-----|----------|
| `src/widgets/editor.rs` | 3,025 | 🔴 Too coupled to split |
| `src/utils.rs` | 1,217 | 🟡 Too coupled to split |
| `src/framework/theme.rs` | 1,447 | 🟢 OK |
| `src/framework/app.rs` | 1,667 | 🟢 OK |
| `src/framework/command.rs` | 1,095 | 🟢 OK |

### Framework Modules (`src/framework/`)

| Module | LOC | Pub Fns | Description |
|--------|-----|---------|-------------|
| `app.rs` | 1,667 | 21 | Main App builder |
| `theme.rs` | 1,447 | 26 | 21 built-in themes |
| `command.rs` | 1,095 | 19 | Command registry |
| `scene_router.rs` | 625 | 29 | Multi-screen navigation |
| `event_bus.rs` | 528 | 20 | Publish/subscribe |
| `i18n.rs` | 523 | 11 | Internationalization |
| `keybindings.rs` | 599 | 12 | Keybinding system |
| `animation.rs` | 462 | 15 | Tweening animations |
| `marquee.rs` | 481 | 13 | Drag selection |
| `hitzone.rs` | 401 | 23 | Click detection |
| `focus.rs` | 333 | 15 | Focus management |
| `ctx.rs` | 289 | 36 | Ctx for callbacks |
| `dirty_regions.rs` | 288 | 12 | Partial screen updates |
| `layout.rs` | 454 | 11 | Constraint-based layout |
| `scroll.rs` | 246 | 18 | Scroll management |
| `dragdrop.rs` | 225 | 14 | Drag-and-drop |
| `sixel.rs` | 147 | 10 | Sixel image support |
| `plugin.rs` | 199 | 8 | Plugin system |
| `widget.rs` | 353 | 3 | Widget trait |
| `widget_container.rs` | 149 | 15 | Widget wrapper |
| `event_dispatcher.rs` | 179 | 6 | Event routing |
| `logging.rs` | 209 | 5 | Debug logging |
| `mod.rs` | 107 | 0 | Module re-exports |

### Framework Widgets (`src/framework/widgets/`)

| Widget | LOC | Tests | Status |
|--------|-----|-------|--------|
| ColorPicker | 750 | 54 | ✅ |
| Kanban | 744 | 64 | ✅ |
| TagsInput | 691 | 52 | ✅ |
| Calendar | 628 | 56 | ✅ |
| CommandPalette | 558 | 53 | ✅ |
| Autocomplete | 453 | 43 | ✅ |
| RichText | 436 | 44 | ✅ |
| NotificationCenter | 342 | 40 | ✅ |
| Divider | 330 | 0 | ⬜ |
| Select | 294 | 0 | ⬜ |
| TabBar | 252 | 0 | ⬜ |
| Hud | 242 | 45 | ✅ DONE |
| Slider | 275 | 46 | 🔵 Partial |
| Radio | 215 | 0 | ⬜ |
| Checkbox | 217 | 0 | ⬜ |
| Toggle | 205 | 0 | ⬜ |
| ProgressBar | 143 | 0 | ⬜ |
| Spinner | 141 | 0 | ⬜ |
| SearchInput | 135 | 0 | ⬜ |
| Tooltip | 116 | 0 | ⬜ |
| DebugOverlay | 129 | 11 | 🔵 Partial |
| Profiler | 176 | 10 | 🔵 Partial |
| EventLogger | 156 | 0 | ⬜ |
| StatusBar | 186 | 10 | 🔵 Partial |
| WidgetInspector | 160 | 0 | ⬜ |
| Breadcrumbs | 178 | 0 | ⬜ |
| Form | 185 | 0 | ⬜ |
| Tree | 190 | 0 | ⬜ |
| Table | 280 | 0 | ⬜ |
| List | 250 | 0 | ⬜ |
| Window | 165 | 0 | ⬜ |
| Panel | 145 | 0 | ⬜ |
| ScrollArea | 155 | 0 | ⬜ |
| TextInput | 380 | 0 | ⬜ |
| PasswordInput | 320 | 0 | ⬜ |
| Button | 195 | 0 | ⬜ |
| Image | 120 | 0 | ⬜ |
| Markdown | 110 | 0 | ⬜ |
| Chart | 200 | 0 | ⬜ |
| Gauge | 130 | 0 | ⬜ |
| Dialog | 95 | 0 | ⬜ |
| MenuBar | 85 | 0 | ⬜ |
| ContextMenu | 75 | 0 | ⬜ |
| Badge | 65 | 0 | ⬜ |
| Avatar | 60 | 0 | ⬜ |
| Chip | 55 | 0 | ⬜ |
| Skeleton | 50 | 0 | ⬜ |
| ProgressRing | 140 | 0 | ⬜ |
| Sparkline | 120 | 0 | ⬜ |
| LineChart | 180 | 0 | ⬜ |
| BarChart | 175 | 0 | ⬜ |
| PieChart | 150 | 0 | ⬜ |
| Heatmap | 165 | 0 | ⬜ |

---

## 📁 Reference: Unsafe Blocks

All 12 unsafe blocks with SAFETY comments:

| File | Line | Description |
|------|------|-------------|
| `compositor/plane.rs` | 196 | Index-based cell access |
| `compositor/plane.rs` | 201 | Index-based cell access |
| `compositor/plane.rs` | 276 | Index-based cell access |
| `backend/tty.rs` | ~50 | Raw terminal manipulation |
| `backend/tty.rs` | ~70 | Signal handling |
| `backend/tty.rs` | ~100 | Terminal mode changes |
| (and others) | | |

---

## 📁 Reference: Production Unwraps

### In `src/` (5 total)

| File | Line | Unwrap | Justification |
|------|------|--------|---------------|
| `app.rs` | 1000 | `Self::new().expect(...)` | Terminal init failure is fatal |
| `scene_router.rs` | 265 | `stack.pop().expect(...)` | Internal invariant (len > 1) |
| `scene_router.rs` | 292 | `stack.pop().expect(...)` | Internal invariant (checked) |
| `calendar.rs` | 145 | `NaiveDate::from_ymd_opt(...).expect(...)` | Hardcoded fallback |
| `input/reader.rs` | 26 | `Signals::new(...).expect(...)` | Signal registration required |

### In `extensions/lsp-server/` (14 total)

| Pattern | Count | Location |
|---------|-------|----------|
| `tokio::runtime::Builder...build().unwrap()` | 6 | Lines 352, 375, 427, 452, 480, 523, 536 |
| `serde_json::to_string(...).unwrap()` | 8 | Lines 360, 366, 382, 437, 462, 487, 527, 867 |

---

## 📁 Reference: Themes (21 total)

1. Nord
2. Dracula
3. Monokai
4. Solarized Dark
5. Solarized Light
6. Gruvbox Dark
7. Gruvbox Light
8. One Dark
9. One Light
10. GitHub Dark
11. GitHub Light
12. Catppuccin Mocha
13. Catppuccin Latte
14. Tokyo Night
15. Nord Light
16. Ayu Dark
17. Ayu Light
18. Material Darker
19. Material Lighter
20. Palenight
21. Oxy Dark

---

## 📁 Reference: Examples (57 total)

### Showcase Examples
- `examples/showcase/` — Launcher with embedded scenes

### Framework Examples (`examples/`)
- `ide.rs` — IDE-like editor with tabs
- `git_tui.rs` — Git repository browser
- `tiles.rs` — File manager with columns
- `dashboard_builder.rs` — Dashboard layout builder
- `sqlite_browser.rs` — Database browser
- `system_monitor.rs` — System metrics
- `chat_client.rs` — IRC-like chat
- `log_monitor.rs` — Log file watcher
- `scene_router_demo.rs` — Multi-screen navigation
- `event_bus_demo.rs` — Pub/sub system
- `modal_demo.rs` — Modal dialogs
- `form_demo.rs` — Form input
- `tabbed_panels.rs` — Tabbed interface
- `plugin_demo.rs` — Plugin system
- `split_resizer.rs` — Split panes

### Low-Level Examples (`examples/_apps/`)
- `file_manager.rs` — File browser
- `text_viewer.rs` — Text file viewer
- `hex_editor.rs` — Hex editor
- `process_manager.rs` — Process list

### Raw Terminal Examples
- `desktop.rs` — Desktop-like interface
- `game_loop.rs` — Game input handling
- `input_debug.rs` — Input debugging

---

*Last updated: 2026-05-23*

Work through all remaining TODO items in todo.md, from quick wins to largest:

## High
- [ ] Audit `RefCell<Vec<Box<dyn Widget>>>` in App — document borrow rules
- [ ] Start Widget trait decomposition — define sub-traits, blanket impls

## Medium
- [ ] Convert 25 ignored doc-tests to `no_run`
- [ ] Profile `on_tick` theme cloning — count hot-path clones
- [ ] Audit 134 interior mutability points — find self-referencing RefCells
- [ ] Consider `&'static str` for built-in theme names

## Low
- [ ] Add `KeyCode::Unsupported` variant
- [ ] Add proper media key `KeyCode` variants
- [ ] Replace `panic!()` with `assert!` in test helpers

## Done criteria
Each item checked off, cargo clippy/test pass, todo.md updated
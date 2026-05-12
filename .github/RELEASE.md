# Release Checklist

This document outlines the steps to release a new version of dracon-terminal-engine to crates.io.

## Pre-release Checklist

Before starting the release process:

- [ ] All tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code is formatted: `cargo fmt -- --check`
- [ ] CHANGELOG.md is updated with release notes
- [ ] Version number is bumped in Cargo.toml
- [ ] No large binary files or unnecessary assets in the package

## Release Steps

### 1. Validate Package (Dry Run)

```bash
cargo publish --dry-run
```

This validates that the package will be accepted by crates.io. Fix any errors before proceeding.

### 2. Update Version

Update the version in `Cargo.toml`:

```toml
[package]
version = "X.Y.Z"
```

### 3. Update CHANGELOG

Create a new section at the top of `CHANGELOG.md`:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- ...

### Changed
- ...

### Fixed
- ...
```

### 4. Commit Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "Release vX.Y.Z"
```

### 5. Create Git Tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

### 6. Publish to crates.io

```bash
cargo publish
```

### 7. Create GitHub Release

Create a GitHub Release for the tag:
- Go to https://github.com/DraconDev/dracon-terminal-engine/releases/new
- Select the tag you just created
- Copy the relevant section from CHANGELOG.md as the release description
- Publish the release

## Post-release Checklist

- [ ] Verify package appears on crates.io
- [ ] Verify documentation builds at docs.rs
- [ ] Announce release (if applicable)

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):
- **MAJOR** version for incompatible API changes
- **MINOR** version for new functionality in a backwards compatible manner
- **PATCH** version for backwards compatible bug fixes
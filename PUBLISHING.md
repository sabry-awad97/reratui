# Publishing Guide for Reratui

This guide covers version management, release workflows, and publishing to crates.io.

## 🚀 Quick Start

For most releases, use the automated Python scripts:

```bash
# 1. Bump version (automatically updates all crates, runs tests, creates tag)
python scripts/bump_version.py minor

# 2. Publish to crates.io (in dependency order)
cargo publish --package reratui-panic
cargo publish --package reratui-core
cargo publish --package reratui-macro
cargo publish --package reratui-hooks
cargo publish --package reratui-ratatui
cargo publish --package reratui-runtime
cargo publish --package reratui

# 3. Create GitHub release
python scripts/create_release.py
```

See [`scripts/README.md`](scripts/README.md) for detailed script documentation.

## Version Management

Each crate has its own explicit version in its `Cargo.toml` file. This allows for:

- Independent versioning if needed
- Clearer dependency management
- Better tooling compatibility

## Current Versions (v0.2.0)

All crates are currently at version **0.2.0**:

- `reratui-panic` - 0.2.0
- `reratui-core` - 0.2.0
- `reratui-macro` - 0.2.0
- `reratui-hooks` - 0.2.0
- `reratui-ratatui` - 0.2.0
- `reratui-runtime` - 0.2.0
- `reratui` - 0.2.0

## Publishing Order

Crates must be published in dependency order:

```powershell
# 1. Base crates (no internal dependencies)
cargo publish --package reratui-panic
cargo publish --package reratui-core

# 2. Macro crate
cargo publish --package reratui-macro

# 3. Hooks (depends on core and panic)
cargo publish --package reratui-hooks

# 4. Ratatui adapter (depends on core)
cargo publish --package reratui-ratatui

# 5. Runtime (depends on core, hooks, panic)
cargo publish --package reratui-runtime

# 6. Main crate (depends on all above)
cargo publish --package reratui
```

## 🔄 Automated Version Bumping

Use the `bump_version.py` script for automated version management:

### Basic Usage

```bash
# Patch version (0.2.0 -> 0.2.1)
python scripts/bump_version.py patch

# Minor version (0.2.0 -> 0.3.0)
python scripts/bump_version.py minor

# Major version (0.2.0 -> 1.0.0)
python scripts/bump_version.py major

# Specific version
python scripts/bump_version.py --version 0.3.0
```

### What the Script Does

1. ✅ Updates version in all crate `Cargo.toml` files
2. ✅ Updates internal dependency versions
3. ✅ Updates workspace version
4. ✅ Runs `cargo build --workspace --lib` to verify
5. ✅ Runs `cargo test --workspace --lib` (optional with `--no-test`)
6. ✅ Commits changes with version message
7. ✅ Creates git tag (e.g., `v0.3.0`)
8. ✅ Pushes to remote (optional with `--no-push`)

### Advanced Options

```bash
# Skip tests (faster for minor changes)
python scripts/bump_version.py patch --no-test

# Don't push to remote (review changes first)
python scripts/bump_version.py minor --no-push

# Custom tag message
python scripts/bump_version.py patch --tag-message "Hotfix: Fix critical bug"

# Don't create tag
python scripts/bump_version.py minor --no-tag

# Don't commit (manual control)
python scripts/bump_version.py patch --no-commit
```

## 📝 Manual Version Bump Checklist

If you need to bump versions manually:

1. **Update crate versions** in these files:

   - `crates/reratui-panic/Cargo.toml`
   - `crates/reratui-core/Cargo.toml`
   - `crates/reratui-macro/Cargo.toml`
   - `crates/reratui-hooks/Cargo.toml`
   - `crates/reratui-ratatui/Cargo.toml`
   - `crates/reratui-runtime/Cargo.toml`
   - `crates/reratui/Cargo.toml`

2. **Update internal dependencies** in:

   - `crates/reratui-hooks/Cargo.toml` (depends on core, panic)
   - `crates/reratui-ratatui/Cargo.toml` (depends on core)
   - `crates/reratui-runtime/Cargo.toml` (depends on core, hooks, panic)
   - `crates/reratui/Cargo.toml` (depends on all above)

3. **Update workspace version** in root `Cargo.toml`

4. **Run tests**:

   ```bash
   cargo test --workspace --lib
   cargo test --doc --workspace
   ```

5. **Format and commit**:
   ```bash
   cargo fmt
   git add .
   git commit -m "chore: bump version to X.Y.Z"
   git tag vX.Y.Z
   git push origin main --tags
   ```

## 📦 GitHub Release Creation

Use the `create_release.py` script to create GitHub releases:

### Basic Usage

```bash
# Auto-detect version from Cargo.toml
python scripts/create_release.py

# Specific version
python scripts/create_release.py --version 0.3.0

# Custom title
python scripts/create_release.py --title "v0.3.0 - Amazing Features"

# Custom notes file
python scripts/create_release.py --notes RELEASE_NOTES_v0.3.0.md
```

### What the Script Does

1. ✅ Auto-detects version from `Cargo.toml`
2. ✅ Checks for release notes file (`RELEASE_NOTES_vX.Y.Z.md`)
3. ✅ Creates GitHub release using `gh` CLI (if available)
4. ✅ Falls back to manual instructions with browser link

### Requirements

- **GitHub CLI** (optional but recommended)
  ```bash
  winget install --id GitHub.cli
  ```

## 🔍 Pre-Publish Checklist

Before publishing, ensure:

- [ ] All tests pass: `cargo test --workspace`
- [ ] Documentation builds: `cargo doc --workspace --no-deps`
- [ ] Examples compile: `cargo build --examples`
- [ ] Formatting is correct: `cargo fmt --check`
- [ ] No clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] CHANGELOG.md is updated
- [ ] README.md reflects new features
- [ ] Version numbers are consistent across all crates

## 📋 Complete Release Workflow

### 1. Prepare Release

```bash
# Update code, add features, fix bugs
git add .
git commit -m "feat: add awesome feature"
```

### 2. Bump Version

```bash
# Use automated script
python scripts/bump_version.py minor

# Or manually update versions
# See "Manual Version Bump Checklist" above
```

### 3. Create Release Notes

Create `RELEASE_NOTES_vX.Y.Z.md` with:

- What's new
- Breaking changes
- Bug fixes
- Examples

See `RELEASE_NOTES_v0.2.0.md` for template.

### 4. Publish to crates.io

```bash
# Publish in dependency order
cargo publish --package reratui-panic
cargo publish --package reratui-core
cargo publish --package reratui-macro
cargo publish --package reratui-hooks
cargo publish --package reratui-ratatui
cargo publish --package reratui-runtime
cargo publish --package reratui

# Wait for crates.io to index between publishes
```

### 5. Create GitHub Release

```bash
python scripts/create_release.py
```

### 6. Announce

- Update README.md badges
- Post on social media
- Update documentation site

## 💡 Tips & Best Practices

### Testing Before Publish

```bash
# Dry run to check what would be published
cargo publish --package reratui --dry-run

# Check package contents
cargo package --package reratui --list
```

### Handling Publish Errors

If a publish fails:

1. **Check crates.io status**: https://status.crates.io/
2. **Verify version isn't already published**
3. **Check dependency versions are available**
4. **Wait a few minutes and retry**

### Version Strategy

- **Patch (0.2.x)**: Bug fixes, documentation updates
- **Minor (0.x.0)**: New features, backward compatible
- **Major (x.0.0)**: Breaking changes

## 🔗 Useful Links

- **Scripts Documentation**: [`scripts/README.md`](scripts/README.md)
- **crates.io**: https://crates.io/crates/reratui
- **docs.rs**: https://docs.rs/reratui
- **GitHub Releases**: https://github.com/sabry-awad97/reratui/releases

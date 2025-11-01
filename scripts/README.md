# Reratui Release Scripts

Python scripts for managing versions and releases of Reratui.

## Scripts

### `bump_version.py` - Version Bumping & Git Tagging

Automatically bumps version across all crates, runs tests, and creates git tags.

**Usage:**

```bash
# Bump patch version (0.2.0 -> 0.2.1)
python scripts/bump_version.py patch

# Bump minor version (0.2.0 -> 0.3.0)
python scripts/bump_version.py minor

# Bump major version (0.2.0 -> 1.0.0)
python scripts/bump_version.py major

# Set specific version
python scripts/bump_version.py --version 0.3.0

# Skip tests
python scripts/bump_version.py patch --no-test

# Don't push to remote
python scripts/bump_version.py minor --no-push

# Custom tag message
python scripts/bump_version.py patch --tag-message "Hotfix release"
```

**What it does:**

1. ✅ Updates version in all crate `Cargo.toml` files
2. ✅ Updates internal dependency versions
3. ✅ Updates workspace version
4. ✅ Verifies build (`cargo build --workspace --lib`)
5. ✅ Runs tests (`cargo test --workspace --lib`)
6. ✅ Commits changes to git
7. ✅ Creates git tag (e.g., `v0.3.0`)
8. ✅ Pushes to remote

**Options:**

- `--version X.Y.Z` - Set specific version instead of bumping
- `--no-test` - Skip running tests
- `--no-commit` - Don't commit changes
- `--no-tag` - Don't create git tag
- `--no-push` - Don't push to remote
- `--tag-message "msg"` - Custom tag message

### `create_release.py` - GitHub Release Creation

Creates GitHub releases with release notes.

**Usage:**

```bash
# Create release for current version (auto-detected)
python scripts/create_release.py

# Create release for specific version
python scripts/create_release.py --version 0.3.0

# Custom title
python scripts/create_release.py --title "v0.3.0 - Amazing Features"

# Custom notes file
python scripts/create_release.py --notes RELEASE_NOTES_v0.3.0.md
```

**What it does:**

1. ✅ Auto-detects version from `Cargo.toml`
2. ✅ Checks for release notes file
3. ✅ Creates GitHub release using `gh` CLI (if available)
4. ✅ Falls back to manual instructions with browser link

**Options:**

- `--version X.Y.Z` - Version to release (default: auto-detect)
- `--title "Title"` - Release title (default: `vX.Y.Z`)
- `--notes file.md` - Release notes file (default: `RELEASE_NOTES_vX.Y.Z.md`)

**Requirements:**

- GitHub CLI (`gh`) - Optional but recommended
  - Install: `winget install --id GitHub.cli`
  - Or download from: https://cli.github.com/

## Complete Release Workflow

### 1. Bump Version

```bash
# Bump version, run tests, create tag, and push
python scripts/bump_version.py minor
```

This will:

- Update all crate versions
- Run tests
- Commit changes
- Create git tag
- Push to GitHub

### 2. Publish to crates.io

```bash
# Publish in dependency order
cargo publish --package reratui-panic
cargo publish --package reratui-core
cargo publish --package reratui-macro
cargo publish --package reratui-hooks
cargo publish --package reratui-ratatui
cargo publish --package reratui-runtime
cargo publish --package reratui
```

### 3. Create Release Notes

Create a file named `RELEASE_NOTES_vX.Y.Z.md` with your release notes.

See `RELEASE_NOTES_v0.2.0.md` for an example.

### 4. Create GitHub Release

```bash
python scripts/create_release.py
```

## Examples

### Patch Release (Bug Fix)

```bash
# 1. Bump version
python scripts/bump_version.py patch

# 2. Publish crates
cargo publish --package reratui-panic
# ... (publish all crates)

# 3. Create release notes
# Edit RELEASE_NOTES_v0.2.1.md

# 4. Create GitHub release
python scripts/create_release.py
```

### Minor Release (New Features)

```bash
# 1. Bump version
python scripts/bump_version.py minor --tag-message "New form hooks and validators"

# 2. Publish crates
# ... (publish all crates)

# 3. Create release notes
# Edit RELEASE_NOTES_v0.3.0.md

# 4. Create GitHub release
python scripts/create_release.py --title "v0.3.0 - Form Enhancements"
```

### Major Release (Breaking Changes)

```bash
# 1. Bump version
python scripts/bump_version.py major --tag-message "v1.0.0 - Stable Release"

# 2. Publish crates
# ... (publish all crates)

# 3. Create release notes
# Edit RELEASE_NOTES_v1.0.0.md

# 4. Create GitHub release
python scripts/create_release.py
```

## Troubleshooting

### "Could not find version in Cargo.toml"

Make sure the root `Cargo.toml` has a version field in the `[workspace.package]` section.

### "Build failed"

Fix the build errors before proceeding. The script will not continue if the build fails.

### "Tests failed"

You can skip tests with `--no-test` or fix the failing tests.

### "GitHub CLI not found"

Install GitHub CLI:

```bash
winget install --id GitHub.cli
```

Or create the release manually using the provided link.

## Requirements

- Python 3.7+
- Git
- Cargo/Rust
- GitHub CLI (optional, for `create_release.py`)

## License

MIT OR Apache-2.0

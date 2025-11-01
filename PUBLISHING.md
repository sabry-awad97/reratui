# Publishing Guide for Reratui

## Version Management

Each crate now has its own explicit version in its `Cargo.toml` file instead of using `version.workspace = true`. This makes it easier to publish individual crates with different versions if needed.

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

## Version Bump Checklist

When bumping versions:

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

3. **Update workspace version** in root `Cargo.toml` (optional, for reference)

4. **Run tests**:

   ```powershell
   cargo test --workspace --lib
   cargo test --doc --workspace
   ```

5. **Format and commit**:

   ```powershell
   cargo fmt
   git add .
   git commit -m "chore: bump version to X.Y.Z"
   git tag vX.Y.Z
   git push origin main --tags
   ```

6. **Publish** (in order shown above)

## Quick Version Bump Script

For a minor version bump (e.g., 0.2.0 -> 0.3.0):

```powershell
# Update all version fields
$OLD_VERSION = "0.2.0"
$NEW_VERSION = "0.3.0"

# Update crate versions
(Get-Content crates/reratui-panic/Cargo.toml) -replace "version = `"$OLD_VERSION`"", "version = `"$NEW_VERSION`"" | Set-Content crates/reratui-panic/Cargo.toml
(Get-Content crates/reratui-core/Cargo.toml) -replace "version = `"$OLD_VERSION`"", "version = `"$NEW_VERSION`"" | Set-Content crates/reratui-core/Cargo.toml
(Get-Content crates/reratui-macro/Cargo.toml) -replace "version = `"$OLD_VERSION`"", "version = `"$NEW_VERSION`"" | Set-Content crates/reratui-macro/Cargo.toml
(Get-Content crates/reratui-hooks/Cargo.toml) -replace "version = `"$OLD_VERSION`"", "version = `"$NEW_VERSION`"" | Set-Content crates/reratui-hooks/Cargo.toml
(Get-Content crates/reratui-ratatui/Cargo.toml) -replace "version = `"$OLD_VERSION`"", "version = `"$NEW_VERSION`"" | Set-Content crates/reratui-ratatui/Cargo.toml
(Get-Content crates/reratui-runtime/Cargo.toml) -replace "version = `"$OLD_VERSION`"", "version = `"$NEW_VERSION`"" | Set-Content crates/reratui-runtime/Cargo.toml
(Get-Content crates/reratui/Cargo.toml) -replace "version = `"$OLD_VERSION`"", "version = `"$NEW_VERSION`"" | Set-Content crates/reratui/Cargo.toml

# Update internal dependencies
(Get-Content crates/reratui-hooks/Cargo.toml) -replace "reratui-core = \{ version = `"$OLD_VERSION`"", "reratui-core = { version = `"$NEW_VERSION`"" | Set-Content crates/reratui-hooks/Cargo.toml
(Get-Content crates/reratui-hooks/Cargo.toml) -replace "reratui-panic = \{ version = `"$OLD_VERSION`"", "reratui-panic = { version = `"$NEW_VERSION`"" | Set-Content crates/reratui-hooks/Cargo.toml
(Get-Content crates/reratui-ratatui/Cargo.toml) -replace "reratui-core = \{ version = `"$OLD_VERSION`"", "reratui-core = { version = `"$NEW_VERSION`"" | Set-Content crates/reratui-ratatui/Cargo.toml
(Get-Content crates/reratui-runtime/Cargo.toml) -replace "reratui-core = \{ version = `"$OLD_VERSION`"", "reratui-core = { version = `"$NEW_VERSION`"" | Set-Content crates/reratui-runtime/Cargo.toml
(Get-Content crates/reratui-runtime/Cargo.toml) -replace "reratui-hooks = \{ version = `"$OLD_VERSION`"", "reratui-hooks = { version = `"$NEW_VERSION`"" | Set-Content crates/reratui-runtime/Cargo.toml
(Get-Content crates/reratui-runtime/Cargo.toml) -replace "reratui-panic = \{ version = `"$OLD_VERSION`"", "reratui-panic = { version = `"$NEW_VERSION`"" | Set-Content crates/reratui-runtime/Cargo.toml
(Get-Content crates/reratui/Cargo.toml) -replace "reratui-core = \{ version = `"$OLD_VERSION`"", "reratui-core = { version = `"$NEW_VERSION`"" | Set-Content crates/reratui/Cargo.toml
(Get-Content crates/reratui/Cargo.toml) -replace "reratui-hooks = \{ version = `"$OLD_VERSION`"", "reratui-hooks = { version = `"$NEW_VERSION`"" | Set-Content crates/reratui/Cargo.toml
(Get-Content crates/reratui/Cargo.toml) -replace "reratui-macro = \{ version = `"$OLD_VERSION`"", "reratui-macro = { version = `"$NEW_VERSION`"" | Set-Content crates/reratui/Cargo.toml
(Get-Content crates/reratui/Cargo.toml) -replace "reratui-ratatui = \{ version = `"$OLD_VERSION`"", "reratui-ratatui = { version = `"$NEW_VERSION`"" | Set-Content crates/reratui/Cargo.toml
(Get-Content crates/reratui/Cargo.toml) -replace "reratui-runtime = \{ version = `"$OLD_VERSION`"", "reratui-runtime = { version = `"$NEW_VERSION`"" | Set-Content crates/reratui/Cargo.toml

# Update workspace version
(Get-Content Cargo.toml) -replace "version = `"$OLD_VERSION`"", "version = `"$NEW_VERSION`"" | Set-Content Cargo.toml

# Verify build
cargo build --workspace --lib

Write-Host "Version bumped from $OLD_VERSION to $NEW_VERSION"
Write-Host "Run 'cargo test --workspace' to verify everything works"
```

## Notes

- Always test before publishing: `cargo test --workspace`
- Use `--dry-run` flag to test publishing without actually publishing
- Wait a few seconds between publishes for crates.io to index
- Keep versions in sync across all published crates for consistency

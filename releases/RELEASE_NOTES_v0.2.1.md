# Reratui v0.2.1 - Enhanced Panic Handling & Terminal Restoration

A focused release improving panic handling and terminal restoration for TUI applications.

## 🎉 What's New

### ✨ Improved Panic Handler

Enhanced panic handling with proper terminal restoration and process cleanup:

#### Terminal State Restoration

- **Raw mode disabled** - Terminal returns to normal input mode
- **Alternate screen exited** - Panic output visible in main terminal buffer
- **Mouse capture disabled** - Text selection works after panic
- **Process exit** - Clean termination with exit code 1

#### Before v0.2.1

```
❌ Panic occurs → Terminal stuck in raw mode → Can't select text → Process hangs
```

#### After v0.2.1

```
✅ Panic occurs → Terminal fully restored → Text selectable → Process exits cleanly
```

### 🔧 Panic Handler Features

The panic handler now ensures:

1. **Terminal Restoration**: Automatically restores terminal to normal mode
2. **Text Selection**: Mouse and keyboard selection work after panic
3. **Clean Exit**: Process terminates with proper exit code
4. **Panic Formatting**: Preserves better_panic (debug) and human_panic (release) output

### 📝 Usage

The panic handler is automatically initialized by the runtime:

```rust
use reratui::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Panic handler is set up automatically by render()
    render(|| MyApp::new().into()).await?;
    Ok(())
}
```

Manual setup (if needed):

```rust
use reratui_panic::setup_panic_handler;

fn main() {
    setup_panic_handler();
    // Your application code
}
```

### 📚 Examples

Three new examples demonstrate logging integration:

#### Basic Usage (No Logging)

```rust
// examples/basic.rs
fn main() {
    reratui_panic::setup_panic_handler();
    println!("Application started");
    // Panic handler configured, no logging
}
```

#### Console Logging

```rust
// examples/with_logging.rs
use tracing_subscriber::{filter::LevelFilter, fmt};

fn main() {
    // Set up logging first
    tracing_subscriber::fmt()
        .with_env_filter(LevelFilter::INFO)
        .init();

    // Then set up panic handler
    reratui_panic::setup_panic_handler();
}
```

#### File Logging

```rust
// examples/with_file_logging.rs
use tracing_appender::rolling::daily;

fn main() {
    let file_appender = daily("logs", "application.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .json()
        .init();

    reratui_panic::setup_panic_handler();
}
```

## 🔄 Breaking Changes

### Removed Built-in Logging

**Before (v0.2.0)**:

```rust
// Logging was automatically set up
setup_panic_handler(); // Also initialized tracing
```

**After (v0.2.1)**:

```rust
// Set up your own logging first
tracing_subscriber::fmt().init();
// Then set up panic handler
setup_panic_handler(); // Only handles panics
```

**Migration**: If you relied on automatic logging, set up `tracing-subscriber` before calling `setup_panic_handler()`. See examples for patterns.

### Dependency Changes

**`reratui-panic` crate**:

- ❌ Removed: `tracing`, `tracing-subscriber`, `tracing-appender` from dependencies
- ✅ Added: Same packages as **dev-dependencies** (for examples only)
- ✅ Kept: `better-panic`, `human-panic`, `crossterm`, `tokio`

**Impact**: Smaller dependency tree, faster compile times, more control over logging setup.

## 🐛 Bug Fixes

- **Fixed**: Terminal stuck in raw mode after panic
- **Fixed**: Unable to select panic output text
- **Fixed**: Process hanging after panic instead of exiting
- **Fixed**: Mouse capture not disabled after panic

## ✅ Testing

Added comprehensive test coverage for panic handling:

### New Tests

- `test_terminal_state_restoration_on_panic` - Verifies raw mode, alternate screen, and mouse capture restoration
- `test_panic_hook_calls_terminal_restoration` - Tests panic hook triggers terminal cleanup
- `test_terminal_restoration_with_catch_panic` - Validates `catch_panic` works with panic handler
- `test_spawn_catch_panic_terminal_restoration` - Async panic handling tests
- `test_panic_hook_wrapping` - Ensures panic messages are preserved
- `test_multiple_panic_handler_setups` - Verifies idempotent setup

**Test Results**: All 24 tests passing ✅

## 📦 Crate Changes

### `reratui-panic` v0.2.0

**Removed**:

- Built-in tracing subscriber initialization
- Automatic file logging
- `WorkerGuard` static variable
- Tracing dependencies from main dependencies

**Added**:

- Process exit after panic (`std::process::exit(1)`)
- Mouse capture disable in panic hook
- Three example programs (basic, with_logging, with_file_logging)
- Comprehensive terminal restoration tests

**Improved**:

- Terminal restoration now double-checks all state
- Panic hook wraps better_panic/human_panic correctly
- Documentation clarifies logging is user-controlled

## 🎯 Design Philosophy

This release follows the principle of **separation of concerns**:

- **Panic Handler**: Restores terminal, formats panic, exits process
- **Logging**: User's responsibility, full control over configuration
- **Examples**: Show common patterns, not forced defaults

## 📖 Documentation

- Added panic handler usage guide
- Created logging integration examples
- Updated API documentation
- Added terminal restoration test documentation

## 🔗 Links

- **Documentation**: https://docs.rs/reratui/0.2.1
- **Repository**: https://github.com/sabry-awad97/reratui
- **crates.io**: https://crates.io/crates/reratui
- **Examples**: https://github.com/sabry-awad97/reratui/tree/main/examples

## 📝 Full Changelog

### Added

- Process exit (`std::process::exit(1)`) after panic handling
- Mouse capture disable in panic hook
- Three example programs for panic handler usage
- Comprehensive terminal restoration tests
- Documentation for logging integration patterns

### Changed

- Panic handler no longer initializes logging (user responsibility)
- Tracing dependencies moved to dev-dependencies
- Terminal restoration now double-checks all state

### Removed

- Built-in tracing subscriber initialization
- Automatic file logging setup
- `WorkerGuard` static variable
- Tracing dependencies from main dependencies list

### Fixed

- Terminal stuck in raw mode after panic
- Unable to select panic output text
- Process hanging after panic
- Mouse capture not disabled after panic

## 🙏 Acknowledgments

Thanks to the community for reporting terminal restoration issues and helping test the fixes!

---

**Full Diff**: https://github.com/sabry-awad97/reratui/compare/v0.2.0...v0.2.1

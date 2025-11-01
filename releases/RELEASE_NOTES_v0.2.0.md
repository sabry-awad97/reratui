# Reratui v0.2.0 - Form Management & Watch Hooks

We're excited to announce **Reratui v0.2.0**, a major update that brings powerful form management capabilities and reactive field watching to your TUI applications!

## 🎉 What's New

### ✨ Form Management System

A complete React Hook Form inspired solution for building forms in TUI applications:

#### `use_form` Hook
```rust
use reratui::prelude::*;

#[component]
fn LoginForm() -> Element {
    let form = use_form(
        FormConfig::builder()
            .field("email", "")
            .field("password", "")
            .validator("email", Validator::required("Email is required"))
            .validator("email", Validator::email("Invalid email format"))
            .validator("password", Validator::min_length(8, "Min 8 characters"))
            .on_submit(|values| {
                println!("Login with: {:?}", values);
            })
            .build()
    );
    
    // Form automatically provides context to children
    rsx! { <Form form={form}> /* fields */ </Form> }
}
```

#### Built-in Validators
- ✅ `Validator::required()` - Required field validation
- ✅ `Validator::email()` - Email format validation
- ✅ `Validator::min_length()` - Minimum length validation
- ✅ `Validator::max_length()` - Maximum length validation
- ✅ `Validator::pattern()` - Regex pattern validation
- ✅ `Validator::custom()` - Custom validation logic

#### Form Context API
```rust
#[component]
fn FormField() -> Element {
    // Access form from any child component - no prop drilling!
    let form = use_form_context();
    let registration = form.register("email");
    
    rsx! { /* use registration */ }
}
```

### 🔍 Watch Hooks

React Hook Form's `useWatch` pattern for reactive field watching:

#### Watch Single Field
```rust
let email = use_watch(&form, "email");
// Re-renders automatically when email changes
```

#### Watch Multiple Fields
```rust
let values = use_watch_multiple(&form, &["email", "username"]);
// Re-renders when any watched field changes
```

#### Watch All Fields
```rust
let all_values = use_watch_all(&form);
// Perfect for form debuggers
```

#### Watch with Callback
```rust
use_watch_callback(&form, "email", |value| {
    println!("Email changed to: {}", value);
});
```

### 🆔 Unique ID Generation

Generate stable, unique IDs for your components:

```rust
#[component]
fn MyComponent() -> Element {
    let id = use_id(); // Generates UUID v7
    let field_id = use_id_with_prefix("field"); // "field_550e8400-..."
    
    rsx! { /* use IDs */ }
}
```

**Features:**
- UUID v7 based (time-sortable)
- Stable across re-renders
- Built with `use_state` for reactivity
- Prefix support for namespacing

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
reratui = "0.2.0"
```

Or use cargo:

```bash
cargo add reratui@0.2.0
```

## 🔧 Breaking Changes

None! This is a fully backward-compatible release.

## 📚 Examples

Check out the new `form_example` in the repository:

```bash
cargo run --example form_example
```

Features demonstrated:
- Complete form with validation
- Field focus management (Tab/Shift+Tab)
- Real-time validation feedback
- Error messages and touched state
- Form submission handling

## 🐛 Bug Fixes

- Fixed doctests to use `ignore` for framework-dependent examples
- Improved error messages in form validation
- Better type inference for form field values

## 📖 Documentation

- Added comprehensive form management guide
- Updated API documentation with examples
- Added `PUBLISHING.md` for maintainers

## 🙏 Acknowledgments

This release was inspired by:
- [React Hook Form](https://react-hook-form.com/) - Form management patterns
- [shadcn/ui](https://ui.shadcn.com/) - Component composition patterns
- The Rust TUI community for feedback and support

## 🔗 Links

- **Documentation**: https://docs.rs/reratui/0.2.0
- **Repository**: https://github.com/sabry-awad97/reratui
- **crates.io**: https://crates.io/crates/reratui
- **Examples**: https://github.com/sabry-awad97/reratui/tree/main/examples

## 📝 Full Changelog

### Added
- Form management system with `use_form` hook
- `use_form_context` for accessing forms from child components
- `FormConfig::builder()` fluent API for form configuration
- Built-in validators (required, email, min_length, max_length, pattern, custom)
- `use_watch` hook for watching single field values
- `use_watch_multiple` hook for watching multiple fields
- `use_watch_all` hook for watching all form values
- `use_watch_callback` hook for field change callbacks
- `use_id` hook for generating unique IDs with UUID v7
- `use_id_with_prefix` helper for namespaced IDs
- Form example demonstrating all features
- `PUBLISHING.md` guide for maintainers

### Changed
- Updated all crate versions to 0.2.0
- Improved doctest examples to use `ignore` where appropriate
- Each crate now has explicit version instead of workspace version

### Fixed
- Doctest compilation issues
- Form state reactivity edge cases

## 🚀 What's Next

We're working on:
- Router system for multi-page TUI apps
- Chart components for data visualization
- Icon library integration
- More form components (select, checkbox, radio, etc.)

Stay tuned for v0.3.0!

---

**Full Diff**: https://github.com/sabry-awad97/reratui/compare/v0.1.0...v0.2.0

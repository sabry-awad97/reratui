# Contributing to Reratui

Thank you for your interest in contributing to Reratui! This document provides guidelines and instructions for contributing.

## 🎯 Code of Conduct

Be respectful, inclusive, and professional in all interactions.

## 🏗️ Architecture

Reratui follows Domain-Driven Design (DDD) and SOLID principles:

```
/domain/         → Business logic, entities, value objects
/application/    → Use cases, services, orchestration
/infrastructure/ → External integrations (DB, APIs, terminal)
```

## 📝 Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/sabry-awad97/reratui.git
   cd reratui
   ```

2. **Build the workspace**
   ```bash
   cargo build --all
   ```

3. **Run tests**
   ```bash
   cargo test --all
   ```

4. **Run examples**
   ```bash
   cargo run --example counter
   ```

## 🔧 Making Changes

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Follow coding standards**
   - Use `rustfmt` for formatting: `cargo fmt`
   - Use `clippy` for linting: `cargo clippy --all`
   - Write tests for new functionality
   - Add documentation for public APIs

3. **Commit your changes**
   ```bash
   git commit -m "feat: add new feature"
   ```

   Use conventional commits:
   - `feat:` - New features
   - `fix:` - Bug fixes
   - `docs:` - Documentation changes
   - `refactor:` - Code refactoring
   - `test:` - Test additions/changes
   - `chore:` - Maintenance tasks

4. **Push and create a pull request**
   ```bash
   git push origin feature/your-feature-name
   ```

## 🧪 Testing

- Write unit tests in the same file as the code
- Write integration tests in the `tests/` directory
- Ensure all tests pass before submitting PR

## 📚 Documentation

- Add rustdoc comments for all public APIs
- Update README.md if adding new features
- Add examples for significant features

## 🎨 Code Style

- Follow Rust naming conventions
- Keep functions small and focused
- Use meaningful variable names
- Prefer composition over inheritance
- Use dependency injection via traits

## 🐛 Reporting Bugs

Open an issue with:
- Clear description of the bug
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version)

## 💡 Feature Requests

Open an issue with:
- Clear description of the feature
- Use cases and benefits
- Proposed API design (if applicable)

## 📄 License

By contributing, you agree that your contributions will be licensed under MIT OR Apache-2.0.

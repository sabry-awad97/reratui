# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-16

### Breaking Changes

This release removes all `_v2` and `V2` suffixes from the public API. The fiber-based architecture introduced in v0.2.x is now the standard API.

#### Renamed Types

| Old Name (v0.2.x)     | New Name (v1.0.0)   |
| --------------------- | ------------------- |
| `StateSetterV2<T>`    | `StateSetter<T>`    |
| `CallbackV2<F>`       | `Callback<F>`       |
| `DispatchV2<A>`       | `Dispatch<A>`       |
| `RefV2<T>`            | `Ref<T>`            |
| `EffectEventV2<F>`    | `EffectEvent<F>`    |
| `QueryResultV2<T,E>`  | `QueryResult<T,E>`  |
| `MutationHandleV2`    | `MutationHandle`    |
| `FutureHandleV2`      | `FutureHandle`      |
| `FormHandleV2`        | `FormHandle`        |
| `FormStateV2`         | `FormState`         |
| `FormConfigV2`        | `FormConfig`        |
| `FormConfigBuilderV2` | `FormConfigBuilder` |
| `FieldRegistrationV2` | `FieldRegistration` |
| `ValidatorV2`         | `Validator`         |

#### Renamed Examples

| Old Name           | New Name        |
| ------------------ | --------------- |
| `counter_v2`       | `counter_fiber` |
| `effect_timing_v2` | `effect_timing` |

### Migration

To migrate from v0.2.x:

1. Update your `Cargo.toml` to use version `1.0.0`
2. Find and replace all `V2` suffixed types with their new names
3. Update any example references

See [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) for detailed migration instructions.

## [0.2.1] - Previous Release

- Initial fiber-based architecture with `_v2` suffixed APIs
- React-inspired hooks system
- Async support with queries and mutations
- Comprehensive event handling

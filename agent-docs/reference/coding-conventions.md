# Coding Conventions

## Language & Edition

- Rust edition 2021
- Workspace resolver version 2

## Formatting & Style

- **Formatter**: `cargo fmt` (default rustfmt settings)
- **Linting**: `cargo clippy --workspace -- -D warnings`
- All warnings treated as errors in CI

## Async Patterns

- **Native async traits**: Use `impl Future<Output = T> + Send` return types rather than `#[async_trait]`
- **Rationale**: Zero-cost abstractions, no heap allocation for futures
- Example:
  ```rust
  fn genesis(&self) -> impl Future<Output = Self::Block> + Send {
      async move { /* ... */ }
  }
  ```

## Thread Safety

- All public traits require `Send + Sync + 'static`
- Cross-thread state uses `Arc<AtomicU64>` / `Arc<AtomicBool>` with `Ordering::SeqCst`
- No `Mutex` for hot-path state — atomics preferred

## Error Handling

- Use `thiserror` for error enums
- Include transparent `Other(#[from] Box<dyn Error + Send + Sync>)` variant for boxed error propagation
- Never use `unwrap()` in library code (test code may use it)

## Type Safety

- Never suppress type errors with `as any`, `@ts-ignore`, or `#[allow(unused)]` on real code
- Trait bounds should be explicit and documented

## Module Organization

- One trait per file (block.rs, app.rs, event.rs, engine.rs)
- Re-export public API from lib.rs
- Mock/test utilities behind `#[cfg(any(test, feature = "mock"))]`

## Testing

- **TDD approach**: Write tests first (Red), implement (Green), refactor
- **Test runner**: `cargo nextest run` (parallel execution)
- **Async tests**: Use `#[tokio::test]` for async tests
- **Deterministic runtime**: Use `commonware_runtime::deterministic::Runner` for reproducible async tests
- **Naming**: `test_<behavior>_<expected_outcome>` (e.g., `test_verify_wrong_height_fails`)

## Dependencies

- Prefer existing workspace/vendor dependencies over adding new ones
- Vendor crates referenced via path dependencies: `path = "../../vendor/commonware/<crate>"`
- Keep consensus crate dependency-light (only `thiserror` + `tokio`)

## Vendor Code

- `vendor/` is reference-only — never modify
- Always consult `vendor/<project>/agent-docs/index.md` before reading vendor source
- Use vendor examples as implementation templates (e.g., `vendor/commonware/examples/log/`)

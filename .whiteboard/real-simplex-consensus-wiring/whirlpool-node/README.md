# whirlpool-node — Contract Document

## Purpose

Update the production node binary to pass the required runtime context, signer, and validator set to the real consensus engine.

## Public Interface Changes

None — binary entrypoint only.

## Internal Changes

### `main.rs` — Update engine construction

**Current** (Grounded):
```rust
let engine = CommonwareEngine::new(app, sink, engine_config, network_provider);
let running = engine.start().expect("Failed to start engine");
```

**Proposed**:
```rust
let engine = CommonwareEngine::new(app, sink, engine_config, network_provider, context);
let running = engine.start().expect("Failed to start engine");
```

Additional changes:
- Pass `context` (from `tokio::Runner.start(|context| ...)`) to engine constructor
- Add signer/validators to `CommonwareConfig` construction
- Use `oracle_handle` (currently `_oracle_handle`, unused) for blocker creation
- Thread oracle handle to engine or config as needed

### Config changes

- Remove `_` prefix from `oracle_handle` in main.rs
- Add validators list (single validator = own public key) to engine config

## Dependencies

No new crate dependencies. Uses existing vendor types already in scope.

## Risks

- Runtime context ownership: context is consumed by `Runner.start()` closure; must be cloned or shared with both network builder and engine
- Oracle handle must remain valid for engine lifetime

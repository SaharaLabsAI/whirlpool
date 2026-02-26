# Task 3 Learnings: MultiplexSender Implementation

## Pattern: Arc<HashMap<>> for Shared Immutable Routing

When implementing a multiplex sender that routes to multiple per-channel senders:
- Wrap `HashMap<Channel, T>` in `Arc` to enable cheap cloning across threads
- The HashMap is immutable at initialization; routing doesn't modify it
- Arc allows multiple MultiplexSender clones to share the same routing table without allocation

```rust
pub struct MultiplexSender<S> {
    senders: Arc<HashMap<Channel, CommonwareSender<S>>>,
}

impl<S> Clone for MultiplexSender<S> {
    fn clone(&self) -> Self {
        Self {
            senders: Arc::clone(&self.senders),
        }
    }
}
```

Or use `#[derive(Clone)]` which auto-implements Clone via Arc's Clone impl.

## Generic Trait Bounds for Sender Types

When wrapping a generic sender type that must satisfy trait bounds:
- Add bounds to `impl` block, not struct definition
- Required bounds for network senders:
  - `S: commonware_p2p::Sender + Clone + Send + Sync + 'static` (the sender type)
  - `S::PublicKey: Clone + Eq + Hash + Debug + Send + Sync + 'static` (for HashMap key)

```rust
impl<S> NetworkSender for MultiplexSender<S>
where
    S: commonware_p2p::Sender + Clone + Send + Sync + 'static,
    S::PublicKey: Clone + Eq + Hash + Debug + Send + Sync + 'static,
{
    // implementation
}
```

## Unit Testing with Concrete Generic Types

When testing generic structs where the real usage requires complex trait implementations:
- Use a simple concrete type like `String` that satisfies the necessary bounds
- For MultiplexSender, String satisfies `Clone` bound, allowing the struct to instantiate
- This lets you test struct behavior (instantiation, cloning) without implementing full integration

**Why not test `send()` with String?**
- The `send()` implementation calls `CommonwareSender::send()`, which expects a real `Sender` type
- Testing that requires mocking the entire Sender trait or using integration tests
- Unit tests focus on what's feasible: struct construction and trait implementation correctness

## Error Mapping Pattern

Use helper functions to map domain errors:
```rust
fn map_send_error(e: S::Error) -> P2pError {
    P2pError::Send(e.to_string())
}
```

For channel routing errors, return directly:
```rust
let sender = self.senders.get(&channel)
    .ok_or_else(|| P2pError::InvalidChannel(channel.0))?;
```

## Test Organization

When a crate has multiple structs with tests:
- Group tests by struct (CommonwarePeerId, Error helpers, MultiplexSender, MultiplexReceiver)
- Use clear test names: `test_<struct>_<behavior>`
- In TDD RED phase, use `panic!("not yet implemented - RED phase")` for unstarted tests
- This allows cargo test to run the full suite while being explicit about what's TODO

## Diagnostic Insights

1. **Clone trait implementation**: `#[derive(Clone)]` works when the wrapped type (Arc) is Clone
2. **Discriminant errors**: `std::mem::discriminant` is for enums only; don't use on struct types
3. **Arc<HashMap> cloning**: Both clones point to same Arc allocation; they are cheap aliases, not deep copies

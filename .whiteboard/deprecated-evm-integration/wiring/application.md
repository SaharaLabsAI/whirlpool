# Wiring: Application

## Wiring matrix

| Capability | Owning crate | Upstream deps | Downstream consumers | Public types | Trait interface | Default provider | Evidence |
|---|---|---|---|---|---|---|---|
| Genesis block | `app` [PROPOSED] | — | `consensus-simplex`, `whirlpool-node` | `Application::Block` | `Application::genesis()` | `EmptyBlockApp` (current) | `crates/consensus/src/app.rs::ConsensusApp::genesis` |
| Block proposal | `app` [PROPOSED] | tx source (BLOCKER) | `consensus-simplex` | `(Block, ExecutionResult)` | `Application::propose()` | `EmptyBlockApp` (current) | `crates/consensus/src/app.rs::ConsensusApp::propose` |
| Block verification | `app` [PROPOSED] | state provider | `consensus-simplex` | `ExecutionResult` | `Application::verify()` | `EmptyBlockApp` (current) | `crates/consensus/src/app.rs::ConsensusApp::verify` |
| ConsensusApp bridging | `app` [PROPOSED] | `consensus` | `consensus-simplex` | `ApplicationAdapter<A>` [PROPOSED] | `impl ConsensusApp for ApplicationAdapter<A: Application>` | — | Pattern: `crates/consensus-simplex/src/lib.rs::AppAdapter` |

## Provider swap points

### Application trait → ConsensusApp bridge [PROPOSED]

The `app` crate provides an adapter that wraps any `Application` impl and presents it as a `ConsensusApp`:

```rust
/// [PROPOSED] Bridges Application to ConsensusApp
pub struct ApplicationAdapter<A: Application> {
    inner: A,
}

impl<A: Application> ConsensusApp for ApplicationAdapter<A>
where
    A::Block: consensus::Block,
{
    type Block = A::Block;

    fn genesis(&self) -> impl Future<Output = Self::Block> + Send {
        self.inner.genesis()
    }

    fn propose(&self, parent: &Self::Block, height: u64) -> impl Future<Output = Option<Self::Block>> + Send {
        async move {
            self.inner.propose(parent, height).await.ok().map(|(block, _)| block)
        }
    }

    fn verify(&self, parent: &Self::Block, block: &Self::Block) -> impl Future<Output = Result<(), ConsensusError>> + Send {
        async move {
            self.inner.verify(parent, block).await
                .map(|_| ())
                .map_err(|e| ConsensusError::Verification(e.to_string()))
        }
    }
}
```

**Rationale**: This adapter discards execution results when bridging to `ConsensusApp` (which doesn't know about execution). The execution results are captured separately by the node for state persistence.

## Blockers

- **Transaction source for propose()**: `Application::propose()` needs transactions. Options:
  1. `propose()` takes `Vec<Transaction>` parameter — requires changing `ConsensusApp` upstream
  2. `Application` holds a `TxPool` reference — couples to pool impl
  3. `Application::propose()` pulls from an internal source (tx pool injected at construction)
  - **Recommendation**: Option 3 — inject tx source at `Application` construction time. Keeps trait clean.

# Flow: Node Startup Wiring

## Trigger
`whirlpool-node` binary starts up (`main.rs`).

## Current wiring (grounded)

**Grounded**: `crates/whirlpool-node/src/main.rs`

```pseudo
fn main():
    Runner::start(async {
        app = EmptyBlockApp
        sink = FinalizationSink::new()
        signer = ed25519::from_seed(VALIDATOR_SEED)
        network = CommonwareNetworkProviderBuilder::new(BIND_ADDR, BIND_ADDR, MAX_MSG_SIZE)
        config = CommonwareConfig { namespace, timeouts, ... }
        engine = CommonwareEngine::new(app, sink, config, network)
        engine.start()
    })
```

## Proposed wiring [PROPOSED]

```pseudo
fn main():
    Runner::start(async {
        // 1. Chain configuration
        chain_spec = Arc::new(build_sahara_chain_spec())

        // 2. State database
        // <!-- continuation round 2: B-002 resolved -->
        state_db = InMemoryStateDb::with_genesis(chain_spec.genesis_alloc())
        // 3. EVM configuration
        evm_config = WhirlpoolEvmConfig::new(chain_spec)

        // 4. EVM application — state wrapped in Arc<RwLock> for shared access
        evm_app = EvmApplication::new(evm_config, Arc::new(RwLock::new(state_db.clone())))

        // 5. Consensus bridge
        app = ApplicationAdapter::new(evm_app)

        // 6. Event sink (handles finalization → state commitment)
        // Uses Arc<RwLock<InMemoryStateDb>> — same instance as EvmApplication
        sink = EvmFinalizationSink::new(evm_app.state_db())  // shares Arc clone

        // 7. Networking + consensus (unchanged)
        signer = ed25519::from_seed(VALIDATOR_SEED)
        network = CommonwareNetworkProviderBuilder::new(BIND_ADDR, BIND_ADDR, MAX_MSG_SIZE)
        config = CommonwareConfig { namespace, timeouts, ... }
        engine = CommonwareEngine::new(app, sink, config, network)
        engine.start()
    })
```

## Key changes from current

| Component | Current | Proposed |
|---|---|---|
| App type | `EmptyBlockApp` | `ApplicationAdapter<EvmApplication<DB>>` |
| Block type | `EmptyBlock` | `EvmBlock` |
| State database | None (stateless) | `InMemoryStateDb` from `state` crate — initialized via `with_genesis()` |
| Chain config | None | `Arc<ChainSpec>` via reth-chainspec |
| EVM executor | None | `WhirlpoolEvmConfig` + `EthBlockExecutorFactory` |
| Event sink | `FinalizationSink` (height counter) | `EvmFinalizationSink` [PROPOSED] (state commitment) |

## Backwards compatibility

[PROPOSED] The node should support both modes:
- `--app=empty` → current `EmptyBlockApp` (default for testing)
- `--app=evm` → new `EvmApplication` (requires chain spec + state DB)

This can be feature-gated or config-driven.

## Ownership

| Component | Crate |
|---|---|
| Main binary wiring | `whirlpool-node` |
| `WhirlpoolEvmConfig` | `app-evm` |
| `EvmApplication` | `app-evm` |
| `ApplicationAdapter` | `app` |
| `CommonwareEngine` | `consensus-simplex` |

<!-- continuation round 2 -->
| `InMemoryStateDb` | `state` |
| `DbAccount` | `state` |
| `StateError` | `state` |

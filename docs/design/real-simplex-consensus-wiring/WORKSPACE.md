# WORKSPACE — Real Simplex Consensus Wiring

## Workspace Map

```
whirlpool/
├── crates/
│   ├── consensus/          # Core consensus traits (ConsensusEngine, ConsensusApp, EventSink)
│   ├── consensus-simplex/  # ★ Simplex BFT wiring (STUB → REAL) — primary target
│   ├── app/                # Application adapter (ConsensusApp → Application bridge)
│   ├── app-evm/            # EVM executor (propose/verify with reth) — already implemented
│   ├── p2p/                # P2P trait layer (NetworkProvider, Channel)
│   ├── p2p-commonware/     # ★ Commonware P2P provider — expose channel pairs
│   ├── state/              # In-memory state database
│   ├── whirlpool-node/     # ★ Production binary — pass runtime context
│   └── whirlpool-node-simple/ # Dev binary (EmptyBlockApp, same stub)
├── vendor/
│   └── commonware/         # Vendored commonware framework
│       ├── consensus/      #   simplex::Engine, simplex::Config
│       ├── runtime/        #   tokio::Runner, Spawner/Clock/Metrics
│       ├── p2p/            #   discovery::Network, Blocker trait, Oracle
│       ├── parallel/       #   Sequential (Strategy trait)
│       └── cryptography/   #   ed25519, Digest
└── docs/design/
    └── real-simplex-consensus-wiring/  # This design
```

## Crate Dependency Graph (in-scope)

```
whirlpool-node ──→ consensus-simplex ──→ consensus (traits)
     │                    │                   │
     │                    ├──→ p2p (traits)   │
     │                    │                   │
     │                    ├──→ vendor/commonware-consensus (simplex::Engine)
     │                    ├──→ vendor/commonware-runtime (Spawner, Clock)
     │                    └──→ vendor/commonware-cryptography (ed25519, Digest)
     │
     ├──→ p2p-commonware ──→ p2p (traits)
     │         └──→ vendor/commonware-p2p (discovery::Network, Oracle)
     │
     ├──→ app ──→ consensus (traits)
     │     └──→ app-evm ──→ vendor/reth-* (EVM execution)
     │
     └──→ state (InMemoryStateDb)
```

## Build Entry Points

- `cargo build -p whirlpool-node` — builds the production binary
- `cargo test -p consensus-simplex` — tests the engine wiring (primary test target)
- `cargo test --workspace` — full workspace test suite

## Read Entry Points

- `crates/consensus-simplex/src/engine.rs` — **START HERE** — the stub to replace
- `crates/p2p-commonware/src/provider.rs` — P2P channel registration/splitting
- `crates/whirlpool-node/src/main.rs` — node binary wiring

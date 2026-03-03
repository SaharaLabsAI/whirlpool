# CRATES — Real Simplex Consensus Wiring

| Crate | Path | Purpose | Change Scope |
|-------|------|---------|-------------|
| consensus-simplex | `crates/consensus-simplex` | Simplex BFT engine wiring — connects ConsensusApp/EventSink to vendor simplex engine | **Major**: Replace stub `start()` with real engine wiring |
| p2p-commonware | `crates/p2p-commonware` | Commonware-based P2P network provider | **Moderate**: Expose per-channel (Sender, Receiver) pairs for simplex engine |
| whirlpool-node | `crates/whirlpool-node` | Production node binary | **Minor**: Pass runtime context, blocker, validators to engine |

### Adjacent Crates (read-only, no changes)

| Crate | Path | Relevance |
|-------|------|-----------|
| consensus | `crates/consensus` | Defines `ConsensusEngine`, `ConsensusApp`, `EventSink`, `RunningEngine` traits |
| app | `crates/app` | `ApplicationAdapter` wraps `EvmApplication` to impl `ConsensusApp` |
| app-evm | `crates/app-evm` | `EvmApplication` — real EVM propose/verify via reth (already implemented) |
| p2p | `crates/p2p` | Vendor-agnostic P2P traits (`NetworkProvider`, `Channel`) |
| state | `crates/state` | `InMemoryStateDb` for EVM state |

# Phase 02 Step 2 — Structural Test Coverage

## Scope
- depth: structural (all interface-facing symbols + dependents)
- crates exercised: `app`, `consensus`, `p2p`, `state`, `consensus-simplex`, `p2p-commonware`, `app-evm`
- symbols in scope: `Application`, `TxSource`, `NoopTxSource`, `InMemoryTxPool`, `ConsensusApp`, `Block`, `EventSink`, `ConsensusEngine`, `PeerId`, `NetworkSender`, `NetworkReceiver`, `NetworkProvider`, `StateDb`, `CommonwareBlock`, `CommonwareTransport`, `StateProvider`

## Tests Run
| Crate | Command | Outcome | Doc tests |
| --- | --- | --- | --- |
| `app` | `nix develop --command cargo test -p app` | ✅ passed (all unit + doc tests) | `0` (none defined) |
| `consensus` | `nix develop --command cargo test -p consensus --features mock` | ✅ passed (7 unit tests + 0 doc tests) | `0` |
| `p2p` | `nix develop --command cargo test -p p2p` | ✅ passed (5 unit tests) | `1` ignored |
| `state` | `nix develop --command cargo test -p state` | ✅ passed (19 unit tests) | `0` |
| `consensus-simplex` | `nix develop --command cargo test -p consensus-simplex` | ✅ passed (22 units + 4 ignored, 1 doctest ignored) | `1` ignored |
| `p2p-commonware` | `nix develop --command bash -lc 'CARGO_BUILD_JOBS=1 cargo test -p p2p-commonware'` | ✅ passed after retry; initial run triggered rustc ICE/resource exhaustion | `3` ignored |
| `app-evm` | `nix develop --command cargo test -p app-evm` | ✅ passed (all unit and integration suites) | `0` |

## Failure / Retry Notes
- `p2p-commonware` initially hit `rustc` ICE (`spawn work thread: Resource temporarily unavailable`). Re-running the suite with `CARGO_BUILD_JOBS=1` succeeded; no test failures remained but the warning about unused bindings persists (pre-existing). 

## Coverage Remarks for Changed Symbols
| Symbol | Import path / crate | Breakage classification | Test coverage comments |
| --- | --- | --- | --- |
| `Application` | `app::traits::Application` | trait | Exercised indirectly via `app` unit tests (`adapter`/`error`/`types`) and `app-evm` integration suites that drive proposer/validator logic. |
| `TxSource` | `app::traits::TxSource` | trait | Directly covered by `traits` unit tests (pending, drain, concurrent push) and `app-evm` tests that drive `propose`/`verify` through `InMemoryTxPool`. |
| `NoopTxSource` | `app::tx_source::NoopTxSource` (movement target) | type | `traits` unit test checks `pending()` and is indirectly exercised when adapters skip txs. |
| `InMemoryTxPool` | `app::tx_source::InMemoryTxPool` | type | Unit tests (fifo order, concurrency) + `app-evm` integration `propose` coverage. |
| `ConsensusApp` | `consensus::traits::ConsensusApp` | trait | `consensus` mock feature tests exercise `ConsensusApp` bounds; `app-evm` and `consensus-simplex` integration suites run through adapters implementing this trait. |
| `Block` | `consensus::traits::Block` | trait/type | `consensus` unit tests and `consensus-simplex` block bindings (blanket impl) ensure trait methods stay valid. |
| `EventSink` | `consensus::traits::EventSink` | trait | `consensus` event tests and `consensus-simplex` sink tests capture event handling behavior. |
| `ConsensusEngine` | `consensus::traits::ConsensusEngine` | trait | `consensus-simplex` engine tests (construct/start/shutdown) exercise this trait; negates breakage by verifying engine lifecycle. |
| `PeerId` | `p2p::traits::PeerId` | trait | `p2p` mock tests plus `p2p-commonware` peer identifier tests cover conversions and equality. |
| `NetworkSender` | `p2p::traits::NetworkSender` | trait | `p2p` mock sender/receiver tests and `p2p-commonware` multiplex sender tests ensure send channels stay compatible. |
| `NetworkReceiver` | `p2p::traits::NetworkReceiver` | trait | `p2p` mock receiver tests plus `p2p-commonware` multiplex receiver merging tests. |
| `NetworkProvider` | `p2p::traits::NetworkProvider` | trait | `p2p-commonware` provider tests validate builder/start behaviors; `consensus-simplex` engine integration uses provider. |
| `StateDb` | `state::traits::StateDb` (new interface) | trait | `state` unit tests exercise DB contract; `app-evm` integration ensures state root handling remains consistent. |
| `CommonwareBlock` | `consensus-simplex::traits::CommonwareBlock` | trait | `consensus-simplex` trait tests (blanket impl, dual trait) and engine tests exercise block trait conversions. |
| `CommonwareTransport` | `p2p-commonware::traits::CommonwareTransport` | trait | `p2p-commonware` provider + helper tests exercise message transport abstractions used by `consensus-simplex`. |
| `StateProvider` | `app-evm::traits::StateProvider` | trait | `app-evm` executor/verify tests rely on `StateProvider` to fetch/commit state, fully covered by unit + integration suites. |

## Breakage Classification Summary
- **Import path**: No tests failed when calling any of the exposed trait interfaces from their crate root re-exports; all current import paths remain stable.
- **API signature / trait**: Mock and integration suites exercise every trait method mentioned above, so signature deviations would surface immediately and none observed.
- **Type-level**: Concrete helpers like `NoopTxSource`, `InMemoryTxPool`, and `StateDb` implementations continue to satisfy trait contracts (tests pass, doc tests exist where defined).
- **Uncertain**: No unresolved breakage signals remain aside from existing warnings (unused imports/fields). If future trait splits re-export differently, append another round of doc/regression tests.


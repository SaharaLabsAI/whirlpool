| Crate | Domain | Output Path | Key Exports | Ownership Hint |
|---|---|---|---|---|
| `whirlpool-node` | Block Production | `whirlpool-node/README.md` | `config::{NAMESPACE, BLOCK_INTERVAL, BIND_ADDR, VALIDATOR_SEED}`, `block::EmptyBlock`, `app::EmptyBlockApp`, `main::TestStateDb` (private to binary) | Sole owner of runtime wiring and engine startup |
| `app-evm` | EVM Execution | `app-evm/README.md` | `executor::{EvmApplication, StateProvider}`, `config::{WhirlpoolEvmConfig, build_sahara_chain_spec, SAHARA_CHAIN_ID}`, `error::EvmAppError` | Sole owner of concrete Application impl and EVM config |
| `app` | Application Layer | `app/README.md` | `traits::{Application, TxSource, NoopTxSource}`, `types::{EvmBlock, ExecutionResult}`, `adapter::ApplicationAdapter`, `error::ApplicationError` | Sole owner of consensus-to-app abstraction |
| `state` | State Management | `state/README.md` | `db::{InMemoryStateDb, DbAccount}`, `error::StateError` | Sole owner of canonical state storage |


# Architecture Flows

## Scope
- Sub-Intent B only: `REQ-4` and `REQ-5`.
- Modified crate: `crates/whirlpool-node`.
- Read-only builder boundary: `crates/p2p-commonware`.
- Source verification anchors:
  - `crates/whirlpool-node/src/config.rs`
  - `crates/whirlpool-node/src/main.rs`

## Flow 1: CLI Parsing to `NodeConfig`

```text
process start
  -> tracing initialization
  -> clap parses NodeArgs
       - --listen-addr -> SocketAddr
       - --dialable-addr -> SocketAddr
       - --bootstrap-peer -> Vec<String>
       - --dial-peer -> Vec<String>
       - --validator-seed -> u64
       - --rpc-addr -> SocketAddr
       - --data-dir -> PathBuf
       - --max-message-size -> u32
       - --network-namespace -> String
       - --consensus-namespace -> String
       - --block-interval-ms -> u64
  -> NodeConfig::from(args)
       - network.namespace = args.network_namespace.into_bytes()
       - network.listen_addr = args.listen_addr
       - network.dialable_addr = args.dialable_addr
       - network.bootstrap_peers = parse(bootstrap_peers) + parse(dial_peers)
       - network.max_message_size = args.max_message_size
       - identity.seed = args.validator_seed
       - rpc.bind_addr = args.rpc_addr
       - storage.data_dir = args.data_dir
       - consensus.namespace = args.consensus_namespace
       - consensus.block_interval = Duration::from_millis(args.block_interval_ms)
  -> validated runtime-owned NodeConfig
```

### Flow guarantees
- All startup inputs become typed before `commonware_runtime::tokio::Runner::new(...)` is called.
- `--dial-peer` and `--bootstrap-peer` converge into a single `network.bootstrap_peers` collection.
- Invalid peer entries abort startup before the async runtime starts.
- No fallback path leaves `main.rs` responsible for defaulting individual fields.

## Flow 2: Startup Wiring With Config-Owned Inputs

```text
NodeArgs::parse()
  -> NodeConfig::from(args)
  -> tokio::Config::new().with_storage_directory(config.storage.runtime_dir())
  -> tokio::Runner::new(runtime_cfg)
  -> executor.start(|context| async move { ... })
       -> signer = ed25519::PrivateKey::from_seed(config.identity.seed)
       -> validators = vec![signer.public_key()]
       -> CommonwareNetworkProviderBuilder::new(
              signer.clone(),
              config.network.namespace.clone(),
          )
          .listen_addr(config.network.listen_addr)
          .dialable_addr(config.network.dialable_addr)
          .bootstrappers(config.network.bootstrap_peers.clone())
          .max_message_size(config.network.max_message_size)
          .initial_validators(0, validators.clone())
          .build(context.with_label("network"))
          .await
       -> open state DB at config.storage.state_dir()
       -> open mempool DB at config.storage.mempool_dir()
       -> build CommonwareConfig using config.consensus.namespace
       -> set block interval from config.consensus.block_interval
       -> start RPC server at config.rpc.bind_addr
       -> keep oracle_handle alive
       -> await forever
```

### Flow guarantees
- Every startup value currently split between `config.rs` constants and `main.rs` literals moves behind `NodeConfig`.
- The Commonware builder contract remains unchanged; only the caller-side values change.
- Default startup still binds ephemeral local P2P addresses, uses seed `0`, uses `data/` storage, and binds RPC on `127.0.0.1:8545` when no flags are passed.
- The current namespace split is preserved explicitly:
  - `config.network.namespace` -> Commonware network provider
  - `config.consensus.namespace` -> `consensus_simplex::CommonwareConfig`

## Flow 3: Bootstrap Peer Parsing

```text
input: "PUBKEY@HOST:PORT"
  -> split_once('@')
       - fail if separator missing
       - fail if either side empty
  -> parse pubkey segment
       - hex decode
       - validate expected Ed25519 public-key bytes
       - construct commonware_cryptography::ed25519::PublicKey
  -> parse address segment
       - SocketAddr::from_str(HOST:PORT)
  -> return BootstrapPeer = (public_key, socket_addr)
```

### Failure branches
- Missing `@` -> configuration error
- Empty pubkey segment -> configuration error
- Empty address segment -> configuration error
- Invalid hex -> configuration error
- Invalid Ed25519 public key bytes or wrong length -> configuration error
- Invalid `HOST:PORT` -> configuration error

### Flow guarantees
- Parsing is fail-fast and deterministic.
- Successful output matches `p2p_commonware::Bootstrapper<commonware_cryptography::ed25519::PublicKey>` exactly.
- No runtime warning or best-effort skip is allowed for malformed peers.

## Flow 4: Storage Path Derivation

```text
storage.data_dir
  -> runtime_dir() = data_dir / "runtime"
  -> state_dir() = data_dir / "state"
  -> mempool_dir() = data_dir / "mempool"
```

### Flow guarantees
- One root flag, `--data-dir`, controls all persistent node paths.
- Relative and absolute roots both remain valid.
- No separate state/runtime/mempool flags are introduced in this pass.

## Traceability
- `REQ-4` -> Flow 1, Flow 3, Flow 4
- `REQ-5` -> Flow 2

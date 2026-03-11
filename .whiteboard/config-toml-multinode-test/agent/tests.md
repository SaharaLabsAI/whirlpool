# Test Contracts

## AC_VERSION: 1

## Acceptance Criteria

### AC-1: Config TOML Loading
GIVEN a valid config.toml file with node settings
WHEN the node is started with `--config <path>`
THEN all settings from the TOML file are applied to the node configuration

### AC-2: CLI Override Precedence
GIVEN a config.toml with `rpc_addr = "127.0.0.1:8545"`
WHEN the node is started with `--config <path> --rpc-addr 127.0.0.1:9999`
THEN the CLI value (9999) takes precedence over the TOML value (8545)

### AC-3: Backward Compatibility
GIVEN no --config flag is provided
WHEN the node is started with only CLI args
THEN behavior is identical to current implementation

### AC-4: Multi-Validator Config
GIVEN a config.toml with `validators = ["<hex_pk1>", "<hex_pk2>", "<hex_pk3>", "<hex_pk4>"]`
WHEN the node starts
THEN the consensus engine and P2P provider are initialized with all 4 validators

### AC-5: 4-Node P2P Connectivity
GIVEN 4 nodes started in-process with matching validator sets and bootstrap peers
WHEN nodes run for sufficient time
THEN each node logs peer connection events showing connections to the other 3 nodes
AND block height grows on all 4 nodes (proving message exchange)

### AC-6: Block Height Growth
GIVEN 4 connected nodes running consensus
WHEN observed over a bounded time window
THEN block height increases on all 4 nodes (height > 0)
AND all 4 nodes are within ±1 block height of each other (proving sync)

### AC-7: Per-Node Height and Peer Logging
GIVEN the 4-node test is running
WHEN polling each node during the verification loop
THEN each node's block height and connected peer info is logged at INFO level
AND the test output shows all 4 nodes participating

## QA Scenarios

### QA-1: Config file not found
GIVEN `--config nonexistent.toml`
THEN node exits with a clear error message

### QA-2: Invalid TOML syntax
GIVEN a malformed config.toml
THEN node exits with a parse error message

### QA-3: Partial TOML config
GIVEN a config.toml with only `rpc_addr` set
THEN all other fields use defaults

### QA-4: Empty validators list
GIVEN `validators = []` in config.toml
THEN node exits with an error (need at least 1 validator)

## Invariants

### INV-1: Config Precedence
CLI > TOML > Defaults. This ordering MUST hold for every config field.

### INV-2: Validator Set Consistency
All nodes in a network MUST have identical validator sets. Mismatched sets will cause consensus failure.

### INV-3: Test Isolation
Each test node MUST use isolated data directories and unique network ports.

# Task 1: Config TOML Loading and CLI Layering

**Complexity**: M
**Covers**: AC-1, AC-2, AC-3, QA-1, QA-2, QA-3, INV-1

## Pre-Task Gate
- [ ] `crates/whirlpool-node/src/config.rs` exists and contains `NodeArgs` struct
- [ ] `crates/whirlpool-node/Cargo.toml` is readable
- [ ] `nix develop --command cargo build -p whirlpool-node` passes

## What to Do

### Step 1: Add dependencies
Add `toml = "0.8"` and `serde = { version = "1", features = ["derive"] }` to `crates/whirlpool-node/Cargo.toml`.

### Step 2: Write behavior tests (FIRST)
In `crates/whirlpool-node/src/config.rs`, add these test functions:

```rust
// TST-01: TOML file loading
#[test]
fn test_toml_config_loading() {
    // Write a valid TOML string with known values
    // Parse via TomlConfig::from_str or load_toml_config
    // Assert all fields are populated correctly
}

// TST-02: CLI overrides TOML
#[test]
fn test_cli_overrides_toml() {
    // Create TomlConfig with listen_addr = "1.2.3.4:1000"
    // Create NodeArgs with listen_addr = "5.6.7.8:2000"
    // Merge → result should have CLI's value
}

// TST-03: No --config = backward compat
#[test]
fn test_no_config_flag_backward_compat() {
    // Create NodeArgs with no config field
    // load_config should produce same NodeConfig as From<NodeArgs>
}

// TST-08: Missing config file
#[test]
fn test_missing_config_file_error() {
    // NodeArgs with config = Some("/nonexistent.toml")
    // load_config should return descriptive error
}

// TST-09: Invalid TOML
#[test]
fn test_invalid_toml_error() {
    // Write invalid TOML content to tempfile
    // load_config should return parse error
}

// TST-10: Partial TOML + CLI merge
#[test]
fn test_partial_toml_with_cli() {
    // TOML has only listen_addr and data_dir
    // CLI has only rpc_addr
    // Merge: all 3 fields from respective sources, rest = defaults
}
```

### Step 3: Create `TomlConfig` struct
```rust
#[derive(Debug, Default, Deserialize)]
pub struct TomlConfig {
    pub listen_addr: Option<String>,
    pub dialable_addr: Option<String>,
    pub bootstrap_peers: Option<Vec<String>>,
    pub validator_seed: Option<u64>,
    pub rpc_addr: Option<String>,
    pub data_dir: Option<String>,
    pub max_message_size: Option<usize>,
    pub network_namespace: Option<String>,
    pub consensus_namespace: Option<String>,
    pub block_interval_ms: Option<u64>,
    pub validators: Option<Vec<String>>,
}
```

### Step 4: Add `--config` to NodeArgs
```rust
#[arg(long)]
pub config: Option<PathBuf>,
```

### Step 5: Implement `load_config()`
```rust
pub fn load_config(args: NodeArgs) -> Result<NodeConfig, ConfigError> {
    let toml_config = if let Some(path) = &args.config {
        let content = std::fs::read_to_string(path)?;
        toml::from_str::<TomlConfig>(&content)?
    } else {
        TomlConfig::default()
    };
    merge_config(args, toml_config)
}
```

### Step 6: Implement `merge_config()`
CLI values (if non-default/provided) override TOML values override built-in defaults. Use clap's `value_source()` or check against defaults to detect CLI-provided values.

### Step 7: Verify tests pass
```bash
nix develop --command cargo test -p whirlpool-node -- config 2>&1
```

## Post-Task Gate
- [ ] `nix develop --command cargo build -p whirlpool-node` passes
- [ ] `nix develop --command cargo test -p whirlpool-node` passes — all new + existing config tests green
- [ ] Evidence saved to `.sisyphus/evidence/task-1-config-toml-loading.txt`

## Mock Boundary
None — this task is self-contained within whirlpool-node.

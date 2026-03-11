# Task 2: Multi-Validator Configuration

**Complexity**: M
**Covers**: AC-4, QA-4, INV-2

## Pre-Task Gate
- [ ] Task 1 complete (TomlConfig, load_config, merge_config exist)
- [ ] `nix develop --command cargo test -p whirlpool-node` passes

## What to Do

### Step 1: Write behavior tests (FIRST)
In `crates/whirlpool-node/src/config.rs`:

```rust
// TST-04: Multi-validator from config
#[test]
fn test_multi_validator_from_toml() {
    // TOML with validators = ["<hex1>", "<hex2>", "<hex3>", "<hex4>"]
    // load_config → NodeConfig.validators == Some(vec![pk1, pk2, pk3, pk4])
}

// TST-11: Empty validators rejection
#[test]
fn test_empty_validators_rejected() {
    // TOML with validators = []
    // load_config should error OR main should reject
}

#[test]
fn test_no_validators_falls_back_to_signer() {
    // No validators in TOML or CLI
    // NodeConfig.validators should be None (caller provides fallback)
}
```

### Step 2: Add `--validator` to NodeArgs
```rust
#[arg(long)]
pub validator: Vec<String>,  // hex-encoded ed25519 pubkeys, repeatable
```

### Step 3: Add `validators` field to NodeConfig
```rust
pub validators: Option<Vec<ed25519::PublicKey>>,
```

### Step 4: Implement validator parsing in merge_config
Parse hex strings to `ed25519::PublicKey`. CLI `--validator` overrides TOML `validators` (if CLI provided). Both are `Vec<String>` → `Vec<PublicKey>`.

### Step 5: Update main.rs validator wiring
```rust
// Before:
let validators = vec![signer.public_key()];

// After:
let validators = config.validators
    .clone()
    .unwrap_or_else(|| vec![signer.public_key()]);
```

### Step 6: Verify
```bash
nix develop --command cargo test -p whirlpool-node 2>&1
nix develop --command cargo build -p whirlpool-node 2>&1
```

## Post-Task Gate
- [ ] `nix develop --command cargo build -p whirlpool-node` passes
- [ ] `nix develop --command cargo test -p whirlpool-node` passes — all config tests green
- [ ] main.rs uses config-driven validators (no hardcoded single validator)
- [ ] Evidence saved to `.sisyphus/evidence/task-2-multi-validator-config.txt`

## Mock Boundary
None — self-contained in whirlpool-node.

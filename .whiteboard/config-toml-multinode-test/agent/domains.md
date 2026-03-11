# Domains

## Domain: Configuration
Responsible crate: whirlpool-node
Changes: TOML config loading, CLI extension, config merge logic, multi-validator field

## Domain: Node Lifecycle
Responsible crate: whirlpool-node
Changes: Extract start_node() from main(), accept NodeConfig, return handle for graceful control

## Domain: Testing
Responsible crate: integration-tests
Changes: Multi-node test harness, block height verification, P2P connectivity proof

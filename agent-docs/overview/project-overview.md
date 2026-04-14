# Whirlpool — Project Overview

## Purpose
Whirlpool is a modular Rust workspace for consensus-driven node binaries using a layered architecture.

## Architecture
Three layers with explicit interface boundaries:
1. `consensus`: canonical trait interfaces (`consensus::traits::*`).
2. `consensus-simplex`: Commonware adapter implementation.
3. Node binaries: concrete applications and wiring.

## Interface/Implementation Direction
- Interface crates expose trait modules (`traits.rs`) as canonical boundaries.
- Implementations live in separate modules and consume canonical trait paths.
- Canonical import convention across workspace: `crate::traits::...`.

## Current Scope
- EVM path: `whirlpool-node` + `app` + `app-evm` + `state`.
- Grouped on disk under `crates/app/execute/evm/{app,rpc,state}` for EVM-specific implementations.
- Networking path: `network` interfaces + `network-commonware` implementation.

## Design Principles
- Minimal trait-first boundaries.
- Adapter isolation for vendor-specific code.
- Canonical `::traits::` imports for cross-crate wiring.

## Reth Core Pinning
- Workspace patches `reth-primitives-traits` plus companion `reth-codecs`, `reth-codecs-derive`, and `reth-zstd-compressors` to `paradigmxyz/reth-core` tag `v0.1.1` for `reth v2.0.0` compatibility.

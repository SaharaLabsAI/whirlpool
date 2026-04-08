# Design

## Purpose
Design notes explain why Whirlpool is shaped the way it is, not just what each crate exports.

## Read in this order
1. `precompiles/index.md` — one-page summary.
2. `precompiles/availability.md` — when custom precompiles exist and why no deployment step is needed.
3. `precompiles/call-model.md` — allowed call paths, rejected call paths, and stateful behavior.
4. `precompiles/wiring.md` — where the registry is attached and why the seam lives in config/factory code.

## Topics
- [Precompiles](precompiles/index.md): Whirlpool custom EVM precompile design, rationale, and execution model.

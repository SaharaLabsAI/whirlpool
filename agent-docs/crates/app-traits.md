# app-traits

## Purpose
Application-facing traits and adapters that bridge execution logic to consensus.

## Location
`crates/app/traits/`

## Owns
- `Application`: generic app lifecycle trait (`genesis`, `propose`, `verify`).
- `TxSource`: transaction source interface with `pending()` and `push(tx)`.
- `NoopTxSource` and `InMemoryTxPool` simple tx-source implementations.
- `ApplicationAdapter`: maps any `Application` block type to `consensus::traits::ConsensusApp` without depending on concrete carriers.
- `ApplicationError`: app-layer error type.

## Boundary
`app-traits` does not define or re-export concrete block primitives, receipts, execution results, or header `extra_data` helpers. Use `app-primitives` for those carrier types.

## Canonical Imports
- `app_traits::traits::{Application, TxSource}`
- `app_traits::{ApplicationAdapter, ApplicationError, InMemoryTxPool, NoopTxSource}`

## Status
Breaking pre-production split complete: the old package name `app` is removed; callers must import traits from `app_traits` and concrete carriers from `app_primitives`.

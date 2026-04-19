# consensus-manager: Trusted-Dealer Bootstrap Artifacts

## Summary
`consensus-manager` owns the v1 trusted-dealer bootstrap artifact format used to load BLS threshold material for simplex startup.

Location: `crates/consensus/manager/`

## Public API
- `run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig) -> TrustedDealerBootstrapResult`
  - Generates one session manifest plus one bundle per validator.
  - Writes files under `session-<hex>/manifest.bin` and `session-<hex>/bundles/*.bundle`.
- `load_local_bundle(LoadLocalBundleConfig) -> LocalBundleMaterial`
  - Loads manifest + local bundle.
  - Validates session completeness by checking and verifying all participant bundles.
  - Validates local recipient/session/share binding before returning polynomial + share.
  - Returned `LocalBundleMaterial` now includes both `dealers` and `participants` (ed25519 keys), plus `polynomial` + local `share`.

## Artifact Contracts
- Manifest stores `session_id` and DKG public output.
- Bundle stores `session_id`, intended `recipient`, and threshold share.
- Bundle filename key is the hex-encoded ed25519 public key (`<pubkey>.bundle`).

## Failure Model
- Rejects empty or duplicate participant sets during bootstrap.
- Rejects missing bundles, recipient mismatches, session mismatches, and invalid share/public bindings when loading.
- Designed for fail-closed startup wiring in `whirlpool-node`.

## Tests
- Bootstrap writes expected manifest/bundle set.
- Local load validates share integrity.
- Negative coverage includes wrong recipient, incomplete sessions, and tampered session IDs.

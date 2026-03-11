# Risk Assessment

## Iteration: 1

## Resolved Risks
- None yet (first iteration)

## Accepted Risks
- R1 (medium): Large provider trait surface (~20 impls). Accepted — NoopProvider reference shows pattern; most methods can start as stubs returning None/empty.
- R2 (low): Blob code interleaved in reth. Accepted — exclude at API response level, don't modify vendor.
- R3 (low): Type conversion gaps. Accepted — state-reth and app-evm already bridge most types.

## Blocker Conversions
- None

## Expansion Outcomes
- None (scope confirmed as-is)

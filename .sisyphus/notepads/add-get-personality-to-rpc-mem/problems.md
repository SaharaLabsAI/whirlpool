## 2026-03-23T09:15:12Z
- Open design-to-implementation problem remains: choose and lock the not-found response shape (`null` vs explicit object) before coding to avoid client ambiguity.
- Open mapping problem remains: pin storage-failure RPC error code/message contract in implementation tests.

## 2026-03-23T09:31:07Z
- Open implementation-level ambiguity remains for not-found wire shape and storage-failure error mapping; preserved as explicit downstream contract items in planning artifacts.

## 2026-03-23T09:37:28Z
- Existing downstream ambiguity remains for not-found response shape and storage-failure error mapping; left unchanged at gate close because they are implementation-phase concerns.

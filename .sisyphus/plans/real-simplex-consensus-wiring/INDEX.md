# real-simplex-consensus-wiring — Execution Plan

## Execution Order

### Wave 1 — Foundation (P2P & Config)
- [x] [01-p2p-per-channel](tasks/01-p2p-per-channel.md) `M`
- [x] [02-config-extend](tasks/02-config-extend.md) `S`

### Wave 2 — Engine Base
- [x] [03-engine-constructor](tasks/03-engine-constructor.md) `S`

### Wave 3 — Engine Wiring
- [ ] [04-engine-replace-stub](tasks/04-engine-replace-stub.md) `L`

### Wave 4 — Main Wiring
- [ ] [05-main-wiring](tasks/05-main-wiring.md) `M`

### Wave 5 — Verification
- [ ] [06-integration-tests](tasks/06-integration-tests.md) `M`

<!-- TASKS_START -->
1. [01-p2p-per-channel](tasks/01-p2p-per-channel.md)
2. [02-config-extend](tasks/02-config-extend.md)
3. [03-engine-constructor](tasks/03-engine-constructor.md)
4. [04-engine-replace-stub](tasks/04-engine-replace-stub.md)
5. [05-main-wiring](tasks/05-main-wiring.md)
6. [06-integration-tests](tasks/06-integration-tests.md)
<!-- TASKS_END -->

## Dependency Graph
```text
(01) P2P Channel   (02) Config Ext
      \               |
       \           (03) Constructor
        \           /
         (04) Engine Wiring
               |
         (05) Main Wiring
               |
         (06) Integration Tests
```

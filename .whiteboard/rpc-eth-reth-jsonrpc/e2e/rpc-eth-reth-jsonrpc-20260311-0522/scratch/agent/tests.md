# Test Contracts (QA Baseline)

## Protected Tests

- TST-1: WhirlpoolProvider implements all required reth storage traits and compiles against RpcModuleBuilder bounds.
- TST-2: WhirlpoolTxPool implements TransactionPool trait and correctly bridges TxSource push/pending.
- TST-3: WhirlpoolNetwork implements NetworkInfo with correct chain_id and minimal peer info.
- TST-4: RPC server starts via RpcModuleBuilder with WhirlpoolProvider/TxPool/Network and accepts HTTP connections.
- TST-5: eth_chainId returns configured chain ID through reth EthApi.
- TST-6: eth_blockNumber returns latest block from BlockStorage.
- TST-7: eth_getBalance returns account balance from StateDb.
- TST-8: eth_getBlockByNumber returns block data from BlockStorage.
- TST-9: eth_sendRawTransaction submits tx through TxSource.
- TST-10: eth_blobBaseFee returns unsupported method error.
- TST-11: Integration test mirrors reth rpc-builder test patterns (typed clients, param permutations).
- TST-12: whirlpool-node successfully starts with new RPC wiring and serves eth_ requests.

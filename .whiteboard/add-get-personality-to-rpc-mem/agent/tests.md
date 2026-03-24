# Tests (Alignment Baseline)

## TST-1: RPC get personality returns latest finalized entry
- Method: `mem_getPersonality`
- Given: storage has a `StoredPersonality` for requested `personality_id`
- Expect: response contains tx hash, block height, signer, personality_id, nonce, markdown, markdown_hash in stable encoding

## TST-2: RPC get personality returns null/not-found when absent
- Method: `mem_getPersonality`
- Given: storage has no entry for requested `personality_id`
- Expect: deterministic not-found response contract (null or explicit notFound)

## TST-3: RPC get personality rejects malformed identity hex
- Method: `mem_getPersonality`
- Given: missing `0x` prefix or invalid hex payload
- Expect: validation error surfaced through rpc-mem error mapping

## TST-4: Submit path remains functional
- Method: `mem_submitPersonality`
- Given: existing valid request fixture
- Expect: enqueue behavior and tx hash response unchanged

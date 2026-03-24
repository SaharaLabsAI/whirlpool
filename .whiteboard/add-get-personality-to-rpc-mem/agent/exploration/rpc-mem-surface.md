# Exploration: RPC-MEM Surface

## Current methods
- `mem_submitPersonality` only.

## Current service contract
- `MemoryTxService::submit_personality(request) -> Result<[u8; 32], RpcMemError>`.

## Key change requirement
- Add get/read capability while preserving submit method compatibility.

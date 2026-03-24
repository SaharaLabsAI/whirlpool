# Flows

## Flow 1: Submit personality (existing)
1. Client calls `mem_submitPersonality`.
2. rpc-mem validates and encodes transaction bytes.
3. Service pushes bytes to tx source.
4. Response returns tx hash.

## Flow 2: Get latest personality (new)
1. Client calls `mem_getPersonality` with hex `personality_id`.
2. rpc-mem decodes and validates ID bytes.
3. Service queries finalized personality storage (`get_latest`).
4. rpc-mem returns deterministic found/not-found response.

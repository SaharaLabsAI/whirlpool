#!/usr/bin/env python3

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import secrets
import shutil
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


ROOT_DIR = Path(__file__).resolve().parents[3]
DEMO_DIR = ROOT_DIR / "devtools" / "demo" / "personality"
RUN_DIR = DEMO_DIR / ".run"
PROFILES_DIR = DEMO_DIR / "profiles"
CONFIG_FILE = DEMO_DIR / "whirlpool-node-demo.toml"
NODE_LOG_FILE = RUN_DIR / "whirlpool-node.log"
NODE_PID_FILE = RUN_DIR / "node.pid"
SUBMIT_EVENTS_FILE = RUN_DIR / "save-events.jsonl"
SUBMIT_MESSAGE_FILE = RUN_DIR / "save-message.txt"
SUBMIT_RESPONSE_FILE = RUN_DIR / "submit-response.json"
SUBMIT_TX_FILE = RUN_DIR / "submit-tx.json"
SUBMIT_RECEIPT_FILE = RUN_DIR / "submit-receipt.json"
FETCH_RESPONSE_FILE = RUN_DIR / "fetch-response.json"
PERSONALITY_FILE = RUN_DIR / "personality.md"
BOOTSTRAP_FILE = RUN_DIR / "codex-bootstrap.md"
SAVE_PROMPT_FILE = RUN_DIR / "save-prompt.md"
PROFILE_NAME_FILE = RUN_DIR / "profile-name.txt"
FETCH_PROMPT_FILE = RUN_DIR / "fetch-prompt.md"
FETCH_EVENTS_FILE = RUN_DIR / "fetch-events.jsonl"
FETCH_MESSAGE_FILE = RUN_DIR / "fetch-message.txt"
CODEX_HOME_DIR = RUN_DIR / "codex-home"
PROFILE_STORE_DIR = RUN_DIR / "fetched-profiles"
PROFILE_INDEX_FILE = PROFILE_STORE_DIR / "index.json"
PROFILE_REGISTRY_FILE = PROFILE_STORE_DIR / "registry.json"
CARGO_BUILD_JOBS = os.environ.get("CARGO_BUILD_JOBS", "1")

ETH_RPC_URL = "http://127.0.0.1:9545"
MEM_RPC_URL = "http://127.0.0.1:9645"
NETWORK_HOST = "127.0.0.1"
P2P_PORT = 4015
ETH_RPC_PORT = 9545
MEM_RPC_PORT = 9645

SIGNER = "0x2222222222222222222222222222222222222222"
SIGNATURE_SCHEME = "raw_secp256k1"
SIGNATURE = "0x" + ("22" * 65)
PERSONALITY_VERSION = 1
SAVE_TIMEOUT_SECONDS = 30
INITIAL_NONCE = 0
METHOD_PROBE_PERSONALITY_ID = "0x00"

PERSONALITY_MARKDOWN = """# Codex Demo Personality

- Be direct, concise, and technically grounded.
- Treat Whirlpool mem personality state as the source of truth for this session.
- When explaining changes, start with the concrete outcome, then the reason.
- Prefer short progress updates and avoid filler.
"""

PROFILE_FILES = {
    "default": None,
    "leon": PROFILES_DIR / "leon.md",
    "ada": PROFILES_DIR / "ada.md",
    "sherry": PROFILES_DIR / "sherry.md",
}


def utc_now_iso() -> str:
    return dt.datetime.now(tz=dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def slugify(value: str) -> str:
    slug = re.sub(r"[^a-zA-Z0-9._-]+", "-", value.strip()).strip("-").lower()
    return slug or "profile"


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def require_tool(tool: str) -> None:
    if shutil.which(tool) is None:
        fail(f"missing required tool: {tool}")


def resolve_personality_markdown(profile_name: str) -> str:
    profile_path = PROFILE_FILES.get(profile_name)
    if profile_path is None:
        return PERSONALITY_MARKDOWN
    if not profile_path.exists():
        fail(f"missing profile file: {profile_path}")
    return profile_path.read_text(encoding="utf-8")


def materialize_builtin_profile(profile_name: str) -> Path:
    markdown = resolve_personality_markdown(profile_name)
    path = RUN_DIR / f"builtin-profile-{profile_name}.md"
    path.write_text(markdown, encoding="utf-8")
    return path


def ensure_run_dir() -> None:
    RUN_DIR.mkdir(parents=True, exist_ok=True)


def ensure_profile_store() -> None:
    PROFILE_STORE_DIR.mkdir(parents=True, exist_ok=True)


def load_profile_index() -> dict[str, Any]:
    if not PROFILE_INDEX_FILE.exists():
        return {"version": 1, "entries": []}
    try:
        payload = json.loads(PROFILE_INDEX_FILE.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {"version": 1, "entries": []}
    if not isinstance(payload, dict):
        return {"version": 1, "entries": []}
    entries = payload.get("entries")
    if not isinstance(entries, list):
        payload["entries"] = []
    return payload


def save_profile_index(index: dict[str, Any]) -> None:
    ensure_profile_store()
    PROFILE_INDEX_FILE.write_text(json.dumps(index, indent=2) + "\n", encoding="utf-8")


def index_entries_sorted(index: dict[str, Any]) -> list[dict[str, Any]]:
    entries = [entry for entry in index.get("entries", []) if isinstance(entry, dict)]
    entries.sort(key=lambda item: str(item.get("fetched_at", "")), reverse=True)
    return entries


def load_profile_registry() -> dict[str, Any]:
    if not PROFILE_REGISTRY_FILE.exists():
        return {"version": 1, "profiles": {}}
    try:
        payload = json.loads(PROFILE_REGISTRY_FILE.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {"version": 1, "profiles": {}}
    if not isinstance(payload, dict):
        return {"version": 1, "profiles": {}}
    profiles = payload.get("profiles")
    if not isinstance(profiles, dict):
        payload["profiles"] = {}
    return payload


def save_profile_registry(registry: dict[str, Any]) -> None:
    ensure_profile_store()
    PROFILE_REGISTRY_FILE.write_text(json.dumps(registry, indent=2) + "\n", encoding="utf-8")


def generate_personality_id() -> str:
    return "0x" + secrets.token_hex(16)


def normalize_personality_id(value: str) -> str:
    normalized = value.strip().lower()
    if not re.fullmatch(r"0x[0-9a-f]{2,64}", normalized):
        fail(f"invalid personality_id: {value}")
    return normalized


def resolve_personality_profile(profile_name: str, override_personality_id: str | None = None) -> dict[str, Any]:
    registry = load_profile_registry()
    profiles = registry.setdefault("profiles", {})
    raw = profiles.get(profile_name)
    entry = raw if isinstance(raw, dict) else {}

    personality_id = override_personality_id or str(entry.get("personality_id", "")).strip()
    if not personality_id:
        personality_id = generate_personality_id()
    personality_id = normalize_personality_id(personality_id)

    next_nonce_raw = entry.get("next_nonce", INITIAL_NONCE)
    try:
        next_nonce = int(next_nonce_raw)
    except (TypeError, ValueError):
        next_nonce = INITIAL_NONCE
    if next_nonce < 0:
        next_nonce = INITIAL_NONCE

    resolved = {
        "name": profile_name,
        "personality_id": personality_id,
        "next_nonce": next_nonce,
        "signer": str(entry.get("signer", SIGNER)),
    }
    profiles[profile_name] = resolved
    registry["version"] = 1
    save_profile_registry(registry)
    return resolved


def update_profile_after_finalize(profile_name: str, personality_id: str, finalized_nonce: int) -> None:
    registry = load_profile_registry()
    profiles = registry.setdefault("profiles", {})
    raw = profiles.get(profile_name)
    entry = raw if isinstance(raw, dict) else {}
    try:
        current_next = int(entry.get("next_nonce", INITIAL_NONCE))
    except (TypeError, ValueError):
        current_next = INITIAL_NONCE
    next_nonce = max(finalized_nonce + 1, current_next)
    profiles[profile_name] = {
        "name": profile_name,
        "personality_id": personality_id,
        "next_nonce": next_nonce,
        "signer": SIGNER,
    }
    registry["version"] = 1
    save_profile_registry(registry)


def cleanup_data_dir() -> None:
    shutil.rmtree(RUN_DIR / "data", ignore_errors=True)


def node_pid() -> int | None:
    if not NODE_PID_FILE.exists():
        recovered = find_demo_node_pid()
        if recovered is not None:
            NODE_PID_FILE.write_text(f"{recovered}\n", encoding="utf-8")
        return recovered
    contents = NODE_PID_FILE.read_text(encoding="utf-8").strip()
    if not contents:
        recovered = find_demo_node_pid()
        if recovered is not None:
            NODE_PID_FILE.write_text(f"{recovered}\n", encoding="utf-8")
        return recovered
    try:
        pid = int(contents)
    except ValueError:
        recovered = find_demo_node_pid()
        if recovered is not None:
            NODE_PID_FILE.write_text(f"{recovered}\n", encoding="utf-8")
        return recovered
    if process_is_running(pid):
        return pid
    recovered = find_demo_node_pid()
    if recovered is not None:
        NODE_PID_FILE.write_text(f"{recovered}\n", encoding="utf-8")
        return recovered
    return None


def find_demo_node_pid() -> int | None:
    config_marker = str(CONFIG_FILE)
    commands = (
        ["pgrep", "-f", f"whirlpool-node.*{config_marker}"],
        ["pgrep", "-f", f"cargo.*run.*-p\\s+whirlpool-node.*{config_marker}"],
        ["pgrep", "-f", "whirlpool-node"],
    )
    for cmd in commands:
        try:
            output = subprocess.check_output(cmd, text=True, stderr=subprocess.DEVNULL).strip()
        except (FileNotFoundError, subprocess.CalledProcessError):
            continue
        for line in output.splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                pid = int(line)
            except ValueError:
                continue
            if process_is_running(pid):
                return pid

    # Fallback when pgrep is unavailable.
    try:
        ps_output = subprocess.check_output(["ps", "-eo", "pid=,args="], text=True)
    except (FileNotFoundError, subprocess.CalledProcessError):
        return None
    for row in ps_output.splitlines():
        row = row.strip()
        if not row:
            continue
        parts = row.split(maxsplit=1)
        if len(parts) != 2:
            continue
        pid_str, args = parts
        if "whirlpool-node" not in args:
            continue
        try:
            pid = int(pid_str)
        except ValueError:
            continue
        if process_is_running(pid):
            return pid
    return None


def process_is_running(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    else:
        return True


def node_is_running() -> bool:
    pid = node_pid()
    if pid is None:
        return False
    if process_is_running(pid):
        return True
    NODE_PID_FILE.unlink(missing_ok=True)
    return False


def rpc_call(url: str, method: str, params: list[dict[str, Any]] | list[Any]) -> dict[str, Any]:
    payload = json.dumps(
        {
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1,
        }
    ).encode("utf-8")
    request = Request(url, data=payload, headers={"content-type": "application/json"})
    try:
        with urlopen(request, timeout=2) as response:
            return json.loads(response.read().decode("utf-8"))
    except HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        if body:
            return json.loads(body)
        raise
    except URLError as exc:
        raise RuntimeError(str(exc.reason)) from exc


def rpc_result_field(path: Path, field: str) -> Any:
    value: Any = json.loads(path.read_text(encoding="utf-8"))
    for part in field.split("."):
        if not isinstance(value, dict):
            raise KeyError(field)
        value = value.get(part)
        if value is None:
            raise KeyError(field)
    return value


def wait_for_eth_height() -> None:
    deadline = time.monotonic() + SAVE_TIMEOUT_SECONDS
    while time.monotonic() < deadline:
        if NODE_PID_FILE.exists() and not node_is_running():
            fail(f"whirlpool-node exited before the RPC server became ready; inspect {NODE_LOG_FILE}")

        try:
            response = rpc_call(ETH_RPC_URL, "eth_blockNumber", [])
        except Exception:
            time.sleep(0.2)
            continue

        result = response.get("result")
        if isinstance(result, str) and int(result, 16) >= 1:
            return
        time.sleep(0.2)

    fail(f"timed out waiting for eth_blockNumber >= 1 on {ETH_RPC_URL}")


def wait_for_finalized_personality(personality_id: str) -> None:
    deadline = time.monotonic() + SAVE_TIMEOUT_SECONDS
    while time.monotonic() < deadline:
        response = rpc_call(
            MEM_RPC_URL,
            "mem_getPersonality",
            [{"personality_id": personality_id}],
        )
        FETCH_RESPONSE_FILE.write_text(json.dumps(response, indent=2) + "\n", encoding="utf-8")
        if isinstance(response.get("result"), dict):
            return
        time.sleep(0.2)

    fail(f"timed out waiting for mem_getPersonality({personality_id}) to finalize")


def ensure_demo_codex_home() -> None:
    real_codex_home = Path.home() / ".codex"
    (CODEX_HOME_DIR / "skills").mkdir(parents=True, exist_ok=True)
    (CODEX_HOME_DIR / "memories").mkdir(parents=True, exist_ok=True)

    link_if_missing(real_codex_home / "auth.json", CODEX_HOME_DIR / "auth.json")
    link_if_missing(real_codex_home / "config.toml", CODEX_HOME_DIR / "config.toml")
    link_if_missing(real_codex_home / "skills" / ".system", CODEX_HOME_DIR / "skills" / ".system")
    link_if_missing(ROOT_DIR / "skills" / "whirlpool-mem-personality", CODEX_HOME_DIR / "skills" / "whirlpool-mem-personality")


def link_if_missing(source: Path, target: Path) -> None:
    if source.exists() and not target.exists():
        target.symlink_to(source)


def demo_codex(*args: str, stdin_path: Path | None = None, stdout_path: Path | None = None) -> None:
    env = os.environ.copy()
    env["CODEX_HOME"] = str(CODEX_HOME_DIR)
    stdin_handle = stdin_path.open("rb") if stdin_path else None
    stdout_handle = stdout_path.open("wb") if stdout_path else None
    try:
        subprocess.run(
            ["codex", *args],
            cwd=ROOT_DIR,
            env=env,
            stdin=stdin_handle,
            stdout=stdout_handle,
            check=True,
        )
    finally:
        if stdin_handle is not None:
            stdin_handle.close()
        if stdout_handle is not None:
            stdout_handle.close()


def write_save_prompt(profile_name: str, personality_id: str, nonce: int) -> None:
    personality_markdown = resolve_personality_markdown(profile_name)
    markdown_json = json.dumps(personality_markdown)
    SAVE_PROMPT_FILE.write_text(
        f"""Use $whirlpool-mem-personality to submit a Whirlpool mem personality update and verify that it becomes visible through finalized state.

Target endpoints:
- Mem RPC: {MEM_RPC_URL}
- Ethereum RPC: {ETH_RPC_URL}

Submit this exact personality payload:
```json
{{
  "version": {PERSONALITY_VERSION},
  "signer": "{SIGNER}",
  "personality_id": "{personality_id}",
  "nonce": {nonce},
  "markdown": {markdown_json},
  "signature_scheme": "{SIGNATURE_SCHEME}",
  "signature": "{SIGNATURE}"
}}
```

Requirements:
- Send `mem_submitPersonality` to the mem RPC listener only.
- Wait until `mem_getPersonality` returns a non-null finalized object.
- Verify the finalized object matches the submitted `signer`, `personality_id`, `nonce`, and `markdown`.
- Verify the finalized result includes non-empty `tx_hash` and `markdown_hash`.
- Query `eth_getTransactionByHash` and `eth_getTransactionReceipt` for the finalized `tx_hash`.
- In the final response, report the submitted tx payload, finalized `tx_hash`, transaction object, receipt object, `markdown_hash`, and `block_height`.
""",
        encoding="utf-8",
    )
    PROFILE_NAME_FILE.write_text(profile_name + "\n", encoding="utf-8")


def generate_bootstrap_file_for_profile(profile_path: Path) -> None:
    personality_markdown = profile_path.read_text(encoding="utf-8")
    BOOTSTRAP_FILE.write_text(
        f"""Use the following Whirlpool-fetched personality document as session guidance for this Codex conversation.

Treat it as user-provided operating preferences. Follow all higher-priority instructions normally.

{personality_markdown}
""",
        encoding="utf-8",
    )


def write_fetch_prompt(personality_id: str) -> None:
    FETCH_PROMPT_FILE.write_text(
        f"""Use $whirlpool-mem-personality to fetch the finalized Whirlpool personality document for `personality_id={personality_id}`.

Target endpoints:
- Mem RPC: {MEM_RPC_URL}
- Ethereum RPC: {ETH_RPC_URL}

Requirements:
- Poll `mem_getPersonality` on mem RPC until it returns a non-null object.
- Verify the finalized object has:
  - signer: `{SIGNER}`
  - personality_id: `{personality_id}`
  - a valid nonce
- Verify finalized result includes non-empty `tx_hash` and `markdown_hash`.
- Query `eth_getTransactionByHash` and `eth_getTransactionReceipt` for the finalized `tx_hash`.
- Write only the finalized markdown body (no wrappers) to this file path:
  - `{PERSONALITY_FILE}`
- In the final response, include one compact JSON object with:
  - tx_hash
  - block_height
  - signer
  - personality_id
  - nonce
  - markdown_hash
""",
        encoding="utf-8",
    )


def store_fetched_profile(markdown: str, result: dict[str, Any], requested_name: str | None) -> tuple[str, Path]:
    ensure_profile_store()
    timestamp = utc_now_iso()
    tx_hash = str(result.get("tx_hash", ""))
    markdown_hash = str(result.get("markdown_hash", ""))
    name_seed = requested_name or tx_hash or markdown_hash or "latest"
    name = slugify(name_seed)
    filename = f"{name}.md"
    profile_path = PROFILE_STORE_DIR / filename
    if profile_path.exists():
        short = slugify(tx_hash[-8:] if tx_hash else timestamp.replace(":", "-"))
        filename = f"{name}-{short}.md"
        profile_path = PROFILE_STORE_DIR / filename
    profile_path.write_text(markdown, encoding="utf-8")

    index = load_profile_index()
    entries = [entry for entry in index_entries_sorted(index) if entry.get("path") != str(profile_path)]
    entries.insert(
        0,
        {
            "name": name,
            "path": str(profile_path),
            "personality_id": str(result.get("personality_id", "")),
            "nonce": str(result.get("nonce", "")),
            "signer": str(result.get("signer", "")),
            "tx_hash": tx_hash,
            "markdown_hash": markdown_hash,
            "block_height": str(result.get("block_height", "")),
            "fetched_at": timestamp,
            "source": "whirlpool-mem-personality",
        },
    )
    index["version"] = 1
    index["entries"] = entries
    save_profile_index(index)
    return name, profile_path


def resolve_profile_ref(profile_ref: str | None) -> tuple[str, Path]:
    if profile_ref in PROFILE_FILES:
        return str(profile_ref), materialize_builtin_profile(str(profile_ref))

    index = load_profile_index()
    entries = index_entries_sorted(index)
    if profile_ref is None:
        if entries:
            entry = entries[0]
            path = Path(str(entry.get("path", "")))
            if path.exists():
                return str(entry.get("name", "latest")), path
        return "default", materialize_builtin_profile("default")

    ref = profile_ref.strip()
    as_path = Path(ref)
    if as_path.exists():
        return as_path.stem, as_path
    candidate = PROFILE_STORE_DIR / (ref if ref.endswith(".md") else f"{ref}.md")
    if candidate.exists():
        return candidate.stem, candidate

    for entry in entries:
        for field in ("name", "tx_hash", "markdown_hash", "personality_id"):
            if str(entry.get(field, "")) == ref:
                path = Path(str(entry.get("path", "")))
                if not path.exists():
                    fail(f"profile '{ref}' points to missing file: {path}")
                return str(entry.get("name", ref)), path

    fail(
        f"profile '{ref}' not found in {PROFILE_STORE_DIR}; "
        "use `codex_personality.sh profiles` to list available profiles"
    )


def resolve_fetch_personality_id(profile_ref: str | None, personality_id: str | None) -> tuple[str | None, str]:
    if personality_id:
        return profile_ref, normalize_personality_id(personality_id)
    if not profile_ref:
        fail("fetch requires --profile <name> or --personality-id <0x...>")
    registry = load_profile_registry()
    profiles = registry.get("profiles", {})
    if not isinstance(profiles, dict):
        fail(f"profile registry is invalid: {PROFILE_REGISTRY_FILE}")
    raw = profiles.get(profile_ref)
    if not isinstance(raw, dict):
        fail(
            f"profile '{profile_ref}' has no registered personality_id; "
            f"use save --profile {profile_ref} first or pass --personality-id"
        )
    resolved = str(raw.get("personality_id", "")).strip()
    if not resolved:
        fail(f"profile '{profile_ref}' has empty personality_id in {PROFILE_REGISTRY_FILE}")
    return profile_ref, normalize_personality_id(resolved)


def check_eth_rpc_rejects_mem_methods() -> None:
    response = rpc_call(
        ETH_RPC_URL,
        "mem_getPersonality",
        [{"personality_id": METHOD_PROBE_PERSONALITY_ID}],
    )
    if not isinstance(response.get("error"), dict):
        fail("expected mem_getPersonality on eth RPC to return an error")


def port_is_available(host: str, port: int) -> bool:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            sock.bind((host, port))
        except OSError:
            return False
    return True


def ensure_demo_ports_available() -> None:
    unavailable = [
        f"{NETWORK_HOST}:{port}"
        for port in (P2P_PORT, ETH_RPC_PORT, MEM_RPC_PORT)
        if not port_is_available(NETWORK_HOST, port)
    ]
    if unavailable:
        joined = ", ".join(unavailable)
        fail(
            "demo ports are already in use: "
            f"{joined}. Stop the previous demo process or free those ports before running start."
        )


def start_node() -> None:
    require_tool("nix")
    require_tool("python3")
    ensure_run_dir()

    if node_is_running():
        print(f"whirlpool-node is already running with pid {node_pid()}")
        return

    ensure_demo_ports_available()
    cleanup_data_dir()
    for path in (
        NODE_LOG_FILE,
        NODE_PID_FILE,
        SUBMIT_EVENTS_FILE,
        SUBMIT_MESSAGE_FILE,
        SUBMIT_RESPONSE_FILE,
        SUBMIT_TX_FILE,
        SUBMIT_RECEIPT_FILE,
        FETCH_RESPONSE_FILE,
        FETCH_EVENTS_FILE,
        FETCH_MESSAGE_FILE,
        PERSONALITY_FILE,
        BOOTSTRAP_FILE,
        SAVE_PROMPT_FILE,
        FETCH_PROMPT_FILE,
        PROFILE_NAME_FILE,
    ):
        path.unlink(missing_ok=True)

    log_handle = NODE_LOG_FILE.open("wb")
    process = subprocess.Popen(
        [
            "nix",
            "develop",
            "--command",
            "env",
            f"CARGO_BUILD_JOBS={CARGO_BUILD_JOBS}",
            "cargo",
            "run",
            "-p",
            "whirlpool-node",
            "--",
            "--config",
            str(CONFIG_FILE),
        ],
        cwd=ROOT_DIR,
        stdout=log_handle,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    log_handle.close()
    NODE_PID_FILE.write_text(f"{process.pid}\n", encoding="utf-8")

    try:
        wait_for_eth_height()
    except Exception:
        if process.poll() is not None:
            NODE_PID_FILE.unlink(missing_ok=True)
        raise

    print(f"whirlpool-node started on {ETH_RPC_URL} (mem RPC {MEM_RPC_URL})")


def save_personality(profile_name: str, personality_id_override: str | None) -> None:
    require_tool("codex")
    require_tool("python3")
    ensure_run_dir()

    if not node_is_running():
        fail("whirlpool-node is not running; start it first")

    profile_state = resolve_personality_profile(profile_name, personality_id_override)
    personality_id = str(profile_state["personality_id"])
    nonce = int(profile_state["next_nonce"])

    ensure_demo_codex_home()
    write_save_prompt(profile_name, personality_id, nonce)
    demo_codex(
        "exec",
        "--cd",
        str(ROOT_DIR),
        "--sandbox",
        "danger-full-access",
        "--json",
        "-o",
        str(SUBMIT_MESSAGE_FILE),
        "-",
        stdin_path=SAVE_PROMPT_FILE,
        stdout_path=SUBMIT_EVENTS_FILE,
    )

    response = rpc_call(
        MEM_RPC_URL,
        "mem_getPersonality",
        [{"personality_id": personality_id}],
    )
    SUBMIT_RESPONSE_FILE.write_text(json.dumps(response, indent=2) + "\n", encoding="utf-8")
    result = response.get("result")
    if not isinstance(result, dict):
        fail("save completed but mem_getPersonality did not return a finalized object; run fetch and retry")

    tx_hash = result.get("tx_hash")
    if not isinstance(tx_hash, str) or not tx_hash:
        fail("finalized save result missing tx_hash")
    finalized_nonce = result.get("nonce")
    try:
        finalized_nonce_int = int(finalized_nonce)
    except (TypeError, ValueError):
        fail(f"finalized save result missing valid nonce: {finalized_nonce}")
    update_profile_after_finalize(profile_name, personality_id, finalized_nonce_int)

    tx_response = rpc_call(ETH_RPC_URL, "eth_getTransactionByHash", [tx_hash])
    SUBMIT_TX_FILE.write_text(json.dumps(tx_response, indent=2) + "\n", encoding="utf-8")
    receipt_response = rpc_call(ETH_RPC_URL, "eth_getTransactionReceipt", [tx_hash])
    SUBMIT_RECEIPT_FILE.write_text(json.dumps(receipt_response, indent=2) + "\n", encoding="utf-8")

    check_eth_rpc_rejects_mem_methods()
    print(f"personality profile '{profile_name}' submitted and verified via Codex skill")
    print(f"personality_id: {personality_id}")
    print(f"nonce: {finalized_nonce_int}")
    print(f"finalized tx hash: {tx_hash}")
    print(f"submit tx response: {SUBMIT_TX_FILE}")
    print(f"submit receipt response: {SUBMIT_RECEIPT_FILE}")


def fetch_personality(profile_ref: str | None, personality_id: str | None) -> None:
    require_tool("codex")
    require_tool("python3")
    ensure_run_dir()

    if not node_is_running():
        fail("whirlpool-node is not running; start it first")

    saved_name, resolved_personality_id = resolve_fetch_personality_id(profile_ref, personality_id)

    ensure_demo_codex_home()
    write_fetch_prompt(resolved_personality_id)
    demo_codex(
        "exec",
        "--cd",
        str(ROOT_DIR),
        "--sandbox",
        "danger-full-access",
        "--json",
        "-o",
        str(FETCH_MESSAGE_FILE),
        "-",
        stdin_path=FETCH_PROMPT_FILE,
        stdout_path=FETCH_EVENTS_FILE,
    )

    wait_for_finalized_personality(resolved_personality_id)
    check_eth_rpc_rejects_mem_methods()

    result = rpc_result_field(FETCH_RESPONSE_FILE, "result")
    if not isinstance(result, dict):
        fail("mem_getPersonality did not return a finalized object")

    if result.get("signer") != SIGNER:
        fail(f"unexpected signer: {result.get('signer')}")
    if result.get("personality_id") != resolved_personality_id:
        fail(f"unexpected personality_id: {result.get('personality_id')}")

    markdown = result.get("markdown")
    if not isinstance(markdown, str):
        fail("missing finalized markdown")

    if not PERSONALITY_FILE.exists():
        PERSONALITY_FILE.write_text(markdown, encoding="utf-8")

    name, profile_path = store_fetched_profile(markdown, result, saved_name)
    generate_bootstrap_file_for_profile(profile_path)
    print(f"finalized personality written to {PERSONALITY_FILE}")
    print(f"fetched profile saved as '{name}' at {profile_path}")
    print(f"Codex bootstrap prompt written to {BOOTSTRAP_FILE}")


def launch_codex(profile_ref: str | None) -> None:
    require_tool("codex")
    ensure_run_dir()
    ensure_demo_codex_home()

    profile_name, profile_path = resolve_profile_ref(profile_ref)
    generate_bootstrap_file_for_profile(profile_path)

    bootstrap_prompt = BOOTSTRAP_FILE.read_text(encoding="utf-8")
    env = os.environ.copy()
    env["CODEX_HOME"] = str(CODEX_HOME_DIR)
    subprocess.run(
        ["codex", "--cd", str(ROOT_DIR), bootstrap_prompt],
        cwd=ROOT_DIR,
        env=env,
        check=True,
    )
    print(f"launched codex with profile '{profile_name}' from {profile_path}")


def hot_switch_demo() -> None:
    print(
        """Run these inside the live Codex session to demonstrate built-in mid-session switching:

  /personality pragmatic
  /personality friendly
  /personality none

These commands change Codex's built-in style only. They do not hot-load arbitrary Whirlpool markdown."""
    )


def status() -> None:
    ensure_run_dir()

    if node_is_running():
        print(f"node: running (pid {node_pid()})")
    else:
        print("node: stopped")

    try:
        response = rpc_call(ETH_RPC_URL, "eth_blockNumber", [])
    except Exception:
        print("eth_blockNumber: unavailable")
    else:
        result = response.get("result")
        if isinstance(result, str):
            print(f"eth_blockNumber: {int(result, 16)}")
        else:
            print("eth_blockNumber: unavailable")

    print(f"save result: {SUBMIT_MESSAGE_FILE}" if SUBMIT_MESSAGE_FILE.exists() else "save result: missing")
    print(f"submit tx: {SUBMIT_TX_FILE}" if SUBMIT_TX_FILE.exists() else "submit tx: missing")
    print(f"submit receipt: {SUBMIT_RECEIPT_FILE}" if SUBMIT_RECEIPT_FILE.exists() else "submit receipt: missing")
    print(f"personality markdown: {PERSONALITY_FILE}" if PERSONALITY_FILE.exists() else "personality markdown: missing")
    print(f"bootstrap prompt: {BOOTSTRAP_FILE}" if BOOTSTRAP_FILE.exists() else "bootstrap prompt: missing")
    print(f"fetch result: {FETCH_MESSAGE_FILE}" if FETCH_MESSAGE_FILE.exists() else "fetch result: missing")
    if PROFILE_NAME_FILE.exists():
        print(f"saved profile: {PROFILE_NAME_FILE.read_text(encoding='utf-8').strip()}")
    else:
        print("saved profile: missing")

    index = load_profile_index()
    entries = index_entries_sorted(index)
    print(f"fetched profiles store: {PROFILE_STORE_DIR}")
    print(f"fetched profiles count: {len(entries)}")
    registry = load_profile_registry()
    profiles = registry.get("profiles", {})
    profile_count = len(profiles) if isinstance(profiles, dict) else 0
    print(f"profile registry: {PROFILE_REGISTRY_FILE}")
    print(f"registered remote profiles: {profile_count}")


def list_profiles() -> None:
    ensure_run_dir()
    ensure_profile_store()
    index = load_profile_index()
    entries = index_entries_sorted(index)
    print(f"profile store: {PROFILE_STORE_DIR}")
    if not entries:
        print("no fetched profiles")
        return
    for entry in entries:
        name = str(entry.get("name", "unknown"))
        personality_id = str(entry.get("personality_id", ""))
        tx_hash = str(entry.get("tx_hash", ""))
        fetched_at = str(entry.get("fetched_at", ""))
        path = str(entry.get("path", ""))
        print(f"{name}\t{personality_id}\t{tx_hash}\t{fetched_at}\t{path}")


def stop_node() -> None:
    pid = node_pid()
    if pid is None or not node_is_running():
        NODE_PID_FILE.unlink(missing_ok=True)
        print("whirlpool-node is not running")
        return

    os.kill(pid, signal.SIGTERM)
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        if not process_is_running(pid):
            break
        time.sleep(0.1)

    if process_is_running(pid):
        os.kill(pid, signal.SIGKILL)

    NODE_PID_FILE.unlink(missing_ok=True)
    print(f"stopped whirlpool-node (pid {pid})")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog=Path(__file__).name)
    parser.add_argument(
        "command",
        choices=["start", "save", "fetch", "launch-codex", "profiles", "hot-switch-demo", "status", "stop"],
    )
    parser.add_argument(
        "--profile",
        default=None,
        help=(
            "save: built-in profile name (default/leon/ada/sherry); "
            "fetch: profile name used to resolve personality_id from registry and save local alias; "
            "launch-codex: fetched profile name/id/path"
        ),
    )
    parser.add_argument(
        "--personality-id",
        default=None,
        help="save/fetch: explicit remote personality_id (0x...)",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    commands = {
        "start": start_node,
        "profiles": list_profiles,
        "hot-switch-demo": hot_switch_demo,
        "status": status,
        "stop": stop_node,
    }
    if args.command == "save":
        profile_name = args.profile or "default"
        if profile_name not in PROFILE_FILES:
            allowed = ", ".join(sorted(PROFILE_FILES))
            fail(f"unknown save profile '{profile_name}'. allowed: {allowed}")
        save_personality(profile_name, args.personality_id)
        return
    if args.command == "fetch":
        fetch_personality(args.profile, args.personality_id)
        return
    if args.command == "launch-codex":
        launch_codex(args.profile)
        return
    commands[args.command]()


if __name__ == "__main__":
    main()

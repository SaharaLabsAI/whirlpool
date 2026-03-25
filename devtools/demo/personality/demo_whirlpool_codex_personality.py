#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import os
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
CONFIG_FILE = DEMO_DIR / "whirlpool-node-demo.toml"
NODE_LOG_FILE = RUN_DIR / "whirlpool-node.log"
NODE_PID_FILE = RUN_DIR / "node.pid"
SUBMIT_EVENTS_FILE = RUN_DIR / "save-events.jsonl"
SUBMIT_MESSAGE_FILE = RUN_DIR / "save-message.txt"
SUBMIT_RESPONSE_FILE = RUN_DIR / "submit-response.json"
FETCH_RESPONSE_FILE = RUN_DIR / "fetch-response.json"
PERSONALITY_FILE = RUN_DIR / "personality.md"
BOOTSTRAP_FILE = RUN_DIR / "codex-bootstrap.md"
SAVE_PROMPT_FILE = RUN_DIR / "save-prompt.md"
CODEX_HOME_DIR = RUN_DIR / "codex-home"
CARGO_BUILD_JOBS = os.environ.get("CARGO_BUILD_JOBS", "1")

ETH_RPC_URL = "http://127.0.0.1:9545"
MEM_RPC_URL = "http://127.0.0.1:9645"
NETWORK_HOST = "127.0.0.1"
P2P_PORT = 4015
ETH_RPC_PORT = 9545
MEM_RPC_PORT = 9645

PERSONALITY_ID = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
SIGNER = "0x2222222222222222222222222222222222222222"
NONCE = "7"
SIGNATURE_SCHEME = "raw_secp256k1"
SIGNATURE = "0x" + ("22" * 65)
PERSONALITY_VERSION = 1
SAVE_TIMEOUT_SECONDS = 30

PERSONALITY_MARKDOWN = """# Codex Demo Personality

- Be direct, concise, and technically grounded.
- Treat Whirlpool mem personality state as the source of truth for this session.
- When explaining changes, start with the concrete outcome, then the reason.
- Prefer short progress updates and avoid filler.
"""


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def require_tool(tool: str) -> None:
    if shutil.which(tool) is None:
        fail(f"missing required tool: {tool}")


def ensure_run_dir() -> None:
    RUN_DIR.mkdir(parents=True, exist_ok=True)


def cleanup_data_dir() -> None:
    shutil.rmtree(RUN_DIR / "data", ignore_errors=True)


def node_pid() -> int | None:
    if not NODE_PID_FILE.exists():
        return None
    contents = NODE_PID_FILE.read_text(encoding="utf-8").strip()
    if not contents:
        return None
    try:
        return int(contents)
    except ValueError:
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


def wait_for_finalized_personality() -> None:
    deadline = time.monotonic() + SAVE_TIMEOUT_SECONDS
    while time.monotonic() < deadline:
        response = rpc_call(
            MEM_RPC_URL,
            "mem_getPersonality",
            [{"personality_id": PERSONALITY_ID}],
        )
        FETCH_RESPONSE_FILE.write_text(json.dumps(response, indent=2) + "\n", encoding="utf-8")
        if isinstance(response.get("result"), dict):
            return
        time.sleep(0.2)

    fail(f"timed out waiting for mem_getPersonality({PERSONALITY_ID}) to finalize")


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


def write_save_prompt() -> None:
    markdown_json = json.dumps(PERSONALITY_MARKDOWN)
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
  "personality_id": "{PERSONALITY_ID}",
  "nonce": {NONCE},
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
- In the final response, report the finalized `tx_hash`, `markdown_hash`, and `block_height`.
""",
        encoding="utf-8",
    )


def generate_bootstrap_file() -> None:
    personality_markdown = PERSONALITY_FILE.read_text(encoding="utf-8")
    BOOTSTRAP_FILE.write_text(
        f"""Use the following Whirlpool-fetched personality document as session guidance for this Codex conversation.

Treat it as user-provided operating preferences. Follow all higher-priority instructions normally.

{personality_markdown}
""",
        encoding="utf-8",
    )


def check_eth_rpc_rejects_mem_methods() -> None:
    response = rpc_call(
        ETH_RPC_URL,
        "mem_getPersonality",
        [{"personality_id": PERSONALITY_ID}],
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
        FETCH_RESPONSE_FILE,
        PERSONALITY_FILE,
        BOOTSTRAP_FILE,
        SAVE_PROMPT_FILE,
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


def save_personality() -> None:
    require_tool("codex")
    require_tool("python3")
    ensure_run_dir()

    if not node_is_running():
        fail("whirlpool-node is not running; start it first")

    ensure_demo_codex_home()
    write_save_prompt()
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
        [{"personality_id": PERSONALITY_ID}],
    )
    SUBMIT_RESPONSE_FILE.write_text(json.dumps(response, indent=2) + "\n", encoding="utf-8")
    check_eth_rpc_rejects_mem_methods()
    print("personality submitted and verified via Codex skill")


def fetch_personality() -> None:
    require_tool("python3")
    ensure_run_dir()

    if not node_is_running():
        fail("whirlpool-node is not running; start it first")

    wait_for_finalized_personality()
    check_eth_rpc_rejects_mem_methods()

    result = rpc_result_field(FETCH_RESPONSE_FILE, "result")
    if not isinstance(result, dict):
        fail("mem_getPersonality did not return a finalized object")

    if result.get("signer") != SIGNER:
        fail(f"unexpected signer: {result.get('signer')}")
    if result.get("personality_id") != PERSONALITY_ID:
        fail(f"unexpected personality_id: {result.get('personality_id')}")
    if str(result.get("nonce")) != NONCE:
        fail(f"unexpected nonce: {result.get('nonce')}")

    markdown = result.get("markdown")
    if not isinstance(markdown, str):
        fail("missing finalized markdown")

    PERSONALITY_FILE.write_text(markdown, encoding="utf-8")
    generate_bootstrap_file()
    print(f"finalized personality written to {PERSONALITY_FILE}")
    print(f"Codex bootstrap prompt written to {BOOTSTRAP_FILE}")


def launch_codex() -> None:
    require_tool("codex")
    ensure_run_dir()
    ensure_demo_codex_home()

    if not BOOTSTRAP_FILE.exists():
        fail(f"missing {BOOTSTRAP_FILE}; run fetch first")

    bootstrap_prompt = BOOTSTRAP_FILE.read_text(encoding="utf-8")
    env = os.environ.copy()
    env["CODEX_HOME"] = str(CODEX_HOME_DIR)
    subprocess.run(
        ["codex", "--cd", str(ROOT_DIR), bootstrap_prompt],
        cwd=ROOT_DIR,
        env=env,
        check=True,
    )


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
    print(f"personality markdown: {PERSONALITY_FILE}" if PERSONALITY_FILE.exists() else "personality markdown: missing")
    print(f"bootstrap prompt: {BOOTSTRAP_FILE}" if BOOTSTRAP_FILE.exists() else "bootstrap prompt: missing")


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
        choices=["start", "save", "fetch", "launch-codex", "hot-switch-demo", "status", "stop"],
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    commands = {
        "start": start_node,
        "save": save_personality,
        "fetch": fetch_personality,
        "launch-codex": launch_codex,
        "hot-switch-demo": hot_switch_demo,
        "status": status,
        "stop": stop_node,
    }
    commands[args.command]()


if __name__ == "__main__":
    main()

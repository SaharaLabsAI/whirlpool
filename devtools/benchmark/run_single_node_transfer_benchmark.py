#!/usr/bin/env python3
"""Run benchmark command and emit normalized benchmark JSON."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def git_sha_fallback() -> str:
    sha = os.getenv("GITHUB_SHA")
    if sha:
        return sha
    try:
        return subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    except Exception:
        return "unknown"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, help="Raw benchmark JSON output path")
    parser.add_argument("--command", required=True, help="Benchmark command to execute")
    parser.add_argument(
        "--timeout-seconds",
        type=int,
        default=int(os.getenv("BENCHMARK_TIMEOUT_SECONDS", "3600")),
        help="Benchmark command timeout in seconds",
    )
    return parser.parse_args()


def extract_metrics_from_stdout(stdout: str) -> dict[str, Any]:
    for line in reversed(stdout.splitlines()):
        candidate = line.strip()
        if not candidate:
            continue
        if not candidate.startswith("{"):
            continue
        try:
            parsed = json.loads(candidate)
        except json.JSONDecodeError:
            continue
        if isinstance(parsed, dict):
            return parsed

    raise ValueError("benchmark command did not emit a JSON metrics line on stdout")


def main() -> int:
    args = parse_args()

    start = time.perf_counter()
    proc = subprocess.run(
        args.command,
        shell=True,
        timeout=args.timeout_seconds,
        text=True,
        capture_output=True,
    )
    command_elapsed_seconds = max(time.perf_counter() - start, 1e-9)

    if proc.stdout:
        print(proc.stdout, end="" if proc.stdout.endswith("\n") else "\n")
    if proc.stderr:
        print(proc.stderr, end="" if proc.stderr.endswith("\n") else "\n")

    metrics: dict[str, Any] = {}
    if proc.returncode == 0:
        metrics = extract_metrics_from_stdout(proc.stdout)
        required_metrics = [
            "measurement_window_seconds",
            "sender_accounts",
            "recipient_accounts",
            "start_block",
            "end_block",
            "block_count",
            "average_block_time_seconds",
            "transaction_count",
            "tps",
        ]
        missing = [key for key in required_metrics if key not in metrics]
        if missing:
            raise SystemExit(f"benchmark metrics JSON missing keys: {missing}")

    payload = {
        "timestamp": utc_now_iso(),
        "git_sha": git_sha_fallback(),
        "command": args.command,
        "command_exit_code": proc.returncode,
        "command_elapsed_seconds": command_elapsed_seconds,
        **metrics,
    }

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    print(f"raw benchmark result written to {output_path}")

    if proc.returncode != 0:
        print("benchmark command failed")
        return proc.returncode

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

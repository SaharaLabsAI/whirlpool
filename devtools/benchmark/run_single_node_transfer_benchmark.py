#!/usr/bin/env python3
"""Run a deterministic benchmark command and emit raw benchmark JSON."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def git_sha_fallback() -> str:
    sha = os.getenv("GITHUB_SHA")
    if sha:
        return sha
    try:
        return (
            subprocess.check_output(["git", "rev-parse", "HEAD"], text=True)
            .strip()
        )
    except Exception:
        return "unknown"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, help="Raw benchmark JSON output path")
    parser.add_argument(
        "--command",
        required=True,
        help="Benchmark command to execute",
    )
    parser.add_argument(
        "--transfer-count",
        type=int,
        default=int(os.getenv("BENCH_TRANSFER_COUNT", "1")),
        help="Logical transfer count used for TPS computation",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=int,
        default=int(os.getenv("BENCHMARK_TIMEOUT_SECONDS", "3600")),
        help="Benchmark command timeout in seconds",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    start = time.perf_counter()
    proc = subprocess.run(args.command, shell=True, timeout=args.timeout_seconds)
    elapsed_seconds = max(time.perf_counter() - start, 1e-9)

    successful_transfers = args.transfer_count if proc.returncode == 0 else 0
    tps = successful_transfers / elapsed_seconds

    payload = {
        "timestamp": utc_now_iso(),
        "git_sha": git_sha_fallback(),
        "transfer_count": args.transfer_count,
        "successful_transfers": successful_transfers,
        "elapsed_seconds": elapsed_seconds,
        "tps": tps,
        "command": args.command,
        "command_exit_code": proc.returncode,
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

#!/usr/bin/env python3
"""Update benchmark history and render latest report + gate status."""

from __future__ import annotations

import argparse
import json
import statistics
from datetime import datetime, timezone
from html import escape
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--raw-result", required=True)
    parser.add_argument("--history-in", required=True)
    parser.add_argument("--history-out", required=True)
    parser.add_argument("--report-out", required=True)
    parser.add_argument("--status-out", required=True)
    return parser.parse_args()


def read_json(path: Path, default: Any) -> Any:
    if not path.exists():
        return default
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def to_iso_utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")



def main() -> int:
    args = parse_args()

    raw = read_json(Path(args.raw_result), default={})
    required = [
        "timestamp",
        "git_sha",
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
    missing = [k for k in required if k not in raw]
    if missing:
        raise SystemExit(f"raw benchmark result missing keys: {missing}")

    history = read_json(Path(args.history_in), default={"schema_version": 2, "samples": []})
    samples = history.get("samples", [])
    if not isinstance(samples, list):
        raise SystemExit("history.json has invalid 'samples' value")

    prior_samples = list(samples)
    prior_tps = [float(s["tps"]) for s in prior_samples if "tps" in s]
    recent_7 = prior_tps[-7:]

    warming_up = len(recent_7) < 7
    rolling_median = statistics.median(recent_7) if not warming_up else None

    current_tps = float(raw["tps"])
    threshold_tps = (rolling_median * 0.5) if rolling_median is not None else None
    regression = False if warming_up else (current_tps < float(threshold_tps))

    sample_entry = {
        "timestamp": raw["timestamp"],
        "git_sha": raw["git_sha"],
        "measurement_window_seconds": int(raw["measurement_window_seconds"]),
        "sender_accounts": int(raw["sender_accounts"]),
        "recipient_accounts": int(raw["recipient_accounts"]),
        "start_block": int(raw["start_block"]),
        "end_block": int(raw["end_block"]),
        "block_count": int(raw["block_count"]),
        "average_block_time_seconds": float(raw["average_block_time_seconds"]),
        "transaction_count": int(raw["transaction_count"]),
        "submitted_transactions": int(raw.get("submitted_transactions", 0)),
        "accepted_transactions": int(raw.get("accepted_transactions", 0)),
        "rejected_submissions": int(raw.get("rejected_submissions", 0)),
        "tps": current_tps,
    }

    updated_samples = prior_samples + [sample_entry]
    history_out = {
        "schema_version": 2,
        "updated_at": to_iso_utc_now(),
        "samples": updated_samples,
    }

    status = {
        "schema_version": 2,
        "warming_up_baseline": warming_up,
        "current_tps": current_tps,
        "rolling_7_night_median_tps": rolling_median,
        "threshold_tps": threshold_tps,
        "regression_detected": regression,
        "should_fail": bool(regression),
        "source_of_truth": "gh-pages/benchmarks/history.json",
        "note": "Coarse large-regression signal on shared CI runners.",
    }

    acceptance_rate = None
    if sample_entry["submitted_transactions"] > 0:
        acceptance_rate = (
            sample_entry["accepted_transactions"] / sample_entry["submitted_transactions"]
        ) * 100.0

    report = f"""<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\" />
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
  <title>Whirlpool Nightly ERC20 Benchmark (Single Node)</title>
  <style>
    body {{ font-family: system-ui, sans-serif; margin: 2rem; line-height: 1.4; }}
    code {{ background: #f5f5f5; padding: 0.1rem 0.3rem; border-radius: 4px; }}
    .ok {{ color: #0a7f28; font-weight: 600; }}
    .bad {{ color: #b00020; font-weight: 700; }}
    table {{ border-collapse: collapse; margin-top: 1rem; }}
    td, th {{ border: 1px solid #ddd; padding: 0.4rem 0.6rem; text-align: left; }}
  </style>
</head>
<body>
  <h1>Whirlpool Nightly ERC20 Benchmark (Single Node)</h1>
  <p><strong>Run timestamp:</strong> {escape(str(raw['timestamp']))}</p>
  <p><strong>Commit SHA:</strong> <code>{escape(str(raw['git_sha']))}</code></p>
  <p><strong>Source of truth:</strong> <code>gh-pages/benchmarks/history.json</code></p>
  <p><strong>Policy:</strong> fail when <code>current_tps &lt; 0.5 × rolling_7_night_median_tps</code> (only after warm-up).</p>

  <table>
    <tr><th>Metric</th><th>Value</th></tr>
    <tr><td>Measurement window (seconds)</td><td>{sample_entry['measurement_window_seconds']}</td></tr>
    <tr><td>Sender accounts</td><td>{sample_entry['sender_accounts']}</td></tr>
    <tr><td>Recipient accounts</td><td>{sample_entry['recipient_accounts']}</td></tr>
    <tr><td>Start block</td><td>{sample_entry['start_block']}</td></tr>
    <tr><td>End block</td><td>{sample_entry['end_block']}</td></tr>
    <tr><td>Number of blocks</td><td>{sample_entry['block_count']}</td></tr>
    <tr><td>Average block time (seconds)</td><td>{sample_entry['average_block_time_seconds']:.6f}</td></tr>
    <tr><td>Number of transactions (packaged)</td><td>{sample_entry['transaction_count']}</td></tr>
    <tr><td>Submitted transactions</td><td>{sample_entry['submitted_transactions']}</td></tr>
    <tr><td>Accepted submissions</td><td>{sample_entry['accepted_transactions']}</td></tr>
    <tr><td>Rejected submissions</td><td>{sample_entry['rejected_submissions']}</td></tr>
    <tr><td>Submission acceptance rate</td><td>{'n/a' if acceptance_rate is None else f'{acceptance_rate:.2f}%'} </td></tr>
    <tr><td>Current TPS</td><td>{current_tps:.6f}</td></tr>
    <tr><td>Rolling 7-night median TPS</td><td>{'warming up' if rolling_median is None else f'{rolling_median:.6f}'}</td></tr>
    <tr><td>Threshold TPS (50% median)</td><td>{'warming up' if threshold_tps is None else f'{threshold_tps:.6f}'}</td></tr>
  </table>

  <p><strong>Warm-up baseline:</strong> {'true' if warming_up else 'false'}</p>
  <p><strong>Regression verdict:</strong>
    <span class=\"{'bad' if regression else 'ok'}\">{'REGRESSION DETECTED' if regression else 'OK'}</span>
  </p>
  <p><em>This benchmark is a coarse large-regression signal and not a strict lab-grade benchmark.</em></p>
</body>
</html>
"""

    write_json(Path(args.history_out), history_out)
    write_json(Path(args.status_out), status)

    report_path = Path(args.report_out)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(report, encoding="utf-8")

    print(f"history updated: {args.history_out}")
    print(f"report generated: {args.report_out}")
    print(f"status generated: {args.status_out}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

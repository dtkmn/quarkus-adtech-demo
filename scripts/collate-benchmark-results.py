#!/usr/bin/env python3
from __future__ import annotations

import csv
import json
import re
import statistics
import sys
from collections import defaultdict
from pathlib import Path


RUN_FILE_RE = re.compile(r"^(?P<service>.+)-run-(?P<run>\d+)-summary\.json$")
ANSI_ESCAPE_RE = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")


def load_meta(meta_path: Path) -> dict[str, str]:
    meta: dict[str, str] = {}
    if not meta_path.exists():
        return meta

    for line in meta_path.read_text().splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            meta[key.strip()] = value.strip()
    return meta


def metric_value(metrics: dict, name: str, key: str, default: float = 0.0) -> float:
    metric = metrics.get(name, {})
    values = metric.get("values", metric)
    value = values.get(key, default)
    if value is None:
        return default
    return float(value)


def parse_percent(raw: str) -> float:
    return float(raw.strip().rstrip("%") or 0.0)


def parse_bytes(raw: str) -> float:
    units = {
        "b": 1,
        "kb": 1000,
        "mb": 1000**2,
        "gb": 1000**3,
        "tb": 1000**4,
        "kib": 1024,
        "mib": 1024**2,
        "gib": 1024**3,
        "tib": 1024**4,
    }

    raw = raw.strip()
    match = re.match(r"^(?P<value>[0-9]+(?:\.[0-9]+)?)\s*(?P<unit>[A-Za-z]+)?$", raw)
    if not match:
        raise ValueError(f"Cannot parse byte quantity: {raw!r}")

    value = float(match.group("value"))
    unit = (match.group("unit") or "b").lower()
    return value * units[unit]


def parse_mem_usage(raw: str) -> tuple[float, float]:
    usage, limit = [part.strip() for part in raw.split("/", 1)]
    return parse_bytes(usage), parse_bytes(limit)


def summarize_stats(stats_path: Path) -> dict[str, float]:
    if not stats_path.exists():
        return {}

    cpu_values: list[float] = []
    mem_values: list[float] = []
    mem_limits: list[float] = []
    pids_values: list[int] = []

    cleaned = ANSI_ESCAPE_RE.sub("", stats_path.read_text())
    decoder = json.JSONDecoder()
    index = 0

    while index < len(cleaned):
        next_object = cleaned.find("{", index)
        if next_object == -1:
            break
        try:
            record, index = decoder.raw_decode(cleaned, next_object)
        except json.JSONDecodeError:
            index = next_object + 1
            continue

        cpu_values.append(parse_percent(record["CPUPerc"]))
        mem_usage, mem_limit = parse_mem_usage(record["MemUsage"])
        mem_values.append(mem_usage)
        mem_limits.append(mem_limit)
        pids_values.append(int(record["PIDs"]))

    if not cpu_values:
        return {}

    return {
        "cpu_avg_pct": statistics.fmean(cpu_values),
        "cpu_max_pct": max(cpu_values),
        "mem_avg_bytes": statistics.fmean(mem_values),
        "mem_max_bytes": max(mem_values),
        "mem_limit_bytes": max(mem_limits),
        "pids_max": max(pids_values),
    }


def service_run_rows(results_dir: Path) -> list[dict[str, float | int | str]]:
    rows: list[dict[str, float | int | str]] = []
    for summary_file in sorted(results_dir.glob("*-run-*-summary.json")):
        match = RUN_FILE_RE.match(summary_file.name)
        if not match:
            continue

        service = match.group("service")
        run = int(match.group("run"))
        metrics = json.loads(summary_file.read_text())["metrics"]

        row: dict[str, float | int | str] = {
            "service": service,
            "run": run,
            "http_reqs_count": metric_value(metrics, "http_reqs", "count"),
            "http_reqs_rate": metric_value(metrics, "http_reqs", "rate"),
            "http_req_failed_rate": metric_value(metrics, "http_req_failed", "rate"),
            "checks_rate": metric_value(metrics, "checks", "value"),
            "valid_responses_rate": metric_value(metrics, "valid_responses", "value"),
            "accepted_responses": metric_value(metrics, "accepted_responses", "count"),
            "filtered_responses": metric_value(metrics, "filtered_responses", "count"),
            "http_req_duration_avg_ms": metric_value(metrics, "http_req_duration", "avg"),
            "http_req_duration_p90_ms": metric_value(metrics, "http_req_duration", "p(90)"),
            "http_req_duration_p95_ms": metric_value(metrics, "http_req_duration", "p(95)"),
            "http_req_duration_max_ms": metric_value(metrics, "http_req_duration", "max"),
        }

        receiver_stats = summarize_stats(results_dir / f"{service}-run-{run:02d}-receiver-stats.ndjson")
        kafka_stats = summarize_stats(results_dir / f"{service}-run-{run:02d}-kafka-stats.ndjson")

        for key, value in receiver_stats.items():
            row[f"receiver_{key}"] = value
        for key, value in kafka_stats.items():
            row[f"kafka_{key}"] = value

        rows.append(row)

    return rows


def aggregate_rows(rows: list[dict[str, float | int | str]]) -> list[dict[str, float | int | str]]:
    grouped: dict[str, list[dict[str, float | int | str]]] = defaultdict(list)
    for row in rows:
        grouped[str(row["service"])].append(row)

    aggregates: list[dict[str, float | int | str]] = []
    for service, service_rows in sorted(grouped.items()):
        aggregate: dict[str, float | int | str] = {
            "service": service,
            "runs": len(service_rows),
        }

        numeric_keys = sorted(
            {
                key
                for row in service_rows
                for key, value in row.items()
                if key not in {"service", "run"} and isinstance(value, (int, float))
            }
        )

        for key in numeric_keys:
            values = [float(row[key]) for row in service_rows if key in row]
            aggregate[f"{key}_avg"] = statistics.fmean(values)
            aggregate[f"{key}_max"] = max(values)
            aggregate[f"{key}_min"] = min(values)

        aggregates.append(aggregate)

    return aggregates


def write_csv(path: Path, rows: list[dict[str, float | int | str]]) -> None:
    if not rows:
        return

    fieldnames = sorted({key for row in rows for key in row.keys()})
    with path.open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def write_summary_markdown(path: Path, rows: list[dict[str, float | int | str]]) -> None:
    if not rows:
        return

    lines = [
        "| service | runs | req/s avg | p95 avg (ms) | receiver cpu max % | receiver mem max MiB | kafka cpu max % | kafka mem max MiB |",
        "|---|---:|---:|---:|---:|---:|---:|---:|",
    ]

    for row in rows:
        lines.append(
            "| {service} | {runs} | {reqs:.2f} | {p95:.2f} | {receiver_cpu:.2f} | {receiver_mem:.2f} | {kafka_cpu:.2f} | {kafka_mem:.2f} |".format(
                service=row["service"],
                runs=int(row["runs"]),
                reqs=float(row.get("http_reqs_rate_avg", 0.0)),
                p95=float(row.get("http_req_duration_p95_ms_avg", 0.0)),
                receiver_cpu=float(row.get("receiver_cpu_max_pct_max", 0.0)),
                receiver_mem=float(row.get("receiver_mem_max_bytes_max", 0.0)) / (1024**2),
                kafka_cpu=float(row.get("kafka_cpu_max_pct_max", 0.0)),
                kafka_mem=float(row.get("kafka_mem_max_bytes_max", 0.0)) / (1024**2),
            )
        )

    path.write_text("\n".join(lines) + "\n")


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: collate-benchmark-results.py <results-dir>", file=sys.stderr)
        return 1

    results_dir = Path(sys.argv[1]).resolve()
    meta = load_meta(results_dir / "run-meta.txt")
    rows = service_run_rows(results_dir)
    aggregates = aggregate_rows(rows)

    write_csv(results_dir / "runs.csv", rows)
    write_csv(results_dir / "summary.csv", aggregates)
    write_summary_markdown(results_dir / "summary.md", aggregates)

    payload = {
        "meta": meta,
        "runs": rows,
        "summary": aggregates,
    }
    (results_dir / "summary.json").write_text(json.dumps(payload, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

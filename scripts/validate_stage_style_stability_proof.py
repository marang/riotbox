#!/usr/bin/env python3
"""Validate a normalized repeated-run proof for the stage-style stability seam."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.stage_style_stability_proof.v1"
SCHEMA_VERSION = 1
MIX_RELATIVE_PATH = Path("feral-grid/04_riotbox_generated_support_mix.wav")
MANIFEST_RELATIVE_PATH = Path("feral-grid/manifest.json")
SUMMARY_RELATIVE_PATH = Path("observer-audio-summary.json")
EVENTS_RELATIVE_PATH = Path("events.ndjson")


def main() -> int:
    args = parse_args()
    try:
        proof = build_proof(args.run_dirs)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid stage-style stability proof: {error}", file=sys.stderr)
        return 1

    if args.write_proof is not None:
        args.write_proof.write_text(json.dumps(proof, indent=2, sort_keys=True) + "\n")

    print(
        "stage-style stability proof ok: "
        f"runs={proof['run_count']} mix={proof['stable_mix_sha256']} "
        f"proof={proof['normalized_proof_sha256']}"
    )
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate normalized repeated-run stage-style stability evidence."
    )
    parser.add_argument(
        "--write-proof",
        type=Path,
        help="optional path for the normalized stability proof JSON",
    )
    parser.add_argument("run_dirs", type=Path, nargs="+")
    return parser.parse_args()


def build_proof(run_dirs: list[Path]) -> dict[str, Any]:
    if len(run_dirs) < 2:
        raise ValueError("expected at least two stage-style run directories")

    runs = [normalize_run(index + 1, run_dir) for index, run_dir in enumerate(run_dirs)]
    mix_hashes = {run["mix_sha256"] for run in runs}
    if len(mix_hashes) != 1:
        raise ValueError(f"stage-style mix hashes differ: {sorted(mix_hashes)}")

    proof_without_hash = {
        "schema": SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "boundary": "stage-style restore-diversity repeated run",
        "run_count": len(runs),
        "stable_mix_sha256": runs[0]["mix_sha256"],
        "required_commit_boundaries": ["Bar", "Beat", "Phrase"],
        "scope": "bounded CI-safe observer/audio stability proof, not host-audio soak",
        "runs": runs,
    }
    return {
        **proof_without_hash,
        "normalized_proof_sha256": sha256_text(canonical_json(proof_without_hash)),
    }


def normalize_run(index: int, run_dir: Path) -> dict[str, Any]:
    events_path = run_dir / EVENTS_RELATIVE_PATH
    manifest_path = run_dir / MANIFEST_RELATIVE_PATH
    summary_path = run_dir / SUMMARY_RELATIVE_PATH
    mix_path = run_dir / MIX_RELATIVE_PATH

    events = read_ndjson(events_path)
    manifest = read_json(manifest_path)
    summary = read_json(summary_path)

    validate_events(events, index)
    validate_manifest(manifest)
    validate_summary(summary)
    require_existing_file(mix_path, f"run {index} mix")

    control_path = summary["control_path"]
    output_path = summary["output_path"]
    metrics = output_path["metrics"]

    return {
        "run": index,
        "observer_event_count": len(events),
        "observer_sha256": sha256_file(events_path),
        "summary_sha256": sha256_file(summary_path),
        "manifest_sha256": sha256_file(manifest_path),
        "mix_sha256": sha256_file(mix_path),
        "commit_count": control_path["commit_count"],
        "commit_boundaries": sorted(control_path["commit_boundaries"]),
        "key_outcomes": sorted(control_path["key_outcomes"]),
        "pack_id": output_path["pack_id"],
        "artifact_count": output_path["artifact_count"],
        "full_mix_rms": metrics["full_mix_rms"],
        "full_mix_low_band_rms": metrics["full_mix_low_band_rms"],
        "needs_human_listening": summary["needs_human_listening"],
    }


def validate_events(events: list[dict[str, Any]], index: int) -> None:
    if len(events) < 24:
        raise ValueError(f"run {index} must contain at least 24 observer events")
    first = events[0]
    require_equal(first, "event", "observer_started")
    launch = require_object_field(first, "launch")
    require_equal(launch, "probe", "stage-style-restore-diversity")
    for event in events:
        require_object_field(event, "snapshot")
    required_outcomes = {
        ("c", "queue_capture_bar"),
        ("w", "queue_w30_trigger_pad"),
        ("f", "queue_tr909_fill"),
        ("g", "queue_mc202_generate_follower"),
    }
    outcomes = {
        (event.get("key"), event.get("outcome"))
        for event in events
        if event.get("event") == "key_outcome"
    }
    missing = required_outcomes - outcomes
    if missing:
        raise ValueError(f"run {index} missing key outcomes: {sorted(missing)}")


def validate_manifest(manifest: dict[str, Any]) -> None:
    require_equal(manifest, "schema_version", 1)
    require_equal(manifest, "pack_id", "feral-grid-demo")
    require_equal(manifest, "result", "pass")
    artifacts = require_list(manifest, "artifacts")
    if len(artifacts) < 4:
        raise ValueError("manifest must include generated audio artifacts")
    metrics = require_object_field(manifest, "metrics")
    full_grid = require_object_field(metrics, "full_grid_mix")
    require_metric_above(full_grid, ("signal", "rms"), 0.000001)
    require_metric_above(full_grid, ("low_band", "rms"), 0.000001)


def validate_summary(summary: dict[str, Any]) -> None:
    require_equal(summary, "schema", "riotbox.observer_audio_summary.v1")
    require_equal(summary, "schema_version", 1)
    require_bool(summary, "needs_human_listening")

    control_path = require_object_field(summary, "control_path")
    require_equal(control_path, "present", True)
    commit_count = require_int(control_path, "commit_count")
    if commit_count < 12:
        raise ValueError(f"commit_count must be at least 12, got {commit_count}")
    boundaries = set(require_string_list(control_path, "commit_boundaries"))
    required_boundaries = {"Bar", "Beat", "Phrase"}
    if not required_boundaries.issubset(boundaries):
        raise ValueError(f"missing commit boundaries: {sorted(required_boundaries - boundaries)}")

    output_path = require_object_field(summary, "output_path")
    require_equal(output_path, "present", True)
    require_equal(output_path, "pack_id", "feral-grid-demo")
    require_equal(output_path, "manifest_result", "pass")
    if require_string_list(output_path, "issues"):
        raise ValueError("output_path issues must be empty")
    metrics = require_object_field(output_path, "metrics")
    require_number_above(metrics, "full_mix_rms", 0.000001)
    require_number_above(metrics, "full_mix_low_band_rms", 0.000001)


def read_json(path: Path) -> dict[str, Any]:
    return require_object(json.loads(path.read_text()), str(path))


def read_ndjson(path: Path) -> list[dict[str, Any]]:
    events = []
    for line_number, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip():
            continue
        events.append(require_object(json.loads(line), f"{path}:{line_number}"))
    return events


def require_existing_file(path: Path, name: str) -> Path:
    if not path.is_file():
        raise ValueError(f"{name} does not exist: {path}")
    return path


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{name} must be an object")
    return value


def require_object_field(parent: dict[str, Any], field: str) -> dict[str, Any]:
    return require_object(parent.get(field), field)


def require_list(parent: dict[str, Any], field: str) -> list[Any]:
    value = parent.get(field)
    if not isinstance(value, list):
        raise TypeError(f"{field} must be an array")
    return value


def require_equal(parent: dict[str, Any], field: str, expected: Any) -> None:
    value = parent.get(field)
    if value != expected:
        raise ValueError(f"{field} must be {expected!r}, got {value!r}")


def require_bool(parent: dict[str, Any], field: str) -> bool:
    value = parent.get(field)
    if not isinstance(value, bool):
        raise TypeError(f"{field} must be a boolean")
    return value


def require_int(parent: dict[str, Any], field: str) -> int:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(f"{field} must be an integer")
    return value


def require_string_list(parent: dict[str, Any], field: str) -> list[str]:
    value = parent.get(field)
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        raise TypeError(f"{field} must be an array of strings")
    return value


def require_number_above(parent: dict[str, Any], field: str, minimum: float) -> float:
    value = parent.get(field)
    if not isinstance(value, (int, float)) or isinstance(value, bool) or value <= minimum:
        raise ValueError(f"{field} must be greater than {minimum}, got {value!r}")
    return float(value)


def require_metric_above(parent: dict[str, Any], path: tuple[str, str], minimum: float) -> None:
    value: Any = parent
    for key in path:
        value = require_object(value, ".".join(path)).get(key)
    if not isinstance(value, (int, float)) or isinstance(value, bool) or value <= minimum:
        joined = ".".join(path)
        raise ValueError(f"{joined} must be greater than {minimum}, got {value!r}")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def sha256_text(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def canonical_json(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


if __name__ == "__main__":
    raise SystemExit(main())

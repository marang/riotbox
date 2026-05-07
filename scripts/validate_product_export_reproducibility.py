#!/usr/bin/env python3
"""Validate a bounded Riotbox product-export reproducibility seam.

The current closest product-facing export boundary is the Feral grid pack's
generated-support full mix plus its listening manifest. This validator compares
two independently rendered pack manifests after normalizing temp paths away and
requiring stable audio artifact hashes.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.product_export_reproducibility.v1"
SCHEMA_VERSION = 1
DEFAULT_EXPORT_ROLE = "full_grid_mix"


def main() -> int:
    args = parse_args()
    try:
        proof = build_reproducibility_proof(
            args.manifest_a,
            args.manifest_b,
            args.export_role,
        )
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid product export reproducibility proof: {error}", file=sys.stderr)
        return 1

    if args.write_proof is not None:
        args.write_proof.write_text(json.dumps(proof, indent=2, sort_keys=True) + "\n")

    print(
        "product export reproducibility ok: "
        f"{proof['export_role']} {proof['export_sha256']} "
        f"manifest {proof['normalized_manifest_sha256']}"
    )
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate normalized reproducibility for a Riotbox export boundary."
    )
    parser.add_argument(
        "--export-role",
        default=DEFAULT_EXPORT_ROLE,
        help=f"manifest artifact role to treat as the product export (default: {DEFAULT_EXPORT_ROLE})",
    )
    parser.add_argument(
        "--write-proof",
        type=Path,
        help="optional path for the normalized reproducibility proof JSON",
    )
    parser.add_argument("manifest_a", type=Path)
    parser.add_argument("manifest_b", type=Path)
    return parser.parse_args()


def build_reproducibility_proof(
    manifest_a_path: Path,
    manifest_b_path: Path,
    export_role: str,
) -> dict[str, Any]:
    normalized_a = normalize_manifest(read_json(manifest_a_path), manifest_a_path, export_role)
    normalized_b = normalize_manifest(read_json(manifest_b_path), manifest_b_path, export_role)

    if normalized_a != normalized_b:
        raise ValueError("normalized export manifests differ")

    export_hash = normalized_a["audio_artifact_sha256"][export_role]
    return {
        "schema": SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "boundary": "feral-grid generated-support export",
        "pack_id": normalized_a["pack_id"],
        "export_role": export_role,
        "export_artifact": normalized_a["export_artifact"],
        "source_sha256": normalized_a["source_sha256"],
        "export_sha256": export_hash,
        "normalized_manifest_sha256": sha256_text(canonical_json(normalized_a)),
        "audio_artifact_sha256": normalized_a["audio_artifact_sha256"],
    }


def normalize_manifest(
    manifest: dict[str, Any],
    manifest_path: Path,
    export_role: str,
) -> dict[str, Any]:
    require_equal(manifest, "schema_version", 1)
    require_equal(manifest, "result", "pass")
    require_equal(manifest, "pack_id", "feral-grid-demo")

    manifest_dir = manifest_path.parent
    source_path = require_existing_file(Path(require_string(manifest, "source")), "source")
    artifacts = require_list(manifest, "artifacts")
    export_artifact = find_artifact(artifacts, export_role)
    require_equal(export_artifact, "kind", "audio_wav")

    normalized_artifacts: list[dict[str, Any]] = []
    audio_hashes: dict[str, str] = {}
    for index, raw_artifact in enumerate(artifacts):
        artifact = require_object(raw_artifact, f"artifact {index}")
        role = require_string(artifact, "role", "artifact role")
        kind = require_string(artifact, "kind", f"{role} kind")
        artifact_path = resolve_manifest_path(
            manifest_dir,
            Path(require_string(artifact, "path", f"{role} path")),
        )
        require_existing_file(artifact_path, f"{role} artifact")
        metrics_path = artifact.get("metrics_path")
        normalized_metrics_path = None
        if metrics_path is not None:
            if not isinstance(metrics_path, str) or not metrics_path.strip():
                raise TypeError(f"{role} metrics_path must be a non-empty string or null")
            metrics_artifact_path = resolve_manifest_path(manifest_dir, Path(metrics_path))
            require_existing_file(metrics_artifact_path, f"{role} metrics")
            normalized_metrics_path = pack_path(manifest_dir, metrics_artifact_path)

        record: dict[str, Any] = {
            "role": role,
            "kind": kind,
            "path": pack_path(manifest_dir, artifact_path),
            "metrics_path": normalized_metrics_path,
        }
        if kind == "audio_wav":
            audio_hashes[role] = sha256_file(artifact_path)
            record["sha256"] = audio_hashes[role]
        normalized_artifacts.append(record)

    if export_role not in audio_hashes:
        raise ValueError(f"export role is not an audio artifact: {export_role}")

    scorecard = require_object_field(manifest, "feral_scorecard")
    require_equal(scorecard, "readiness", "ready")
    require_equal(scorecard, "source_backed", True)
    require_equal(scorecard, "generated", True)
    require_equal(scorecard, "fallback_like", False)

    metrics = require_object_field(manifest, "metrics")
    full_grid_metrics = require_object_field(metrics, "full_grid_mix")
    require_metric_above(full_grid_metrics, ("signal", "rms"), 0.000001)
    require_metric_above(full_grid_metrics, ("low_band", "rms"), 0.000001)

    return {
        "pack_id": manifest["pack_id"],
        "source_sha256": sha256_file(source_path),
        "sample_rate": manifest.get("sample_rate"),
        "channel_count": manifest.get("channel_count"),
        "bpm": manifest.get("bpm"),
        "beats_per_bar": manifest.get("beats_per_bar"),
        "bars": manifest.get("bars"),
        "total_beats": manifest.get("total_beats"),
        "total_frames": manifest.get("total_frames"),
        "duration_seconds": manifest.get("duration_seconds"),
        "source_start_seconds": manifest.get("source_start_seconds"),
        "source_window_seconds": manifest.get("source_window_seconds"),
        "export_artifact": pack_path(
            manifest_dir,
            resolve_manifest_path(manifest_dir, Path(export_artifact["path"])),
        ),
        "artifacts": sorted(normalized_artifacts, key=lambda item: item["role"]),
        "audio_artifact_sha256": dict(sorted(audio_hashes.items())),
        "feral_scorecard": scorecard,
        "thresholds": manifest.get("thresholds"),
        "metrics": metrics,
    }


def read_json(path: Path) -> dict[str, Any]:
    return require_object(json.loads(path.read_text()), str(path))


def find_artifact(artifacts: list[Any], role: str) -> dict[str, Any]:
    matches = [
        require_object(artifact, f"artifact {index}")
        for index, artifact in enumerate(artifacts)
        if isinstance(artifact, dict) and artifact.get("role") == role
    ]
    if len(matches) != 1:
        raise ValueError(f"expected exactly one artifact with role {role}, found {len(matches)}")
    return matches[0]


def resolve_manifest_path(manifest_dir: Path, path: Path) -> Path:
    return path if path.is_absolute() else manifest_dir / path


def pack_path(manifest_dir: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(manifest_dir.resolve()).as_posix()
    except ValueError:
        return path.name


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


def require_string(parent: dict[str, Any], field: str, name: str | None = None) -> str:
    value = parent.get(field)
    if not isinstance(value, str) or not value.strip():
        raise TypeError(f"{name or field} must be a non-empty string")
    return value


def require_equal(parent: dict[str, Any], field: str, expected: Any) -> None:
    value = parent.get(field)
    if value != expected:
        raise ValueError(f"{field} must be {expected!r}, got {value!r}")


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

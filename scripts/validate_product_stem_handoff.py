#!/usr/bin/env python3
"""Build, stage, and validate the bounded product-stem handoff."""

from __future__ import annotations

import argparse
import json
import math
import shutil
import struct
import sys
import wave
from pathlib import Path, PurePosixPath
from typing import Any

try:
    from validate_product_export_reproducibility import (
        canonical_json,
        normalize_manifest,
        read_json,
        sha256_file,
        sha256_text,
    )
except ModuleNotFoundError:
    from scripts.validate_product_export_reproducibility import (
        canonical_json,
        normalize_manifest,
        read_json,
        sha256_file,
        sha256_text,
    )


SCHEMA = "riotbox.product_stem_handoff.v1"
SCHEMA_VERSION = 1
PROOF_FILE = "product_stem_handoff_proof.json"
BOUNDARY = "feral-grid generated-support product stems"
MATERIAL_STATUS = "development_only"
RECONSTRUCTION_SCHEMA = "riotbox.product_stem_reconstruction.v1"
RECONSTRUCTION_RULE = "pcm_sum_v1"
EXPECTED_LIMITATION = "mc202_primitive_renderer_non_product"
EXPECTED_PACK_ID = "feral-grid-demo"
PCM16_DECODE_SCALE = 32_767.0
PCM_MAX_ABS_ERROR = 3.0 / 32_768.0
PCM_MAX_RMS_ERROR = 1.5 / 32_768.0
FLOAT_CONTRACT_ABS_TOLERANCE = 1.0e-12
DECLARED_METRIC_ABS_TOLERANCE = 1.0e-7

ARTIFACT_CONTRACT = (
    ("stem_drums", "product_stem_drums", "stems/stem_drums.wav", "source_derived"),
    ("stem_music", "product_stem_music", "stems/stem_music.wav", "source_derived"),
    (
        "stem_bass",
        "product_stem_bass",
        "stems/stem_bass.wav",
        "primitive_renderer",
    ),
    ("full_grid_mix", "full_grid_mix", "full_grid_mix.wav", "composite"),
)


def main() -> int:
    args = parse_args()
    try:
        if args.command == "build":
            proof = build_proof(args.manifest_a, args.manifest_b)
            args.write_proof.write_text(json.dumps(proof, indent=2, sort_keys=True) + "\n")
            print(f"product stem handoff proof ready: {args.write_proof}")
        elif args.command == "stage":
            stage_bundle(args.proof, args.manifest, args.destination)
            print(f"product stem handoff staged: {args.destination}")
        else:
            validate_published_proof(args.proof)
            print(f"valid {SCHEMA} bundle: {args.proof.parent}")
    except (OSError, ValueError, TypeError, wave.Error) as error:
        print(f"invalid product stem handoff: {error}", file=sys.stderr)
        return 1
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    build = subparsers.add_parser("build", help="build proof from two independent renders")
    build.add_argument("--write-proof", type=Path, required=True)
    build.add_argument("manifest_a", type=Path)
    build.add_argument("manifest_b", type=Path)

    stage = subparsers.add_parser("stage", help="copy the proven files into an empty stage")
    stage.add_argument("--proof", type=Path, required=True)
    stage.add_argument("--manifest", type=Path, required=True)
    stage.add_argument("--destination", type=Path, required=True)

    validate = subparsers.add_parser("validate", help="validate a published proof bundle")
    validate.add_argument("proof", type=Path)
    return parser.parse_args()


def build_proof(manifest_a_path: Path, manifest_b_path: Path) -> dict[str, Any]:
    manifest_a = read_json(manifest_a_path)
    manifest_b = read_json(manifest_b_path)
    normalized_a = normalize_manifest(manifest_a, manifest_a_path, "full_grid_mix")
    normalized_b = normalize_manifest(manifest_b, manifest_b_path, "full_grid_mix")
    if normalized_a != normalized_b:
        raise ValueError("normalized product-stem render manifests differ")

    artifacts_by_role = {
        artifact["role"]: artifact
        for artifact in normalized_a["artifacts"]
        if artifact.get("kind") == "audio_wav"
    }
    artifacts = []
    for role, source_role, path, origin in ARTIFACT_CONTRACT:
        source_artifact = artifacts_by_role.get(source_role)
        if source_artifact is None:
            raise ValueError(f"missing product-stem render artifact {source_role}")
        artifact_hash = source_artifact.get("sha256")
        if not is_sha256(artifact_hash):
            raise ValueError(f"invalid SHA-256 for product-stem artifact {source_role}")
        artifacts.append(
            {
                "role": role,
                "source_role": source_role,
                "path": path,
                "media_type": "audio/wav",
                "sha256": artifact_hash,
                "origin": origin,
            }
        )

    reconstruction = require_reconstruction(normalized_a)
    primitive_boundary = require_primitive_boundary(manifest_a)
    if primitive_boundary != require_primitive_boundary(manifest_b):
        raise ValueError("primitive renderer boundaries differ across renders")
    grid = {
        "sample_rate_hz": require_positive_int(normalized_a, "sample_rate"),
        "channel_count": require_positive_int(normalized_a, "channel_count"),
        "bpm": require_positive_number(normalized_a, "bpm"),
        "beats_per_bar": require_positive_int(normalized_a, "beats_per_bar"),
        "bars": require_positive_int(normalized_a, "bars"),
        "total_beats": require_positive_int(normalized_a, "total_beats"),
        "frame_count": require_positive_int(normalized_a, "total_frames"),
        "duration_seconds": require_positive_number(normalized_a, "duration_seconds"),
    }

    return {
        "schema": SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "boundary": BOUNDARY,
        "pack_id": normalized_a["pack_id"],
        "material_status": MATERIAL_STATUS,
        "release_ready": False,
        "musician_export_action_ready": False,
        "source_sha256": normalized_a["source_sha256"],
        "normalized_manifest_sha256": sha256_text(canonical_json(normalized_a)),
        "grid": grid,
        "artifacts": artifacts,
        "reconstruction": reconstruction,
        "renderer_status": {
            "primitive_renderer_boundary": primitive_boundary,
            "limitations": [EXPECTED_LIMITATION],
        },
    }


def require_reconstruction(normalized: dict[str, Any]) -> dict[str, Any]:
    metrics = require_object(normalized.get("metrics"), "metrics")
    value = require_object(metrics.get("product_stem_reconstruction"), "reconstruction")
    if value.get("schema") != RECONSTRUCTION_SCHEMA:
        raise ValueError("product stem reconstruction schema mismatch")
    if value.get("rule") != RECONSTRUCTION_RULE or value.get("passed") is not True:
        raise ValueError("product stem reconstruction did not pass pcm_sum_v1")
    for field in ("sample_rate_hz", "channel_count", "frame_count"):
        require_positive_int(value, field)
    for field in (
        "max_abs_error",
        "rms_error",
        "max_allowed_abs_error",
        "max_allowed_rms_error",
    ):
        require_non_negative_number(value, field)
    require_frozen_reconstruction_tolerances(value)
    if value["max_abs_error"] > value["max_allowed_abs_error"]:
        raise ValueError("product stem maximum reconstruction error exceeds tolerance")
    if value["rms_error"] > value["max_allowed_rms_error"]:
        raise ValueError("product stem RMS reconstruction error exceeds tolerance")
    return dict(value)


def require_primitive_boundary(normalized: dict[str, Any]) -> dict[str, Any]:
    value = require_object(normalized.get("primitive_renderer_boundary"), "primitive boundary")
    expected = {
        "schema": "riotbox.primitive_renderer_boundary.v1",
        "evidence_role": "non_product_diagnostic_control",
        "product_output_allowed": False,
        "quality_proof": False,
        "demo_readiness": "unverified",
        "promotion_blocked": True,
    }
    for field, expected_value in expected.items():
        if value.get(field) != expected_value:
            raise ValueError(f"primitive renderer boundary {field} must be {expected_value!r}")
    affected = value.get("affected_paths")
    if affected != ["metrics.mc202_bass_pressure.pattern_origin"]:
        raise ValueError("primitive renderer affected path mismatch")
    return dict(value)


def stage_bundle(proof_path: Path, manifest_path: Path, destination: Path) -> None:
    proof = read_json(proof_path)
    validate_proof_contract(proof)
    if not destination.is_dir() or any(destination.iterdir()):
        raise ValueError("product stem handoff stage must be an existing empty directory")

    manifest = read_json(manifest_path)
    normalized_manifest = normalize_manifest(manifest, manifest_path, "full_grid_mix")
    normalized_manifest_sha256 = sha256_text(canonical_json(normalized_manifest))
    if normalized_manifest_sha256 != proof["normalized_manifest_sha256"]:
        raise ValueError("staging manifest does not match the proven normalized manifest")
    if normalized_manifest["source_sha256"] != proof["source_sha256"]:
        raise ValueError("staging manifest source does not match the proven source")
    manifest_dir = manifest_path.parent.resolve()
    manifest_artifacts = {
        artifact.get("role"): artifact
        for artifact in require_list(manifest.get("artifacts"), "manifest artifacts")
        if isinstance(artifact, dict)
    }
    for artifact in proof["artifacts"]:
        source_role = artifact["source_role"]
        source_record = require_object(
            manifest_artifacts.get(source_role), f"manifest artifact {source_role}"
        )
        source_path = Path(require_string(source_record, "path"))
        if not source_path.is_absolute():
            source_path = manifest_dir / source_path
        source_path = source_path.resolve()
        try:
            source_path.relative_to(manifest_dir)
        except ValueError as error:
            raise ValueError(f"source artifact escapes render directory: {source_path}") from error
        if sha256_file(source_path) != artifact["sha256"]:
            raise ValueError(f"source artifact hash drifted for {source_role}")
        destination_path = contained_path(destination, artifact["path"])
        destination_path.parent.mkdir(parents=True, exist_ok=True)
        if destination_path.exists():
            raise ValueError(f"staged artifact already exists: {destination_path}")
        shutil.copyfile(source_path, destination_path)

    published_proof = destination / PROOF_FILE
    if published_proof.exists():
        raise ValueError(f"staged proof already exists: {published_proof}")
    shutil.copyfile(proof_path, published_proof)
    validate_published_proof(published_proof)


def validate_published_proof(proof_path: Path) -> None:
    proof = read_json(proof_path)
    validate_proof_contract(proof)
    bundle_dir = proof_path.parent
    decoded = {}
    for artifact in proof["artifacts"]:
        artifact_path = contained_path(bundle_dir, artifact["path"])
        if artifact_path.is_symlink():
            raise ValueError(
                f"product stem artifact must be a regular contained file: {artifact_path}"
            )
        if not artifact_path.is_file():
            raise ValueError(f"missing product stem artifact {artifact['role']}: {artifact_path}")
        actual_hash = sha256_file(artifact_path)
        if actual_hash != artifact["sha256"]:
            raise ValueError(
                f"product stem artifact hash mismatch for {artifact['role']}: "
                f"proof {artifact['sha256']} actual {actual_hash}"
            )
        decoded[artifact["role"]] = read_pcm16_wav(artifact_path)

    grid = proof["grid"]
    expected_format = (
        grid["sample_rate_hz"],
        grid["channel_count"],
        grid["frame_count"],
    )
    for role, audio in decoded.items():
        if audio[:3] != expected_format:
            raise ValueError(
                f"{role} format/grid mismatch: expected {expected_format!r}, got {audio[:3]!r}"
            )

    drums = decoded["stem_drums"][3]
    music = decoded["stem_music"][3]
    bass = decoded["stem_bass"][3]
    full_mix = decoded["full_grid_mix"][3]
    errors = [drums[i] + music[i] + bass[i] - full_mix[i] for i in range(len(full_mix))]
    max_abs_error = max((abs(value) for value in errors), default=0.0) / PCM16_DECODE_SCALE
    rms_error = (
        math.sqrt(sum(value * value for value in errors) / max(len(errors), 1))
        / PCM16_DECODE_SCALE
    )
    if max_abs_error > PCM_MAX_ABS_ERROR:
        raise ValueError("published product stems exceed maximum reconstruction tolerance")
    if rms_error > PCM_MAX_RMS_ERROR:
        raise ValueError("published product stems exceed RMS reconstruction tolerance")
    reconstruction = proof["reconstruction"]
    if not math.isclose(
        max_abs_error,
        reconstruction["max_abs_error"],
        rel_tol=0.0,
        abs_tol=DECLARED_METRIC_ABS_TOLERANCE,
    ):
        raise ValueError("declared maximum reconstruction error does not match published audio")
    if not math.isclose(
        rms_error,
        reconstruction["rms_error"],
        rel_tol=0.0,
        abs_tol=DECLARED_METRIC_ABS_TOLERANCE,
    ):
        raise ValueError("declared RMS reconstruction error does not match published audio")


def validate_proof_contract(proof: dict[str, Any]) -> None:
    if proof.get("schema") != SCHEMA or proof.get("schema_version") != SCHEMA_VERSION:
        raise ValueError("product stem handoff schema mismatch")
    if proof.get("boundary") != BOUNDARY or proof.get("material_status") != MATERIAL_STATUS:
        raise ValueError("product stem handoff boundary/status mismatch")
    if proof.get("pack_id") != EXPECTED_PACK_ID:
        raise ValueError(f"product stem handoff pack_id must be {EXPECTED_PACK_ID!r}")
    if proof.get("release_ready") is not False or proof.get("musician_export_action_ready") is not False:
        raise ValueError("development product stem handoff cannot claim musician/release readiness")
    if not is_sha256(proof.get("source_sha256")):
        raise ValueError("product stem handoff source_sha256 must be lowercase SHA-256")
    if not is_sha256(proof.get("normalized_manifest_sha256")):
        raise ValueError("product stem normalized manifest hash must be lowercase SHA-256")

    grid = require_object(proof.get("grid"), "grid")
    for field in ("sample_rate_hz", "channel_count", "beats_per_bar", "bars", "total_beats", "frame_count"):
        require_positive_int(grid, field)
    require_positive_number(grid, "bpm")
    require_positive_number(grid, "duration_seconds")
    if grid["total_beats"] != grid["beats_per_bar"] * grid["bars"]:
        raise ValueError("product stem grid beat/bar identity mismatch")
    expected_frames = round(
        grid["total_beats"] * grid["sample_rate_hz"] * 60.0 / grid["bpm"]
    )
    if abs(grid["frame_count"] - expected_frames) > 1:
        raise ValueError("product stem grid frame/tempo identity mismatch")
    expected_duration = grid["frame_count"] / grid["sample_rate_hz"]
    if abs(grid["duration_seconds"] - expected_duration) > 1.0 / grid["sample_rate_hz"]:
        raise ValueError("product stem grid duration identity mismatch")

    artifacts = require_list(proof.get("artifacts"), "artifacts")
    expected = {item[0]: item for item in ARTIFACT_CONTRACT}
    if len(artifacts) != len(expected):
        raise ValueError("product stem handoff must contain exactly four audio artifacts")
    seen_roles: set[str] = set()
    seen_paths: set[str] = set()
    for raw_artifact in artifacts:
        artifact = require_object(raw_artifact, "artifact")
        role = require_string(artifact, "role")
        contract = expected.get(role)
        if contract is None or role in seen_roles:
            raise ValueError(f"unexpected or duplicate product stem role {role}")
        _, source_role, path, origin = contract
        if artifact.get("source_role") != source_role or artifact.get("origin") != origin:
            raise ValueError(f"product stem role/source/origin mismatch for {role}")
        if artifact.get("media_type") != "audio/wav" or artifact.get("path") != path:
            raise ValueError(f"product stem media/path mismatch for {role}")
        if not is_sha256(artifact.get("sha256")):
            raise ValueError(f"invalid product stem artifact SHA-256 for {role}")
        contained_relative_path(path)
        if path in seen_paths:
            raise ValueError(f"duplicate product stem artifact path {path}")
        seen_roles.add(role)
        seen_paths.add(path)

    reconstruction = require_object(proof.get("reconstruction"), "reconstruction")
    if reconstruction.get("schema") != RECONSTRUCTION_SCHEMA:
        raise ValueError("product stem reconstruction schema mismatch")
    if reconstruction.get("rule") != RECONSTRUCTION_RULE or reconstruction.get("passed") is not True:
        raise ValueError("product stem reconstruction contract did not pass")
    if reconstruction.get("sample_rate_hz") != grid["sample_rate_hz"]:
        raise ValueError("product stem reconstruction sample rate mismatch")
    if reconstruction.get("channel_count") != grid["channel_count"]:
        raise ValueError("product stem reconstruction channel count mismatch")
    if reconstruction.get("frame_count") != grid["frame_count"]:
        raise ValueError("product stem reconstruction frame count mismatch")
    for field in ("max_abs_error", "rms_error", "max_allowed_abs_error", "max_allowed_rms_error"):
        require_non_negative_number(reconstruction, field)
    require_frozen_reconstruction_tolerances(reconstruction)
    if reconstruction["max_abs_error"] > reconstruction["max_allowed_abs_error"]:
        raise ValueError("declared maximum reconstruction error exceeds tolerance")
    if reconstruction["rms_error"] > reconstruction["max_allowed_rms_error"]:
        raise ValueError("declared RMS reconstruction error exceeds tolerance")

    renderer_status = require_object(proof.get("renderer_status"), "renderer status")
    boundary = require_object(
        renderer_status.get("primitive_renderer_boundary"), "primitive renderer boundary"
    )
    require_primitive_boundary({"primitive_renderer_boundary": boundary})
    if renderer_status.get("limitations") != [EXPECTED_LIMITATION]:
        raise ValueError("product stem renderer limitations mismatch")


def require_frozen_reconstruction_tolerances(reconstruction: dict[str, Any]) -> None:
    expected = {
        "max_allowed_abs_error": PCM_MAX_ABS_ERROR,
        "max_allowed_rms_error": PCM_MAX_RMS_ERROR,
    }
    for field, expected_value in expected.items():
        if not math.isclose(
            reconstruction[field],
            expected_value,
            rel_tol=0.0,
            abs_tol=FLOAT_CONTRACT_ABS_TOLERANCE,
        ):
            raise ValueError(f"product stem reconstruction {field} is not the frozen tolerance")


def read_pcm16_wav(path: Path) -> tuple[int, int, int, tuple[int, ...]]:
    with wave.open(str(path), "rb") as handle:
        if handle.getsampwidth() != 2 or handle.getcomptype() != "NONE":
            raise ValueError(f"product stem WAV must be uncompressed PCM16: {path}")
        sample_rate = handle.getframerate()
        channel_count = handle.getnchannels()
        frame_count = handle.getnframes()
        payload = handle.readframes(frame_count)
    sample_count = frame_count * channel_count
    samples = struct.unpack(f"<{sample_count}h", payload)
    return sample_rate, channel_count, frame_count, samples


def contained_path(root: Path, value: str) -> Path:
    relative = contained_relative_path(value)
    return root.joinpath(*relative.parts)


def contained_relative_path(value: str) -> PurePosixPath:
    path = PurePosixPath(value)
    if path.is_absolute() or ".." in path.parts or not path.parts:
        raise ValueError(f"product stem artifact must be a contained relative path: {value}")
    return path


def is_sha256(value: Any) -> bool:
    return isinstance(value, str) and len(value) == 64 and all(
        character in "0123456789abcdef" for character in value
    )


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{name} must be an object")
    return value


def require_list(value: Any, name: str) -> list[Any]:
    if not isinstance(value, list):
        raise TypeError(f"{name} must be an array")
    return value


def require_string(parent: dict[str, Any], field: str) -> str:
    value = parent.get(field)
    if not isinstance(value, str) or not value.strip():
        raise TypeError(f"{field} must be a non-empty string")
    return value


def require_positive_int(parent: dict[str, Any], field: str) -> int:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        raise TypeError(f"{field} must be a positive integer")
    return value


def require_positive_number(parent: dict[str, Any], field: str) -> float | int:
    value = parent.get(field)
    if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value) or value <= 0:
        raise TypeError(f"{field} must be a positive finite number")
    return value


def require_non_negative_number(parent: dict[str, Any], field: str) -> float | int:
    value = parent.get(field)
    if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value) or value < 0:
        raise TypeError(f"{field} must be a non-negative finite number")
    return value


if __name__ == "__main__":
    raise SystemExit(main())

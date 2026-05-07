#!/usr/bin/env python3
"""Validate source diversity across Riotbox source-showcase listening packs."""

from __future__ import annotations

import hashlib
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


GENERATED_ROLE_HINTS = ("tr909", "mc202", "generated", "kick_bass")
SOURCE_BACKED_ROLE_HINTS = ("source", "w30", "capture", "slice", "chop")
FULL_MIX_ROLE_HINTS = ("full", "mix")
DEFAULT_MAX_IDENTICAL_GENERATED_TO_SOURCE_RMS_RATIO = 0.75


@dataclass(frozen=True)
class ArtifactEvidence:
    role: str
    path: Path
    sha256: str
    signal_rms: float | None
    low_band_rms: float | None


@dataclass(frozen=True)
class PackEvidence:
    manifest_path: Path
    source: str
    artifacts: tuple[ArtifactEvidence, ...]


def main() -> int:
    try:
        args = parse_args(sys.argv[1:])
        if args.show_help:
            print(usage())
            return 0
        packs = tuple(read_pack(path) for path in args.manifest_paths)
        validate_diversity(packs, args.max_generated_to_source_rms_ratio)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid source-showcase diversity: {error}", file=sys.stderr)
        return 1

    print(f"valid source-showcase diversity across {len(packs)} manifests")
    return 0


@dataclass(frozen=True)
class Args:
    manifest_paths: tuple[Path, ...]
    max_generated_to_source_rms_ratio: float
    show_help: bool = False


def parse_args(args: list[str]) -> Args:
    manifest_paths: list[Path] = []
    max_ratio = DEFAULT_MAX_IDENTICAL_GENERATED_TO_SOURCE_RMS_RATIO
    index = 0

    while index < len(args):
        arg = args[index]
        if arg == "--max-generated-to-source-rms-ratio":
            index += 1
            if index >= len(args):
                raise ValueError("--max-generated-to-source-rms-ratio requires a value")
            max_ratio = parse_positive_float(
                "--max-generated-to-source-rms-ratio",
                args[index],
            )
        elif arg in {"--help", "-h"}:
            return Args((), max_ratio, show_help=True)
        elif arg.startswith("-"):
            raise ValueError(f"unknown option: {arg}\n{usage()}")
        else:
            manifest_paths.append(resolve_manifest_path(Path(arg)))
        index += 1

    if len(manifest_paths) < 2:
        raise ValueError(f"expected at least two manifests or pack directories\n{usage()}")

    return Args(tuple(manifest_paths), max_ratio)


def usage() -> str:
    return (
        "usage: validate_source_showcase_diversity.py "
        "[--max-generated-to-source-rms-ratio RATIO] <manifest-or-pack-dir>..."
    )


def parse_positive_float(name: str, value: str) -> float:
    try:
        parsed = float(value)
    except ValueError as error:
        raise ValueError(f"{name} must be a positive number") from error
    if not parsed > 0.0:
        raise ValueError(f"{name} must be a positive number")
    return parsed


def resolve_manifest_path(path: Path) -> Path:
    if path.is_dir():
        return path / "manifest.json"
    return path


def read_pack(manifest_path: Path) -> PackEvidence:
    manifest = json.loads(manifest_path.read_text())
    source = require_string(manifest, "source")
    artifacts = tuple(read_artifacts(manifest, manifest_path.parent))
    if not artifacts:
        raise ValueError(f"{manifest_path}: no audio artifacts")
    return PackEvidence(manifest_path, source, artifacts)


def read_artifacts(manifest: dict[str, Any], manifest_dir: Path) -> list[ArtifactEvidence]:
    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, list):
        raise ValueError("manifest artifacts must be an array")

    evidence = []
    for index, artifact in enumerate(artifacts):
        if not isinstance(artifact, dict):
            raise ValueError(f"artifact {index} must be an object")
        kind = require_string(artifact, "kind")
        if kind != "audio_wav":
            continue
        role = require_string(artifact, "role")
        artifact_path = resolve_artifact_path(manifest_dir, require_string(artifact, "path"))
        metric = manifest.get("metrics", {}).get(role, {})
        evidence.append(
            ArtifactEvidence(
                role=role,
                path=artifact_path,
                sha256=sha256_file(artifact_path),
                signal_rms=metric_float(metric, "signal", "rms"),
                low_band_rms=metric_float(metric, "low_band", "rms"),
            )
        )
    return evidence


def resolve_artifact_path(manifest_dir: Path, artifact_path: str) -> Path:
    path = Path(artifact_path)
    if not path.is_absolute():
        path = manifest_dir / path
    if not path.is_file():
        raise ValueError(f"artifact path does not exist: {path}")
    return path


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as file:
        for chunk in iter(lambda: file.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def metric_float(metric: Any, group: str, field: str) -> float | None:
    if not isinstance(metric, dict):
        return None
    group_value = metric.get(group)
    if not isinstance(group_value, dict):
        return None
    value = group_value.get(field)
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        return float(value)
    return None


def validate_diversity(
    packs: tuple[PackEvidence, ...],
    max_generated_to_source_rms_ratio: float,
) -> None:
    sources = {pack.source for pack in packs}
    if len(sources) < 2:
        raise ValueError("source showcase needs at least two distinct source values")

    failures: list[str] = []
    failures.extend(full_mix_hash_failures(packs))
    failures.extend(source_backed_hash_failures(packs))
    failures.extend(generated_dominance_failures(packs, max_generated_to_source_rms_ratio))

    if failures:
        raise ValueError("; ".join(failures))


def full_mix_hash_failures(packs: tuple[PackEvidence, ...]) -> list[str]:
    failures = []
    for role, artifacts in artifacts_by_role(packs).items():
        if not is_full_mix_role(role):
            continue
        for sha256, duplicate_artifacts in group_by_hash(artifacts).items():
            sources = {source for source, _ in duplicate_artifacts}
            if len(sources) > 1:
                failures.append(
                    f"{role} has identical hash {short_hash(sha256)} across sources "
                    f"{sorted(sources)}"
                )
    return failures


def source_backed_hash_failures(packs: tuple[PackEvidence, ...]) -> list[str]:
    failures = []
    for role, artifacts in artifacts_by_role(packs).items():
        if not is_source_backed_role(role):
            continue
        for sha256, duplicate_artifacts in group_by_hash(artifacts).items():
            sources = {source for source, _ in duplicate_artifacts}
            if len(sources) > 1:
                failures.append(
                    f"source-backed {role} has identical hash {short_hash(sha256)} "
                    f"across sources {sorted(sources)}"
                )
    return failures


def generated_dominance_failures(
    packs: tuple[PackEvidence, ...],
    max_generated_to_source_rms_ratio: float,
) -> list[str]:
    failures = []
    for role, artifacts in artifacts_by_role(packs).items():
        if not is_generated_role(role):
            continue
        for sha256, duplicate_artifacts in group_by_hash(artifacts).items():
            sources = {source for source, _ in duplicate_artifacts}
            if len(sources) < 2:
                continue
            for source, artifact in duplicate_artifacts:
                source_rms = max_source_backed_rms_for_source(packs, source)
                if source_rms is None:
                    failures.append(
                        f"{role} is identical across sources but source-backed RMS is missing "
                        f"for {source}"
                    )
                    continue
                if artifact.signal_rms is None:
                    failures.append(f"{role} RMS is missing for {source}")
                    continue
                failures.extend(
                    dominance_metric_failures(
                        role,
                        sha256,
                        source,
                        "signal",
                        artifact.signal_rms,
                        source_rms,
                        max_generated_to_source_rms_ratio,
                    )
                )
                source_low_band_rms = max_source_backed_low_band_rms_for_source(packs, source)
                if artifact.low_band_rms is not None and source_low_band_rms is not None:
                    failures.extend(
                        dominance_metric_failures(
                            role,
                            sha256,
                            source,
                            "low-band",
                            artifact.low_band_rms,
                            source_low_band_rms,
                            max_generated_to_source_rms_ratio,
                        )
                    )
    return failures


def dominance_metric_failures(
    role: str,
    sha256: str,
    source: str,
    metric_name: str,
    generated_rms: float,
    source_rms: float,
    max_generated_to_source_rms_ratio: float,
) -> list[str]:
    ratio = generated_rms / max(source_rms, 1e-12)
    if ratio < max_generated_to_source_rms_ratio:
        return []
    return [
        f"{role} identical hash {short_hash(sha256)} is too dominant for "
        f"{source}: generated/source {metric_name} RMS ratio {ratio:.3f} >= "
        f"{max_generated_to_source_rms_ratio:.3f}"
    ]


def artifacts_by_role(
    packs: tuple[PackEvidence, ...],
) -> dict[str, list[tuple[str, ArtifactEvidence]]]:
    by_role: dict[str, list[tuple[str, ArtifactEvidence]]] = {}
    for pack in packs:
        for artifact in pack.artifacts:
            by_role.setdefault(artifact.role, []).append((pack.source, artifact))
    return by_role


def group_by_hash(
    artifacts: list[tuple[str, ArtifactEvidence]],
) -> dict[str, list[tuple[str, ArtifactEvidence]]]:
    by_hash: dict[str, list[tuple[str, ArtifactEvidence]]] = {}
    for source, artifact in artifacts:
        by_hash.setdefault(artifact.sha256, []).append((source, artifact))
    return by_hash


def max_source_backed_rms_for_source(
    packs: tuple[PackEvidence, ...],
    source: str,
) -> float | None:
    rms_values = [
        artifact.signal_rms
        for pack in packs
        if pack.source == source
        for artifact in pack.artifacts
        if is_source_backed_role(artifact.role) and artifact.signal_rms is not None
    ]
    return max(rms_values) if rms_values else None


def max_source_backed_low_band_rms_for_source(
    packs: tuple[PackEvidence, ...],
    source: str,
) -> float | None:
    rms_values = [
        artifact.low_band_rms
        for pack in packs
        if pack.source == source
        for artifact in pack.artifacts
        if is_source_backed_role(artifact.role) and artifact.low_band_rms is not None
    ]
    return max(rms_values) if rms_values else None


def is_generated_role(role: str) -> bool:
    normalized = role.lower()
    return any(hint in normalized for hint in GENERATED_ROLE_HINTS)


def is_source_backed_role(role: str) -> bool:
    normalized = role.lower()
    return any(hint in normalized for hint in SOURCE_BACKED_ROLE_HINTS)


def is_full_mix_role(role: str) -> bool:
    normalized = role.lower()
    return all(hint in normalized for hint in FULL_MIX_ROLE_HINTS)


def require_string(parent: dict[str, Any], field: str) -> str:
    value = parent.get(field)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{field} must be a non-empty string")
    return value


def short_hash(sha256: str) -> str:
    return sha256[:12]


if __name__ == "__main__":
    raise SystemExit(main())

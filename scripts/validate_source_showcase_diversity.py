#!/usr/bin/env python3
"""Validate source diversity across Riotbox source-showcase listening packs."""

from __future__ import annotations

import hashlib
import json
import math
import sys
import wave
from array import array
from dataclasses import dataclass
from pathlib import Path
from typing import Any


GENERATED_ROLE_HINTS = ("tr909", "mc202", "generated", "kick_bass")
SOURCE_BACKED_ROLE_HINTS = ("source", "w30", "capture", "slice", "chop")
FULL_MIX_ROLE_HINTS = ("full", "mix")
DEFAULT_MAX_IDENTICAL_GENERATED_TO_SOURCE_RMS_RATIO = 0.75
DEFAULT_MIN_FULL_MIX_NORMALIZED_RMS_DELTA = 0.05
DEFAULT_MAX_FULL_MIX_WAVEFORM_CORRELATION = 0.995
DEFAULT_MIN_FULL_MIX_SPECTRAL_DISTANCE = 0.02
MAX_CORRELATION_SAMPLES = 200_000


@dataclass(frozen=True)
class ArtifactEvidence:
    role: str
    path: Path
    sha256: str
    signal_rms: float | None
    low_band_rms: float | None
    spectral_energy: dict[str, float]


@dataclass(frozen=True)
class PackEvidence:
    manifest_path: Path
    source: str
    artifacts: tuple[ArtifactEvidence, ...]


@dataclass(frozen=True)
class Thresholds:
    max_generated_to_source_rms_ratio: float
    min_full_mix_normalized_rms_delta: float
    max_full_mix_waveform_correlation: float
    min_full_mix_spectral_distance: float


@dataclass(frozen=True)
class Failure:
    code: str
    message: str
    role: str | None = None
    source: str | None = None

    def as_dict(self) -> dict[str, Any]:
        data = {
            "code": self.code,
            "message": self.message,
            "role": self.role,
            "source": self.source,
        }
        return {key: value for key, value in data.items() if value is not None}


def main() -> int:
    try:
        args = parse_args(sys.argv[1:])
        if args.show_help:
            print(usage())
            return 0
        packs = tuple(read_pack(path) for path in args.manifest_paths)
        summary = build_summary(packs, args.thresholds)
        write_optional_reports(summary, args)
        if summary["failures"]:
            raise ValueError("; ".join(failure["message"] for failure in summary["failures"]))
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid source-showcase diversity: {error}", file=sys.stderr)
        return 1

    print(f"valid source-showcase diversity across {len(packs)} manifests")
    return 0


@dataclass(frozen=True)
class Args:
    manifest_paths: tuple[Path, ...]
    thresholds: Thresholds
    json_output: Path | None = None
    markdown_output: Path | None = None
    show_help: bool = False


def parse_args(args: list[str]) -> Args:
    manifest_paths: list[Path] = []
    max_generated_ratio = DEFAULT_MAX_IDENTICAL_GENERATED_TO_SOURCE_RMS_RATIO
    min_full_mix_rms_delta = DEFAULT_MIN_FULL_MIX_NORMALIZED_RMS_DELTA
    max_full_mix_correlation = DEFAULT_MAX_FULL_MIX_WAVEFORM_CORRELATION
    min_full_mix_spectral_distance = DEFAULT_MIN_FULL_MIX_SPECTRAL_DISTANCE
    json_output = None
    markdown_output = None
    index = 0

    while index < len(args):
        arg = args[index]
        if arg == "--max-generated-to-source-rms-ratio":
            index += 1
            if index >= len(args):
                raise ValueError("--max-generated-to-source-rms-ratio requires a value")
            max_generated_ratio = parse_positive_float(
                "--max-generated-to-source-rms-ratio",
                args[index],
            )
        elif arg == "--min-full-mix-normalized-rms-delta":
            index += 1
            if index >= len(args):
                raise ValueError("--min-full-mix-normalized-rms-delta requires a value")
            min_full_mix_rms_delta = parse_positive_float(
                "--min-full-mix-normalized-rms-delta",
                args[index],
            )
        elif arg == "--max-full-mix-waveform-correlation":
            index += 1
            if index >= len(args):
                raise ValueError("--max-full-mix-waveform-correlation requires a value")
            max_full_mix_correlation = parse_positive_float(
                "--max-full-mix-waveform-correlation",
                args[index],
            )
        elif arg == "--min-full-mix-spectral-distance":
            index += 1
            if index >= len(args):
                raise ValueError("--min-full-mix-spectral-distance requires a value")
            min_full_mix_spectral_distance = parse_positive_float(
                "--min-full-mix-spectral-distance",
                args[index],
            )
        elif arg == "--json-output":
            index += 1
            if index >= len(args):
                raise ValueError("--json-output requires a path")
            json_output = Path(args[index])
        elif arg == "--markdown-output":
            index += 1
            if index >= len(args):
                raise ValueError("--markdown-output requires a path")
            markdown_output = Path(args[index])
        elif arg in {"--help", "-h"}:
            return Args((), default_thresholds(), show_help=True)
        elif arg.startswith("-"):
            raise ValueError(f"unknown option: {arg}\n{usage()}")
        else:
            manifest_paths.append(resolve_manifest_path(Path(arg)))
        index += 1

    if len(manifest_paths) < 2:
        raise ValueError(f"expected at least two manifests or pack directories\n{usage()}")

    return Args(
        manifest_paths=tuple(manifest_paths),
        thresholds=Thresholds(
            max_generated_to_source_rms_ratio=max_generated_ratio,
            min_full_mix_normalized_rms_delta=min_full_mix_rms_delta,
            max_full_mix_waveform_correlation=max_full_mix_correlation,
            min_full_mix_spectral_distance=min_full_mix_spectral_distance,
        ),
        json_output=json_output,
        markdown_output=markdown_output,
    )


def usage() -> str:
    return (
        "usage: validate_source_showcase_diversity.py "
        "[--json-output PATH] [--markdown-output PATH] "
        "[--max-generated-to-source-rms-ratio RATIO] "
        "[--min-full-mix-normalized-rms-delta RATIO] "
        "[--max-full-mix-waveform-correlation RATIO] "
        "[--min-full-mix-spectral-distance RATIO] <manifest-or-pack-dir>..."
    )


def default_thresholds() -> Thresholds:
    return Thresholds(
        max_generated_to_source_rms_ratio=DEFAULT_MAX_IDENTICAL_GENERATED_TO_SOURCE_RMS_RATIO,
        min_full_mix_normalized_rms_delta=DEFAULT_MIN_FULL_MIX_NORMALIZED_RMS_DELTA,
        max_full_mix_waveform_correlation=DEFAULT_MAX_FULL_MIX_WAVEFORM_CORRELATION,
        min_full_mix_spectral_distance=DEFAULT_MIN_FULL_MIX_SPECTRAL_DISTANCE,
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
        spectral = manifest.get("metrics", {}).get("spectral_energy", {}).get(role, {})
        evidence.append(
            ArtifactEvidence(
                role=role,
                path=artifact_path,
                sha256=sha256_file(artifact_path),
                signal_rms=metric_float(metric, "signal", "rms"),
                low_band_rms=metric_float(metric, "low_band", "rms"),
                spectral_energy=spectral_energy(spectral),
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


def spectral_energy(metric: Any) -> dict[str, float]:
    if not isinstance(metric, dict):
        return {}
    result = {}
    for field in (
        "low_band_energy_ratio",
        "mid_band_energy_ratio",
        "high_band_energy_ratio",
    ):
        value = metric.get(field)
        if isinstance(value, (int, float)) and not isinstance(value, bool):
            result[field] = float(value)
    return result


def build_summary(packs: tuple[PackEvidence, ...], thresholds: Thresholds) -> dict[str, Any]:
    sources = {pack.source for pack in packs}
    if len(sources) < 2:
        raise ValueError("source showcase needs at least two distinct source values")

    pairwise = pairwise_metrics(packs)
    failures: list[Failure] = []
    failures.extend(full_mix_hash_failures(packs))
    failures.extend(full_mix_similarity_failures(pairwise, thresholds))
    failures.extend(source_backed_hash_failures(packs))
    dominance = generated_dominance_metrics(packs, thresholds)
    failures.extend(failure for failure in (metric.get("failure") for metric in dominance) if failure)

    return {
        "schema": "riotbox.source_showcase_diversity.v1",
        "schema_version": 1,
        "result": "fail" if failures else "pass",
        "thresholds": thresholds.__dict__,
        "sources": sorted(sources),
        "packs": [
            {
                "source": pack.source,
                "manifest_path": str(pack.manifest_path),
                "artifact_count": len(pack.artifacts),
            }
            for pack in packs
        ],
        "role_hash_groups": role_hash_groups(packs),
        "pairwise_role_metrics": pairwise,
        "generated_dominance": [
            {key: value for key, value in metric.items() if key != "failure"}
            for metric in dominance
        ],
        "failures": [failure.as_dict() for failure in failures],
    }


def full_mix_hash_failures(packs: tuple[PackEvidence, ...]) -> list[Failure]:
    failures = []
    for role, artifacts in artifacts_by_role(packs).items():
        if not is_full_mix_role(role):
            continue
        for sha256, duplicate_artifacts in group_by_hash(artifacts).items():
            sources = {source for source, _ in duplicate_artifacts}
            if len(sources) > 1:
                failures.append(
                    Failure(
                        code="full_mix_identical_across_sources",
                        role=role,
                        message=(
                            f"{role} has identical hash {short_hash(sha256)} across sources "
                            f"{sorted(sources)}"
                        ),
                    )
                )
    return failures


def source_backed_hash_failures(packs: tuple[PackEvidence, ...]) -> list[Failure]:
    failures = []
    for role, artifacts in artifacts_by_role(packs).items():
        if not is_source_backed_role(role):
            continue
        for sha256, duplicate_artifacts in group_by_hash(artifacts).items():
            sources = {source for source, _ in duplicate_artifacts}
            if len(sources) > 1:
                failures.append(
                    Failure(
                        code="source_backed_stem_identical_across_sources",
                        role=role,
                        message=(
                            f"source-backed {role} has identical hash {short_hash(sha256)} "
                            f"across sources {sorted(sources)}"
                        ),
                    )
                )
    return failures


def generated_dominance_metrics(
    packs: tuple[PackEvidence, ...],
    thresholds: Thresholds,
) -> list[dict[str, Any]]:
    metrics = []
    for role, artifacts in artifacts_by_role(packs).items():
        if not is_generated_role(role):
            continue
        for sha256, duplicate_artifacts in group_by_hash(artifacts).items():
            sources = {source for source, _ in duplicate_artifacts}
            if len(sources) < 2:
                continue
            for source, artifact in duplicate_artifacts:
                source_rms = max_source_backed_rms_for_source(packs, source)
                source_low_band_rms = max_source_backed_low_band_rms_for_source(packs, source)
                metric = {
                    "source": source,
                    "role": role,
                    "sha256": artifact.sha256,
                    "short_hash": short_hash(artifact.sha256),
                    "identical_across_sources": True,
                    "source_backed_signal_rms": source_rms,
                    "source_backed_low_band_rms": source_low_band_rms,
                    "generated_signal_rms": artifact.signal_rms,
                    "generated_low_band_rms": artifact.low_band_rms,
                    "signal_rms_ratio": ratio_or_none(artifact.signal_rms, source_rms),
                    "low_band_rms_ratio": ratio_or_none(artifact.low_band_rms, source_low_band_rms),
                }
                if source_rms is None:
                    metric["failure"] = Failure(
                        code="source_backed_rms_missing",
                        role=role,
                        source=source,
                        message=(
                            f"{role} is identical across sources but source-backed RMS is missing "
                            f"for {source}"
                        ),
                    )
                    metrics.append(metric)
                    continue
                if artifact.signal_rms is None:
                    metric["failure"] = Failure(
                        code="generated_stem_rms_missing",
                        role=role,
                        source=source,
                        message=f"{role} RMS is missing for {source}",
                    )
                    metrics.append(metric)
                    continue
                failures = dominance_metric_failures(
                    role,
                    sha256,
                    source,
                    "signal",
                    artifact.signal_rms,
                    source_rms,
                    thresholds.max_generated_to_source_rms_ratio,
                )
                if (
                    not failures
                    and artifact.low_band_rms is not None
                    and source_low_band_rms is not None
                ):
                    failures = dominance_metric_failures(
                        role,
                        sha256,
                        source,
                        "low-band",
                        artifact.low_band_rms,
                        source_low_band_rms,
                        thresholds.max_generated_to_source_rms_ratio,
                    )
                if failures:
                    metric["failure"] = failures[0]
                metrics.append(metric)
    return metrics


def dominance_metric_failures(
    role: str,
    sha256: str,
    source: str,
    metric_name: str,
    generated_rms: float,
    source_rms: float,
    max_generated_to_source_rms_ratio: float,
) -> list[Failure]:
    ratio = generated_rms / max(source_rms, 1e-12)
    if ratio < max_generated_to_source_rms_ratio:
        return []
    return [
        Failure(
            code="generated_stem_dominates_mix",
            role=role,
            source=source,
            message=(
                f"{role} identical hash {short_hash(sha256)} is too dominant for "
                f"{source}: generated/source {metric_name} RMS ratio {ratio:.3f} >= "
                f"{max_generated_to_source_rms_ratio:.3f}"
            ),
        )
    ]


def full_mix_similarity_failures(
    pairwise: list[dict[str, Any]],
    thresholds: Thresholds,
) -> list[Failure]:
    failures = []
    for metric in pairwise:
        role = metric["role"]
        if not is_full_mix_role(role):
            continue
        normalized_rms_delta = metric.get("normalized_signal_rms_delta")
        if (
            normalized_rms_delta is not None
            and normalized_rms_delta < thresholds.min_full_mix_normalized_rms_delta
        ):
            failures.append(
                Failure(
                    code="full_mix_normalized_rms_delta_too_low",
                    role=role,
                    message=(
                        f"{role} normalized RMS delta {normalized_rms_delta:.3f} is below "
                        f"{thresholds.min_full_mix_normalized_rms_delta:.3f} for "
                        f"{metric['source_a']} vs {metric['source_b']}"
                    ),
                )
            )
        correlation = metric.get("waveform_correlation")
        if correlation is not None and correlation >= thresholds.max_full_mix_waveform_correlation:
            failures.append(
                Failure(
                    code="full_mix_cross_source_correlation_too_high",
                    role=role,
                    message=(
                        f"{role} waveform correlation {correlation:.6f} is above "
                        f"{thresholds.max_full_mix_waveform_correlation:.6f} for "
                        f"{metric['source_a']} vs {metric['source_b']}"
                    ),
                )
            )
        spectral_distance = metric.get("spectral_energy_distance")
        if (
            spectral_distance is not None
            and spectral_distance < thresholds.min_full_mix_spectral_distance
            and normalized_rms_delta is not None
            and normalized_rms_delta < thresholds.min_full_mix_normalized_rms_delta
        ):
            failures.append(
                Failure(
                    code="full_mix_spectral_distance_too_low",
                    role=role,
                    message=(
                        f"{role} spectral distance {spectral_distance:.3f} is below "
                        f"{thresholds.min_full_mix_spectral_distance:.3f} for "
                        f"{metric['source_a']} vs {metric['source_b']}"
                    ),
                )
            )
    return failures


def role_hash_groups(packs: tuple[PackEvidence, ...]) -> list[dict[str, Any]]:
    groups = []
    for role, artifacts in artifacts_by_role(packs).items():
        for sha256, duplicate_artifacts in group_by_hash(artifacts).items():
            groups.append(
                {
                    "role": role,
                    "sha256": sha256,
                    "short_hash": short_hash(sha256),
                    "sources": sorted({source for source, _ in duplicate_artifacts}),
                    "artifact_paths": [str(artifact.path) for _, artifact in duplicate_artifacts],
                    "identical_across_sources": len({source for source, _ in duplicate_artifacts})
                    > 1,
                }
            )
    return groups


def pairwise_metrics(packs: tuple[PackEvidence, ...]) -> list[dict[str, Any]]:
    metrics = []
    for left_index, left in enumerate(packs):
        for right in packs[left_index + 1 :]:
            if left.source == right.source:
                continue
            right_by_role = {artifact.role: artifact for artifact in right.artifacts}
            for left_artifact in left.artifacts:
                right_artifact = right_by_role.get(left_artifact.role)
                if right_artifact is None:
                    continue
                metrics.append(pairwise_artifact_metrics(left, left_artifact, right, right_artifact))
    return metrics


def pairwise_artifact_metrics(
    left_pack: PackEvidence,
    left: ArtifactEvidence,
    right_pack: PackEvidence,
    right: ArtifactEvidence,
) -> dict[str, Any]:
    return {
        "source_a": left_pack.source,
        "source_b": right_pack.source,
        "role": left.role,
        "same_hash": left.sha256 == right.sha256,
        "normalized_signal_rms_delta": normalized_delta(left.signal_rms, right.signal_rms),
        "normalized_low_band_rms_delta": normalized_delta(left.low_band_rms, right.low_band_rms),
        "spectral_energy_distance": spectral_energy_distance(
            left.spectral_energy,
            right.spectral_energy,
        ),
        "waveform_correlation": waveform_correlation(left.path, right.path),
    }


def normalized_delta(left: float | None, right: float | None) -> float | None:
    if left is None or right is None:
        return None
    denominator = max(abs(left), abs(right), 1e-12)
    return abs(left - right) / denominator


def ratio_or_none(numerator: float | None, denominator: float | None) -> float | None:
    if numerator is None or denominator is None:
        return None
    return numerator / max(denominator, 1e-12)


def spectral_energy_distance(
    left: dict[str, float],
    right: dict[str, float],
) -> float | None:
    keys = sorted(set(left) & set(right))
    if not keys:
        return None
    return math.sqrt(sum((left[key] - right[key]) ** 2 for key in keys))


def waveform_correlation(left_path: Path, right_path: Path) -> float | None:
    left = read_pcm16_samples(left_path)
    right = read_pcm16_samples(right_path)
    if left is None or right is None:
        return None
    sample_count = min(len(left), len(right))
    if sample_count < 2:
        return None
    if sample_count > MAX_CORRELATION_SAMPLES:
        stride = math.ceil(sample_count / MAX_CORRELATION_SAMPLES)
        left = left[:sample_count:stride]
        right = right[:sample_count:stride]
        sample_count = min(len(left), len(right))
    left_mean = sum(left[:sample_count]) / sample_count
    right_mean = sum(right[:sample_count]) / sample_count
    numerator = 0.0
    left_power = 0.0
    right_power = 0.0
    for left_sample, right_sample in zip(left[:sample_count], right[:sample_count]):
        left_centered = left_sample - left_mean
        right_centered = right_sample - right_mean
        numerator += left_centered * right_centered
        left_power += left_centered * left_centered
        right_power += right_centered * right_centered
    denominator = math.sqrt(left_power * right_power)
    if denominator <= 1e-12:
        return None
    return numerator / denominator


def read_pcm16_samples(path: Path) -> list[float] | None:
    try:
        with wave.open(str(path), "rb") as wav:
            if wav.getsampwidth() != 2:
                return None
            raw = wav.readframes(wav.getnframes())
    except (OSError, wave.Error):
        return None

    samples = array("h")
    samples.frombytes(raw)
    if sys.byteorder != "little":
        samples.byteswap()
    return [sample / 32768.0 for sample in samples]


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


def write_optional_reports(summary: dict[str, Any], args: Args) -> None:
    if args.json_output is not None:
        args.json_output.parent.mkdir(parents=True, exist_ok=True)
        args.json_output.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    if args.markdown_output is not None:
        args.markdown_output.parent.mkdir(parents=True, exist_ok=True)
        args.markdown_output.write_text(markdown_summary(summary))


def markdown_summary(summary: dict[str, Any]) -> str:
    lines = [
        "# Source Showcase Diversity Summary",
        "",
        f"- Result: `{summary['result']}`",
        f"- Sources: `{len(summary['sources'])}`",
        "",
        "## Thresholds",
        "",
    ]
    for key, value in summary["thresholds"].items():
        lines.append(f"- `{key}`: `{value}`")

    lines.extend(["", "## Failures", ""])
    if summary["failures"]:
        for failure in summary["failures"]:
            role = failure.get("role", "n/a")
            lines.append(f"- `{failure['code']}` / `{role}`: {failure['message']}")
    else:
        lines.append("- none")

    lines.extend(["", "## Pairwise Metrics", ""])
    lines.append(
        "| Role | Source A | Source B | Same Hash | Norm RMS Delta | Low RMS Delta | Spectral Distance | Wave Corr |"
    )
    lines.append("| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |")
    for metric in summary["pairwise_role_metrics"]:
        lines.append(
            "| {role} | {source_a} | {source_b} | {same_hash} | {signal} | {low} | {spectral} | {corr} |".format(
                role=metric["role"],
                source_a=metric["source_a"],
                source_b=metric["source_b"],
                same_hash="yes" if metric["same_hash"] else "no",
                signal=format_optional_float(metric["normalized_signal_rms_delta"]),
                low=format_optional_float(metric["normalized_low_band_rms_delta"]),
                spectral=format_optional_float(metric["spectral_energy_distance"]),
                corr=format_optional_float(metric["waveform_correlation"]),
            )
        )

    lines.extend(["", "## Generated Dominance", ""])
    lines.append(
        "| Role | Source | Identical Across Sources | Signal Ratio | Low-Band Ratio |"
    )
    lines.append("| --- | --- | ---: | ---: | ---: |")
    for metric in summary["generated_dominance"]:
        lines.append(
            "| {role} | {source} | {identical} | {signal} | {low} |".format(
                role=metric["role"],
                source=metric["source"],
                identical="yes" if metric["identical_across_sources"] else "no",
                signal=format_optional_float(metric["signal_rms_ratio"]),
                low=format_optional_float(metric["low_band_rms_ratio"]),
            )
        )

    return "\n".join(lines) + "\n"


def format_optional_float(value: float | None) -> str:
    if value is None:
        return "n/a"
    return f"{value:.6f}"


def require_string(parent: dict[str, Any], field: str) -> str:
    value = parent.get(field)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{field} must be a non-empty string")
    return value


def short_hash(sha256: str) -> str:
    return sha256[:12]


if __name__ == "__main__":
    raise SystemExit(main())

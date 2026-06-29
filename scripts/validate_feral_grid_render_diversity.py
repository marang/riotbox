#!/usr/bin/env python3
"""Validate cross-source diversity for rendered feral-grid WAV packs."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import tempfile
import wave
from array import array
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.feral_grid_render_diversity.v1"
MAX_CORRELATION_SAMPLES = 200_000
DEFAULT_MAX_CORRELATION = 0.995
DEFAULT_MIN_NORMALIZED_RMS_DELTA = 0.020
DEFAULT_MIN_MIX_NORMALIZED_RMS_DELTA = 0.050

PRODUCT_ROLES = {
    "source_first_mix": Path("04_riotbox_source_first_mix.wav"),
    "generated_support_mix": Path("05_riotbox_generated_support_mix.wav"),
    "tr909_beat_fill": Path("stems/01_tr909_beat_fill.wav"),
    "w30_feral_source_chop": Path("stems/02_w30_feral_source_chop.wav"),
    "mc202_bass_pressure": Path("stems/03_mc202_bass_pressure.wav"),
}
MIX_ROLES = {"source_first_mix", "generated_support_mix"}


@dataclass(frozen=True)
class RoleEvidence:
    source: str
    pack_dir: Path
    role: str
    path: Path
    sha256: str
    rms: float
    peak_abs: float


@dataclass(frozen=True)
class PairMetric:
    role: str
    source_a: str
    source_b: str
    same_hash: bool
    normalized_rms_delta: float
    waveform_correlation: float | None

    def as_dict(self) -> dict[str, Any]:
        return {
            "role": self.role,
            "source_a": self.source_a,
            "source_b": self.source_b,
            "same_hash": self.same_hash,
            "normalized_rms_delta": self.normalized_rms_delta,
            "waveform_correlation": self.waveform_correlation,
        }


def main() -> int:
    args = parse_args()
    if args.self_test_fixtures:
        return run_self_test_fixtures()

    try:
        packs = [read_pack(path) for path in args.pack_dirs]
        summary = build_summary(
            packs,
            max_correlation=args.max_correlation,
            min_normalized_rms_delta=args.min_normalized_rms_delta,
            min_mix_normalized_rms_delta=args.min_mix_normalized_rms_delta,
        )
        write_optional_outputs(summary, args)
    except (OSError, ValueError, json.JSONDecodeError, wave.Error) as error:
        print(f"invalid feral-grid render diversity: {error}")
        return 1
    if summary["failures"]:
        print("invalid feral-grid render diversity: " + summary["failures"][0]["message"])
        return 1
    print(f"valid feral-grid render diversity across {len(packs)} packs")
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack_dirs", nargs="*", type=Path)
    parser.add_argument("--json-output", type=Path)
    parser.add_argument("--markdown-output", type=Path)
    parser.add_argument("--max-correlation", type=float, default=DEFAULT_MAX_CORRELATION)
    parser.add_argument(
        "--min-normalized-rms-delta",
        type=float,
        default=DEFAULT_MIN_NORMALIZED_RMS_DELTA,
    )
    parser.add_argument(
        "--min-mix-normalized-rms-delta",
        type=float,
        default=DEFAULT_MIN_MIX_NORMALIZED_RMS_DELTA,
    )
    parser.add_argument("--self-test-fixtures", action="store_true")
    args = parser.parse_args()
    if not args.self_test_fixtures and len(args.pack_dirs) < 2:
        parser.error("expected at least two feral-grid pack directories")
    return args


def read_pack(pack_dir: Path) -> tuple[RoleEvidence, ...]:
    pack_dir = pack_dir.resolve()
    source = source_label(pack_dir)
    evidence = []
    for role, relative_path in PRODUCT_ROLES.items():
        path = pack_dir / relative_path
        if not path.is_file():
            raise ValueError(f"missing {role} WAV: {path}")
        samples = read_pcm16_samples(path)
        evidence.append(
            RoleEvidence(
                source=source,
                pack_dir=pack_dir,
                role=role,
                path=path,
                sha256=sha256_file(path),
                rms=rms(samples),
                peak_abs=peak_abs(samples),
            )
        )
    return tuple(evidence)


def source_label(pack_dir: Path) -> str:
    manifest_path = pack_dir / "manifest.json"
    if manifest_path.is_file():
        try:
            manifest = json.loads(manifest_path.read_text())
        except json.JSONDecodeError:
            manifest = {}
        source = manifest.get("source")
        if isinstance(source, str) and source:
            return source
    parent = pack_dir.parent.name
    return parent if parent else pack_dir.name


def build_summary(
    packs: list[tuple[RoleEvidence, ...]],
    *,
    max_correlation: float,
    min_normalized_rms_delta: float,
    min_mix_normalized_rms_delta: float,
) -> dict[str, Any]:
    flat = [item for pack in packs for item in pack]
    sources = sorted({item.source for item in flat})
    if len(sources) < 2:
        raise ValueError("feral-grid diversity needs at least two distinct sources")

    pair_metrics = pairwise_metrics(packs)
    failures = []
    for metric in pair_metrics:
        role_min_delta = (
            min_mix_normalized_rms_delta
            if metric.role in MIX_ROLES
            else min_normalized_rms_delta
        )
        if metric.same_hash:
            failures.append(
                failure(
                    "feral_grid_role_identical_across_sources",
                    metric,
                    f"{metric.role} has identical audio across {metric.source_a} and {metric.source_b}",
                )
            )
        correlation_too_high = (
            metric.waveform_correlation is not None
            and metric.waveform_correlation >= max_correlation
        )
        rms_too_close = metric.normalized_rms_delta < role_min_delta
        if rms_too_close and correlation_too_high:
            failures.append(
                failure(
                    "feral_grid_role_rms_delta_too_low",
                    metric,
                    (
                        f"{metric.role} normalized RMS delta {metric.normalized_rms_delta:.4f} "
                        f"is below {role_min_delta:.4f} for {metric.source_a} vs {metric.source_b}"
                    ),
                )
            )
        if correlation_too_high:
            failures.append(
                failure(
                    "feral_grid_role_cross_source_correlation_too_high",
                    metric,
                    (
                        f"{metric.role} waveform correlation {metric.waveform_correlation:.6f} "
                        f"is above {max_correlation:.6f} for {metric.source_a} vs {metric.source_b}"
                    ),
                )
            )

    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "fail" if failures else "pass",
        "thresholds": {
            "max_correlation": max_correlation,
            "min_normalized_rms_delta": min_normalized_rms_delta,
            "min_mix_normalized_rms_delta": min_mix_normalized_rms_delta,
        },
        "sources": sources,
        "packs": [
            {
                "source": pack[0].source,
                "pack_dir": str(pack[0].pack_dir),
                "roles": [item.role for item in pack],
            }
            for pack in packs
        ],
        "pairwise_role_metrics": [metric.as_dict() for metric in pair_metrics],
        "failures": failures,
    }


def failure(code: str, metric: PairMetric, message: str) -> dict[str, Any]:
    return {
        "code": code,
        "role": metric.role,
        "source_a": metric.source_a,
        "source_b": metric.source_b,
        "message": message,
    }


def pairwise_metrics(packs: list[tuple[RoleEvidence, ...]]) -> list[PairMetric]:
    metrics = []
    for left_index, left_pack in enumerate(packs):
        for right_pack in packs[left_index + 1 :]:
            left_by_role = {item.role: item for item in left_pack}
            right_by_role = {item.role: item for item in right_pack}
            if left_pack[0].source == right_pack[0].source:
                continue
            for role in PRODUCT_ROLES:
                left = left_by_role[role]
                right = right_by_role[role]
                metrics.append(
                    PairMetric(
                        role=role,
                        source_a=left.source,
                        source_b=right.source,
                        same_hash=left.sha256 == right.sha256,
                        normalized_rms_delta=normalized_delta(left.rms, right.rms),
                        waveform_correlation=waveform_correlation(left.path, right.path),
                    )
                )
    return metrics


def normalized_delta(left: float, right: float) -> float:
    return abs(left - right) / max(abs(left), abs(right), 1e-12)


def read_pcm16_samples(path: Path) -> list[float]:
    with wave.open(str(path), "rb") as wav:
        if wav.getsampwidth() != 2:
            raise ValueError(f"{path}: expected 16-bit PCM WAV")
        raw = wav.readframes(wav.getnframes())
    samples = array("h")
    samples.frombytes(raw)
    return [sample / 32768.0 for sample in samples]


def waveform_correlation(left_path: Path, right_path: Path) -> float | None:
    left = read_pcm16_samples(left_path)
    right = read_pcm16_samples(right_path)
    count = min(len(left), len(right), MAX_CORRELATION_SAMPLES)
    if count < 2:
        return None
    if count < min(len(left), len(right)):
        stride = max(1, min(len(left), len(right)) // count)
        left = left[::stride][:count]
        right = right[::stride][:count]
    else:
        left = left[:count]
        right = right[:count]
    left_mean = sum(left) / count
    right_mean = sum(right) / count
    numerator = 0.0
    left_energy = 0.0
    right_energy = 0.0
    for left_sample, right_sample in zip(left, right, strict=True):
        left_centered = left_sample - left_mean
        right_centered = right_sample - right_mean
        numerator += left_centered * right_centered
        left_energy += left_centered * left_centered
        right_energy += right_centered * right_centered
    denominator = math.sqrt(left_energy * right_energy)
    if denominator <= 1e-12:
        return None
    return numerator / denominator


def rms(samples: list[float]) -> float:
    if not samples:
        return 0.0
    return math.sqrt(sum(sample * sample for sample in samples) / len(samples))


def peak_abs(samples: list[float]) -> float:
    return max((abs(sample) for sample in samples), default=0.0)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as file:
        for chunk in iter(lambda: file.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_optional_outputs(summary: dict[str, Any], args: argparse.Namespace) -> None:
    if args.json_output:
        args.json_output.parent.mkdir(parents=True, exist_ok=True)
        args.json_output.write_text(json.dumps(summary, indent=2) + "\n")
    if args.markdown_output:
        args.markdown_output.parent.mkdir(parents=True, exist_ok=True)
        args.markdown_output.write_text(markdown_summary(summary))


def markdown_summary(summary: dict[str, Any]) -> str:
    lines = [
        "# Feral-grid Render Diversity",
        "",
        f"- Result: `{summary['result']}`",
        f"- Sources: `{', '.join(summary['sources'])}`",
        "",
        "## Pairwise Role Metrics",
        "",
        "| Role | Sources | RMS delta | Correlation | Same hash |",
        "| --- | --- | ---: | ---: | --- |",
    ]
    for metric in summary["pairwise_role_metrics"]:
        correlation = metric["waveform_correlation"]
        lines.append(
            "| {role} | {source_a} vs {source_b} | {delta:.4f} | {corr} | {same} |".format(
                role=metric["role"],
                source_a=metric["source_a"],
                source_b=metric["source_b"],
                delta=metric["normalized_rms_delta"],
                corr="n/a" if correlation is None else f"{correlation:.6f}",
                same="yes" if metric["same_hash"] else "no",
            )
        )
    if summary["failures"]:
        lines.extend(["", "## Failures", ""])
        for item in summary["failures"]:
            lines.append(f"- `{item['code']}` {item['message']}")
    return "\n".join(lines) + "\n"


def run_self_test_fixtures() -> int:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        valid_a = root / "valid-a" / "feral-grid-demo"
        valid_b = root / "valid-b" / "feral-grid-demo"
        invalid_a = root / "invalid-a" / "feral-grid-demo"
        invalid_b = root / "invalid-b" / "feral-grid-demo"
        write_fixture_pack(valid_a, frequency=110.0, source="fixture-a.wav")
        write_fixture_pack(valid_b, frequency=197.0, source="fixture-b.wav")
        write_fixture_pack(invalid_a, frequency=110.0, source="fixture-a.wav")
        write_fixture_pack(invalid_b, frequency=110.0, source="fixture-b.wav")

        valid_summary = build_summary(
            [read_pack(valid_a), read_pack(valid_b)],
            max_correlation=DEFAULT_MAX_CORRELATION,
            min_normalized_rms_delta=DEFAULT_MIN_NORMALIZED_RMS_DELTA,
            min_mix_normalized_rms_delta=DEFAULT_MIN_MIX_NORMALIZED_RMS_DELTA,
        )
        invalid_summary = build_summary(
            [read_pack(invalid_a), read_pack(invalid_b)],
            max_correlation=DEFAULT_MAX_CORRELATION,
            min_normalized_rms_delta=DEFAULT_MIN_NORMALIZED_RMS_DELTA,
            min_mix_normalized_rms_delta=DEFAULT_MIN_MIX_NORMALIZED_RMS_DELTA,
        )
        if valid_summary["failures"]:
            raise AssertionError(valid_summary["failures"])
        failure_codes = {item["code"] for item in invalid_summary["failures"]}
        expected = {
            "feral_grid_role_identical_across_sources",
            "feral_grid_role_rms_delta_too_low",
            "feral_grid_role_cross_source_correlation_too_high",
        }
        if not expected.issubset(failure_codes):
            raise AssertionError(invalid_summary["failures"])
    print("valid feral-grid render diversity self-test fixtures")
    return 0


def write_fixture_pack(pack_dir: Path, *, frequency: float, source: str) -> None:
    (pack_dir / "stems").mkdir(parents=True)
    (pack_dir / "manifest.json").write_text(json.dumps({"source": source}) + "\n")
    role_frequencies = {
        "source_first_mix": frequency,
        "generated_support_mix": frequency * 1.25,
        "tr909_beat_fill": frequency * 1.5,
        "w30_feral_source_chop": frequency * 1.75,
        "mc202_bass_pressure": frequency * 2.0,
    }
    for role, relative_path in PRODUCT_ROLES.items():
        write_tone_wav(pack_dir / relative_path, role_frequencies[role])


def write_tone_wav(path: Path, frequency: float) -> None:
    sample_rate = 44_100
    frames = sample_rate // 2
    samples = array("h")
    for index in range(frames):
        envelope = 0.65 + 0.35 * math.sin(index / sample_rate * math.tau * 7.0)
        value = int(math.sin(index / sample_rate * math.tau * frequency) * envelope * 10_000)
        samples.append(value)
        samples.append(int(value * 0.97))
    with wave.open(str(path), "wb") as wav:
        wav.setnchannels(2)
        wav.setsampwidth(2)
        wav.setframerate(sample_rate)
        wav.writeframes(samples.tobytes())


if __name__ == "__main__":
    raise SystemExit(main())

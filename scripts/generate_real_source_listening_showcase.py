#!/usr/bin/env python3
"""Generate a local real-source Riotbox listening showcase.

This is a musician-facing review helper, not a CI fixture generator. It renders
existing local example WAVs through `feral_grid_pack`, keeps the source window as
a separate before/after file, and reports technical status separately from the
musical verdict.
"""

from __future__ import annotations

import argparse
import json
import math
import shutil
import subprocess
import sys
import wave
from dataclasses import dataclass
from pathlib import Path


SAMPLE_RATE = 44_100
CHANNELS = 2
DEFAULT_MANIFEST = Path("data/showcase_sources/local_listening_manifest.json")
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-real-source-listening-showcase")
DEFAULT_DATE_LABEL = "local-real-source-listening-showcase"
REQUIRED_RENDER_FILES = (
    "stems/01_tr909_beat_fill.wav",
    "stems/02_w30_feral_source_chop.wav",
    "stems/03_mc202_bass_pressure.wav",
    "04_riotbox_source_first_mix.wav",
    "05_riotbox_generated_support_mix.wav",
)

np = None


@dataclass(frozen=True)
class AudioMetrics:
    rms: float
    dbfs: float
    peak_abs: float
    low_band_ratio: float
    mid_band_ratio: float
    high_band_ratio: float
    bar_similarity: float | None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default=DEFAULT_DATE_LABEL)
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--keep-output", action="store_true")
    args = parser.parse_args()

    repo = repo_root()
    manifest_path = resolve_repo_path(repo, args.manifest)
    manifest = load_manifest(manifest_path)
    cases = validate_manifest(repo, manifest)
    if args.validate_only:
        print(f"valid real-source listening showcase manifest: {manifest_path}")
        return 0

    output = resolve_repo_path(repo, args.output)
    ensure_safe_output(repo, output)
    if output.exists() and not args.keep_output:
        shutil.rmtree(output)
    (output / "packs").mkdir(parents=True, exist_ok=True)
    (output / "validation").mkdir(parents=True, exist_ok=True)

    results = []
    for source, window in cases:
        result = render_case(repo, output, args.date, source, window)
        results.append(result)

    write_reports(output, manifest_path, manifest, results)
    print(f"real-source listening showcase written to {output}")
    return 0


def repo_root() -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return Path(result.stdout.strip())


def resolve_repo_path(repo: Path, path: Path) -> Path:
    return path if path.is_absolute() else repo / path


def load_manifest(path: Path) -> dict:
    with path.open() as handle:
        data = json.load(handle)
    if data.get("schema") != "riotbox.real_source_listening_showcase.v1":
        raise SystemExit(f"unsupported showcase manifest schema: {data.get('schema')}")
    if data.get("schema_version") != 1:
        raise SystemExit(f"unsupported showcase manifest version: {data.get('schema_version')}")
    return data


def validate_manifest(repo: Path, manifest: dict) -> list[tuple[dict, dict]]:
    cases: list[tuple[dict, dict]] = []
    source_ids: set[str] = set()
    for source in manifest.get("sources", []):
        for field in ("id", "path", "usage", "license_note", "role", "bpm", "expected_outcome"):
            if field not in source:
                raise SystemExit(f"source is missing required field {field}: {source}")
        source_id = source["id"]
        if source_id in source_ids:
            raise SystemExit(f"duplicate source id: {source_id}")
        source_ids.add(source_id)
        source_path = resolve_repo_path(repo, Path(source["path"]))
        if not source_path.exists():
            raise SystemExit(f"missing local showcase source: {source['path']}")
        duration = wav_duration(source_path)
        windows = source.get("windows") or []
        if not windows:
            raise SystemExit(f"source has no windows: {source_id}")
        window_ids: set[str] = set()
        for window in windows:
            for field in ("id", "start_seconds", "duration_seconds", "bars"):
                if field not in window:
                    raise SystemExit(f"window is missing required field {field}: {source_id}")
            window_id = window["id"]
            if window_id in window_ids:
                raise SystemExit(f"duplicate window id for {source_id}: {window_id}")
            window_ids.add(window_id)
            start = float(window["start_seconds"])
            length = float(window["duration_seconds"])
            if start < 0.0 or length <= 0.0:
                raise SystemExit(f"invalid window timing for {source_id}/{window_id}")
            if start + length > duration + 0.02:
                raise SystemExit(
                    f"window exceeds source duration for {source_id}/{window_id}: "
                    f"{start + length:.3f}s > {duration:.3f}s"
                )
            if int(window["bars"]) <= 0:
                raise SystemExit(f"window bars must be positive for {source_id}/{window_id}")
            cases.append((source, window))
    if not cases:
        raise SystemExit("manifest contains no render cases")
    return cases


def ensure_safe_output(repo: Path, output: Path) -> None:
    allowed = (repo / "artifacts" / "audio_qa").resolve()
    output_resolved = output.resolve()
    if allowed not in output_resolved.parents:
        raise SystemExit(f"refusing to write outside artifacts/audio_qa: {output}")


def render_case(repo: Path, output: Path, date: str, source: dict, window: dict) -> dict:
    source_id = source["id"]
    window_id = window["id"]
    source_path = resolve_repo_path(repo, Path(source["path"]))
    pack_dir = output / "packs" / source_id / window_id
    pack_dir.mkdir(parents=True, exist_ok=True)

    source_window = pack_dir / "00_source_window.wav"
    extract_wav_window(
        source_path,
        source_window,
        float(window["start_seconds"]),
        float(window["duration_seconds"]),
    )

    render_result = subprocess.run(
        [
            "cargo",
            "run",
            "-p",
            "riotbox-audio",
            "--bin",
            "feral_grid_pack",
            "--",
            "--source",
            str(source_path),
            "--output-dir",
            str(pack_dir),
            "--date",
            date,
            "--bpm",
            str(float(source["bpm"])),
            "--bars",
            str(int(window["bars"])),
            "--source-window-seconds",
            str(float(window["duration_seconds"])),
            "--source-start-seconds",
            str(float(window["start_seconds"])),
        ],
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    (pack_dir / "render.log").write_text(
        render_result.stdout
        + ("\n" if render_result.stdout and render_result.stderr else "")
        + render_result.stderr
    )
    render_error = None
    if render_result.returncode != 0:
        render_error = f"feral_grid_pack_exit_{render_result.returncode}"

    metrics = collect_case_metrics(pack_dir, float(source["bpm"]), int(window["bars"]))
    technical_issues = technical_issues_for(metrics)
    if render_error:
        technical_issues.insert(0, render_error)
    musical_issues = musical_issues_for(metrics)
    technical_status = "pass" if not technical_issues else "fail"
    if render_error:
        musical_issues.insert(0, "render_failed")
        musical_verdict = "unavailable"
    else:
        musical_verdict = musical_verdict_for(musical_issues)
    write_case_report(
        pack_dir,
        source,
        window,
        metrics,
        technical_status,
        technical_issues,
        musical_verdict,
        musical_issues,
        render_error,
    )

    return {
        "source_id": source_id,
        "window_id": window_id,
        "role": source["role"],
        "pack_dir": str(pack_dir.relative_to(output)),
        "technical_status": technical_status,
        "technical_issues": technical_issues,
        "render_error": render_error,
        "musical_verdict": musical_verdict,
        "musical_issues": musical_issues,
        "metrics": {name: metrics_to_json(value) for name, value in metrics.items()},
    }


def extract_wav_window(source: Path, target: Path, start_seconds: float, duration_seconds: float) -> None:
    with wave.open(str(source), "rb") as src:
        channels = src.getnchannels()
        sample_width = src.getsampwidth()
        sample_rate = src.getframerate()
        start = int(round(start_seconds * sample_rate))
        count = int(round(duration_seconds * sample_rate))
        src.setpos(start)
        frames = src.readframes(count)
    with wave.open(str(target), "wb") as dst:
        dst.setnchannels(channels)
        dst.setsampwidth(sample_width)
        dst.setframerate(sample_rate)
        dst.writeframes(frames)


def wav_duration(path: Path) -> float:
    with wave.open(str(path), "rb") as wav:
        return wav.getnframes() / float(wav.getframerate())


def read_wav_mono(path: Path) -> tuple[int, np.ndarray]:
    require_numpy()
    with wave.open(str(path), "rb") as wav:
        sample_rate = wav.getframerate()
        channels = wav.getnchannels()
        sample_width = wav.getsampwidth()
        frames = wav.readframes(wav.getnframes())
    if sample_width == 2:
        samples = np.frombuffer(frames, dtype="<i2").astype(np.float32) / 32768.0
    elif sample_width == 3:
        raw = np.frombuffer(frames, dtype=np.uint8).reshape(-1, 3)
        values = (
            raw[:, 0].astype(np.int32)
            | (raw[:, 1].astype(np.int32) << 8)
            | (raw[:, 2].astype(np.int32) << 16)
        )
        values = np.where(values & 0x800000, values - 0x1000000, values)
        samples = values.astype(np.float32) / 8388608.0
    elif sample_width == 4:
        samples = np.frombuffer(frames, dtype="<i4").astype(np.float32) / 2147483648.0
    else:
        raise SystemExit(f"unsupported WAV sample width for listening showcase metrics: {path}")
    if channels > 1:
        samples = samples.reshape(-1, channels).mean(axis=1)
    return sample_rate, samples


def require_numpy() -> None:
    global np
    if np is None:
        import numpy as numpy

        np = numpy


def collect_case_metrics(pack_dir: Path, bpm: float, bars: int) -> dict[str, AudioMetrics | None]:
    files = {
        "source_window": "00_source_window.wav",
        "tr909": "stems/01_tr909_beat_fill.wav",
        "w30": "stems/02_w30_feral_source_chop.wav",
        "mc202": "stems/03_mc202_bass_pressure.wav",
        "source_first_mix": "04_riotbox_source_first_mix.wav",
        "riotbox_mix": "05_riotbox_generated_support_mix.wav",
    }
    return {
        name: audio_metrics(pack_dir / relative, bpm, bars) if (pack_dir / relative).exists() else None
        for name, relative in files.items()
    }


def audio_metrics(path: Path, bpm: float, bars: int) -> AudioMetrics:
    sample_rate, samples = read_wav_mono(path)
    if samples.size == 0:
        return AudioMetrics(0.0, -240.0, 0.0, 0.0, 0.0, 0.0, None)
    rms = float(np.sqrt(np.mean(samples * samples)))
    peak = float(np.max(np.abs(samples)))
    low, mid, high = spectral_ratios(samples, sample_rate)
    return AudioMetrics(
        rms=rms,
        dbfs=20.0 * math.log10(rms + 1e-12),
        peak_abs=peak,
        low_band_ratio=low,
        mid_band_ratio=mid,
        high_band_ratio=high,
        bar_similarity=bar_similarity(samples, sample_rate, bpm, bars),
    )


def spectral_ratios(samples: np.ndarray, sample_rate: int) -> tuple[float, float, float]:
    if samples.size < 2:
        return 0.0, 0.0, 0.0
    window = np.hanning(samples.size)
    spectrum = np.abs(np.fft.rfft(samples * window)) + 1e-12
    freqs = np.fft.rfftfreq(samples.size, 1.0 / sample_rate)
    power = spectrum * spectrum
    total = float(np.sum(power))
    return (
        float(np.sum(power[freqs < 180.0]) / total),
        float(np.sum(power[(freqs >= 180.0) & (freqs < 2500.0)]) / total),
        float(np.sum(power[freqs >= 2500.0]) / total),
    )


def bar_similarity(samples: np.ndarray, sample_rate: int, bpm: float, bars: int) -> float | None:
    bar_frames = int(round(sample_rate * 60.0 / bpm * 4.0))
    if bar_frames <= 0 or bars < 2:
        return None
    envelopes = []
    for bar in range(bars):
        segment = samples[bar * bar_frames : (bar + 1) * bar_frames]
        if segment.size != bar_frames:
            continue
        bins = np.array([float(np.mean(np.abs(chunk))) for chunk in np.array_split(segment, 64)])
        bins -= float(np.mean(bins))
        norm = float(np.linalg.norm(bins))
        if norm > 1e-9:
            bins /= norm
        envelopes.append(bins)
    if len(envelopes) < 2:
        return None
    values = []
    for index, left in enumerate(envelopes):
        for right in envelopes[index + 1 :]:
            values.append(float(np.dot(left, right)))
    return float(np.mean(values))


def technical_issues_for(metrics: dict[str, AudioMetrics | None]) -> list[str]:
    issues = []
    for name in ("source_window", "tr909", "w30", "mc202", "source_first_mix", "riotbox_mix"):
        metric = metrics.get(name)
        if metric is None:
            issues.append(f"{name}_missing")
        elif metric.rms < 0.0005:
            issues.append(f"{name}_too_quiet_or_silent")
        elif metric.peak_abs > 0.98:
            issues.append(f"{name}_near_clipping")
    return issues


def musical_issues_for(metrics: dict[str, AudioMetrics | None]) -> list[str]:
    issues = []
    tr909 = metrics.get("tr909")
    mc202 = metrics.get("mc202")
    w30 = metrics.get("w30")
    mix = metrics.get("riotbox_mix")

    if tr909 and tr909.bar_similarity is not None and tr909.bar_similarity >= 0.95:
        issues.append("tr909_static_bar_shape")
    if mc202 and mc202.dbfs < -38.0:
        issues.append("mc202_buried")
    if mc202 and mc202.bar_similarity is not None and mc202.bar_similarity >= 0.90:
        issues.append("mc202_static_bar_shape")
    if mix and mix.low_band_ratio >= 0.92 and mix.high_band_ratio < 0.04:
        issues.append("mix_low_band_dominant")
    if w30 and mix and w30.rms > 0.0 and mc202 and mc202.rms / w30.rms < 0.35:
        issues.append("mc202_weak_relative_to_w30")
    if tr909 and mix and tr909.rms / (mix.rms + 1e-12) < 0.08:
        issues.append("tr909_low_mix_contribution")
    return issues


def musical_verdict_for(issues: list[str]) -> str:
    if not issues:
        return "promising"
    if len(issues) <= 2:
        return "weak"
    return "fail"


def write_case_report(
    pack_dir: Path,
    source: dict,
    window: dict,
    metrics: dict[str, AudioMetrics | None],
    technical_status: str,
    technical_issues: list[str],
    musical_verdict: str,
    musical_issues: list[str],
    render_error: str | None,
) -> None:
    lines = [
        f"# Listening Case: {source['id']} / {window['id']}",
        "",
        f"- Role: `{source['role']}`",
        f"- Source: `{source['path']}`",
        f"- BPM: `{source['bpm']}`",
        f"- Window: `{window['start_seconds']}`s + `{window['duration_seconds']}`s",
        f"- Expected outcome: {source['expected_outcome']}",
        f"- Technical status: `{technical_status}`",
        f"- Musical verdict: `{musical_verdict}`",
        f"- Render error: `{render_error or 'none'}`",
        "",
        "## Issues",
        "",
        f"- Technical: `{', '.join(technical_issues) if technical_issues else 'none'}`",
        f"- Musical: `{', '.join(musical_issues) if musical_issues else 'none'}`",
        "",
        "## Files",
        "",
        "- `00_source_window.wav`: source comparison window only",
        "- `stems/01_tr909_beat_fill.wav`: Riotbox TR-909 lane",
        "- `stems/02_w30_feral_source_chop.wav`: Riotbox W-30 lane",
        "- `stems/03_mc202_bass_pressure.wav`: Riotbox MC-202 lane",
        "- `04_riotbox_source_first_mix.wav`: source-first Riotbox render",
        "- `05_riotbox_generated_support_mix.wav`: Riotbox generated-support mix",
        "",
        "## Metrics",
        "",
        "| Role | RMS dBFS | Peak | Low | Mid | High | Bar Similarity |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for name, metric in metrics.items():
        if metric is None:
            lines.append(f"| `{name}` | missing | missing | missing | missing | missing | missing |")
            continue
        bar = "n/a" if metric.bar_similarity is None else f"{metric.bar_similarity:.3f}"
        lines.append(
            f"| `{name}` | {metric.dbfs:.1f} | {metric.peak_abs:.3f} | "
            f"{metric.low_band_ratio * 100.0:.1f}% | {metric.mid_band_ratio * 100.0:.1f}% | "
            f"{metric.high_band_ratio * 100.0:.1f}% | {bar} |"
        )
    (pack_dir / "analysis.md").write_text("\n".join(lines) + "\n")


def write_reports(output: Path, manifest_path: Path, manifest: dict, results: list[dict]) -> None:
    technical_status = "pass" if all(result["technical_status"] == "pass" for result in results) else "fail"
    if any(result["musical_verdict"] in {"fail", "unavailable"} for result in results):
        musical_verdict = "fail"
    elif any(result["musical_verdict"] == "weak" for result in results):
        musical_verdict = "weak"
    else:
        musical_verdict = "promising"

    summary = {
        "schema": "riotbox.real_source_listening_showcase.report.v1",
        "schema_version": 1,
        "manifest": str(manifest_path),
        "technical_status": technical_status,
        "musical_verdict": musical_verdict,
        "case_count": len(results),
        "results": results,
    }
    (output / "showcase-report.json").write_text(json.dumps(summary, indent=2) + "\n")

    lines = [
        "# Real-Source Listening Showcase",
        "",
        f"- Technical status: `{technical_status}`",
        f"- Musical verdict: `{musical_verdict}`",
        f"- Manifest: `{manifest_path}`",
        "",
        "This is a local listening review pack from real local source files. It is not a CI fixture.",
        "A weak or fail musical verdict can still produce valid WAV artifacts.",
        "",
        "## Cases",
        "",
        "| Case | Role | Technical | Musical | First Listen |",
        "| --- | --- | --- | --- | --- |",
    ]
    for result in results:
        first_listen = f"{result['pack_dir']}/analysis.md"
        lines.append(
            f"| `{result['source_id']}/{result['window_id']}` | `{result['role']}` | "
            f"`{result['technical_status']}` | `{result['musical_verdict']}` | `{first_listen}` |"
        )
    lines.extend(
        [
            "",
            "## Listening Order Per Case",
            "",
            "1. `00_source_window.wav`",
            "2. `stems/01_tr909_beat_fill.wav`",
            "3. `stems/02_w30_feral_source_chop.wav`",
            "4. `stems/03_mc202_bass_pressure.wav`",
            "5. `04_riotbox_source_first_mix.wav`",
            "6. `05_riotbox_generated_support_mix.wav`",
            "",
            "## Boundary",
            "",
            "This report separates render validity from musical quality. It should not be used to claim product-level sound quality unless the musical verdict is `promising` and human listening agrees.",
        ]
    )
    (output / "README.md").write_text("\n".join(lines) + "\n")


def metrics_to_json(metric: AudioMetrics | None) -> dict | None:
    if metric is None:
        return None
    return {
        "rms": metric.rms,
        "dbfs": metric.dbfs,
        "peak_abs": metric.peak_abs,
        "low_band_ratio": metric.low_band_ratio,
        "mid_band_ratio": metric.mid_band_ratio,
        "high_band_ratio": metric.high_band_ratio,
        "bar_similarity": metric.bar_similarity,
    }


if __name__ == "__main__":
    sys.exit(main())

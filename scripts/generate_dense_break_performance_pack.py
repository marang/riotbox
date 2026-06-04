#!/usr/bin/env python3
"""Generate the dense-break 8-bar Riotbox performance Golden Path pack."""

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
DEFAULT_SOURCE = Path("data/test_audio/examples/Beat03_130BPM(Full).wav")
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-dense-break-performance-pack")
DEFAULT_DATE = "local-dense-break-performance-pack"
DEFAULT_BPM = 130.0
DEFAULT_BARS = 8
BEATS_PER_BAR = 4
SCHEMA = "riotbox.dense_break_performance_pack.v1"
MIN_W30_TO_SOURCE_RMS_RATIO = 0.18
MIN_PRESSURE_LOW_BAND_LIFT_RATIO = 1.12
MAX_DROPOUT_TO_STUTTER_RMS_RATIO = 0.18
MIN_STUTTER_TO_HOOK_TRANSIENT_RATIO = 0.58
MIN_RESTORE_TO_HOOK_TRANSIENT_RATIO = 0.85
MAX_ADJACENT_BAR_CORRELATION = 0.985
MAX_SOURCE_TO_PERFORMANCE_CORRELATION = 0.975
MIN_MC202_TO_W30_RMS_RATIO = 0.12

np = None


@dataclass(frozen=True)
class AudioMetrics:
    rms: float
    dbfs: float
    peak_abs: float
    low_band_rms: float
    high_band_ratio: float
    transient_score: float


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, default=DEFAULT_SOURCE)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default=DEFAULT_DATE)
    parser.add_argument("--bpm", type=float, default=DEFAULT_BPM)
    parser.add_argument("--bars", type=int, default=DEFAULT_BARS)
    parser.add_argument("--source-start-seconds", type=float, default=0.0)
    parser.add_argument("--keep-output", action="store_true")
    args = parser.parse_args()

    repo = repo_root()
    source = resolve_repo_path(repo, args.source)
    output = resolve_repo_path(repo, args.output)
    validate_args(source, output, args.bpm, args.bars)
    ensure_safe_output(repo, output)

    if output.exists() and not args.keep_output:
        shutil.rmtree(output)
    output.mkdir(parents=True, exist_ok=True)

    duration = performance_duration_seconds(args.bpm, args.bars)
    available_source_window = max(0.0, wav_duration(source) - args.source_start_seconds)
    if available_source_window <= 0.0:
        raise SystemExit(f"source start exceeds source duration: {args.source_start_seconds}")
    render_source_window_seconds = min(duration, available_source_window)
    render_dir = output / "_feral_grid_render"
    render_feral_grid_pack(
        repo,
        source,
        render_dir,
        args.date,
        args.bpm,
        args.bars,
        args.source_start_seconds,
        render_source_window_seconds,
    )

    source_audio = read_wav_window_looped(source, args.source_start_seconds, duration)
    tr909 = read_wav(render_dir / "stems" / "01_tr909_beat_fill.wav")
    w30 = read_wav(render_dir / "stems" / "02_w30_feral_source_chop.wav")
    mc202 = read_wav(render_dir / "stems" / "03_mc202_bass_pressure.wav")

    frame_count = min(
        source_audio.shape[0],
        tr909.shape[0],
        w30.shape[0],
        mc202.shape[0],
        frames_for_seconds(duration),
    )
    source_audio = source_audio[:frame_count]
    tr909 = tr909[:frame_count]
    w30 = apply_gain(w30[:frame_count], 1.22)
    mc202 = apply_gain(mc202[:frame_count], 1.35)

    bar_frames = frames_for_beats(args.bpm, BEATS_PER_BAR)
    performance, sections = render_performance(
        source_audio,
        tr909,
        w30,
        mc202,
        bar_frames,
        args.bars,
    )

    write_wav(output / "00_source_window.wav", source_audio)
    write_wav(output / "01_chop_hook.wav", sections["chop_hook"])
    write_wav(output / "02_pressure_lift.wav", sections["pressure_lift"])
    write_wav(output / "03_dropout_stutter.wav", sections["dropout_stutter"])
    write_wav(output / "04_restore_hit.wav", sections["restore_hit"])
    write_wav(output / "05_full_performance.wav", performance)

    report = build_report(
        source,
        output,
        args,
        source_audio,
        tr909,
        w30,
        mc202,
        performance,
        sections,
        bar_frames,
    )
    write_reports(output, report)
    if report["result"] != "pass":
        print(
            "dense-break performance pack failed: "
            + ", ".join(report["failure_codes"]),
            file=sys.stderr,
        )
        return 1

    print(f"dense-break performance pack written to {output}")
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


def validate_args(source: Path, output: Path, bpm: float, bars: int) -> None:
    if not source.is_file():
        raise SystemExit(f"missing dense-break source: {source}")
    if not bpm > 0.0 or not math.isfinite(bpm):
        raise SystemExit("--bpm must be greater than zero")
    if bars != DEFAULT_BARS:
        raise SystemExit("dense-break Golden Path currently requires exactly 8 bars")
    if wav_duration(source) < performance_duration_seconds(bpm, 1):
        raise SystemExit(f"source is too short for a dense-break loop: {source}")
    if not output.name:
        raise SystemExit("--output must name a directory")


def ensure_safe_output(repo: Path, output: Path) -> None:
    allowed = (repo / "artifacts" / "audio_qa").resolve()
    output_resolved = output.resolve()
    if allowed not in output_resolved.parents:
        raise SystemExit(f"refusing to write outside artifacts/audio_qa: {output}")


def render_feral_grid_pack(
    repo: Path,
    source: Path,
    output: Path,
    date: str,
    bpm: float,
    bars: int,
    source_start_seconds: float,
    source_window_seconds: float,
) -> None:
    command = [
        "cargo",
        "run",
        "-p",
        "riotbox-audio",
        "--bin",
        "feral_grid_pack",
        "--",
        "--source",
        str(source),
        "--output-dir",
        str(output),
        "--date",
        date,
        "--bpm",
        f"{bpm:.6f}",
        "--bars",
        str(bars),
        "--source-start-seconds",
        f"{source_start_seconds:.6f}",
        "--source-window-seconds",
        f"{source_window_seconds:.6f}",
    ]
    result = subprocess.run(
        command,
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    (output / "render.log").parent.mkdir(parents=True, exist_ok=True)
    (output / "render.log").write_text(
        result.stdout + ("\n" if result.stdout and result.stderr else "") + result.stderr
    )
    if result.returncode != 0:
        raise SystemExit(f"feral_grid_pack failed; see {output / 'render.log'}")


def render_performance(
    source: np.ndarray,
    tr909: np.ndarray,
    w30: np.ndarray,
    mc202: np.ndarray,
    bar_frames: int,
    bars: int,
) -> tuple[np.ndarray, dict[str, np.ndarray]]:
    performance = np.zeros_like(source)

    def put_bar(bar: int, mix: np.ndarray) -> None:
        start = bar * bar_frames
        end = min(start + bar_frames, performance.shape[0])
        if start >= end:
            return
        performance[start:end] = mix[start:end]

    hook_mix = saturate(source * 0.36 + w30 * 1.42 + tr909 * 0.34 + mc202 * 0.28, 1.18)
    chop_mix = saturate(source * 0.14 + w30 * 1.82 + tr909 * 0.58 + mc202 * 0.44, 1.28)
    bass_pressure = render_bass_pressure_layer(source, bar_frames, bars)
    pressure_mix = saturate(
        source * 0.08 + w30 * 0.88 + tr909 * 2.35 + mc202 * 5.80 + bass_pressure * 1.15,
        1.46,
    )
    restore_mix = saturate(
        source * 0.38 + w30 * 1.58 + tr909 * 1.24 + mc202 * 1.74 + bass_pressure * 0.62,
        1.38,
    )

    for bar in (0, 1):
        put_bar(bar, hook_mix)
    for bar in (2, 3):
        put_bar(bar, chop_mix)
    for bar in (4, 5):
        put_bar(bar, pressure_mix)

    dropout_stutter_bar = render_dropout_stutter_bar(
        source,
        tr909,
        w30,
        mc202,
        bar_frames,
        source_bar=6,
    )
    start = 6 * bar_frames
    end = min(start + dropout_stutter_bar.shape[0], performance.shape[0])
    performance[start:end] = dropout_stutter_bar[: end - start]

    put_bar(
        7,
        restore_with_hit(
            restore_mix,
            source,
            w30,
            mc202,
            tr909,
            7 * bar_frames,
            bar_frames,
        ),
    )

    performance = saturate(performance, 1.08)
    sections = {
        "chop_hook": performance[0 : min(2 * bar_frames, performance.shape[0])],
        "pressure_lift": performance[4 * bar_frames : min(6 * bar_frames, performance.shape[0])],
        "dropout_stutter": performance[6 * bar_frames : min(7 * bar_frames, performance.shape[0])],
        "restore_hit": performance[7 * bar_frames : min(8 * bar_frames, performance.shape[0])],
    }
    return performance, sections


def render_dropout_stutter_bar(
    source: np.ndarray,
    tr909: np.ndarray,
    w30: np.ndarray,
    mc202: np.ndarray,
    bar_frames: int,
    source_bar: int,
) -> np.ndarray:
    bar = np.zeros((bar_frames, CHANNELS), dtype=np.float32)
    source_start = source_bar * bar_frames
    source_end = min(source_start + bar_frames, source.shape[0])
    if source_start >= source_end:
        return bar

    base = saturate(
        source[source_start:source_end] * 0.10
        + w30[source_start:source_end] * 1.35
        + tr909[source_start:source_end] * 0.46
        + mc202[source_start:source_end] * 0.70,
        1.22,
    )
    bar[: base.shape[0]] = base

    dropout_end = bar_frames // 2
    bar[:dropout_end] *= 0.015

    grain_len = max(128, bar_frames // 32)
    grain_source_start = source_start + bar_frames // 8
    grain_source_end = min(grain_source_start + grain_len, w30.shape[0])
    if grain_source_end <= grain_source_start:
        return bar
    grain = w30[grain_source_start:grain_source_end].copy()
    grain *= hann_envelope(grain.shape[0])[:, None]

    step = max(1, bar_frames // 16)
    for index, target in enumerate(range(dropout_end, bar_frames - grain.shape[0], step)):
        decay = 1.0 - min(index, 7) * 0.07
        accent = tr909[min(source_start + target, tr909.shape[0] - 1)]
        end = target + grain.shape[0]
        bar[target:end] += grain * (1.82 * decay)
        bar[target : min(target + 96, bar.shape[0])] += accent * (0.30 * decay)

    return saturate(bar, 1.42)


def render_bass_pressure_layer(source: np.ndarray, bar_frames: int, bars: int) -> np.ndarray:
    layer = np.zeros_like(source)
    total_frames = source.shape[0]
    for bar in (4, 5, 7):
        bar_start = bar * bar_frames
        if bar_start >= total_frames:
            continue
        bar_end = min(bar_start + bar_frames, total_frames)
        frames = bar_end - bar_start
        t = np.arange(frames, dtype=np.float32) / SAMPLE_RATE
        base_frequency = 51.5 if bar != 5 else 64.0
        sine = np.sin(2.0 * np.pi * base_frequency * t).astype(np.float32)
        envelope = np.zeros(frames, dtype=np.float32)
        beat_frames = max(1, bar_frames // BEATS_PER_BAR)
        for beat in range(BEATS_PER_BAR):
            start = beat * beat_frames
            end = min(start + beat_frames, frames)
            if start >= end:
                continue
            beat_t = np.arange(end - start, dtype=np.float32) / max(1, end - start)
            punch = np.exp(-beat_t * (5.4 if beat in (0, 2) else 7.2))
            envelope[start:end] += punch * (1.0 if beat in (0, 2) else 0.62)
        source_drive = low_band_rms(source[bar_start:bar_end]) / 0.10
        gain = float(np.clip(source_drive, 0.42, 1.20)) * (0.245 if bar != 7 else 0.185)
        mono = sine * np.clip(envelope, 0.0, 1.0) * gain
        layer[bar_start:bar_end, 0] = mono
        layer[bar_start:bar_end, 1] = mono * 0.98
    return layer


def restore_with_hit(
    restore_mix: np.ndarray,
    source: np.ndarray,
    w30: np.ndarray,
    mc202: np.ndarray,
    tr909: np.ndarray,
    start: int,
    bar_frames: int,
) -> np.ndarray:
    end = min(start + bar_frames, restore_mix.shape[0])
    restored = restore_mix.copy()
    if start >= end:
        return restored
    hit_frames = min(frames_for_seconds(0.115), end - start)
    envelope = np.linspace(1.0, 0.0, hit_frames, dtype=np.float32)[:, None]
    source_hit = source[:hit_frames]
    snap = transient_emphasis(source_hit)
    restored[start : start + hit_frames] += (
        source_hit * 1.20
        + snap * 2.85
        + w30[start : start + hit_frames] * 2.10
        + mc202[start : start + hit_frames] * 3.20
        + tr909[start : start + hit_frames] * 2.20
    ) * envelope
    return saturate(restored, 1.28)


def build_report(
    source: Path,
    output: Path,
    args: argparse.Namespace,
    source_audio: np.ndarray,
    tr909: np.ndarray,
    w30: np.ndarray,
    mc202: np.ndarray,
    performance: np.ndarray,
    sections: dict[str, np.ndarray],
    bar_frames: int,
) -> dict:
    metrics = {
        "source_window": metrics_to_json(audio_metrics(source_audio)),
        "tr909": metrics_to_json(audio_metrics(tr909)),
        "w30": metrics_to_json(audio_metrics(w30)),
        "mc202": metrics_to_json(audio_metrics(mc202)),
        "full_performance": metrics_to_json(audio_metrics(performance)),
        "chop_hook": metrics_to_json(audio_metrics(sections["chop_hook"])),
        "pressure_lift": metrics_to_json(audio_metrics(sections["pressure_lift"])),
        "dropout_stutter": metrics_to_json(audio_metrics(sections["dropout_stutter"])),
        "restore_hit": metrics_to_json(audio_metrics(sections["restore_hit"])),
    }
    proof = performance_proof(source_audio, tr909, w30, mc202, performance, sections, bar_frames)
    failure_codes = failure_codes_for(metrics, proof)
    verdict = "agent_promising" if not failure_codes else "agent_fail"
    if failure_codes and len(failure_codes) <= 2:
        verdict = "agent_weak"
    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass" if not failure_codes else "fail",
        "agent_verdict": verdict,
        "human_verdict": "unverified",
        "source": str(source),
        "output": str(output),
        "bpm": args.bpm,
        "bars": args.bars,
        "structure": [
            {
                "bars": "1-2",
                "role": "break hook",
                "intent": "source character plus W-30 chop motif",
            },
            {
                "bars": "3-4",
                "role": "chop riff",
                "intent": "W-30 source chop becomes the main hook",
            },
            {
                "bars": "5-6",
                "role": "pressure lift",
                "intent": "TR-909 and MC-202 add body and bass pressure",
            },
            {
                "bars": "7",
                "role": "dropout stutter",
                "intent": "hard silence cut followed by repeated source chop",
            },
            {
                "bars": "8",
                "role": "restore hit",
                "intent": "snare/break transient and bass pressure land together",
            },
        ],
        "thresholds": {
            "min_w30_to_source_rms_ratio": MIN_W30_TO_SOURCE_RMS_RATIO,
            "min_pressure_low_band_lift_ratio": MIN_PRESSURE_LOW_BAND_LIFT_RATIO,
            "max_dropout_to_stutter_rms_ratio": MAX_DROPOUT_TO_STUTTER_RMS_RATIO,
            "min_stutter_to_hook_transient_ratio": MIN_STUTTER_TO_HOOK_TRANSIENT_RATIO,
            "min_restore_to_hook_transient_ratio": MIN_RESTORE_TO_HOOK_TRANSIENT_RATIO,
            "max_adjacent_bar_correlation": MAX_ADJACENT_BAR_CORRELATION,
            "max_source_to_performance_correlation": MAX_SOURCE_TO_PERFORMANCE_CORRELATION,
            "min_mc202_to_w30_rms_ratio": MIN_MC202_TO_W30_RMS_RATIO,
        },
        "files": {
            "source_window": "00_source_window.wav",
            "chop_hook": "01_chop_hook.wav",
            "pressure_lift": "02_pressure_lift.wav",
            "dropout_stutter": "03_dropout_stutter.wav",
            "restore_hit": "04_restore_hit.wav",
            "full_performance": "05_full_performance.wav",
        },
        "metrics": metrics,
        "proof": proof,
        "failure_codes": failure_codes,
    }


def performance_proof(
    source: np.ndarray,
    tr909: np.ndarray,
    w30: np.ndarray,
    mc202: np.ndarray,
    performance: np.ndarray,
    sections: dict[str, np.ndarray],
    bar_frames: int,
) -> dict:
    source_rms = rms(source)
    w30_rms = rms(w30)
    mc202_rms = rms(mc202)
    tr909_rms = rms(tr909)
    full_rms = rms(performance)
    pressure_low = low_band_rms(sections["pressure_lift"])
    hook_low = low_band_rms(sections["chop_hook"])
    restore_transient = max(
        transient_score(sections["restore_hit"][: frames_for_seconds(0.250)]),
        transient_score(sections["restore_hit"][: frames_for_seconds(0.500)]),
    )
    hook_transient = transient_score(sections["chop_hook"])
    dropout = sections["dropout_stutter"]
    dropout_first = dropout[: dropout.shape[0] // 2]
    dropout_second = dropout[dropout.shape[0] // 2 :]
    bar_similarity = max_adjacent_bar_correlation(performance, bar_frames)
    source_similarity = waveform_correlation(source, performance)
    return {
        "w30_to_source_rms_ratio": w30_rms / max(source_rms, 1e-9),
        "w30_to_full_performance_rms_ratio": w30_rms / max(full_rms, 1e-9),
        "generated_to_w30_rms_ratio": (tr909_rms + mc202_rms) / max(w30_rms, 1e-9),
        "pressure_low_band_lift_ratio": pressure_low / max(hook_low, 1e-9),
        "dropout_to_stutter_rms_ratio": rms(dropout_first) / max(rms(dropout_second), 1e-9),
        "stutter_to_hook_transient_ratio": transient_score(dropout_second) / max(hook_transient, 1e-9),
        "restore_to_hook_transient_ratio": restore_transient / max(hook_transient, 1e-9),
        "max_adjacent_bar_correlation": bar_similarity,
        "source_to_performance_correlation": source_similarity,
        "mc202_to_w30_rms_ratio": mc202_rms / max(w30_rms, 1e-9),
    }


def failure_codes_for(metrics: dict[str, dict], proof: dict[str, float]) -> list[str]:
    failures = []
    for name, item in metrics.items():
        if item["rms"] < 0.001:
            failures.append(f"{name}_too_quiet_or_silent")
        if item["peak_abs"] > 0.985:
            failures.append(f"{name}_near_clipping")
    if proof["w30_to_source_rms_ratio"] < MIN_W30_TO_SOURCE_RMS_RATIO:
        failures.append("w30_hook_not_present_enough")
    if proof["pressure_low_band_lift_ratio"] < MIN_PRESSURE_LOW_BAND_LIFT_RATIO:
        failures.append("pressure_section_lacks_bass_lift")
    if proof["dropout_to_stutter_rms_ratio"] > MAX_DROPOUT_TO_STUTTER_RMS_RATIO:
        failures.append("dropout_not_contrasting_with_stutter")
    if proof["stutter_to_hook_transient_ratio"] < MIN_STUTTER_TO_HOOK_TRANSIENT_RATIO:
        failures.append("stutter_lacks_transient_impact")
    if proof["restore_to_hook_transient_ratio"] < MIN_RESTORE_TO_HOOK_TRANSIENT_RATIO:
        failures.append("restore_hit_lacks_break_transient_impact")
    if proof["max_adjacent_bar_correlation"] > MAX_ADJACENT_BAR_CORRELATION:
        failures.append("performance_bars_too_similar")
    if proof["source_to_performance_correlation"] > MAX_SOURCE_TO_PERFORMANCE_CORRELATION:
        failures.append("performance_too_close_to_source_window")
    if proof["mc202_to_w30_rms_ratio"] < MIN_MC202_TO_W30_RMS_RATIO:
        failures.append("bass_pressure_too_buried_relative_to_w30")
    return failures


def write_reports(output: Path, report: dict) -> None:
    (output / "performance-report.json").write_text(json.dumps(report, indent=2) + "\n")
    lines = [
        "# Dense-Break Riotbox Performance Pack",
        "",
        f"- Result: `{report['result']}`",
        f"- Agent verdict: `{report['agent_verdict']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Source: `{report['source']}`",
        f"- BPM: `{report['bpm']}`",
        f"- Bars: `{report['bars']}`",
        "",
        "## Musician Target",
        "",
        "An 8-bar source-backed rave-punk break performance: source character, W-30-style chop hook, TR-909/MC-202 pressure lift, dropout/stutter, then restore hit.",
        "",
        "## Files",
        "",
    ]
    for role, path in report["files"].items():
        lines.append(f"- `{path}`: `{role}`")
    lines.extend(["", "## Structure", ""])
    for item in report["structure"]:
        lines.append(f"- Bars `{item['bars']}`: {item['intent']}")
    lines.extend(["", "## Proof", ""])
    for key, value in report["proof"].items():
        lines.append(f"- `{key}`: `{value:.6f}`")
    lines.extend(["", "## Failure Codes", ""])
    if report["failure_codes"]:
        lines.extend(f"- `{code}`" for code in report["failure_codes"])
    else:
        lines.append("- `none`")
    lines.extend(
        [
            "",
            "## Boundary",
            "",
            "`agent_promising` means the pack avoided known weak-output modes. "
            "It is not a final human musical pass.",
        ]
    )
    (output / "README.md").write_text("\n".join(lines) + "\n")


def read_wav(path: Path) -> np.ndarray:
    require_numpy()
    with wave.open(str(path), "rb") as wav:
        channels = wav.getnchannels()
        sample_rate = wav.getframerate()
        sample_width = wav.getsampwidth()
        frames = wav.readframes(wav.getnframes())
    if sample_rate != SAMPLE_RATE or channels != CHANNELS:
        raise SystemExit(f"expected {SAMPLE_RATE} Hz / {CHANNELS} channels: {path}")
    return decode_pcm_frames(frames, sample_width, channels, path)


def read_wav_window(path: Path, start_seconds: float, duration_seconds: float) -> np.ndarray:
    require_numpy()
    with wave.open(str(path), "rb") as wav:
        channels = wav.getnchannels()
        sample_rate = wav.getframerate()
        sample_width = wav.getsampwidth()
        if sample_rate != SAMPLE_RATE or channels != CHANNELS:
            raise SystemExit(f"expected {SAMPLE_RATE} Hz / {CHANNELS} channel WAV: {path}")
        start = frames_for_seconds(start_seconds)
        count = frames_for_seconds(duration_seconds)
        wav.setpos(start)
        frames = wav.readframes(count)
    return decode_pcm_frames(frames, sample_width, channels, path)


def read_wav_window_looped(path: Path, start_seconds: float, duration_seconds: float) -> np.ndarray:
    window = read_wav_window(path, start_seconds, wav_duration(path) - start_seconds)
    target_frames = frames_for_seconds(duration_seconds)
    if window.shape[0] == 0:
        raise SystemExit(f"source loop window is empty: {path}")
    repeats = int(math.ceil(target_frames / window.shape[0]))
    return np.tile(window, (repeats, 1))[:target_frames].astype(np.float32)


def decode_pcm_frames(frames: bytes, sample_width: int, channels: int, path: Path) -> np.ndarray:
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
        raise SystemExit(f"unsupported WAV sample width for dense-break pack: {path}")
    return samples.reshape(-1, channels)


def write_wav(path: Path, samples: np.ndarray) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    clipped = np.clip(samples, -0.98, 0.98)
    pcm = (clipped * 32767.0).astype("<i2")
    with wave.open(str(path), "wb") as wav:
        wav.setnchannels(CHANNELS)
        wav.setsampwidth(2)
        wav.setframerate(SAMPLE_RATE)
        wav.writeframes(pcm.tobytes())


def wav_duration(path: Path) -> float:
    with wave.open(str(path), "rb") as wav:
        return wav.getnframes() / float(wav.getframerate())


def audio_metrics(samples: np.ndarray) -> AudioMetrics:
    return AudioMetrics(
        rms=rms(samples),
        dbfs=20.0 * math.log10(rms(samples) + 1e-12),
        peak_abs=float(np.max(np.abs(samples))) if samples.size else 0.0,
        low_band_rms=low_band_rms(samples),
        high_band_ratio=high_band_ratio(samples),
        transient_score=transient_score(samples),
    )


def metrics_to_json(metrics: AudioMetrics) -> dict:
    return {
        "rms": metrics.rms,
        "dbfs": metrics.dbfs,
        "peak_abs": metrics.peak_abs,
        "low_band_rms": metrics.low_band_rms,
        "high_band_ratio": metrics.high_band_ratio,
        "transient_score": metrics.transient_score,
    }


def saturate(samples: np.ndarray, drive: float) -> np.ndarray:
    return np.tanh(samples * drive).astype(np.float32)


def apply_gain(samples: np.ndarray, gain: float) -> np.ndarray:
    return np.clip(samples * gain, -0.98, 0.98).astype(np.float32)


def transient_emphasis(samples: np.ndarray) -> np.ndarray:
    if samples.shape[0] < 2:
        return np.zeros_like(samples)
    previous = np.vstack([samples[0:1], samples[:-1]])
    return np.clip((samples - previous) * 2.0, -0.98, 0.98).astype(np.float32)


def rms(samples: np.ndarray) -> float:
    if samples.size == 0:
        return 0.0
    return float(np.sqrt(np.mean(samples * samples)))


def low_band_rms(samples: np.ndarray) -> float:
    if samples.size == 0:
        return 0.0
    mono = samples.mean(axis=1)
    filtered = one_pole_lowpass(mono, 165.0)
    return float(np.sqrt(np.mean(filtered * filtered)))


def high_band_ratio(samples: np.ndarray) -> float:
    if samples.shape[0] < 2:
        return 0.0
    mono = samples.mean(axis=1)
    spectrum = np.abs(np.fft.rfft(mono * np.hanning(mono.shape[0]))) + 1e-12
    freqs = np.fft.rfftfreq(mono.shape[0], 1.0 / SAMPLE_RATE)
    power = spectrum * spectrum
    return float(np.sum(power[freqs >= 2500.0]) / np.sum(power))


def transient_score(samples: np.ndarray) -> float:
    if samples.shape[0] < 2:
        return 0.0
    mono = samples.mean(axis=1)
    return float(np.percentile(np.abs(np.diff(mono)), 99.0))


def one_pole_lowpass(samples: np.ndarray, cutoff_hz: float) -> np.ndarray:
    dt = 1.0 / SAMPLE_RATE
    rc = 1.0 / (2.0 * math.pi * cutoff_hz)
    alpha = dt / (rc + dt)
    output = np.zeros_like(samples)
    state = 0.0
    for index, sample in enumerate(samples):
        state += alpha * (float(sample) - state)
        output[index] = state
    return output


def max_adjacent_bar_correlation(samples: np.ndarray, bar_frames: int) -> float:
    values = []
    for bar in range(DEFAULT_BARS - 1):
        left = mono_envelope(samples[bar * bar_frames : (bar + 1) * bar_frames])
        right = mono_envelope(samples[(bar + 1) * bar_frames : (bar + 2) * bar_frames])
        if left.size == right.size and left.size:
            values.append(float(np.dot(left, right)))
    return max(values) if values else 1.0


def waveform_correlation(left: np.ndarray, right: np.ndarray) -> float:
    count = min(left.shape[0], right.shape[0])
    if count < 2:
        return 1.0
    left_mono = left[:count].mean(axis=1)
    right_mono = right[:count].mean(axis=1)
    left_mono -= float(np.mean(left_mono))
    right_mono -= float(np.mean(right_mono))
    denom = float(np.linalg.norm(left_mono) * np.linalg.norm(right_mono))
    if denom <= 1e-12:
        return 1.0
    return float(np.dot(left_mono, right_mono) / denom)


def mono_envelope(samples: np.ndarray) -> np.ndarray:
    if samples.shape[0] == 0:
        return np.array([], dtype=np.float32)
    mono = np.abs(samples.mean(axis=1))
    bins = np.array(
        [float(np.mean(chunk)) for chunk in np.array_split(mono, 64)],
        dtype=np.float32,
    )
    bins -= float(np.mean(bins))
    norm = float(np.linalg.norm(bins))
    if norm > 1e-9:
        bins /= norm
    return bins


def hann_envelope(size: int) -> np.ndarray:
    if size <= 1:
        return np.ones((size,), dtype=np.float32)
    return np.hanning(size).astype(np.float32)


def performance_duration_seconds(bpm: float, bars: int) -> float:
    return bars * BEATS_PER_BAR * 60.0 / bpm


def frames_for_seconds(seconds: float) -> int:
    return int(round(seconds * SAMPLE_RATE))


def frames_for_beats(bpm: float, beats: int) -> int:
    return int(round(beats * SAMPLE_RATE * 60.0 / bpm))


def require_numpy() -> None:
    global np
    if np is None:
        import numpy as numpy

        np = numpy


if __name__ == "__main__":
    sys.exit(main())

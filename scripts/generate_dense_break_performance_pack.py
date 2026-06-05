#!/usr/bin/env python3
"""Generate the dense-break 8-bar Riotbox performance Golden Path pack."""

from __future__ import annotations

import argparse
import json
import math
import shutil
import struct
import subprocess
import sys
import wave
import zlib
from dataclasses import asdict, dataclass
from pathlib import Path

from audio_qa_evidence_boundary import apply_evidence_boundary


SAMPLE_RATE = 44_100
CHANNELS = 2
DEFAULT_SOURCE = Path("data/test_audio/examples/Beat03_130BPM(Full).wav")
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-dense-break-performance-pack")
DEFAULT_DATE = "local-dense-break-performance-pack"
DEFAULT_BPM = 130.0
DEFAULT_BARS = 8
BEATS_PER_BAR = 4
SCHEMA = "riotbox.dense_break_performance_pack.v1"
AGENT_REVIEW_SCHEMA = "riotbox.agent_musical_review_pack.v1"
MIN_W30_TO_SOURCE_RMS_RATIO = 0.18
MIN_PRESSURE_LOW_BAND_LIFT_RATIO = 1.12
MAX_DROPOUT_TO_STUTTER_RMS_RATIO = 0.18
MIN_STUTTER_TO_HOOK_TRANSIENT_RATIO = 0.58
MIN_RESTORE_TO_HOOK_TRANSIENT_RATIO = 0.85
MAX_ADJACENT_BAR_CORRELATION = 0.985
MAX_SOURCE_TO_PERFORMANCE_CORRELATION = 0.975
MIN_MC202_TO_W30_RMS_RATIO = 0.12
MIN_FULL_TO_SOURCE_RMS_RATIO = 0.78
MIN_HOOK_TO_SOURCE_TRANSIENT_RATIO = 0.48
MIN_PRESSURE_TO_HOOK_RMS_RATIO = 1.30
MIN_RESTORE_TO_PRESSURE_RMS_RATIO = 1.12
MIN_REBUILD_ONLY_TO_FULL_RMS_RATIO = 0.42
MIN_REBUILD_ONLY_TO_SOURCE_RMS_RATIO = 0.30
MIN_REBUILD_ONLY_RESTORE_TO_PRESSURE_RMS_RATIO = 1.08
MAX_REBUILD_ONLY_TO_SOURCE_CORRELATION = 0.920
MAX_SOURCE_ON_TO_REBUILD_ONLY_CORRELATION = 0.995
TARGET_PERFORMANCE_PEAK = 0.92

np = None


@dataclass(frozen=True)
class AudioMetrics:
    rms: float
    dbfs: float
    peak_abs: float
    low_band_rms: float
    high_band_ratio: float
    transient_score: float


@dataclass(frozen=True)
class PressureLiftPolicy:
    source_aware: bool
    source_family: str
    lift_shape: str
    lift_intent: str
    source_bleed_gain: float
    hook_bleed_gain: float
    tr909_drive: float
    break_snap_drive: float
    mc202_drive: float
    bass_drive: float
    bar4_intensity: float
    bar5_intensity: float
    bar4_bass_frequency_hz: float
    bar5_bass_frequency_hz: float


@dataclass(frozen=True)
class ArrangementPolicy:
    source_aware: bool
    source_family: str
    role_order: tuple[str, ...]
    role_order_signature: str
    arrangement_shape: str
    arrangement_intent: str


@dataclass(frozen=True)
class DenseBreakSourcePolicy:
    source_aware: bool
    pressure_shape: str
    stutter_density: str
    restore_hit_shape: str
    pressure_lift_policy: PressureLiftPolicy
    arrangement_policy: ArrangementPolicy
    bass_bar4_frequency_hz: float
    bass_bar5_frequency_hz: float
    bass_restore_frequency_hz: float
    pressure_gain: float
    bass_gain: float
    stutter_step_divisor: int
    stutter_grain_beat_offset: float
    restore_snap_gain: float
    source_low_band_rms: float
    source_high_band_ratio: float
    source_transient_score: float


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
    w30 = apply_gain(
        w30[:frame_count],
        source_relative_gain(
            source_audio,
            w30[:frame_count],
            target_ratio=MIN_W30_TO_SOURCE_RMS_RATIO * 1.10,
            minimum_gain=1.22,
            maximum_gain=2.35,
        ),
    )
    mc202 = apply_gain(mc202[:frame_count], 1.35)

    bar_frames = frames_for_beats(args.bpm, BEATS_PER_BAR)
    source_policy = dense_break_source_policy(source_audio, bar_frames)
    performance, sections = render_performance(
        source_audio,
        tr909,
        w30,
        mc202,
        source_policy,
        bar_frames,
        args.bars,
    )
    rebuild_only_performance, rebuild_only_sections = render_performance(
        source_audio,
        tr909,
        w30,
        mc202,
        source_policy,
        bar_frames,
        args.bars,
        source_layer_gain=0.0,
    )

    audio_files = {
        "source_window": "00_source_window.wav",
        "chop_hook": "01_chop_hook.wav",
        "pressure_lift": "02_pressure_lift.wav",
        "dropout_stutter": "03_dropout_stutter.wav",
        "restore_hit": "04_restore_hit.wav",
        "full_performance": "05_full_performance.wav",
        "rebuild_only_performance": "06_rebuild_only_performance.wav",
    }
    write_wav(output / audio_files["source_window"], source_audio)
    write_wav(output / audio_files["chop_hook"], sections["chop_hook"])
    write_wav(output / audio_files["pressure_lift"], sections["pressure_lift"])
    write_wav(output / audio_files["dropout_stutter"], sections["dropout_stutter"])
    write_wav(output / audio_files["restore_hit"], sections["restore_hit"])
    write_wav(output / audio_files["full_performance"], performance)
    write_wav(output / audio_files["rebuild_only_performance"], rebuild_only_performance)
    visual_files = write_visual_evidence(
        output,
        {
            "source_window": source_audio,
            "chop_hook": sections["chop_hook"],
            "pressure_lift": sections["pressure_lift"],
            "dropout_stutter": sections["dropout_stutter"],
            "restore_hit": sections["restore_hit"],
            "full_performance": performance,
            "rebuild_only_performance": rebuild_only_performance,
        },
    )

    report = build_report(
        source,
        output,
        args,
        audio_files,
        visual_files,
        source_audio,
        tr909,
        w30,
        mc202,
        source_policy,
        performance,
        sections,
        rebuild_only_performance,
        rebuild_only_sections,
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


def dense_break_source_policy(source: np.ndarray, bar_frames: int) -> DenseBreakSourcePolicy:
    first_two_bars = source[: min(2 * bar_frames, source.shape[0])]
    profile = audio_metrics(first_two_bars)
    pressure_lift_policy = pressure_lift_policy_for(
        low_band_rms=profile.low_band_rms,
        high_band_ratio=profile.high_band_ratio,
        transient_score=profile.transient_score,
    )
    arrangement_policy = arrangement_policy_for(pressure_lift_policy.source_family)

    if pressure_lift_policy.source_family == "dense_break":
        pressure_shape = "transient-forward break pressure"
        stutter_density = "tight sixteenth stutter"
        restore_hit_shape = "hard transient restore"
        stutter_step_divisor = 16
        stutter_grain_beat_offset = 0.50
        restore_snap_gain = 1.10
    elif pressure_lift_policy.source_family == "sparse_bass_pressure":
        pressure_shape = "low-band shove"
        stutter_density = "wide eighth stutter"
        restore_hit_shape = "bass-weighted restore"
        stutter_step_divisor = 12
        stutter_grain_beat_offset = 1.00
        restore_snap_gain = 1.28
    elif pressure_lift_policy.source_family == "tonal_hook":
        pressure_shape = "tonal-hook support lift"
        stutter_density = "hook-preserving eighth stutter"
        restore_hit_shape = "hook-forward restore"
        stutter_step_divisor = 12
        stutter_grain_beat_offset = 0.75
        restore_snap_gain = 1.04
    else:
        pressure_shape = "thin-source support lift"
        stutter_density = "busy recovery stutter"
        restore_hit_shape = "snap-assisted restore"
        stutter_step_divisor = 18
        stutter_grain_beat_offset = 0.25
        restore_snap_gain = 1.18

    if pressure_lift_policy.source_family == "tonal_hook":
        bass_restore = 51.5
        pressure_gain = 1.08
        bass_gain = 1.08
    elif pressure_lift_policy.source_family == "sparse_bass_pressure":
        bass_restore = 48.0
        pressure_gain = 1.04
        bass_gain = 1.08
    elif pressure_lift_policy.source_family == "dense_break":
        bass_restore = 48.0
        pressure_gain = 0.96
        bass_gain = 0.98
    else:
        bass_restore = 51.5
        pressure_gain = 1.02
        bass_gain = 1.00

    return DenseBreakSourcePolicy(
        source_aware=True,
        pressure_shape=pressure_shape,
        stutter_density=stutter_density,
        restore_hit_shape=restore_hit_shape,
        pressure_lift_policy=pressure_lift_policy,
        arrangement_policy=arrangement_policy,
        bass_bar4_frequency_hz=pressure_lift_policy.bar4_bass_frequency_hz,
        bass_bar5_frequency_hz=pressure_lift_policy.bar5_bass_frequency_hz,
        bass_restore_frequency_hz=bass_restore,
        pressure_gain=pressure_gain,
        bass_gain=bass_gain,
        stutter_step_divisor=stutter_step_divisor,
        stutter_grain_beat_offset=stutter_grain_beat_offset,
        restore_snap_gain=restore_snap_gain,
        source_low_band_rms=profile.low_band_rms,
        source_high_band_ratio=profile.high_band_ratio,
        source_transient_score=profile.transient_score,
    )


def arrangement_policy_for(source_family: str) -> ArrangementPolicy:
    if source_family == "dense_break":
        roles = ("hook", "hook", "chop", "chop", "pressure", "pressure", "dropout", "restore")
        shape = "classic break slam"
        intent = "establish hook, build chop pressure, cut hard, then restore with impact"
    elif source_family == "tonal_hook":
        roles = ("hook", "chop", "hook", "chop", "pressure", "pressure", "dropout", "restore")
        shape = "hook-return lift"
        intent = "bring the tonal hook back before pressure so the riff stays readable"
    elif source_family == "sparse_bass_pressure":
        roles = ("hook", "pressure", "chop", "hook", "chop", "pressure", "dropout", "restore")
        shape = "early bass shove"
        intent = "introduce bass pressure early, re-state the hook, then lift again before the cut"
    else:
        roles = ("hook", "chop", "hook", "pressure", "chop", "pressure", "dropout", "restore")
        shape = "cautious recovery lift"
        intent = "avoid pretending weak material is a dense break while still proving contrast"
    return ArrangementPolicy(
        source_aware=True,
        source_family=source_family,
        role_order=roles,
        role_order_signature=">".join(roles),
        arrangement_shape=shape,
        arrangement_intent=intent,
    )


def pressure_lift_policy_for(
    low_band_rms: float,
    high_band_ratio: float,
    transient_score: float,
) -> PressureLiftPolicy:
    if high_band_ratio >= 0.050 and transient_score >= 0.080:
        return PressureLiftPolicy(
            source_aware=True,
            source_family="dense_break",
            lift_shape="transient-pressure slam",
            lift_intent="snare and break transient hit with low-band shove",
            source_bleed_gain=0.055,
            hook_bleed_gain=0.74,
            tr909_drive=1.08,
            break_snap_drive=1.14,
            mc202_drive=1.04,
            bass_drive=1.02,
            bar4_intensity=0.94,
            bar5_intensity=1.08,
            bar4_bass_frequency_hz=38.0,
            bar5_bass_frequency_hz=45.0,
        )
    if low_band_rms < 0.020 and high_band_ratio < 0.020 and transient_score < 0.020:
        return PressureLiftPolicy(
            source_aware=True,
            source_family="thin_or_uncertain",
            lift_shape="support-recovery lift",
            lift_intent="add pressure without pretending the weak source proved a strong lift",
            source_bleed_gain=0.075,
            hook_bleed_gain=0.88,
            tr909_drive=1.00,
            break_snap_drive=1.02,
            mc202_drive=1.02,
            bass_drive=1.00,
            bar4_intensity=0.96,
            bar5_intensity=1.07,
            bar4_bass_frequency_hz=45.0,
            bar5_bass_frequency_hz=58.0,
        )
    if low_band_rms < 0.120:
        return PressureLiftPolicy(
            source_aware=True,
            source_family="tonal_hook",
            lift_shape="hook-support lift",
            lift_intent="keep the tonal hook readable while pressure rises underneath",
            source_bleed_gain=0.100,
            hook_bleed_gain=1.02,
            tr909_drive=0.92,
            break_snap_drive=1.08,
            mc202_drive=0.86,
            bass_drive=0.88,
            bar4_intensity=0.88,
            bar5_intensity=1.00,
            bar4_bass_frequency_hz=42.0,
            bar5_bass_frequency_hz=49.0,
        )
    if low_band_rms >= 0.120:
        return PressureLiftPolicy(
            source_aware=True,
            source_family="sparse_bass_pressure",
            lift_shape="bass-rebuild lift",
            lift_intent="turn sparse source weight into a harder bass-pressure rise",
            source_bleed_gain=0.045,
            hook_bleed_gain=0.62,
            tr909_drive=1.16,
            break_snap_drive=0.96,
            mc202_drive=1.18,
            bass_drive=1.12,
            bar4_intensity=0.97,
            bar5_intensity=1.06,
            bar4_bass_frequency_hz=36.0,
            bar5_bass_frequency_hz=43.0,
        )
    raise AssertionError("pressure_lift_policy_for source-family classification is exhaustive")


def render_performance(
    source: np.ndarray,
    tr909: np.ndarray,
    w30: np.ndarray,
    mc202: np.ndarray,
    source_policy: DenseBreakSourcePolicy,
    bar_frames: int,
    bars: int,
    source_layer_gain: float = 1.0,
) -> tuple[np.ndarray, dict[str, np.ndarray]]:
    performance = np.zeros_like(source)
    hook_riff = render_w30_hook_riff_layer(w30, source, bar_frames, bars)
    break_snap = render_break_snap_layer(source, tr909, w30, bar_frames, bars)
    bass_pressure = render_bass_pressure_layer(source, source_policy, bar_frames, bars)
    lift_policy = source_policy.pressure_lift_policy
    role_order = source_policy.arrangement_policy.role_order

    def put_bar(bar: int, mix: np.ndarray) -> None:
        start = bar * bar_frames
        end = min(start + bar_frames, performance.shape[0])
        if start >= end:
            return
        performance[start:end] = mix[start:end]

    hook_mix = glue_bus(
        source * (0.50 * source_layer_gain)
        + w30 * 1.38
        + hook_riff * 1.62
        + tr909 * 0.62
        + break_snap * 1.50
        + mc202 * 0.34,
        drive=1.04,
        slam=0.05,
    )
    chop_mix = glue_bus(
        source * (0.16 * source_layer_gain)
        + w30 * 1.54
        + hook_riff * 1.78
        + tr909 * 0.78
        + break_snap * 1.36
        + mc202 * 0.58,
        drive=1.24,
        slam=0.18,
    )
    pressure_mix = saturate(
        source * (lift_policy.source_bleed_gain * source_layer_gain)
        + w30 * 0.84
        + hook_riff * lift_policy.hook_bleed_gain
        + tr909 * (2.28 + lift_policy.tr909_drive * 0.52)
        + break_snap * (1.36 * lift_policy.break_snap_drive)
        + mc202 * (5.00 + lift_policy.mc202_drive * 1.42)
        + bass_pressure * (1.14 + lift_policy.bass_drive * 0.62),
        1.58,
    )
    pressure_mix = normalize_peak(glue_bus(pressure_mix, drive=1.34, slam=0.30), 0.79)
    restore_mix = glue_bus(
        source * (0.28 * source_layer_gain)
        + w30 * 1.76
        + hook_riff * 1.46
        + tr909 * 2.78
        + break_snap * 3.64
        + mc202 * 3.65
        + bass_pressure * 1.68,
        drive=1.72,
        slam=0.40,
    )

    pressure_index = 0
    for bar, role in enumerate(role_order[:bars]):
        if role == "hook":
            put_bar(bar, hook_mix)
        elif role == "chop":
            put_bar(bar, chop_mix)
        elif role == "pressure":
            intensity = (
                lift_policy.bar4_intensity if pressure_index == 0 else lift_policy.bar5_intensity
            )
            put_bar(bar, apply_gain(pressure_mix, intensity))
            pressure_index += 1
        elif role == "dropout":
            dropout_stutter_bar = render_dropout_stutter_bar(
                source,
                tr909,
                w30,
                mc202,
                hook_riff,
                break_snap,
                source_policy,
                bar_frames,
                source_bar=bar,
                source_layer_gain=source_layer_gain,
            )
            start = bar * bar_frames
            end = min(start + dropout_stutter_bar.shape[0], performance.shape[0])
            performance[start:end] = dropout_stutter_bar[: end - start]
        elif role == "restore":
            put_bar(
                bar,
                restore_with_hit(
                    restore_mix,
                    source,
                    w30,
                    mc202,
                    tr909,
                    source_policy,
                    bar * bar_frames,
                    bar_frames,
                    source_layer_gain=source_layer_gain,
                ),
            )
        else:
            raise ValueError(f"unsupported arrangement role: {role}")

    performance = normalize_peak(glue_bus(performance, drive=1.22, slam=0.22), TARGET_PERFORMANCE_PEAK)
    sections = {
        "chop_hook": concatenate_role_bars(
            performance, bar_frames, role_order, {"hook", "chop"}, max_bars=2
        ),
        "pressure_lift": concatenate_role_bars(performance, bar_frames, role_order, {"pressure"}),
        "dropout_stutter": concatenate_role_bars(performance, bar_frames, role_order, {"dropout"}),
        "restore_hit": concatenate_role_bars(performance, bar_frames, role_order, {"restore"}),
    }
    return performance, sections


def concatenate_role_bars(
    performance: np.ndarray,
    bar_frames: int,
    role_order: tuple[str, ...],
    roles: set[str],
    max_bars: int | None = None,
) -> np.ndarray:
    chunks = []
    for bar, role in enumerate(role_order):
        if role not in roles:
            continue
        if max_bars is not None and len(chunks) >= max_bars:
            break
        start = bar * bar_frames
        end = min(start + bar_frames, performance.shape[0])
        if start < end:
            chunks.append(performance[start:end])
    if not chunks:
        return np.zeros((0, CHANNELS), dtype=np.float32)
    return np.concatenate(chunks, axis=0)


def arrangement_structure(policy: ArrangementPolicy) -> list[dict[str, str]]:
    role_intents = {
        "hook": "source character plus W-30 chop motif",
        "chop": "W-30 source chop becomes the main hook movement",
        "pressure": "TR-909 and MC-202 add body and bass pressure",
        "dropout": "hard silence cut followed by repeated source chop",
        "restore": "snare/break transient and bass pressure land together",
    }
    groups = []
    start = 0
    roles = policy.role_order
    while start < len(roles):
        end = start + 1
        while end < len(roles) and roles[end] == roles[start]:
            end += 1
        bars = f"{start + 1}" if end == start + 1 else f"{start + 1}-{end}"
        role = roles[start]
        groups.append(
            {
                "bars": bars,
                "role": role.replace("_", " "),
                "intent": role_intents[role],
            }
        )
        start = end
    return groups


def arrangement_failure_codes(policy: ArrangementPolicy) -> list[str]:
    failures = []
    role_counts = {role: policy.role_order.count(role) for role in set(policy.role_order)}
    if len(policy.role_order) != DEFAULT_BARS:
        failures.append("arrangement_role_order_not_8_bars")
    for role in ("hook", "chop", "pressure", "dropout", "restore"):
        if role_counts.get(role, 0) == 0:
            failures.append(f"arrangement_missing_{role}_role")
    if role_counts.get("pressure", 0) < 2:
        failures.append("arrangement_pressure_lift_too_short")
    if policy.role_order[-2:] != ("dropout", "restore"):
        failures.append("arrangement_destructive_restore_tail_missing")
    return failures


def arrangement_failure_routes(failures: list[str]) -> list[dict[str, object]]:
    if not failures:
        return []
    from route_weak_output_fixes import route_signals

    routes = []
    for code in failures:
        route = route_signals([code], {}, [])
        routes.append(
            {
                "failure_code": code,
                "proposed_next_fix_category": route["proposed_next_fix_category"],
                "proposed_fix_categories": route["proposed_fix_categories"],
                "main_weakness": route["main_weakness"],
            }
        )
    return routes


def arrangement_role_frequency_policy(
    source_policy: DenseBreakSourcePolicy,
) -> dict[int, float]:
    frequencies = {}
    pressure_index = 0
    for bar, role in enumerate(source_policy.arrangement_policy.role_order):
        if role == "pressure":
            frequencies[bar] = (
                source_policy.bass_bar4_frequency_hz
                if pressure_index == 0
                else source_policy.bass_bar5_frequency_hz
            )
            pressure_index += 1
        elif role == "restore":
            frequencies[bar] = source_policy.bass_restore_frequency_hz
    return frequencies


def pressure_role_count(source_policy: DenseBreakSourcePolicy) -> int:
    return source_policy.arrangement_policy.role_order.count("pressure")


def destructive_role_count(source_policy: DenseBreakSourcePolicy) -> int:
    roles = source_policy.arrangement_policy.role_order
    return roles.count("dropout") + roles.count("restore")


def role_order_hash(policy: ArrangementPolicy) -> float:
    return float(zlib.crc32(policy.role_order_signature.encode("utf-8")) & 0xFFFFFFFF)


def pressure_role_first_two(
    sections: dict[str, np.ndarray], bar_frames: int
) -> tuple[np.ndarray, np.ndarray]:
    pressure = sections["pressure_lift"]
    first = pressure[: min(bar_frames, pressure.shape[0])]
    second = pressure[min(bar_frames, pressure.shape[0]) : min(2 * bar_frames, pressure.shape[0])]
    return first, second


def render_dropout_stutter_bar(
    source: np.ndarray,
    tr909: np.ndarray,
    w30: np.ndarray,
    mc202: np.ndarray,
    hook_riff: np.ndarray,
    break_snap: np.ndarray,
    source_policy: DenseBreakSourcePolicy,
    bar_frames: int,
    source_bar: int,
    source_layer_gain: float,
) -> np.ndarray:
    bar = np.zeros((bar_frames, CHANNELS), dtype=np.float32)
    source_start = source_bar * bar_frames
    source_end = min(source_start + bar_frames, source.shape[0])
    if source_start >= source_end:
        return bar

    base = saturate(
        source[source_start:source_end] * (0.10 * source_layer_gain)
        + w30[source_start:source_end] * 1.35
        + tr909[source_start:source_end] * 0.46
        + mc202[source_start:source_end] * 0.70,
        1.22,
    )
    bar[: base.shape[0]] = base

    dropout_end = bar_frames // 2
    bar[:dropout_end] *= 0.015

    grain_len = max(128, bar_frames // 32)
    beat_frames = max(1, bar_frames // BEATS_PER_BAR)
    grain_source_start = source_start + int(round(source_policy.stutter_grain_beat_offset * beat_frames))
    grain_source_end = min(grain_source_start + grain_len, w30.shape[0])
    if grain_source_end <= grain_source_start:
        return bar
    grain = w30[grain_source_start:grain_source_end].copy()
    grain *= hann_envelope(grain.shape[0])[:, None]
    source_snap_grain = transient_emphasis(source[grain_source_start:grain_source_end].copy())
    source_snap_grain *= impact_envelope(source_snap_grain.shape[0], decay=0.040)[:, None]
    source_snap_grain = normalize_peak(source_snap_grain, 0.78)

    step = max(1, bar_frames // source_policy.stutter_step_divisor)
    for index, target in enumerate(range(dropout_end, bar_frames - grain.shape[0], step)):
        decay = 1.0 - min(index, 7) * 0.07
        accent = tr909[min(source_start + target, tr909.shape[0] - 1)]
        end = target + grain.shape[0]
        riff = hook_riff[min(source_start + target, hook_riff.shape[0] - 1)]
        snap = break_snap[min(source_start + target, break_snap.shape[0] - 1)]
        bar[target:end] += grain * (3.15 * decay * source_policy.pressure_gain)
        bar[target:end] += source_snap_grain[: end - target] * (2.05 * decay * source_policy.restore_snap_gain)
        bar[target : min(target + 96, bar.shape[0])] += accent * (0.58 * decay)
        bar[target : min(target + 160, bar.shape[0])] += (riff + snap) * (1.02 * decay)

    return normalize_peak(saturate(bar, 1.78), 0.90)


def render_w30_hook_riff_layer(
    w30: np.ndarray,
    source: np.ndarray,
    bar_frames: int,
    bars: int,
) -> np.ndarray:
    layer = np.zeros_like(w30)
    grain_len = min(frames_for_seconds(0.090), max(1, bar_frames // 8))
    first_bar = w30[: min(bar_frames, w30.shape[0])]
    grain_start = strongest_window_start(first_bar, grain_len)
    grain_end = min(grain_start + grain_len, w30.shape[0], source.shape[0])
    if grain_end <= grain_start:
        return layer

    grain = w30[grain_start:grain_end].copy()
    grain += transient_emphasis(source[grain_start:grain_end]) * 0.42
    grain *= decay_envelope(grain.shape[0], attack=0.010, decay=0.135)[:, None]

    beat_frames = max(1, bar_frames // BEATS_PER_BAR)
    patterns = {
        0: [(0.00, 0.95, False), (1.50, 0.70, False), (2.50, 0.88, False), (3.25, 0.62, True)],
        1: [(0.00, 1.06, False), (0.75, 0.52, True), (2.00, 0.92, False), (3.50, 0.78, False)],
        2: [(0.00, 1.12, False), (0.50, 0.54, True), (1.50, 0.72, False), (2.25, 1.00, False), (3.25, 0.70, True)],
        3: [(0.00, 1.18, False), (0.75, 0.60, True), (1.75, 0.74, False), (2.50, 0.96, False), (3.50, 0.88, True)],
        4: [(0.00, 0.76, False), (2.00, 0.68, False), (3.50, 0.54, True)],
        5: [(0.00, 0.82, False), (1.50, 0.58, True), (2.50, 0.72, False), (3.50, 0.56, True)],
        7: [(0.00, 1.05, False), (1.00, 0.64, False), (2.00, 0.88, False), (3.25, 0.72, True)],
    }
    for bar in range(min(bars, DEFAULT_BARS)):
        for beat, gain, reverse in patterns.get(bar, []):
            target = bar * bar_frames + int(round(beat * beat_frames))
            if target >= layer.shape[0]:
                continue
            stab = grain[::-1] if reverse else grain
            end = min(target + stab.shape[0], layer.shape[0])
            if end > target:
                layer[target:end] += stab[: end - target] * gain
    return saturate(layer, 1.35)


def render_break_snap_layer(
    source: np.ndarray,
    tr909: np.ndarray,
    w30: np.ndarray,
    bar_frames: int,
    bars: int,
) -> np.ndarray:
    layer = np.zeros_like(source)
    hit_frames = min(frames_for_seconds(0.052), max(1, bar_frames // 10))
    beat_frames = max(1, bar_frames // BEATS_PER_BAR)
    source_hit_start = strongest_window_start(source[: min(bar_frames, source.shape[0])], hit_frames)
    source_hit = source[source_hit_start : min(source_hit_start + hit_frames, source.shape[0])]
    if source_hit.shape[0] == 0:
        return layer
    source_snap = transient_emphasis(source_hit) * 2.85 + source_hit * 0.28
    source_snap *= impact_envelope(source_snap.shape[0], decay=0.034)[:, None]
    source_snap = normalize_peak(source_snap, 0.86)

    bar_gains = {
        0: [1.08, 0.76, 0.92, 0.82],
        1: [1.22, 0.82, 0.96, 0.90],
        2: [1.20, 0.82, 1.06, 0.88],
        3: [1.26, 0.84, 1.10, 0.96],
        4: [1.24, 0.92, 1.12, 0.98],
        5: [1.28, 0.96, 1.16, 1.04],
        7: [1.40, 0.86, 1.06, 0.92],
    }
    for bar in range(min(bars, DEFAULT_BARS)):
        gains = bar_gains.get(bar)
        if gains is None:
            continue
        for beat, gain in enumerate(gains):
            target = bar * bar_frames + beat * beat_frames
            if target >= layer.shape[0]:
                continue
            end = min(target + source_snap.shape[0], layer.shape[0])
            if end <= target:
                continue
            tr_slice = tr909[target:end]
            w30_slice = w30[target:end]
            snap = source_snap[: end - target] * gain
            snap += transient_emphasis(tr_slice) * (0.62 * gain)
            snap += transient_emphasis(w30_slice) * (0.34 * gain)
            layer[target:end] += snap
    return normalize_peak(saturate(layer, 1.80), 0.82)


def render_bass_pressure_layer(
    source: np.ndarray,
    source_policy: DenseBreakSourcePolicy,
    bar_frames: int,
    bars: int,
) -> np.ndarray:
    layer = np.zeros_like(source)
    total_frames = source.shape[0]
    frequencies = arrangement_role_frequency_policy(source_policy)
    for bar, base_frequency in frequencies.items():
        bar_start = bar * bar_frames
        if bar_start >= total_frames:
            continue
        bar_end = min(bar_start + bar_frames, total_frames)
        frames = bar_end - bar_start
        t = np.arange(frames, dtype=np.float32) / SAMPLE_RATE
        sine = np.sin(2.0 * np.pi * base_frequency * t).astype(np.float32)
        harmonic = np.sin(2.0 * np.pi * base_frequency * 2.0 * t).astype(np.float32)
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
        role = source_policy.arrangement_policy.role_order[bar]
        gain = (
            float(np.clip(source_drive, 0.44, 1.24))
            * source_policy.bass_gain
            * (0.245 if role == "restore" else 0.305)
        )
        mono = (sine + harmonic * 0.18) * np.clip(envelope, 0.0, 1.0) * gain
        layer[bar_start:bar_end, 0] = mono
        layer[bar_start:bar_end, 1] = mono * 0.98
    return layer


def restore_with_hit(
    restore_mix: np.ndarray,
    source: np.ndarray,
    w30: np.ndarray,
    mc202: np.ndarray,
    tr909: np.ndarray,
    source_policy: DenseBreakSourcePolicy,
    start: int,
    bar_frames: int,
    source_layer_gain: float,
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
        source_hit * (1.35 * source_layer_gain)
        + snap * (4.80 * source_policy.restore_snap_gain)
        + w30[start : start + hit_frames] * 2.62
        + mc202[start : start + hit_frames] * 4.05
        + tr909[start : start + hit_frames] * 3.45
    ) * envelope
    return normalize_peak(glue_bus(restored, drive=1.95, slam=0.44), 0.98)


def build_report(
    source: Path,
    output: Path,
    args: argparse.Namespace,
    audio_files: dict[str, str],
    visual_files: dict[str, dict[str, str]],
    source_audio: np.ndarray,
    tr909: np.ndarray,
    w30: np.ndarray,
    mc202: np.ndarray,
    source_policy: DenseBreakSourcePolicy,
    performance: np.ndarray,
    sections: dict[str, np.ndarray],
    rebuild_only_performance: np.ndarray,
    rebuild_only_sections: dict[str, np.ndarray],
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
        "rebuild_only_performance": metrics_to_json(audio_metrics(rebuild_only_performance)),
        "rebuild_only_chop_hook": metrics_to_json(audio_metrics(rebuild_only_sections["chop_hook"])),
        "rebuild_only_pressure_lift": metrics_to_json(
            audio_metrics(rebuild_only_sections["pressure_lift"])
        ),
        "rebuild_only_dropout_stutter": metrics_to_json(
            audio_metrics(rebuild_only_sections["dropout_stutter"])
        ),
        "rebuild_only_restore_hit": metrics_to_json(
            audio_metrics(rebuild_only_sections["restore_hit"])
        ),
    }
    proof = performance_proof(
        source_audio,
        tr909,
        w30,
        mc202,
        source_policy,
        performance,
        sections,
        rebuild_only_performance,
        rebuild_only_sections,
        bar_frames,
    )
    failure_codes = failure_codes_for(metrics, proof)
    verdict = "agent_promising" if not failure_codes else "agent_fail"
    if failure_codes and len(failure_codes) <= 2:
        verdict = "agent_weak"
    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass" if not failure_codes else "fail",
        "agent_verdict": verdict,
        "human_verdict": "unverified",
        "source": str(source),
        "output": str(output),
        "bpm": args.bpm,
        "bars": args.bars,
        "structure": arrangement_structure(source_policy.arrangement_policy),
        "source_policy": {
            "source_aware": source_policy.source_aware,
            "decisions": asdict(source_policy),
            "pressure_lift_policy": asdict(source_policy.pressure_lift_policy),
            "arrangement_policy": asdict(source_policy.arrangement_policy),
            "arrangement_failure_routes": arrangement_failure_routes(
                arrangement_failure_codes(source_policy.arrangement_policy)
            ),
            "scripted_boundaries": [
                "8-bar role grammar remains scripted even though role order is source-aware",
                "arrangement policy is diagnostic and does not claim human musical approval",
                "roles remain bounded to hook, chop, pressure, dropout, restore",
                "human_verdict remains unverified until structured listening review",
            ],
        },
        "thresholds": {
            "min_w30_to_source_rms_ratio": MIN_W30_TO_SOURCE_RMS_RATIO,
            "min_pressure_low_band_lift_ratio": MIN_PRESSURE_LOW_BAND_LIFT_RATIO,
            "max_dropout_to_stutter_rms_ratio": MAX_DROPOUT_TO_STUTTER_RMS_RATIO,
            "min_stutter_to_hook_transient_ratio": MIN_STUTTER_TO_HOOK_TRANSIENT_RATIO,
            "min_restore_to_hook_transient_ratio": MIN_RESTORE_TO_HOOK_TRANSIENT_RATIO,
            "max_adjacent_bar_correlation": MAX_ADJACENT_BAR_CORRELATION,
            "max_source_to_performance_correlation": MAX_SOURCE_TO_PERFORMANCE_CORRELATION,
            "min_mc202_to_w30_rms_ratio": MIN_MC202_TO_W30_RMS_RATIO,
            "min_full_to_source_rms_ratio": MIN_FULL_TO_SOURCE_RMS_RATIO,
            "min_hook_to_source_transient_ratio": MIN_HOOK_TO_SOURCE_TRANSIENT_RATIO,
            "min_pressure_to_hook_rms_ratio": MIN_PRESSURE_TO_HOOK_RMS_RATIO,
            "min_restore_to_pressure_rms_ratio": MIN_RESTORE_TO_PRESSURE_RMS_RATIO,
            "min_rebuild_only_to_full_rms_ratio": MIN_REBUILD_ONLY_TO_FULL_RMS_RATIO,
            "min_rebuild_only_to_source_rms_ratio": MIN_REBUILD_ONLY_TO_SOURCE_RMS_RATIO,
            "min_rebuild_only_restore_to_pressure_rms_ratio": (
                MIN_REBUILD_ONLY_RESTORE_TO_PRESSURE_RMS_RATIO
            ),
            "max_rebuild_only_to_source_correlation": MAX_REBUILD_ONLY_TO_SOURCE_CORRELATION,
            "max_source_on_to_rebuild_only_correlation": MAX_SOURCE_ON_TO_REBUILD_ONLY_CORRELATION,
            "target_performance_peak": TARGET_PERFORMANCE_PEAK,
        },
        "files": audio_files,
        "visuals": visual_files,
        "metrics": metrics,
        "proof": proof,
        "failure_codes": failure_codes,
    }
    return apply_evidence_boundary(
        report,
        evidence_role="diagnostic",
        source_backed=True,
        source_timing_backed=True,
        scripted_generation=True,
        notes=(
            "Dense-break render uses source-backed stems, source timing, and a "
            "bounded source-aware pressure_lift/stutter/restore and arrangement "
            "policy plus a source-layer-off rebuild diagnostic, but the 8-bar role grammar remains scripted; this is smoke/"
            "regression/diagnostic evidence, not product-quality proof."
        ),
    )


def performance_proof(
    source: np.ndarray,
    tr909: np.ndarray,
    w30: np.ndarray,
    mc202: np.ndarray,
    source_policy: DenseBreakSourcePolicy,
    performance: np.ndarray,
    sections: dict[str, np.ndarray],
    rebuild_only_performance: np.ndarray,
    rebuild_only_sections: dict[str, np.ndarray],
    bar_frames: int,
) -> dict:
    source_rms = rms(source)
    w30_rms = rms(w30)
    mc202_rms = rms(mc202)
    tr909_rms = rms(tr909)
    full_rms = rms(performance)
    rebuild_only_rms = rms(rebuild_only_performance)
    hook_rms = rms(sections["chop_hook"])
    pressure_rms = rms(sections["pressure_lift"])
    rebuild_only_hook_rms = rms(rebuild_only_sections["chop_hook"])
    rebuild_only_pressure_rms = rms(rebuild_only_sections["pressure_lift"])
    rebuild_only_restore_rms = rms(rebuild_only_sections["restore_hit"])
    pressure_bar4, pressure_bar5 = pressure_role_first_two(sections, bar_frames)
    restore_rms = rms(sections["restore_hit"])
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
    rebuild_only_source_similarity = waveform_correlation(source, rebuild_only_performance)
    source_on_rebuild_only_similarity = waveform_correlation(performance, rebuild_only_performance)
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
        "full_to_source_rms_ratio": full_rms / max(source_rms, 1e-9),
        "hook_to_source_transient_ratio": hook_transient / max(transient_score(source), 1e-9),
        "pressure_to_hook_rms_ratio": pressure_rms / max(hook_rms, 1e-9),
        "restore_to_pressure_rms_ratio": restore_rms / max(pressure_rms, 1e-9),
        "rebuild_only_to_full_rms_ratio": rebuild_only_rms / max(full_rms, 1e-9),
        "rebuild_only_to_source_rms_ratio": rebuild_only_rms / max(source_rms, 1e-9),
        "rebuild_only_to_source_correlation": rebuild_only_source_similarity,
        "source_on_to_rebuild_only_correlation": source_on_rebuild_only_similarity,
        "rebuild_only_pressure_to_hook_rms_ratio": rebuild_only_pressure_rms
        / max(rebuild_only_hook_rms, 1e-9),
        "rebuild_only_restore_to_pressure_rms_ratio": rebuild_only_restore_rms
        / max(rebuild_only_pressure_rms, 1e-9),
        "source_policy_decision_count": 8.0 if source_policy.source_aware else 0.0,
        "pressure_lift_policy_decision_count": (
            12.0 if source_policy.pressure_lift_policy.source_aware else 0.0
        ),
        "arrangement_policy_decision_count": (
            8.0 if source_policy.arrangement_policy.source_aware else 0.0
        ),
        "arrangement_role_order_hash": role_order_hash(source_policy.arrangement_policy),
        "arrangement_role_count": float(len(source_policy.arrangement_policy.role_order)),
        "arrangement_pressure_role_count": float(pressure_role_count(source_policy)),
        "arrangement_destructive_role_count": float(destructive_role_count(source_policy)),
        "arrangement_failure_count": float(
            len(arrangement_failure_codes(source_policy.arrangement_policy))
        ),
        "pressure_lift_bar5_to_bar4_rms_ratio": rms(pressure_bar5) / max(rms(pressure_bar4), 1e-9),
        "source_policy_pressure_gain": source_policy.pressure_gain,
        "source_policy_bass_gain": source_policy.bass_gain,
        "source_policy_stutter_step_divisor": float(source_policy.stutter_step_divisor),
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
    if proof["full_to_source_rms_ratio"] < MIN_FULL_TO_SOURCE_RMS_RATIO:
        failures.append("full_performance_not_assertive_enough_vs_source")
    if proof["hook_to_source_transient_ratio"] < MIN_HOOK_TO_SOURCE_TRANSIENT_RATIO:
        failures.append("hook_lacks_source_break_snap")
    if proof["pressure_to_hook_rms_ratio"] < MIN_PRESSURE_TO_HOOK_RMS_RATIO:
        failures.append("pressure_section_not_louder_than_hook_enough")
    if proof["restore_to_pressure_rms_ratio"] < MIN_RESTORE_TO_PRESSURE_RMS_RATIO:
        failures.append("restore_hit_not_bigger_than_pressure_section")
    if proof["rebuild_only_to_full_rms_ratio"] < MIN_REBUILD_ONLY_TO_FULL_RMS_RATIO:
        failures.append("rebuild_only_too_weak_relative_to_full_mix")
    if proof["rebuild_only_to_source_rms_ratio"] < MIN_REBUILD_ONLY_TO_SOURCE_RMS_RATIO:
        failures.append("rebuild_only_too_quiet_vs_source")
    if proof["rebuild_only_to_source_correlation"] > MAX_REBUILD_ONLY_TO_SOURCE_CORRELATION:
        failures.append("rebuild_only_too_source_masked")
    if proof["source_on_to_rebuild_only_correlation"] > MAX_SOURCE_ON_TO_REBUILD_ONLY_CORRELATION:
        failures.append("source_layer_toggle_did_not_change_output")
    if proof["rebuild_only_pressure_to_hook_rms_ratio"] < MIN_PRESSURE_TO_HOOK_RMS_RATIO:
        failures.append("rebuild_only_pressure_not_louder_than_hook_enough")
    if (
        proof["rebuild_only_restore_to_pressure_rms_ratio"]
        < MIN_REBUILD_ONLY_RESTORE_TO_PRESSURE_RMS_RATIO
    ):
        failures.append("rebuild_only_restore_not_bigger_than_pressure")
    if proof["arrangement_policy_decision_count"] < 8.0:
        failures.append("arrangement_policy_not_source_aware_enough")
    if proof["arrangement_role_count"] != float(DEFAULT_BARS):
        failures.append("arrangement_role_order_not_8_bars")
    if proof["arrangement_pressure_role_count"] < 2.0:
        failures.append("arrangement_pressure_lift_too_short")
    if proof["arrangement_destructive_role_count"] < 2.0:
        failures.append("arrangement_destructive_restore_tail_missing")
    if proof["arrangement_failure_count"] > 0.0:
        failures.append("arrangement_policy_contract_failed")
    return failures


def write_visual_evidence(
    output: Path,
    named_samples: dict[str, np.ndarray],
) -> dict[str, dict[str, str]]:
    visual_dir = output / "visuals"
    visual_dir.mkdir(parents=True, exist_ok=True)
    result = {}
    for role, samples in named_samples.items():
        waveform = visual_dir / f"{role}.waveform.png"
        spectrogram = visual_dir / f"{role}.spectrogram.png"
        write_waveform_png(waveform, samples)
        write_spectrogram_png(spectrogram, samples)
        result[role] = {
            "waveform": str(waveform.relative_to(output)),
            "spectrogram": str(spectrogram.relative_to(output)),
        }
    return result


def write_waveform_png(path: Path, samples: np.ndarray, width: int = 960, height: int = 220) -> None:
    image = np.zeros((height, width, 3), dtype=np.uint8)
    image[:, :, :] = np.array([15, 17, 23], dtype=np.uint8)
    mid = height // 2
    image[mid : mid + 1, :, :] = np.array([72, 76, 90], dtype=np.uint8)
    mono = samples.mean(axis=1) if samples.size else np.zeros((0,), dtype=np.float32)
    if mono.size:
        chunks = np.array_split(mono, width)
        for x, chunk in enumerate(chunks):
            if chunk.size == 0:
                continue
            low = float(np.min(chunk))
            high = float(np.max(chunk))
            y1 = int(round((1.0 - (high + 1.0) * 0.5) * (height - 1)))
            y2 = int(round((1.0 - (low + 1.0) * 0.5) * (height - 1)))
            y1 = max(0, min(height - 1, y1))
            y2 = max(0, min(height - 1, y2))
            if y2 < y1:
                y1, y2 = y2, y1
            image[y1 : y2 + 1, x, :] = np.array([106, 220, 181], dtype=np.uint8)
    write_rgb_png(path, image)


def write_spectrogram_png(
    path: Path,
    samples: np.ndarray,
    width: int = 960,
    height: int = 280,
) -> None:
    mono = samples.mean(axis=1) if samples.size else np.zeros((0,), dtype=np.float32)
    if mono.size < 1024:
        write_rgb_png(path, np.zeros((height, width, 3), dtype=np.uint8))
        return

    n_fft = 1024
    hop = max(1, (mono.size - n_fft) // width)
    columns = []
    window = np.hanning(n_fft).astype(np.float32)
    for start in range(0, mono.size - n_fft + 1, hop):
        frame = mono[start : start + n_fft] * window
        spectrum = np.abs(np.fft.rfft(frame))
        columns.append(np.log1p(spectrum))
        if len(columns) >= width:
            break
    if not columns:
        write_rgb_png(path, np.zeros((height, width, 3), dtype=np.uint8))
        return

    spec = np.stack(columns, axis=1)
    spec = spec[: spec.shape[0] // 2, :]
    spec -= float(np.min(spec))
    max_value = float(np.max(spec))
    if max_value > 1e-9:
        spec /= max_value
    y_indices = np.linspace(spec.shape[0] - 1, 0, height).astype(np.int32)
    x_indices = np.linspace(0, spec.shape[1] - 1, width).astype(np.int32)
    values = spec[y_indices[:, None], x_indices[None, :]]
    image = heatmap(values)
    write_rgb_png(path, image)


def heatmap(values: np.ndarray) -> np.ndarray:
    clipped = np.clip(values, 0.0, 1.0)
    red = np.clip((clipped - 0.35) / 0.65, 0.0, 1.0)
    green = np.clip(1.0 - np.abs(clipped - 0.55) / 0.55, 0.0, 1.0)
    blue = np.clip(1.0 - clipped * 1.45, 0.0, 1.0)
    image = np.stack([red, green, blue], axis=2)
    return (image * 255.0).astype(np.uint8)


def write_rgb_png(path: Path, image: np.ndarray) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    height, width, channels = image.shape
    if channels != 3:
        raise ValueError("PNG image must be RGB")
    raw = b"".join(b"\x00" + image[row].tobytes() for row in range(height))
    payload = (
        png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
        + png_chunk(b"IDAT", zlib.compress(raw, level=9))
        + png_chunk(b"IEND", b"")
    )
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + payload)


def png_chunk(kind: bytes, data: bytes) -> bytes:
    checksum = zlib.crc32(kind + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", checksum)


def write_reports(output: Path, report: dict) -> None:
    (output / "performance-report.json").write_text(json.dumps(report, indent=2) + "\n")
    agent_review = agent_review_record(report)
    (output / "agent-review.json").write_text(json.dumps(agent_review, indent=2) + "\n")
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
    lines.extend(["", "## Visual Evidence", ""])
    for role, paths in report["visuals"].items():
        lines.append(
            f"- `{role}`: waveform `{paths['waveform']}`, spectrogram `{paths['spectrogram']}`"
        )
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
    write_agent_review_markdown(output / "agent-review.md", agent_review)


def agent_review_record(report: dict) -> dict:
    passed = report["result"] == "pass"
    proof = report["proof"]
    if passed:
        strongest = "source_break_hook_pressure_stutter_restore"
        summary = (
            "Promising: the dense break is transformed into a louder sectional "
            "performance with measurable break-hook transient, pressure lift, "
            "dropout/stutter impact, and a bigger restore hit."
        )
    elif len(report["failure_codes"]) <= 2:
        strongest = "partial_audio_evidence"
        summary = "Weak: the pack renders, but one or two musical guardrails still fail."
    else:
        strongest = "none"
        summary = "Fail: the pack trips multiple weak-output guardrails."
    review = {
        "schema": AGENT_REVIEW_SCHEMA,
        "schema_version": 1,
        "source_report_schema": report["schema"],
        "result": report["result"],
        "agent_verdict": report["agent_verdict"],
        "human_verdict": report["human_verdict"],
        "strongest_element": strongest,
        "source_recognition": source_recognition_label(proof["source_to_performance_correlation"]),
        "hook_after_two_bars": "present" if proof["w30_to_source_rms_ratio"] >= 0.18 else "weak",
        "summary": summary,
        "failure_codes": report["failure_codes"],
        "audio_files": report["files"],
        "visual_files": report["visuals"],
        "review_focus": [
            "Does the W-30 chop read as a hook after two bars?",
            "Does the pressure lift hit harder than the opening hook?",
            "Does the dropout/stutter feel like a playable destructive gesture?",
            "Does the restore hit land as a break transient plus bass-pressure moment?",
        ],
        "proof": {
            "w30_to_source_rms_ratio": proof["w30_to_source_rms_ratio"],
            "pressure_low_band_lift_ratio": proof["pressure_low_band_lift_ratio"],
            "dropout_to_stutter_rms_ratio": proof["dropout_to_stutter_rms_ratio"],
            "restore_to_hook_transient_ratio": proof["restore_to_hook_transient_ratio"],
            "max_adjacent_bar_correlation": proof["max_adjacent_bar_correlation"],
            "source_to_performance_correlation": proof["source_to_performance_correlation"],
            "full_to_source_rms_ratio": proof["full_to_source_rms_ratio"],
            "hook_to_source_transient_ratio": proof["hook_to_source_transient_ratio"],
            "pressure_to_hook_rms_ratio": proof["pressure_to_hook_rms_ratio"],
            "restore_to_pressure_rms_ratio": proof["restore_to_pressure_rms_ratio"],
        },
        "boundary": (
            "Agent review may block fail/weak outputs and mark this pack promising. "
            "It must not claim final human musical pass while human_verdict is unverified."
        ),
    }
    return apply_evidence_boundary(
        review,
        evidence_role="diagnostic",
        source_backed=bool(report.get("source_backed")),
        source_timing_backed=bool(report.get("source_timing_backed")),
        scripted_generation=bool(report.get("scripted_generation")),
        notes=(
            "Agent review summarizes diagnostic render evidence. It is not a "
            "human musical pass or product-quality proof."
        ),
    )


def source_recognition_label(correlation: float) -> str:
    if correlation >= 0.90:
        return "source_too_close_to_original"
    if correlation >= 0.20:
        return "source_transformed_but_present"
    return "source_character_at_risk"


def write_agent_review_markdown(path: Path, review: dict) -> None:
    lines = [
        "# Agent Musical Review",
        "",
        f"- Result: `{review['result']}`",
        f"- Agent verdict: `{review['agent_verdict']}`",
        f"- Human verdict: `{review['human_verdict']}`",
        f"- Strongest element: `{review['strongest_element']}`",
        f"- Source recognition: `{review['source_recognition']}`",
        f"- Hook after two bars: `{review['hook_after_two_bars']}`",
        "",
        "## Summary",
        "",
        review["summary"],
        "",
        "## Review Focus",
        "",
    ]
    lines.extend(f"- {item}" for item in review["review_focus"])
    lines.extend(["", "## Visual Files", ""])
    for role, paths in review["visual_files"].items():
        lines.append(
            f"- `{role}`: waveform `{paths['waveform']}`, spectrogram `{paths['spectrogram']}`"
        )
    lines.extend(["", "## Boundary", "", review["boundary"]])
    path.write_text("\n".join(lines) + "\n")


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


def glue_bus(samples: np.ndarray, drive: float, slam: float) -> np.ndarray:
    dry = np.clip(samples, -0.98, 0.98).astype(np.float32)
    crushed = saturate(dry, drive)
    mixed = dry * (1.0 - slam) + crushed * slam
    return saturate(mixed, 1.04)


def normalize_peak(samples: np.ndarray, target_peak: float) -> np.ndarray:
    if samples.size == 0:
        return samples.astype(np.float32)
    peak = float(np.max(np.abs(samples)))
    if peak <= 1e-9:
        return samples.astype(np.float32)
    gain = min(target_peak / peak, 2.4)
    return np.clip(samples * gain, -target_peak, target_peak).astype(np.float32)


def apply_gain(samples: np.ndarray, gain: float) -> np.ndarray:
    return np.clip(samples * gain, -0.98, 0.98).astype(np.float32)


def source_relative_gain(
    source: np.ndarray,
    stem: np.ndarray,
    target_ratio: float,
    minimum_gain: float,
    maximum_gain: float,
) -> float:
    stem_rms = rms(stem)
    if stem_rms <= 1e-9:
        return minimum_gain
    target = rms(source) * target_ratio
    gain = target / stem_rms
    return float(np.clip(gain, minimum_gain, maximum_gain))


def transient_emphasis(samples: np.ndarray) -> np.ndarray:
    if samples.shape[0] < 2:
        return np.zeros_like(samples)
    previous = np.vstack([samples[0:1], samples[:-1]])
    return np.clip((samples - previous) * 2.0, -0.98, 0.98).astype(np.float32)


def strongest_window_start(samples: np.ndarray, window_frames: int) -> int:
    if samples.shape[0] <= window_frames:
        return 0
    mono = np.abs(samples.mean(axis=1))
    window = np.ones(max(1, window_frames), dtype=np.float32)
    energy = np.convolve(mono, window, mode="valid")
    return int(np.argmax(energy))


def decay_envelope(size: int, attack: float, decay: float) -> np.ndarray:
    if size <= 0:
        return np.zeros((0,), dtype=np.float32)
    t = np.arange(size, dtype=np.float32) / SAMPLE_RATE
    attack_env = np.clip(t / max(attack, 1.0 / SAMPLE_RATE), 0.0, 1.0)
    decay_env = np.exp(-t / max(decay, 1.0 / SAMPLE_RATE))
    return (attack_env * decay_env).astype(np.float32)


def impact_envelope(size: int, decay: float) -> np.ndarray:
    if size <= 0:
        return np.zeros((0,), dtype=np.float32)
    t = np.arange(size, dtype=np.float32) / SAMPLE_RATE
    return np.exp(-t / max(decay, 1.0 / SAMPLE_RATE)).astype(np.float32)


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

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

from audio_qa_evidence_boundary import apply_evidence_boundary, evidence_boundary_failure_codes


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
MIN_BAD_TIMING_CUE_TRANSIENT_SCORE = 0.030
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
MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ = 0.25
MIN_SPARSE_BASS_MOVEMENT_SPAN_HZ = 2.00
MIN_HOOK_CHOP_SELECTION_CANDIDATES = 3
MIN_HOOK_CHOP_STATIC_DISTANCE_FRAMES = 256.0
MIN_HOOK_CHOP_OFFSET_DISTANCE_FRAMES = 512.0
MIN_DESTRUCTIVE_GESTURE_CANDIDATES = 3
MIN_DESTRUCTIVE_STATIC_DISTANCE_FRAMES = 256.0
MIN_DESTRUCTIVE_OFFSET_DISTANCE_FRAMES = 512.0
MIN_ARRANGEMENT_ROLE_CANDIDATES = 6
MIN_ARRANGEMENT_SCRIPTED_ROLE_DISTANCE = 1.0
MIN_MIX_TREATMENT_CANDIDATES = 6
MIN_MIX_TREATMENT_FIXED_DISTANCE = 0.08
MIN_MIX_TREATMENT_OUTPUT_CONTRAST = 2.10
MIN_PAD_NOISE_TEXTURE_CANDIDATES = 3
MIN_PAD_NOISE_TEXTURE_STATIC_DISTANCE_FRAMES = 256.0
MIN_PAD_NOISE_TEXTURE_OFFSET_DISTANCE_FRAMES = 512.0
MIN_PAD_NOISE_TEXTURE_TRANSIENT_RATIO = 0.72
MIN_TAIL_SHAPE_CANDIDATES = 6
MIN_TAIL_SHAPE_FIXED_DISTANCE = 0.20
MIN_TAIL_SHAPE_OUTPUT_CONTRAST = 3.00
TARGET_PERFORMANCE_PEAK = 0.92
MAX_PAD_NOISE_LOW_BAND_RMS = 0.030
MIN_PAD_NOISE_HIGH_BAND_RATIO = 0.180
MIN_PAD_NOISE_TRANSIENT_SCORE = 0.050

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
    role_order_source_derived: bool
    source_family: str
    selection_strategy: str
    role_order: tuple[str, ...]
    role_order_signature: str
    scripted_role_order: tuple[str, ...]
    scripted_role_order_signature: str
    scripted_role_distance: int
    candidate_count: int
    section_score_span: float
    arrangement_shape: str
    arrangement_intent: str


@dataclass(frozen=True)
class HookChopPolicy:
    source_aware: bool
    source_family: str
    selection_strategy: str
    hook_start_frames: int
    chop_start_frames: int
    static_first_bar_start_frames: int
    hook_start_seconds: float
    chop_start_seconds: float
    hook_static_distance_frames: int
    chop_static_distance_frames: int
    hook_chop_distance_frames: int
    candidate_count: int


@dataclass(frozen=True)
class DestructiveGesturePolicy:
    source_aware: bool
    source_family: str
    selection_strategy: str
    stutter_start_frames: int
    restore_start_frames: int
    fixed_stutter_start_frames: int
    fixed_restore_start_frames: int
    stutter_start_seconds: float
    restore_start_seconds: float
    stutter_static_distance_frames: int
    restore_static_distance_frames: int
    stutter_restore_distance_frames: int
    candidate_count: int


@dataclass(frozen=True)
class MixTreatmentPolicy:
    source_aware: bool
    source_family: str
    selection_strategy: str
    hook_drive: float
    hook_slam: float
    hook_w30_gain: float
    hook_break_snap_gain: float
    chop_drive: float
    chop_slam: float
    chop_w30_gain: float
    chop_break_snap_gain: float
    pressure_drive: float
    pressure_slam: float
    pressure_peak: float
    pressure_w30_gain: float
    pressure_break_snap_gain: float
    restore_drive: float
    restore_slam: float
    restore_bass_gain: float
    restore_break_snap_gain: float
    final_drive: float
    final_slam: float
    fixed_treatment_distance: float
    source_energy_span: float
    candidate_count: int


@dataclass(frozen=True)
class PadNoiseTexturePolicy:
    source_aware: bool
    source_family: str
    selection_strategy: str
    gate_start_frames: int
    stab_start_frames: int
    fixed_gate_start_frames: int
    fixed_stab_start_frames: int
    gate_start_seconds: float
    stab_start_seconds: float
    gate_static_distance_frames: int
    stab_static_distance_frames: int
    gate_stab_distance_frames: int
    gate_duty: float
    texture_gain: float
    stab_gain: float
    candidate_count: int


@dataclass(frozen=True)
class TailShapePolicy:
    source_aware: bool
    source_family: str
    selection_strategy: str
    dropout_silence_fraction: float
    dropout_silence_gain: float
    stutter_step_divisor: int
    stutter_grain_gain: float
    stutter_snap_gain: float
    restore_source_gain: float
    restore_snap_gain: float
    restore_w30_gain: float
    restore_mc202_gain: float
    restore_tr909_gain: float
    restore_drive: float
    restore_slam: float
    fixed_tail_distance: float
    source_energy_span: float
    candidate_count: int


@dataclass(frozen=True)
class DenseBreakSourcePolicy:
    source_aware: bool
    pressure_shape: str
    stutter_density: str
    restore_hit_shape: str
    pressure_lift_policy: PressureLiftPolicy
    arrangement_policy: ArrangementPolicy
    hook_chop_policy: HookChopPolicy
    destructive_gesture_policy: DestructiveGesturePolicy
    mix_treatment_policy: MixTreatmentPolicy
    pad_noise_texture_policy: PadNoiseTexturePolicy
    tail_shape_policy: TailShapePolicy
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
    parser.add_argument("--validate-report", type=Path)
    parser.add_argument("--timing-confidence-result")
    parser.add_argument("--timing-grid-use")
    args = parser.parse_args()

    repo = repo_root()
    if args.validate_report:
        report_path = resolve_repo_path(repo, args.validate_report)
        validate_report_file(report_path)
        print(f"valid dense-break performance report: {report_path}")
        return 0

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
    source_policy = dense_break_source_policy(
        source_audio,
        w30,
        bar_frames,
        timing_confidence_result=args.timing_confidence_result,
        timing_grid_use=args.timing_grid_use,
    )
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


def hook_chop_policy_for(
    source: np.ndarray,
    w30: np.ndarray,
    bar_frames: int,
    source_family: str,
) -> HookChopPolicy:
    grain_len = min(frames_for_seconds(0.090), max(1, bar_frames // 8))
    first_bar = w30[: min(bar_frames, w30.shape[0])]
    static_start = strongest_window_start(first_bar, grain_len)
    scan_end = min(max(2 * bar_frames, grain_len), 4 * bar_frames, source.shape[0], w30.shape[0])
    if scan_end <= grain_len:
        return HookChopPolicy(
            source_aware=False,
            source_family=source_family,
            selection_strategy="fallback-static-first-bar",
            hook_start_frames=static_start,
            chop_start_frames=static_start,
            static_first_bar_start_frames=static_start,
            hook_start_seconds=static_start / SAMPLE_RATE,
            chop_start_seconds=static_start / SAMPLE_RATE,
            hook_static_distance_frames=0,
            chop_static_distance_frames=0,
            hook_chop_distance_frames=0,
            candidate_count=1,
        )

    stride = max(1, grain_len // 2)
    candidates = []
    for start in range(0, scan_end - grain_len + 1, stride):
        end = start + grain_len
        source_chunk = source[start:end]
        w30_chunk = w30[start:end]
        source_rms = rms(source_chunk)
        w30_rms = rms(w30_chunk)
        transient = transient_score(source_chunk)
        low = low_band_rms(source_chunk)
        high = high_band_ratio(source_chunk)
        if source_family == "tonal_hook":
            hook_score = source_rms * 1.25 + w30_rms * 1.05 + low * 0.90 + high * 0.020
            chop_score = transient * 16.0 + w30_rms * 1.20 + source_rms * 0.55
            strategy = "tonal-sustain-hook-transient-chop"
        else:
            hook_score = transient * 20.0 + w30_rms * 1.15 + high * 0.025
            chop_score = transient * 17.0 + w30_rms * 1.35 + source_rms * 0.45
            strategy = "transient-break-hook-energy-chop"
        candidates.append(
            {
                "start": start,
                "hook_score": float(hook_score),
                "chop_score": float(chop_score),
            }
        )

    min_separation = max(grain_len * 2, int(MIN_HOOK_CHOP_OFFSET_DISTANCE_FRAMES))

    def select(score_key: str, avoid_start: int | None = None) -> int:
        ranked = sorted(candidates, key=lambda item: item[score_key], reverse=True)
        if not ranked:
            return static_start
        best_score = max(ranked[0][score_key], 1e-9)
        for item in ranked:
            start = int(item["start"])
            if avoid_start is not None and abs(start - avoid_start) < min_separation:
                continue
            if abs(start - static_start) < MIN_HOOK_CHOP_STATIC_DISTANCE_FRAMES:
                if item[score_key] < best_score * 0.65:
                    continue
            return start
        return int(ranked[0]["start"])

    hook_start = select("hook_score")
    chop_start = select("chop_score", avoid_start=hook_start)
    return HookChopPolicy(
        source_aware=True,
        source_family=source_family,
        selection_strategy=strategy,
        hook_start_frames=hook_start,
        chop_start_frames=chop_start,
        static_first_bar_start_frames=static_start,
        hook_start_seconds=hook_start / SAMPLE_RATE,
        chop_start_seconds=chop_start / SAMPLE_RATE,
        hook_static_distance_frames=abs(hook_start - static_start),
        chop_static_distance_frames=abs(chop_start - static_start),
        hook_chop_distance_frames=abs(chop_start - hook_start),
        candidate_count=len(candidates),
    )


def destructive_gesture_policy_for(
    source: np.ndarray,
    w30: np.ndarray,
    bar_frames: int,
    source_family: str,
    stutter_grain_beat_offset: float,
) -> DestructiveGesturePolicy:
    beat_frames = max(1, bar_frames // BEATS_PER_BAR)
    fixed_stutter_start = int(round(stutter_grain_beat_offset * beat_frames))
    fixed_restore_start = 0
    grain_len = max(128, bar_frames // 32)
    restore_len = min(frames_for_seconds(0.115), max(1, bar_frames // 4))
    scan_len = max(grain_len, restore_len)
    source_eligible = source_family in ("dense_break", "tonal_hook", "sparse_bass_pressure")
    scan_end = min(max(2 * bar_frames, scan_len), 4 * bar_frames, source.shape[0], w30.shape[0])
    if not source_eligible or scan_end <= scan_len:
        return DestructiveGesturePolicy(
            source_aware=False,
            source_family=source_family,
            selection_strategy="fallback-fixed-destructive-gesture",
            stutter_start_frames=fixed_stutter_start,
            restore_start_frames=fixed_restore_start,
            fixed_stutter_start_frames=fixed_stutter_start,
            fixed_restore_start_frames=fixed_restore_start,
            stutter_start_seconds=fixed_stutter_start / SAMPLE_RATE,
            restore_start_seconds=fixed_restore_start / SAMPLE_RATE,
            stutter_static_distance_frames=0,
            restore_static_distance_frames=0,
            stutter_restore_distance_frames=0,
            candidate_count=1,
        )

    stride = max(1, min(grain_len, restore_len) // 2)
    candidates = []
    for start in range(0, scan_end - scan_len + 1, stride):
        stutter_chunk = source[start : start + grain_len]
        restore_chunk = source[start : start + restore_len]
        w30_chunk = w30[start : start + grain_len]
        stutter_transient = transient_score(stutter_chunk)
        restore_transient = transient_score(restore_chunk)
        source_rms = rms(restore_chunk)
        w30_rms = rms(w30_chunk)
        low = low_band_rms(restore_chunk)
        high = high_band_ratio(stutter_chunk)
        if source_family == "tonal_hook":
            stutter_score = stutter_transient * 13.0 + w30_rms * 1.20 + high * 0.018
            restore_score = restore_transient * 12.0 + source_rms * 1.05 + low * 0.95
            strategy = "tonal-transient-stutter-sustain-restore"
        elif source_family == "sparse_bass_pressure":
            stutter_score = stutter_transient * 15.0 + w30_rms * 1.05 + low * 0.65
            restore_score = restore_transient * 14.0 + low * 1.35 + source_rms * 0.85
            strategy = "sparse-pressure-stutter-lowband-restore"
        else:
            stutter_score = stutter_transient * 20.0 + w30_rms * 1.25 + high * 0.030
            restore_score = restore_transient * 21.0 + low * 1.05 + source_rms * 0.75
            strategy = "dense-transient-stutter-pressure-restore"
        candidates.append(
            {
                "start": start,
                "stutter_score": float(stutter_score),
                "restore_score": float(restore_score),
            }
        )

    min_separation = max(grain_len * 2, int(MIN_DESTRUCTIVE_OFFSET_DISTANCE_FRAMES))

    def select(score_key: str, fixed_start: int, avoid_start: int | None = None) -> int:
        ranked = sorted(candidates, key=lambda item: item[score_key], reverse=True)
        if not ranked:
            return fixed_start
        best_score = max(ranked[0][score_key], 1e-9)
        for item in ranked:
            start = int(item["start"])
            if avoid_start is not None and abs(start - avoid_start) < min_separation:
                continue
            if abs(start - fixed_start) < MIN_DESTRUCTIVE_STATIC_DISTANCE_FRAMES:
                if item[score_key] < best_score * 0.65:
                    continue
            return start
        return int(ranked[0]["start"])

    stutter_start = select("stutter_score", fixed_stutter_start)
    restore_start = select("restore_score", fixed_restore_start, avoid_start=stutter_start)
    return DestructiveGesturePolicy(
        source_aware=True,
        source_family=source_family,
        selection_strategy=strategy,
        stutter_start_frames=stutter_start,
        restore_start_frames=restore_start,
        fixed_stutter_start_frames=fixed_stutter_start,
        fixed_restore_start_frames=fixed_restore_start,
        stutter_start_seconds=stutter_start / SAMPLE_RATE,
        restore_start_seconds=restore_start / SAMPLE_RATE,
        stutter_static_distance_frames=abs(stutter_start - fixed_stutter_start),
        restore_static_distance_frames=abs(restore_start - fixed_restore_start),
        stutter_restore_distance_frames=abs(restore_start - stutter_start),
        candidate_count=len(candidates),
    )


def fixed_restore_bass_gain(source_family: str) -> float:
    return 2.38 if source_family == "sparse_bass_pressure" else 1.86


def fixed_mix_treatment_policy(
    source_family: str,
    candidate_count: int = 1,
) -> MixTreatmentPolicy:
    return MixTreatmentPolicy(
        source_aware=False,
        source_family=source_family,
        selection_strategy="fallback-fixed-mix-treatment",
        hook_drive=1.04,
        hook_slam=0.05,
        hook_w30_gain=1.38,
        hook_break_snap_gain=1.50,
        chop_drive=1.24,
        chop_slam=0.18,
        chop_w30_gain=1.54,
        chop_break_snap_gain=1.36,
        pressure_drive=1.34,
        pressure_slam=0.30,
        pressure_peak=0.79,
        pressure_w30_gain=0.84,
        pressure_break_snap_gain=1.36,
        restore_drive=1.72,
        restore_slam=0.40,
        restore_bass_gain=fixed_restore_bass_gain(source_family),
        restore_break_snap_gain=3.64,
        final_drive=1.22,
        final_slam=0.22,
        fixed_treatment_distance=0.0,
        source_energy_span=0.0,
        candidate_count=candidate_count,
    )


def mix_treatment_policy_for(
    source: np.ndarray,
    w30: np.ndarray,
    bar_frames: int,
    source_family: str,
) -> MixTreatmentPolicy:
    eligible = source_family in ("dense_break", "tonal_hook", "sparse_bass_pressure")
    candidate_count = min(6, max(0, source.shape[0] // max(1, bar_frames)))
    fixed = fixed_mix_treatment_policy(source_family, max(candidate_count, 1))
    if not eligible or candidate_count < MIN_MIX_TREATMENT_CANDIDATES:
        return fixed

    candidates = []
    for bar in range(candidate_count):
        start = bar * bar_frames
        end = min(start + bar_frames, source.shape[0], w30.shape[0])
        chunk = source[start:end]
        w30_chunk = w30[start:end]
        if chunk.shape[0] == 0:
            continue
        candidates.append(
            {
                "source_rms": rms(chunk),
                "w30_rms": rms(w30_chunk),
                "low": low_band_rms(chunk),
                "high": high_band_ratio(chunk),
                "transient": transient_score(chunk),
            }
        )
    if len(candidates) < MIN_MIX_TREATMENT_CANDIDATES:
        return fixed_mix_treatment_policy(source_family, len(candidates))

    source_rms_values = [item["source_rms"] for item in candidates]
    low_values = [item["low"] for item in candidates]
    high_values = [item["high"] for item in candidates]
    transient_values = [item["transient"] for item in candidates]
    w30_values = [item["w30_rms"] for item in candidates]
    source_mean = max(float(np.mean(source_rms_values)), 1e-9)
    low_norm = float(np.clip(np.mean(low_values) / source_mean, 0.0, 1.25))
    high_norm = float(np.clip(np.mean(high_values) / 0.35, 0.0, 1.25))
    transient_norm = float(
        np.clip(np.mean(transient_values) / max(source_mean * 7.0, 1e-9), 0.0, 1.25)
    )
    w30_norm = float(np.clip(np.mean(w30_values) / source_mean, 0.0, 1.25))
    energy_span = float(max(source_rms_values) - min(source_rms_values))
    energy_span_norm = float(np.clip(energy_span / source_mean, 0.0, 1.25))

    if source_family == "tonal_hook":
        strategy = "tonal-hook-readable-mix-treatment"
        hook_bias = 0.08
        chop_bias = 0.02
        pressure_bias = -0.02
        restore_bias = 0.03
    elif source_family == "sparse_bass_pressure":
        strategy = "sparse-bass-pressure-mix-treatment"
        hook_bias = -0.02
        chop_bias = 0.02
        pressure_bias = 0.10
        restore_bias = 0.10
    else:
        strategy = "dense-break-snap-mix-treatment"
        hook_bias = 0.03
        chop_bias = 0.08
        pressure_bias = 0.05
        restore_bias = 0.07

    policy_values = {
        "hook_drive": float(
            np.clip(
                fixed.hook_drive + hook_bias + transient_norm * 0.035 - low_norm * 0.012,
                1.00,
                1.16,
            )
        ),
        "hook_slam": float(
            np.clip(
                fixed.hook_slam + high_norm * 0.020 + transient_norm * 0.010,
                0.04,
                0.10,
            )
        ),
        "hook_w30_gain": float(
            np.clip(fixed.hook_w30_gain + w30_norm * 0.090 + hook_bias, 1.28, 1.58)
        ),
        "hook_break_snap_gain": float(
            np.clip(
                fixed.hook_break_snap_gain + transient_norm * 0.140 + hook_bias,
                1.42,
                1.78,
            )
        ),
        "chop_drive": float(
            np.clip(
                fixed.chop_drive + chop_bias + high_norm * 0.050 + transient_norm * 0.030,
                1.18,
                1.40,
            )
        ),
        "chop_slam": float(
            np.clip(
                fixed.chop_slam + high_norm * 0.035 + energy_span_norm * 0.015,
                0.16,
                0.27,
            )
        ),
        "chop_w30_gain": float(
            np.clip(fixed.chop_w30_gain + w30_norm * 0.110 + chop_bias, 1.46, 1.78)
        ),
        "chop_break_snap_gain": float(
            np.clip(
                fixed.chop_break_snap_gain + transient_norm * 0.120 + chop_bias,
                1.30,
                1.58,
            )
        ),
        "pressure_drive": float(
            np.clip(
                fixed.pressure_drive
                + pressure_bias
                + low_norm * 0.045
                + transient_norm * 0.015,
                1.28,
                1.50,
            )
        ),
        "pressure_slam": float(
            np.clip(
                fixed.pressure_slam + pressure_bias * 0.25 + low_norm * 0.030,
                0.26,
                0.40,
            )
        ),
        "pressure_peak": float(
            np.clip(
                fixed.pressure_peak + pressure_bias * 0.10 + low_norm * 0.018,
                0.77,
                0.83,
            )
        ),
        "pressure_w30_gain": float(
            np.clip(
                fixed.pressure_w30_gain + w30_norm * 0.050 - low_norm * 0.015,
                0.78,
                0.94,
            )
        ),
        "pressure_break_snap_gain": float(
            np.clip(
                fixed.pressure_break_snap_gain + transient_norm * 0.130 + pressure_bias,
                1.26,
                1.62,
            )
        ),
        "restore_drive": float(
            np.clip(
                fixed.restore_drive + restore_bias + transient_norm * 0.050 + low_norm * 0.025,
                1.66,
                1.90,
            )
        ),
        "restore_slam": float(
            np.clip(
                fixed.restore_slam
                + restore_bias * 0.20
                + transient_norm * 0.020
                + low_norm * 0.020,
                0.38,
                0.50,
            )
        ),
        "restore_bass_gain": float(
            np.clip(
                fixed.restore_bass_gain + restore_bias + low_norm * 0.180,
                1.82,
                2.72,
            )
        ),
        "restore_break_snap_gain": float(
            np.clip(
                fixed.restore_break_snap_gain + transient_norm * 0.220 + restore_bias,
                3.52,
                4.02,
            )
        ),
        "final_drive": float(
            np.clip(
                fixed.final_drive + energy_span_norm * 0.025 + transient_norm * 0.010,
                1.20,
                1.28,
            )
        ),
        "final_slam": float(
            np.clip(fixed.final_slam + energy_span_norm * 0.020, 0.21, 0.28)
        ),
    }
    distance = float(
        sum(
            abs(policy_values[name] - getattr(fixed, name))
            for name in policy_values
        )
    )
    return MixTreatmentPolicy(
        source_aware=True,
        source_family=source_family,
        selection_strategy=strategy,
        fixed_treatment_distance=distance,
        source_energy_span=energy_span,
        candidate_count=len(candidates),
        **policy_values,
    )


def fixed_pad_noise_texture_policy(source_family: str, candidate_count: int = 0) -> PadNoiseTexturePolicy:
    return PadNoiseTexturePolicy(
        source_aware=False,
        source_family=source_family,
        selection_strategy="fallback-no-pad-noise-texture",
        gate_start_frames=0,
        stab_start_frames=0,
        fixed_gate_start_frames=0,
        fixed_stab_start_frames=0,
        gate_start_seconds=0.0,
        stab_start_seconds=0.0,
        gate_static_distance_frames=0,
        stab_static_distance_frames=0,
        gate_stab_distance_frames=0,
        gate_duty=0.0,
        texture_gain=0.0,
        stab_gain=0.0,
        candidate_count=candidate_count,
    )


def pad_noise_texture_policy_for(
    source: np.ndarray,
    w30: np.ndarray,
    bar_frames: int,
    source_family: str,
) -> PadNoiseTexturePolicy:
    if source_family != "pad_noise":
        return fixed_pad_noise_texture_policy(source_family)
    window = min(frames_for_seconds(0.120), max(1, bar_frames // 6))
    step = max(1, bar_frames // 8)
    scan_end = min(source.shape[0], w30.shape[0], 6 * bar_frames)
    fixed_gate_start = 0
    fixed_stab_start = min(max(0, bar_frames // 2), max(0, scan_end - window))
    candidates = []
    for start in range(0, max(0, scan_end - window), step):
        end = min(start + window, source.shape[0], w30.shape[0])
        if end <= start:
            continue
        source_chunk = source[start:end]
        w30_chunk = w30[start:end]
        source_rms = rms(source_chunk)
        high = high_band_ratio(source_chunk)
        transient = transient_score(source_chunk)
        w30_rms = rms(w30_chunk)
        candidates.append(
            {
                "start": start,
                "gate_score": high * 1.40 + source_rms * 0.55 + w30_rms * 0.22,
                "stab_score": transient * 1.25 + high * 0.35 + w30_rms * 0.45,
                "high": high,
                "transient": transient,
                "source_rms": source_rms,
            }
        )
    if len(candidates) < MIN_PAD_NOISE_TEXTURE_CANDIDATES:
        return fixed_pad_noise_texture_policy(source_family, len(candidates))

    gate = max(candidates, key=lambda item: (item["gate_score"], item["start"]))
    sorted_stabs = sorted(
        candidates,
        key=lambda item: (item["stab_score"], abs(int(item["start"]) - int(gate["start"]))),
        reverse=True,
    )
    stab = sorted_stabs[0]
    for candidate in sorted_stabs:
        if abs(int(candidate["start"]) - int(gate["start"])) >= MIN_PAD_NOISE_TEXTURE_OFFSET_DISTANCE_FRAMES:
            stab = candidate
            break

    high_values = [float(item["high"]) for item in candidates]
    transient_values = [float(item["transient"]) for item in candidates]
    high_mean = float(np.mean(high_values)) if high_values else 0.0
    transient_mean = float(np.mean(transient_values)) if transient_values else 0.0
    gate_duty = float(np.clip(0.30 + high_mean * 0.80, 0.34, 0.62))
    texture_gain = float(np.clip(0.70 + high_mean * 1.80, 0.82, 1.32))
    stab_gain = float(np.clip(1.05 + transient_mean * 3.20, 1.12, 1.82))
    gate_start = int(gate["start"])
    stab_start = int(stab["start"])
    return PadNoiseTexturePolicy(
        source_aware=True,
        source_family=source_family,
        selection_strategy="source-derived-gated-texture-stab",
        gate_start_frames=gate_start,
        stab_start_frames=stab_start,
        fixed_gate_start_frames=fixed_gate_start,
        fixed_stab_start_frames=fixed_stab_start,
        gate_start_seconds=gate_start / SAMPLE_RATE,
        stab_start_seconds=stab_start / SAMPLE_RATE,
        gate_static_distance_frames=abs(gate_start - fixed_gate_start),
        stab_static_distance_frames=abs(stab_start - fixed_stab_start),
        gate_stab_distance_frames=abs(gate_start - stab_start),
        gate_duty=gate_duty,
        texture_gain=texture_gain,
        stab_gain=stab_gain,
        candidate_count=len(candidates),
    )


def fixed_tail_shape_policy(
    source_family: str,
    stutter_step_divisor: int,
    restore_snap_gain: float,
    candidate_count: int = 1,
) -> TailShapePolicy:
    return TailShapePolicy(
        source_aware=False,
        source_family=source_family,
        selection_strategy="fallback-fixed-dropout-restore-tail",
        dropout_silence_fraction=0.50,
        dropout_silence_gain=0.015,
        stutter_step_divisor=stutter_step_divisor,
        stutter_grain_gain=3.15,
        stutter_snap_gain=2.05 * restore_snap_gain,
        restore_source_gain=1.35,
        restore_snap_gain=4.80 * restore_snap_gain,
        restore_w30_gain=2.62,
        restore_mc202_gain=4.05,
        restore_tr909_gain=3.45,
        restore_drive=1.95,
        restore_slam=0.44,
        fixed_tail_distance=0.0,
        source_energy_span=0.0,
        candidate_count=candidate_count,
    )


def tail_shape_policy_for(
    source: np.ndarray,
    w30: np.ndarray,
    bar_frames: int,
    source_family: str,
    stutter_step_divisor: int,
    restore_snap_gain: float,
) -> TailShapePolicy:
    eligible = source_family in ("dense_break", "tonal_hook", "sparse_bass_pressure")
    candidate_count = min(6, max(0, source.shape[0] // max(1, bar_frames)))
    fixed = fixed_tail_shape_policy(
        source_family,
        stutter_step_divisor,
        restore_snap_gain,
        max(candidate_count, 1),
    )
    if not eligible or candidate_count < MIN_TAIL_SHAPE_CANDIDATES:
        return fixed

    candidates = []
    for bar in range(candidate_count):
        start = bar * bar_frames
        end = min(start + bar_frames, source.shape[0], w30.shape[0])
        if end <= start:
            continue
        chunk = source[start:end]
        w30_chunk = w30[start:end]
        candidates.append(
            {
                "source_rms": rms(chunk),
                "w30_rms": rms(w30_chunk),
                "low": low_band_rms(chunk),
                "high": high_band_ratio(chunk),
                "transient": transient_score(chunk),
            }
        )
    if len(candidates) < MIN_TAIL_SHAPE_CANDIDATES:
        return fixed_tail_shape_policy(
            source_family,
            stutter_step_divisor,
            restore_snap_gain,
            len(candidates),
        )

    source_rms_values = [item["source_rms"] for item in candidates]
    low_values = [item["low"] for item in candidates]
    high_values = [item["high"] for item in candidates]
    transient_values = [item["transient"] for item in candidates]
    w30_values = [item["w30_rms"] for item in candidates]
    source_mean = max(float(np.mean(source_rms_values)), 1e-9)
    low_norm = float(np.clip(np.mean(low_values) / source_mean, 0.0, 1.35))
    high_norm = float(np.clip(np.mean(high_values) / 0.35, 0.0, 1.35))
    transient_norm = float(
        np.clip(np.mean(transient_values) / max(source_mean * 7.0, 1e-9), 0.0, 1.35)
    )
    w30_norm = float(np.clip(np.mean(w30_values) / source_mean, 0.0, 1.35))
    energy_span = float(max(source_rms_values) - min(source_rms_values))
    energy_span_norm = float(np.clip(energy_span / source_mean, 0.0, 1.35))

    if source_family == "sparse_bass_pressure":
        strategy = "source-derived-bass-weighted-tail"
        silence_bias = 0.020
        density_bias = -1
        restore_bias = 0.18
    elif source_family == "tonal_hook":
        strategy = "source-derived-hook-readable-tail"
        silence_bias = -0.012
        density_bias = 0
        restore_bias = -0.08
    else:
        strategy = "source-derived-break-snap-tail"
        silence_bias = 0.006
        density_bias = 2
        restore_bias = 0.06

    dropout_silence_fraction = float(
        np.clip(
            fixed.dropout_silence_fraction
            + silence_bias
            + energy_span_norm * 0.035
            - high_norm * 0.018,
            0.42,
            0.59,
        )
    )
    dropout_silence_gain = float(
        np.clip(
            fixed.dropout_silence_gain
            + low_norm * 0.003
            - transient_norm * 0.004
            + (0.004 if source_family == "sparse_bass_pressure" else 0.0),
            0.006,
            0.024,
        )
    )
    derived_step = int(
        round(
            fixed.stutter_step_divisor
            + density_bias
            + transient_norm * 2.8
            - low_norm * 1.3
            + energy_span_norm * 1.2
        )
    )
    stutter_step_divisor = int(np.clip(derived_step, 10, 20))
    policy_values = {
        "dropout_silence_fraction": dropout_silence_fraction,
        "dropout_silence_gain": dropout_silence_gain,
        "stutter_step_divisor": stutter_step_divisor,
        "stutter_grain_gain": float(
            np.clip(fixed.stutter_grain_gain + transient_norm * 0.42 + high_norm * 0.18, 2.86, 4.12)
        ),
        "stutter_snap_gain": float(
            np.clip(
                fixed.stutter_snap_gain
                + transient_norm * 0.34
                + high_norm * 0.15
                + restore_bias * 0.35,
                1.82,
                3.30,
            )
        ),
        "restore_source_gain": float(
            np.clip(fixed.restore_source_gain + low_norm * 0.12 + restore_bias, 1.05, 1.68)
        ),
        "restore_snap_gain": float(
            np.clip(fixed.restore_snap_gain + transient_norm * 0.76 + restore_bias, 4.20, 7.20)
        ),
        "restore_w30_gain": float(
            np.clip(fixed.restore_w30_gain + w30_norm * 0.36 + restore_bias, 2.12, 3.26)
        ),
        "restore_mc202_gain": float(
            np.clip(fixed.restore_mc202_gain + low_norm * 0.30 + restore_bias, 3.42, 4.86)
        ),
        "restore_tr909_gain": float(
            np.clip(fixed.restore_tr909_gain + transient_norm * 0.34 + high_norm * 0.12, 2.92, 4.16)
        ),
        "restore_drive": float(
            np.clip(fixed.restore_drive + transient_norm * 0.08 + low_norm * 0.05, 1.88, 2.12)
        ),
        "restore_slam": float(
            np.clip(fixed.restore_slam + transient_norm * 0.035 + energy_span_norm * 0.020, 0.40, 0.52)
        ),
    }
    distance = float(
        abs(policy_values["dropout_silence_fraction"] - fixed.dropout_silence_fraction) / 0.10
        + abs(policy_values["dropout_silence_gain"] - fixed.dropout_silence_gain) / 0.010
        + abs(policy_values["stutter_step_divisor"] - fixed.stutter_step_divisor) / 4.0
        + abs(policy_values["stutter_grain_gain"] - fixed.stutter_grain_gain) / 0.80
        + abs(policy_values["stutter_snap_gain"] - fixed.stutter_snap_gain) / 0.80
        + abs(policy_values["restore_source_gain"] - fixed.restore_source_gain) / 0.50
        + abs(policy_values["restore_snap_gain"] - fixed.restore_snap_gain) / 1.80
        + abs(policy_values["restore_w30_gain"] - fixed.restore_w30_gain) / 0.80
        + abs(policy_values["restore_mc202_gain"] - fixed.restore_mc202_gain) / 1.00
        + abs(policy_values["restore_tr909_gain"] - fixed.restore_tr909_gain) / 0.80
        + abs(policy_values["restore_drive"] - fixed.restore_drive) / 0.30
        + abs(policy_values["restore_slam"] - fixed.restore_slam) / 0.12
    )
    return TailShapePolicy(
        source_aware=True,
        source_family=source_family,
        selection_strategy=strategy,
        fixed_tail_distance=distance,
        source_energy_span=energy_span,
        candidate_count=len(candidates),
        **policy_values,
    )


def dense_break_source_policy(
    source: np.ndarray,
    w30: np.ndarray,
    bar_frames: int,
    *,
    timing_confidence_result: str | None = None,
    timing_grid_use: str | None = None,
) -> DenseBreakSourcePolicy:
    first_two_bars = source[: min(2 * bar_frames, source.shape[0])]
    profile = audio_metrics(first_two_bars)
    pressure_lift_policy = pressure_lift_policy_for(
        low_band_rms=profile.low_band_rms,
        high_band_ratio=profile.high_band_ratio,
        transient_score=profile.transient_score,
        timing_confidence_result=timing_confidence_result,
        timing_grid_use=timing_grid_use,
    )
    arrangement_policy = arrangement_policy_for(
        source,
        w30,
        bar_frames,
        pressure_lift_policy.source_family,
    )
    hook_chop_policy = hook_chop_policy_for(
        source,
        w30,
        bar_frames,
        pressure_lift_policy.source_family,
    )

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
    elif pressure_lift_policy.source_family == "pad_noise":
        pressure_shape = "gated pad/noise texture lift"
        stutter_density = "slow noise-gate stutter"
        restore_hit_shape = "texture stab restore"
        stutter_step_divisor = 12
        stutter_grain_beat_offset = 0.25
        restore_snap_gain = 1.30
    elif pressure_lift_policy.source_family == "bad_timing":
        pressure_shape = "manual-confirm cautious lift"
        stutter_density = "cautious downbeat-check stutter"
        restore_hit_shape = "confirmation-cue restore"
        stutter_step_divisor = 16
        stutter_grain_beat_offset = 0.25
        restore_snap_gain = 1.80
    else:
        pressure_shape = "thin-source support lift"
        stutter_density = "busy recovery stutter"
        restore_hit_shape = "snap-assisted restore"
        stutter_step_divisor = 18
        stutter_grain_beat_offset = 0.25
        restore_snap_gain = 1.18

    destructive_gesture_policy = destructive_gesture_policy_for(
        source,
        w30,
        bar_frames,
        pressure_lift_policy.source_family,
        stutter_grain_beat_offset,
    )
    mix_treatment_policy = mix_treatment_policy_for(
        source,
        w30,
        bar_frames,
        pressure_lift_policy.source_family,
    )
    pad_noise_texture_policy = pad_noise_texture_policy_for(
        source,
        w30,
        bar_frames,
        pressure_lift_policy.source_family,
    )
    tail_shape_policy = tail_shape_policy_for(
        source,
        w30,
        bar_frames,
        pressure_lift_policy.source_family,
        stutter_step_divisor,
        restore_snap_gain,
    )

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
    elif pressure_lift_policy.source_family == "pad_noise":
        bass_restore = 55.0
        pressure_gain = 1.06
        bass_gain = 0.92
    elif pressure_lift_policy.source_family == "bad_timing":
        bass_restore = 50.0
        pressure_gain = 1.10
        bass_gain = 0.96
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
        hook_chop_policy=hook_chop_policy,
        destructive_gesture_policy=destructive_gesture_policy,
        mix_treatment_policy=mix_treatment_policy,
        pad_noise_texture_policy=pad_noise_texture_policy,
        tail_shape_policy=tail_shape_policy,
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


def scripted_arrangement_roles(source_family: str) -> tuple[tuple[str, ...], str, str]:
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
    elif source_family == "pad_noise":
        roles = ("hook", "chop", "hook", "chop", "pressure", "pressure", "dropout", "restore")
        shape = "texture gate caution"
        intent = "treat noisy pad material as a texture gate, not a proven breakbeat"
    elif source_family == "bad_timing":
        roles = ("hook", "chop", "hook", "chop", "pressure", "pressure", "dropout", "restore")
        shape = "manual-confirm cautious cut"
        intent = "avoid confident bar-locked rearrangement until ambiguous downbeat timing is confirmed"
    else:
        roles = ("hook", "chop", "hook", "pressure", "chop", "pressure", "dropout", "restore")
        shape = "cautious recovery lift"
        intent = "avoid pretending weak material is a dense break while still proving contrast"
    return roles, shape, intent


def arrangement_policy_for(
    source: np.ndarray,
    w30: np.ndarray,
    bar_frames: int,
    source_family: str,
) -> ArrangementPolicy:
    scripted_roles, scripted_shape, scripted_intent = scripted_arrangement_roles(source_family)
    eligible = source_family in ("dense_break", "tonal_hook", "sparse_bass_pressure")
    candidate_count = min(6, max(0, source.shape[0] // max(1, bar_frames)))
    if not eligible or candidate_count < MIN_ARRANGEMENT_ROLE_CANDIDATES:
        return ArrangementPolicy(
            source_aware=True,
            role_order_source_derived=False,
            source_family=source_family,
            selection_strategy="fallback-scripted-source-family-role-order",
            role_order=scripted_roles,
            role_order_signature=">".join(scripted_roles),
            scripted_role_order=scripted_roles,
            scripted_role_order_signature=">".join(scripted_roles),
            scripted_role_distance=0,
            candidate_count=max(candidate_count, 1),
            section_score_span=0.0,
            arrangement_shape=scripted_shape,
            arrangement_intent=scripted_intent,
        )

    candidates = []
    for bar in range(candidate_count):
        start = bar * bar_frames
        end = min(start + bar_frames, source.shape[0], w30.shape[0])
        chunk = source[start:end]
        w30_chunk = w30[start:end]
        if chunk.shape[0] == 0:
            continue
        source_rms = rms(chunk)
        w30_rms = rms(w30_chunk)
        transient = transient_score(chunk)
        low = low_band_rms(chunk)
        high = high_band_ratio(chunk)
        early_bias = max(0.0, (5.0 - float(bar))) * 0.004
        late_bias = float(bar) * 0.008
        middle_bias = (1.0 - abs(float(bar) - 2.5) / 2.5) * 0.006
        if source_family == "tonal_hook":
            hook_score = source_rms * 1.22 + low * 0.90 + w30_rms * 0.82 + early_bias
            chop_score = transient * 12.0 + w30_rms * 1.18 + high * 0.018 + middle_bias
            pressure_score = low * 1.18 + source_rms * 0.82 + late_bias
            strategy = "tonal-section-hook-pressure-arrangement"
        elif source_family == "sparse_bass_pressure":
            hook_score = transient * 7.0 + w30_rms * 0.95 + source_rms * 0.62 + early_bias
            chop_score = transient * 10.0 + w30_rms * 1.05 + high * 0.014 + middle_bias
            pressure_score = low * 1.55 + source_rms * 0.76 + late_bias
            strategy = "sparse-lowband-pressure-arrangement"
        else:
            hook_score = transient * 15.0 + high * 0.022 + w30_rms * 1.02 + early_bias
            chop_score = transient * 13.0 + w30_rms * 1.28 + source_rms * 0.46 + middle_bias
            pressure_score = low * 1.08 + source_rms * 0.72 + transient * 2.0 + late_bias
            strategy = "dense-transient-pressure-arrangement"
        candidates.append(
            {
                "bar": bar,
                "hook": float(hook_score),
                "chop": float(chop_score),
                "pressure": float(pressure_score),
            }
        )

    if len(candidates) < MIN_ARRANGEMENT_ROLE_CANDIDATES:
        return ArrangementPolicy(
            source_aware=True,
            role_order_source_derived=False,
            source_family=source_family,
            selection_strategy="fallback-scripted-source-family-role-order",
            role_order=scripted_roles,
            role_order_signature=">".join(scripted_roles),
            scripted_role_order=scripted_roles,
            scripted_role_order_signature=">".join(scripted_roles),
            scripted_role_distance=0,
            candidate_count=len(candidates),
            section_score_span=0.0,
            arrangement_shape=scripted_shape,
            arrangement_intent=scripted_intent,
        )

    pressure_bars = {
        int(item["bar"])
        for item in sorted(candidates[1:], key=lambda item: item["pressure"], reverse=True)[:2]
    }
    remaining = [item for item in candidates if int(item["bar"]) not in pressure_bars]
    hook_bars = {
        int(item["bar"])
        for item in sorted(remaining, key=lambda item: item["hook"], reverse=True)[:2]
    }
    roles = []
    for item in candidates:
        bar = int(item["bar"])
        if bar in pressure_bars:
            roles.append("pressure")
        elif bar in hook_bars:
            roles.append("hook")
        else:
            roles.append("chop")
    roles = tuple(roles[:6]) + ("dropout", "restore")
    score_values = [
        score
        for item in candidates
        for score in (float(item["hook"]), float(item["chop"]), float(item["pressure"]))
    ]
    scripted_distance = sum(1 for left, right in zip(roles, scripted_roles) if left != right)
    if source_family == "tonal_hook":
        shape = "source-section hook return"
        intent = "place hook, chop, and pressure where tonal section evidence supports contrast"
    elif source_family == "sparse_bass_pressure":
        shape = "source-section bass shove"
        intent = "place pressure where low-band source sections can carry the rebuild"
    else:
        shape = "source-section break slam"
        intent = "place hook, chop, and pressure from transient and W-30 section evidence before the cut"
    return ArrangementPolicy(
        source_aware=True,
        role_order_source_derived=True,
        source_family=source_family,
        selection_strategy=strategy,
        role_order=roles,
        role_order_signature=">".join(roles),
        scripted_role_order=scripted_roles,
        scripted_role_order_signature=">".join(scripted_roles),
        scripted_role_distance=scripted_distance,
        candidate_count=len(candidates),
        section_score_span=float(max(score_values) - min(score_values)) if score_values else 0.0,
        arrangement_shape=shape,
        arrangement_intent=intent,
    )


def pressure_lift_policy_for(
    low_band_rms: float,
    high_band_ratio: float,
    transient_score: float,
    *,
    timing_confidence_result: str | None = None,
    timing_grid_use: str | None = None,
) -> PressureLiftPolicy:
    if timing_confidence_result == "candidate_ambiguous" or timing_grid_use == "manual_confirm_only":
        return PressureLiftPolicy(
            source_aware=True,
            source_family="bad_timing",
            lift_shape="manual-confirm cautious lift",
            lift_intent="keep the render audible but cue timing confirmation before confident bar-locked moves",
            source_bleed_gain=0.050,
            hook_bleed_gain=0.66,
            tr909_drive=1.02,
            break_snap_drive=0.98,
            mc202_drive=0.92,
            bass_drive=0.94,
            bar4_intensity=0.92,
            bar5_intensity=1.03,
            bar4_bass_frequency_hz=40.0,
            bar5_bass_frequency_hz=47.0,
        )
    if (
        low_band_rms < MAX_PAD_NOISE_LOW_BAND_RMS
        and high_band_ratio >= MIN_PAD_NOISE_HIGH_BAND_RATIO
        and transient_score >= MIN_PAD_NOISE_TRANSIENT_SCORE
    ):
        return PressureLiftPolicy(
            source_aware=True,
            source_family="pad_noise",
            lift_shape="gated texture lift",
            lift_intent="gate noisy pad material as texture instead of promoting it to breakbeat proof",
            source_bleed_gain=0.030,
            hook_bleed_gain=0.56,
            tr909_drive=0.98,
            break_snap_drive=0.92,
            mc202_drive=0.76,
            bass_drive=0.82,
            bar4_intensity=0.90,
            bar5_intensity=1.04,
            bar4_bass_frequency_hz=48.0,
            bar5_bass_frequency_hz=55.0,
        )
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
            hook_bleed_gain=0.96,
            tr909_drive=0.92,
            break_snap_drive=1.08,
            mc202_drive=0.92,
            bass_drive=0.94,
            bar4_intensity=0.92,
            bar5_intensity=1.05,
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
            bar4_intensity=0.92,
            bar5_intensity=1.08,
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
    hook_riff = render_w30_hook_riff_layer(w30, source, source_policy, bar_frames, bars)
    break_snap = render_break_snap_layer(source, tr909, w30, bar_frames, bars)
    bass_pressure = render_bass_pressure_layer(source, source_policy, bar_frames, bars)
    pad_noise_texture = render_pad_noise_texture_layer(source, w30, source_policy, bar_frames, bars)
    lift_policy = source_policy.pressure_lift_policy
    mix_policy = source_policy.mix_treatment_policy
    role_order = source_policy.arrangement_policy.role_order

    def put_bar(bar: int, mix: np.ndarray) -> None:
        start = bar * bar_frames
        end = min(start + bar_frames, performance.shape[0])
        if start >= end:
            return
        performance[start:end] = mix[start:end]

    hook_mix = glue_bus(
        source * (0.50 * source_layer_gain)
        + w30 * mix_policy.hook_w30_gain
        + hook_riff * 1.62
        + tr909 * 0.62
        + break_snap * mix_policy.hook_break_snap_gain
        + pad_noise_texture * 0.68
        + mc202 * 0.34,
        drive=mix_policy.hook_drive,
        slam=mix_policy.hook_slam,
    )
    chop_mix = glue_bus(
        source * (0.16 * source_layer_gain)
        + w30 * mix_policy.chop_w30_gain
        + hook_riff * 1.78
        + tr909 * 0.78
        + break_snap * mix_policy.chop_break_snap_gain
        + pad_noise_texture * 1.05
        + mc202 * 0.58,
        drive=mix_policy.chop_drive,
        slam=mix_policy.chop_slam,
    )
    pressure_mix = saturate(
        source * (lift_policy.source_bleed_gain * source_layer_gain)
        + w30 * mix_policy.pressure_w30_gain
        + hook_riff * lift_policy.hook_bleed_gain
        + tr909 * (2.28 + lift_policy.tr909_drive * 0.52)
        + break_snap * (mix_policy.pressure_break_snap_gain * lift_policy.break_snap_drive)
        + mc202 * (5.00 + lift_policy.mc202_drive * 1.42)
        + pad_noise_texture * 1.28
        + bass_pressure * (1.14 + lift_policy.bass_drive * 0.62),
        1.58,
    )
    pressure_mix = normalize_peak(
        glue_bus(
            pressure_mix,
            drive=mix_policy.pressure_drive,
            slam=mix_policy.pressure_slam,
        ),
        mix_policy.pressure_peak,
    )
    restore_mix = glue_bus(
        source * (0.28 * source_layer_gain)
        + w30 * 1.76
        + hook_riff * 1.46
        + tr909 * 2.78
        + break_snap * mix_policy.restore_break_snap_gain
        + mc202 * 3.65
        + pad_noise_texture * 1.42
        + bass_pressure * mix_policy.restore_bass_gain,
        drive=mix_policy.restore_drive,
        slam=mix_policy.restore_slam,
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

    performance = normalize_peak(
        glue_bus(performance, drive=mix_policy.final_drive, slam=mix_policy.final_slam),
        TARGET_PERFORMANCE_PEAK,
    )
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
    eligible = policy.source_family in ("dense_break", "tonal_hook", "sparse_bass_pressure")
    if eligible and not policy.role_order_source_derived:
        failures.append("arrangement_role_order_not_source_derived")
    if eligible and policy.candidate_count < MIN_ARRANGEMENT_ROLE_CANDIDATES:
        failures.append("arrangement_role_order_not_enough_candidates")
    if eligible and policy.scripted_role_distance < MIN_ARRANGEMENT_SCRIPTED_ROLE_DISTANCE:
        failures.append("arrangement_role_order_collapsed_to_scripted")
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


def bass_movement_frequency_policy(
    source: np.ndarray,
    source_policy: DenseBreakSourcePolicy,
    bar_frames: int,
) -> dict[int, float]:
    static = arrangement_role_frequency_policy(source_policy)
    if source_policy.pressure_lift_policy.source_family != "sparse_bass_pressure":
        return static

    reference = source[: min(max(1, 2 * bar_frames), source.shape[0])]
    reference_low = max(low_band_rms(reference), 1e-6)
    derived = {}
    for bar, base_frequency in static.items():
        start = bar * bar_frames
        end = min(start + bar_frames, source.shape[0])
        if start >= end:
            derived[bar] = base_frequency
            continue
        chunk = source[start:end]
        local_low = low_band_rms(chunk)
        mono = chunk.mean(axis=1)
        low = np.abs(one_pole_lowpass(mono.astype(np.float32), 140.0))
        weight = float(np.sum(low))
        centroid = 0.50
        if weight > 1e-9:
            positions = np.arange(low.shape[0], dtype=np.float32) / max(1, low.shape[0] - 1)
            centroid = float(np.sum(positions * low) / weight)
        energy_offset = float(np.clip((local_low / reference_low - 1.0) * 4.0, -5.0, 5.0))
        timing_offset = float(np.clip((centroid - 0.50) * 9.0, -4.5, 4.5))
        transient_offset = float(np.clip(transient_score(chunk) * 14.0, 0.0, 3.0))
        if source_policy.arrangement_policy.role_order[bar] == "restore":
            timing_offset *= 0.55
            transient_offset *= 0.60
        derived[bar] = float(
            np.clip(base_frequency + energy_offset + timing_offset + transient_offset, 32.0, 62.0)
        )
    return derived


def bass_movement_policy_proof(
    source: np.ndarray,
    source_policy: DenseBreakSourcePolicy,
    bar_frames: int,
) -> dict[str, float]:
    static = arrangement_role_frequency_policy(source_policy)
    derived = bass_movement_frequency_policy(source, source_policy, bar_frames)
    if not derived:
        return {
            "bass_movement_source_derived": 0.0,
            "sparse_bass_movement_static_distance_hz": 0.0,
            "sparse_bass_movement_frequency_span_hz": 0.0,
        }
    distances = [abs(derived[bar] - static.get(bar, derived[bar])) for bar in derived]
    frequencies = list(derived.values())
    is_sparse = source_policy.pressure_lift_policy.source_family == "sparse_bass_pressure"
    return {
        "bass_movement_source_derived": 1.0 if is_sparse else 0.0,
        "sparse_bass_movement_static_distance_hz": float(np.mean(distances)) if is_sparse else 0.0,
        "sparse_bass_movement_frequency_span_hz": (
            float(max(frequencies) - min(frequencies)) if is_sparse else 0.0
        ),
    }


def hook_chop_policy_proof(source_policy: DenseBreakSourcePolicy) -> dict[str, float]:
    policy = source_policy.hook_chop_policy
    return {
        "hook_chop_selection_source_derived": 1.0 if policy.source_aware else 0.0,
        "hook_chop_selection_candidate_count": float(policy.candidate_count),
        "hook_chop_hook_start_frames": float(policy.hook_start_frames),
        "hook_chop_chop_start_frames": float(policy.chop_start_frames),
        "hook_chop_static_first_bar_start_frames": float(policy.static_first_bar_start_frames),
        "hook_chop_hook_static_distance_frames": float(policy.hook_static_distance_frames),
        "hook_chop_chop_static_distance_frames": float(policy.chop_static_distance_frames),
        "hook_chop_static_distance_frames": float(
            max(policy.hook_static_distance_frames, policy.chop_static_distance_frames)
        ),
        "hook_chop_offset_distance_frames": float(policy.hook_chop_distance_frames),
    }


def destructive_gesture_policy_proof(source_policy: DenseBreakSourcePolicy) -> dict[str, float]:
    policy = source_policy.destructive_gesture_policy
    return {
        "destructive_gesture_source_derived": 1.0 if policy.source_aware else 0.0,
        "destructive_gesture_candidate_count": float(policy.candidate_count),
        "destructive_stutter_start_frames": float(policy.stutter_start_frames),
        "destructive_restore_start_frames": float(policy.restore_start_frames),
        "destructive_fixed_stutter_start_frames": float(policy.fixed_stutter_start_frames),
        "destructive_fixed_restore_start_frames": float(policy.fixed_restore_start_frames),
        "destructive_stutter_static_distance_frames": float(
            policy.stutter_static_distance_frames
        ),
        "destructive_restore_static_distance_frames": float(
            policy.restore_static_distance_frames
        ),
        "destructive_static_distance_frames": float(
            min(policy.stutter_static_distance_frames, policy.restore_static_distance_frames)
        ),
        "destructive_offset_distance_frames": float(policy.stutter_restore_distance_frames),
    }


def mix_treatment_policy_proof(source_policy: DenseBreakSourcePolicy) -> dict[str, float]:
    policy = source_policy.mix_treatment_policy
    return {
        "mix_treatment_source_derived": 1.0 if policy.source_aware else 0.0,
        "mix_treatment_candidate_count": float(policy.candidate_count),
        "mix_treatment_fixed_distance": float(policy.fixed_treatment_distance),
        "mix_treatment_source_energy_span": float(policy.source_energy_span),
        "mix_treatment_hook_drive": float(policy.hook_drive),
        "mix_treatment_chop_drive": float(policy.chop_drive),
        "mix_treatment_pressure_drive": float(policy.pressure_drive),
        "mix_treatment_restore_drive": float(policy.restore_drive),
        "mix_treatment_pressure_peak": float(policy.pressure_peak),
        "mix_treatment_restore_bass_gain": float(policy.restore_bass_gain),
    }


def pad_noise_texture_policy_proof(source_policy: DenseBreakSourcePolicy) -> dict[str, float]:
    policy = source_policy.pad_noise_texture_policy
    return {
        "pad_noise_texture_source_derived": 1.0 if policy.source_aware else 0.0,
        "pad_noise_texture_candidate_count": float(policy.candidate_count),
        "pad_noise_texture_gate_static_distance_frames": float(
            policy.gate_static_distance_frames
        ),
        "pad_noise_texture_stab_static_distance_frames": float(
            policy.stab_static_distance_frames
        ),
        "pad_noise_texture_gate_stab_distance_frames": float(
            policy.gate_stab_distance_frames
        ),
        "pad_noise_texture_gate_duty": float(policy.gate_duty),
        "pad_noise_texture_gain": float(policy.texture_gain),
        "pad_noise_texture_stab_gain": float(policy.stab_gain),
    }


def tail_shape_policy_proof(source_policy: DenseBreakSourcePolicy) -> dict[str, float]:
    policy = source_policy.tail_shape_policy
    return {
        "tail_shape_source_derived": 1.0 if policy.source_aware else 0.0,
        "tail_shape_candidate_count": float(policy.candidate_count),
        "tail_shape_fixed_distance": float(policy.fixed_tail_distance),
        "tail_shape_source_energy_span": float(policy.source_energy_span),
        "tail_shape_dropout_silence_fraction": float(policy.dropout_silence_fraction),
        "tail_shape_dropout_silence_gain": float(policy.dropout_silence_gain),
        "tail_shape_stutter_step_divisor": float(policy.stutter_step_divisor),
        "tail_shape_stutter_grain_gain": float(policy.stutter_grain_gain),
        "tail_shape_stutter_snap_gain": float(policy.stutter_snap_gain),
        "tail_shape_restore_source_gain": float(policy.restore_source_gain),
        "tail_shape_restore_snap_gain": float(policy.restore_snap_gain),
        "tail_shape_restore_drive": float(policy.restore_drive),
        "tail_shape_restore_slam": float(policy.restore_slam),
    }


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
    tail_policy = source_policy.tail_shape_policy

    base = saturate(
        source[source_start:source_end] * (0.10 * source_layer_gain)
        + w30[source_start:source_end] * 1.35
        + tr909[source_start:source_end] * 0.46
        + mc202[source_start:source_end] * 0.70,
        1.22,
    )
    bar[: base.shape[0]] = base

    dropout_end = int(round(bar_frames * tail_policy.dropout_silence_fraction))
    dropout_end = max(1, min(bar_frames - 1, dropout_end))
    bar[:dropout_end] *= tail_policy.dropout_silence_gain

    grain_len = max(128, bar_frames // 32)
    beat_frames = max(1, bar_frames // BEATS_PER_BAR)
    if source_policy.destructive_gesture_policy.source_aware:
        grain_source_start = source_policy.destructive_gesture_policy.stutter_start_frames
    else:
        grain_source_start = source_start + int(round(source_policy.stutter_grain_beat_offset * beat_frames))
    grain_source_end = min(grain_source_start + grain_len, w30.shape[0])
    if grain_source_end <= grain_source_start:
        return bar
    grain = w30[grain_source_start:grain_source_end].copy()
    grain *= hann_envelope(grain.shape[0])[:, None]
    source_snap_grain = transient_emphasis(source[grain_source_start:grain_source_end].copy())
    source_snap_grain *= impact_envelope(source_snap_grain.shape[0], decay=0.040)[:, None]
    source_snap_grain = normalize_peak(source_snap_grain, 0.78)
    cue_snap_grain = None
    if source_policy.pressure_lift_policy.source_family == "bad_timing":
        cue_snap_grain = np.zeros_like(grain)
        click_len = min(cue_snap_grain.shape[0], 192)
        cue = np.linspace(1.0, -0.65, click_len, dtype=np.float32)
        cue *= impact_envelope(click_len, decay=0.018)
        cue_snap_grain[:click_len, 0] = cue
        cue_snap_grain[:click_len, 1] = cue * 0.35

    step = max(1, bar_frames // max(1, tail_policy.stutter_step_divisor))
    for index, target in enumerate(range(dropout_end, bar_frames - grain.shape[0], step)):
        decay = 1.0 - min(index, 7) * 0.07
        accent = tr909[min(source_start + target, tr909.shape[0] - 1)]
        end = target + grain.shape[0]
        riff = hook_riff[min(source_start + target, hook_riff.shape[0] - 1)]
        snap = break_snap[min(source_start + target, break_snap.shape[0] - 1)]
        bar[target:end] += grain * (tail_policy.stutter_grain_gain * decay * source_policy.pressure_gain)
        bar[target:end] += source_snap_grain[: end - target] * (
            tail_policy.stutter_snap_gain * decay
        )
        if cue_snap_grain is not None:
            bar[target:end] += cue_snap_grain[: end - target] * (5.50 * decay)
        bar[target : min(target + 96, bar.shape[0])] += accent * (0.58 * decay)
        bar[target : min(target + 160, bar.shape[0])] += (riff + snap) * (1.02 * decay)

    return normalize_peak(saturate(bar, 1.78), 0.90)


def render_pad_noise_texture_layer(
    source: np.ndarray,
    w30: np.ndarray,
    source_policy: DenseBreakSourcePolicy,
    bar_frames: int,
    bars: int,
) -> np.ndarray:
    policy = source_policy.pad_noise_texture_policy
    layer = np.zeros_like(source)
    if not policy.source_aware or policy.source_family != "pad_noise":
        return layer

    grain_len = min(frames_for_seconds(0.160), max(1, bar_frames // 5))
    gate_start = min(policy.gate_start_frames, max(0, source.shape[0] - 1))
    stab_start = min(policy.stab_start_frames, max(0, source.shape[0] - 1))
    gate_end = min(gate_start + grain_len, source.shape[0], w30.shape[0])
    stab_end = min(stab_start + grain_len, source.shape[0], w30.shape[0])
    if gate_end <= gate_start or stab_end <= stab_start:
        return layer

    gate_grain = source[gate_start:gate_end].copy() * 0.72 + w30[gate_start:gate_end] * 0.36
    gate_grain += transient_emphasis(source[gate_start:gate_end]) * 0.24
    gate_grain *= hann_envelope(gate_grain.shape[0])[:, None]
    gate_grain = normalize_peak(saturate(gate_grain, 1.35), 0.72)

    stab_grain = transient_emphasis(source[stab_start:stab_end]) * 1.18 + w30[stab_start:stab_end] * 0.50
    stab_grain *= impact_envelope(stab_grain.shape[0], decay=0.060)[:, None]
    stab_grain = normalize_peak(saturate(stab_grain, 1.55), 0.82)

    eighth = max(1, bar_frames // 8)
    pulse_len = max(96, min(gate_grain.shape[0], int(round(eighth * policy.gate_duty))))
    gate_pulse = gate_grain[:pulse_len].copy()
    gate_pulse *= decay_envelope(gate_pulse.shape[0], attack=0.008, decay=0.090)[:, None]

    role_order = source_policy.arrangement_policy.role_order
    role_gain = {
        "hook": 0.62,
        "chop": 0.96,
        "pressure": 1.18,
        "dropout": 0.72,
        "restore": 1.36,
    }
    role_offsets = {
        "hook": (0, 3, 6),
        "chop": (0, 2, 5, 7),
        "pressure": (0, 1, 3, 5, 7),
        "dropout": (4, 6),
        "restore": (0, 2, 6),
    }
    for bar in range(min(bars, len(role_order))):
        role = role_order[bar]
        base = bar * bar_frames
        gain = role_gain.get(role, 0.80) * policy.texture_gain
        for offset in role_offsets.get(role, (0, 4)):
            target = base + offset * eighth
            if target >= layer.shape[0]:
                continue
            end = min(target + gate_pulse.shape[0], layer.shape[0])
            if end > target:
                layer[target:end] += gate_pulse[: end - target] * gain
        if role in {"chop", "pressure", "restore"}:
            target = base + (eighth if role == "restore" else 0)
            end = min(target + stab_grain.shape[0], layer.shape[0])
            if end > target:
                layer[target:end] += stab_grain[: end - target] * (
                    policy.stab_gain * (1.25 if role == "restore" else 0.86)
                )
    return saturate(layer, 1.18)


def render_w30_hook_riff_layer(
    w30: np.ndarray,
    source: np.ndarray,
    source_policy: DenseBreakSourcePolicy,
    bar_frames: int,
    bars: int,
) -> np.ndarray:
    layer = np.zeros_like(w30)
    grain_len = min(frames_for_seconds(0.090), max(1, bar_frames // 8))
    hook_start = min(source_policy.hook_chop_policy.hook_start_frames, w30.shape[0] - 1)
    chop_start = min(source_policy.hook_chop_policy.chop_start_frames, w30.shape[0] - 1)
    hook_end = min(hook_start + grain_len, w30.shape[0], source.shape[0])
    chop_end = min(chop_start + grain_len, w30.shape[0], source.shape[0])
    if hook_end <= hook_start or chop_end <= chop_start:
        return layer

    hook_grain = w30[hook_start:hook_end].copy()
    hook_grain += transient_emphasis(source[hook_start:hook_end]) * 0.42
    hook_grain *= decay_envelope(hook_grain.shape[0], attack=0.010, decay=0.135)[:, None]
    chop_grain = w30[chop_start:chop_end].copy()
    chop_grain += transient_emphasis(source[chop_start:chop_end]) * 0.54
    chop_grain *= decay_envelope(chop_grain.shape[0], attack=0.006, decay=0.105)[:, None]

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
            source_grain = chop_grain if reverse or bar in (2, 3, 5) else hook_grain
            stab = source_grain[::-1] if reverse else source_grain
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
    frequencies = bass_movement_frequency_policy(source, source_policy, bar_frames)
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
    tail_policy = source_policy.tail_shape_policy
    hit_frames = min(frames_for_seconds(0.115), end - start)
    source_hit_start = 0
    if source_policy.destructive_gesture_policy.source_aware:
        source_hit_start = min(
            source_policy.destructive_gesture_policy.restore_start_frames,
            source.shape[0] - 1,
        )
    source_hit_end = min(source_hit_start + hit_frames, source.shape[0], w30.shape[0])
    if source_hit_end <= source_hit_start:
        return restored
    hit_frames = min(hit_frames, source_hit_end - source_hit_start)
    envelope = np.linspace(1.0, 0.0, hit_frames, dtype=np.float32)[:, None]
    source_hit = source[source_hit_start:source_hit_end]
    w30_hit = w30[source_hit_start:source_hit_end]
    snap = transient_emphasis(source_hit)
    restored[start : start + hit_frames] += (
        source_hit * (tail_policy.restore_source_gain * source_layer_gain)
        + snap * tail_policy.restore_snap_gain
        + w30_hit * tail_policy.restore_w30_gain
        + mc202[start : start + hit_frames] * tail_policy.restore_mc202_gain
        + tr909[start : start + hit_frames] * tail_policy.restore_tr909_gain
    ) * envelope
    return normalize_peak(
        glue_bus(restored, drive=tail_policy.restore_drive, slam=tail_policy.restore_slam),
        0.98,
    )


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
    failure_codes = failure_codes_for(
        metrics, proof, source_policy.pressure_lift_policy.source_family
    )
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
            "hook_chop_policy": asdict(source_policy.hook_chop_policy),
            "destructive_gesture_policy": asdict(source_policy.destructive_gesture_policy),
            "mix_treatment_policy": asdict(source_policy.mix_treatment_policy),
            "pad_noise_texture_policy": asdict(source_policy.pad_noise_texture_policy),
            "tail_shape_policy": asdict(source_policy.tail_shape_policy),
            "arrangement_failure_routes": arrangement_failure_routes(
                arrangement_failure_codes(source_policy.arrangement_policy)
            ),
            "scripted_boundaries": [
                "role vocabulary remains bounded even when first-six role placement, mix treatment, and eligible dropout/restore tail shape are source-derived",
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
            "min_bad_timing_cue_transient_score": MIN_BAD_TIMING_CUE_TRANSIENT_SCORE,
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
            "min_sparse_bass_movement_static_distance_hz": (
                MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ
            ),
            "min_sparse_bass_movement_span_hz": MIN_SPARSE_BASS_MOVEMENT_SPAN_HZ,
            "min_hook_chop_selection_candidates": MIN_HOOK_CHOP_SELECTION_CANDIDATES,
            "min_hook_chop_static_distance_frames": MIN_HOOK_CHOP_STATIC_DISTANCE_FRAMES,
            "min_hook_chop_offset_distance_frames": MIN_HOOK_CHOP_OFFSET_DISTANCE_FRAMES,
            "min_destructive_gesture_candidates": MIN_DESTRUCTIVE_GESTURE_CANDIDATES,
            "min_destructive_static_distance_frames": MIN_DESTRUCTIVE_STATIC_DISTANCE_FRAMES,
            "min_destructive_offset_distance_frames": MIN_DESTRUCTIVE_OFFSET_DISTANCE_FRAMES,
            "min_arrangement_role_candidates": MIN_ARRANGEMENT_ROLE_CANDIDATES,
            "min_arrangement_scripted_role_distance": MIN_ARRANGEMENT_SCRIPTED_ROLE_DISTANCE,
            "min_mix_treatment_candidates": MIN_MIX_TREATMENT_CANDIDATES,
            "min_mix_treatment_fixed_distance": MIN_MIX_TREATMENT_FIXED_DISTANCE,
            "min_mix_treatment_output_contrast": MIN_MIX_TREATMENT_OUTPUT_CONTRAST,
            "min_pad_noise_texture_candidates": MIN_PAD_NOISE_TEXTURE_CANDIDATES,
            "min_pad_noise_texture_static_distance_frames": (
                MIN_PAD_NOISE_TEXTURE_STATIC_DISTANCE_FRAMES
            ),
            "min_pad_noise_texture_offset_distance_frames": (
                MIN_PAD_NOISE_TEXTURE_OFFSET_DISTANCE_FRAMES
            ),
            "min_pad_noise_texture_transient_ratio": MIN_PAD_NOISE_TEXTURE_TRANSIENT_RATIO,
            "min_tail_shape_candidates": MIN_TAIL_SHAPE_CANDIDATES,
            "min_tail_shape_fixed_distance": MIN_TAIL_SHAPE_FIXED_DISTANCE,
            "min_tail_shape_output_contrast": MIN_TAIL_SHAPE_OUTPUT_CONTRAST,
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
            "bounded source-aware pressure_lift/stutter/restore, arrangement, and mix "
            "policy plus eligible source-derived dropout/restore tail shaping and a "
            "source-layer-off rebuild diagnostic, but the renderer vocabulary remains "
            "bounded; this is smoke/"
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
    source_hook_window = source[: min(source.shape[0], sections["chop_hook"].shape[0])]
    dropout = sections["dropout_stutter"]
    dropout_first = dropout[: dropout.shape[0] // 2]
    dropout_second = dropout[dropout.shape[0] // 2 :]
    tail_silence = dropout[
        : int(round(dropout.shape[0] * source_policy.tail_shape_policy.dropout_silence_fraction))
    ]
    tail_stutter = dropout[
        int(round(dropout.shape[0] * source_policy.tail_shape_policy.dropout_silence_fraction)) :
    ]
    bar_similarity = max_adjacent_bar_correlation(performance, bar_frames)
    source_similarity = waveform_correlation(source, performance)
    rebuild_only_source_similarity = waveform_correlation(source, rebuild_only_performance)
    source_on_rebuild_only_similarity = waveform_correlation(performance, rebuild_only_performance)
    proof = {
        "w30_to_source_rms_ratio": w30_rms / max(source_rms, 1e-9),
        "w30_to_full_performance_rms_ratio": w30_rms / max(full_rms, 1e-9),
        "generated_to_w30_rms_ratio": (tr909_rms + mc202_rms) / max(w30_rms, 1e-9),
        "pressure_low_band_lift_ratio": pressure_low / max(hook_low, 1e-9),
        "dropout_to_stutter_rms_ratio": rms(dropout_first) / max(rms(dropout_second), 1e-9),
        "stutter_to_hook_transient_ratio": transient_score(dropout_second) / max(hook_transient, 1e-9),
        "manual_confirm_cue_transient_score": transient_score(dropout_second),
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
        "arrangement_role_order_source_derived": (
            1.0 if source_policy.arrangement_policy.role_order_source_derived else 0.0
        ),
        "arrangement_role_candidate_count": float(source_policy.arrangement_policy.candidate_count),
        "arrangement_scripted_role_distance": float(
            source_policy.arrangement_policy.scripted_role_distance
        ),
        "arrangement_section_score_span": float(
            source_policy.arrangement_policy.section_score_span
        ),
        "arrangement_role_order_hash": role_order_hash(source_policy.arrangement_policy),
        "arrangement_role_count": float(len(source_policy.arrangement_policy.role_order)),
        "arrangement_pressure_role_count": float(pressure_role_count(source_policy)),
        "arrangement_destructive_role_count": float(destructive_role_count(source_policy)),
        "arrangement_failure_count": float(
            len(arrangement_failure_codes(source_policy.arrangement_policy))
        ),
        "pressure_lift_bar5_to_bar4_rms_ratio": rms(pressure_bar5) / max(rms(pressure_bar4), 1e-9),
        "mix_treatment_output_contrast_ratio": (
            (pressure_rms + restore_rms) / max(hook_rms, 1e-9)
        ),
        "tail_shape_output_contrast_ratio": (
            (rms(tail_stutter) + restore_rms) / max(rms(tail_silence), 1e-9)
        ),
        "pad_noise_texture_transient_ratio": hook_transient
        / max(transient_score(source_hook_window), 1e-9),
        "pad_noise_texture_high_band_ratio": high_band_ratio(sections["chop_hook"]),
        "source_policy_pressure_gain": source_policy.pressure_gain,
        "source_policy_bass_gain": source_policy.bass_gain,
        "source_policy_stutter_step_divisor": float(source_policy.stutter_step_divisor),
    }
    proof.update(bass_movement_policy_proof(source, source_policy, bar_frames))
    proof.update(hook_chop_policy_proof(source_policy))
    proof.update(destructive_gesture_policy_proof(source_policy))
    proof.update(mix_treatment_policy_proof(source_policy))
    proof.update(pad_noise_texture_policy_proof(source_policy))
    proof.update(tail_shape_policy_proof(source_policy))
    return proof


def failure_codes_for(
    metrics: dict[str, dict],
    proof: dict[str, float],
    source_family: str | None = None,
) -> list[str]:
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
    if source_family == "bad_timing":
        if proof["manual_confirm_cue_transient_score"] < MIN_BAD_TIMING_CUE_TRANSIENT_SCORE:
            failures.append("bad_timing_confirmation_cue_too_weak")
    elif source_family == "pad_noise":
        if proof.get("pad_noise_texture_source_derived", 0.0) < 1.0:
            failures.append("pad_noise_texture_not_source_derived")
        if proof.get("pad_noise_texture_candidate_count", 0.0) < MIN_PAD_NOISE_TEXTURE_CANDIDATES:
            failures.append("pad_noise_texture_not_enough_candidates")
        if (
            proof.get("pad_noise_texture_gate_static_distance_frames", 0.0)
            < MIN_PAD_NOISE_TEXTURE_STATIC_DISTANCE_FRAMES
        ):
            failures.append("pad_noise_texture_gate_collapsed_to_fixed_choice")
        if (
            proof.get("pad_noise_texture_stab_static_distance_frames", 0.0)
            < MIN_PAD_NOISE_TEXTURE_STATIC_DISTANCE_FRAMES
        ):
            failures.append("pad_noise_texture_stab_collapsed_to_fixed_choice")
        if (
            proof.get("pad_noise_texture_gate_stab_distance_frames", 0.0)
            < MIN_PAD_NOISE_TEXTURE_OFFSET_DISTANCE_FRAMES
        ):
            failures.append("pad_noise_texture_gate_stab_offsets_too_close")
        if (
            proof.get("pad_noise_texture_transient_ratio", 0.0)
            < MIN_PAD_NOISE_TEXTURE_TRANSIENT_RATIO
        ):
            failures.append("pad_noise_texture_lacks_transient_shape")
    elif proof["stutter_to_hook_transient_ratio"] < MIN_STUTTER_TO_HOOK_TRANSIENT_RATIO:
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
    if source_family in ("dense_break", "tonal_hook", "sparse_bass_pressure"):
        if proof.get("arrangement_role_order_source_derived", 0.0) < 1.0:
            failures.append("arrangement_role_order_not_source_derived")
        if proof.get("arrangement_role_candidate_count", 0.0) < MIN_ARRANGEMENT_ROLE_CANDIDATES:
            failures.append("arrangement_role_order_not_enough_candidates")
        if (
            proof.get("arrangement_scripted_role_distance", 0.0)
            < MIN_ARRANGEMENT_SCRIPTED_ROLE_DISTANCE
        ):
            failures.append("arrangement_role_order_collapsed_to_scripted")
        if proof.get("mix_treatment_source_derived", 0.0) < 1.0:
            failures.append("mix_treatment_not_source_derived")
        if proof.get("mix_treatment_candidate_count", 0.0) < MIN_MIX_TREATMENT_CANDIDATES:
            failures.append("mix_treatment_not_enough_candidates")
        if proof.get("mix_treatment_fixed_distance", 0.0) < MIN_MIX_TREATMENT_FIXED_DISTANCE:
            failures.append("mix_treatment_collapsed_to_fixed_recipe")
        if proof.get("mix_treatment_output_contrast_ratio", 0.0) < MIN_MIX_TREATMENT_OUTPUT_CONTRAST:
            failures.append("mix_treatment_output_contrast_too_weak")
        if proof.get("tail_shape_source_derived", 0.0) < 1.0:
            failures.append("tail_shape_not_source_derived")
        if proof.get("tail_shape_candidate_count", 0.0) < MIN_TAIL_SHAPE_CANDIDATES:
            failures.append("tail_shape_not_enough_candidates")
        if proof.get("tail_shape_fixed_distance", 0.0) < MIN_TAIL_SHAPE_FIXED_DISTANCE:
            failures.append("tail_shape_collapsed_to_fixed_recipe")
        if proof.get("tail_shape_output_contrast_ratio", 0.0) < MIN_TAIL_SHAPE_OUTPUT_CONTRAST:
            failures.append("tail_shape_output_contrast_too_weak")
    if proof["arrangement_role_count"] != float(DEFAULT_BARS):
        failures.append("arrangement_role_order_not_8_bars")
    if proof["arrangement_pressure_role_count"] < 2.0:
        failures.append("arrangement_pressure_lift_too_short")
    if proof["arrangement_destructive_role_count"] < 2.0:
        failures.append("arrangement_destructive_restore_tail_missing")
    if proof["arrangement_failure_count"] > 0.0:
        failures.append("arrangement_policy_contract_failed")
    if source_family in ("dense_break", "tonal_hook"):
        if proof.get("hook_chop_selection_source_derived", 0.0) < 1.0:
            failures.append("hook_chop_selection_not_source_derived")
        if proof.get("hook_chop_selection_candidate_count", 0.0) < MIN_HOOK_CHOP_SELECTION_CANDIDATES:
            failures.append("hook_chop_selection_not_enough_candidates")
        if proof.get("hook_chop_static_distance_frames", 0.0) < MIN_HOOK_CHOP_STATIC_DISTANCE_FRAMES:
            failures.append("hook_chop_selection_collapsed_to_static_first_bar")
        if proof.get("hook_chop_offset_distance_frames", 0.0) < MIN_HOOK_CHOP_OFFSET_DISTANCE_FRAMES:
            failures.append("hook_chop_selection_not_enough_offset_contrast")
        if proof.get("destructive_gesture_source_derived", 0.0) < 1.0:
            failures.append("destructive_gesture_not_source_derived")
        if proof.get("destructive_gesture_candidate_count", 0.0) < MIN_DESTRUCTIVE_GESTURE_CANDIDATES:
            failures.append("destructive_gesture_not_enough_candidates")
        if (
            proof.get("destructive_static_distance_frames", 0.0)
            < MIN_DESTRUCTIVE_STATIC_DISTANCE_FRAMES
        ):
            failures.append("destructive_gesture_collapsed_to_fixed_choice")
        if (
            proof.get("destructive_offset_distance_frames", 0.0)
            < MIN_DESTRUCTIVE_OFFSET_DISTANCE_FRAMES
        ):
            failures.append("destructive_gesture_not_enough_offset_contrast")
    if source_family == "sparse_bass_pressure":
        if proof.get("bass_movement_source_derived", 0.0) < 1.0:
            failures.append("sparse_bass_movement_not_source_derived")
        if (
            proof.get("sparse_bass_movement_static_distance_hz", 0.0)
            < MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ
        ):
            failures.append("sparse_bass_movement_collapsed_to_fixed_contour")
        if (
            proof.get("sparse_bass_movement_frequency_span_hz", 0.0)
            < MIN_SPARSE_BASS_MOVEMENT_SPAN_HZ
        ):
            failures.append("sparse_bass_movement_not_varied_enough")
    return failures


def validate_report_file(path: Path) -> None:
    try:
        report = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"invalid dense-break performance report: {error}") from error
    if not isinstance(report, dict):
        raise SystemExit("invalid dense-break performance report: root must be an object")
    if report.get("schema") != SCHEMA:
        raise SystemExit(f"invalid dense-break performance report: schema must be {SCHEMA}")
    metrics = report.get("metrics")
    proof = report.get("proof")
    files = report.get("files")
    source_policy = report.get("source_policy")
    if not isinstance(metrics, dict):
        raise SystemExit("invalid dense-break performance report: metrics must be an object")
    if not isinstance(proof, dict):
        raise SystemExit("invalid dense-break performance report: proof must be an object")
    if not isinstance(files, dict) or not str(files.get("rebuild_only_performance", "")).endswith(
        ".wav"
    ):
        raise SystemExit(
            "invalid dense-break performance report: rebuild_only_performance file missing"
        )
    boundary_failures = evidence_boundary_failure_codes(report)
    source_family = None
    if isinstance(source_policy, dict):
        pressure_lift_policy = source_policy.get("pressure_lift_policy")
        if isinstance(pressure_lift_policy, dict):
            source_family = str(pressure_lift_policy.get("source_family") or "")
    computed_failures = failure_codes_for(metrics, proof, source_family)
    if boundary_failures or computed_failures:
        failures = boundary_failures + computed_failures
        raise SystemExit("invalid dense-break performance report: " + ", ".join(failures))
    if report.get("result") != "pass":
        raise SystemExit("invalid dense-break performance report: result_not_pass")
    if report.get("failure_codes") != []:
        raise SystemExit("invalid dense-break performance report: stale_failure_codes")
    if report.get("human_verdict") != "unverified":
        raise SystemExit("invalid dense-break performance report: unexpected_human_verdict")
    if report.get("quality_proof") is not False:
        raise SystemExit("invalid dense-break performance report: quality_proof_not_false")


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

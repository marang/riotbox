#!/usr/bin/env python3
"""Write deterministic diverse source WAVs for Riotbox example/test packs."""

from __future__ import annotations

import argparse
import json
import math
import struct
import wave
from dataclasses import dataclass
from pathlib import Path


SAMPLE_RATE = 44_100
CHANNELS = 2
DEFAULT_SECONDS = 4.0
SCHEMA = "riotbox.diverse_test_source_wavs.v1"


@dataclass(frozen=True)
class SourceSpec:
    case_id: str
    source_family: str
    bpm: float
    expected_role: str


SOURCES = (
    SourceSpec("dense_break_132", "dense_break", 132.0, "break_transients"),
    SourceSpec("sparse_kick_snare_118", "sparse_bass_pressure", 118.0, "kick_snare_anchors"),
    SourceSpec("tonal_hook_124", "tonal_hook", 124.0, "riff_stab"),
    SourceSpec("pad_noise_96", "pad_noise", 96.0, "texture_without_drum_grid"),
    SourceSpec("sub_pressure_140", "bass_pressure", 140.0, "low_band_pressure"),
    SourceSpec("broken_timing_127", "bad_timing", 127.0, "unstable_grid_warning"),
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=Path("artifacts/audio_qa/local-diverse-test-sources"))
    parser.add_argument("--seconds", type=float, default=DEFAULT_SECONDS)
    args = parser.parse_args()
    if args.seconds <= 0:
        parser.error("--seconds must be positive")

    args.output.mkdir(parents=True, exist_ok=True)
    entries = []
    for spec in SOURCES:
        path = args.output / f"{spec.case_id}.wav"
        write_wav(path, spec, args.seconds)
        entries.append(
            {
                "case_id": spec.case_id,
                "source_family": spec.source_family,
                "bpm": spec.bpm,
                "path": str(path),
                "expected_role": spec.expected_role,
                "usage": "generated_test_fixture",
                "quality_proof": False,
            }
        )

    manifest = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass",
        "case_count": len(entries),
        "seconds": args.seconds,
        "sample_rate": SAMPLE_RATE,
        "channels": CHANNELS,
        "entries": entries,
        "notes": (
            "Deterministic synthetic source corpus for examples and regression tests. "
            "These files are source-diversity fixtures, not musical quality proof."
        ),
    }
    (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"wrote diverse test source WAVs to {args.output}")
    return 0


def write_wav(path: Path, spec: SourceSpec, seconds: float) -> None:
    frame_count = int(SAMPLE_RATE * seconds)
    frames = bytearray()
    for frame in range(frame_count):
        left, right = sample_frame(frame, spec)
        frames.extend(pcm16(left))
        frames.extend(pcm16(right))
    with wave.open(str(path), "wb") as wav:
        wav.setnchannels(CHANNELS)
        wav.setsampwidth(2)
        wav.setframerate(SAMPLE_RATE)
        wav.writeframes(frames)


def sample_frame(frame: int, spec: SourceSpec) -> tuple[float, float]:
    t = frame / SAMPLE_RATE
    beat = t * spec.bpm / 60.0
    beat_pos = beat % 1.0
    beat_index = int(beat)
    family = spec.source_family
    if family == "dense_break":
        mono = dense_break(t, beat_pos, beat_index, frame)
    elif family == "sparse_bass_pressure":
        mono = sparse_kick_snare(t, beat_pos, beat_index, frame)
    elif family == "tonal_hook":
        mono = tonal_hook(t, beat_pos, beat_index)
    elif family == "pad_noise":
        mono = pad_noise(t, beat_pos, frame)
    elif family == "bass_pressure":
        mono = sub_pressure(t, beat_pos, beat_index)
    elif family == "bad_timing":
        warped = beat + math.sin(t * math.tau * 0.41) * 0.115
        mono = dense_break(t, warped % 1.0, int(warped), frame) * 0.86
    else:
        mono = 0.0
    side = math.sin(t * math.tau * (0.18 + spec.bpm * 0.001)) * 0.035
    return clamp(mono + side), clamp(mono * 0.91 - side)


def dense_break(t: float, beat_pos: float, beat_index: int, frame: int) -> float:
    kick = env(beat_pos, 0.050) * sine(t, 52.0 + 38.0 * (1.0 - beat_pos)) * 0.80
    snare_pos = abs(beat_pos - 0.5)
    snare = env(snare_pos, 0.040) * noise(frame * 5 + 17) * (0.62 if beat_index % 2 else 0.26)
    ghost = env((beat_pos - 0.75) % 1.0, 0.018) * noise(frame * 9 + 5) * 0.22
    bass = sine(t, 82.0 if beat_index % 4 < 2 else 97.0) * 0.16
    return clamp(kick + snare + ghost + bass)


def sparse_kick_snare(t: float, beat_pos: float, beat_index: int, frame: int) -> float:
    kick = env(beat_pos, 0.070) * sine(t, 45.0 + 24.0 * (1.0 - beat_pos)) * 0.92
    snare = env(abs(beat_pos - 0.5), 0.026) * noise(frame * 11 + 31) * 0.34
    hat = env((beat_pos - 0.875) % 1.0, 0.010) * noise(frame * 3) * 0.10
    rest_gate = 1.0 if beat_index % 4 != 3 else 0.55
    return clamp((kick + snare + hat) * rest_gate)


def tonal_hook(t: float, beat_pos: float, beat_index: int) -> float:
    notes = (146.83, 174.61, 220.00, 196.00, 261.63, 220.00)
    note = notes[beat_index % len(notes)]
    gate = 0.26 + env(beat_pos, 0.180) * 0.74
    stab = sine(t, note) * 0.40 + sine(t, note * 2.01) * 0.16
    grit = math.tanh(stab * 2.3) * 0.18
    return clamp((stab + grit) * gate)


def pad_noise(t: float, beat_pos: float, frame: int) -> float:
    shimmer = sine(t, 233.08) * 0.20 + sine(t, 349.23) * 0.16 + sine(t, 523.25) * 0.10
    slow_gate = 0.62 + math.sin(t * math.tau * 0.19) * 0.28
    breath = noise(frame * 13 + 19) * 0.035
    pulse = env((beat_pos - 0.125) % 1.0, 0.030) * 0.05
    return clamp(shimmer * slow_gate + breath + pulse)


def sub_pressure(t: float, beat_pos: float, beat_index: int) -> float:
    root = 39.0 if beat_index % 8 < 4 else 46.25
    pulse = env(beat_pos, 0.160) + env((beat_pos - 0.5) % 1.0, 0.080) * 0.55
    sub = sine(t, root) * 0.82 + sine(t, root * 2.0) * 0.16
    click = env(beat_pos, 0.014) * 0.11
    return clamp(sub * pulse + click)


def env(position: float, width: float) -> float:
    if position < 0.0 or position > width:
        return 0.0
    return math.exp(-position * 32.0)


def sine(t: float, hz: float) -> float:
    return math.sin(t * math.tau * hz)


def noise(seed: int) -> float:
    value = (seed * 1_103_515_245 + 12_345) & 0x7FFF_FFFF
    return (value / 0x7FFF_FFFF) * 2.0 - 1.0


def clamp(value: float) -> float:
    return max(-0.95, min(0.95, value))


def pcm16(value: float) -> bytes:
    return struct.pack("<h", int(clamp(value) * 32_767.0))


if __name__ == "__main__":
    raise SystemExit(main())

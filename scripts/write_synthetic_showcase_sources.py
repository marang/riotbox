#!/usr/bin/env python3
"""Write deterministic synthetic source WAVs for representative Riotbox showcase QA."""

from __future__ import annotations

import json
import math
import struct
import sys
import wave
from dataclasses import dataclass
from pathlib import Path

SAMPLE_RATE = 44_100
CHANNELS = 2
DEFAULT_SECONDS = 8.0


@dataclass(frozen=True)
class SourceCase:
    case_id: str
    bpm: float
    description: str


CASES = (
    SourceCase("break_low_drive", 128.0, "dense low-drive break with bass weight"),
    SourceCase("hat_cut_pressure", 132.0, "high transient pressure with cut-up hats"),
    SourceCase("sparse_bass_pulse", 120.0, "sparse bass pulses with wide rests"),
    SourceCase("syncopated_snare", 136.0, "syncopated snare and percussion movement"),
    SourceCase("tonal_hook_chop", 124.0, "tonal hook chopped against a light pulse"),
)

WINDOWS = (
    {"id": "head", "start_seconds": 0.0, "window_seconds": 1.0},
    {"id": "late", "start_seconds": 1.5, "window_seconds": 1.0},
)


def main() -> int:
    if len(sys.argv) not in {2, 3}:
        print(
            "usage: write_synthetic_showcase_sources.py <output-dir> [seconds]",
            file=sys.stderr,
        )
        return 2

    output_dir = Path(sys.argv[1])
    seconds = float(sys.argv[2]) if len(sys.argv) == 3 else DEFAULT_SECONDS
    if seconds <= 0.0:
        print("seconds must be positive", file=sys.stderr)
        return 2

    output_dir.mkdir(parents=True, exist_ok=True)
    records = []
    for case in CASES:
        path = output_dir / f"{case.case_id}_{case.bpm:.0f}bpm.wav"
        write_wav(path, case, seconds)
        records.append(
            {
                "id": case.case_id,
                "bpm": case.bpm,
                "path": str(path),
                "description": case.description,
                "windows": list(WINDOWS),
            }
        )

    manifest_path = output_dir / "sources.json"
    manifest_path.write_text(json.dumps({"sources": records}, indent=2) + "\n")
    print(manifest_path)
    return 0


def write_wav(path: Path, case: SourceCase, seconds: float) -> None:
    frame_count = int(SAMPLE_RATE * seconds)
    frames = bytearray()
    for frame in range(frame_count):
        left, right = sample_pair(case, frame)
        frames.extend(pcm16(left))
        frames.extend(pcm16(right))

    with wave.open(str(path), "wb") as wav:
        wav.setnchannels(CHANNELS)
        wav.setsampwidth(2)
        wav.setframerate(SAMPLE_RATE)
        wav.writeframes(frames)


def sample_pair(case: SourceCase, frame: int) -> tuple[float, float]:
    t = frame / SAMPLE_RATE
    beat = t * case.bpm / 60.0
    beat_pos = beat % 1.0
    beat_index = int(beat)

    if case.case_id == "break_low_drive":
        mono = (
            kick(t, beat_pos, 68.0, 0.78)
            + snare(frame, beat_pos, beat_index, 0.34)
            + hat(frame, beat_pos, 0.16)
            + bass(t, beat_index, 86.0, 0.22)
        )
    elif case.case_id == "hat_cut_pressure":
        cut = 0.0 if beat_index % 8 in {3, 7} and beat_pos > 0.42 else 1.0
        mono = (
            hat(frame * 3 + 11, beat_pos, 0.46) * cut
            + snare(frame, (beat_pos - 0.25) % 1.0, beat_index, 0.20)
            + tone(t, 310.0 + (beat_index % 5) * 19.0, 0.07)
        )
    elif case.case_id == "sparse_bass_pulse":
        active = beat_index % 4 in {0, 3}
        mono = (
            (kick(t, beat_pos, 52.0, 0.55) + bass(t, beat_index, 58.0, 0.42))
            if active
            else tone(t, 116.0, 0.035)
        )
    elif case.case_id == "syncopated_snare":
        mono = (
            kick(t, (beat_pos - 0.12) % 1.0, 74.0, 0.62)
            + snare(frame * 5, (beat_pos - 0.37) % 1.0, beat_index, 1.35)
            + hat(frame, (beat_pos - 0.67) % 1.0, 0.50)
            + tone(t, 185.0 + (beat_index % 3) * 14.0, 0.18)
        )
    else:
        hook_note = (0, 3, 7, 10)[beat_index % 4]
        mono = (
            tone(t, midi_frequency(43 + hook_note), 0.42)
            * (0.45 + 0.55 * envelope(beat_pos, 0.32))
            + kick(t, beat_pos, 62.0, 0.32)
            + hat(frame, (beat_pos - 0.5) % 1.0, 0.12)
        )

    mono = clamp(mono)
    width = 0.88 + 0.08 * math.sin(2.0 * math.pi * 0.17 * t)
    return clamp(mono * 0.96), clamp(mono * width)


def kick(t: float, beat_pos: float, hz: float, level: float) -> float:
    return envelope(beat_pos, 0.060) * math.sin(2.0 * math.pi * hz * t) * level


def snare(frame: int, beat_pos: float, beat_index: int, level: float) -> float:
    accent = 1.0 if beat_index % 2 else 0.62
    return envelope(abs(beat_pos - 0.5), 0.035) * noise(frame * 7 + 19) * level * accent


def hat(frame: int, beat_pos: float, level: float) -> float:
    return envelope((beat_pos - 0.5) % 1.0, 0.020) * noise(frame * 13 + 5) * level


def bass(t: float, beat_index: int, base_hz: float, level: float) -> float:
    hz = base_hz * (1.0 if beat_index % 4 < 2 else 1.125)
    return math.sin(2.0 * math.pi * hz * t) * level


def tone(t: float, hz: float, level: float) -> float:
    return math.sin(2.0 * math.pi * hz * t) * level


def envelope(position: float, width: float) -> float:
    if position < 0.0 or position > width:
        return 0.0
    return math.exp(-position * 36.0)


def midi_frequency(note: int) -> float:
    return 440.0 * 2.0 ** ((note - 69) / 12.0)


def noise(seed: int) -> float:
    value = (seed * 1_103_515_245 + 12_345) & 0x7FFF_FFFF
    return (value / 0x7FFF_FFFF) * 2.0 - 1.0


def clamp(value: float) -> float:
    return max(-0.95, min(0.95, value))


def pcm16(value: float) -> bytes:
    return struct.pack("<h", int(clamp(value) * 32_767.0))


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Write a deterministic stereo PCM16 break-like WAV for CI audio QA."""

from __future__ import annotations

import math
import struct
import sys
import wave
from pathlib import Path

SAMPLE_RATE = 44_100
CHANNELS = 2
DEFAULT_SECONDS = 4.0
DEFAULT_BPM = 128.0


def main() -> int:
    if len(sys.argv) not in {2, 3}:
        print("usage: write_synthetic_break_wav.py <output.wav> [seconds]", file=sys.stderr)
        return 2

    output_path = Path(sys.argv[1])
    seconds = float(sys.argv[2]) if len(sys.argv) == 3 else DEFAULT_SECONDS
    if seconds <= 0.0:
        print("seconds must be positive", file=sys.stderr)
        return 2

    output_path.parent.mkdir(parents=True, exist_ok=True)
    write_wav(output_path, seconds)
    print(f"wrote {output_path}")
    return 0


def write_wav(path: Path, seconds: float) -> None:
    frame_count = int(SAMPLE_RATE * seconds)
    frames = bytearray()
    for frame in range(frame_count):
        mono = sample_at(frame)
        frames.extend(pcm16(mono * 0.96))
        frames.extend(pcm16(mono * 0.86))

    with wave.open(str(path), "wb") as wav:
        wav.setnchannels(CHANNELS)
        wav.setsampwidth(2)
        wav.setframerate(SAMPLE_RATE)
        wav.writeframes(frames)


def sample_at(frame: int) -> float:
    t = frame / SAMPLE_RATE
    beat = t * DEFAULT_BPM / 60.0
    beat_pos = beat % 1.0
    beat_index = int(beat)

    kick = envelope(beat_pos, 0.045) * math.sin(
        2.0 * math.pi * (55.0 + 35.0 * (1.0 - beat_pos)) * t
    )
    hat = envelope((beat_pos - 0.5) % 1.0, 0.018) * noise(frame) * 0.28
    snare_pos = abs(beat_pos - 0.5)
    snare = envelope(snare_pos, 0.030) * noise(frame * 7 + 13) * (
        0.40 if beat_index % 2 == 1 else 0.18
    )
    bass = 0.18 * math.sin(
        2.0 * math.pi * (82.0 if beat_index % 4 < 2 else 98.0) * t
    )
    return clamp(kick * 0.68 + snare + hat + bass)


def envelope(position: float, width: float) -> float:
    if position < 0.0 or position > width:
        return 0.0
    return math.exp(-position * 36.0)


def noise(seed: int) -> float:
    value = (seed * 1_103_515_245 + 12_345) & 0x7FFF_FFFF
    return (value / 0x7FFF_FFFF) * 2.0 - 1.0


def clamp(value: float) -> float:
    return max(-0.95, min(0.95, value))


def pcm16(value: float) -> bytes:
    return struct.pack("<h", int(clamp(value) * 32_767.0))


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Extract a PCM WAV source window for local listening-pack comparison."""

from __future__ import annotations

import sys
import wave
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 5:
        print(
            "usage: extract_wav_window.py <source.wav> <output.wav> <start-seconds> <duration-seconds>",
            file=sys.stderr,
        )
        return 2

    source = Path(sys.argv[1])
    output = Path(sys.argv[2])
    start_seconds = float(sys.argv[3])
    duration_seconds = float(sys.argv[4])
    if start_seconds < 0.0 or duration_seconds <= 0.0:
        print("start must be non-negative and duration must be positive", file=sys.stderr)
        return 2

    extract(source, output, start_seconds, duration_seconds)
    print(f"wrote {output}")
    return 0


def extract(source: Path, output: Path, start_seconds: float, duration_seconds: float) -> None:
    with wave.open(str(source), "rb") as wav:
        channels = wav.getnchannels()
        sample_width = wav.getsampwidth()
        sample_rate = wav.getframerate()
        start_frame = int(start_seconds * sample_rate)
        frame_count = int(duration_seconds * sample_rate)
        start_frame = min(start_frame, wav.getnframes())
        wav.setpos(start_frame)
        frames = wav.readframes(frame_count)

    output.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(output), "wb") as wav:
        wav.setnchannels(channels)
        wav.setsampwidth(sample_width)
        wav.setframerate(sample_rate)
        wav.writeframes(frames)


if __name__ == "__main__":
    raise SystemExit(main())

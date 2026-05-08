#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

source_wav="$tmpdir/flat-pulse.wav"
summary_json="$tmpdir/source-timing-probe-ambiguous.json"

python3 - "$source_wav" <<'PY'
from __future__ import annotations

import math
import struct
import sys
import wave
from pathlib import Path

path = Path(sys.argv[1])
sample_rate = 44_100
seconds = 16.0
bpm = 128.0
frames = bytearray()

for frame in range(int(sample_rate * seconds)):
    t = frame / sample_rate
    beat_position = (t * bpm / 60.0) % 1.0
    envelope = math.exp(-beat_position * 70.0) if beat_position < 0.05 else 0.0
    sample = envelope * math.sin(2.0 * math.pi * 120.0 * t) * 0.80
    packed = struct.pack("<h", int(max(-0.95, min(0.95, sample)) * 32_767.0))
    frames.extend(packed)
    frames.extend(packed)

with wave.open(str(path), "wb") as wav:
    wav.setnchannels(2)
    wav.setsampwidth(2)
    wav.setframerate(sample_rate)
    wav.writeframes(frames)

print(f"wrote {path}")
PY

cargo run -p riotbox-audio --bin source_timing_probe -- --json "$source_wav" > "$summary_json"
python3 scripts/validate_source_timing_probe_json.py "$summary_json"

python3 - "$summary_json" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path

summary = json.loads(Path(sys.argv[1]).read_text())

expected = {
    "cue": "needs confirm",
    "readiness": "weak",
    "requires_manual_confirm": True,
    "beat_status": "stable",
    "downbeat_status": "weak",
    "phrase_status": "ambiguous_downbeat",
    "confidence_result": "candidate_ambiguous",
    "drift_status": "stable",
}
for key, value in expected.items():
    actual = summary.get(key)
    if actual != value:
        raise SystemExit(f"{key} must be {value!r}, got {actual!r}")

checks = {
    "primary_bpm": lambda value: 127.0 <= value <= 130.0,
    "primary_beat_score": lambda value: value >= 0.90,
    "primary_downbeat_score": lambda value: value < 0.30,
    "alternate_downbeat_phase_count": lambda value: value >= 1,
    "onset_count": lambda value: value > 0,
}
for key, predicate in checks.items():
    value = summary.get(key)
    if not isinstance(value, (int, float)) or isinstance(value, bool) or not predicate(value):
        raise SystemExit(f"{key} failed generated ambiguous source timing smoke: {value!r}")

warnings = set(summary.get("warning_codes", []))
required_warnings = {"phrase_uncertain", "ambiguous_downbeat"}
missing = sorted(required_warnings - warnings)
if missing:
    raise SystemExit(f"missing expected warning codes: {missing}")

print(
    "generated ambiguous source timing probe ok: "
    f"bpm={summary['primary_bpm']:.3f} "
    f"beat={summary['primary_beat_score']:.3f} "
    f"downbeat={summary['primary_downbeat_score']:.3f} "
    f"alternates={summary['alternate_downbeat_phase_count']}"
)
PY

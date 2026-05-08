#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

source_wav="$tmpdir/silence.wav"
summary_json="$tmpdir/source-timing-probe-degraded.json"

python3 - "$source_wav" <<'PY'
from __future__ import annotations

import struct
import sys
import wave
from pathlib import Path

path = Path(sys.argv[1])
sample_rate = 44_100
seconds = 4.0
channels = 2

with wave.open(str(path), "wb") as wav:
    wav.setnchannels(channels)
    wav.setsampwidth(2)
    wav.setframerate(sample_rate)
    wav.writeframes(struct.pack("<h", 0) * channels * int(sample_rate * seconds))

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
    "readiness": "unavailable",
    "requires_manual_confirm": True,
    "primary_bpm": None,
    "beat_status": "unavailable",
    "downbeat_status": "unavailable",
    "phrase_status": "unavailable",
    "confidence_result": "degraded",
    "onset_count": 0,
}
for key, value in expected.items():
    actual = summary.get(key)
    if actual != value:
        raise SystemExit(f"{key} must be {value!r}, got {actual!r}")

warnings = set(summary.get("warning_codes", []))
required_warnings = {"low_timing_confidence", "weak_kick_anchor"}
missing = sorted(required_warnings - warnings)
if missing:
    raise SystemExit(f"missing expected warning codes: {missing}")

for key in ("primary_beat_score", "primary_downbeat_score"):
    if summary.get(key) is not None:
        raise SystemExit(f"{key} must stay null for degraded silence, got {summary.get(key)!r}")

anchors = summary.get("anchor_evidence", {})
for key in (
    "primary_anchor_count",
    "primary_kick_anchor_count",
    "primary_backbeat_anchor_count",
    "primary_transient_anchor_count",
):
    if anchors.get(key) != 0:
        raise SystemExit(f"anchor_evidence.{key} must be 0 for degraded silence, got {anchors.get(key)!r}")
if anchors.get("primary_anchor_preview") != []:
    raise SystemExit(
        "anchor_evidence.primary_anchor_preview must stay empty for degraded silence"
    )

print(
    "generated degraded source timing probe ok: "
    f"cue={summary['cue']} warnings={','.join(sorted(warnings))}"
)
PY

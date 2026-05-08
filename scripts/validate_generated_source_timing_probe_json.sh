#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

source_wav="$tmpdir/source.wav"
summary_json="$tmpdir/source-timing-probe.json"

python3 scripts/write_synthetic_break_wav.py "$source_wav" 16.0
cargo run -p riotbox-audio --bin source_timing_probe -- --json "$source_wav" > "$summary_json"
python3 scripts/validate_source_timing_probe_json.py "$summary_json"

python3 - "$summary_json" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path

summary = json.loads(Path(sys.argv[1]).read_text())

expected = {
    "cue": "grid locked",
    "readiness": "ready",
    "requires_manual_confirm": False,
    "beat_status": "stable",
    "downbeat_status": "stable",
    "phrase_status": "stable",
}
for key, value in expected.items():
    actual = summary.get(key)
    if actual != value:
        raise SystemExit(f"{key} must be {value!r}, got {actual!r}")

checks = {
    "primary_bpm": lambda value: 127.0 <= value <= 130.0,
    "primary_beat_score": lambda value: value >= 0.90,
    "primary_downbeat_score": lambda value: value >= 0.30,
    "duration_seconds": lambda value: value >= 15.9,
    "onset_count": lambda value: value > 0,
}
for key, predicate in checks.items():
    value = summary.get(key)
    if not isinstance(value, (int, float)) or isinstance(value, bool) or not predicate(value):
        raise SystemExit(f"{key} failed generated source timing smoke: {value!r}")

if summary.get("warning_codes") != []:
    raise SystemExit(
        f"warning_codes must be empty for generated locked source: {summary.get('warning_codes')!r}"
    )

anchors = summary.get("anchor_evidence", {})
anchor_checks = {
    "primary_anchor_count": lambda value: value > 0,
    "primary_kick_anchor_count": lambda value: value > 0,
    "primary_backbeat_anchor_count": lambda value: value > 0,
    "primary_transient_anchor_count": lambda value: value >= 0,
}
for key, predicate in anchor_checks.items():
    value = anchors.get(key)
    if not isinstance(value, int) or isinstance(value, bool) or not predicate(value):
        raise SystemExit(f"anchor_evidence.{key} failed generated source timing smoke: {value!r}")

preview = anchors.get("primary_anchor_preview", [])
if not preview or not any(anchor.get("anchor_type") == "kick" for anchor in preview):
    raise SystemExit("anchor_evidence.primary_anchor_preview must include a kick anchor")
if not any(anchor.get("anchor_type") == "backbeat" for anchor in preview):
    raise SystemExit("anchor_evidence.primary_anchor_preview must include a backbeat anchor")

groove = summary.get("groove_evidence", {})
groove_checks = {
    "primary_groove_residual_count": lambda value: value >= 0,
    "primary_max_abs_offset_ms": lambda value: value >= 0.0,
}
for key, predicate in groove_checks.items():
    value = groove.get(key)
    if not isinstance(value, (int, float)) or isinstance(value, bool) or not predicate(value):
        raise SystemExit(f"groove_evidence.{key} failed generated source timing smoke: {value!r}")

groove_preview = groove.get("primary_groove_preview", [])
if not isinstance(groove_preview, list):
    raise SystemExit("groove_evidence.primary_groove_preview must be a list")
for residual in groove_preview:
    if residual.get("subdivision") not in {"eighth", "triplet", "sixteenth", "thirty_second"}:
        raise SystemExit(f"unexpected groove subdivision: {residual!r}")
    confidence = residual.get("confidence")
    if not isinstance(confidence, (int, float)) or isinstance(confidence, bool) or not 0 <= confidence <= 1:
        raise SystemExit(f"invalid groove confidence: {residual!r}")

print(
    "generated source timing probe ok: "
    f"bpm={summary['primary_bpm']:.3f} "
    f"beat={summary['primary_beat_score']:.3f} "
    f"downbeat={summary['primary_downbeat_score']:.3f} "
    f"anchors={anchors['primary_anchor_count']} "
    f"groove_residuals={groove['primary_groove_residual_count']}"
)
PY

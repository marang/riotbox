#!/usr/bin/env bash
set -euo pipefail

source_path="data/test_audio/examples/Beat03_130BPM(Full).wav"
date="${1:-local-beat03-feral-grid-auto-proof}"
output_dir="artifacts/audio_qa/${date}/feral-grid-demo"
manifest_path="${output_dir}/manifest.json"

if [[ ! -f "${source_path}" ]]; then
  echo "skip: ${source_path} is not present in this checkout"
  exit 0
fi

cargo run -p riotbox-audio --bin feral_grid_pack -- \
  --source "${source_path}" \
  --date "${date}" \
  --bars 8 \
  --source-window-seconds 1.0 \
  --source-start-seconds 0.0

python3 - "${manifest_path}" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
with manifest_path.open() as fh:
    manifest = json.load(fh)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"{manifest_path}: {message}")


source_timing = manifest["source_timing"]
metrics = manifest["metrics"]

require(
    manifest["grid_bpm_source"] == "source_timing",
    "grid_bpm_source is not source_timing",
)
require(
    manifest["grid_bpm_decision_reason"] == "source_timing_needs_review_manual_confirm",
    "grid_bpm_decision_reason is not source_timing_needs_review_manual_confirm",
)
require(manifest["source_timing_bpm_delta"] == 0.0, "source timing BPM delta is not zero")
require(source_timing["readiness"] == "needs_review", "readiness is not needs_review")
require(source_timing["requires_manual_confirm"] is True, "manual confirmation is not required")
require(
    source_timing["grid_use"] == "short_loop_manual_confirm",
    "grid_use is not short_loop_manual_confirm",
)
require(
    129.5 <= source_timing["primary_bpm"] <= 131.0,
    "primary BPM is outside Beat03 range",
)
require(source_timing["beat_status"] == "stable", "beat evidence is not stable")
require(source_timing["downbeat_status"] == "stable", "downbeat evidence is not stable")
require(source_timing["primary_downbeat_offset_beats"] == 2, "unexpected Beat03 downbeat offset")
require(source_timing["confidence_result"] == "candidate_cautious", "confidence is not candidate_cautious")
require(source_timing["alternate_evidence_count"] == 0, "unexpected alternate evidence")
require(source_timing["warning_codes"] == ["PhraseUncertain"], "unexpected warning codes")

for metric_name, minimum in (
    ("tr909_source_grid_alignment", 0.5),
    ("mc202_source_grid_alignment", 0.5),
    ("w30_source_grid_alignment", 0.5),
    ("source_grid_output_drift", 0.5),
):
    metric = metrics[metric_name]
    require(metric["hit_ratio"] >= minimum, f"{metric_name}.hit_ratio below {minimum}")

print(
    "ok: Beat03 auto Feral grid uses Source Timing "
    f"bpm={source_timing['primary_bpm']:.3f} "
    f"tr909={metrics['tr909_source_grid_alignment']['hit_ratio']:.3f} "
    f"mc202={metrics['mc202_source_grid_alignment']['hit_ratio']:.3f} "
    f"w30={metrics['w30_source_grid_alignment']['hit_ratio']:.3f} "
    f"mix={metrics['source_grid_output_drift']['hit_ratio']:.3f}"
)
PY

#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

sources_manifest="$(python3 scripts/write_synthetic_showcase_sources.py "$tmpdir/sources" 3.0)"
source_path="$(python3 - "$sources_manifest" <<'PY'
import json
import sys
from pathlib import Path

manifest = json.loads(Path(sys.argv[1]).read_text())
for source in manifest["sources"]:
    if source["id"] == "syncopated_snare":
        print(source["path"])
        break
else:
    raise SystemExit("syncopated_snare source missing from synthetic showcase manifest")
PY
)"

cargo run -p riotbox-audio --bin feral_grid_pack -- \
  --source "$source_path" \
  --output-dir "$tmpdir/pack" \
  --date local-syncopated-source-showcase-smoke \
  --bpm 136.0 \
  --bars 4 \
  --source-window-seconds 1.0 \
  --source-start-seconds 0.0

python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$tmpdir/pack/manifest.json"

jq -e \
  '.result == "pass"
    and .grid_bpm_source == "user_override"
    and .grid_bpm_decision_reason == "user_override"
    and .bpm == 136.0
    and (.source_timing.primary_bpm | type == "number")
    and .source_timing.beat_status == "stable"
    and .source_timing.downbeat_status == "stable"
    and .source_timing.anchor_evidence.primary_anchor_count > 0
    and .source_timing_bpm_delta > 0.0
    and .metrics.source_grid_output_drift.hit_ratio >= 0.75
    and .metrics.source_grid_output_drift.max_peak_offset_ms <= .metrics.source_grid_output_drift.max_allowed_peak_offset_ms
    and .metrics.tr909_source_grid_alignment.hit_ratio >= 0.95
    and .metrics.tr909_source_grid_alignment.max_peak_offset_ms <= .metrics.tr909_source_grid_alignment.max_allowed_peak_offset_ms
    and .metrics.tr909_kick_pressure.applied == true
    and .metrics.tr909_kick_pressure.low_band_rms_ratio >= 1.06
    and .metrics.mc202_bass_pressure.applied == true
    and .metrics.mc202_bass_pressure.signal_rms >= 0.003
    and .metrics.mc202_bass_pressure.low_band_rms >= 0.001
    and .metrics.w30_source_grid_alignment.hit_ratio >= 0.50
    and .metrics.w30_source_grid_alignment.max_peak_offset_ms <= .metrics.w30_source_grid_alignment.max_allowed_peak_offset_ms
    and .metrics.w30_source_loop_closure.passed == true
    and .metrics.w30_source_trigger_variation.applied == true
    and .metrics.w30_source_trigger_variation.offbeat_trigger_count > 0
    and .metrics.w30_source_trigger_variation.distinct_bar_pattern_count >= 2
    and .metrics.w30_source_slice_choice.applied == true
    and .metrics.w30_source_slice_choice.unique_source_offset_count >= 4
    and .metrics.full_grid_mix.signal.rms > 0.000001' \
  "$tmpdir/pack/manifest.json"

#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

bars="${RIOTBOX_FULL_GRID_EXPORT_BARS:-4}"
if ! [[ "$bars" =~ ^[0-9]+$ ]] || (( bars < 2 )); then
  echo "RIOTBOX_FULL_GRID_EXPORT_BARS must be an integer >= 2" >&2
  exit 1
fi

source_seconds="${RIOTBOX_FULL_GRID_EXPORT_SOURCE_SECONDS:-8.0}"
source_window_seconds="${RIOTBOX_FULL_GRID_EXPORT_SOURCE_WINDOW_SECONDS:-1.0}"
python3 - "$source_seconds" "$source_window_seconds" <<'PY'
import math
import sys

try:
    source_seconds = float(sys.argv[1])
    source_window_seconds = float(sys.argv[2])
except ValueError:
    raise SystemExit("full-grid export durations must be finite numbers")
if not math.isfinite(source_seconds) or source_seconds <= 0.0:
    raise SystemExit("RIOTBOX_FULL_GRID_EXPORT_SOURCE_SECONDS must be a positive finite number")
if not math.isfinite(source_window_seconds) or source_window_seconds <= 0.0:
    raise SystemExit("RIOTBOX_FULL_GRID_EXPORT_SOURCE_WINDOW_SECONDS must be a positive finite number")
if source_window_seconds > source_seconds:
    raise SystemExit("RIOTBOX_FULL_GRID_EXPORT_SOURCE_WINDOW_SECONDS must not exceed source seconds")
PY

tmpdir="$(mktemp -d)"
handoff_stage=""
cleanup() {
  rm -rf "$tmpdir"
  if [[ -n "$handoff_stage" ]]; then
    rm -rf "$handoff_stage"
  fi
}
trap cleanup EXIT

source_a="$tmpdir/source-a.wav"
source_b="$tmpdir/source-b.wav"
run_a="$tmpdir/run-a"
run_b="$tmpdir/run-b"
proof="$tmpdir/product-export-proof.json"

source_override="${RIOTBOX_PRODUCT_EXPORT_SOURCE:-}"
handoff_dir="${RIOTBOX_PRODUCT_EXPORT_HANDOFF_DIR:-}"
stem_handoff_dir="${RIOTBOX_PRODUCT_STEM_HANDOFF_DIR:-}"
if [[ -n "$handoff_dir" && -n "$stem_handoff_dir" ]]; then
  echo "product-mix and product-stem handoff destinations are mutually exclusive" >&2
  exit 1
fi
requested_handoff_dir="${handoff_dir:-$stem_handoff_dir}"
if [[ -n "$source_override" || -n "$requested_handoff_dir" ]]; then
  if [[ -z "$source_override" || -z "$requested_handoff_dir" ]]; then
    echo "RIOTBOX_PRODUCT_EXPORT_SOURCE and exactly one handoff destination must be supplied together" >&2
    exit 1
  fi
  if [[ ! -f "$source_override" ]]; then
    echo "product export source is not a file: $source_override" >&2
    exit 1
  fi
  if [[ -e "$requested_handoff_dir" ]]; then
    echo "product export handoff destination already exists: $requested_handoff_dir" >&2
    exit 1
  fi
  source_a="$source_override"
  source_b="$source_override"
else
  python3 scripts/write_synthetic_break_wav.py "$source_a" "$source_seconds"
  python3 scripts/write_synthetic_break_wav.py "$source_b" "$source_seconds"
fi

source_hash_a="$(sha256sum "$source_a" | awk '{print $1}')"
source_hash_b="$(sha256sum "$source_b" | awk '{print $1}')"
if [[ "$source_hash_a" != "$source_hash_b" ]]; then
  echo "deterministic full-grid source generation drifted: $source_hash_a != $source_hash_b" >&2
  exit 1
fi

render_grid_export() {
  local source_wav="$1"
  local run_dir="$2"
  cargo run -p riotbox-audio --bin feral_grid_pack -- \
    --source "$source_wav" \
    --output-dir "$run_dir" \
    --bars "$bars" \
    --source-window-seconds "$source_window_seconds"
  python3 scripts/validate_listening_manifest_json.py \
    --require-existing-artifacts \
    "$run_dir/manifest.json"
  jq -e \
    '.pack_id == "feral-grid-demo"
      and .result == "pass"
      and .feral_scorecard.readiness == "ready"
      and .feral_scorecard.source_backed == true
      and .feral_scorecard.fallback_like == false
      and (has("primitive_renderer_boundary") | not)
      and (.feral_scorecard.lane_gestures | index("mc202 question/answer")) == null
      and .metrics.full_grid_mix.signal.rms > 0.000001
      and .metrics.full_grid_mix.low_band.rms > 0.000001
      and .metrics.tr909_beat_fill.signal.rms > 0.000001
      and .metrics.tr909_kick_pressure.pattern_origin == "source_derived"
      and .metrics.tr909_kick_pressure.source_evidence_role == "tr909_source_profile_and_accent_dynamics"
      and (.metrics.tr909_kick_pressure.source_profile_reason | startswith("source_"))
      and .metrics.tr909_kick_pressure.applied == true
      and .metrics.tr909_kick_pressure.anchor_count >= 2
      and .metrics.tr909_kick_pressure.low_band_rms_ratio >= 1.06
      and .metrics.tr909_source_accent_dynamics.pattern_origin == "source_derived"
      and .metrics.tr909_source_accent_dynamics.applied == true
      and .metrics.tr909_source_accent_dynamics.anchor_count >= 2
      and .metrics.tr909_source_accent_dynamics.distinct_accent_count >= 3
      and .metrics.tr909_source_accent_dynamics.accent_span >= .metrics.tr909_source_accent_dynamics.min_required_accent_span
      and .metrics.w30_source_trigger_variation.pattern_origin == "source_derived"
      and .metrics.mc202_bass_pressure.pattern_origin == "source_derived"
      and .metrics.mc202_bass_pressure.applied == true
      and .metrics.mc202_bass_pressure.source_expression_render_plan_applied == true
      and (.metrics.mc202_bass_pressure.source_expression_role | IN("bass_pressure", "answer_lift", "hook_restraint_hold"))
      and .metrics.mc202_bass_pressure.source_failure_fallback == false
      and .metrics.mc202_bass_pressure.reason == "mc202_source_grid_proof_renderer"
      and .metrics.mc202_source_contour.pattern_origin == "source_derived_contour"
      and .metrics.mc202_source_contour.applied == true
      and .metrics.mc202_source_contour.source_contour_delta_rms >= .metrics.mc202_source_contour.min_required_delta_rms
      and .metrics.mc202_source_grid_alignment.hit_ratio >= 0.50
      and .metrics.mc202_bass_pressure_stem.signal.rms > 0.000001
      and .metrics.w30_feral_source_chop.signal.rms > 0.000001
      and (.metrics | has("mc202_question_answer_delta") | not)' \
    "$run_dir/manifest.json"
}

render_grid_export "$source_a" "$run_a"
render_grid_export "$source_b" "$run_b"

python3 scripts/validate_product_export_reproducibility.py \
  --write-proof "$proof" \
  "$run_a/manifest.json" \
  "$run_b/manifest.json"

publish_handoff_dir="$handoff_dir"
report_handoff=false
if [[ -n "$stem_handoff_dir" ]]; then
  stem_proof="$tmpdir/product-stem-handoff-proof.json"
  python3 scripts/validate_product_stem_handoff.py build \
    --write-proof "$stem_proof" \
    "$run_a/manifest.json" \
    "$run_b/manifest.json"

  handoff_parent="$(dirname "$stem_handoff_dir")"
  mkdir -p "$handoff_parent"
  handoff_stage="$(mktemp -d "$handoff_parent/.riotbox-product-stem-handoff.XXXXXX")"
  python3 scripts/validate_product_stem_handoff.py stage \
    --proof "$stem_proof" \
    --manifest "$run_a/manifest.json" \
    --destination "$handoff_stage"
  if ! mv -Tn -- "$handoff_stage" "$stem_handoff_dir" || [[ -e "$handoff_stage" ]]; then
    echo "product stem handoff destination appeared before atomic publish: $stem_handoff_dir" >&2
    exit 1
  fi
  handoff_stage=""
  python3 scripts/validate_product_stem_handoff.py validate \
    "$stem_handoff_dir/product_stem_handoff_proof.json"
  echo "product stem handoff ready: $stem_handoff_dir"
  echo "proof: $stem_handoff_dir/product_stem_handoff_proof.json"
  exit 0
elif [[ -z "$publish_handoff_dir" ]]; then
  publish_handoff_dir="$tmpdir/handoff-smoke"
else
  report_handoff=true
fi

artifact_rel="$(python3 - "$proof" <<'PY'
import json
import sys
from pathlib import Path, PurePosixPath

value = json.loads(Path(sys.argv[1]).read_text()).get("export_artifact")
if not isinstance(value, str) or not value.strip():
    raise SystemExit("product export proof has no export_artifact")
path = PurePosixPath(value)
if path.is_absolute() or ".." in path.parts:
    raise SystemExit(f"product export artifact must be a contained relative path: {value}")
print(path.as_posix())
PY
)"
artifact_source="$run_a/$artifact_rel"
if [[ ! -f "$artifact_source" ]]; then
  echo "validated product export artifact is missing: $artifact_source" >&2
  exit 1
fi

handoff_parent="$(dirname "$publish_handoff_dir")"
mkdir -p "$handoff_parent"
handoff_stage="$(mktemp -d "$handoff_parent/.riotbox-product-export-handoff.XXXXXX")"
mkdir -p "$handoff_stage/$(dirname "$artifact_rel")"
cp "$artifact_source" "$handoff_stage/$artifact_rel"
cp "$proof" "$handoff_stage/product_export_proof.json"
if ! mv -Tn -- "$handoff_stage" "$publish_handoff_dir" || [[ -e "$handoff_stage" ]]; then
  echo "product export handoff destination appeared before atomic publish: $publish_handoff_dir" >&2
  exit 1
fi
handoff_stage=""

published_hash="$(sha256sum "$publish_handoff_dir/$artifact_rel" | awk '{print $1}')"
expected_hash="$(jq -r '.export_sha256' "$publish_handoff_dir/product_export_proof.json")"
if [[ "$published_hash" != "$expected_hash" ]]; then
  echo "published product export handoff hash mismatch: $published_hash != $expected_hash" >&2
  exit 1
fi

if [[ "$report_handoff" == true ]]; then
  echo "product export handoff ready: $publish_handoff_dir"
  echo "proof: $publish_handoff_dir/product_export_proof.json"
fi

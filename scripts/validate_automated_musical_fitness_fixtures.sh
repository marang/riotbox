#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VALIDATOR="$ROOT/scripts/validate_automated_musical_fitness.py"
FIXTURES="$ROOT/scripts/fixtures/automated_musical_fitness"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

python3 "$VALIDATOR" \
  --json-output "$TMP/valid.json" \
  "$FIXTURES/valid" >/dev/null
jq -e \
  '.schema == "riotbox.automated_musical_fitness.v1"
   and .technical_status == "pass"
   and .automated_musical_fitness_status == "pass"
   and .result == "pass"
   and .human_verdict == "unverified"
   and (.failure_codes | length == 0)' \
  "$TMP/valid.json" >/dev/null

python3 "$VALIDATOR" \
  "$ROOT/scripts/fixtures/representative_showcase_musical_quality/valid" >/dev/null

python3 "$VALIDATOR" "$FIXTURES/valid_break_low_drive" >/dev/null
python3 "$VALIDATOR" "$FIXTURES/valid_tonal_hook_chop" >/dev/null
python3 "$VALIDATOR" "$FIXTURES/valid_sparse_bass_pulse" >/dev/null

expect_failure() {
  local fixture="$1"
  local code="$2"
  local out="$TMP/${fixture}.out"
  if python3 "$VALIDATOR" "$FIXTURES/$fixture" >"$out" 2>&1; then
    cat "$out" >&2
    echo "expected automated musical fitness fixture to fail: $fixture" >&2
    exit 1
  fi
  grep -q "$code" "$out"
}

expect_failure invalid_static movement_bar_similarity_too_static
expect_failure invalid_source_masked source_first_generated_support_masks_source
expect_failure invalid_weak_low_transient low_end_too_weak
expect_failure invalid_fallback_collapse fallback_collapse
expect_failure invalid_source_fake source_relation_not_source_derived
expect_failure invalid_grid_drift grid_drift_alignment_too_weak
expect_failure invalid_identical_response_across_sources cross_source_identical_response

mkdir -p "$TMP/tr909_missing_source_evidence"
jq 'del(.metrics.tr909_kick_pressure.pattern_origin)
    | del(.metrics.tr909_kick_pressure.source_evidence_role)
    | del(.metrics.tr909_kick_pressure.source_profile_reason)
    | del(.metrics.tr909_source_accent_dynamics)' \
  "$FIXTURES/valid/manifest.json" >"$TMP/tr909_missing_source_evidence/manifest.json"
if python3 "$VALIDATOR" "$TMP/tr909_missing_source_evidence" \
  >"$TMP/tr909_missing_source_evidence.out" 2>&1; then
  cat "$TMP/tr909_missing_source_evidence.out" >&2
  echo "expected TR-909 source-evidence mutation to fail" >&2
  exit 1
fi
grep -q low_end_tr909_kick_pressure_missing_source_evidence \
  "$TMP/tr909_missing_source_evidence.out"
grep -q low_end_tr909_accent_dynamics_missing "$TMP/tr909_missing_source_evidence.out"

if python3 "$VALIDATOR" \
  "$ROOT/scripts/fixtures/representative_showcase_musical_quality/invalid_static" \
  >"$TMP/representative_invalid_static.out" 2>&1; then
  cat "$TMP/representative_invalid_static.out" >&2
  echo "expected representative invalid static fixture to fail" >&2
  exit 1
fi
grep -q movement_w30_trigger_variation_missing "$TMP/representative_invalid_static.out"

echo "automated musical fitness fixture gate ok"

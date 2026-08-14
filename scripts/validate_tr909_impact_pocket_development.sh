#!/usr/bin/env bash
set -euo pipefail

contract="docs/benchmarks/tr909_impact_pocket_development_v1.json"
output="${1:-artifacts/audio_qa/local-tr909-impact-pocket-development}"
access_log="$output/development-access-log.json"

if [[ -e "$output" ]]; then
  echo "output already exists: $output" >&2
  exit 1
fi

jq -e '
  .schema == "riotbox.tr909_impact_pocket_development.v1"
  and .ticket == "RIOTBOX-1436"
  and .decision == "RBX-287"
  and .partition == "development"
  and .access.directory_discovery_allowed == false
  and .access.holdout_audio_allowed == false
  and .access.commercial_reference_audio_allowed == false
  and (.cases | length) == 3
  and ([.cases[].case_id] | unique | length) == 3
  and ([.cases[].source_path] | unique | length) == 3
' "$contract" >/dev/null

mkdir -p "$output"
contract_sha256="$(sha256sum "$contract" | awk '{print $1}')"
jq --arg contract_sha256 "$contract_sha256" '{
  schema: "riotbox.tr909_impact_pocket_development_access.v1",
  ticket,
  decision,
  contract_sha256: $contract_sha256,
  status: "started",
  directory_discovery_performed: false,
  holdout_audio_opened: false,
  commercial_reference_audio_opened: false,
  files: [.cases[] | {
    case_id,
    source_family,
    source_path,
    status: "not_opened",
    content_hash: null
  }]
}' "$contract" >"$access_log"

mark_status() {
  local case_id="$1"
  local status="$2"
  local content_hash="${3:-}"
  local tmp="$output/.access-log.tmp"
  jq --arg case_id "$case_id" --arg status "$status" --arg content_hash "$content_hash" '
    .files |= map(
      if .case_id == $case_id then
        .status = $status
        | if $content_hash == "" then . else .content_hash = $content_hash end
      else . end
    )
  ' "$access_log" >"$tmp"
  mv "$tmp" "$access_log"
}

mark_session_failed() {
  local exit_code=$?
  mark_session_failed_now
  exit "$exit_code"
}

mark_session_failed_now() {
  if [[ -f "$access_log" ]]; then
    local tmp="$output/.access-log.tmp"
    jq '.status = "failed_closed"' "$access_log" >"$tmp" && mv "$tmp" "$access_log"
  fi
}
trap mark_session_failed ERR

while IFS=$'\t' read -r case_id source_path bpm downbeat; do
  mark_status "$case_id" "authorized_exact_path_open_started"
  args=(
    --source "$source_path"
    --bpm "$bpm"
    --output "$output/$case_id"
    --controlled-source-review
  )
  if [[ "$downbeat" != "null" ]]; then
    args+=(--downbeat-seconds "$downbeat")
  fi
  manifest="$output/$case_id/controlled-source-manifest.json"
  if ! cargo run -p riotbox-app --bin dense_break_live_path_render -- "${args[@]}"; then
    if [[ -f "$manifest" ]]; then
      content_hash="$(jq -r '.source.content_hash // "" | sub("^sha256:"; "")' "$manifest")"
      mark_status "$case_id" "opened_then_render_failed_closed" "$content_hash"
    fi
    mark_session_failed_now
    exit 1
  fi

  python3 scripts/validate_listening_manifest_json.py --require-existing-artifacts "$manifest"
  canonical_source_path="$(realpath "$source_path")"
  jq -e --arg source_path "$canonical_source_path" '
    .result == "pass"
    and .source.path == $source_path
    and (.source.content_hash | test("^sha256:[0-9a-f]{64}$"))
    and .tr909_impact_pocket_proof.schema == "riotbox.tr909_impact_pocket_proof.v1"
    and .tr909_impact_pocket_proof.decision == "RBX-287"
    and .tr909_impact_pocket_proof.performer_action == "tr909.set_slam"
    and .tr909_impact_pocket_proof.mode == "break_reinforce"
    and .tr909_impact_pocket_proof.routing == "drum_bus_support"
    and .tr909_impact_pocket_proof.slammed_tr909_render_is_identical == true
    and .tr909_impact_pocket_proof.delta.active_samples > 0
    and .tr909_impact_pocket_proof.locality.inside_changed_frames > 0
    and (.tr909_impact_pocket_proof.locality.outside_delta_peak
      <= .tr909_impact_pocket_proof.locality.maximum_outside_delta_peak)
    and .tr909_impact_pocket_proof.control_limiter.pre.clip_count == 0
    and .tr909_impact_pocket_proof.control_limiter.limited_sample_count == 0
    and .tr909_impact_pocket_proof.control_limiter.post.clip_count == 0
    and .tr909_impact_pocket_proof.candidate_limiter.pre.clip_count == 0
    and .tr909_impact_pocket_proof.candidate_limiter.limited_sample_count == 0
    and .tr909_impact_pocket_proof.candidate_limiter.post.clip_count == 0
  ' "$manifest" >/dev/null
  content_hash="$(jq -r '.source.content_hash | sub("^sha256:"; "")' "$manifest")"
  mark_status "$case_id" "verified_and_rendered" "$content_hash"
done < <(jq -r '.cases[] | [.case_id, .source_path, (.bpm | tostring), (.downbeat_seconds | tostring)] | @tsv' "$contract")

jq -e '([.files[].content_hash] | unique | length) == 3 and all(.files[]; .status == "verified_and_rendered")' "$access_log" >/dev/null

tmp="$output/.access-log.tmp"
jq '.status = "completed"' "$access_log" >"$tmp"
mv "$tmp" "$access_log"

report="$output/technical-report.json"
jq -n \
  --slurpfile contract "$contract" \
  --slurpfile access "$access_log" \
  --slurpfile dense "$output/dense_beat03_130/controlled-source-manifest.json" \
  --slurpfile tonal "$output/tonal_rusharp_120/controlled-source-manifest.json" \
  --slurpfile sparse "$output/sparse_kicksnr_120/controlled-source-manifest.json" \
  '{
    schema: "riotbox.tr909_impact_pocket_development_result.v1",
    ticket: "RIOTBOX-1436",
    decision: "RBX-287",
    result: "technical_pass_human_unverified",
    human_verdict: "unverified",
    quality_proof: false,
    hardness_proof: false,
    access: $access[0],
    contract_sha256: $access[0].contract_sha256,
    cases: [
      {case_id: "dense_beat03_130", source_path: $dense[0].source.path, source_hash: $dense[0].source.content_hash, impact_pocket: $dense[0].tr909_impact_pocket_proof},
      {case_id: "tonal_rusharp_120", source_path: $tonal[0].source.path, source_hash: $tonal[0].source.content_hash, impact_pocket: $tonal[0].tr909_impact_pocket_proof},
      {case_id: "sparse_kicksnr_120", source_path: $sparse[0].source.path, source_hash: $sparse[0].source.content_hash, impact_pocket: $sparse[0].tr909_impact_pocket_proof}
    ],
    unique_source_hash_count: ([$dense[0].source.content_hash, $tonal[0].source.content_hash, $sparse[0].source.content_hash] | unique | length),
    source_support_profiles: ([$dense[0].tr909_impact_pocket_proof.source_support_profile, $tonal[0].tr909_impact_pocket_proof.source_support_profile, $sparse[0].tr909_impact_pocket_proof.source_support_profile] | unique),
    holdout_audio_opened: false,
    commercial_reference_audio_opened: false
  }' >"$report"

trap - ERR

echo "TR-909 impact-pocket Development gate passed: $report"

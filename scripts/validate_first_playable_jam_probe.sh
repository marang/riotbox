#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

probe_dir="$tmpdir/first-playable-jam"
mkdir -p "$probe_dir"

python3 scripts/write_synthetic_break_wav.py "$tmpdir/source.wav" 4.0
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --role baseline \
  --out "$probe_dir/baseline.wav" \
  --duration-seconds 0.5
cargo run -p riotbox-audio --bin w30_preview_render -- \
  --role candidate \
  --out "$probe_dir/candidate.wav" \
  --duration-seconds 0.5 \
  --source "$tmpdir/source.wav" \
  --source-duration-seconds 0.25
cargo run -p riotbox-audio --bin w30_preview_compare -- \
  --baseline "$probe_dir/baseline.metrics.md" \
  --candidate "$probe_dir/candidate.metrics.md" \
  --report "$probe_dir/comparison.md" \
  --min-rms-delta 0.001 \
  --min-sum-delta 1.0 \
  --max-active-samples-delta 200000 \
  --max-peak-delta 1.0 \
  --max-rms-delta 1.0 \
  --max-sum-delta 1000.0
python3 scripts/validate_listening_manifest_json.py \
  --require-existing-artifacts \
  "$probe_dir/manifest.json"

summary="$probe_dir/observer-audio-summary.json"
cargo run -p riotbox-app --bin observer_audio_correlate -- \
  --observer crates/riotbox-app/tests/fixtures/first_playable_jam_probe/events.ndjson \
  --manifest "$probe_dir/manifest.json" \
  --output "$summary" \
  --json \
  --require-evidence
jq -e \
  '.schema == "riotbox.observer_audio_summary.v1"
    and .control_path.present == true
    and (.control_path.key_outcomes | index("space -> transport started")) != null
    and (.control_path.key_outcomes | index("c -> capture queued")) != null
    and (.control_path.key_outcomes | index("o -> audition raw/src")) != null
    and (.control_path.key_outcomes | index("p -> promote queued")) != null
    and (.control_path.key_outcomes | index("w -> recall/src")) != null
    and .output_path.present == true
    and (.output_path.issues | length == 0)
    and .output_path.metrics.w30_candidate_rms > 0.000001
    and .output_path.metrics.w30_candidate_active_sample_ratio > 0.000001
    and .output_path.metrics.w30_rms_delta > 0.000001' \
  "$summary"
python3 scripts/validate_observer_audio_summary_json.py "$summary"

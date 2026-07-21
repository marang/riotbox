#!/usr/bin/env bash
set -euo pipefail

output="${1:-artifacts/audio_qa/local-controlled-source-live-matrix}"
mkdir -p "$output"

render_case() {
  local case_id="$1"
  local source="$2"
  local bpm="$3"
  shift 3
  cargo run -p riotbox-app --bin dense_break_live_path_render -- \
    --source "$source" \
    --bpm "$bpm" \
    --controlled-source-review \
    --output "$output/$case_id" \
    "$@"
  python3 scripts/validate_listening_manifest_json.py \
    --require-existing-artifacts \
    "$output/$case_id/controlled-source-manifest.json"
}

render_case dense-a 'data/test_audio/examples/Beat03_130BPM(Full).wav' 130
render_case tonal-a data/test_audio/examples/DH_RushArp_120_A.wav 120 --downbeat-seconds 0
render_case tonal-b data/test_audio/examples/DH_RushArp_120_A.wav 120 --downbeat-seconds 0
render_case sparse-a data/test_audio/examples/DH_BeatC_KickSnr_120-01.wav 120
render_case sparse-b data/test_audio/examples/DH_BeatC_KickSnr_120-01.wav 120

python3 scripts/validate_controlled_source_live_matrix.py "$output"

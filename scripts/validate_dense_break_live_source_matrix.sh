#!/usr/bin/env bash
set -euo pipefail

output="${1:-artifacts/audio_qa/local-dense-break-live-source-matrix}"
mkdir -p "$output"

cases=(
  "beat03|data/test_audio/examples/Beat03_130BPM(Full).wav|130"
  "beat08|data/test_audio/examples/Beat08_128BPM(Full).wav|128"
  "beat20|data/test_audio/examples/Beat20_128BPM(Full).wav|128"
  "dh-beatc|data/test_audio/examples/DH_BeatC_120-01.wav|120"
)

case_ids=()
for case in "${cases[@]}"; do
  IFS="|" read -r case_id source bpm <<<"$case"
  case_ids+=("$case_id")
  mkdir -p "$output/$case_id"
  cargo run -p riotbox-app --bin dense_break_live_path_render -- \
    --source "$source" \
    --bpm "$bpm" \
    --output "$output/$case_id"
done

python3 scripts/validate_dense_break_live_source_matrix.py "$output" "${case_ids[@]}"

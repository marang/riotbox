#!/usr/bin/env bash
set -euo pipefail

output="${1:-artifacts/audio_qa/local-dense-break-live-path-smoke}"
bpm="${2:-132}"
source_dir="$output/generated-source"

python3 scripts/write_diverse_test_source_wavs.py --output "$source_dir" --seconds 8.0
cargo run -p riotbox-app --bin dense_break_live_path_render -- \
  --source "$source_dir/dense_break_132.wav" \
  --bpm "$bpm" \
  --output "$output"

for artifact in \
  00_source.wav \
  01_all_lane_hook.wav \
  02_all_lane_destructive.wav \
  stems/01_w30_hook.wav \
  stems/02_tr909_pressure.wav \
  stems/03_mc202_selected_role.wav \
  session.json \
  source-graph.json
do
  test -s "$output/$artifact"
done

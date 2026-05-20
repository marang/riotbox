#!/usr/bin/env bash
set -euo pipefail

output_dir="$(realpath -m "${1:-artifacts/audio_qa/local-melodic-source-chop-showcase}")"
date_label="${2:-local-melodic-source-chop-showcase}"
source_path="${3:-data/test_audio/examples/DH_RushArp_120_A.wav}"
duration_seconds="${4:-2.0}"
source_window_seconds="${5:-1.0}"
source_start_seconds="${6:-0.0}"
source_end_seconds="$(python3 - "$source_start_seconds" "$duration_seconds" <<'PY'
import sys
print(float(sys.argv[1]) + float(sys.argv[2]))
PY
)"

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
case "$output_dir" in
    "$repo_root"/artifacts/audio_qa/*|/tmp/riotbox-*) ;;
    *)
        echo "refusing to reset melodic source-chop showcase output outside artifacts/audio_qa or /tmp/riotbox-*: $output_dir" >&2
        exit 1
        ;;
esac

if [[ ! -f "$source_path" ]]; then
    echo "melodic source-chop source missing: $source_path" >&2
    exit 1
fi

rm -rf "$output_dir"
mkdir -p "$output_dir/validation"

cargo run -p riotbox-audio --bin source_timing_probe -- --json "$source_path" \
    >"$output_dir/validation/source-timing.json"
python3 scripts/validate_source_timing_probe_json.py \
    "$output_dir/validation/source-timing.json"
jq -e '
    .cue == "needs confirm"
    and .readiness == "unavailable"
    and .requires_manual_confirm == true
    and .grid_use == "unavailable"
    and .primary_bpm == null
    and (.warning_codes | index("low_timing_confidence"))
    and (.warning_codes | index("weak_kick_anchor"))
' "$output_dir/validation/source-timing.json" >/dev/null

cargo run -p riotbox-audio --bin feral_before_after_pack -- \
    --source "$source_path" \
    --output-dir "$output_dir" \
    --date "$date_label" \
    --source-start-seconds "$source_start_seconds" \
    --duration-seconds "$duration_seconds" \
    --source-window-seconds "$source_window_seconds" \
    >/dev/null

python3 scripts/validate_listening_manifest_json.py \
    --require-existing-artifacts \
    "$output_dir/manifest.json"
jq -e '
    .pack_id == "feral-before-after"
    and .result == "pass"
    and .metrics.source_excerpt.rms > .thresholds.min_source_rms
    and .metrics.riotbox_after.rms > .thresholds.min_after_rms
    and .metrics.source_after_delta.rms > .thresholds.min_delta_rms
    and .metrics.w30_source_chop.rms > 0.001
' "$output_dir/manifest.json" >/dev/null

cat >"$output_dir/README.md" <<EOF
# Melodic Source-Chop Showcase

Result: \`pass\`

This local Riotbox showcase proves a bounded non-drum source-chop path for:

- Source: \`$source_path\`
- Source window: \`${source_start_seconds}s\` to \`${source_end_seconds}s\`
- W-30 preview window: \`${source_window_seconds}s\`

## Why This Exists

\`feral_grid_pack\` is a drum-support showcase. It should not present melodic
or arp-like material as trusted TR-style kick/backbeat timing when Source Timing
reports \`grid_use: unavailable\`.

This pack keeps that boundary explicit:

1. \`validation/source-timing.json\` must stay \`needs confirm\` /
   \`unavailable\` with \`low_timing_confidence\` and \`weak_kick_anchor\`.
2. \`stems/w30_source_chop.wav\` must render non-silent source-backed chop
   material.
3. \`02_riotbox_feral_changed.wav\` must differ from \`01_source_excerpt.wav\`
   by more than the manifest delta threshold.

## Listening Order

1. \`01_source_excerpt.wav\`: raw melodic source window.
2. \`stems/w30_source_chop.wav\`: source-backed W-30 chop from the same material.
3. \`02_riotbox_feral_changed.wav\`: bounded Riotbox after render.
4. \`03_before_then_after.wav\`: source excerpt, silence, then after render.

## Boundary

This is not a new arranger and not a drum-grid trust path. It is a local
melodic/source-chop proof that reuses the existing \`feral_before_after_pack\`
manifest and output QA seam until a fuller melodic source showcase exists.
EOF

echo "melodic source-chop showcase written to $output_dir"

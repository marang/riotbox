#!/usr/bin/env bash
set -euo pipefail

output_dir="$(realpath -m "${1:-artifacts/audio_qa/local-representative-source-showcase}")"
date_label="${2:-local-representative-source-showcase}"
source_seconds="${3:-8.0}"
bars="${4:-4}"

rm -rf "$output_dir"
mkdir -p "$output_dir"/{sources,packs,validation,observer}

sources_manifest="$(python3 scripts/write_synthetic_showcase_sources.py "$output_dir/sources" "$source_seconds")"
mapfile -t cases < <(python3 - "$sources_manifest" <<'PY'
import json
import sys
from pathlib import Path

data = json.loads(Path(sys.argv[1]).read_text())
for source in data["sources"]:
    for window in source["windows"]:
        print(
            "|".join(
                [
                    source["id"],
                    str(source["bpm"]),
                    source["path"],
                    window["id"],
                    str(window["start_seconds"]),
                    str(window["window_seconds"]),
                    source["description"],
                ]
            )
        )
PY
)

primary_packs=()
all_packs=()
for row in "${cases[@]}"; do
    IFS='|' read -r case_id bpm source_path window_id start_seconds window_seconds description <<<"$row"
    pack_dir="$output_dir/packs/${case_id}/${window_id}"
    mkdir -p "$pack_dir"
    python3 scripts/extract_wav_window.py \
        "$source_path" \
        "$pack_dir/00_source_window.wav" \
        "$start_seconds" \
        "$window_seconds" \
        >/dev/null
    cargo run -p riotbox-audio --bin feral_grid_pack -- \
        --source "$source_path" \
        --output-dir "$pack_dir" \
        --date "$date_label" \
        --bpm "$bpm" \
        --bars "$bars" \
        --source-window-seconds "$window_seconds" \
        --source-start-seconds "$start_seconds" \
        >/dev/null
    all_packs+=("$pack_dir")
    if [[ "$window_id" == "head" ]]; then
        primary_packs+=("$pack_dir")
    fi
done

python3 scripts/validate_source_showcase_diversity.py \
    --json-output "$output_dir/validation/source-diversity.json" \
    --markdown-output "$output_dir/validation/source-diversity.md" \
    "${primary_packs[@]}"

repro_a="$output_dir/validation/repro-a"
repro_b="$output_dir/validation/repro-b"
first_source="$(python3 - "$sources_manifest" <<'PY'
import json
import sys
from pathlib import Path
source = json.loads(Path(sys.argv[1]).read_text())["sources"][0]
print("|".join([source["path"], str(source["bpm"])]))
PY
)"
IFS='|' read -r repro_source repro_bpm <<<"$first_source"
for repro_dir in "$repro_a" "$repro_b"; do
    cargo run -p riotbox-audio --bin feral_grid_pack -- \
        --source "$repro_source" \
        --output-dir "$repro_dir" \
        --date "$date_label" \
        --bpm "$repro_bpm" \
        --bars "$bars" \
        --source-window-seconds "1.0" \
        --source-start-seconds "0.0" \
        >/dev/null
done
hash_a="$(sha256sum "$repro_a/04_riotbox_generated_support_mix.wav" | awk '{print $1}')"
hash_b="$(sha256sum "$repro_b/04_riotbox_generated_support_mix.wav" | awk '{print $1}')"
if [[ "$hash_a" != "$hash_b" ]]; then
    echo "representative source showcase reproducibility failed: $hash_a != $hash_b" >&2
    exit 1
fi
cat >"$output_dir/validation/reproducibility.md" <<EOF
# Reproducibility

- Source: \`$repro_source\`
- Mix hash A: \`$hash_a\`
- Mix hash B: \`$hash_b\`
- Result: \`pass\`
EOF

cargo run -p riotbox-app --bin user_session_observer_probe -- \
    --probe feral-grid-jam \
    --observer "$output_dir/observer/events.ndjson" \
    >/dev/null
cargo run -p riotbox-app --bin observer_audio_correlate -- \
    --observer "$output_dir/observer/events.ndjson" \
    --manifest "${primary_packs[0]}/manifest.json" \
    --output "$output_dir/observer/observer-audio-summary.json" \
    --json \
    --require-evidence \
    >/dev/null
python3 scripts/validate_observer_audio_summary_json.py \
    "$output_dir/observer/observer-audio-summary.json"

cat >"$output_dir/README.md" <<EOF
# Representative Source Showcase

Result: \`pass\`

This is a local, ignored Riotbox source-showcase pack generated from deterministic synthetic fixture sources. It is meant to prove that the current Feral grid path reacts differently to distinct source material after the source-diversity, mix-balance, source-aware TR-909, and W-30 source-chop fixes.

## Listening Order

For each \`packs/<case>/<window>/\` directory:

1. \`00_source_window.wav\`: raw source comparison window.
2. \`stems/02_w30_feral_source_chop.wav\`: source-backed W-30 chop.
3. \`03_riotbox_source_first_mix.wav\`: source-first Riotbox render.
4. \`04_riotbox_generated_support_mix.wav\`: generated-support mix.

## Validation

- Source-diversity summary: \`validation/source-diversity.md\`
- Reproducibility summary: \`validation/reproducibility.md\`
- Observer/audio JSON summary: \`observer/observer-audio-summary.json\`

## Boundary

This pack does not claim full kick/snare/bass source separation. It is a representative source-response pack, not a finished musical demo or release asset.
EOF

cat >"$output_dir/validation/pack-index.txt" <<EOF
Representative source showcase packs:
$(printf '%s\n' "${all_packs[@]}")
EOF

echo "representative source showcase written to $output_dir"

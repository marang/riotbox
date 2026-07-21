#!/usr/bin/env bash
set -euo pipefail

output="${1:-artifacts/audio_qa/local-dense-break-live-source-matrix}"
echo "dense source matrix migrated to typed dense/tonal/sparse controlled live matrix"
exec scripts/validate_controlled_source_live_matrix.sh "$output"

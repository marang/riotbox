#!/usr/bin/env bash
set -euo pipefail

expected="registered-development-only"
actual="${RIOTBOX_BROAD_AUDIO_QA_ACCESS:-}"

if [[ "$actual" != "$expected" ]]; then
  cat >&2 <<'EOF'
broad audio QA is fail-closed because it opens registered Development audio.
Use the source-free normal PR gate:
  just ci

Only an explicitly authorized phase/release run may opt in with:
  RIOTBOX_BROAD_AUDIO_QA_ACCESS=registered-development-only just ci-broad

This opt-in never authorizes Holdout audio, commercial references, or source
directory discovery.
EOF
  exit 2
fi

printf '%s\n' \
  'broad audio QA access acknowledged: registered Development audio only; Holdout and commercial references remain forbidden'

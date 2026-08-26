#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
guard="$script_dir/require_broad_audio_qa_access.sh"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

if env -u RIOTBOX_BROAD_AUDIO_QA_ACCESS "$guard" >"$tmpdir/missing.out" 2>&1; then
  echo "expected missing broad audio-QA access acknowledgement to fail" >&2
  exit 1
fi
grep -q "broad audio QA is fail-closed" "$tmpdir/missing.out"

if RIOTBOX_BROAD_AUDIO_QA_ACCESS=wrong "$guard" >"$tmpdir/wrong.out" 2>&1; then
  echo "expected incorrect broad audio-QA access acknowledgement to fail" >&2
  exit 1
fi
grep -q "registered-development-only" "$tmpdir/wrong.out"

RIOTBOX_BROAD_AUDIO_QA_ACCESS=registered-development-only \
  "$guard" >"$tmpdir/accepted.out"
grep -q "registered Development audio only" "$tmpdir/accepted.out"
grep -q "Holdout and commercial references remain forbidden" "$tmpdir/accepted.out"

echo "broad audio-QA access guard fixtures passed"

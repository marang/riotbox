#!/usr/bin/env bash
set -euo pipefail

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

source_a="$tmpdir/source-a.wav"
source_b="$tmpdir/source-b.wav"
run_a="$tmpdir/run-a"
run_b="$tmpdir/run-b"
out_a="$run_a/candidate.wav"
out_b="$run_b/candidate.wav"

python3 scripts/write_synthetic_break_wav.py "$source_a" 4.0
python3 scripts/write_synthetic_break_wav.py "$source_b" 4.0

source_hash_a="$(sha256sum "$source_a" | awk '{print $1}')"
source_hash_b="$(sha256sum "$source_b" | awk '{print $1}')"
if [[ "$source_hash_a" != "$source_hash_b" ]]; then
  echo "deterministic source generation drifted: $source_hash_a != $source_hash_b" >&2
  exit 1
fi

cargo run -p riotbox-audio --bin w30_preview_render -- \
  --role candidate \
  --out "$out_a" \
  --duration-seconds 2.0 \
  --source "$source_a" \
  --source-duration-seconds 1.0

cargo run -p riotbox-audio --bin w30_preview_render -- \
  --role candidate \
  --out "$out_b" \
  --duration-seconds 2.0 \
  --source "$source_b" \
  --source-duration-seconds 1.0

if [[ ! -s "$out_a" || ! -s "$out_b" ]]; then
  echo "offline render reproducibility smoke produced an empty WAV" >&2
  exit 1
fi

hash_a="$(sha256sum "$out_a" | awk '{print $1}')"
hash_b="$(sha256sum "$out_b" | awk '{print $1}')"
if [[ "$hash_a" != "$hash_b" ]]; then
  echo "offline render output is not byte-reproducible: $hash_a != $hash_b" >&2
  exit 1
fi

echo "offline render reproducibility ok: $hash_a"

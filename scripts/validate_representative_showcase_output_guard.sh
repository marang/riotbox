#!/usr/bin/env bash
set -euo pipefail

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

scripts/guard_showcase_output_dir.sh \
    "artifacts/audio_qa/local-representative-output-guard" \
    "representative showcase"

scripts/guard_showcase_output_dir.sh \
    "/tmp/riotbox-representative-output-guard" \
    "representative showcase"

unsafe_path="$tmpdir/not-under-riotbox-prefix"
if scripts/guard_showcase_output_dir.sh "$unsafe_path" "representative showcase" >"$tmpdir/unsafe.out" 2>&1; then
    cat "$tmpdir/unsafe.out" >&2
    echo "expected unsafe representative showcase output path to fail" >&2
    exit 1
fi
grep -q "refusing to reset representative showcase output" "$tmpdir/unsafe.out"

scripts/guard_showcase_output_dir.sh \
    "$unsafe_path" \
    "representative showcase" \
    --force-output-reset \
    >"$tmpdir/force.out" 2>&1
grep -q "forcing representative showcase output reset" "$tmpdir/force.out"

RIOTBOX_FORCE_SHOWCASE_OUTPUT_RESET=1 scripts/guard_showcase_output_dir.sh \
    "$unsafe_path" \
    "representative showcase" \
    >"$tmpdir/env-force.out" 2>&1
grep -q "forcing representative showcase output reset" "$tmpdir/env-force.out"

if scripts/guard_showcase_output_dir.sh "$unsafe_path" "representative showcase" --unknown >"$tmpdir/unknown.out" 2>&1; then
    cat "$tmpdir/unknown.out" >&2
    echo "expected unknown output reset guard argument to fail" >&2
    exit 1
fi
grep -q "unsupported output reset guard argument" "$tmpdir/unknown.out"

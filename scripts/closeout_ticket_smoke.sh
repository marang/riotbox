#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

if scripts/closeout_ticket.sh --ticket RIOTBOX-999999 >"$tmp/missing.out" 2>"$tmp/missing.err"; then
  echo "expected missing archive handoff to fail" >&2
  exit 1
fi
grep -q "archive file missing" "$tmp/missing.err"

scripts/closeout_ticket.sh --ticket RIOTBOX-755 >"$tmp/dry-run.out"
grep -q "archive handoff ok for RIOTBOX-755" "$tmp/dry-run.out"
grep -q "dry-run complete for RIOTBOX-755" "$tmp/dry-run.out"

echo "closeout ticket smoke ok"

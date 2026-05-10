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

scripts/closeout_ticket.sh --ticket RIOTBOX-755 --mem-status --mem-status-timeout 1 >"$tmp/mem-dry-run.out"
grep -q "dry-run: timeout 1s" "$tmp/mem-dry-run.out"

CLOSEOUT_MEM_STATUS_COMMAND='sleep 2' \
  scripts/closeout_ticket.sh --ticket RIOTBOX-755 --mem-status --mem-status-timeout 1 --execute \
  >"$tmp/mem-timeout.out"
grep -q "optional command timed out after 1s" "$tmp/mem-timeout.out"

placeholder_ticket="RIOTBOX-999998"
placeholder_archive="docs/archive/linear_issues/${placeholder_ticket}.md"
cp docs/archive/linear_issues/index.md "$tmp/index.md"
trap 'rm -f "$placeholder_archive"; cp "$tmp/index.md" docs/archive/linear_issues/index.md; rm -rf "$tmp"' EXIT
cat >"$placeholder_archive" <<'MD'
# `RIOTBOX-999998` Placeholder Closeout Smoke

- Ticket: `RIOTBOX-999998`

## Why This Ticket Existed

TODO: summarize why this ticket existed before closeout.
MD
printf '\n- [RIOTBOX-999998.md](./RIOTBOX-999998.md)\n  Placeholder closeout smoke.\n' >>docs/archive/linear_issues/index.md
if scripts/closeout_ticket.sh --ticket "$placeholder_ticket" >"$tmp/placeholder.out" 2>"$tmp/placeholder.err"; then
  echo "expected placeholder archive handoff to fail" >&2
  exit 1
fi
grep -q "archive still contains generator TODO placeholders" "$tmp/placeholder.err"

echo "closeout ticket smoke ok"

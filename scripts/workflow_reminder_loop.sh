#!/usr/bin/env bash
set -euo pipefail

WORKDIR="${1:-$(pwd)}"
INTERVAL_SECONDS="${WORKFLOW_REMINDER_INTERVAL_SECONDS:-30}"

cd "$WORKDIR"

while true; do
  ts="$(date -Iseconds)"
  branch="$(git branch --show-current 2>/dev/null || echo detached)"
  if [ -z "$(git status --short 2>/dev/null)" ]; then
    tree_state="clean"
  else
    tree_state="dirty"
  fi

  printf '[%s] workflow-reminder: branch=%s tree=%s | keep implementing, keep Linear/archive aligned, re-check open PRs periodically, do not idle on CI or review gates.\n' \
    "$ts" "$branch" "$tree_state"

  sleep "$INTERVAL_SECONDS"
done

#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DECISION_LOG="$REPO_ROOT/docs/research_decision_log.md"

MAX_LINES="${MAX_LINES:-80}"
RG_CONTEXT="${RG_CONTEXT:-2}"

usage() {
  cat <<'EOF'
Usage:
  scripts/research_decision_search.sh "query text"

Behavior:
  - searches docs/research_decision_log.md with bounded rg output
  - first tries the exact query as a fixed string
  - then falls back to query terms with at least 3 characters

Environment:
  MAX_LINES=80
  RG_CONTEXT=2
EOF
}

if [ "${1:-}" = "--mine" ]; then
  echo "--mine is no longer supported; semantic-memory mining has been removed." >&2
  shift
fi

if [ "$#" -eq 0 ]; then
  usage >&2
  exit 1
fi

QUERY="$*"

run_rg_search() {
  local tmp
  tmp="$(mktemp)"

  echo "== rg decision search: docs/research_decision_log.md =="
  if rg -n -i -F -C "$RG_CONTEXT" -- "$QUERY" "$DECISION_LOG" >"$tmp"; then
    sed -n "1,${MAX_LINES}p" "$tmp"
    rm -f "$tmp"
    return 0
  fi

  local -a rg_args
  rg_args=(-n -i -F -C "$RG_CONTEXT")
  local term
  local added=0
  for term in $QUERY; do
    if [ "${#term}" -ge 3 ]; then
      rg_args+=(-e "$term")
      added=$((added + 1))
    fi
  done

  if [ "$added" -eq 0 ]; then
    echo "No searchable terms with at least 3 characters." >&2
    rm -f "$tmp"
    return 1
  fi

  if rg "${rg_args[@]}" -- "$DECISION_LOG" >"$tmp"; then
    sed -n "1,${MAX_LINES}p" "$tmp"
    rm -f "$tmp"
    return 0
  fi

  echo "No matches in docs/research_decision_log.md." >&2
  rm -f "$tmp"
  return 1
}

run_rg_search

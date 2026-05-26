#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_DIR="$REPO_ROOT/.mempalace"
DECISION_LOG="$REPO_ROOT/docs/research_decision_log.md"
CURRENT_MANIFEST="$STATE_DIR/current_manifest.sha256"
INDEXED_MANIFEST="$STATE_DIR/indexed_manifest.sha256"
CURRENT_ROOM_CONFIG="$STATE_DIR/current_room_config.sha256"
INDEXED_ROOM_CONFIG="$STATE_DIR/indexed_room_config.sha256"

MEM_TIMEOUT="${MEM_TIMEOUT:-20s}"
MAX_LINES="${MAX_LINES:-80}"
RG_CONTEXT="${RG_CONTEXT:-2}"
ALLOW_MINE=0

usage() {
  cat <<'EOF'
Usage:
  scripts/research_decision_search.sh [--mine] "query text"

Behavior:
  - syncs the MemPalace corpus so docs/research_decision_log.md is copied into
    the decisions room
  - uses MemPalace only when the index is already fresh, unless --mine is passed
  - bounds MemPalace runtime with MEM_TIMEOUT, default 20s
  - falls back to bounded rg output from docs/research_decision_log.md

Environment:
  MEM_TIMEOUT=20s
  MAX_LINES=80
  RG_CONTEXT=2
EOF
}

if [ "${1:-}" = "--mine" ]; then
  ALLOW_MINE=1
  shift
fi

if [ "$#" -eq 0 ]; then
  usage >&2
  exit 1
fi

QUERY="$*"

index_is_fresh() {
  [ -f "$CURRENT_MANIFEST" ] &&
    [ -f "$INDEXED_MANIFEST" ] &&
    [ -f "$CURRENT_ROOM_CONFIG" ] &&
    [ -f "$INDEXED_ROOM_CONFIG" ] &&
    cmp -s "$CURRENT_MANIFEST" "$INDEXED_MANIFEST" &&
    cmp -s "$CURRENT_ROOM_CONFIG" "$INDEXED_ROOM_CONFIG"
}

run_mempalace_search() {
  local tmp
  tmp="$(mktemp)"

  if command -v timeout >/dev/null 2>&1; then
    if timeout --kill-after=5s "$MEM_TIMEOUT" \
      "$REPO_ROOT/scripts/mempalace.sh" search "research_decision_log.md decisions $QUERY" \
      >"$tmp" 2>&1; then
      sed -n "1,${MAX_LINES}p" "$tmp"
      rm -f "$tmp"
      return 0
    fi
  elif "$REPO_ROOT/scripts/mempalace.sh" search "research_decision_log.md decisions $QUERY" \
    >"$tmp" 2>&1; then
    sed -n "1,${MAX_LINES}p" "$tmp"
    rm -f "$tmp"
    return 0
  fi

  echo "MemPalace search did not finish cleanly; falling back to rg." >&2
  sed -n "1,20p" "$tmp" >&2
  rm -f "$tmp"
  return 1
}

run_rg_fallback() {
  local tmp
  tmp="$(mktemp)"

  echo "== rg fallback: docs/research_decision_log.md =="
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

"$REPO_ROOT/scripts/mempalace.sh" sync >/dev/null

if [ "$ALLOW_MINE" -eq 1 ]; then
  echo "== MemPalace decision search, mining allowed =="
  run_mempalace_search || run_rg_fallback
elif index_is_fresh; then
  echo "== MemPalace decision search, fresh index =="
  run_mempalace_search || run_rg_fallback
else
  echo "MemPalace corpus synced, but the index is stale; skipping auto-mine."
  echo "Use --mine when semantic retrieval is worth the indexing cost."
  run_rg_fallback
fi

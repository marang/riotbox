#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_DIR="$REPO_ROOT/.mempalace"
CORPUS_DIR="$STATE_DIR/corpus"
PALACE_DIR="$STATE_DIR/palace"
CACHE_DIR="$STATE_DIR/cache"
RESULTS_DIR="$STATE_DIR/results"
CURRENT_MANIFEST="$STATE_DIR/current_manifest.sha256"
INDEXED_MANIFEST="$STATE_DIR/indexed_manifest.sha256"
COMPOSE_FILE="$REPO_ROOT/compose.mempalace.yaml"
IMAGE_PREPARED=0

usage() {
  cat <<'EOF'
Usage:
  scripts/mempalace.sh init
  scripts/mempalace.sh sync
  scripts/mempalace.sh mine
  scripts/mempalace.sh status
  scripts/mempalace.sh search "query text"
  scripts/mempalace.sh shell

Behavior:
  - keeps all MemPalace state under .mempalace/
  - syncs docs/, plan/, crates/, and AGENTS.md into the local corpus
  - automatically re-mines before search/status if the source corpus changed
EOF
}

ensure_dirs() {
  mkdir -p "$STATE_DIR" "$CORPUS_DIR" "$PALACE_DIR" "$CACHE_DIR" "$RESULTS_DIR"
}

sync_corpus() {
  ensure_dirs

  rm -rf "$CORPUS_DIR/docs" "$CORPUS_DIR/plan" "$CORPUS_DIR/crates" "$CORPUS_DIR/AGENTS.md"
  cp -R "$REPO_ROOT/docs" "$CORPUS_DIR/docs"
  cp -R "$REPO_ROOT/plan" "$CORPUS_DIR/plan"
  cp -R "$REPO_ROOT/crates" "$CORPUS_DIR/crates"
  cp "$REPO_ROOT/AGENTS.md" "$CORPUS_DIR/AGENTS.md"
}

write_manifest() {
  (
    cd "$REPO_ROOT"
    {
      find docs plan crates -type f -print0
      printf '%s\0' "AGENTS.md"
    } | sort -z | while IFS= read -r -d '' path; do
      sha256sum "$path"
    done
  ) > "$CURRENT_MANIFEST"
}

compose_run() {
  local command="$1"
  compose_prepare
  (
    cd "$REPO_ROOT"
    podman compose -f "$COMPOSE_FILE" run --rm mempalace bash -lc "$command"
  )
}

compose_prepare() {
  if [ "$IMAGE_PREPARED" -eq 0 ]; then
    (
      cd "$REPO_ROOT"
      podman compose -f "$COMPOSE_FILE" build mempalace
    )
    IMAGE_PREPARED=1
  fi
}

ensure_initialized() {
  if [ ! -f "$CORPUS_DIR/mempalace.yaml" ]; then
    compose_run "mempalace --palace /palace init --yes /repo/.mempalace/corpus"
  fi
}

mine_corpus() {
  sync_corpus
  write_manifest
  ensure_initialized
  compose_run "mempalace --palace /palace mine /repo/.mempalace/corpus"
  cp "$CURRENT_MANIFEST" "$INDEXED_MANIFEST"
}

ensure_fresh_index() {
  sync_corpus
  write_manifest

  if [ ! -f "$INDEXED_MANIFEST" ] || ! cmp -s "$CURRENT_MANIFEST" "$INDEXED_MANIFEST"; then
    echo "MemPalace corpus changed, re-mining..."
    ensure_initialized
    compose_run "mempalace --palace /palace mine /repo/.mempalace/corpus"
    cp "$CURRENT_MANIFEST" "$INDEXED_MANIFEST"
  fi
}

command="${1:-}"

case "$command" in
  init)
    mine_corpus
    ;;
  sync)
    sync_corpus
    write_manifest
    ;;
  mine)
    mine_corpus
    ;;
  status)
    ensure_fresh_index
    compose_run "mempalace --palace /palace status"
    ;;
  search)
    shift || true
    if [ "$#" -eq 0 ]; then
      echo "search requires a query" >&2
      exit 1
    fi
    ensure_fresh_index
    query="$*"
    printf -v escaped_query '%q' "$query"
    compose_run "mempalace --palace /palace search $escaped_query --results 5"
    ;;
  shell)
    ensure_dirs
    (
      cd "$REPO_ROOT"
      podman compose -f "$COMPOSE_FILE" run --rm mempalace bash
    )
    ;;
  ""|help|-h|--help)
    usage
    ;;
  *)
    echo "unknown command: $command" >&2
    usage >&2
    exit 1
    ;;
esac

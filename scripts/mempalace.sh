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
CURRENT_ROOM_CONFIG="$STATE_DIR/current_room_config.sha256"
INDEXED_ROOM_CONFIG="$STATE_DIR/indexed_room_config.sha256"
CURRENT_IMAGE_MANIFEST="$STATE_DIR/current_image.sha256"
BUILT_IMAGE_MANIFEST="$STATE_DIR/built_image.sha256"
LOCK_FILE="$STATE_DIR/mempalace.lock"
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
  - writes a repo-specific room structure into the generated corpus
  - automatically re-mines before search/status if the source corpus changed
  - serializes operations so multiple status/search calls do not mine concurrently
  - rebuilds the MemPalace image only when its compose/container files change
EOF
}

ensure_dirs() {
  mkdir -p "$STATE_DIR" "$CORPUS_DIR" "$PALACE_DIR" "$CACHE_DIR" "$RESULTS_DIR"
}

acquire_lock() {
  ensure_dirs
  exec 9>"$LOCK_FILE"
  if ! flock -n 9; then
    echo "MemPalace operation already running; waiting for the repo lock..." >&2
    flock 9
  fi
}

write_room_config() {
  cat > "$CORPUS_DIR/mempalace.yaml" <<'EOF'
wing: corpus
rooms:
- name: specs
  description: Product, architecture, runtime, audio, and workflow specs under docs/specs/
  keywords:
  - docs/specs
  - specification
  - contract
  - architecture
- name: workflow
  description: Agent, GitHub, Linear, MemPalace, and repository operating conventions
  keywords:
  - workflow
  - linear
  - github
  - pull request
  - branch
  - ticket
  - AGENTS.md
  - mempalace
- name: reviews
  description: Codebase, seam, MVP exit, and periodic review artifacts
  keywords:
  - review
  - finding
  - gap
  - audit
  - exit review
  - hotspot
- name: audio_qa
  description: Audio QA, listening packs, benchmarks, probes, manifests, and output-proof material
  keywords:
  - audio qa
  - listening
  - benchmark
  - manifest
  - observer
  - render
  - wav
  - output proof
  - signal metrics
- name: plan
  description: Strategy, roadmap, masterplan, and phase planning material
  keywords:
  - plan
  - roadmap
  - phase
  - masterplan
  - milestone
- name: decisions
  description: Decision logs, spikes, research notes, and frozen technical choices
  keywords:
  - decision
  - decision log
  - spike
  - research
  - frozen
  - tradeoff
- name: code
  description: Rust crate source and test implementation details
  keywords:
  - crates/
  - rust
  - cargo
  - test
  - module
  - struct
  - enum
  - impl
- name: documentation
  description: Product-facing docs, recipes, README material, and general documentation
  keywords:
  - documentation
  - docs
  - readme
  - recipe
  - guide
- name: general
  description: Files that do not fit a more specific Riotbox room
  keywords: []
EOF
}

copy_file_to_room() {
  local source="$1"
  local room="$2"
  local target="$CORPUS_DIR/$room/$source"

  if [ -f "$REPO_ROOT/$source" ]; then
    mkdir -p "$(dirname "$target")"
    cp "$REPO_ROOT/$source" "$target"
  fi
}

copy_tree_to_room() {
  local source="$1"
  local room="$2"
  local target="$CORPUS_DIR/$room/$source"

  if [ -d "$REPO_ROOT/$source" ]; then
    mkdir -p "$(dirname "$target")"
    cp -R "$REPO_ROOT/$source" "$target"
  fi
}

sync_corpus() {
  ensure_dirs

  rm -rf "$CORPUS_DIR"/*

  copy_tree_to_room "docs/specs" "specs"

  copy_file_to_room "AGENTS.md" "workflow"
  copy_file_to_room "docs/workflow_conventions.md" "workflow"
  copy_file_to_room ".mempalace/README.md" "workflow"
  copy_file_to_room "Justfile" "workflow"
  copy_file_to_room "scripts/mempalace.sh" "workflow"
  copy_file_to_room "scripts/linear_issue_delete.sh" "workflow"

  copy_tree_to_room "docs/reviews" "reviews"

  copy_tree_to_room "docs/benchmarks" "audio_qa"
  copy_file_to_room "docs/specs/audio_qa_workflow_spec.md" "audio_qa"

  copy_tree_to_room "plan" "plan"
  copy_file_to_room "docs/execution_roadmap.md" "plan"
  copy_file_to_room "docs/phase_definition_of_done.md" "plan"

  copy_file_to_room "docs/research_decision_log.md" "decisions"
  copy_tree_to_room "docs/spikes" "decisions"

  copy_tree_to_room "crates" "code"

  copy_file_to_room "docs/README.md" "documentation"
  copy_file_to_room "docs/prd_v1.md" "documentation"
  copy_file_to_room "docs/jam_recipes.md" "documentation"
  copy_tree_to_room "docs/screenshots" "documentation"

  write_room_config
}

write_manifest() {
  (
    cd "$REPO_ROOT"
    {
      find docs -type f ! -path 'docs/archive/linear_issues/*' ! -path 'docs/assets/*' -print0
      find plan crates -type f -print0
      printf '%s\0' "AGENTS.md"
      printf '%s\0' ".mempalace/README.md"
      printf '%s\0' "Justfile"
      printf '%s\0' "scripts/mempalace.sh"
      printf '%s\0' "scripts/linear_issue_delete.sh"
    } | sort -z | while IFS= read -r -d '' path; do
      sha256sum "$path"
    done
  ) > "$CURRENT_MANIFEST"
  sha256sum "$CORPUS_DIR/mempalace.yaml" > "$CURRENT_ROOM_CONFIG"
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
      sha256sum Containerfile.mempalace compose.mempalace.yaml
    ) > "$CURRENT_IMAGE_MANIFEST"

    if podman image exists riotbox-mempalace:latest \
      && [ -f "$BUILT_IMAGE_MANIFEST" ] \
      && cmp -s "$CURRENT_IMAGE_MANIFEST" "$BUILT_IMAGE_MANIFEST"; then
      IMAGE_PREPARED=1
      return
    fi

    (
      cd "$REPO_ROOT"
      podman compose -f "$COMPOSE_FILE" build mempalace
    )
    cp "$CURRENT_IMAGE_MANIFEST" "$BUILT_IMAGE_MANIFEST"
    IMAGE_PREPARED=1
  fi
}

ensure_initialized() {
  if [ ! -f "$CORPUS_DIR/mempalace.yaml" ]; then
    sync_corpus
  fi
  if [ ! -d "$PALACE_DIR" ] || [ -z "$(find "$PALACE_DIR" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]; then
    compose_run "mempalace --palace /palace init --yes /repo/.mempalace/corpus"
    # MemPalace init auto-generates rooms from folders; Riotbox owns a
    # curated room map so retrieval does not collapse into broad buckets.
    write_room_config
    sha256sum "$CORPUS_DIR/mempalace.yaml" > "$CURRENT_ROOM_CONFIG"
  fi
}

ensure_room_config_current() {
  if [ ! -f "$INDEXED_ROOM_CONFIG" ] || ! cmp -s "$CURRENT_ROOM_CONFIG" "$INDEXED_ROOM_CONFIG"; then
    echo "MemPalace room config changed, rebuilding palace index..."
    rm -rf "$PALACE_DIR"/*
    rm -f "$INDEXED_MANIFEST"
  fi
}

mark_indexed() {
  cp "$CURRENT_MANIFEST" "$INDEXED_MANIFEST"
  cp "$CURRENT_ROOM_CONFIG" "$INDEXED_ROOM_CONFIG"
}

mine_corpus() {
  sync_corpus
  write_manifest
  ensure_room_config_current
  ensure_initialized
  compose_run "mempalace --palace /palace mine /repo/.mempalace/corpus"
  mark_indexed
}

ensure_fresh_index() {
  sync_corpus
  write_manifest
  ensure_room_config_current

  if [ ! -f "$INDEXED_MANIFEST" ] || ! cmp -s "$CURRENT_MANIFEST" "$INDEXED_MANIFEST"; then
    echo "MemPalace corpus changed, re-mining..."
    ensure_initialized
    compose_run "mempalace --palace /palace mine /repo/.mempalace/corpus"
    mark_indexed
  fi
}

command="${1:-}"

case "$command" in
  ""|help|-h|--help)
    usage
    exit 0
    ;;
  *)
    acquire_lock
    ;;
esac

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
  *)
    echo "unknown command: $command" >&2
    usage >&2
    exit 1
    ;;
esac

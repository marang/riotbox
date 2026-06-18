#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: scripts/with_audio_qa_lock.sh LOCK_NAME COMMAND [ARGS...]

Run a broad audio-QA command under an exclusive repo-local lock.

Set RIOTBOX_AUDIO_QA_LOCK_ROOT to override the lock directory for tests.
EOF
}

die() {
  echo "audio QA lock: $*" >&2
  exit 73
}

if [ "$#" -lt 2 ]; then
  usage
  exit 2
fi

lock_name="$1"
shift

case "$lock_name" in
  *[!A-Za-z0-9._-]*|"")
    die "invalid lock name: $lock_name"
    ;;
esac

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
lock_root="${RIOTBOX_AUDIO_QA_LOCK_ROOT:-$repo_root/artifacts/audio_qa/.locks}"
lock_dir="$lock_root/$lock_name.lock"
owner_token="$$:$RANDOM:$RANDOM"
acquired=0

write_owner() {
  {
    printf '%s\n' "$$"
  } >"$lock_dir/pid"
  {
    printf '%s\n' "$owner_token"
  } >"$lock_dir/owner"
  {
    printf '%q ' "$@"
    printf '\n'
  } >"$lock_dir/command"
}

cleanup() {
  if [ "$acquired" -eq 1 ] && [ -f "$lock_dir/owner" ]; then
    if [ "$(cat "$lock_dir/owner" 2>/dev/null || true)" = "$owner_token" ]; then
      rm -rf "$lock_dir"
    fi
  fi
}
trap cleanup EXIT
trap 'cleanup; exit 130' INT
trap 'cleanup; exit 143' TERM

mkdir -p "$lock_root"
if mkdir "$lock_dir" 2>/dev/null; then
  acquired=1
  write_owner "$@"
else
  existing_pid="$(cat "$lock_dir/pid" 2>/dev/null || true)"
  if [ "$existing_pid" != "" ] && kill -0 "$existing_pid" 2>/dev/null; then
    existing_command="$(cat "$lock_dir/command" 2>/dev/null || true)"
    die "another broad audio-QA run is active (pid=$existing_pid, command=$existing_command)"
  fi
  rm -rf "$lock_dir"
  if ! mkdir "$lock_dir" 2>/dev/null; then
    die "could not acquire lock after stale cleanup: $lock_dir"
  fi
  acquired=1
  write_owner "$@"
fi

"$@"

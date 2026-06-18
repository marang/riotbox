#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

export RIOTBOX_AUDIO_QA_LOCK_ROOT="$tmp/locks"

scripts/with_audio_qa_lock.sh fixture-lock bash -c '
  test -f "$RIOTBOX_AUDIO_QA_LOCK_ROOT/fixture-lock.lock/pid"
'
test ! -e "$RIOTBOX_AUDIO_QA_LOCK_ROOT/fixture-lock.lock"

scripts/with_audio_qa_lock.sh fixture-lock sleep 2 &
holder_pid="$!"
for _ in $(seq 1 100); do
  if [ -f "$RIOTBOX_AUDIO_QA_LOCK_ROOT/fixture-lock.lock/pid" ]; then
    break
  fi
  sleep 0.02
done
test -f "$RIOTBOX_AUDIO_QA_LOCK_ROOT/fixture-lock.lock/pid"
busy_out="$tmp/busy.out"
if scripts/with_audio_qa_lock.sh fixture-lock true >"$busy_out" 2>&1; then
  cat "$busy_out" >&2
  kill "$holder_pid" 2>/dev/null || true
  wait "$holder_pid" 2>/dev/null || true
  echo "expected live audio-QA lock fixture to fail" >&2
  exit 1
fi
grep -q "another broad audio-QA run is active" "$busy_out"
wait "$holder_pid"
test ! -e "$RIOTBOX_AUDIO_QA_LOCK_ROOT/fixture-lock.lock"

mkdir -p "$RIOTBOX_AUDIO_QA_LOCK_ROOT/fixture-lock.lock"
printf '999999\n' >"$RIOTBOX_AUDIO_QA_LOCK_ROOT/fixture-lock.lock/pid"
scripts/with_audio_qa_lock.sh fixture-lock true
test ! -e "$RIOTBOX_AUDIO_QA_LOCK_ROOT/fixture-lock.lock"

echo "audio QA lock fixture gate ok"

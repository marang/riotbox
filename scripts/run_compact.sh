#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: scripts/run_compact.sh LOG_PATH COMMAND [ARG...]

Run a potentially noisy command with stdout/stderr redirected to LOG_PATH.

On success, print only a compact success line.
On failure, print the command, exit status, log path, and the last log lines.

Environment:
  RUN_COMPACT_TAIL_LINES   number of failure log lines to print; default: 80
EOF
}

if [ "$#" -lt 2 ]; then
  usage
  exit 2
fi

log_path="$1"
shift

tail_lines="${RUN_COMPACT_TAIL_LINES:-80}"
case "$tail_lines" in
  ''|*[!0-9]*)
    echo "run_compact: RUN_COMPACT_TAIL_LINES must be a positive integer" >&2
    exit 2
    ;;
esac

mkdir -p "$(dirname "$log_path")"

status=0
"$@" >"$log_path" 2>&1 || status="$?"

if [ "$status" -eq 0 ]; then
  printf 'ok:'
  printf ' %q' "$@"
  printf ' (log: %s)\n' "$log_path"
  exit 0
fi

printf 'failed (%s):' "$status" >&2
printf ' %q' "$@" >&2
printf ' (log: %s)\n' "$log_path" >&2
tail -n "$tail_lines" "$log_path" >&2 || true
exit "$status"

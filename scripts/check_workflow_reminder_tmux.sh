#!/usr/bin/env bash
set -euo pipefail

SESSION="${SESSION:-riotbox_workflow}"
WINDOW="agents"
OUTPUT_WAIT_SECONDS="${OUTPUT_WAIT_SECONDS:-3}"

if ! tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "missing-session $SESSION"
  exit 1
fi

echo "session=$SESSION"
tmux list-panes -t "${SESSION}:${WINDOW}" -F 'pane=#{pane_id} title=#{pane_title} dead=#{pane_dead} cmd=#{pane_current_command}'
echo "-- recent reminder output --"

deadline=$((SECONDS + OUTPUT_WAIT_SECONDS))
captured=""
while [ "$SECONDS" -lt "$deadline" ]; do
  captured="$(tmux capture-pane -p -S -200 -t "${SESSION}:${WINDOW}.0" | awk 'NF { lines[++count] = $0 } END { start = count > 6 ? count - 5 : 1; for (i = start; i <= count; i++) print lines[i] }')"
  if [ -n "$captured" ]; then
    break
  fi
  sleep 1
done

printf '%s\n' "$captured"

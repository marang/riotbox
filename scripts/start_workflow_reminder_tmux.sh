#!/usr/bin/env bash
set -euo pipefail

SESSION="${SESSION:-riotbox_workflow}"
WORKDIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WINDOW="agents"

command -v tmux >/dev/null

echo "Attach with: tmux attach -t $SESSION"

if tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "Session already exists: $SESSION"
  exit 0
fi

tmux new-session -d -s "$SESSION" -n "$WINDOW" -c "$WORKDIR" \
  "cd '$WORKDIR' && exec scripts/workflow_reminder_loop.sh '$WORKDIR'"
WORKER_PANE="$(tmux list-panes -t "${SESSION}:${WINDOW}" -F '#{pane_id}' | head -n1)"

tmux select-pane -t "$WORKER_PANE" -T "Workflow Reminder"
tmux setw -t "${SESSION}:${WINDOW}" pane-border-status top
tmux setw -t "${SESSION}:${WINDOW}" pane-border-format '#{pane_title}'

echo "Started workflow reminder session: $SESSION"

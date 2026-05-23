#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

observer_fixture="$tmpdir/events.ndjson"

cargo run -p riotbox-app --bin user_session_observer_probe -- \
  --probe source-timing-confirmation \
  --observer "$observer_fixture"
python3 scripts/validate_user_session_observer_ndjson.py "$observer_fixture"
jq -s -e \
  'length >= 4
    and .[0].event == "observer_started"
    and .[0].launch.probe == "source-timing-confirmation"
    and .[0].snapshot.source_timing.grid_confirmed == false
    and .[0].snapshot.source_timing.cue == "needs confirm"
    and .[0].snapshot.source_timing.degraded_policy == "manual_confirm"
    and .[0].snapshot.source_timing.grid_use == "manual_confirm_only"
    and any(.[]; .event == "key_outcome"
      and .key == "C"
      and .outcome == "confirm_source_timing_grid"
      and .status == "confirmed source timing grid"
      and .snapshot.queue.pending_count == 0
      and .snapshot.queue.queue_history_count == 1
      and .snapshot.queue.recent_history[0].command == "source_timing.confirm_grid"
      and .snapshot.queue.recent_history[0].status == "Committed"
      and .snapshot.source_timing.grid_confirmed == true
      and .snapshot.source_timing.confirmed_grid_source_id == "src-source-timing-confirmation"
      and .snapshot.source_timing.confirmed_grid_hypothesis_id == "probe-primary"
      and .snapshot.source_timing.cue == "needs confirm"
      and .snapshot.source_timing.primary_warning_code == "ambiguous_downbeat")
    and any(.[]; .event == "transport_commit"
      and .committed[0].boundary == "Immediate"
      and .snapshot.queue.session_log_count == 1
      and .snapshot.source_timing.grid_confirmed == true)' \
  "$observer_fixture"

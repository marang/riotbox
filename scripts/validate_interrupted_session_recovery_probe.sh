#!/usr/bin/env bash
set -euo pipefail

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

observer="$tmpdir/events.ndjson"

cargo run -p riotbox-app --bin user_session_observer_probe -- \
  --probe interrupted-session-recovery \
  --observer "$observer"

python3 scripts/validate_user_session_observer_ndjson.py "$observer"

jq -s -e '
  .[0].event == "observer_started"
  and .[0].launch.mode == "load"
  and .[0].launch.probe == "interrupted-session-recovery"
  and .[0].snapshot.recovery.present == true
  and .[0].snapshot.recovery.has_manual_candidates == true
  and .[0].snapshot.recovery.selected_candidate == null
  and .[0].snapshot.recovery.manual_choice_dry_run.selected_for_restore == false
  and (.[] | select(.event == "audio_runtime") | .snapshot.recovery.selected_candidate == null)
  and any(.[0].snapshot.recovery.candidates[]; .kind == "orphan temp file" and .status == "invalid session JSON" and .trust == "BrokenClue")
  and any(.[0].snapshot.recovery.candidates[]; .kind == "autosave file" and .status == "parseable session JSON" and .trust == "RecoverableClue")
' "$observer"

echo "interrupted-session recovery observer probe ok"

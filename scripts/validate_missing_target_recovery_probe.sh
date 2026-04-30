#!/usr/bin/env bash
set -euo pipefail

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

observer="$tmpdir/events.ndjson"

cargo run -p riotbox-app --bin user_session_observer_probe -- \
  --probe missing-target-recovery \
  --observer "$observer"

python3 scripts/validate_user_session_observer_ndjson.py "$observer"

jq -s -e '
  .[0].event == "observer_started"
  and .[0].launch.mode == "load"
  and .[0].launch.probe == "missing-target-recovery"
  and .[0].snapshot.recovery.present == true
  and .[0].snapshot.recovery.has_manual_candidates == true
  and .[0].snapshot.recovery.selected_candidate == null
  and .[0].snapshot.recovery.manual_choice_dry_run.selected_for_restore == false
  and (.[] | select(.event == "audio_runtime") | .snapshot.recovery.selected_candidate == null)
  and any(.[0].snapshot.recovery.candidates[]; .kind == "normal session path" and .status == "missing" and .trust == "MissingTarget")
  and any(.[0].snapshot.recovery.candidates[]; .kind == "autosave file" and .status == "parseable session JSON" and .trust == "RecoverableClue")
' "$observer"

echo "missing-target recovery observer probe ok"

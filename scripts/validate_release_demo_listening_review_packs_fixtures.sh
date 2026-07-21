#!/usr/bin/env bash
set -euo pipefail

output="${1:-artifacts/audio_qa/local-release-demo-listening-review-packs}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

python3 scripts/generate_release_demo_human_review_queue.py \
  --evidence-mode fixture_calibration \
  --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json \
  --output "$tmp/queue" \
  --date "local-release-demo-listening-review-packs"

python3 scripts/generate_release_demo_listening_review_packs.py \
  --queue "$tmp/queue/release-demo-human-review-queue.json" \
  --output "$output" \
  --ticket RIOTBOX-1384 \
  --date "local-release-demo-listening-review-packs"

python3 scripts/generate_release_demo_listening_review_packs.py \
  --validate-report "$output/release-demo-listening-review-packs.json"

jq -e \
  '.schema == "riotbox.release_demo_listening_review_packs.v1"
  and .result == "pass"
  and .quality_claim_allowed == false
  and .review_pack_count >= 5
  and any(.packs[];
    .entry_id == "bad-timing-beat20-unverified-candidate"
    and .review_priority == "high"
    and .human_verdict == "unverified"
    and .quality_claim == false)' \
  "$output/release-demo-listening-review-packs.json"

grep -q "Release-Demo Listening Review Packs" \
  "$output/release-demo-listening-review-packs.md"
grep -q "Current verdict state" \
  "$output/bad-timing-beat20-unverified-candidate/prompt.md"

REVIEW_OUTPUT="$output" python3 -c '
import json
import os
import sys
from pathlib import Path

sys.path.insert(0, "scripts")
from listening_review_workflow import validate_review

output = Path(os.environ["REVIEW_OUTPUT"])
review = json.loads((output / "bad-timing-beat20-unverified-candidate" / "review.json").read_text())
validate_review(review, allow_unverified=True)
assert review["human_verdict"] == "unverified"
assert review["release_demo_review"]["source_family"] == "bad_timing"
assert review["release_demo_review"]["quality_claim"] is False
'

tmp_report="$tmp/stale.json"
jq '.packs[0].human_verdict = "pass"' \
  "$output/release-demo-listening-review-packs.json" > "$tmp_report"
if python3 scripts/generate_release_demo_listening_review_packs.py \
  --validate-report "$tmp_report" > "$tmp/stale.out" 2>&1; then
  cat "$tmp/stale.out" >&2
  echo "expected stale generated review-pack summary to fail" >&2
  exit 1
fi
grep -q "pack_0_human_verdict_not_unverified" "$tmp/stale.out"

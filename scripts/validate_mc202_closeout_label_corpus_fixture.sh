#!/usr/bin/env bash
set -euo pipefail

repo="$(git rev-parse --show-toplevel)"
cd "$repo"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

review="artifacts/audio_qa/local-professional-output-listening-pack/reviews/dense_beat03_130/review.json"
test -s "$review"

cp "$review" "$tmp/review.json"
python3 scripts/listening_review_workflow.py record \
  --review "$tmp/review.json" \
  --human-verdict keep \
  --strongest-element snare \
  --source-recognition source_transformed_but_present \
  --hook-after-two-bars clear \
  --preferred-direction "keep the break transient and restore pressure forward" \
  --avoid "flat stutter,source copy" \
  --concrete-follow-up "resolve dense MC-202 queue entry" \
  --reviewer "fixture-listener" >/dev/null

python3 scripts/import_listening_review_label.py \
  --require-artifact-hashes \
  --json-output "$tmp/imported-label-corpus.json" \
  "$tmp/review.json" >/dev/null

python3 scripts/generate_mc202_producer_grade_closeout.py \
  --label-corpus "$tmp/imported-label-corpus.json" \
  --output "$tmp/closeout" \
  --date "local-mc202-closeout-label-corpus-fixture" >/dev/null

jq -e '
  .structured_listening_label_corpus.label_count == 1
  and .structured_listening_label_corpus.matched_label_count == 1
  and .structured_listening_label_corpus.resolved_queue_count == 1
  and any(.structured_listening_review_queue[];
    .case_id == "dense_beat03_130"
    and .listening_label_resolution.status == "resolved"
    and .listening_label_resolution.human_verdict == "pass"
    and .listening_label_resolution.quality_proof == false
    and .listening_label_resolution.automated_musical_approval == false
  )
  and any(.blockers[];
    .code == "structured_human_verdict_missing"
    and (.case_ids | index("tonal_rusharp_120"))
    and (.case_ids | index("sparse_kicksnr_120"))
    and (.case_ids | index("dense_beat03_130") | not)
  )
' "$tmp/closeout/mc202-producer-grade-closeout.json" >/dev/null

echo "MC-202 closeout label-corpus fixture ok"

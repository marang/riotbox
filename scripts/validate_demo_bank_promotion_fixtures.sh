#!/usr/bin/env bash
set -euo pipefail

repo="$(git rev-parse --show-toplevel)"
cd "$repo"

tmp="$(mktemp -d "$repo/artifacts/audio_qa/local-demo-bank-promotion-fixtures.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

python3 scripts/generate_professional_output_listening_pack.py \
  --output "$tmp/pack" \
  --date "local-demo-bank-promotion-fixtures" >/dev/null

pass_review="$tmp/pack/reviews/dense_beat03_130/review.json"
python3 scripts/listening_review_workflow.py record \
  --review "$pass_review" \
  --human-verdict keep \
  --strongest-element snare \
  --source-recognition source_transformed_but_present \
  --hook-after-two-bars clear \
  --preferred-direction "promote the dense break only after human pass" \
  --avoid "flat stutter,source copy" \
  --concrete-follow-up "promote source-backed human pass into demo bank" \
  --reviewer "fixture-listener" >/dev/null

python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$pass_review" \
  --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json \
  --json-output "$tmp/demo-bank-pass.json" \
  --entry-id dense-beat03-promoted-fixture \
  --demo-worthiness-note "Human pass confirms the dense break has a clear hook, pressure lift, destructive contrast, and replay value." \
  --require-artifact-hashes >/dev/null

jq -e '
  .schema == "riotbox.release_grade_demo_bank.v1"
  and any(.entries[];
    .entry_id == "dense-beat03-promoted-fixture"
    and .human_verdict == "pass"
    and .demo_readiness == "demo_ready"
    and (.rendered_wav.sha256 | length == 64)
    and (.metrics.sha256 | length == 64)
    and (.review_prompt.sha256 | length == 64)
    and (.fix_categories | length == 0)
  )
' "$tmp/demo-bank-pass.json" >/dev/null

weak_review="$tmp/pack/reviews/sparse_kicksnr_120/review.json"
python3 scripts/listening_review_workflow.py record \
  --review "$weak_review" \
  --human-verdict technically_ok_but_musically_weak \
  --strongest-element bass \
  --source-recognition source_transformed_but_present \
  --hook-after-two-bars weak \
  --failure-reason "Sparse pressure is useful but the hook and restore are not demo-ready." \
  --preferred-direction "make sparse bass pressure more playable before demo promotion" \
  --avoid "weak hook,soft restore" \
  --concrete-follow-up "route sparse weak output to bass movement" \
  --reviewer "fixture-listener" >/dev/null

python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$weak_review" \
  --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json \
  --json-output "$tmp/demo-bank-weak.json" \
  --entry-id sparse-kicksnr-promoted-weak-fixture \
  --demo-worthiness-note "Human weak review preserves the sparse example only as a concrete bass-movement fix target." \
  --fix-category bass_movement \
  --require-artifact-hashes >/dev/null

jq -e '
  any(.entries[];
    .entry_id == "sparse-kicksnr-promoted-weak-fixture"
    and .human_verdict == "weak"
    and .demo_readiness == "not_demo_ready"
    and (.fix_categories == ["bass_movement"])
  )
' "$tmp/demo-bank-weak.json" >/dev/null

unverified_review="$tmp/pack/reviews/tonal_rusharp_120/review.json"
if python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$unverified_review" \
  --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json \
  --json-output "$tmp/demo-bank-unverified.json" \
  --entry-id tonal-rusharp-unverified-fixture \
  --demo-worthiness-note "This should not promote." \
  --require-artifact-hashes >"$tmp/unverified.out" 2>&1; then
  cat "$tmp/unverified.out" >&2
  echo "expected unverified review promotion to fail" >&2
  exit 1
fi
grep -q "cannot promote human_verdict unverified" "$tmp/unverified.out"

stale_review="$tmp/stale-review.json"
jq '.audio_judge_label.artifact_identity.audio_sha256.full_performance = "0000000000000000000000000000000000000000000000000000000000000000"' \
  "$pass_review" > "$stale_review"
if python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$stale_review" \
  --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json \
  --json-output "$tmp/demo-bank-stale.json" \
  --entry-id dense-beat03-stale-fixture \
  --demo-worthiness-note "This should not promote." \
  --require-artifact-hashes >"$tmp/stale.out" 2>&1; then
  cat "$tmp/stale.out" >&2
  echo "expected stale artifact promotion to fail" >&2
  exit 1
fi
grep -q "stale artifact hash" "$tmp/stale.out"

echo "demo-bank promotion fixture gate ok"

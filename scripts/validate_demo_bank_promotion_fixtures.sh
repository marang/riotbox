#!/usr/bin/env bash
set -euo pipefail

repo="$(git rev-parse --show-toplevel)"
cd "$repo"

tmp="$(mktemp -d "$repo/artifacts/audio_qa/local-demo-bank-promotion-fixtures.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

python3 scripts/generate_professional_output_listening_pack.py \
  --output "$tmp/pack" \
  --date "local-demo-bank-promotion-fixtures" >/dev/null
python3 scripts/generate_mc202_real_source_listening_pack.py \
  --output "$tmp/real-source" \
  --date "local-demo-bank-promotion-fixtures-real-source" >/dev/null
python3 scripts/generate_mc202_producer_grade_closeout.py \
  --professional-pack "$tmp/pack/professional-output-listening-pack.json" \
  --real-source-pack "$tmp/real-source/mc202-real-source-listening-pack.json" \
  --output "$tmp/closeout" \
  --date "local-demo-bank-promotion-fixtures-closeout" >/dev/null
closeout="$tmp/closeout/mc202-producer-grade-closeout.json"

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
  --mc202-producer-closeout "$closeout" \
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
    and .mc202_source_composed_review_gate.source_composed_evidence == true
    and .mc202_source_composed_review_gate.primitive_or_template_only == false
    and .mc202_role_evidence.role == "pressure_answer"
    and .mc202_role_evidence.proof_scope == "demo_bank_promotion_gate"
    and .mc202_role_evidence.source_family == .source_family
    and .mc202_role_evidence.quality_proof == false
    and .demo_readiness_consequence == "human_pass_allows_demo_ready_candidate"
    and .mc202_producer_fix_routing.case_id == "dense_beat03_130"
    and .mc202_producer_fix_routing.demo_bank_fix_categories == []
    and .mc202_producer_fix_routing.quality_proof == false
  )
' "$tmp/demo-bank-pass.json" >/dev/null

invalid_demo_bank_role="$tmp/demo-bank-invalid-role.json"
jq '(.entries[] | select(.entry_id == "dense-beat03-promoted-fixture") | .mc202_role_evidence.role) = "bass_pressure"' \
  "$tmp/demo-bank-pass.json" > "$invalid_demo_bank_role"
if python3 scripts/validate_release_grade_demo_bank.py "$invalid_demo_bank_role" >"$tmp/invalid-demo-bank-role.out" 2>&1; then
  cat "$tmp/invalid-demo-bank-role.out" >&2
  echo "expected invalid demo-bank MC-202 role evidence to fail" >&2
  exit 1
fi
grep -q "dense MC-202 promotion needs pressure_answer role" "$tmp/invalid-demo-bank-role.out"

invalid_demo_bank_gate="$tmp/demo-bank-invalid-gate.json"
jq '(.entries[] | select(.entry_id == "dense-beat03-promoted-fixture") | .mc202_source_composed_review_gate.source_composed_evidence) = false
  | (.entries[] | select(.entry_id == "dense-beat03-promoted-fixture") | .mc202_source_composed_review_gate.primitive_or_template_only) = true' \
  "$tmp/demo-bank-pass.json" > "$invalid_demo_bank_gate"
if python3 scripts/validate_release_grade_demo_bank.py "$invalid_demo_bank_gate" >"$tmp/invalid-demo-bank-gate.out" 2>&1; then
  cat "$tmp/invalid-demo-bank-gate.out" >&2
  echo "expected invalid demo-bank MC-202 gate to fail" >&2
  exit 1
fi
grep -q "MC-202 source-composed evidence is required" "$tmp/invalid-demo-bank-gate.out"

weak_review="$tmp/pack/reviews/tonal_rusharp_120/review.json"
python3 scripts/listening_review_workflow.py record \
  --review "$weak_review" \
  --human-verdict technically_ok_but_musically_weak \
  --strongest-element stab \
  --source-recognition source_transformed_but_present \
  --hook-after-two-bars weak \
  --failure-reason "Tonal hook is useful but still needs a human listening decision before demo readiness." \
  --preferred-direction "keep the tonal hook clear and record a concrete listening verdict before promotion" \
  --avoid "buried answer,hook masking" \
  --concrete-follow-up "block weak tonal promotion until a concrete non-human producer fix category exists" \
  --reviewer "fixture-listener" >/dev/null

if python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$weak_review" \
  --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json \
  --json-output "$tmp/demo-bank-weak.json" \
  --entry-id tonal-rusharp-promoted-weak-fixture \
  --demo-worthiness-note "This should not promote without a concrete non-human producer fix category." \
  --mc202-producer-closeout "$closeout" \
  --require-artifact-hashes >"$tmp/demo-bank-weak.out" 2>&1; then
  cat "$tmp/demo-bank-weak.out" >&2
  echo "expected weak tonal promotion without concrete producer fix category to fail" >&2
  exit 1
fi
grep -q "MC-202 weak/fail verdict needs non-human producer fix categories" "$tmp/demo-bank-weak.out"

if python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$weak_review" \
  --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json \
  --json-output "$tmp/demo-bank-weak-manual-mismatch.json" \
  --entry-id tonal-rusharp-promoted-weak-mismatch-fixture \
  --demo-worthiness-note "This should not promote." \
  --fix-category bass_movement \
  --mc202-producer-closeout "$closeout" \
  --require-artifact-hashes >"$tmp/manual-mismatch.out" 2>&1; then
  cat "$tmp/manual-mismatch.out" >&2
  echo "expected manual MC-202 fix category mismatch to fail" >&2
  exit 1
fi
grep -q "MC-202 weak/fail verdict needs non-human producer fix categories" "$tmp/manual-mismatch.out"

stale_closeout="$tmp/stale-closeout.json"
jq '(.review_candidates[] | select(.case_id == "tonal_rusharp_120") | .candidate_sha256) = "0000000000000000000000000000000000000000000000000000000000000000"' \
  "$closeout" > "$stale_closeout"
if python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$weak_review" \
  --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json \
  --json-output "$tmp/demo-bank-weak-stale-closeout.json" \
  --entry-id tonal-rusharp-promoted-weak-stale-closeout-fixture \
  --demo-worthiness-note "This should not promote." \
  --mc202-producer-closeout "$stale_closeout" \
  --require-artifact-hashes >"$tmp/stale-closeout.out" 2>&1; then
  cat "$tmp/stale-closeout.out" >&2
  echo "expected stale MC-202 closeout hash to fail" >&2
  exit 1
fi
grep -q "MC-202 closeout candidate hash does not match reviewed WAV" "$tmp/stale-closeout.out"

unverified_review="$tmp/pack/reviews/sparse_kicksnr_120/review.json"
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
jq '.audio_judge_label.artifact_identity.audio_sha256.rebuild_only_performance = "0000000000000000000000000000000000000000000000000000000000000000"' \
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

template_only_review="$tmp/template-only-review.json"
jq '.audio_judge_label.mc202_source_composed_review_gate.source_composed_evidence = false
  | .audio_judge_label.mc202_source_composed_review_gate.primitive_or_template_only = true
  | .audio_judge_label.mc202_source_composed_review_gate.failure_codes += ["fixture_template_only"]' \
  "$pass_review" > "$template_only_review"
if python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$template_only_review" \
  --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json \
  --json-output "$tmp/demo-bank-template-only.json" \
  --entry-id dense-beat03-template-only-fixture \
  --demo-worthiness-note "This should not promote." \
  --require-artifact-hashes >"$tmp/template-only.out" 2>&1; then
  cat "$tmp/template-only.out" >&2
  echo "expected template-only MC-202 promotion to fail" >&2
  exit 1
fi
grep -q "MC-202 source-composed evidence is required" "$tmp/template-only.out"

missing_role_review="$tmp/missing-role-review.json"
jq 'del(.audio_judge_label.mc202_role_evidence)' "$pass_review" > "$missing_role_review"
if python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$missing_role_review" \
  --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json \
  --json-output "$tmp/demo-bank-missing-role.json" \
  --entry-id dense-beat03-missing-role-fixture \
  --demo-worthiness-note "This should not promote." \
  --require-artifact-hashes >"$tmp/missing-role.out" 2>&1; then
  cat "$tmp/missing-role.out" >&2
  echo "expected missing MC-202 role promotion to fail" >&2
  exit 1
fi
grep -q "missing mc202_role_evidence" "$tmp/missing-role.out"

stale_role_review="$tmp/stale-role-review.json"
jq '.audio_judge_label.mc202_role_evidence.source_family = "stale_family"' "$pass_review" > "$stale_role_review"
if python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$stale_role_review" \
  --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json \
  --json-output "$tmp/demo-bank-stale-role.json" \
  --entry-id dense-beat03-stale-role-fixture \
  --demo-worthiness-note "This should not promote." \
  --require-artifact-hashes >"$tmp/stale-role.out" 2>&1; then
  cat "$tmp/stale-role.out" >&2
  echo "expected stale MC-202 role promotion to fail" >&2
  exit 1
fi
grep -q "MC-202 role source_family mismatch" "$tmp/stale-role.out"

wrong_role_review="$tmp/wrong-role-review.json"
jq '.audio_judge_label.mc202_role_evidence.role = "bass_pressure"' "$weak_review" > "$wrong_role_review"
if python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$wrong_role_review" \
  --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json \
  --json-output "$tmp/demo-bank-wrong-role.json" \
  --entry-id tonal-rusharp-wrong-role-fixture \
  --demo-worthiness-note "This should not promote." \
  --fix-category mix_bus \
  --require-artifact-hashes >"$tmp/wrong-role.out" 2>&1; then
  cat "$tmp/wrong-role.out" >&2
  echo "expected wrong MC-202 role promotion to fail" >&2
  exit 1
fi
grep -q "tonal MC-202 promotion needs answer/stab role" "$tmp/wrong-role.out"

echo "demo-bank promotion fixture gate ok"

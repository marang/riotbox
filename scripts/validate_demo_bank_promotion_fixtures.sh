#!/usr/bin/env bash
set -euo pipefail

repo="$(git rev-parse --show-toplevel)"
cd "$repo"

tmp="$(mktemp -d "$repo/artifacts/audio_qa/local-demo-bank-promotion-fixtures.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

render_dir="$tmp/pack/renders/dense_beat03_130"
dense_review_dir="$tmp/pack/reviews/dense_beat03_130"
tonal_review_dir="$tmp/pack/reviews/tonal_rusharp_120"
sparse_review_dir="$tmp/pack/reviews/sparse_kicksnr_120"
mkdir -p "$render_dir" "$dense_review_dir" "$tonal_review_dir" "$sparse_review_dir" "$tmp/closeout"

jq -n '{schema:"riotbox.demo_bank_promotion_fixture.performance_report.v1"}' >"$render_dir/performance-report.json"
jq -n '{schema:"riotbox.demo_bank_promotion_fixture.agent_review.v1"}' >"$render_dir/agent-review.json"
printf 'fixture-source-window\n' >"$render_dir/00_source_window.wav"
printf 'fixture-rebuild-only-performance\n' >"$render_dir/05_rebuild_only_performance.wav"
printf '# Fixture listening prompt\n' >"$dense_review_dir/prompt.md"
jq -n '{schema:"riotbox.listening_review.metrics.v1",schema_version:1}' >"$dense_review_dir/metrics.json"

performance_hash="$(sha256sum "$render_dir/performance-report.json" | cut -d ' ' -f1)"
agent_hash="$(sha256sum "$render_dir/agent-review.json" | cut -d ' ' -f1)"
source_hash="$(sha256sum "$render_dir/00_source_window.wav" | cut -d ' ' -f1)"
candidate_hash="$(sha256sum "$render_dir/05_rebuild_only_performance.wav" | cut -d ' ' -f1)"

pass_review="$dense_review_dir/review.json"
jq -n \
  --arg report "$render_dir/performance-report.json" \
  --arg report_hash "$performance_hash" \
  --arg agent "$render_dir/agent-review.json" \
  --arg agent_hash "$agent_hash" \
  --arg source "$render_dir/00_source_window.wav" \
  --arg source_hash "$source_hash" \
  --arg candidate "$render_dir/05_rebuild_only_performance.wav" \
  --arg candidate_hash "$candidate_hash" \
  '{
    schema:"riotbox.listening_review.v1",
    schema_version:1,
    ticket:"RIOTBOX-1197",
    technical_status:"pass",
    automated_musical_fitness_status:"pass",
    human_verdict:"unverified",
    strongest_element:"none",
    source_recognition:"unverified",
    hook_after_two_bars:"unverified",
    failure_reason:"",
    preferred_direction:"",
    avoid:[],
    concrete_follow_up:"",
    expected_audible_behavior:"Fixture-only promotion contract; no source audio is read.",
    artifacts:{
      candidate_audio:[$candidate],
      source_audio:"fixtures/dense-source.wav",
      metrics_json:"metrics.json",
      prompt_markdown:"prompt.md"
    },
    audio_judge_label:{
      created_at:"fixture",
      source_family:"dense_break",
      source_id:"dense_beat03_130",
      review_pack_schema:"riotbox.demo_bank_promotion_fixture.v1",
      review_pack_id:"fixture:dense_beat03_130",
      artifact_identity:{
        performance_report_sha256:$report_hash,
        agent_review_sha256:$agent_hash,
        audio_sha256:{source_window:$source_hash,rebuild_only_performance:$candidate_hash}
      },
      artifact_paths:{
        performance_report:$report,
        agent_review:$agent,
        audio:{source_window:$source,rebuild_only_performance:$candidate}
      },
      reason_tags:{
        hook_clarity:"clear",
        hardest_hit:"break_transient",
        bass_pressure:"present",
        destructive_contrast:"strong",
        source_character:"source_transformed_but_present",
        replay_value_after_eight_bars:"high"
      },
      mc202_source_composed_review_gate:{
        schema:"riotbox.mc202_source_composed_review_gate.v1",
        result:"pass",
        source_family:"dense_break",
        family_kind:"dense_break",
        source_composed_evidence:true,
        primitive_or_template_only:false,
        quality_proof:false,
        human_verdict:"unverified",
        demo_readiness:"unverified",
        promotion_blocked_until_human_pass:true,
        template_only_blocks_promotion:true,
        failure_codes:[]
      },
      mc202_role_evidence:{
        schema:"riotbox.mc202_role_evidence.v1",
        source_family:"dense_break",
        role:"pressure_answer",
        result:"pass",
        proof_scope:"demo_bank_promotion_gate",
        source_derived:true,
        quality_proof:false,
        failure_codes:[],
        musician_reason:"Fixture-only source-composed role evidence."
      },
      summary:"Fixture-only dense promotion evidence."
    }
  }' >"$pass_review"

cp "$dense_review_dir/prompt.md" "$tonal_review_dir/prompt.md"
cp "$dense_review_dir/metrics.json" "$tonal_review_dir/metrics.json"
jq '
  .artifacts.source_audio = "fixtures/tonal-source.wav"
  | .audio_judge_label.source_family = "tonal_hook"
  | .audio_judge_label.source_id = "tonal_rusharp_120"
  | .audio_judge_label.mc202_source_composed_review_gate.source_family = "tonal_hook"
  | .audio_judge_label.mc202_source_composed_review_gate.family_kind = "non_dense_break"
  | .audio_judge_label.mc202_role_evidence.source_family = "tonal_hook"
  | .audio_judge_label.mc202_role_evidence.role = "hook_restraint_stab_answer"
' "$pass_review" >"$tonal_review_dir/review.json"

cp "$dense_review_dir/prompt.md" "$sparse_review_dir/prompt.md"
cp "$dense_review_dir/metrics.json" "$sparse_review_dir/metrics.json"
cp "$pass_review" "$sparse_review_dir/review.json"

closeout="$tmp/closeout/mc202-producer-grade-closeout.json"
jq -n \
  --arg candidate "$render_dir/05_rebuild_only_performance.wav" \
  --arg candidate_hash "$candidate_hash" \
  '{
    schema:"riotbox.mc202_producer_grade_closeout.v1",
    quality_proof:false,
    automated_musical_approval:false,
    review_candidates:[
      {
        case_id:"dense_beat03_130",
        source_family:"dense_break",
        candidate:$candidate,
        candidate_sha256:$candidate_hash,
        mc202_producer_fix_route:{
          proposed_fix_categories:["human_listening"],
          quality_proof:false,
          automated_musical_approval:false
        }
      },
      {
        case_id:"tonal_rusharp_120",
        source_family:"tonal_hook",
        candidate:$candidate,
        candidate_sha256:$candidate_hash,
        mc202_producer_fix_route:{
          proposed_fix_categories:["human_listening"],
          quality_proof:false,
          automated_musical_approval:false
        }
      }
    ],
    mc202_producer_fix_candidates:[
      {category:"human_listening",case_ids:["dense_beat03_130","tonal_rusharp_120"]}
    ]
  }' >"$closeout"

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
    and .human_review_evidence.schema == "riotbox.demo_bank_human_review_evidence.v1"
    and .human_review_evidence.reviewer == "fixture-listener"
    and .human_review_evidence.reviewer_kind == "fixture_calibration"
    and (.human_review_evidence.review_path | endswith("review.json"))
    and (.human_review_evidence.review_sha256 | length == 64)
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

jq '.reviewer = "Markus"' "$pass_review" > "$tmp/live-review.tmp"
mv "$tmp/live-review.tmp" "$pass_review"
live_review="$pass_review"
jq -n '{
  schema: "riotbox.release_grade_demo_bank.v1",
  schema_version: 1,
  evidence_role: "live_review",
  readiness_rubric_schema: "riotbox.sound_product_readiness_rubric.v1",
  hidden_taste_oracle_allowed: false,
  entries: []
}' > "$tmp/live-bank-empty.json"
python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$live_review" \
  --demo-bank "$tmp/live-bank-empty.json" \
  --json-output "$tmp/live-bank-promoted.json" \
  --entry-id dense-beat03-real-human-pass \
  --demo-worthiness-note "Real structured human pass remains eligible for live readiness." \
  --mc202-producer-closeout "$closeout" \
  --require-artifact-hashes >/dev/null
jq -e '
  .evidence_role == "live_review"
  and (.entries | length) == 1
  and .entries[0].human_review_evidence.reviewer == "Markus"
  and .entries[0].human_review_evidence.reviewer_kind == "human"
  and (.entries[0].human_review_evidence.review_sha256 | length) == 64
' "$tmp/live-bank-promoted.json" >/dev/null

exact_product_review="$tmp/pack/reviews/dense_beat03_130/exact-product-review.json"
jq '
  del(.audio_judge_label.mc202_source_composed_review_gate)
  | del(.audio_judge_label.mc202_role_evidence)
  | del(.audio_judge_label.artifact_identity.agent_review_sha256)
  | del(.audio_judge_label.artifact_paths.agent_review)
  | .audio_judge_label.exact_product_path_review_gate = {
      schema: "riotbox.exact_product_path_review_gate.v1",
      result: "pass",
      source_family: "dense_break",
      product_path_kind: "exact_runtime_mix_live_journey",
      source_backed: true,
      source_timing_backed: true,
      source_graph_capture_lineage_proven: true,
      action_lexicon_queue_commit_proven: true,
      session_replay_proven: true,
      callback_partitions_sample_exact: true,
      restart_recall_sample_exact: true,
      source_role_decision_proven: true,
      scripted_performer_driver: true,
      hardcoded_musical_output: false,
      primitive_or_template_only: false,
      fallback_music_present: false,
      quality_proof: false,
      human_verdict: "unverified",
      promotion_blocked_until_human_pass: true,
      failure_codes: []
    }
' "$live_review" > "$exact_product_review"
python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$exact_product_review" \
  --demo-bank "$tmp/live-bank-empty.json" \
  --json-output "$tmp/live-bank-exact-product.json" \
  --entry-id dense-beat03-exact-product-pass \
  --demo-worthiness-note "Exact RuntimeMix product journey earned a structured human pass." \
  --require-artifact-hashes >/dev/null
jq -e '
  .entries[0].exact_product_path_review_gate.result == "pass"
  and .entries[0].exact_product_path_review_gate.product_path_kind == "exact_runtime_mix_live_journey"
  and .entries[0].exact_product_path_review_gate.scripted_performer_driver == true
  and .entries[0].exact_product_path_review_gate.hardcoded_musical_output == false
  and (.entries[0] | has("mc202_source_composed_review_gate") | not)
' "$tmp/live-bank-exact-product.json" >/dev/null

sparse_exact_product_review="$tmp/pack/reviews/dense_beat03_130/sparse-exact-product-review.json"
jq '
  .audio_judge_label.source_family = "sparse_drums"
  | .audio_judge_label.exact_product_path_review_gate.schema = "riotbox.exact_product_path_review_gate.v2"
  | .audio_judge_label.exact_product_path_review_gate.source_family = "sparse_drums"
  | .audio_judge_label.exact_product_path_review_gate.active_contributors_sample_exact = true
  | .audio_judge_label.exact_product_path_review_gate.unassigned_role_not_claimed = true
  | .audio_judge_label.exact_product_path_review_gate.lane_roles = {
      w30: "source_transform",
      tr909: "hardest_transient",
      mc202: "punctuation",
      source_monitor: "stay_out",
      bass_owner: "unassigned"
    }
' "$exact_product_review" > "$sparse_exact_product_review"
python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$sparse_exact_product_review" \
  --demo-bank "$tmp/live-bank-empty.json" \
  --json-output "$tmp/live-bank-sparse-exact-product.json" \
  --entry-id sparse-exact-product-pass \
  --demo-worthiness-note "Exact sparse RuntimeMix journey earned a structured human pass without a bass-pressure claim." \
  --require-artifact-hashes >/dev/null
jq -e '
  .entries[0].source_family == "sparse_drums"
  and .entries[0].exact_product_path_review_gate.schema == "riotbox.exact_product_path_review_gate.v2"
  and .entries[0].exact_product_path_review_gate.lane_roles.mc202 == "punctuation"
  and .entries[0].exact_product_path_review_gate.lane_roles.bass_owner == "unassigned"
' "$tmp/live-bank-sparse-exact-product.json" >/dev/null

invalid_sparse_roles="$tmp/pack/reviews/dense_beat03_130/invalid-sparse-exact-product-review.json"
jq '.audio_judge_label.exact_product_path_review_gate.lane_roles.bass_owner = "mc202"' \
  "$sparse_exact_product_review" > "$invalid_sparse_roles"
if python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$invalid_sparse_roles" \
  --demo-bank "$tmp/live-bank-empty.json" \
  --json-output "$tmp/live-bank-invalid-sparse-exact-product.json" \
  --entry-id sparse-invalid-exact-product \
  --demo-worthiness-note "This should not promote." \
  --require-artifact-hashes >"$tmp/invalid-sparse-exact-product.out" 2>&1; then
  cat "$tmp/invalid-sparse-exact-product.out" >&2
  echo "expected invalid sparse exact-product lane roles to fail" >&2
  exit 1
fi
grep -q "sparse lane roles changed" "$tmp/invalid-sparse-exact-product.out"

python3 - <<'PY'
from copy import deepcopy
from pathlib import Path

from scripts.hash_identical_human_verdict_reuse import validate_reuse_evidence

evidence = {
    "schema": "riotbox.hash_identical_human_verdict_reuse.v1",
    "result": "pass",
    "reuse_contract": {
        "path": "docs/benchmarks/tonal_riff_release_demo_evidence_reuse_v2.json",
        "sha256": "cfdab651ceae05a494ccee5637a5e4fc3fb47bef24901b4ca5e76531a402cfa0",
    },
    "prior_ticket": "RIOTBOX-1454",
    "prior_structured_review_sha256": "8c67d9a45c21e0e061906e1310c2fc64f790c9590aba4e3f51e687420c5365ea",
    "current_replay_created_new_verdict": False,
    "new_quality_evidence": False,
}
dimensions = {
    "strongest_element": "chop",
    "source_recognition": "source_transformed_but_present",
    "hook_after_two_bars": "clear",
}
audio_sha256 = "24eca9572537d81d6ed87c61c13806a0c679092d8f8f73723e2015bfff490e6b"
manifest_sha256 = "28a95aae429361de50b3590e0feabf99f4426e35e1bdb43a818c006a2fe0b27d"

def validate(value, verdict_dimensions=dimensions):
    validate_reuse_evidence(
        value,
        Path("reuse-fixture"),
        current_audio_sha256=audio_sha256,
        current_product_manifest_sha256=manifest_sha256,
        expected_prior_human_verdict="keep",
        current_verdict_dimensions=verdict_dimensions,
    )

validate(evidence)
sparse_evidence = {
    "schema": "riotbox.hash_identical_human_verdict_reuse.v1",
    "result": "pass",
    "reuse_contract": {
        "path": "docs/benchmarks/sparse_drums_release_demo_evidence_reuse_v1.json",
        "sha256": "d0a658f12e75366d0243a230ddbb28af85746e0c7a5c601d3271b81ee5ed46c5",
    },
    "prior_ticket": "RIOTBOX-1455",
    "prior_structured_review_sha256": "7091d1699500857e5cde043fba0930409ede3848d170f999597507f20bd30184",
    "current_replay_created_new_verdict": False,
    "new_quality_evidence": False,
}
validate_reuse_evidence(
    sparse_evidence,
    Path("sparse-reuse-fixture"),
    current_audio_sha256="64bb983b5fccdeced71b03c8d07bd031726a52995a60a6a89aeab8cda8f1c69d",
    current_product_manifest_sha256="0d8359819210acd99cc2f49aeef999e80adca5fd9ef1d41f7994624c83fbc80d",
    expected_prior_human_verdict="keep",
    current_verdict_dimensions=dimensions,
)
mutations = []
replay_claim = deepcopy(evidence)
replay_claim["current_replay_created_new_verdict"] = True
mutations.append((replay_claim, dimensions))
contract_drift = deepcopy(evidence)
contract_drift["reuse_contract"]["sha256"] = "0" * 64
mutations.append((contract_drift, dimensions))
changed_dimensions = dict(dimensions)
changed_dimensions["strongest_element"] = "bass"
mutations.append((evidence, changed_dimensions))
for value, verdict_dimensions in mutations:
    try:
        validate(value, verdict_dimensions)
    except ValueError:
        continue
    raise SystemExit("expected hash-identical verdict reuse mutation to fail")
PY

invalid_exact_product_review="$tmp/pack/reviews/dense_beat03_130/invalid-exact-product-review.json"
jq '.audio_judge_label.exact_product_path_review_gate.callback_partitions_sample_exact = false' \
  "$exact_product_review" > "$invalid_exact_product_review"
if python3 scripts/promote_listening_review_to_demo_bank.py \
  --review "$invalid_exact_product_review" \
  --demo-bank "$tmp/live-bank-empty.json" \
  --json-output "$tmp/live-bank-invalid-exact-product.json" \
  --entry-id dense-beat03-invalid-exact-product \
  --demo-worthiness-note "This should not promote." \
  --require-artifact-hashes >"$tmp/invalid-exact-product.out" 2>&1; then
  cat "$tmp/invalid-exact-product.out" >&2
  echo "expected invalid exact product-path gate to fail" >&2
  exit 1
fi
grep -q "callback_partitions_sample_exact must be true" "$tmp/invalid-exact-product.out"

invalid_exact_demo_bank="$tmp/demo-bank-invalid-exact-product.json"
jq '.entries[0].exact_product_path_review_gate.fallback_music_present = true' \
  "$tmp/live-bank-exact-product.json" > "$invalid_exact_demo_bank"
if python3 scripts/validate_release_grade_demo_bank.py "$invalid_exact_demo_bank" >"$tmp/invalid-exact-demo-bank.out" 2>&1; then
  cat "$tmp/invalid-exact-demo-bank.out" >&2
  echo "expected invalid exact product-path demo-bank evidence to fail" >&2
  exit 1
fi
grep -q "fallback_music_present must be false" "$tmp/invalid-exact-demo-bank.out"

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

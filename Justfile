default:
    just --list

fmt:
    cargo fmt

test:
    cargo test

check:
    cargo check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

ci:
    cargo fmt --check
    cargo test
    just sidecar-contract-fixtures
    just source-timing-fixture-catalog-validator-fixtures
    just source-timing-analyzer-skeleton-fixtures
    just source-timing-fixture-evaluator
    just source-timing-fixture-report-smoke
    just source-timing-fixture-report-markdown-smoke
    just source-timing-fixture-report-json-validator-fixtures
    just source-timing-wav-probe
    just source-timing-bpm-candidates
    just source-timing-beat-evidence
    just source-timing-downbeat-evidence
    just source-timing-readiness-report
    just source-timing-downbeat-ambiguity
    just source-timing-drift-report
    just source-timing-phrase-grid
    just source-timing-candidate-confidence-report
    just source-timing-example-probe-report-fixtures
    just p011-replay-family-manifest
    just p011-exit-evidence-manifest
    just p011-exit-evidence-manifest-validator-fixtures
    just p011-exit-evidence-category-gate-fixtures
    just p011-exit-evidence-gate
    just audio-qa-lock-fixtures
    just audio-qa-ci
    cargo clippy --all-targets --all-features -- -D warnings

sidecar-contract-fixtures:
    python3 -m unittest discover -s python/sidecar -p 'test_*.py'

audio-qa-ci:
    scripts/with_audio_qa_lock.sh broad-audio-qa just _audio-qa-ci-unlocked

_audio-qa-ci-unlocked:
    cargo test -p riotbox-audio --bin w30_preview_render --bin w30_preview_compare --bin lane_recipe_pack --bin feral_before_after_pack --bin feral_grid_pack
    cargo test -p riotbox-app --bin observer_audio_correlate
    just observer-audio-correlate-fixture
    just observer-audio-correlate-json-fixture
    just observer-audio-correlate-locked-grid-json-fixture
    just observer-audio-summary-validator-fixtures
    just user-session-observer-validator-fixtures
    just degraded-product-review-fixtures
    just feral-break-live-review-timing-fixtures
    just source-timing-probe-json-validator-fixtures
    just source-timing-grid-use-contract-fixtures
    just recipe15-strict-missing-fixture-fixture
    just generated-source-timing-probe-json-smoke
    just generated-degraded-source-timing-probe-json-smoke
    just generated-ambiguous-source-timing-probe-json-smoke
    just listening-manifest-validator-fixtures
    just source-showcase-diversity-validator-fixtures
    just source-showcase-diversity-report-fixtures
    just sound-excellence-source-corpus-fixtures
    just source-holdout-rotation-fixtures
    just percussive-force-stage-a-v2-contract-fixtures
    just percussive-force-stage-a-v3-contract-fixtures
    just percussive-force-stage-a-v4-contract-fixtures
    just percussive-force-stage-a-v5-contract-fixtures
    just percussive-force-stage-a-v5-qualification-contracts
    just percussive-force-stage-a-matrix-v6-v2-contracts
    just representative-source-showcase-output-guard-fixtures
    just representative-source-showcase-musical-quality-fixtures
    just automated-musical-fitness-fixtures
    just agent-musical-review-pack-smoke
    just dense-break-live-path-smoke
    just dense-break-foundation-chop-exploration-fixtures
    just dense-break-foundation-event-context-v3-fixtures
    just dense-break-source-native-bar-v1-fixtures
    just pro-pressure-source-matrix-smoke
    just professional-source-wav-pack-smoke
    just edge-source-professional-diagnostics-smoke
    just non-dense-professional-proof-pack-smoke
    just professional-output-listening-pack-smoke
    just mc202-real-source-listening-pack-smoke
    just mc202-producer-grade-closeout-smoke
    just professional-output-listening-verdict-import-fixtures
    just destructive-variation-professional-smoke
    just rendered-weak-professional-output-fixtures
    just weak-output-fix-routing-fixtures
    just professional-output-suite-smoke
    just source-family-release-demo-coverage-fixtures
    just release-demo-human-review-queue-fixtures
    just release-demo-listening-review-packs-fixtures
    just sound-quality-readiness-report-smoke
    just sparse-bass-pressure-professional-fixtures
    just tonal-hook-professional-fixtures
    just human-listening-label-corpus-fixtures
    just listening-review-label-import-fixtures
    just audio-judge-spike-fixtures
    just audio-judge-spike-generated-smoke
    just musical-pass-gate-policy-fixtures
    just sound-product-readiness-rubric-fixtures
    just release-grade-demo-bank-fixtures
    just demo-bank-promotion-fixtures
    just sound-product-2010-future-ideas-fixtures
    just listening-manifest-validate-generated-packs
    just syncopated-source-showcase-smoke
    just w30-smoke-generated-source-diff
    just observer-audio-correlate-generated-feral-grid
    scripts/validate_first_playable_jam_probe.sh --exact-mix-dir artifacts/audio_qa/local-dense-break-live-path-smoke
    just source-timing-confirmation-probe
    just source-transport-map-capture-probe
    just p014-scene-movement-observer-probe
    just stage-style-jam-probe
    just stage-style-restore-diversity-probe
    just stage-style-snapshot-convergence-smoke
    just interrupted-session-recovery-probe
    just missing-target-recovery-probe
    just stage-style-stability-smoke
    just recipe2-observer-audio-gate
    just offline-render-reproducibility-smoke
    just full-grid-export-reproducibility-smoke

audio-qa-lock-fixtures:
    scripts/validate_audio_qa_lock_fixtures.sh

decision-search query:
    ./scripts/research_decision_search.sh {{quote(query)}}

decision-search-fixtures:
    #!/usr/bin/env bash
    set -euo pipefail
    assert_exact_block() {
      local id="$1" output heading_count
      output="$(./scripts/research_decision_search.sh "$id")"
      heading_count="$(printf '%s\n' "$output" | rg -c '^### RBX-[0-9]+[A-Za-z]?$')"
      test "$heading_count" -eq 1
      printf '%s\n' "$output" | rg -q "^### ${id}$"
      printf '%s\n' "$output" | rg -q '^Status:'
      if printf '%s\n' "$output" | rg -q '^---$'; then
        echo "exact decision lookup crossed a record delimiter: $id" >&2
        return 1
      fi
    }
    assert_exact_block RBX-015
    assert_exact_block RBX-015a
    assert_exact_block RBX-025
    assert_exact_block RBX-252
    assert_exact_block RBX-264
    if ./scripts/research_decision_search.sh RBX-999999 >/dev/null 2>&1; then
      echo 'missing decision ID unexpectedly succeeded' >&2
      exit 1
    fi
    if ./scripts/research_decision_search.sh 'riotbox_no_match_a91f2c7e *' >/dev/null 2>&1; then
      echo 'wildcard query unexpectedly expanded or matched' >&2
      exit 1
    fi
    injection_query='missing"; printf "\nJUST_INJECTION_EXECUTED\n"; #'
    injection_output="$(just decision-search "$injection_query" 2>&1 || true)"
    if printf '%s\n' "$injection_output" | rg -q '^JUST_INJECTION_EXECUTED$'; then
      echo 'decision-search query escaped its argument boundary' >&2
      exit 1
    fi

source-timing-fixture-catalog catalog="crates/riotbox-core/tests/fixtures/source_timing/timing_fixture_catalog.json":
    python3 scripts/validate_source_timing_fixture_catalog.py "{{catalog}}"

source-timing-fixture-catalog-validator-fixtures:
    python3 scripts/validate_source_timing_fixture_catalog.py crates/riotbox-core/tests/fixtures/source_timing/timing_fixture_catalog.json
    if python3 scripts/validate_source_timing_fixture_catalog.py crates/riotbox-core/tests/fixtures/source_timing/timing_fixture_catalog_invalid_empty_cases.json; then echo "expected empty timing fixture catalog to fail" >&2; exit 1; fi
    python3 scripts/validate_source_timing_fixture_catalog_label_fixtures.py

source-timing-analyzer-skeleton-fixtures:
    cargo test -p riotbox-core source_timing_fixture_catalog_maps_to_core_timing_contract -- --nocapture

source-timing-fixture-evaluator:
    cargo test -p riotbox-core source_timing_fixture -- --nocapture

source-timing-fixture-report catalog="crates/riotbox-core/tests/fixtures/source_timing/timing_fixture_catalog.json":
    cargo run -p riotbox-core --bin source_timing_fixture_report -- --catalog "{{catalog}}"

source-timing-fixture-report-smoke:
    tmp="$(mktemp)" && cargo run -p riotbox-core --bin source_timing_fixture_report -- --catalog crates/riotbox-core/tests/fixtures/source_timing/timing_fixture_catalog.json > "$tmp" && jq -e '.schema == "riotbox.source_timing_fixture_evaluation_report.v1" and .schema_version == 1 and .passed == true and (.evaluations | length) >= 7 and (.category_coverage | length) >= 7 and any(.category_coverage[]; .category == "high_drift" and .case_count == 1 and .passed == true) and (.evaluations[0].fixture_id == "fx_timing_clean_128_4x4") and (.evaluations[0].primary_confidence | type == "number") and (.evaluations[0].primary_max_drift_ms | type == "number") and (.evaluations[0].issues | type == "array")' "$tmp" && rm "$tmp"

source-timing-fixture-report-markdown-smoke:
    tmp="$(mktemp -d)" && cargo run -p riotbox-core --bin source_timing_fixture_report -- --catalog crates/riotbox-core/tests/fixtures/source_timing/timing_fixture_catalog.json --markdown-output "$tmp/report.md" > "$tmp/report.json" && jq -e '.schema == "riotbox.source_timing_fixture_evaluation_report.v1" and .passed == true and any(.category_coverage[]; .category == "high_drift")' "$tmp/report.json" && grep -q "Source Timing Fixture Evaluation Report" "$tmp/report.md" && grep -q "Category Coverage" "$tmp/report.md" && grep -q "high_drift" "$tmp/report.md" && grep -q "fx_timing_clean_128_4x4" "$tmp/report.md" && rm -rf "$tmp"

source-timing-fixture-report-json-validator-fixtures:
    python3 scripts/validate_source_timing_fixture_report_json.py crates/riotbox-core/tests/fixtures/source_timing_fixture_report/report_valid.json
    python3 scripts/validate_source_timing_fixture_report_json.py crates/riotbox-core/tests/fixtures/source_timing_fixture_report/report_valid_failure.json
    python3 scripts/validate_source_timing_fixture_report_json.py crates/riotbox-core/tests/fixtures/source_timing_fixture_report/report_valid_object_issues.json
    tmp="$(mktemp)" && cargo run -p riotbox-core --bin source_timing_fixture_report -- --catalog crates/riotbox-core/tests/fixtures/source_timing/timing_fixture_catalog.json > "$tmp" && python3 scripts/validate_source_timing_fixture_report_json.py "$tmp" && rm "$tmp"
    if python3 scripts/validate_source_timing_fixture_report_json.py crates/riotbox-core/tests/fixtures/source_timing_fixture_report/report_invalid_schema.json; then echo "expected invalid fixture report schema fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_source_timing_fixture_report_json.py crates/riotbox-core/tests/fixtures/source_timing_fixture_report/report_invalid_case_count.json; then echo "expected invalid fixture report case-count fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_source_timing_fixture_report_json.py crates/riotbox-core/tests/fixtures/source_timing_fixture_report/report_invalid_category_coverage_count.json; then echo "expected invalid fixture report category-coverage fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_source_timing_fixture_report_json.py crates/riotbox-core/tests/fixtures/source_timing_fixture_report/report_invalid_missing_measurement.json; then echo "expected invalid fixture report missing-measurement fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_source_timing_fixture_report_json.py crates/riotbox-core/tests/fixtures/source_timing_fixture_report/report_invalid_passed_consistency.json; then echo "expected invalid fixture report passed-consistency fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_source_timing_fixture_report_json.py crates/riotbox-core/tests/fixtures/source_timing_fixture_report/report_invalid_issue.json; then echo "expected invalid fixture report issue fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_source_timing_fixture_report_json.py crates/riotbox-core/tests/fixtures/source_timing_fixture_report/report_invalid_object_issue_value.json; then echo "expected invalid fixture report object-issue fixture to fail" >&2; exit 1; fi

source-timing-wav-probe:
    cargo test -p riotbox-core source_timing_probe_diagnostics -- --nocapture
    cargo test -p riotbox-audio source_timing_probe -- --nocapture

source-timing-bpm-candidates:
    cargo test -p riotbox-core source_timing_probe_bpm_candidates -- --nocapture
    cargo test -p riotbox-audio source_timing_probe_detects_impulse_onsets_from_pcm_wav_cache -- --nocapture
    cargo test -p riotbox-audio source_timing_probe_candidate_fixture_seed_scores_pcm_wav_grid -- --nocapture
    cargo test -p riotbox-audio source_timing_probe_weights_pcm_wav_downbeat_accents -- --nocapture

source-timing-beat-evidence:
    cargo test -p riotbox-core source_timing_probe_beat_evidence_report -- --nocapture
    cargo test -p riotbox-audio source_timing_probe_candidate_confidence_reports_phrase_grid_for_long_accented_wav -- --nocapture

source-timing-downbeat-evidence:
    cargo test -p riotbox-core source_timing_probe_downbeat_evidence_report -- --nocapture

source-timing-readiness-report:
    cargo test -p riotbox-core source_timing_probe_readiness_report -- --nocapture
    cargo test -p riotbox-audio source_timing_probe_preserves_real_loop_like_weak_readiness -- --nocapture
    cargo test -p riotbox-audio source_timing_probe_keeps_flat_loop_degraded_for_dance_auto_readiness -- --nocapture
    cargo test -p riotbox-audio source_timing_probe_promotes_stable_long_real_loop_like_grid_to_ready -- --nocapture
    cargo test -p riotbox-audio source_timing_probe_keeps_short_real_loop_like_grid_in_review -- --nocapture

source-timing-downbeat-ambiguity:
    cargo test -p riotbox-core source_timing_probe_bpm_candidates_preserve_alternate_downbeat_phases -- --nocapture
    cargo test -p riotbox-core source_timing_probe_bpm_candidates_keep_primary_bar_grid_phase_when_clearer -- --nocapture

source-timing-drift-report:
    cargo test -p riotbox-core source_timing_probe_bpm_candidates_report_stable_grid_drift -- --nocapture
    cargo test -p riotbox-core source_timing_probe_bpm_candidates_report_long_grid_drift_windows -- --nocapture
    cargo test -p riotbox-core source_timing_probe_bpm_candidates_warn_when_grid_drift_is_high -- --nocapture

source-timing-phrase-grid:
    cargo test -p riotbox-core source_timing_probe_bpm_candidates_add_phrase_grid_when_bar_timing_is_stable -- --nocapture
    cargo test -p riotbox-core source_timing_probe_bpm_candidates_keep_phrase_uncertain_for_short_material -- --nocapture

source-timing-candidate-confidence-report:
    cargo test -p riotbox-core source_timing_candidate_confidence_report -- --nocapture

source-timing-example-probe-report output="artifacts/audio_qa/local/source_timing_example_probe_report.md":
    python3 scripts/source_timing_example_probe_report.py --output "{{output}}"

source-timing-example-probe-report-local output="artifacts/audio_qa/local/source_timing_example_probe_report.md":
    python3 scripts/source_timing_example_probe_report.py \
      --expectations scripts/fixtures/source_timing_example_probe_report/local_example_expectations.json \
      --output "{{output}}"

source-timing-example-probe-report-fixtures:
    python3 scripts/assert_source_timing_example_report_fixtures.py

w30-smoke-candidate date="local" duration="2.0":
    cargo run -p riotbox-audio --bin w30_preview_render -- --date "{{date}}" --role candidate --duration-seconds "{{duration}}"

w30-smoke-baseline date="local" duration="2.0":
    cargo run -p riotbox-audio --bin w30_preview_render -- --date "{{date}}" --role baseline --duration-seconds "{{duration}}"

w30-smoke-compare date="local":
    cargo run -p riotbox-audio --bin w30_preview_compare -- --date "{{date}}"

w30-smoke-qa date="local" duration="2.0":
    just w30-smoke-baseline "{{date}}" "{{duration}}"
    just w30-smoke-candidate "{{date}}" "{{duration}}"
    just w30-smoke-compare "{{date}}"

w30-smoke-source-candidate source date="local" start="0.0" source_duration="0.25" duration="2.0":
    cargo run -p riotbox-audio --bin w30_preview_render -- --date "{{date}}" --role candidate --duration-seconds "{{duration}}" --source "{{source}}" --source-start-seconds "{{start}}" --source-duration-seconds "{{source_duration}}"

w30-smoke-source-baseline source date="local" start="0.0" source_duration="0.25" duration="2.0":
    cargo run -p riotbox-audio --bin w30_preview_render -- --date "{{date}}" --role baseline --duration-seconds "{{duration}}" --source "{{source}}" --source-start-seconds "{{start}}" --source-duration-seconds "{{source_duration}}"

w30-smoke-source-qa source date="local" start="0.0" source_duration="0.25" duration="2.0":
    just w30-smoke-source-baseline "{{source}}" "{{date}}" "{{start}}" "{{source_duration}}" "{{duration}}"
    just w30-smoke-source-candidate "{{source}}" "{{date}}" "{{start}}" "{{source_duration}}" "{{duration}}"
    just w30-smoke-compare "{{date}}"

w30-smoke-source-diff source date="local-source-diff" start="0.0" source_duration="0.25" duration="2.0":
    just w30-smoke-baseline "{{date}}" "{{duration}}"
    just w30-smoke-source-candidate "{{source}}" "{{date}}" "{{start}}" "{{source_duration}}" "{{duration}}"
    cargo run -p riotbox-audio --bin w30_preview_compare -- --date "{{date}}" --min-rms-delta 0.001 --min-sum-delta 1.0 --max-active-samples-delta 200000 --max-peak-delta 1.0 --max-rms-delta 1.0 --max-sum-delta 1200.0

w30-smoke-generated-source-diff:
    scripts/validate_generated_w30_source_diff.sh

w30-pitch-dive-product-qualification session:
    python3 scripts/run_w30_pitch_dive_product_qualification.py --session "{{session}}"

w30-pitch-dive-product-review-artifact session:
    python3 scripts/run_w30_pitch_dive_product_qualification.py --session "{{session}}" --prepare-review-from-passed-session

w30-filter-slam-product-qualification session:
    python3 scripts/run_w30_filter_slam_product_qualification.py --session "{{session}}"

w30-filter-slam-product-review-artifact session:
    python3 scripts/run_w30_filter_slam_product_qualification.py --session "{{session}}" --prepare-review-from-passed-session

w30-gesture-vocabulary-qualification session:
    python3 scripts/run_w30_gesture_vocabulary_qualification.py --session "{{session}}"

w30-gesture-vocabulary-review-artifact session:
    python3 scripts/run_w30_gesture_vocabulary_qualification.py --session "{{session}}" --prepare-review-from-passed-session

lane-recipe-pack date="local" duration="2.0":
    cargo run -p riotbox-audio --bin lane_recipe_pack -- --date "{{date}}" --duration-seconds "{{duration}}"

feral-before-after source date="local" start="0.0" duration="2.0" source_window="1.0":
    cargo run -p riotbox-audio --bin feral_before_after_pack -- --source "{{source}}" --date "{{date}}" --source-start-seconds "{{start}}" --duration-seconds "{{duration}}" --source-window-seconds "{{source_window}}"

melodic-source-chop-showcase output="artifacts/audio_qa/local-melodic-source-chop-showcase" date="local-melodic-source-chop-showcase" source="data/test_audio/examples/DH_RushArp_120_A.wav" duration="2.0" source_window="1.0" start="0.0":
    scripts/generate_melodic_source_chop_showcase.sh "{{output}}" "{{date}}" "{{source}}" "{{duration}}" "{{source_window}}" "{{start}}"

diverse-test-source-wavs output="artifacts/audio_qa/local-diverse-test-sources" seconds="4.0":
    python3 scripts/write_diverse_test_source_wavs.py --output "{{output}}" --seconds "{{seconds}}"
    jq -e '.schema == "riotbox.diverse_test_source_wavs.v1" and .result == "pass" and .case_count >= 12 and ([.entries[].source_family] | unique | length) >= 12 and all(.entries[]; .quality_proof == false and (.path | endswith(".wav")))' "{{output}}/manifest.json"
    for wav in "{{output}}"/*.wav; do test -s "$wav"; done

feral-grid-pack source date="local" bpm="auto" bars="8" source_window="1.0" start="0.0":
    if [ "{{bpm}}" = "auto" ]; then \
        cargo run -p riotbox-audio --bin feral_grid_pack -- --source "{{source}}" --date "{{date}}" --bars "{{bars}}" --source-window-seconds "{{source_window}}" --source-start-seconds "{{start}}"; \
    else \
        cargo run -p riotbox-audio --bin feral_grid_pack -- --source "{{source}}" --date "{{date}}" --bpm "{{bpm}}" --bars "{{bars}}" --source-window-seconds "{{source_window}}" --source-start-seconds "{{start}}"; \
    fi

dense-break-performance-pack source="data/test_audio/examples/Beat03_130BPM(Full).wav" output="artifacts/audio_qa/local-dense-break-performance-pack" date="local-dense-break-performance-pack":
    python3 scripts/generate_dense_break_performance_pack.py --source "{{source}}" --output "{{output}}" --date "{{date}}"

dense-break-performance-pack-smoke output="artifacts/audio_qa/local-dense-break-performance-pack-smoke":
    python3 scripts/generate_dense_break_performance_pack.py --output "{{output}}" --date "local-dense-break-performance-pack-smoke"
    python3 scripts/generate_dense_break_performance_pack.py --validate-report "{{output}}/performance-report.json" --mutation-fixtures

dense-break-live-path-smoke output="artifacts/audio_qa/local-dense-break-live-path-smoke" bpm="132":
    scripts/validate_dense_break_live_path.sh "{{output}}" "{{bpm}}"

dense-break-foundation-chop-exploration-fixtures:
    python3 scripts/generate_dense_break_foundation_chop_exploration.py --fixtures

dense-break-foundation-clarity-v2-fixtures:
    python3 scripts/generate_dense_break_foundation_clarity_v2.py --fixtures

dense-break-foundation-event-context-v3-fixtures:
    python3 scripts/generate_dense_break_foundation_event_context_v3.py --fixtures

dense-break-source-native-bar-v1-fixtures:
    python3 scripts/validate_dense_break_source_native_bar_exploration.py
    cargo test -p riotbox-audio source_native_full_bar

dense-break-source-native-bar-v1 session:
    python3 scripts/run_dense_break_source_native_bar_exploration.py --session "{{session}}"

dense-break-source-native-bar-preflight-recovery session:
    python3 scripts/run_dense_break_source_native_bar_exploration.py --session "{{session}}" --recover-preflight

dense-break-live-source-matrix output="artifacts/audio_qa/local-dense-break-live-source-matrix":
    scripts/validate_dense_break_live_source_matrix.sh "{{output}}"

controlled-source-live-matrix output="artifacts/audio_qa/local-controlled-source-live-matrix":
    scripts/validate_controlled_source_live_matrix.sh "{{output}}"

tonal-hook-live-journey output="artifacts/audio_qa/local-tonal-hook-live-journey":
    scripts/validate_tonal_hook_live_journey.sh "{{output}}"

sparse-pressure-live-journey output="artifacts/audio_qa/local-sparse-pressure-live-journey":
    scripts/validate_sparse_pressure_live_journey.sh "{{output}}"

dense-break-weak-source-character-fixture-smoke output="artifacts/audio_qa/local-dense-break-weak-source-character-fixture":
    tmp="$(mktemp)" && if python3 scripts/generate_dense_break_performance_pack.py --output "{{output}}" --date "local-dense-break-weak-source-character-fixture" --weak-source-character-fixture >"$tmp" 2>&1; then cat "$tmp" >&2; rm "$tmp"; echo "expected weak source-character dense-break fixture to fail" >&2; exit 1; fi && grep -q "rebuild_only_source_character_not_surviving" "$tmp" && grep -q "rebuild_only_source_character_margin_too_low" "$tmp" && rm "$tmp"
    python3 scripts/generate_dense_break_performance_pack.py --validate-weak-source-character-report "{{output}}/performance-report.json"
    jq -e '.schema == "riotbox.dense_break_performance_pack.v1" and .result == "fail" and .agent_verdict == "agent_fail" and .human_verdict == "unverified" and .evidence_role == "diagnostic" and .quality_proof == false and (.failure_codes | index("rebuild_only_source_character_not_surviving")) and (.failure_codes | index("rebuild_only_source_character_margin_too_low")) and .proof.rebuild_only_source_character_survival_score < 0.70 and .proof.rebuild_only_source_character_survival_margin < 0.10 and .proof.rebuild_only_source_transient_retention < 0.45 and (.files.rebuild_only_performance | endswith(".wav"))' "{{output}}/performance-report.json"
    test -s "{{output}}/00_source_window.wav"; test -s "{{output}}/05_rebuild_only_performance.wav"; test -s "{{output}}/performance-report.json"

agent-musical-review-pack source="data/test_audio/examples/Beat03_130BPM(Full).wav" output="artifacts/audio_qa/local-agent-musical-review-pack" date="local-agent-musical-review-pack":
    just dense-break-performance-pack "{{source}}" "{{output}}" "{{date}}"

agent-musical-review-pack-smoke output="artifacts/audio_qa/local-agent-musical-review-pack-smoke":
    just dense-break-performance-pack-smoke "{{output}}"
    python3 scripts/generate_dense_break_performance_pack.py --validate-agent-review "{{output}}/agent-review.json" --require-visuals

pro-pressure-source-matrix-smoke output="artifacts/audio_qa/local-pro-pressure-source-matrix":
    python3 scripts/validate_pro_pressure_source_matrix.py --output "{{output}}" --date "local-pro-pressure-source-matrix"
    jq -e '.schema == "riotbox.pro_pressure_source_matrix.v1" and .result == "pass" and .agent_verdict == "agent_promising" and .human_verdict == "unverified" and .evidence_role == "diagnostic" and .source_backed == true and .source_timing_backed == true and .scripted_generation == true and .quality_proof == false and .case_count >= 4 and .passed_case_count == .case_count and (.cases | length) == .case_count and ([.cases[].pressure_lift_policy.source_family] | unique | length) >= 2 and .arrangement_summary.unique_role_order_signature_count >= 2 and (.arrangement_summary.failure_codes | length) == 0 and all(.cases[]; .result == "pass" and .human_verdict == "unverified" and .evidence_role == "diagnostic" and .source_backed == true and .source_timing_backed == true and .scripted_generation == true and .quality_proof == false and .pressure_lift_policy.source_aware == true and .arrangement_policy.source_aware == true and .arrangement_policy.source_family == .pressure_lift_policy.source_family and (.arrangement_policy.role_order_signature | type == "string") and (.arrangement_failure_codes | length) == 0 and .proof.pressure_lift_policy_decision_count >= 12 and .proof.arrangement_policy_decision_count >= 8 and .proof.arrangement_pressure_role_count >= 2 and .proof.arrangement_destructive_role_count >= 2 and .proof.arrangement_failure_count == 0 and .proof.pressure_lift_bar5_to_bar4_rms_ratio >= 1.02 and .proof.w30_to_source_rms_ratio >= 0.18 and .proof.pressure_to_hook_rms_ratio >= 1.30 and .proof.restore_to_pressure_rms_ratio >= 1.12 and .proof.rebuild_only_to_full_rms_ratio >= 0.42 and .proof.rebuild_only_to_source_rms_ratio >= 0.30 and .proof.rebuild_only_to_source_correlation <= 0.92 and .proof.source_on_to_rebuild_only_correlation <= 0.995 and .metrics.source_layered_reference_peak_abs <= 0.985 and .metrics.rebuild_only_performance_peak_abs <= 0.985)' "{{output}}/source-matrix-report.json"
    jq -e 'all(.cases[]; .proof.arrangement_role_order_source_derived == 1.0 and .proof.arrangement_role_candidate_count >= 6.0 and .proof.arrangement_scripted_role_distance >= 1.0)' "{{output}}/source-matrix-report.json"
    jq -e 'all(.cases[]; .proof.mix_treatment_source_derived == 1.0 and .proof.mix_treatment_candidate_count >= 6.0 and .proof.mix_treatment_fixed_distance >= 0.08 and .proof.mix_treatment_output_contrast_ratio >= 2.10)' "{{output}}/source-matrix-report.json"
    jq -e 'all(.cases[]; .proof.tail_shape_source_derived == 1.0 and .proof.tail_shape_candidate_count >= 6.0 and .proof.tail_shape_fixed_distance >= 0.20 and .proof.tail_shape_output_contrast_ratio >= 3.00)' "{{output}}/source-matrix-report.json"
    jq -e 'all(.cases[]; (.proof.strongest_audible_element as $e | ["kick","snare","bass","stab","silence","restore"] | index($e)) and .proof.strongest_audible_element_score >= 1.00 and .proof.strongest_audible_element_margin >= 0.05 and .proof.strongest_audible_element_candidate_count >= 5.0)' "{{output}}/source-matrix-report.json"
    jq -e 'all(.cases[]; .proof.rebuild_only_source_spectral_similarity >= 0.60 and .proof.rebuild_only_source_transient_retention >= 0.45 and .proof.rebuild_only_source_character_survival_score >= 0.70)' "{{output}}/source-matrix-report.json"
    jq -e 'any(.cases[]; .pressure_lift_policy.source_family == "dense_break" and .proof.hook_chop_selection_source_derived == 1.0 and .proof.hook_chop_static_distance_frames >= 256.0 and .proof.hook_chop_offset_distance_frames >= 512.0)' "{{output}}/source-matrix-report.json"
    jq -e 'any(.cases[]; .pressure_lift_policy.source_family == "dense_break" and .proof.w30_to_source_rms_ratio >= 0.22)' "{{output}}/source-matrix-report.json"
    jq -e 'any(.cases[]; .pressure_lift_policy.source_family == "dense_break" and .proof.hook_chop_riff_unique_source_offset_count >= 6.0 and .proof.hook_chop_riff_hit_count >= 10.0 and .proof.hook_chop_riff_velocity_span >= 0.25)' "{{output}}/source-matrix-report.json"
    jq -e 'any(.cases[]; .pressure_lift_policy.source_family == "dense_break" and .proof.hook_chop_source_character_score_floor >= 0.64 and .proof.hook_chop_source_character_score_span >= 0.10)' "{{output}}/source-matrix-report.json"
    jq -e 'any(.cases[]; .pressure_lift_policy.source_family == "dense_break" and .proof.hook_chop_response_delta_ratio >= 0.35 and .proof.hook_chop_response_correlation <= 0.92 and .proof.hook_chop_response_transient_ratio >= 0.58)' "{{output}}/source-matrix-report.json"
    jq -e 'any(.cases[]; .pressure_lift_policy.source_family == "dense_break" and .proof.destructive_gesture_source_derived == 1.0 and .proof.destructive_static_distance_frames >= 256.0 and .proof.destructive_offset_distance_frames >= 512.0)' "{{output}}/source-matrix-report.json"
    jq -e 'any(.cases[]; .pressure_lift_policy.source_family == "dense_break" and .proof.pressure_lift_bar5_to_bar4_rms_ratio >= 1.10)' "{{output}}/source-matrix-report.json"
    jq -e 'any(.cases[]; .pressure_lift_policy.source_family == "dense_break" and .proof.dense_answer_bite_source_derived == 1.0 and .proof.dense_answer_bite_scripted_role_distance >= 3.0 and .proof.dense_answer_bite_stab_score >= 1.65 and .proof.dense_answer_bite_stab_margin >= 0.15 and .proof.dense_answer_bite_pressure_snap_ratio >= 1.06 and .proof.dense_answer_bite_score >= 1.0)' "{{output}}/source-matrix-report.json"
    jq -e 'any(.cases[]; .pressure_lift_policy.source_family == "sparse_bass_pressure" and .proof.bass_movement_source_derived == 1.0 and .proof.sparse_bass_movement_static_distance_hz >= 1.75 and .proof.sparse_bass_movement_frequency_span_hz >= 17.0 and .proof.sparse_bass_movement_span_margin_hz >= 0.0 and .proof.pressure_low_band_lift_ratio >= 2.70 and .proof.sparse_pressure_low_band_share >= 0.36 and .proof.sparse_pressure_low_to_mid_ratio >= 2.45 and .proof.strongest_audible_element == "bass" and .proof.strongest_audible_element_margin >= 0.20)' "{{output}}/source-matrix-report.json"

professional-source-wav-pack-smoke output="artifacts/audio_qa/local-professional-source-wav-pack":
    python3 scripts/generate_professional_source_wav_pack.py --output "{{output}}" --date "local-professional-source-wav-pack"
    python3 scripts/generate_professional_source_wav_pack.py --validate-report "{{output}}/professional-source-wav-pack.json" --require-artifacts --mutation-fixtures

edge-source-professional-diagnostics-smoke output="artifacts/audio_qa/local-edge-source-professional-diagnostics":
    python3 scripts/generate_edge_source_professional_diagnostics.py --output "{{output}}" --date "local-edge-source-professional-diagnostics"
    python3 scripts/generate_edge_source_professional_diagnostics.py --validate-report "{{output}}/edge-source-professional-diagnostics.json" --require-artifacts --mutation-fixtures

non-dense-professional-proof-pack-smoke output="artifacts/audio_qa/local-non-dense-professional-proof-pack" source_wav="artifacts/audio_qa/local-professional-source-wav-pack":
    python3 scripts/generate_non_dense_professional_proof_pack.py --output "{{output}}" --professional-source-wav-pack "{{source_wav}}" --date "local-non-dense-professional-proof-pack"
    python3 scripts/generate_non_dense_professional_proof_pack.py --validate-report "{{output}}/non-dense-professional-proof-pack.json" --require-artifacts --mutation-fixtures

professional-output-listening-pack-smoke output="artifacts/audio_qa/local-professional-output-listening-pack":
    python3 scripts/generate_professional_output_listening_pack.py --output "{{output}}" --date "local-professional-output-listening-pack"
    python3 scripts/validate_professional_output_listening_pack.py --require-review-files --mutation-fixtures "{{output}}/professional-output-listening-pack.json"

mc202-real-source-listening-pack-smoke output="artifacts/audio_qa/local-mc202-real-source-listening-pack":
    python3 scripts/generate_mc202_real_source_listening_pack.py --output "{{output}}" --date "local-mc202-real-source-listening-pack" --mutation-fixtures
    python3 scripts/generate_mc202_real_source_listening_pack.py --validate-report "{{output}}/mc202-real-source-listening-pack.json"
    jq -e '.schema == "riotbox.mc202_real_source_listening_pack.v1" and .result == "pass" and .agent_verdict == "agent_promising" and .human_verdict == "unverified" and .demo_readiness == "unverified" and .quality_proof == false and .evidence_role == "listening_review_scaffold" and .source_backed == true and .source_timing_backed == true and .scripted_generation == true and .case_count >= 2 and .dense_case_count >= 1 and .non_dense_case_count >= 1 and all(.cases[]; .human_verdict == "unverified" and .quality_proof == false and .mc202_expression_summary.contour_origin == "source_derived_contour" and (.selected_motif.stem_rms > 0.0005) and .primitive_ab_control.control_kind == "primitive_renderer_non_product_control" and .primitive_ab_control.product_fallback_allowed == false and .primitive_ab_control.ab_delta_passed == true and (.artifacts.mc202_stem.sha256 | length == 64) and (.artifacts.generated_support_mix.sha256 | length == 64))' "{{output}}/mc202-real-source-listening-pack.json"
    for case in beat03_full_main_loop dh_beatc_kicksnr_main_loop dh_rusharp_main_loop; do test -s "{{output}}/cases/$case/review/review.json"; test -s "{{output}}/cases/$case/render/stems/03_mc202_bass_pressure.wav"; test -s "{{output}}/cases/$case/render/05_riotbox_generated_support_mix.wav"; done

mc202-producer-grade-closeout-smoke output="artifacts/audio_qa/local-mc202-producer-grade-closeout":
    just professional-output-listening-pack-smoke
    just mc202-real-source-listening-pack-smoke
    python3 scripts/generate_mc202_producer_grade_closeout.py --output "{{output}}" --date "local-mc202-producer-grade-closeout" --mutation-fixtures
    python3 scripts/generate_mc202_producer_grade_closeout.py --validate-report "{{output}}/mc202-producer-grade-closeout.json" --require-all-source-composed-candidates
    test -s "{{output}}/mc202-producer-grade-closeout.md"
    grep -q "MC-202 Producer-Grade Closeout" "{{output}}/mc202-producer-grade-closeout.md"
    scripts/validate_mc202_closeout_label_corpus_fixture.sh

sound-excellence-source-corpus-fixtures manifest="docs/benchmarks/sound_excellence_source_corpus_v1.json":
    python3 scripts/validate_sound_excellence_source_corpus.py "{{manifest}}"
    tmp="$(mktemp)" && jq 'del(.entries[0].target_review_questions)' "{{manifest}}" > "$tmp" && if python3 scripts/validate_sound_excellence_source_corpus.py "$tmp"; then echo "expected missing review questions fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.entries[0].source_family = "unknown_family"' "{{manifest}}" > "$tmp" && if python3 scripts/validate_sound_excellence_source_corpus.py "$tmp"; then echo "expected unsupported source family fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"

source-holdout-rotation-fixtures manifest="docs/benchmarks/source_holdout_rotation_v2.json":
    python3 scripts/validate_source_holdout_rotation.py "docs/benchmarks/source_holdout_rotation_v1.json"
    python3 scripts/validate_source_holdout_rotation_fixtures.py "docs/benchmarks/source_holdout_rotation_v1.json"
    python3 scripts/validate_source_holdout_rotation.py "{{manifest}}"
    python3 scripts/validate_source_holdout_rotation_fixtures.py "{{manifest}}"

source-holdout-rotation-local-files manifest="docs/benchmarks/source_holdout_rotation_v1.json":
    python3 scripts/validate_source_holdout_rotation.py --require-existing-source-files "{{manifest}}"

percussive-force-stage-a-protocol-validator protocol="docs/benchmarks/percussive_force_stage_a_protocol_v1.json" matrix="docs/benchmarks/percussive_force_development_matrix_v2.json" registry="docs/benchmarks/source_holdout_rotation_v2.json":
    test "{{protocol}}" = "docs/benchmarks/percussive_force_stage_a_protocol_v1.json" && test "{{matrix}}" = "docs/benchmarks/percussive_force_development_matrix_v2.json" && test "{{registry}}" = "docs/benchmarks/source_holdout_rotation_v2.json"
    python3 scripts/validate_percussive_force_stage_a_protocol.py

percussive-force-stage-a-protocol-validator-fixtures protocol="docs/benchmarks/percussive_force_stage_a_protocol_v1.json" matrix="docs/benchmarks/percussive_force_development_matrix_v2.json" registry="docs/benchmarks/source_holdout_rotation_v2.json":
    test "{{protocol}}" = "docs/benchmarks/percussive_force_stage_a_protocol_v1.json" && test "{{matrix}}" = "docs/benchmarks/percussive_force_development_matrix_v2.json" && test "{{registry}}" = "docs/benchmarks/source_holdout_rotation_v2.json"
    python3 scripts/validate_percussive_force_stage_a_protocol_fixtures.py

percussive-force-stage-a-qualification-fixtures:
    python3 scripts/percussive_force_stage_a_qualification_fixtures.py

percussive-force-stage-a-v2-contract-fixtures:
    python3 scripts/percussive_force_stage_a_v2_contract.py --check
    python3 scripts/validate_percussive_force_stage_a_protocol_v2.py
    python3 scripts/validate_percussive_force_stage_a_protocol_v2_fixtures.py
    python3 scripts/percussive_force_stage_a_analysis_fixtures.py
    python3 scripts/percussive_force_stage_a_qualification_fixtures.py

percussive-force-stage-a-v3-contract-fixtures:
    python3 scripts/validate_percussive_force_stage_a_protocol_v3.py --fixtures

percussive-force-stage-a-v4-contract-fixtures:
    python3 scripts/validate_percussive_force_stage_a_protocol_v4.py --fixtures

percussive-force-stage-a-v5-contract-fixtures:
    python3 scripts/validate_percussive_force_stage_a_protocol_v5.py --fixtures

percussive-force-stage-a-v5-qualification-contracts:
    python3 scripts/run_percussive_force_stage_a_v5_qualification.py --validate-only

percussive-force-stage-a-matrix-v6-v2-contracts:
    python3 scripts/validate_percussive_force_stage_a_matrix_v6_v2.py --validate-only

professional-output-listening-verdict-import-fixtures pack="artifacts/audio_qa/local-professional-output-listening-pack":
    python3 scripts/validate_professional_output_listening_verdict_import_fixtures.py --pack "{{pack}}"

destructive-variation-professional-smoke output="artifacts/audio_qa/local-destructive-variation-professional":
    python3 scripts/generate_dense_break_performance_pack.py --output "{{output}}/dense-break" --date "local-destructive-variation-professional"
    python3 scripts/validate_destructive_variation_professional.py --json-output "{{output}}/destructive-variation.json" --markdown-output "{{output}}/destructive-variation.md" "{{output}}/dense-break/performance-report.json"
    python3 scripts/validate_destructive_variation_professional_smoke.py --output "{{output}}"

rendered-weak-professional-output-fixtures output="artifacts/audio_qa/local-rendered-weak-professional-outputs":
    python3 scripts/generate_rendered_weak_professional_outputs.py --output "{{output}}"
    python3 scripts/validate_rendered_weak_professional_outputs_smoke.py --output "{{output}}"

weak-output-fix-routing-fixtures output="artifacts/audio_qa/local-weak-output-fix-routing":
    python3 scripts/route_weak_output_fixes.py --output "{{output}}" --date "local-weak-output-fix-routing"
    python3 scripts/validate_weak_output_fix_routing_smoke.py --output "{{output}}"
professional-output-suite-smoke output="artifacts/audio_qa/local-professional-output-suite":
    python3 scripts/generate_professional_output_suite.py --output "{{output}}" --date "local-professional-output-suite"
    python3 scripts/validate_professional_output_suite_contract.py "{{output}}/professional-output-suite.json" --output "{{output}}" --mutation-fixtures

sound-quality-readiness-report-smoke output="artifacts/audio_qa/local-sound-quality-readiness-report":
    if ! python3 scripts/validate_professional_output_suite_contract.py "artifacts/audio_qa/local-professional-output-suite/professional-output-suite.json" --output "artifacts/audio_qa/local-professional-output-suite" >/tmp/riotbox-professional-suite-ready.out 2>&1; then just professional-output-suite-smoke; fi
    cargo run -p riotbox-app --bin jam_perform_risk_cue_contract -- --output "{{output}}/jam-perform-risk-cue-contract/jam-perform-risk-cue-contract.json"
    python3 scripts/route_weak_output_fixes.py --output "{{output}}/weak-output" --date "local-sound-quality-readiness-report"
    python3 scripts/generate_release_demo_human_review_queue.py --evidence-mode fixture_calibration --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json --output "{{output}}/human-review-queue" --date "local-sound-quality-readiness-report"
    python3 scripts/generate_release_demo_listening_review_packs.py --queue "{{output}}/human-review-queue/release-demo-human-review-queue.json" --output "{{output}}/release-demo-review-packs" --ticket RIOTBOX-1385 --date "local-sound-quality-readiness-report"
    python3 scripts/generate_sound_quality_readiness_report.py --evidence-mode fixture_calibration --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json --weak-routing "{{output}}/weak-output/weak-output-fix-routing.json" --perform-risk-cue-contract "{{output}}/jam-perform-risk-cue-contract/jam-perform-risk-cue-contract.json" --human-review-queue "{{output}}/human-review-queue/release-demo-human-review-queue.json" --release-demo-review-packs "{{output}}/release-demo-review-packs/release-demo-listening-review-packs.json" --output "{{output}}" --date "local-sound-quality-readiness-report"
    python3 scripts/validate_sound_quality_readiness_smoke.py --output "{{output}}"
    tmp="$(mktemp -d)" && python3 scripts/generate_sound_quality_readiness_report.py --weak-routing "$tmp/missing-weak.json" --professional-output-suite "$tmp/missing-suite.json" --perform-risk-cue-contract "$tmp/missing-cue.json" --human-review-queue "$tmp/missing-queue.json" --release-demo-review-packs "$tmp/missing-packs.json" --output "$tmp/live" --date "live-missing-demo-bank" && jq -e '.demo_bank_evidence.mode == "live_readiness" and .demo_bank_evidence.demo_bank_state == "missing" and .demo_bank.demo_ready_count == 0 and .demo_bank.eligible_human_entry_count == 0 and .blockers[0].code == "real_demo_bank_missing" and .next_actions[0].category == "human_review" and .next_actions[0].target == "dense_break"' "$tmp/live/sound-quality-readiness-report.json" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && if python3 scripts/generate_sound_quality_readiness_report.py --human-review-queue "{{output}}/human-review-queue/release-demo-human-review-queue.json" --weak-routing "$tmp/missing-weak.json" --professional-output-suite "$tmp/missing-suite.json" --perform-risk-cue-contract "$tmp/missing-cue.json" --output "$tmp/live" >"$tmp/mismatch.out" 2>&1; then cat "$tmp/mismatch.out" >&2; echo "expected fixture queue/live readiness mismatch to fail" >&2; exit 1; fi && grep -q "human review queue: evidence mode does not match readiness mode" "$tmp/mismatch.out" && rm -rf "$tmp"

sparse-bass-pressure-professional-fixtures:
    tmp="$(mktemp -d)" && python3 scripts/validate_sparse_bass_pressure_professional.py --json-output "$tmp/sparse-bass.json" --markdown-output "$tmp/sparse-bass.md" scripts/fixtures/automated_musical_fitness/valid_sparse_bass_pulse/manifest.json && jq -e '.schema == "riotbox.sparse_bass_pressure_professional.v1" and .result == "pass" and .source_family == "sparse_bass_pressure" and .human_verdict == "unverified" and .evidence_role == "diagnostic" and .source_backed == true and .source_timing_backed == true and .scripted_generation == true and .quality_proof == false and .metrics.low_band_rms >= .thresholds.min_low_band_rms and .metrics.mc202_bass_signal_rms >= .thresholds.min_mc202_bass_rms and .metrics.tr909_low_band_rms_ratio >= .thresholds.min_tr909_low_band_ratio' "$tmp/sparse-bass.json" && grep -q "Sparse-Bass Pressure" "$tmp/sparse-bass.md" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && if python3 scripts/validate_sparse_bass_pressure_professional.py scripts/fixtures/sparse_bass_pressure_professional/invalid_weak_pressure/manifest.json >"$tmp/invalid.out" 2>&1; then cat "$tmp/invalid.out" >&2; rm -rf "$tmp"; echo "expected weak sparse-bass pressure fixture to fail" >&2; exit 1; fi && grep -q "low_band_pressure_too_weak" "$tmp/invalid.out" && grep -q "mc202_bass_pressure_too_weak" "$tmp/invalid.out" && rm -rf "$tmp"

tonal-hook-professional-fixtures:
    tmp="$(mktemp -d)" && python3 scripts/validate_tonal_hook_professional.py --json-output "$tmp/tonal-hook.json" --markdown-output "$tmp/tonal-hook.md" scripts/fixtures/automated_musical_fitness/valid_tonal_hook_chop/manifest.json && jq -e '.schema == "riotbox.tonal_hook_professional.v1" and .result == "pass" and .source_family == "tonal_hook" and .human_verdict == "unverified" and .evidence_role == "diagnostic" and .source_backed == true and .source_timing_backed == true and .scripted_generation == true and .quality_proof == false and .metrics.w30_contribution_ratio >= .thresholds.min_w30_contribution_ratio and .metrics.w30_contribution_margin >= .thresholds.min_w30_contribution_margin and .metrics.w30_unique_source_offset_count >= .thresholds.min_w30_unique_slice_offsets and .metrics.identity_correlation <= .thresholds.max_identity_correlation' "$tmp/tonal-hook.json" && grep -q "Tonal-Hook Professional" "$tmp/tonal-hook.md" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && if python3 scripts/validate_tonal_hook_professional.py scripts/fixtures/tonal_hook_professional/invalid_hookless/manifest.json >"$tmp/invalid.out" 2>&1; then cat "$tmp/invalid.out" >&2; rm -rf "$tmp"; echo "expected hookless tonal fixture to fail" >&2; exit 1; fi && grep -q "w30_hook_not_dominant" "$tmp/invalid.out" && grep -q "w30_hook_margin_too_low" "$tmp/invalid.out" && grep -q "source_copy_not_transformed_enough" "$tmp/invalid.out" && rm -rf "$tmp"

human-listening-label-corpus-fixtures:
    python3 scripts/validate_human_listening_label_corpus_fixtures.py

listening-review-label-import-fixtures:
    python3 scripts/validate_listening_review_label_import_fixtures.py

audio-judge-spike agent_review="scripts/fixtures/audio_judge_spike/local-agent-musical-review-pack-smoke/agent-review.json" label_corpus="scripts/fixtures/human_listening_label_corpus/valid_dense_break.json" output="artifacts/audio_qa/local-audio-judge-spike":
    mkdir -p "{{output}}"
    python3 scripts/prototype_audio_judge_spike.py --agent-review "{{agent_review}}" --label-corpus "{{label_corpus}}" --json-output "{{output}}/audio-judge-spike.json" --markdown-output "{{output}}/audio-judge-spike.md"

audio-judge-spike-fixtures:
    tmp="$(mktemp -d)" && python3 scripts/prototype_audio_judge_spike.py --agent-review scripts/fixtures/audio_judge_spike/local-agent-musical-review-pack-smoke/agent-review.json --agent-review scripts/fixtures/audio_judge_spike/local-agent-musical-review-pack-smoke-weak/agent-review.json --agent-review scripts/fixtures/audio_judge_spike/local-agent-musical-review-pack-smoke-fail/agent-review.json --agent-review scripts/fixtures/audio_judge_spike/local-agent-musical-review-pack-tonal-hook-pass/agent-review.json --agent-review scripts/fixtures/audio_judge_spike/local-agent-musical-review-pack-sparse-bass-pressure-weak/agent-review.json --label-corpus scripts/fixtures/human_listening_label_corpus/valid_dense_break.json --json-output "$tmp/audio-judge-spike.json" --markdown-output "$tmp/audio-judge-spike.md" && jq -e '.schema == "riotbox.audio_judge_spike.v1" and .result == "pass" and .judge_readiness == "not_ready" and .human_verdict == "unverified" and (.candidate_evaluations | length) == 5 and .metrics_baseline.predicted_label == "pass" and .calibration.label_count == 5 and .calibration.matched_label_count == 5 and (.calibration.matched_verdicts == ["fail", "pass", "weak"]) and (.calibration.source_family_coverage.matched_source_families == ["dense_break", "sparse_bass_pressure", "tonal_hook"]) and (.calibration.failure_examples | length) == 0 and .calibration.confusion_matrix.pass.pass == 2 and .calibration.confusion_matrix.weak.weak == 2 and .calibration.confusion_matrix.fail.fail == 1 and (.providers | length) == 2' "$tmp/audio-judge-spike.json" && grep -q "Matched source families" "$tmp/audio-judge-spike.md" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && if python3 scripts/prototype_audio_judge_spike.py --agent-review scripts/fixtures/audio_judge_spike/invalid_bad_agent_review.json --label-corpus scripts/fixtures/human_listening_label_corpus/valid_dense_break.json >"$tmp/invalid.out" 2>&1; then cat "$tmp/invalid.out" >&2; rm -rf "$tmp"; echo "expected bad agent-review fixture to fail" >&2; exit 1; fi && grep -q "missing source_recognition" "$tmp/invalid.out" && rm -rf "$tmp"

audio-judge-spike-generated-smoke output="artifacts/audio_qa/local-agent-musical-review-pack-smoke" report="artifacts/audio_qa/local-audio-judge-spike-generated-smoke":
    test -s "{{output}}/agent-review.json"
    mkdir -p "{{report}}"
    python3 scripts/prototype_audio_judge_spike.py --agent-review "{{output}}/agent-review.json" --label-corpus scripts/fixtures/human_listening_label_corpus/valid_dense_break.json --json-output "{{report}}/audio-judge-spike.json" --markdown-output "{{report}}/audio-judge-spike.md"
    jq -e '.schema == "riotbox.audio_judge_spike.v1" and .result == "pass" and .judge_readiness == "not_ready" and .metrics_baseline.provider == "riotbox_metrics_baseline" and .calibration.matched_label_count == 1 and (.calibration.failure_examples | length) == 4 and .calibration.source_family_coverage.matched_source_families == ["dense_break"]' "{{report}}/audio-judge-spike.json"

musical-pass-gate-policy-fixtures:
    tmp="$(mktemp)" && python3 scripts/validate_musical_pass_gate_policy.py --json-output "$tmp" scripts/fixtures/musical_pass_gate_policy/policy_v1.json && jq -e '.schema == "riotbox.musical_pass_gate_policy.v1" and .result == "pass" and .state_count == 8 and (.musical_pass_states == ["calibrated_agent_musical_pass", "human_musical_pass"]) and .minimum_calibrated_label_count >= 12' "$tmp" && rm "$tmp"
    tmp="$(mktemp -d)" && if python3 scripts/validate_musical_pass_gate_policy.py scripts/fixtures/musical_pass_gate_policy/invalid_agent_promising_claims_pass.json >"$tmp/invalid.out" 2>&1; then cat "$tmp/invalid.out" >&2; rm -rf "$tmp"; echo "expected agent-promising pass policy fixture to fail" >&2; exit 1; fi && grep -q "agent_promising must not claim musical pass" "$tmp/invalid.out" && rm -rf "$tmp"

sound-product-readiness-rubric-fixtures:
    tmp="$(mktemp)" && python3 scripts/validate_sound_product_readiness_rubric.py --json-output "$tmp" scripts/fixtures/sound_product_readiness_rubric/rubric_v1.json && jq -e '.schema == "riotbox.sound_product_readiness_rubric.v1" and .result == "pass" and .state_count == 7 and (.quality_states == ["demo_ready", "human_pass", "release_ready"]) and .musical_dimension_count == 7 and (.fix_categories == ["bass_movement", "chop_policy", "destructive_gesture", "drum_pressure", "fixture_threshold", "mix_bus", "source_selection", "ui_cue"]) and (.phase_links == ["P021", "P022", "P023"])' "$tmp" && rm "$tmp"
    tmp="$(mktemp -d)" && if python3 scripts/validate_sound_product_readiness_rubric.py scripts/fixtures/sound_product_readiness_rubric/invalid_scripted_claims_quality.json >"$tmp/invalid.out" 2>&1; then cat "$tmp/invalid.out" >&2; rm -rf "$tmp"; echo "expected scripted quality-proof fixture to fail" >&2; exit 1; fi && grep -q "hardcoded_or_scripted must not claim quality proof" "$tmp/invalid.out" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && jq '.evidence_classes.synthetic_oracle = {"meaning":"oracle","may_claim_quality_proof":true,"allowed_use":["pass"],"blocked_claims":["none"]}' scripts/fixtures/sound_product_readiness_rubric/rubric_v1.json >"$tmp/unknown.json" && if python3 scripts/validate_sound_product_readiness_rubric.py "$tmp/unknown.json" >"$tmp/unknown.out" 2>&1; then cat "$tmp/unknown.out" >&2; rm -rf "$tmp"; echo "expected unknown evidence class fixture to fail" >&2; exit 1; fi && grep -q "unknown evidence classes" "$tmp/unknown.out" && rm -rf "$tmp"

release-grade-demo-bank-fixtures:
    tmp="$(mktemp)" && python3 scripts/validate_release_grade_demo_bank.py --json-output "$tmp" scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json && jq -e '.schema == "riotbox.release_grade_demo_bank.v1" and .result == "pass" and .entry_count >= 10 and .demo_ready_count >= 2 and .verdict_counts.pass >= 2 and .verdict_counts.weak >= 2 and .verdict_counts.fail >= 1 and .verdict_counts.unverified >= 5 and (.source_families | index("dense_break")) and (.source_families | index("tonal_hook")) and (.source_families | index("sparse_bass_pressure")) and (.source_families | index("tonal_pad")) and (.source_families | index("bad_timing")) and (.source_families | index("weak_source"))' "$tmp" && rm "$tmp"
    tmp="$(mktemp -d)" && if python3 scripts/validate_release_grade_demo_bank.py scripts/fixtures/release_grade_demo_bank/invalid_weak_missing_fix_category.json >"$tmp/invalid.out" 2>&1; then cat "$tmp/invalid.out" >&2; rm -rf "$tmp"; echo "expected weak missing fix category fixture to fail" >&2; exit 1; fi && grep -q "weak/fail entries need fix_categories" "$tmp/invalid.out" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && jq '.entries[-1].demo_readiness = "demo_ready"' scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json >"$tmp/unverified-demo-ready.json" && if python3 scripts/validate_release_grade_demo_bank.py "$tmp/unverified-demo-ready.json" >"$tmp/unverified-demo-ready.out" 2>&1; then cat "$tmp/unverified-demo-ready.out" >&2; rm -rf "$tmp"; echo "expected unverified demo-ready fixture to fail" >&2; exit 1; fi && grep -q "unverified entries must stay unverified" "$tmp/unverified-demo-ready.out" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && jq '.entries[-1].quality_claim = true' scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json >"$tmp/unverified-quality-claim.json" && if python3 scripts/validate_release_grade_demo_bank.py "$tmp/unverified-quality-claim.json" >"$tmp/unverified-quality-claim.out" 2>&1; then cat "$tmp/unverified-quality-claim.out" >&2; rm -rf "$tmp"; echo "expected unverified quality-claim fixture to fail" >&2; exit 1; fi && grep -q "unverified entries must not claim quality" "$tmp/unverified-quality-claim.out" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && jq '(.entries[] | select(.entry_id == "bad-timing-beat20-unverified-candidate")) |= (del(.rendered_wav) | .human_verdict = "fail" | .demo_readiness = "not_demo_ready" | .fix_categories = ["source_selection"] | .degraded_or_reject_evidence = {schema:"riotbox.demo_bank_degraded_or_reject_review_evidence.v1",outcome:"reject",product_path_reviewed:true,fallback_music_present:false,reason:"Fixture-calibration rejection keeps timing-safe output honest."})' scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json >"$tmp/reviewed-negative-no-wav.json" && python3 scripts/validate_release_grade_demo_bank.py "$tmp/reviewed-negative-no-wav.json" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && jq 'del(.entries[-1].rendered_wav)' scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json >"$tmp/unverified-no-wav.json" && if python3 scripts/validate_release_grade_demo_bank.py "$tmp/unverified-no-wav.json" >"$tmp/unverified-no-wav.out" 2>&1; then cat "$tmp/unverified-no-wav.out" >&2; echo "expected unverified entry without rendered WAV to fail" >&2; exit 1; fi && grep -q "rendered_wav may be omitted only" "$tmp/unverified-no-wav.out" && rm -rf "$tmp"

source-family-release-demo-coverage-fixtures output="artifacts/audio_qa/local-source-family-release-demo-coverage":
    mkdir -p "{{output}}"
    python3 scripts/validate_source_family_release_demo_coverage.py --evidence-mode fixture_calibration --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json --json-output "{{output}}/source-family-release-demo-coverage.json" --markdown-output "{{output}}/source-family-release-demo-coverage.md"
    python3 scripts/validate_source_family_release_demo_coverage.py --validate-report "{{output}}/source-family-release-demo-coverage.json"
    jq -e '.schema == "riotbox.source_family_release_demo_coverage.v1" and .result == "pass" and .source_files_required == false and .release_readiness == "blocked" and .quality_claim_allowed == false and .required_family_count == 6 and .covered_demo_ready_family_count == 2 and (.missing_demo_candidate_families == []) and (.missing_human_verdict_families | index("pad_noise")) and (.missing_human_verdict_families | index("weak_source")) and (.missing_human_verdict_families | index("bad_timing")) and (.missing_demo_ready_families | index("sparse_drums")) and (.missing_demo_ready_families | index("pad_noise")) and (.missing_demo_ready_families | index("weak_source")) and (.missing_demo_ready_families | index("bad_timing")) and (.missing_family_success_families | index("weak_source")) and (.missing_family_success_families | index("bad_timing")) and all(.blockers[]; .code != "source_family_demo_candidate_missing") and (.blockers[] | select(.code == "source_family_human_verdict_missing" and .source_family == "pad_noise")) and (.blockers[] | select(.code == "source_family_demo_or_degraded_review_missing" and .source_family == "pad_noise")) and (.blockers[] | select(.code == "source_family_degraded_or_reject_review_missing" and .source_family == "weak_source")) and (.blockers[] | select(.code == "source_family_degraded_or_reject_review_missing" and .source_family == "bad_timing")) and any(.families[]; .source_family == "dense_break" and .status == "demo_ready_covered" and (.demo_ready_entry_ids | index("dense-break-beat03-human-pass"))) and any(.families[]; .source_family == "sparse_drums" and .status == "human_verdict_non_demo" and (.human_verdict_entry_ids | index("sparse-bass-pressure-human-weak")) and (.candidate_entry_ids | index("sparse-bass-pressure-updated-unverified-candidate")) and (.unverified_entry_ids | index("sparse-bass-pressure-updated-unverified-candidate"))) and any(.families[]; .source_family == "pad_noise" and .status == "candidate_only" and .success_requirement == "demo_ready_or_reviewed_degraded_or_reject" and (.candidate_entry_ids | index("pad-noise-fadapad-unverified-candidate"))) and any(.families[]; .source_family == "bad_timing" and .status == "candidate_only" and .success_requirement == "reviewed_degraded_or_reject" and (.candidate_entry_ids | index("bad-timing-beat20-unverified-candidate")) and (.unverified_entry_ids | index("bad-timing-beat20-unverified-candidate"))) and any(.families[]; .source_family == "weak_source" and .status == "candidate_only" and .success_requirement == "reviewed_degraded_or_reject" and (.candidate_entry_ids | index("weak-source-beat20-rejection-unverified-candidate")) and (.unverified_entry_ids | index("weak-source-beat20-rejection-unverified-candidate")))' "{{output}}/source-family-release-demo-coverage.json"
    test -s "{{output}}/source-family-release-demo-coverage.md"
    grep -q "Human verdict entries" "{{output}}/source-family-release-demo-coverage.md"
    tmp="$(mktemp)" && jq '.quality_claim_allowed = true' "{{output}}/source-family-release-demo-coverage.json" > "$tmp" && if python3 scripts/validate_source_family_release_demo_coverage.py --validate-report "$tmp" >"$tmp.out" 2>&1; then cat "$tmp.out" >&2; rm "$tmp" "$tmp.out"; echo "expected blocked source-family coverage quality claim to fail" >&2; exit 1; fi && grep -q "blocked_report_claims_quality" "$tmp.out" && rm "$tmp" "$tmp.out"
    tmp="$(mktemp -d)" && jq '.demo_bank_evidence.demo_bank_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"' "{{output}}/source-family-release-demo-coverage.json" >"$tmp/stale.json" && if python3 scripts/validate_source_family_release_demo_coverage.py --validate-report "$tmp/stale.json" >"$tmp/stale.out" 2>&1; then cat "$tmp/stale.out" >&2; echo "expected stale demo-bank identity to fail" >&2; exit 1; fi && grep -q "demo_bank_evidence_identity_stale" "$tmp/stale.out" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && jq '.entries += [{entry_id:"weak-source-unverified-candidate",source_family:"weak_source",source_path:"data/test_audio/examples/Beat20_128BPM(Full).wav",rendered_wav:{path:"artifacts/audio_qa/release-demo-bank/weak-source/05_rebuild_only_performance.wav",sha256:"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},metrics:{path:"artifacts/audio_qa/release-demo-bank/weak-source/performance-report.json",sha256:"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},review_prompt:{path:"artifacts/audio_qa/release-demo-bank/weak-source/prompt.md",sha256:"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"},human_verdict:"unverified",demo_readiness:"unverified",quality_claim:false,demo_worthiness_note:"Weak-source candidate exists but has no human verdict yet.",fix_categories:[],musical_summary:{hook_within_two_bars:"unverified",hardest_audible_element:"unverified",source_character:"unverified",destructive_contrast:"unverified",bass_drum_pressure:"unverified",live_triggerability:"unverified",eight_bar_replay_value:"unverified"}}]' scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json >"$tmp/demo-bank.json" && python3 scripts/validate_source_family_release_demo_coverage.py --evidence-mode fixture_calibration --demo-bank "$tmp/demo-bank.json" --json-output "$tmp/coverage.json" && jq -e '.release_readiness == "blocked" and (.missing_demo_candidate_families | index("weak_source") | not) and (.missing_human_verdict_families | index("weak_source")) and any(.families[]; .source_family == "weak_source" and .status == "candidate_only")' "$tmp/coverage.json" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && python3 scripts/validate_source_family_release_demo_coverage.py --json-output "$tmp/live.json" && jq -e '.demo_bank_evidence.mode == "live_readiness" and .demo_bank_evidence.demo_bank_state == "missing" and .covered_demo_ready_family_count == 0 and .demo_bank_evidence.eligible_human_verdict_count == 0' "$tmp/live.json" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && python3 scripts/validate_source_family_release_demo_coverage.py --evidence-mode live_readiness --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json --json-output "$tmp/live-rejects-fixture.json" && jq -e '.demo_bank_evidence.mode == "live_readiness" and .demo_bank_evidence.demo_bank_state == "rejected_non_live_bank" and .covered_demo_ready_family_count == 0 and .demo_bank_evidence.eligible_human_verdict_count == 0 and (.missing_demo_candidate_families | length) == 6 and (.demo_bank_evidence.ignored_unproven_human_verdict_entry_ids | length) >= 5' "$tmp/live-rejects-fixture.json" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && jq '(.entries[] | select(.entry_id == "bad-timing-beat20-unverified-candidate")) |= (.human_verdict = "fail" | .demo_readiness = "not_demo_ready" | .fix_categories = ["source_selection"] | .degraded_or_reject_evidence = {schema:"riotbox.demo_bank_degraded_or_reject_review_evidence.v1",outcome:"reject",product_path_reviewed:true,fallback_music_present:false,reason:"Fixture-calibration rejection keeps untrusted timing out of bar-locked output."})' scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json >"$tmp/calibration-bank.json" && python3 scripts/validate_source_family_release_demo_coverage.py --evidence-mode fixture_calibration --demo-bank "$tmp/calibration-bank.json" --json-output "$tmp/calibration-degraded.json" && jq -e '(.missing_family_success_families | index("bad_timing") | not) and any(.families[]; .source_family == "bad_timing" and .status == "reviewed_degraded_or_reject")' "$tmp/calibration-degraded.json" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && jq '(.entries[] | select(.entry_id == "pad-noise-fadapad-unverified-candidate")) |= (.source_family = "pad_noise" | .human_verdict = "fail" | .demo_readiness = "not_demo_ready" | .fix_categories = ["source_selection"] | .degraded_or_reject_evidence = {schema:"riotbox.demo_bank_degraded_or_reject_review_evidence.v1",outcome:"unavailable",product_path_reviewed:true,fallback_music_present:false,reason:"Fixture-calibration pad timing stayed unavailable without fallback music."})' scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json >"$tmp/pad-degraded-bank.json" && python3 scripts/validate_release_grade_demo_bank.py "$tmp/pad-degraded-bank.json" && python3 scripts/validate_source_family_release_demo_coverage.py --evidence-mode fixture_calibration --demo-bank "$tmp/pad-degraded-bank.json" --json-output "$tmp/pad-degraded.json" && jq -e '(.missing_family_success_families | index("pad_noise") | not) and any(.families[]; .source_family == "pad_noise" and (.demo_bank_family_aliases | index("pad_noise")) and .success_requirement == "demo_ready_or_reviewed_degraded_or_reject" and .status == "reviewed_degraded_or_reject")' "$tmp/pad-degraded.json" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && jq '(.entries[] | select(.entry_id == "pad-noise-fadapad-unverified-candidate")) |= (.source_family = "pad_noise" | .human_verdict = "pass" | .demo_readiness = "demo_ready")' scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json >"$tmp/pad-demo-bank.json" && python3 scripts/validate_release_grade_demo_bank.py "$tmp/pad-demo-bank.json" && python3 scripts/validate_source_family_release_demo_coverage.py --evidence-mode fixture_calibration --demo-bank "$tmp/pad-demo-bank.json" --json-output "$tmp/pad-demo.json" && jq -e '(.missing_family_success_families | index("pad_noise") | not) and any(.families[]; .source_family == "pad_noise" and (.demo_bank_family_aliases | index("pad_noise")) and .success_requirement == "demo_ready_or_reviewed_degraded_or_reject" and .status == "demo_ready_covered")' "$tmp/pad-demo.json" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && jq '(.entries[] | select(.entry_id == "sparse-bass-pressure-human-weak")) |= (.entry_id = "sparse-drums-exact-product-pass" | .source_family = "sparse_drums" | .human_verdict = "pass" | .demo_readiness = "demo_ready" | .fix_categories = [])' scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json >"$tmp/sparse-demo-bank.json" && python3 scripts/validate_release_grade_demo_bank.py "$tmp/sparse-demo-bank.json" && python3 scripts/validate_source_family_release_demo_coverage.py --evidence-mode fixture_calibration --demo-bank "$tmp/sparse-demo-bank.json" --json-output "$tmp/sparse-demo.json" && jq -e '(.missing_family_success_families | index("sparse_drums") | not) and any(.families[]; .source_family == "sparse_drums" and (.demo_bank_family_aliases | index("sparse_drums")) and .success_requirement == "demo_ready_human_pass" and .status == "demo_ready_covered" and (.demo_ready_entry_ids | index("sparse-drums-exact-product-pass")))' "$tmp/sparse-demo.json" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && jq -n '{schema:"riotbox.listening_review.v1",schema_version:1,human_verdict:"reject",reviewer:"Markus"}' >"$tmp/review.json" && review_hash="$(sha256sum "$tmp/review.json" | cut -d ' ' -f1)" && jq --arg review "$tmp/review.json" --arg review_hash "$review_hash" '.evidence_role = "live_review" | (.entries[] | select(.entry_id == "bad-timing-beat20-unverified-candidate")) |= (.human_verdict = "fail" | .demo_readiness = "not_demo_ready" | .fix_categories = ["source_selection"] | .human_review_evidence = {schema:"riotbox.demo_bank_human_review_evidence.v1",reviewer:"Markus",reviewer_kind:"human",review_path:$review,review_sha256:$review_hash} | .degraded_or_reject_evidence = {schema:"riotbox.demo_bank_degraded_or_reject_review_evidence.v1",outcome:"reject",product_path_reviewed:true,fallback_music_present:false,reason:"Timing stayed untrusted, so the live product rejected confident bar-locked output."})' scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json >"$tmp/live-bank.json" && python3 scripts/validate_source_family_release_demo_coverage.py --evidence-mode live_readiness --demo-bank "$tmp/live-bank.json" --json-output "$tmp/live-generic-review.json" && jq -e '(.missing_family_success_families | index("bad_timing")) and any(.families[]; .source_family == "bad_timing" and .status == "human_verdict_non_demo" and (.degraded_or_reject_entry_ids | length) == 0)' "$tmp/live-generic-review.json" && rm -rf "$tmp"

release-demo-human-review-queue-fixtures output="artifacts/audio_qa/local-release-demo-human-review-queue":
    python3 scripts/generate_release_demo_human_review_queue.py --evidence-mode fixture_calibration --demo-bank scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json --output "{{output}}" --date "local-release-demo-human-review-queue"
    python3 scripts/generate_release_demo_human_review_queue.py --validate-report "{{output}}/release-demo-human-review-queue.json" --mutation-fixtures
    test -s "{{output}}/release-demo-human-review-queue.md"
    grep -q "Release-Demo Human Review Queue" "{{output}}/release-demo-human-review-queue.md"
    grep -q "Required listening questions" "{{output}}/release-demo-human-review-queue.md"
    tmp="$(mktemp -d)" && python3 scripts/generate_release_demo_human_review_queue.py --output "$tmp/live" --date "live-missing-demo-bank" && jq -e '.demo_bank_evidence.mode == "live_readiness" and .demo_bank_evidence.demo_bank_state == "missing" and .review_queue_count == 0 and .next_actions[0].category == "human_review" and .next_actions[0].target == "dense_break"' "$tmp/live/release-demo-human-review-queue.json" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && jq '(.entries[] | select(.entry_id == "pad-noise-fadapad-unverified-candidate")) |= (.source_family = "pad_noise" | .human_verdict = "weak" | .demo_readiness = "not_demo_ready" | .fix_categories = ["source_selection"])' scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json >"$tmp/pad-needs-outcome.json" && python3 scripts/generate_release_demo_human_review_queue.py --evidence-mode fixture_calibration --demo-bank "$tmp/pad-needs-outcome.json" --output "$tmp/queue" --date "pad-dual-path" && jq -e 'any(.next_actions[]; .category == "demo_or_degraded_review" and .target == "pad_noise" and (.action | contains("demo or degraded/unavailable pad/noise outcome"))) and all(.review_queue[]; .entry_id != "pad-noise-fadapad-unverified-candidate")' "$tmp/queue/release-demo-human-review-queue.json" && rm -rf "$tmp"

release-demo-listening-review-packs-fixtures output="artifacts/audio_qa/local-release-demo-listening-review-packs":
    scripts/validate_release_demo_listening_review_packs_fixtures.sh "{{output}}"

demo-bank-promotion-fixtures:
    scripts/validate_demo_bank_promotion_fixtures.sh

sound-product-2010-future-ideas-fixtures:
    tmp="$(mktemp)" && python3 scripts/validate_sound_product_2010_future_ideas.py --json-output "$tmp" scripts/fixtures/sound_product_2010_future_ideas/ideas_v1.json && jq -e '.schema == "riotbox.sound_product_2010_future_ideas.v1" and .result == "pass" and .idea_count == 7 and .release_blocking_count == 0 and (.product_spine | index("source_graph")) and (.product_spine | index("session_model")) and (.product_spine | index("audio_qa")) and (.required_ideas | index("producer_loop_take_selection")) and (.required_ideas | index("ecosystem_surfaces_preserve_gates"))' "$tmp" && rm "$tmp"
    tmp="$(mktemp -d)" && if python3 scripts/validate_sound_product_2010_future_ideas.py scripts/fixtures/sound_product_2010_future_ideas/invalid_release_blocking_idea.json >"$tmp/invalid.out" 2>&1; then cat "$tmp/invalid.out" >&2; rm -rf "$tmp"; echo "expected release-blocking 20/10 idea fixture to fail" >&2; exit 1; fi && grep -q "20/10 ideas must not be release_blocking" "$tmp/invalid.out" && rm -rf "$tmp"

beat03-auto-feral-grid-proof date="local-beat03-feral-grid-auto-proof":
    scripts/validate_beat03_auto_feral_grid_pack.sh "{{date}}"

beat08-auto-feral-grid-proof date="local-beat08-feral-grid-auto-proof":
    scripts/validate_auto_feral_grid_source_timing_pack.sh beat08 "{{date}}"

beat20-auto-feral-grid-fallback-proof date="local-beat20-feral-grid-auto-fallback-proof":
    scripts/validate_auto_feral_grid_source_timing_pack.sh beat20 "{{date}}"

dh-beatc-auto-feral-grid-proof date="local-dh-beatc-feral-grid-auto-proof":
    scripts/validate_auto_feral_grid_source_timing_pack.sh dh-beatc "{{date}}"

recipe15-feral-grid-auto-proof:
    just beat03-auto-feral-grid-proof local-beat03-feral-grid-auto-proof
    just beat08-auto-feral-grid-proof local-beat08-feral-grid-auto-proof
    just dh-beatc-auto-feral-grid-proof local-dh-beatc-feral-grid-auto-proof
    just beat20-auto-feral-grid-fallback-proof local-beat20-feral-grid-auto-fallback-proof

recipe15-feral-grid-auto-proof-strict:
    RIOTBOX_REQUIRE_RECIPE15_FIXTURES=1 just recipe15-feral-grid-auto-proof

recipe15-strict-missing-fixture-fixture:
    tmp="$(mktemp -d)" && \
      if RIOTBOX_REQUIRE_RECIPE15_FIXTURES=1 RIOTBOX_RECIPE15_SOURCE_PATH="$tmp/missing.wav" scripts/validate_auto_feral_grid_source_timing_pack.sh beat03 local-missing-fixture-proof >"$tmp/out" 2>"$tmp/err"; then \
        cat "$tmp/out" "$tmp/err" >&2; \
        rm -rf "$tmp"; \
        echo "expected strict Recipe 15 missing fixture check to fail" >&2; \
        exit 1; \
      fi && \
      grep -q "missing required Recipe 15 source fixture" "$tmp/err" && \
      rm -rf "$tmp"

p012-all-lane-proof-summary output="artifacts/audio_qa/local/p012_all_lane_source_grid_output_proof_summary.md":
    python3 scripts/write_p012_all_lane_proof_summary.py --output "{{output}}"
    python3 scripts/validate_p012_all_lane_proof_summary.py "{{output}}"

audio-qa-notes target="artifacts/audio_qa/local/notes.md":
    mkdir -p "$(dirname "{{target}}")"
    cp docs/benchmarks/audio_qa_listening_review_template_2026-04-26.md "{{target}}"

listening-review-pack ticket output="artifacts/audio_qa/local/listening-reviews" source="" candidate="" expected="Review whether the generated audio has a clear hook, source character, and stage-meaningful impact.":
    python3 scripts/listening_review_workflow.py pack --ticket "{{ticket}}" --output "{{output}}/{{ticket}}" --source-file "{{source}}" --candidate "{{candidate}}" --expected "{{expected}}"

listening-review-record review verdict strongest source_recognition hook failure="" direction="" avoid="" follow_up="" reviewer="":
    python3 scripts/listening_review_workflow.py record --review "{{review}}" --human-verdict "{{verdict}}" --strongest-element "{{strongest}}" --source-recognition "{{source_recognition}}" --hook-after-two-bars "{{hook}}" --failure-reason "{{failure}}" --preferred-direction "{{direction}}" --avoid "{{avoid}}" --concrete-follow-up "{{follow_up}}" --reviewer "{{reviewer}}"

listening-review-fixtures:
    tmp="$(mktemp -d)" && python3 scripts/listening_review_workflow.py pack --ticket RIOTBOX-DRYRUN --output "$tmp/review" --candidate scripts/fixtures/automated_musical_fitness/valid/manifest.json --technical-status pass --automated-musical-fitness-status pass --expected "A dry-run fixture should prove structured review shape without audio hardware." && jq -e '.schema == "riotbox.listening_review.v1" and .human_verdict == "unverified" and .automated_musical_fitness_status == "pass"' "$tmp/review/review.json" && python3 scripts/listening_review_workflow.py record --review "$tmp/review/review.json" --human-verdict technically_ok_but_musically_weak --strongest-element chop --source-recognition source_transformed_but_present --hook-after-two-bars weak --failure-reason "dry run weak hook" --preferred-direction "harder chop contrast" --avoid "polite loop,source masking" --concrete-follow-up "tighten chop fixture" --reviewer "fixture" && python3 scripts/listening_review_workflow.py validate "$tmp/review/review.json" && jq -e '.human_verdict == "technically_ok_but_musically_weak" and .strongest_element == "chop" and .source_recognition == "source_transformed_but_present" and .hook_after_two_bars == "weak" and (.avoid | length == 2)' "$tmp/review/review.json" && grep -q "Human verdict" "$tmp/review/review-summary.md" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && python3 scripts/listening_review_workflow.py pack --ticket RIOTBOX-RESTORE --output "$tmp/review" --candidate scripts/fixtures/automated_musical_fitness/valid/manifest.json --expected "A restore-led review should accept restore as the strongest element." && python3 scripts/listening_review_workflow.py record --review "$tmp/review/review.json" --human-verdict keep --strongest-element restore --source-recognition source_transformed_but_present --hook-after-two-bars clear --preferred-direction "keep the restore impact obvious" --concrete-follow-up "preserve restore verdict support" --reviewer "fixture" && python3 scripts/listening_review_workflow.py validate "$tmp/review/review.json" && jq -e '.human_verdict == "keep" and .strongest_element == "restore"' "$tmp/review/review.json" && rm -rf "$tmp"

observer-audio-correlation-notes target="artifacts/audio_qa/local/observer_audio_correlation.md":
    mkdir -p "$(dirname "{{target}}")"
    cp docs/benchmarks/observer_audio_correlation_template_2026-04-29.md "{{target}}"

observer-audio-correlate observer manifest output="artifacts/audio_qa/local/observer_audio_summary.md":
    cargo run -p riotbox-app --bin observer_audio_correlate -- --observer "{{observer}}" --manifest "{{manifest}}" --output "{{output}}"

observer-audio-correlate-json observer manifest output="artifacts/audio_qa/local/observer_audio_summary.json":
    cargo run -p riotbox-app --bin observer_audio_correlate -- --observer "{{observer}}" --manifest "{{manifest}}" --output "{{output}}" --json

observer-audio-correlate-json-fixture:
    tmp="$(mktemp)" && cargo run -p riotbox-app --bin observer_audio_correlate -- --observer crates/riotbox-app/tests/fixtures/observer_audio_correlation/events.ndjson --manifest crates/riotbox-app/tests/fixtures/observer_audio_correlation/manifest.json --output "$tmp" --json && jq -e '.schema == "riotbox.observer_audio_summary.v1" and .schema_version == 1 and .control_path.present == true and .output_path.present == true and .output_path.metrics.mc202_source_grid_alignment.hit_ratio >= 0.5 and (.output_path.issues | length == 0)' "$tmp" && python3 scripts/validate_observer_audio_summary_json.py "$tmp" && rm "$tmp"

observer-audio-correlate-locked-grid-json-fixture:
    tmp="$(mktemp)" && cargo run -p riotbox-app --bin observer_audio_correlate -- --observer crates/riotbox-app/tests/fixtures/observer_audio_correlation/events_locked_grid.ndjson --manifest crates/riotbox-app/tests/fixtures/observer_audio_correlation/manifest_locked_grid.json --output "$tmp" --json --require-evidence && jq -e '.control_path.observer_source_timing.grid_use == "locked_grid" and .output_path.source_timing.grid_use == "locked_grid" and .output_path.source_timing.phrase_status == "stable" and .output_path.source_timing.primary_phrase_count > 0 and .output_path.source_timing.primary_phrase_bar_count > 0 and .output_path.source_timing_alignment.status == "aligned" and .output_path.source_timing_alignment.grid_use_compatibility == "aligned" and .output_path.source_timing_alignment.downbeat_offset_compatibility == "aligned" and .output_path.source_timing_alignment.downbeat_ambiguity_compatibility == "aligned" and .output_path.source_timing_anchor_alignment.status == "aligned" and .output_path.source_timing_groove_alignment.status == "aligned" and .output_path.metrics.mc202_source_grid_alignment.hit_ratio >= 0.5 and (.output_path.issues | length == 0)' "$tmp" && python3 scripts/validate_observer_audio_summary_json.py "$tmp" && rm "$tmp"

observer-audio-summary-validator-fixtures:
    python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_failure.json
    python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_schema.json; then echo "expected invalid observer/audio summary fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_missing_metric_key.json; then echo "expected missing metric observer/audio summary fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_observer_source_timing_quality.json; then echo "expected invalid observer source timing quality fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_observer_source_timing_cue.json; then echo "expected invalid observer source timing cue fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_grid_bpm_decision_reason.json; then echo "expected invalid grid BPM decision reason fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_source_timing_bpm_delta.json; then echo "expected invalid source timing BPM delta fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_user_override_bpm_agrees_true.json; then echo "expected invalid user override BPM agreement true fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_user_override_bpm_agrees_false.json; then echo "expected invalid user override BPM agreement false fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_static_default_bpm_agrees_true.json; then echo "expected invalid static default BPM agreement true fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_static_default_bpm_agrees_false.json; then echo "expected invalid static default BPM agreement false fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_source_timing_grid_use.json; then echo "expected invalid source timing grid use fixture to fail" >&2; exit 1; fi
    tmp="$(mktemp)" && jq 'del(.control_path.observer_source_timing.actionability)' crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json > "$tmp" && if python3 scripts/validate_observer_audio_summary_json.py "$tmp"; then echo "expected missing observer source timing actionability fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq 'del(.control_path.observer_source_timing.primary_downbeat_score)' crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json > "$tmp" && if python3 scripts/validate_observer_audio_summary_json.py "$tmp"; then echo "expected missing observer source timing downbeat score fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.control_path.observer_source_timing.degraded_policy = "manual_confirm" | .control_path.observer_source_timing.cue = "needs confirm" | .control_path.observer_source_timing.actionability = "confirm grid first" | .control_path.observer_source_timing.grid_use = "manual_confirm_only" | .control_path.observer_source_timing.beat_status = "tempo_only" | .control_path.observer_source_timing.beat_count = 0 | .control_path.observer_source_timing.downbeat_status = "ambiguous" | .control_path.observer_source_timing.bar_count = 0 | .control_path.observer_source_timing.phrase_status = "uncertain" | .control_path.observer_source_timing.phrase_count = 0 | .control_path.observer_source_timing.primary_warning_code = "phrase_uncertain" | .control_path.observer_source_timing.warning_codes = ["ambiguous_downbeat", "phrase_uncertain"]' crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json > "$tmp" && if python3 scripts/validate_observer_audio_summary_json.py "$tmp"; then echo "expected mismatched observer source timing primary warning fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq 'del(.output_path.source_timing.actionability)' crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json > "$tmp" && if python3 scripts/validate_observer_audio_summary_json.py "$tmp"; then echo "expected missing source timing actionability fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_source_timing_alignment_status.json; then echo "expected invalid source timing alignment status fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_source_timing_alignment_grid_use_compatibility.json; then echo "expected invalid source timing alignment grid-use compatibility fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_source_timing_alignment_downbeat_offset_compatibility.json; then echo "expected invalid source timing alignment downbeat-offset compatibility fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_source_timing_alignment_downbeat_ambiguity_compatibility.json; then echo "expected invalid source timing alignment downbeat-ambiguity compatibility fixture to fail" >&2; exit 1; fi
    tmp="$(mktemp)" && jq '.control_path.observer_source_timing.beat_count = 0' crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json > "$tmp" && if python3 scripts/validate_observer_audio_summary_json.py "$tmp"; then echo "expected invalid observer source timing beat-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.control_path.observer_source_timing.bar_count = 0' crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json > "$tmp" && if python3 scripts/validate_observer_audio_summary_json.py "$tmp"; then echo "expected invalid observer source timing bar-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.control_path.observer_source_timing.phrase_count = 0' crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json > "$tmp" && if python3 scripts/validate_observer_audio_summary_json.py "$tmp"; then echo "expected invalid observer source timing phrase-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.control_path.observer_source_timing.phrase_status = "uncertain" | .control_path.observer_source_timing.phrase_count = 1' crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json > "$tmp" && if python3 scripts/validate_observer_audio_summary_json.py "$tmp"; then echo "expected invalid non-locked observer source timing phrase-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.output_path.source_timing.primary_phrase_count = 0' crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json > "$tmp" && if python3 scripts/validate_observer_audio_summary_json.py "$tmp"; then echo "expected invalid stable source timing phrase-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.output_path.source_timing.phrase_status = "unavailable"' crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json > "$tmp" && if python3 scripts/validate_observer_audio_summary_json.py "$tmp"; then echo "expected invalid unavailable source timing phrase evidence fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.output_path.source_timing.phrase_status = "not_enough_material" | .output_path.source_timing.primary_phrase_count = 1' crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json > "$tmp" && if python3 scripts/validate_observer_audio_summary_json.py "$tmp"; then echo "expected invalid not-enough-material source timing phrase evidence fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    python3 scripts/validate_observer_audio_scalar_metric_fixtures.py
    python3 scripts/validate_observer_audio_source_timing_status_fixtures.py
    python3 scripts/validate_observer_audio_lane_recipe_metric_fixtures.py
    python3 scripts/validate_observer_audio_mc202_phrase_grid_hit_ratio_fixtures.py
    python3 scripts/validate_observer_audio_mc202_phrase_grid_pass_fixtures.py
    python3 scripts/validate_observer_audio_mc202_source_phrase_slot_fixtures.py
    python3 scripts/validate_observer_audio_source_timing_alignment_fixtures.py
    python3 scripts/validate_observer_audio_source_grid_metric_fixtures.py
    python3 scripts/validate_observer_audio_w30_loop_closure_fixtures.py
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_lane_recipe_case_phrase_grid.json; then echo "expected invalid lane recipe phrase-grid fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_lane_recipe_case_source_phrase_slot.json; then echo "expected invalid lane recipe source-phrase-slot fixture to fail" >&2; exit 1; fi

user-session-observer-validator-fixtures:
    python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/events.ndjson
    python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_recovery.ndjson
    python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing.ndjson
    python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing_locked_grid.ndjson
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_missing_schema.ndjson; then echo "expected invalid user-session observer fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_source_timing_quality.ndjson; then echo "expected invalid source timing quality fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_source_timing_cue.ndjson; then echo "expected invalid source timing cue fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_source_timing_grid_use.ndjson; then echo "expected invalid source timing grid-use fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_source_timing_detail.ndjson; then echo "expected invalid source timing detail fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_source_timing_anchor_evidence.ndjson; then echo "expected invalid source timing anchor-evidence fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_source_timing_groove_evidence.ndjson; then echo "expected invalid source timing groove-evidence fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_source_timing_locked_warning.ndjson; then echo "expected invalid locked source timing warning fixture to fail" >&2; exit 1; fi
    tmp="$(mktemp)" && jq -c '.snapshot.source_timing.beat_count = 0' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing_locked_grid.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected invalid locked source timing beat-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c '.snapshot.source_timing.bar_count = 0' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing_locked_grid.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected invalid locked source timing bar-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c '.snapshot.source_timing.phrase_count = 0' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing_locked_grid.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected invalid locked source timing phrase-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c '.snapshot.source_timing.phrase_count = 1' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected invalid non-locked source timing phrase-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c '.snapshot.source_timing.bar_count = -1' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected invalid source timing bar-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c 'del(.snapshot.source_timing.actionability)' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected missing source timing actionability fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c '.snapshot.source_timing.performance_readiness.state = "trusted"' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected mismatched performance-readiness state fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c '.snapshot.source_timing.performance_readiness.generated_output_configured = true' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected generated-output aggregation fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c '.snapshot.source_timing.performance_readiness.live_source_policy_active = true' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected untrusted live-policy fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c 'del(.snapshot.source_timing.primary_anchor_cue)' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected missing source timing primary anchor cue fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c 'del(.snapshot.source_timing.alternate_downbeat_phase_count)' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected missing source timing alternate downbeat phase count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c '.snapshot.source_timing.primary_warning_code = "phrase_uncertain"' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected mismatched source timing primary warning fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c '.snapshot.source_map = {"present":true,"mode":"bar grid","trust_label":"grid locked","width":4,"energy_row":"1234","peak_row":"....","grid_row":"|..|","playhead_row":" ^  ","playhead_column":1,"capture_range_row":"[==]","capture_range_available":true,"current_region_label":"bar 1 | section intro","navigation_hint":"prev/next bar ready","capture_hint":"cap next bar"}' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing_locked_grid.ndjson > "$tmp" && python3 scripts/validate_user_session_observer_ndjson.py "$tmp" && rm "$tmp"
    tmp="$(mktemp)" && jq -c '.snapshot.source_map = {"present":true,"mode":"bar grid","trust_label":"grid locked","width":4,"energy_row":"1234","peak_row":"....","grid_row":"|..|","playhead_row":" ^  ","playhead_column":4,"capture_range_row":"[==]","capture_range_available":true,"current_region_label":"bar 1 | section intro","navigation_hint":"prev/next bar ready","capture_hint":"cap next bar"}' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing_locked_grid.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected invalid source map playhead fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq -c '.snapshot.source_map = {"present":true,"mode":"time fallback","trust_label":"needs confirm","width":4,"energy_row":"1234","peak_row":"....","grid_row":"....","playhead_row":" ^  ","playhead_column":1,"capture_range_row":"[==]","capture_range_available":true,"current_region_label":"bar 1 | section intro","navigation_hint":"nav listen first","capture_hint":"cap listen first"}' crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_source_timing.ndjson > "$tmp" && if python3 scripts/validate_user_session_observer_ndjson.py "$tmp"; then echo "expected invalid source map fallback capture-range fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_missing_recovery_decision.ndjson; then echo "expected missing recovery decision fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_missing_replay_family.ndjson; then echo "expected missing recovery replay-family fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_recovery_dry_run_selection.ndjson; then echo "expected selected recovery dry-run fixture to fail" >&2; exit 1; fi

degraded-product-review-fixtures:
    scripts/validate_degraded_product_review_fixtures.sh

source-timing-probe-json-validator-fixtures:
    python3 scripts/validate_source_timing_probe_json.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid.json
    python3 scripts/validate_source_timing_short_loop_fixture.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid.json
    python3 scripts/validate_source_timing_probe_json.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid_locked_grid.json
    python3 scripts/validate_source_timing_locked_grid_fixture.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid_locked_grid.json
    tmp="$(mktemp)" && jq '.primary_beat_count = 0' crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid.json > "$tmp" && if python3 scripts/validate_source_timing_probe_json.py "$tmp"; then echo "expected invalid stable source timing probe beat-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.primary_bar_count = 0' crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid.json > "$tmp" && if python3 scripts/validate_source_timing_probe_json.py "$tmp"; then echo "expected invalid stable source timing probe bar-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.beat_status = "unavailable" | .primary_beat_count = 1' crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid.json > "$tmp" && if python3 scripts/validate_source_timing_probe_json.py "$tmp"; then echo "expected invalid unavailable source timing probe beat-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.downbeat_status = "unavailable" | .primary_bar_count = 1' crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid.json > "$tmp" && if python3 scripts/validate_source_timing_probe_json.py "$tmp"; then echo "expected invalid unavailable source timing probe bar-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    if python3 scripts/validate_source_timing_probe_json.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_invalid_actionability.json; then echo "expected invalid source timing probe actionability fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_source_timing_probe_json.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_invalid_cue.json; then echo "expected invalid source timing probe cue fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_source_timing_probe_json.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_invalid_groove_evidence.json; then echo "expected invalid source timing probe groove fixture to fail" >&2; exit 1; fi
    tmp="$(mktemp)" && jq '.primary_phrase_count = 0' crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid_locked_grid.json > "$tmp" && if python3 scripts/validate_source_timing_probe_json.py "$tmp"; then echo "expected invalid stable source timing probe phrase-count fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.phrase_status = "unavailable"' crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid_locked_grid.json > "$tmp" && if python3 scripts/validate_source_timing_probe_json.py "$tmp"; then echo "expected invalid unavailable source timing probe phrase evidence fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.primary_phrase_count = 1' crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid.json > "$tmp" && if python3 scripts/validate_source_timing_probe_json.py "$tmp"; then echo "expected invalid not-enough-material source timing probe phrase evidence fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    if python3 scripts/validate_source_timing_probe_json.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_invalid_score_range.json; then echo "expected invalid source timing probe score-range fixture to fail" >&2; exit 1; fi

source-timing-grid-use-contract-fixtures:
    python3 scripts/validate_source_timing_grid_use_contract_fixtures.py

generated-source-timing-probe-json-smoke:
    scripts/validate_generated_source_timing_probe_json.sh

generated-degraded-source-timing-probe-json-smoke:
    scripts/validate_generated_degraded_source_timing_probe_json.sh

generated-ambiguous-source-timing-probe-json-smoke:
    scripts/validate_generated_ambiguous_source_timing_probe_json.sh

listening-manifest-validator-fixtures:
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid.json
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_feral_scorecard.json
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_source_timing.json
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_source_grid_output_drift.json
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_lane_source_grid_alignment.json
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/manifest.json
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json
    tmp="$(mktemp)" && jq 'del(.primitive_renderer_boundary)' crates/riotbox-app/tests/fixtures/observer_audio_correlation/manifest.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected primitive-renderer manifest without boundary to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.primitive_renderer_boundary.product_output_allowed = true' crates/riotbox-app/tests/fixtures/observer_audio_correlation/manifest.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected primitive-renderer product-output boundary fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq 'del(.pattern_provenance.tr909_fill.recipe_id)' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive without recipe identity to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq 'del(.pattern_provenance.tr909_fill.selection_inputs)' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive without typed selection inputs to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.pattern_provenance.tr909_fill.affected_artifacts = ["missing.wav"] | .primitive_renderer_boundary.affected_artifacts = ["missing.wav"]' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive with undeclared affected artifact to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.pattern_provenance.tr909_fill.activation_ref = "/gesture_transitions/99" | .primitive_renderer_boundary.activation.references = ["/gesture_transitions/99"]' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive with missing activation reference to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.pattern_provenance.tr909_fill.recipe_id = "hardcoded_missing_source_fallback_v1"' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected unregistered product primitive recipe to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.gesture_transitions[0].command = "automatic.missing_source_fallback"' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive without its registered performer command to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.pattern_provenance.tr909_fill.source_failure_fallback = true' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive with a local source-fallback claim to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.quality_proof = true' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive with contradictory top-level quality proof to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.artifacts[0].role = "source_reference"' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive candidate linked only to a source artifact to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.fake_activation = .gesture_transitions[0] | .pattern_provenance.tr909_fill.activation_ref = "/fake_activation" | .primitive_renderer_boundary.activation.references = ["/fake_activation"]' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive with a non-canonical lookalike activation to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.pattern_provenance.tr909_fill.quality_proof = true' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive with contradictory record-local quality proof to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.artifacts += [{"role":"source_reference","kind":"wav","path":"source.wav","metrics_path":null,"case_id":"source"}] | .pattern_provenance.tr909_fill.affected_artifacts += ["source.wav"] | .primitive_renderer_boundary.affected_artifacts += ["source.wav"]' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive with an unrelated extra source artifact to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.pattern_provenance.tr909_fill.affected_artifacts += ["fill.wav"]' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive with duplicate record-local affected artifacts to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.primitive_renderer_boundary.schema = "riotbox.primitive_renderer_boundary.v1"' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected source-modulated product primitive on legacy boundary v1 to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq 'del(.pattern_provenance.tr909_fill.source_modulation)' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive without registered source modulation to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.pattern_provenance.tr909_fill.source_modulation.source_feature_value = 0.5' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive with inconsistent source-pressure derivation to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.pattern_provenance.tr909_fill.source_evidence_selects_pattern = true' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected fixed product primitive claiming source pattern selection to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.primitive_renderer_boundary.affected_runtime_paths -= ["runtime_mix.source_monitor.blend_fill_focus"]' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_product_primitive.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected product primitive omitting its Blend source focus path to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.primitive_renderer_boundary.affected_paths = ["cases[0].pattern_origin"]' crates/riotbox-app/tests/fixtures/observer_audio_correlation/manifest.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected stale primitive-renderer affected-path fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_missing_artifact_file.json
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_schema_version.json; then echo "expected invalid listening manifest schema fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_boolean_schema_version.json; then echo "expected boolean listening manifest schema fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_artifact.json; then echo "expected invalid listening manifest artifact fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_feral_scorecard.json; then echo "expected invalid feral scorecard fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_missing_source_timing.json; then echo "expected missing source timing fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_missing_source_timing_actionability.json; then echo "expected missing source timing actionability fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_grid_bpm_source.json; then echo "expected invalid grid BPM source fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_grid_bpm_decision_reason.json; then echo "expected invalid grid BPM decision reason fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_source_timing_bpm_delta.json; then echo "expected invalid source timing BPM delta fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_user_override_bpm_agrees_true.json; then echo "expected invalid user override BPM agreement true fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_user_override_bpm_agrees_false.json; then echo "expected invalid user override BPM agreement false fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_static_default_bpm_agrees_true.json; then echo "expected invalid static default BPM agreement true fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_static_default_bpm_agrees_false.json; then echo "expected invalid static default BPM agreement false fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_source_timing_needs_review_ambiguous.json; then echo "expected ambiguous needs-review source-timing fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_source_timing_policy_profile.json; then echo "expected invalid source timing policy profile fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_source_timing_phrase_status.json; then echo "expected invalid source timing phrase-status fixture to fail" >&2; exit 1; fi
    tmp="$(mktemp)" && jq '.source_timing.phrase_status = "stable" | .source_timing.primary_phrase_count = 0' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_source_timing.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected invalid stable source timing phrase-count manifest fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.source_timing.phrase_status = "unavailable"' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_source_timing.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected invalid unavailable source timing phrase evidence manifest fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    tmp="$(mktemp)" && jq '.source_timing.phrase_status = "not_enough_material" | .source_timing.primary_phrase_count = 1' crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_source_timing.json > "$tmp" && if python3 scripts/validate_listening_manifest_json.py "$tmp"; then echo "expected invalid not-enough-material source timing phrase evidence manifest fixture to fail" >&2; rm "$tmp"; exit 1; fi; rm "$tmp"
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_source_timing_anchor_evidence.json; then echo "expected invalid source timing anchor-evidence fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_source_timing_groove_evidence.json; then echo "expected invalid source timing groove-evidence fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_source_grid_output_drift.json; then echo "expected invalid source-grid output drift fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_mc202_lane_source_grid_alignment.json; then echo "expected invalid MC-202 lane source-grid alignment fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_lane_source_grid_alignment.json; then echo "expected invalid lane source-grid alignment fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_tr909_groove_timing.json; then echo "expected invalid TR-909 groove timing fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py --require-existing-artifacts crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_missing_artifact_file.json; then echo "expected missing listening manifest artifact file fixture to fail" >&2; exit 1; fi

source-showcase-diversity manifests:
    python3 scripts/validate_source_showcase_diversity.py {{manifests}}

feral-grid-render-diversity packs:
    python3 scripts/validate_feral_grid_render_diversity.py {{packs}}

feral-grid-render-diversity-fixtures:
    python3 scripts/validate_feral_grid_render_diversity.py --self-test-fixtures

synthetic-fixture-showcase output="artifacts/audio_qa/local-synthetic-fixture-showcase" date="local-synthetic-fixture-showcase" source_seconds="8.0" bars="4":
    scripts/generate_synthetic_fixture_showcase.sh "{{output}}" "{{date}}" "{{source_seconds}}" "{{bars}}"

representative-source-showcase output="artifacts/audio_qa/local-representative-source-showcase" date="local-representative-source-showcase" source_seconds="8.0" bars="4" force_output_reset="":
    echo "Deprecated alias: representative-source-showcase is synthetic fixture QA, not a listening demo." >&2
    scripts/generate_representative_source_showcase.sh "{{output}}" "{{date}}" "{{source_seconds}}" "{{bars}}" "{{force_output_reset}}"

real-source-listening-showcase manifest="data/showcase_sources/local_listening_manifest.json" output="artifacts/audio_qa/local-real-source-listening-showcase" date="local-real-source-listening-showcase":
    python3 scripts/generate_real_source_listening_showcase.py --manifest "{{manifest}}" --output "{{output}}" --date "{{date}}"

real-source-listening-showcase-validate manifest="data/showcase_sources/local_listening_manifest.json":
    python3 scripts/generate_real_source_listening_showcase.py --manifest "{{manifest}}" --validate-only

representative-source-showcase-musical-quality showcase="artifacts/audio_qa/local-representative-source-showcase":
    python3 scripts/validate_representative_showcase_musical_quality.py --json-output "{{showcase}}/validation/musical-quality.json" --markdown-output "{{showcase}}/validation/musical-quality.md" "{{showcase}}"

automated-musical-fitness showcase="artifacts/audio_qa/local-representative-source-showcase":
    mkdir -p "{{showcase}}/validation"
    python3 scripts/validate_automated_musical_fitness.py --json-output "{{showcase}}/validation/automated-musical-fitness.json" --markdown-output "{{showcase}}/validation/automated-musical-fitness.md" "{{showcase}}"

automated-musical-fitness-fixtures:
    scripts/validate_automated_musical_fitness_fixtures.sh

representative-source-showcase-output-guard-fixtures:
    scripts/validate_representative_showcase_output_guard.sh

representative-source-showcase-musical-quality-fixtures:
    tmp="$(mktemp -d)" && python3 scripts/validate_representative_showcase_musical_quality.py --json-output "$tmp/musical-quality.json" --markdown-output "$tmp/musical-quality.md" scripts/fixtures/representative_showcase_musical_quality/valid && jq -e '.schema == "riotbox.representative_showcase_musical_quality.v1" and .result == "pass" and .selected_candidate.listening_verdict == "musically_convincing_candidate" and .selected_candidate.case_id == "tonal_hook_chop" and .passing_candidate_count == 1' "$tmp/musical-quality.json" && grep -q "musically_convincing_candidate" "$tmp/musical-quality.md" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && cp -R scripts/fixtures/representative_showcase_musical_quality/valid/. "$tmp/showcase" && python3 scripts/validate_automated_musical_fitness.py --json-output "$tmp/showcase/validation/automated-musical-fitness.json" "$tmp/showcase" && python3 scripts/validate_representative_showcase_musical_quality.py --json-output "$tmp/musical-quality-with-fitness.json" --markdown-output "$tmp/musical-quality-with-fitness.md" "$tmp/showcase" && jq -e '.automated_musical_fitness.technical_status == "pass" and .automated_musical_fitness.automated_musical_fitness_status == "pass" and .automated_musical_fitness.human_verdict == "unverified" and .automated_musical_fitness.selected_candidate.case_id == "tonal_hook_chop" and (.automated_musical_fitness.score_breakdown.technical_sanity.score | type == "number")' "$tmp/musical-quality-with-fitness.json" && grep -q "Automated Musical Fitness" "$tmp/musical-quality-with-fitness.md" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && cp -R scripts/fixtures/representative_showcase_musical_quality/valid/. "$tmp/showcase" && manifest="$tmp/showcase/packs/tonal_hook_chop/late/manifest.json" && jq 'del(.metrics.tr909_kick_pressure.pattern_origin) | del(.metrics.tr909_kick_pressure.source_evidence_role) | del(.metrics.tr909_kick_pressure.source_profile_reason) | del(.metrics.tr909_source_accent_dynamics)' "$manifest" > "$tmp/mutated.json" && mv "$tmp/mutated.json" "$manifest" && if python3 scripts/validate_representative_showcase_musical_quality.py "$tmp/showcase" >"$tmp/tr909-missing-source-evidence.out" 2>&1; then cat "$tmp/tr909-missing-source-evidence.out" >&2; rm -rf "$tmp"; echo "expected missing TR-909 source evidence fixture to fail" >&2; exit 1; fi && grep -q "tr909_kick_pressure_missing_source_evidence" "$tmp/tr909-missing-source-evidence.out" && grep -q "tr909_accent_dynamics_not_applied" "$tmp/tr909-missing-source-evidence.out" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && if python3 scripts/validate_representative_showcase_musical_quality.py scripts/fixtures/representative_showcase_musical_quality/invalid >"$tmp/invalid.out" 2>&1; then cat "$tmp/invalid.out" >&2; rm -rf "$tmp"; echo "expected musical-quality invalid fixture to fail" >&2; exit 1; fi && grep -q "generated_support_balance_out_of_range" "$tmp/invalid.out" && grep -q "mc202_source_contour_not_applied" "$tmp/invalid.out" && grep -q "all_lane_mix_movement_not_applied" "$tmp/invalid.out" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && if python3 scripts/validate_representative_showcase_musical_quality.py scripts/fixtures/representative_showcase_musical_quality/invalid_static >"$tmp/invalid-static.out" 2>&1; then cat "$tmp/invalid-static.out" >&2; rm -rf "$tmp"; echo "expected static W-30 musical-quality fixture to fail" >&2; exit 1; fi && grep -q "w30_trigger_variation_not_applied" "$tmp/invalid-static.out" && grep -q "w30_slice_choice_not_applied" "$tmp/invalid-static.out" && grep -q "tr909_kick_pressure_not_applied" "$tmp/invalid-static.out" && rm -rf "$tmp"
    tmp="$(mktemp -d)" && if python3 scripts/validate_representative_showcase_musical_quality.py scripts/fixtures/representative_showcase_musical_quality/invalid_mc202_drift >"$tmp/invalid-mc202-drift.out" 2>&1; then cat "$tmp/invalid-mc202-drift.out" >&2; rm -rf "$tmp"; echo "expected drifting MC-202 musical-quality fixture to fail" >&2; exit 1; fi && grep -q "mc202_source_grid_alignment_too_weak" "$tmp/invalid-mc202-drift.out" && grep -q "mc202_source_grid_peak_offset_too_high" "$tmp/invalid-mc202-drift.out" && rm -rf "$tmp"

syncopated-source-showcase-smoke:
    scripts/validate_syncopated_source_showcase_smoke.sh

p011-replay-family-manifest manifest="docs/benchmarks/p011_replay_family_manifest.json":
    python3 scripts/validate_p011_replay_family_manifest.py "{{manifest}}"

p011-exit-evidence-manifest manifest="docs/benchmarks/p011_exit_evidence_manifest.json":
    python3 scripts/validate_p011_exit_evidence_manifest.py "{{manifest}}"

p011-exit-evidence-manifest-validator-fixtures:
    python3 scripts/validate_p011_exit_evidence_manifest.py docs/benchmarks/p011_exit_evidence_manifest.json
    if python3 scripts/validate_p011_exit_evidence_manifest.py docs/benchmarks/fixtures/p011_exit_evidence_manifest_missing_just_recipe.json; then echo "expected missing just recipe fixture to fail" >&2; exit 1; fi

p011-exit-evidence-category-gate category manifest="docs/benchmarks/p011_exit_evidence_manifest.json":
    python3 scripts/run_p011_exit_evidence_category.py "{{category}}" "{{manifest}}"

p011-exit-evidence-category-gate-fixtures:
    python3 scripts/run_p011_exit_evidence_category.py --dry-run replay docs/benchmarks/p011_exit_evidence_manifest.json
    python3 scripts/run_p011_exit_evidence_category.py --dry-run all docs/benchmarks/p011_exit_evidence_manifest.json
    if python3 scripts/run_p011_exit_evidence_category.py --dry-run missing docs/benchmarks/p011_exit_evidence_manifest.json; then echo "expected missing P011 evidence category to fail" >&2; exit 1; fi

p011-exit-evidence-gate manifest="docs/benchmarks/p011_exit_evidence_manifest.json":
    python3 scripts/run_p011_exit_evidence_category.py all "{{manifest}}"

p011-replay-evidence-gate manifest="docs/benchmarks/p011_exit_evidence_manifest.json":
    python3 scripts/run_p011_exit_evidence_category.py replay "{{manifest}}"

p011-recovery-evidence-gate manifest="docs/benchmarks/p011_exit_evidence_manifest.json":
    python3 scripts/run_p011_exit_evidence_category.py recovery "{{manifest}}"

p011-export-evidence-gate manifest="docs/benchmarks/p011_exit_evidence_manifest.json":
    python3 scripts/run_p011_exit_evidence_category.py export_reproducibility "{{manifest}}"

p011-stage-style-evidence-gate manifest="docs/benchmarks/p011_exit_evidence_manifest.json":
    python3 scripts/run_p011_exit_evidence_category.py stage_style_stability "{{manifest}}"

source-showcase-diversity-validator-fixtures:
    python3 scripts/validate_source_showcase_diversity.py crates/riotbox-audio/tests/fixtures/source_showcase_diversity/valid_beat03 crates/riotbox-audio/tests/fixtures/source_showcase_diversity/valid_beat08
    python3 scripts/validate_source_showcase_diversity.py crates/riotbox-audio/tests/fixtures/source_showcase_diversity/valid_close_rms_spectral_a crates/riotbox-audio/tests/fixtures/source_showcase_diversity/valid_close_rms_spectral_b
    if python3 scripts/validate_source_showcase_diversity.py crates/riotbox-audio/tests/fixtures/source_showcase_diversity/invalid_dominated_beat03 crates/riotbox-audio/tests/fixtures/source_showcase_diversity/invalid_dominated_beat08; then echo "expected dominated source-showcase fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_source_showcase_diversity.py crates/riotbox-audio/tests/fixtures/source_showcase_diversity/invalid_full_mix_beat03 crates/riotbox-audio/tests/fixtures/source_showcase_diversity/invalid_full_mix_beat08; then echo "expected identical full-mix source-showcase fixture to fail" >&2; exit 1; fi

source-showcase-diversity-report-fixtures:
    tmp="$(mktemp -d)" && python3 scripts/validate_source_showcase_diversity.py --json-output "$tmp/source-diversity.json" --markdown-output "$tmp/source-diversity.md" crates/riotbox-audio/tests/fixtures/source_showcase_diversity/valid_beat03 crates/riotbox-audio/tests/fixtures/source_showcase_diversity/valid_beat08 && python3 -c 'import json, pathlib, sys; data=json.loads(pathlib.Path(sys.argv[1]).read_text()); assert data["schema"] == "riotbox.source_showcase_diversity.v1"; assert data["result"] == "pass"; assert data["pairwise_role_metrics"]; assert data["generated_dominance"]; assert any(item["role"] == "full_grid_mix" and item["spectral_energy_distance"] is not None for item in data["pairwise_role_metrics"])' "$tmp/source-diversity.json" && grep -q "Pairwise Metrics" "$tmp/source-diversity.md" && rm -rf "$tmp"

listening-manifest-validate-generated-packs:
    scripts/validate_generated_listening_manifests.sh

observer-audio-correlate-fixture:
    cargo run -p riotbox-app --bin observer_audio_correlate -- --observer crates/riotbox-app/tests/fixtures/observer_audio_correlation/events.ndjson --manifest crates/riotbox-app/tests/fixtures/observer_audio_correlation/manifest.json --require-evidence

observer-audio-correlate-generated-feral-grid:
    scripts/correlate_generated_feral_grid_observer.sh

first-playable-jam-probe:
    scripts/validate_first_playable_jam_probe.sh

feral-break-live-review-timing-fixtures:
    python3 scripts/validate_feral_break_live_review_timing.py --timing-only scripts/fixtures/feral_break_live_review_timing/valid.ndjson
    if python3 scripts/validate_feral_break_live_review_timing.py --timing-only scripts/fixtures/feral_break_live_review_timing/invalid_missed_bars.ndjson; then echo "expected missed-bar live review timing fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_feral_break_live_review_timing.py --timing-only scripts/fixtures/feral_break_live_review_timing/invalid_latest_take.ndjson; then echo "expected invalid latest live review take fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_feral_break_live_review_timing.py --timing-only scripts/fixtures/feral_break_live_review_timing/invalid_non_bar_stage.ndjson; then echo "expected non-bar live review stage fixture to fail" >&2; exit 1; fi

source-timing-confirmation-probe:
    scripts/validate_source_timing_confirmation_probe.sh

source-transport-map-capture-probe:
    scripts/validate_source_transport_map_capture_probe.sh

p014-scene-movement-observer-probe:
    scripts/validate_p014_scene_movement_observer_probe.sh

p015-jam-taste-recipe-proof:
    scripts/validate_p015_jam_taste_recipe_proof.sh

stage-style-jam-probe:
    scripts/validate_stage_style_jam_probe.sh

stage-style-restore-diversity-probe:
    scripts/validate_stage_style_restore_diversity_probe.sh

stage-style-snapshot-convergence-smoke:
    cargo test -p riotbox-app stage_style_snapshot_payload_restore_converges_supported_multi_lane_suffix -- --nocapture

interrupted-session-recovery-probe:
    scripts/validate_interrupted_session_recovery_probe.sh

missing-target-recovery-probe:
    scripts/validate_missing_target_recovery_probe.sh

stage-style-stability-smoke:
    scripts/validate_stage_style_stability_smoke.sh

stage-style-stability-proof:
    scripts/validate_stage_style_stability_smoke.sh

stage-style-stability-gate:
    RIOTBOX_STAGE_STYLE_STABILITY_REPETITIONS=3 RIOTBOX_STAGE_STYLE_STABILITY_BARS=8 RIOTBOX_STAGE_STYLE_STABILITY_SOURCE_SECONDS=16.0 RIOTBOX_STAGE_STYLE_STABILITY_SOURCE_WINDOW_SECONDS=2.0 scripts/validate_stage_style_stability_smoke.sh

recipe2-observer-audio-gate:
    scripts/validate_recipe2_observer_audio_gate.sh

p012-all-lane-source-grid-output-proof:
    just observer-audio-correlate-generated-feral-grid
    just recipe2-observer-audio-gate
    just recipe15-feral-grid-auto-proof-strict
    just p012-all-lane-proof-summary artifacts/audio_qa/local/p012_all_lane_source_grid_output_proof_summary.md

offline-render-reproducibility-smoke:
    scripts/validate_offline_render_reproducibility.sh

full-grid-export-reproducibility-smoke:
    scripts/validate_full_grid_export_reproducibility.sh

product-export-reproducibility-smoke:
    scripts/validate_full_grid_export_reproducibility.sh

stem-package-local-ci-report-smoke:
    cargo test -p riotbox-app --test stem_package_report_smoke -- --nocapture

live-recording-readiness-report-smoke:
    cargo test -p riotbox-app --test live_recording_report_smoke -- --nocapture

live-recording-reserved-action-lifecycle-smoke:
    cargo test -p riotbox-app --bin riotbox-app observer_snapshot_reports_rejected_reserved_live_recording_lifecycle_without_receipt -- --nocapture

daw-export-readiness-report-smoke:
    cargo test -p riotbox-app --test daw_export_report_smoke -- --nocapture

daw-session-writer-plan-smoke:
    cargo test -p riotbox-app --test daw_session_writer_plan_smoke -- --nocapture

daw-session-json-writer-smoke:
    cargo test -p riotbox-app --test daw_session_writer_plan_smoke daw_session_json_writer -- --nocapture

daw-session-json-package-execute-smoke:
    cargo test -p riotbox-app --test daw_session_writer_plan_smoke daw_session_json_package_execute -- --nocapture

daw-session-json-package-evidence-apply-smoke:
    cargo test -p riotbox-app --test daw_session_json_package_evidence_apply_smoke -- --nocapture

daw-session-writer-proof-smoke:
    cargo test -p riotbox-app --test daw_session_writer_proof_smoke -- --nocapture

daw-session-writer-export-execute-smoke:
    cargo test -p riotbox-app --test daw_session_writer_proof_smoke daw_session_writer_export_execute -- --nocapture

daw-session-host-import-proof-apply-smoke:
    cargo test -p riotbox-app --test daw_session_host_import_proof_apply_smoke -- --nocapture

daw-session-host-import-proof-export-execute-smoke:
    cargo test -p riotbox-app --test daw_session_host_import_proof_apply_smoke daw_session_host_import_proof_export_execute -- --nocapture

daw-session-audible-output-proof-apply-smoke:
    cargo test -p riotbox-app --test daw_session_audible_output_proof_apply_smoke -- --nocapture

daw-session-json-package-report-smoke:
    cargo test -p riotbox-app --test daw_session_writer_plan_smoke daw_session_json_package_report -- --nocapture

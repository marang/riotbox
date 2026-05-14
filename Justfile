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
    just audio-qa-ci
    cargo clippy --all-targets --all-features -- -D warnings

audio-qa-ci:
    cargo test -p riotbox-audio --bin w30_preview_render --bin w30_preview_compare --bin lane_recipe_pack --bin feral_before_after_pack --bin feral_grid_pack
    cargo test -p riotbox-app --bin observer_audio_correlate
    just observer-audio-correlate-fixture
    just observer-audio-correlate-json-fixture
    just observer-audio-correlate-locked-grid-json-fixture
    just observer-audio-summary-validator-fixtures
    just user-session-observer-validator-fixtures
    just source-timing-probe-json-validator-fixtures
    just source-timing-grid-use-contract-fixtures
    just generated-source-timing-probe-json-smoke
    just generated-degraded-source-timing-probe-json-smoke
    just generated-ambiguous-source-timing-probe-json-smoke
    just listening-manifest-validator-fixtures
    just source-showcase-diversity-validator-fixtures
    just source-showcase-diversity-report-fixtures
    just listening-manifest-validate-generated-packs
    just syncopated-source-showcase-smoke
    just w30-smoke-generated-source-diff
    just observer-audio-correlate-generated-feral-grid
    just first-playable-jam-probe
    just stage-style-jam-probe
    just stage-style-restore-diversity-probe
    just stage-style-snapshot-convergence-smoke
    just interrupted-session-recovery-probe
    just missing-target-recovery-probe
    just stage-style-stability-smoke
    just recipe2-observer-audio-gate
    just offline-render-reproducibility-smoke
    just full-grid-export-reproducibility-smoke

mem-init:
    ./scripts/mempalace.sh init

mem-sync:
    ./scripts/mempalace.sh sync

mem-mine:
    ./scripts/mempalace.sh mine

mem-repair:
    ./scripts/mempalace.sh repair

mem-status:
    ./scripts/mempalace.sh status

mem-search query:
    ./scripts/mempalace.sh search "{{query}}"

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
    cargo test -p riotbox-audio source_timing_probe_keeps_manual_confirmed_real_loop_like_grid_in_review -- --nocapture
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
    output="$(mktemp)"; \
      python3 scripts/source_timing_example_probe_report.py \
        --fixture-json scripts/fixtures/source_timing_example_probe_report/beat08_source_timing_probe.json \
        --fixture-json crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid_locked_grid.json \
        --expectations scripts/fixtures/source_timing_example_probe_report/beat08_expectations.json \
        --output "$output"; \
      grep -q "| Beat08_128BPM(Full).wav | probed | needs confirm | needs_review | yes | short_loop_manual_confirm | 128.397 | stable | stable | not_enough_material | phrase_uncertain | 9/2/4/3 | 1 | ok |" "$output"; \
      grep -q "| long_stable_lock.wav | probed | grid locked | ready | no | locked_grid | 128.397 | stable | stable | stable | none | 11/6/3/2 | 4 | ok |" "$output"; \
      if python3 scripts/source_timing_example_probe_report.py \
        --fixture-json scripts/fixtures/source_timing_example_probe_report/beat08_source_timing_probe.json \
        --expectations scripts/fixtures/source_timing_example_probe_report/beat08_expectations_mismatch.json \
        --output "$output"; then \
        echo "expected mismatched source timing example expectations to fail" >&2; \
        exit 1; \
      fi; \
      grep -q "mismatch:" "$output"; \
      rm -f "$output"

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
    cargo run -p riotbox-audio --bin w30_preview_compare -- --date "{{date}}" --min-rms-delta 0.001 --min-sum-delta 1.0 --max-active-samples-delta 200000 --max-peak-delta 1.0 --max-rms-delta 1.0 --max-sum-delta 1000.0

w30-smoke-generated-source-diff:
    scripts/validate_generated_w30_source_diff.sh

lane-recipe-pack date="local" duration="2.0":
    cargo run -p riotbox-audio --bin lane_recipe_pack -- --date "{{date}}" --duration-seconds "{{duration}}"

feral-before-after source date="local" start="0.0" duration="2.0" source_window="1.0":
    cargo run -p riotbox-audio --bin feral_before_after_pack -- --source "{{source}}" --date "{{date}}" --source-start-seconds "{{start}}" --duration-seconds "{{duration}}" --source-window-seconds "{{source_window}}"

feral-grid-pack source date="local" bpm="auto" bars="8" source_window="1.0" start="0.0":
    if [ "{{bpm}}" = "auto" ]; then \
        cargo run -p riotbox-audio --bin feral_grid_pack -- --source "{{source}}" --date "{{date}}" --bars "{{bars}}" --source-window-seconds "{{source_window}}" --source-start-seconds "{{start}}"; \
    else \
        cargo run -p riotbox-audio --bin feral_grid_pack -- --source "{{source}}" --date "{{date}}" --bpm "{{bpm}}" --bars "{{bars}}" --source-window-seconds "{{source_window}}" --source-start-seconds "{{start}}"; \
    fi

audio-qa-notes target="artifacts/audio_qa/local/notes.md":
    mkdir -p "$(dirname "{{target}}")"
    cp docs/benchmarks/audio_qa_listening_review_template_2026-04-26.md "{{target}}"

observer-audio-correlation-notes target="artifacts/audio_qa/local/observer_audio_correlation.md":
    mkdir -p "$(dirname "{{target}}")"
    cp docs/benchmarks/observer_audio_correlation_template_2026-04-29.md "{{target}}"

observer-audio-correlate observer manifest output="artifacts/audio_qa/local/observer_audio_summary.md":
    cargo run -p riotbox-app --bin observer_audio_correlate -- --observer "{{observer}}" --manifest "{{manifest}}" --output "{{output}}"

observer-audio-correlate-json observer manifest output="artifacts/audio_qa/local/observer_audio_summary.json":
    cargo run -p riotbox-app --bin observer_audio_correlate -- --observer "{{observer}}" --manifest "{{manifest}}" --output "{{output}}" --json

observer-audio-correlate-json-fixture:
    tmp="$(mktemp)" && cargo run -p riotbox-app --bin observer_audio_correlate -- --observer crates/riotbox-app/tests/fixtures/observer_audio_correlation/events.ndjson --manifest crates/riotbox-app/tests/fixtures/observer_audio_correlation/manifest.json --output "$tmp" --json && jq -e '.schema == "riotbox.observer_audio_summary.v1" and .schema_version == 1 and .control_path.present == true and .output_path.present == true and (.output_path.issues | length == 0)' "$tmp" && python3 scripts/validate_observer_audio_summary_json.py "$tmp" && rm "$tmp"

observer-audio-correlate-locked-grid-json-fixture:
    tmp="$(mktemp)" && cargo run -p riotbox-app --bin observer_audio_correlate -- --observer crates/riotbox-app/tests/fixtures/observer_audio_correlation/events_locked_grid.ndjson --manifest crates/riotbox-app/tests/fixtures/observer_audio_correlation/manifest_locked_grid.json --output "$tmp" --json --require-evidence && jq -e '.control_path.observer_source_timing.grid_use == "locked_grid" and .output_path.source_timing.grid_use == "locked_grid" and .output_path.source_timing_alignment.status == "aligned" and .output_path.source_timing_alignment.grid_use_compatibility == "aligned" and .output_path.source_timing_anchor_alignment.status == "aligned" and .output_path.source_timing_groove_alignment.status == "aligned" and (.output_path.issues | length == 0)' "$tmp" && python3 scripts/validate_observer_audio_summary_json.py "$tmp" && rm "$tmp"

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
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_source_timing_alignment_status.json; then echo "expected invalid source timing alignment status fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_source_timing_alignment_grid_use_compatibility.json; then echo "expected invalid source timing alignment grid-use compatibility fixture to fail" >&2; exit 1; fi
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
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_missing_recovery_decision.ndjson; then echo "expected missing recovery decision fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_missing_replay_family.ndjson; then echo "expected missing recovery replay-family fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_recovery_dry_run_selection.ndjson; then echo "expected selected recovery dry-run fixture to fail" >&2; exit 1; fi

source-timing-probe-json-validator-fixtures:
    python3 scripts/validate_source_timing_probe_json.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid.json
    python3 scripts/validate_source_timing_short_loop_fixture.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid.json
    python3 scripts/validate_source_timing_probe_json.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid_locked_grid.json
    python3 scripts/validate_source_timing_locked_grid_fixture.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid_locked_grid.json
    if python3 scripts/validate_source_timing_probe_json.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_invalid_cue.json; then echo "expected invalid source timing probe cue fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_source_timing_probe_json.py crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_invalid_groove_evidence.json; then echo "expected invalid source timing probe groove fixture to fail" >&2; exit 1; fi

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
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_missing_artifact_file.json
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_schema_version.json; then echo "expected invalid listening manifest schema fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_boolean_schema_version.json; then echo "expected boolean listening manifest schema fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_artifact.json; then echo "expected invalid listening manifest artifact fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_feral_scorecard.json; then echo "expected invalid feral scorecard fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_missing_source_timing.json; then echo "expected missing source timing fixture to fail" >&2; exit 1; fi
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
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_source_timing_anchor_evidence.json; then echo "expected invalid source timing anchor-evidence fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_source_timing_groove_evidence.json; then echo "expected invalid source timing groove-evidence fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_source_grid_output_drift.json; then echo "expected invalid source-grid output drift fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_lane_source_grid_alignment.json; then echo "expected invalid lane source-grid alignment fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_tr909_groove_timing.json; then echo "expected invalid TR-909 groove timing fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py --require-existing-artifacts crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_missing_artifact_file.json; then echo "expected missing listening manifest artifact file fixture to fail" >&2; exit 1; fi

source-showcase-diversity manifests:
    python3 scripts/validate_source_showcase_diversity.py {{manifests}}

representative-source-showcase output="artifacts/audio_qa/local-representative-source-showcase" date="local-representative-source-showcase" source_seconds="8.0" bars="4":
    scripts/generate_representative_source_showcase.sh "{{output}}" "{{date}}" "{{source_seconds}}" "{{bars}}"

representative-source-showcase-musical-quality showcase="artifacts/audio_qa/local-representative-source-showcase":
    python3 scripts/validate_representative_showcase_musical_quality.py --json-output "{{showcase}}/validation/musical-quality.json" --markdown-output "{{showcase}}/validation/musical-quality.md" "{{showcase}}"

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

offline-render-reproducibility-smoke:
    scripts/validate_offline_render_reproducibility.sh

full-grid-export-reproducibility-smoke:
    scripts/validate_full_grid_export_reproducibility.sh

product-export-reproducibility-smoke:
    scripts/validate_full_grid_export_reproducibility.sh

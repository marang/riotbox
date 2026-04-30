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
    just audio-qa-ci
    cargo clippy --all-targets --all-features -- -D warnings

audio-qa-ci:
    cargo test -p riotbox-audio --bin w30_preview_render --bin w30_preview_compare --bin lane_recipe_pack --bin feral_before_after_pack --bin feral_grid_pack
    cargo test -p riotbox-app --bin observer_audio_correlate
    just observer-audio-correlate-fixture
    just observer-audio-correlate-json-fixture
    just observer-audio-summary-validator-fixtures
    just user-session-observer-validator-fixtures
    just listening-manifest-validator-fixtures
    just listening-manifest-validate-generated-packs
    just w30-smoke-generated-source-diff
    just observer-audio-correlate-generated-feral-grid
    just first-playable-jam-probe
    just stage-style-jam-probe
    just stage-style-restore-diversity-probe
    just interrupted-session-recovery-probe
    just stage-style-stability-smoke
    just recipe2-observer-audio-gate
    just offline-render-reproducibility-smoke

mem-init:
    ./scripts/mempalace.sh init

mem-sync:
    ./scripts/mempalace.sh sync

mem-mine:
    ./scripts/mempalace.sh mine

mem-status:
    ./scripts/mempalace.sh status

mem-search query:
    ./scripts/mempalace.sh search "{{query}}"

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

feral-grid-pack source date="local" bpm="128.0" bars="8" source_window="1.0" start="0.0":
    cargo run -p riotbox-audio --bin feral_grid_pack -- --source "{{source}}" --date "{{date}}" --bpm "{{bpm}}" --bars "{{bars}}" --source-window-seconds "{{source_window}}" --source-start-seconds "{{start}}"

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

observer-audio-summary-validator-fixtures:
    python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_failure.json
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_schema.json; then echo "expected invalid observer/audio summary fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_observer_audio_summary_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_missing_metric_key.json; then echo "expected missing metric observer/audio summary fixture to fail" >&2; exit 1; fi

user-session-observer-validator-fixtures:
    python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/events.ndjson
    python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_valid_recovery.ndjson
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_missing_schema.ndjson; then echo "expected invalid user-session observer fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_missing_recovery_decision.ndjson; then echo "expected missing recovery decision fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_user_session_observer_ndjson.py crates/riotbox-app/tests/fixtures/user_session_observer/events_invalid_recovery_dry_run_selection.ndjson; then echo "expected selected recovery dry-run fixture to fail" >&2; exit 1; fi

listening-manifest-validator-fixtures:
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid.json
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_feral_scorecard.json
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-app/tests/fixtures/observer_audio_correlation/manifest.json
    python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_missing_artifact_file.json
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_schema_version.json; then echo "expected invalid listening manifest schema fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_boolean_schema_version.json; then echo "expected boolean listening manifest schema fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_artifact.json; then echo "expected invalid listening manifest artifact fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_invalid_feral_scorecard.json; then echo "expected invalid feral scorecard fixture to fail" >&2; exit 1; fi
    if python3 scripts/validate_listening_manifest_json.py --require-existing-artifacts crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_missing_artifact_file.json; then echo "expected missing listening manifest artifact file fixture to fail" >&2; exit 1; fi

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

interrupted-session-recovery-probe:
    scripts/validate_interrupted_session_recovery_probe.sh

stage-style-stability-smoke:
    scripts/validate_stage_style_stability_smoke.sh

recipe2-observer-audio-gate:
    scripts/validate_recipe2_observer_audio_gate.sh

offline-render-reproducibility-smoke:
    scripts/validate_offline_render_reproducibility.sh

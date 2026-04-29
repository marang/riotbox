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

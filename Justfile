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
    cargo clippy --all-targets --all-features -- -D warnings

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

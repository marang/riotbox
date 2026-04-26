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

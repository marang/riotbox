# RIOTBOX-1337 — Semantic library CLI modules

Date: 2026-09-30
Baseline: `800f1dfee708d980990af1b3cec26217193f64a0`
Owning contract: `docs/engineering/module_policy.md`; decision: RBX-384.

## Scope and ownership

Replace the relocated lexical include shell with real Rust modules, without
changing CLI behavior or adding a TUI feature. The binary still calls only
`riotbox_app::cli::run()`. No new action, persistence/replay model, dependency,
audio policy or runtime-local product truth is introduced.

`cli.rs` declares private owners and retains the public `run` export:

- `model` / `args`: transient launch configuration and argument validation;
  DAW argument validation and shared parsing/help have real argument children.
- `launch`: mode dispatch, app loading and recovery-surface attachment.
- `terminal` / `event_loop` / `controls`: terminal lifetime, event routing and
  performer commands through existing app queue/commit paths.
- `observer`: opt-in writer, timestamp and unchanged launch serialization.
- Offline export/report modules: existing semantic mode families, including
  the already-real live-master modules; no second export architecture.
- Real regression modules: CLI, observer and control families, with a separate
  test-only synthetic Ghost fixture owner.

All 45 former production/test includes are removed. The allowlist and current
inventory now describe 203 sites in 16 remaining legacy owners. The old source
shards are rehomed, not discarded; Git retains their history.

## Compatibility evidence

- Pre-migration and post-migration `cargo test --locked -p riotbox-app --lib
  cli::`: 144 passed, none ignored, same scenarios. Logs:
  `/tmp/riotbox-1337-baseline-cli.log`, `/tmp/riotbox-1337-cli-green.log`.
- Source-token inventory accounts for the same 349 named functions, methods,
  types and constants. After visibility and the one qualified parser-path
  adjustment, 341 match exactly. The eight remaining full token diffs contain
  only rustfmt expression braces or optional trailing commas; no changed
  literal, operator, call or control flow. This is supporting mechanical
  evidence, not a general semantic-equivalence proof.
- A temporary Rust probe embeds the exact old help function and its two
  defaults and checks the rebuilt executable. Help bytes, empty stdout,
  stderr placement and legacy exit code 1 match. Log:
  `/tmp/riotbox-1337-help-compatibility.log`.
- `scripts/check_no_textual_includes.sh` and staged diff checks pass.
- Full `just ci` passes, including App/Audio/Core/Sidecar tests, subprocess
  report/export smokes, synthetic source-free audio checks and strict Clippy.
  Logs: `/tmp/riotbox-1337-ci.log`, `/tmp/riotbox-1337-ci-final.log`.
  Integrated totals: App 770, Audio 279, Core 470, Sidecar 24; the shared
  Hook/Chop diagnostic fixture gate also passes after merging current main.

## Solo branch review and self-review

The code-review and Rust review lenses were run sequentially by the same
agent, not an independent panel. Scope: all migrated production/test modules,
explicit import resolution, public/crate-internal compatibility, terminal
cleanup, observer serialization, action ordering, inventory and workflow proof.

No unresolved correctness, architecture, missing-test, Rust-safety or workflow
finding was identified. Compilation exposed mechanical import/visibility
mistakes during implementation; they were corrected before the green test and
Clippy evidence. No behavior fix was bundled into the migration.

Two deliberate review notes: the compatibility re-export has a narrowly scoped
`unused_imports` allowance because its current external caller is a Jam test;
it preserves the former crate-internal function/type path, not a public API.
The 815-line stem-package test family remains cohesive and is not split just
to satisfy line-count pressure. All production owners are below 700 lines.

Follow-up self-review: zero unresolved findings. Native CI is required before
merge. No real source, holdout, commercial reference, DAW/device or playback
access was used; synthetic output proof is not a new human or musical pass.

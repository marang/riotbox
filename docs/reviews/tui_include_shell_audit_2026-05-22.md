# TUI Include Shell Audit 2026-05-22

Context:

- trigger: RIOTBOX-909 after RIOTBOX-908 converted the audio runtime include
  shell into semantic Rust modules
- scope: `crates/riotbox-app/src/ui.rs`, production UI shards, and UI test shards
- review mode: module-ownership audit only, not a visual redesign or behavior
  review

## Summary

The TUI include shell is still acceptable as a behavior-preserving split, but it
is not durable module ownership.

Do not convert the whole TUI shell in one branch. The current shard set is broad
enough that a full conversion would mix import churn, visibility churn, snapshot
risk, and screen ownership decisions. The better path is to convert one low-risk
leaf shard at a time when the conversion clarifies ownership or reduces review
cost.

The first recommended follow-up is RIOTBOX-910: convert the recovery prompt
shard into a real module.

## Current Shape

Production:

- `crates/riotbox-app/src/ui.rs` is a 19-line include shell.
- It textually includes 14 production shards.
- Production shard sizes range from 107 to 493 lines.
- No production UI shard is currently over the roughly 500-line review budget.

Tests:

- `crates/riotbox-app/src/ui/tests.rs` is also a textual include shell.
- It includes 14 test shards.
- `crates/riotbox-app/src/ui/tests/shell_state_log_source.rs` is 518 lines and
  is the only UI shard found over the soft budget in this audit.

## Why A Broad Conversion Should Wait

The current include shell gives every shard one private namespace. That keeps
small UI changes cheap, but it hides real ownership:

- screen roots, panel helpers, footer helpers, recovery prompts, W-30
  diagnostics, and Source panels can call sibling helpers without explicit
  module visibility
- include order makes `types_state.rs` effectively the root context provider
- converting every shard at once would require many `pub(super)` and import
  decisions across render roots, screen bodies, footer/help, capture, source,
  diagnostics, and tests
- snapshot risk would be incidental to module churn rather than tied to a
  musician-facing UI improvement

That conversion would be technically possible, but it would not be a good review
unit.

## Best First Module Boundary

`crates/riotbox-app/src/ui/recovery_prompt.rs` is the best first candidate:

- small at 139 lines
- safety-relevant: it explains manual recovery and no-auto-restore behavior
- clear external surface:
  - `recovery_warning_line(...)`
  - `recovery_help_lines(...)`
- only two current UI call paths need those helpers:
  - Jam warning/source inspect lines
  - footer/help overlay lines
- most helper functions inside the shard can remain private to the module

This is small enough to verify with focused UI snapshot tests and full CI
without changing screen output.

Follow-up:

- RIOTBOX-910: convert `recovery_prompt.rs` into a semantic module

## Deferred Boundaries

Keep these out of the first conversion:

- Source screen panels. They are active and valuable, but their helpers are split
  across `source_timing_panel.rs`, `source_trust_summary.rs`, and source
  identity/section helpers in `capture_log_source_lists.rs`.
- W-30 diagnostics/capture shards. They are near the file-size budget and touch
  many workflow cues, so they should move only with a targeted W-30 UI slice.
- The UI test include shell. It has one over-budget shard, but test conversion
  should follow behavior area ownership rather than happen as mechanical cleanup.

## Recommendation

Keep the TUI include shell for now.

Convert leaf shards incrementally when all of these are true:

- the shard has a clear caller surface
- helper internals can stay private after conversion
- snapshot output stays unchanged
- the branch does not mix module ownership with visual redesign
- the conversion reduces future review cost for an active UI surface

Do not copy the RIOTBOX-908 audio-runtime conversion pattern wholesale into the
TUI. The audio runtime conversion was ready because the production shards already
had semantic realtime responsibilities. The TUI still needs narrower leaf-first
ownership decisions.

# External Review Refresh 2026-05-22

Context:

- trigger: external source/roadmap review described Riotbox as serious prototype,
  not AI slop, but not professional production software yet
- scope: current `main`, source code, review docs, workflow docs, and Linear
  status around the cited architecture and audio-QA concerns
- review mode: freshness check, not a replacement for the original review

## Summary

The external review is directionally useful, but several observations need a
current-state correction before they become backlog work.

The fair high-level reading remains:

- Riotbox is not an empty AI-slop repository.
- Riotbox is not yet professional 1.0 audio software.
- The roadmap and implementation process are credible, but the hard work is
  still timing confidence, musical quality, live stability, export, and release
  hardening.

The current-state corrections are:

- `jam_app.rs` should no longer be described as a monolith. `RIOTBOX-140`
  shipped the split, and `RIOTBOX-690` audited the remaining coupling.
- Audio QA is stronger than the review implied. Current CI and `Justfile` gates
  include source-timing, listener manifest, observer/audio correlation,
  reproducibility, and generated-pack validation. This is real engineering
  proof, but it is still not a commercial audio-quality guarantee.
- The `runtime.rs` criticism remains valid in a narrower form. The current
  `runtime.rs` shell is small, but it used textually included shards. That
  reduced review cost without creating durable Rust module ownership.

## Why A Reviewer Could Still See A `jam_app` Monolith

The old monolith finding has plausible source-level causes:

- the product-facing `jam_app` subsystem is still large even after the root file
  became a small module root
- many child modules historically shared root imports through broad
  `use super::*` boundaries
- commit, runtime-view, queueing, recovery, persistence, and side-effect seams
  still form one orchestration subsystem

That does not mean the current root file is still a monolith. The current
`crates/riotbox-app/src/jam_app.rs` is a wiring root, while the remaining
architecture risk is hidden coupling across the module tree. The canonical
current-state audit is `docs/reviews/riotbox_690_jam_app_boundary_audit_2026-05-10.md`.

Action:

- do not reopen `RIOTBOX-140`
- do not create another broad `jam_app` refactor ticket from the stale wording
- use the existing RIOTBOX-690 findings when a future touched module needs
  narrower import, commit-order, or runtime-view hardening

## Why The Audio-QA Reading Was Too Weak

The review underweighted current audio-QA because a quick source scan can see
many prototype DSP paths before it sees the accumulated proof harness.

Current proof surfaces include:

- `just audio-qa-ci`
- GitHub Rust CI audio-QA smoke gates
- offline render and reproducibility smokes
- listening manifest validators
- observer/audio correlation validators
- source-timing JSON/report validators
- generated feral/source-grid pack validators

This is substantially more than UI/log proof.

The correction is not "audio quality is guaranteed." The honest current claim is:

- Riotbox has meaningful automated audio-output proof for current seams.
- The harness protects against silence, fallback collapse, schema drift,
  source-grid drift, and reproducibility regressions where those gates exist.
- The harness does not yet guarantee professional musical taste, release-grade
  DSP quality, long-session stability, or live-performance readiness.

Those stronger guarantees remain roadmap work across P012 through P019.

## Why The Runtime Model-Cut Criticism Remained Valid

Before this refresh, `crates/riotbox-audio/src/runtime.rs` used `include!` to
hold behavior-preserving shards in one textual module. That was an acceptable
intermediate step under the drift guardrails, but not a durable model cut.

The problem was not file length. The problem was ownership:

- textually included shards shared one private namespace
- sibling responsibilities were harder to make explicit
- realtime shared state, callback state, render helpers, telemetry, and public
  shell behavior were not real Rust module boundaries

Action:

- convert the audio runtime include shell to semantic Rust modules without
  changing public API or audio behavior
- keep the TUI include shell as separate future work, because mixing runtime and
  TUI module conversion would create a broad cleanup branch

## Follow-Up Policy

When an external review produces a plausible finding:

1. check current `main`
2. check current Linear, including done and archived issues when needed
3. check `docs/reviews/`
4. classify the finding as open, stale, duplicate, superseded, or intentionally
   deferred
5. create only the smallest bounded follow-up ticket for findings still open

This refresh classifies the external review as:

- useful high-level product-readiness assessment
- stale on `jam_app.rs` as a root monolith
- incomplete on current audio-QA proof
- still useful on textual include shells, especially audio runtime ownership

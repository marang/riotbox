# `RIOTBOX-80` Add first bounded W-30 loop-freezer reuse cue on the capture seam

- Ticket: `RIOTBOX-80`
- Title: `Add first bounded W-30 loop-freezer reuse cue on the capture seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-80/add-first-bounded-w-30-loop-freezer-reuse-cue-on-the-capture-seam`
- Project: `P007 | W-30 MVP`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-80-w30-loop-freezer-reuse`
- Linear branch: `feature/riotbox-80-add-first-bounded-w-30-loop-freezer-reuse-cue-on-the-capture`
- Assignee: `Markus`
- Labels: `None`
- PR: `#74`
- Merge commit: `086dbb1261d68a3ad0483d8f6f43138ee98812af`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#203`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

The W-30 MVP already had one honest capture, preview, and resample lineage seam, but there was still no bounded way to freeze and reuse a loop on that same path. The next real slice was not a full W-30 editor or browser; it was one replay-safe loop-freezer cue that preserved lineage and pad assignment on the existing seam.

## What Shipped

- added `w30.loop_freeze` to the core action lexicon and Jam view pending-cue summary
- queued the new cue on `NextPhrase` against the current committed W-30 pad target instead of creating a second reuse path
- materialized committed freeze actions as new pinned captures with preserved lineage and reused W-30 bank/pad assignment
- surfaced pending and committed freeze diagnostics across the `Jam`, `Capture`, and `Log` shell screens
- extended the shared `w30_regression.json` corpus so the new cue is covered by both committed-state and shell regressions
- recorded the bounded capture-seam decision in `docs/research_decision_log.md`

## Notes

- this slice stays intentionally bounded: no full W-30 loop browser, no editor workflow, and no second persistence model
- later freezer and reuse work should keep extending the same capture lineage and preview seam unless the roadmap explicitly calls for a larger operator flow

# `RIOTBOX-57` Ticket Archive

- Ticket: `RIOTBOX-57`
- Title: `Surface W-30 audition and recall diagnostics in Capture and Log screens`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-57/surface-w-30-audition-and-recall-diagnostics-in-capture-and-log`
- Project: `P007 | W-30 MVP`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-16`
- Started: `2026-04-16`
- Finished: `2026-04-16`
- Branch: `feature/riotbox-57-w30-diagnostics`
- Linear branch: `feature/riotbox-57-surface-w-30-audition-and-recall-diagnostics-in-capture-and`
- Assignee: `Markus`
- Labels: `None`
- PR: `#51`
- Merge commit: `bd38ba4`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`
- Docs touched: `docs/research_decision_log.md`, `docs/README.md`, `docs/screenshots/w30_diagnostics_baseline.txt`
- Follow-ups: `RIOTBOX-58`, `RIOTBOX-59`

## Why This Ticket Existed

`RIOTBOX-54` made W-30 live recall and promoted-material audition real on the existing queue and commit seam, but operators still had to infer too much of that lane state from generic action history. Riotbox needed one bounded diagnostics slice that made the current W-30 seam legible in the existing shell without opening a second W-30 page, browser, or preview-only control surface.

## What Shipped

- Added a dedicated `W-30 Lane` diagnostics panel to the `Log` screen.
- Deepened `Capture -> Routing / Promotion` with pending cue, promoted-target, focused bank/pad, and last-lane-capture context.
- Added shell regressions for queued recall, queued audition, and committed W-30 diagnostics.
- Added the normalized review artifact at `docs/screenshots/w30_diagnostics_baseline.txt`.

## Notes

- This slice stayed read-only on top of the existing W-30 recall/audition seam and did not introduce a second device model.
- The MC-202 shell regression fixture corpus was relaxed slightly to assert stable semantic tokens instead of width-sensitive `Log` summary strings after the new W-30 diagnostics panel changed line packing.

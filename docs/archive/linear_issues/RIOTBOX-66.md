# `RIOTBOX-66` Ticket Archive

- Ticket: `RIOTBOX-66`
- Title: `Surface W-30 resample lineage diagnostics in Capture and Log`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-66/surface-w-30-resample-lineage-diagnostics-in-capture-and-log`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-66-resample-lineage-diagnostics`
- Linear branch: `feature/riotbox-66-surface-w-30-resample-lineage-diagnostics-in-capture-and-log`
- Assignee: `Markus`
- Labels: `None`
- PR: `#60`
- Merge commit: `3546603`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`, `GitHub Actions rust-ci`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-67`

## Why This Ticket Existed

`RIOTBOX-65` added the first real committed W-30 resample action on the canonical capture-lineage seam, but the shell still hid too much of that lineage once resampled material existed. The next honest slice was to keep lineage, generation depth, and tap-state cues legible inside the current Capture and Log shell spine instead of opening a second W-30 diagnostics surface.

## What Shipped

- Deepened the Capture screen so committed W-30 resample lineage stays visible through compact lineage, generation-depth, and pending-cue summaries.
- Deepened the Log screen so recent W-30 resample actions and committed lineage context are legible without relying on a second diagnostics page.
- Surfaced explicit pending `promote.resample` cues from the actual queue state in shell summaries.
- Added shell regressions covering committed lineage diagnostics and recorded the architectural decision in `docs/research_decision_log.md`.

## Notes

- This slice stayed presentation-only and intentionally did not add new W-30 audio behavior, new capture semantics, or a separate diagnostics surface.
- Later W-30 work should keep extending the existing committed preview and capture-lineage seam rather than splitting lineage visibility into a second W-30-only UI path.

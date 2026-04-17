# `RIOTBOX-61` Ticket Archive

- Ticket: `RIOTBOX-61`
- Title: `Add first playable W-30 pad trigger on the committed preview seam`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-61/add-first-playable-w-30-pad-trigger-on-the-committed-preview-seam`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-16`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-61-playable-w30-pad-trigger`
- Linear branch: `feature/riotbox-61-add-first-playable-w-30-pad-trigger-on-the-committed-preview`
- Assignee: `Markus`
- Labels: `None`
- PR: `#55`
- Merge commit: `91c9e0f`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `git diff --check`, `branch-level code-review`
- Docs touched: `docs/research_decision_log.md`, `docs/specs/action_lexicon_spec.md`
- Follow-ups: `RIOTBOX-62`, `RIOTBOX-63`, `RIOTBOX-64`

## Why This Ticket Existed

`RIOTBOX-60` made the typed W-30 preview seam audible, but the lane still lacked a first playable one-shot pad hit on top of that committed seam. Riotbox needed the smallest honest slice that made the W-30 lane feel more instrument-like without opening a second playback path, a callback-only trigger path, or a shell-only device model.

## What Shipped

- Added committed `w30.trigger_pad` on the existing W-30 lane seam with `NextBeat` quantization and explicit queue blocking against conflicting W-30 pad cues.
- Extended committed W-30 side effects so trigger actions update lane focus, preserve capture lineage, and log a stable result summary.
- Carried trigger revision and trigger velocity through `W30PreviewRenderState` so the existing audio callback can retrigger the current preview accent instead of inventing a second playback architecture.
- Surfaced pending trigger cues in the Jam shell, lane summaries, shell help text, and the app binary event loop.
- Added app, shell, core-view, and audio regression coverage for the new committed trigger path and callback retrigger behavior.

## Notes

- This slice deliberately keeps W-30 pad triggering on the current preview seam rather than pretending full W-30 sample playback already exists.
- Later W-30 work should deepen the same committed preview seam with richer diagnostics, fuller pad-bank interaction, and internal resample taps instead of bypassing it with a separate trigger runtime.

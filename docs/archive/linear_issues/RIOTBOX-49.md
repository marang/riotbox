# `RIOTBOX-49` Ticket Archive

- Ticket: `RIOTBOX-49`
- Title: `Add committed TR-909 scene-lock variation control`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-49/add-committed-tr-909-scene-lock-variation-control`
- Project: `Riotbox MVP Buildout`
- Milestone: `TR-909 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-16`
- Finished: `2026-04-16`
- Branch: `feature/riotbox-49-tr909-scene-lock-variation`
- Linear branch: `feature/riotbox-49-add-committed-tr-909-scene-lock-variation-control`
- Assignee: `Markus`
- Labels: `None`
- PR: `#48`
- Merge commit: `f52c41c`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`
- Docs touched: `docs/research_decision_log.md`, `docs/specs/action_lexicon_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The TR-909 MVP already had committed takeover, release, typed pattern adoption, and typed phrase variation, and the audio path plus fixtures already knew about a `scene_lock` render profile. The remaining gap was that Riotbox still had no honest committed action that could reach that profile through the normal queue and phrase-boundary seam.

## What Shipped

- Added a bounded canonical `tr909.scene_lock` action on the existing `NextPhrase` queue seam.
- Committed the new action into the same takeover lane state and render projection already used by TR-909 takeover and release.
- Surfaced the pending scene-lock profile in the Jam shell while keeping Log visibility on the normal action-history path.
- Recorded the new command in the action lexicon spec and the scene-lock control decision in the research log.

## Notes

- This slice intentionally reused the current takeover seam instead of opening a second TR-909 editor or render-only toggle path.
- Later TR-909 work should keep extending the same committed lane-state and render seam rather than splitting device behavior across separate control models.

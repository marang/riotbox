# `RIOTBOX-98` Tighten the first 30 seconds of play after Riotbox ingest completes

- Ticket: `RIOTBOX-98`
- Title: `Tighten the first 30 seconds of play after Riotbox ingest completes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-98/tighten-the-first-30-seconds-of-play-after-riotbox-ingest-completes`
- Project: `P004 | Jam-First Playable Slice`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-98-first-30-seconds`
- Linear branch: `feature/riotbox-98-tighten-the-first-30-seconds-of-play-after-riotbox-ingest`
- Assignee: `Markus`
- Labels: `None`
- PR: `#90`
- Merge commit: `936beafc96df9116b93ddeba26d093b3f186d980`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#254`
- Docs touched: `docs/research_decision_log.md`, `docs/screenshots/jam_first_30_seconds_baseline.txt`, `docs/README.md`
- Follow-ups: `RIOTBOX-99`

## Why This Ticket Existed

The first-run onramp and the perform-first Jam surface already existed, but the first success path still asked a new user to interpret too much too early. Riotbox needed one clearer first move, one clearer verification step, and one clearer keep-or-undo decision after the first result landed.

## What Shipped

- rewrote the inline `Start Here` block around one explicit first-success loop: start transport, queue a first fill, then confirm it in `Log`
- tightened the queued-first-move guidance to focus on one boundary crossing and one post-landing decision
- tightened the first-result guidance to focus on `capture` versus `undo` before suggesting one more gesture
- aligned the first-run help overlay with the same narrower first-contact flow
- added a normalized baseline artifact and a decision-log entry for the first-30-seconds path

## Notes

- this slice intentionally stayed inside the existing Jam/help surface and did not introduce a new onboarding mode
- the guidance now treats the first success path as singular on purpose; richer post-success onboarding remains follow-up work
- the next UX language slice is now `RIOTBOX-99`, which can refine the broader primary-versus-secondary gesture vocabulary without reopening the first-run architecture

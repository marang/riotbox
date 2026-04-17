# `RIOTBOX-63` Ticket Archive

- Ticket: `RIOTBOX-63`
- Title: `Prepare internal W-30 resample tap seam behind capture lineage`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-63/prepare-internal-w-30-resample-tap-seam-behind-capture-lineage`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-16`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-63-w30-resample-tap-seam`
- Linear branch: `feature/riotbox-63-prepare-internal-w-30-resample-tap-seam-behind-capture`
- Assignee: `Markus`
- Labels: `None`
- PR: `#57`
- Merge commit: `141056d`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `git diff --check`, `branch-level code-review`
- Docs touched: `docs/README.md`, `docs/research_decision_log.md`, `docs/screenshots/w30_resample_tap_baseline.txt`, `docs/specs/session_file_spec.md`
- Follow-ups: `RIOTBOX-64`

## Why This Ticket Existed

`RIOTBOX-59` through `RIOTBOX-62` established the typed W-30 preview seam, made it audible, added the first playable pad trigger, and kept that audible state legible in the shell, but the repo still had no explicit capture-to-capture lineage or typed runtime seam for later internal resample taps. Riotbox needed that next bounded W-30 slice to extend the canonical capture/provenance path instead of inventing a second capture system.

## What Shipped

- Extended `CaptureRef` with explicit lineage metadata and resample-generation depth while keeping old session files backward-compatible through `serde(default)`.
- Added typed `W30ResampleTapState` plus callback-reachable runtime storage in `riotbox-audio`.
- Derived the tap seam in `riotbox-app` from the committed W-30 lane capture, mix context, and current transport state.
- Surfaced the first compact shell proof point inside the Capture screen and recorded the baseline at `docs/screenshots/w30_resample_tap_baseline.txt`.
- Recorded the session-contract and runtime-seam decision in the research decision log and session-file spec instead of leaving the lineage model implicit in code.

## Notes

- This slice prepares the internal resample seam without adding actual resample actions or a second capture inventory.
- Later W-30 lab work should keep extending the same capture lineage path and typed runtime seam rather than introducing a parallel resample runtime.

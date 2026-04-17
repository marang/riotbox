# `RIOTBOX-88` Prepare bounded W-30 slice-pool preview profile on the lineage seam

- Ticket: `RIOTBOX-88`
- Title: `Prepare bounded W-30 slice-pool preview profile on the lineage seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-88/prepare-bounded-w-30-slice-pool-preview-profile-on-the-lineage-seam`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-88-w30-slice-pool-preview-profile`
- Linear branch: `feature/riotbox-88-prepare-bounded-w-30-slice-pool-preview-profile-on-the`
- Assignee: `Markus`
- Labels: `None`
- PR: `#82`
- Merge commit: `b802e91a24d00ba5d1e9409930adafed9cc0ada2`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#227`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-89`

## Why This Ticket Existed

`RIOTBOX-87` proved the committed W-30 slice-pool browse seam, but browse still read and sounded like ordinary promoted recall. The next smallest honest move was to give committed browse one distinct preview consequence on the existing live-recall seam without opening a second W-30 browser or editor path.

## What Shipped

- added a typed `slice_pool_browse` W-30 preview source profile in `riotbox-audio`
- derived that profile from committed `w30.browse_slice_pool` history while keeping preview mode on `live_recall`
- surfaced committed browse state in the Jam shell as `W-30 recall/browse`
- widened app, shell, and audio fixture regressions so browse stays distinct from normal promoted recall

## Notes

- this slice keeps W-30 browse on the same committed pad-lineage seam; it does not add a separate preview mode or richer pool browser
- later slice-pool work can deepen the same seam with better pool UI or cross-pad movement if the roadmap calls for it

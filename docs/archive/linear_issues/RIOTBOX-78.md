# `RIOTBOX-78` Surface W-30 resample-lab diagnostics in the existing shell

- Ticket: `RIOTBOX-78`
- Title: `Surface W-30 resample-lab diagnostics in the existing shell`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-78/surface-w-30-resample-lab-diagnostics-in-the-existing-shell`
- Project: `P007 | W-30 MVP`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-78-w30-resample-diagnostics`
- Linear branch: `feature/riotbox-78-surface-w-30-resample-lab-diagnostics-in-the-existing-shell`
- Assignee: `Markus`
- Labels: `None`
- PR: `#72`
- Merge commit: `618eb1f383df836a919886d5f4cf34c0599722dc`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#197`
- Docs touched: `docs/README.md`, `docs/research_decision_log.md`, `docs/screenshots/w30_resample_lab_diagnostics_baseline.txt`
- Follow-ups: `RIOTBOX-79`, `RIOTBOX-80`

## Why This Ticket Existed

`RIOTBOX-77` made the W-30 internal resample tap audibly real on the current audio callback seam, but the operator surface still hid most of that state behind generic action history and capture lineage. The W-30 MVP needed the shipped resample seam to become legible in the current shell without opening a separate W-30 diagnostics page.

## What Shipped

- deepened the existing `Jam`, `Capture`, and `Log` shell surfaces with compact W-30 resample-lab diagnostics
- kept the slice presentation-only on top of the shipped runtime path instead of adding any new audio or session logic
- added a cross-surface shell regression for the committed resample-lineage path
- updated the shared W-30 shell fixture expectation, decision log, and normalized baseline artifact at `docs/screenshots/w30_resample_lab_diagnostics_baseline.txt`

## Notes

- the slice intentionally improves legibility only and does not change the audible resample behavior shipped in `RIOTBOX-77`
- later W-30 resample work should keep extending the same Jam/Capture/Log spine unless the roadmap explicitly calls for a separate operator page

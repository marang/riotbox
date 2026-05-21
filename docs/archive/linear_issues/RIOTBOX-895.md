# `RIOTBOX-895` Add sparse-onset Source Timing warning for very low-onset sources

- Ticket: `RIOTBOX-895`
- Title: `Add sparse-onset Source Timing warning for very low-onset sources`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-895/add-sparse-onset-source-timing-warning-for-very-low-onset-sources`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-895-sparse-onset-warning`
- Linear branch: `feature/riotbox-895-add-sparse-onset-source-timing-warning-for-very-low-onset`
- Assignee: `Markus`
- Labels: `timing`, `ux`
- PR: `#888 (https://github.com/marang/riotbox/pull/888)`
- Merge commit: `d0f2b7d6315be4a57b07d0eba051a2674f6ac9da`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check && git diff --check`; `cargo test -p riotbox-core source_timing_probe_diagnostics`; `cargo test -p riotbox-core source_timing_summary`; `just source-timing-example-probe-report-local`; `just source-timing-probe-json-validator-fixtures`; `just source-timing-wav-probe`; `just source-timing-example-probe-report-fixtures`; `generated source timing probe smokes`; `cargo test -p riotbox-app ui::tests`; `just ci`; `GitHub Rust CI #2206 passed`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `Continue real-source timing confidence work while keeping sparse sources unavailable unless stronger evidence exists.`

## Why This Ticket Existed

RIOTBOX-895 existed to make very low-onset Source Timing failures more explainable: sparse melodic/pad examples should remain unavailable, but should say the timing evidence lacks enough onsets instead of only reporting generic low-confidence/kick-anchor warnings.

## What Shipped

- Added typed sparse_onsets Source Timing warning.
- Sparse non-silent sources below the onset-count policy minimum remain unavailable but now report sparse_onsets through probe, Jam/Source summary, observer, and local example surfaces.
- Updated Source Timing spec warning priority and local example expectations.

## Notes

- None

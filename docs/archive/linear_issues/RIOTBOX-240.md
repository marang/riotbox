# `RIOTBOX-240` Add fixture-backed TR-909 support accent diagnostic regression

- Ticket: `RIOTBOX-240`
- Title: `Add fixture-backed TR-909 support accent diagnostic regression`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-240/add-fixture-backed-tr-909-support-accent-diagnostic-regression`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-240-tr909-accent-fixture`
- Linear branch: `feature/riotbox-240-add-fixture-backed-tr-909-support-accent-diagnostic`
- PR: `#230`
- Merge commit: `b1ebebc`
- Labels: `benchmark`, `review-followup`
- Follow-ups: `RIOTBOX-241`

## Why This Ticket Existed

`RIOTBOX-238` added the runtime/UI accent cue for TR-909 support diagnostics. The committed render projection fixture already protected profile/context drift, but it did not yet protect the derived support-accent label across `scene`, `off fallback`, and `off` cases.

## What Shipped

- Added `expected_support_accent` to the app TR-909 committed render projection fixture.
- Asserted `state.runtime_view.tr909_render_support_accent` against fixture expectations.
- Covered `accent scene`, `accent off fallback`, and `accent off` rows without changing runtime policy.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app committed_state_fixture_backed_render_projections_hold -- --nocapture`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Diagnostic fixture-only slice; no audio behavior, TUI layout, recipe wording, or Scene policy changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

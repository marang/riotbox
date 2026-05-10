# `RIOTBOX-233` Add fixture-backed TR-909 support-context regression

- Ticket: `RIOTBOX-233`
- Title: `Add fixture-backed TR-909 support-context regression`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-233/add-fixture-backed-tr-909-support-context-regression`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-233-tr909-support-context-fixture`
- Linear branch: `feature/riotbox-233-add-fixture-backed-tr-909-support-context-regression`
- PR: `#223`
- Merge commit: `18d92fc`
- Labels: `benchmark`, `review-followup`
- Follow-ups: `RIOTBOX-234`

## Why This Ticket Existed

`RIOTBOX-231` added `scene_target` / `transport_bar` diagnostics for TR-909 source-support rendering, but the committed render projection fixture still only checked the support profile. The support-context label needed fixture-backed regression coverage so future Scene/TR-909 projection changes do not silently drop it.

## What Shipped

- Extended the TR-909 committed render projection fixture schema with optional expected source-support context.
- Asserted source-support context in both core and app fixture-backed projection tests.
- Added a projected Scene target fixture row that verifies `scene_target` context with the expected `break_lift` support profile.
- Kept non-source-support rows as `null` context and transport fallback rows as `transport_bar`.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-core fixture_backed_render_policy_projection_holds`
- `cargo test -p riotbox-app committed_state_fixture_backed_render_projections_hold`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Fixture/regression slice only; no new audio behavior, TUI redesign, or Scene selection policy changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

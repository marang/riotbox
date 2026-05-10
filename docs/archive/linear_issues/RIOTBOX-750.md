# `RIOTBOX-750` Use MC-202 typed helpers in render projection

- Ticket: `RIOTBOX-750`
- Title: `Use MC-202 typed helpers in render projection`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-750/use-mc-202-typed-helpers-in-render-projection`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `P000 | Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-05-10`
- Started: `2026-05-10`
- Finished: `2026-05-10`
- Branch: `feature/riotbox-750-use-mc-202-typed-helpers-in-render-projection`
- Linear branch: `feature/riotbox-750-use-mc-202-typed-helpers-in-render-projection`
- Assignee: `Markus`
- Labels: `review-followup`
- PR: `#749 (https://github.com/marang/riotbox/pull/749)`
- Merge commit: `afb12ae7281bd5d2399fbc3877734ccc50c1b431`
- Deleted from Linear: `2026-05-10`
- Verification: `cargo fmt --check`; `cargo test -p riotbox-app mc202_render_projection -- --nocapture`; `cargo test -p riotbox-app mc202 -- --nocapture`; `cargo test -p riotbox-core mc202 -- --nocapture`; `cargo test -p riotbox-audio mc202 -- --nocapture`; `cargo test -p riotbox-app strict_evidence_accepts_lane_recipe_manifest_for_recipe2_mc202_cases -- --nocapture`; `just ci`; GitHub Rust CI success on PR #749.
- Docs touched: `None`
- Follow-ups: `RIOTBOX-751`, `RIOTBOX-752`

## Why This Ticket Existed

The MC-202 typed-contract migration had already moved queue, side effects, and replay onto typed role and phrase helper boundaries. Render projection still branched on raw role strings, which left a stringly-state drift risk between Session/replay/render.

## What Shipped

- Parsed persisted MC-202 role labels through `Mc202RoleState` before render projection.
- Derived MC-202 render mode and base phrase shape from typed roles.
- Derived mutated phrase shape through `Mc202PhraseIntentState::from_phrase_variant`.
- Kept unknown role labels projected as idle/silent instead of creating a second render path.
- Added render projection tests proving typed `pressure + mutated_drive` output is non-silent and unknown-role projection stays silent.

## Notes

- No new `ActionCommand` was added.
- No session JSON shape changed in this slice.
- No `JamAppState` state was added.

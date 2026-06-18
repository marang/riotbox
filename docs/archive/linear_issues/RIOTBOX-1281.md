# `RIOTBOX-1281` Remove primitive_renderer as positive musical output in lane recipe and feral-grid packs

- Ticket: `RIOTBOX-1281`
- Title: `Remove primitive_renderer as positive musical output in lane recipe and feral-grid packs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1281/remove-primitive-renderer-as-positive-musical-output-in-lane-recipe`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-15`
- Started: `2026-06-18`
- Finished: `2026-06-18`
- Branch: `feature/riotbox-1281-remove-primitive-renderer-positive-output`
- Linear branch: `feature/riotbox-1281-remove-primitive_renderer-as-positive-musical-output-in-lane`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1254 (https://github.com/marang/riotbox/pull/1254)`
- Merge commit: `a930ea4083a2c7e2f2184e98c15437e78227c182`
- Deleted from Linear: `2026-06-18`
- Verification: `just ci: pass; GitHub rust-ci: pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `Continue replacing primitive renderer controls with source-composed product lanes; primitive controls remain diagnostic only.`

## Why This Ticket Existed

Primitive renderer output remained useful as a diagnostic control but could still appear in listening manifests without a machine-enforced non-product boundary.

## What Shipped

- Listening manifests containing pattern_origin primitive_renderer now require primitive_renderer_boundary metadata marking the output non-product, not quality proof, unverified, and promotion-blocked with exact affected paths; feral-grid, lane-recipe, observer fixtures, and smoke validators enforce the rule.

## Notes

- No product fallback sound was added; this slice labels and validates existing primitive evidence boundaries.

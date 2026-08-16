# `RIOTBOX-1438` P023: Make a source-backed cut–hit–changed-return gesture land audibly in the live mix

- Ticket: `RIOTBOX-1438`
- Title: `P023: Make a source-backed cut–hit–changed-return gesture land audibly in the live mix`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1438/p023-make-a-source-backed-cut-hit-changed-return-gesture-land-audibly`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-08-15`
- Started: `2026-08-15`
- Finished: `2026-08-16`
- Branch: `feature/riotbox-1438-source-backed-cut-hit-return`
- Linear branch: `feature/riotbox-1438-p023-make-a-source-backed-cuthitchanged-return-gesture-land`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`, `Feral`
- PR: `#1396 (https://github.com/marang/riotbox/pull/1396)`
- Merge commit: `d2a2b5dbed75273f72ee6b5778dfad340639cb36`
- Deleted from Linear: `2026-08-16`
- Verification: `cargo test -p riotbox-app --lib: 634 passed; just ci: pass; GitHub rust-ci: pass`
- Docs touched: `docs/reviews/riotbox_1438_cut_hit_return_development_2026-08-15.md`, `docs/benchmarks/tr909_cut_hit_return_development_v1.json`
- Follow-ups: `RIOTBOX-1439 corrects discovery-versus-qualification order; RIOTBOX-1417 is the next narrowed audible follow-up.`

## Why This Ticket Existed

Test whether composing the existing source-backed Fill and Slam vocabulary as one cut-hit-changed-return gesture creates stronger arrangement impact.

## What Shipped

- Frozen Development contract, exact technical and structured human negative evidence, and RBX-293/RBX-294 landed.
- The unqualified one-key product gesture and qualification implementation were removed; existing useful transformations remain unchanged.

## Notes

- Human review found both underlying transformations useful and source-clear, but the complete arcs substantially similar apart from B opening. No Holdout or commercial-reference audio was accessed.

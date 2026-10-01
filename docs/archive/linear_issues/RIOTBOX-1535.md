# `RIOTBOX-1535` Reject Feral grid pack output paths that overwrite the input source

- Ticket: `RIOTBOX-1535`
- Title: `Reject Feral grid pack output paths that overwrite the input source`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1535/reject-feral-grid-pack-output-paths-that-overwrite-the-input-source`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1535-source-output-collision`
- Linear branch: `feature/riotbox-1535-reject-feral-grid-pack-output-paths-that-overwrite-the-input`
- Assignee: `Markus`
- Labels: `Audio`, `review-followup`
- PR: `#1614 (https://github.com/marang/riotbox/pull/1614)`
- Merge commit: `6d6de8d3efa32246b8c6b21156a2588e09958d93`
- Deleted from Linear: `2026-10-01`
- Verification: `Actual CLI red/green; six source-preservation CLI regressions Debug/Release plus retained 52 unit and two integrations; exact 31 normal CLI records and 39 hashes; both manifests; full frozen source-free just ci actual exit zero; exact-head native Rust and Windows transport SUCCESS; solo review.`
- Docs touched: `docs/reviews/riotbox_1535_source_output_preservation_2026-10-01.md`, `docs/specs/audio_qa/manifests_and_artifacts.md`
- Follow-ups: `RIOTBOX-1536: independently reproduced W-30 preview source loss; reuse the physical identity guard in a binary-only common owner after this merge.`

## Why This Ticket Existed

A successful offline Feral pack could overwrite its own original input through any planned artifact pathname or filesystem alias.

## What Shipped

- One typed 19-artifact plan consumed by writers and physical-identity preflight; reject source collisions and unknown existing destinations before artifact replacement; preserve ordinary output bytes.

## Notes

- RBX-405. Stable-namespace preflight only; directory creation can precede rejection; no concurrent namespace lock/atomic pack/power-loss or Windows Audio proof. Only generated synthetic controls; no real audio, playback, DAW or subagents.

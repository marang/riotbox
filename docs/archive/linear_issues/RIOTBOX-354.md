# `RIOTBOX-354` Split audio runtime and runtime tests below 500-line budget

- Ticket: `RIOTBOX-354`
- Title: `Split audio runtime and runtime tests below 500-line budget`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-354/split-audio-runtime-and-runtime-tests-below-500-line-budget`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-354-split-audio-runtime-files`
- Linear branch: `feature/riotbox-354-split-audio-runtime-and-runtime-tests-below-500-line-budget`
- Assignee: `Markus`
- Labels: `review-followup`, `workflow`
- PR: `#344`
- Merge commit: `d58130eaf06a3485b3ec94ddab69292afc30c4ae`
- Deleted from Linear: `2026-04-29`
- Verification: `cargo test -p riotbox-audio`, `cargo test`, `just ci`, GitHub Actions `rust-ci`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-355`, `RIOTBOX-356`

## Why This Ticket Existed

The audio runtime and runtime tests are central to Riotbox's output correctness, but they had become too large to review efficiently. The realtime and offline render seams needed smaller files without changing behavior.

## What Shipped

- replaced `crates/riotbox-audio/src/runtime.rs` with a small include index
- moved runtime implementation into behavior-area files under `crates/riotbox-audio/src/runtime/`
- split runtime tests by render seam and fixture group
- preserved audio behavior while reducing review scope

## Notes

- this was a mechanical organization slice
- no audio synthesis behavior changed

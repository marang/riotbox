# `RIOTBOX-289` Add just targets for W-30 smoke QA helpers

- Ticket: `RIOTBOX-289`
- Title: `Add just targets for W-30 smoke QA helpers`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-289/add-just-targets-for-w-30-smoke-qa-helpers`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `P000 | Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-289-add-just-targets-for-w-30-smoke-qa-helpers`
- Linear branch: `feature/riotbox-289-add-just-targets-for-w-30-smoke-qa-helpers`
- PR: `#279`
- Merge commit: `41a63d6`
- Labels: `workflow`, `benchmark`
- Follow-ups: `RIOTBOX-290`

## Why This Ticket Existed

The W-30 smoke QA path had render and compare binaries plus a local artifact convention, but users and agents still had to type long `cargo run` invocations. A thin Justfile wrapper made the path repeatable without changing the underlying helpers.

## What Shipped

- Added `just w30-smoke-candidate`, `w30-smoke-baseline`, `w30-smoke-compare`, and `w30-smoke-qa`.
- Documented the short wrappers in the W-30 preview smoke pack convention.
- Added the combined smoke QA wrapper to the agent command shortlist.

## Verification

- `just --list`
- `just w30-smoke-qa 2026-04-26 0.1`
- Verified generated artifacts remain under ignored `artifacts/audio_qa/`.
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Command ergonomics only; no generated artifact CI gate, baseline promotion workflow, waveform/perceptual audio diff, or broad multi-pack runner changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.

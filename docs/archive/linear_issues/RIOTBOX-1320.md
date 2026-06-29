# `RIOTBOX-1320` P023: Incorporate improvement README into roadmap and follow-up backlog

- Ticket: `RIOTBOX-1320`
- Title: `P023: Incorporate improvement README into roadmap and follow-up backlog`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1320/p023-incorporate-improvement-readme-into-roadmap-and-follow-up-backlog`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1320-p023-incorporate-improvement-readme-into-roadmap-and-follow`
- Linear branch: `feature/riotbox-1320-p023-incorporate-improvement-readme-into-roadmap-and-follow`
- Assignee: `Markus`
- Labels: `Docs`, `workflow`
- PR: `#1294 (https://github.com/marang/riotbox/pull/1294)`
- Merge commit: `919588fd64f8a5dcee5047f739b6b4d194791b74`
- Deleted from Linear: `2026-06-29`
- Verification: `git diff --check; local linked-doc existence check; GitHub rust-ci pass`
- Docs touched: `AGENTS.md; docs/README.md; docs/engineering/module_policy.md; docs/plans/riotbox_improvement_tracks_plan.md; docs/execution_roadmap.md; docs/phase_definition_of_done.md; docs/research_decision_log.md; docs/specs/audio_core_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/rust_engineering_guidelines.md`
- Follow-ups: `Created RIOTBOX-1321 through RIOTBOX-1336 for include inventory/migrations, runtime audio quality, source timing, TR-909, MC-202, W-30, sidecar, first-playable UX, and Scene Brain proof.`

## Why This Ticket Existed

A temporary improvement README mixed module policy, runtime hardening, musical roadmap, sidecar/QA/UX, and ticket proposals. The durable parts needed to be evaluated, split, incorporated into canonical docs, and converted into bounded backlog without leaving a parallel plan file.

## What Shipped

- Added module policy and improvement-track plan, updated roadmap/DoD/Rust/audio/QA/README/AGENTS/decision-log docs, removed the temporary README, and created the follow-up backlog.

## Notes

- Accepted direction keeps audible instrument quality as the primary product goal while making code quality non-optional; broad musical quick wins are split by lane/seam, and musical fallback output remains forbidden on product paths.

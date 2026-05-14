# `RIOTBOX-794` Move MC-202 phrase-slot timing proof toward real source timing

- Ticket: `RIOTBOX-794`
- Title: `Move MC-202 phrase-slot timing proof toward real source timing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-794/move-mc-202-phrase-slot-timing-proof-toward-real-source-timing`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-794-real-source-mc202-phrase-proof`
- Linear branch: `feature/riotbox-794-move-mc-202-phrase-slot-timing-proof-toward-real-source`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#789 (https://github.com/marang/riotbox/pull/789)`
- Merge commit: `c76b2718a15f3719ba36b8265a7818bdf885dfd2`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-794-lane-recipe-all-tests.log cargo test -p riotbox-audio --bin lane_recipe_pack`; `scripts/run_compact.sh /tmp/riotbox-794-recipe2-gate.log just recipe2-observer-audio-gate`; `scripts/run_compact.sh /tmp/riotbox-794-audio-qa-ci.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-794-fmt-after-docwrap.log cargo fmt --check`; `git diff --check`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P012 needed MC-202 phrase-slot timing proof to move away from a hand-built synthetic TimingModel and toward the existing Source Timing contract path.

## What Shipped

- Replaced the hand-built lane recipe source TimingModel with generated onset-grid evidence passed through timing_model_from_probe_bpm_candidates; tests now assert probe-bpm provenance and docs describe the bounded CI-safe bridge.

## Notes

- No rendered audio behavior changed; timing proof provenance changed and remains explicitly not an arbitrary-source phrase arranger.

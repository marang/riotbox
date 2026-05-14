# `RIOTBOX-796` Drive MC-202 lane recipe phrase timing from generated probe audio

- Ticket: `RIOTBOX-796`
- Title: `Drive MC-202 lane recipe phrase timing from generated probe audio`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-796/drive-mc-202-lane-recipe-phrase-timing-from-generated-probe-audio`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-796-generated-probe-audio-mc202-phrase`
- Linear branch: `feature/riotbox-796-drive-mc-202-lane-recipe-phrase-timing-from-generated-probe`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#790 (https://github.com/marang/riotbox/pull/790)`
- Merge commit: `d4a51f67912e51150130797b1ef3ba56cf179bde`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-796-source-audio-tests.log cargo test -p riotbox-audio source_audio::tests`; `scripts/run_compact.sh /tmp/riotbox-796-lane-recipe-tests.log cargo test -p riotbox-audio --bin lane_recipe_pack`; `scripts/run_compact.sh /tmp/riotbox-796-recipe2-gate.log just recipe2-observer-audio-gate`; `scripts/run_compact.sh /tmp/riotbox-796-audio-qa-ci.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-796-clippy-fix.log cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings`; `scripts/run_compact.sh /tmp/riotbox-796-fmt-fix.log cargo fmt --check`; `git diff --check`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P012 needed MC-202 lane recipe phrase-slot proof to exercise the audio-derived Source Timing probe seam instead of feeding generated onset evidence directly.

## What Shipped

- Added a validated in-memory SourceAudioCache constructor and drove the lane recipe MC-202 source timing bridge from generated PCM probe audio through analyze_source_timing_probe and the probe-BPM TimingModel path.

## Notes

- No rendered product audio changed; generated QA probe audio is internal to timing proof and remains a bounded CI-safe bridge, not an arbitrary-source phrase arranger.

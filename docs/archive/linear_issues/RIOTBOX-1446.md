# `RIOTBOX-1446` P023: Integrate and qualify the frozen W-30 Filter Slam

- Ticket: `RIOTBOX-1446`
- Title: `P023: Integrate and qualify the frozen W-30 Filter Slam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1446/p023-integrate-and-qualify-the-frozen-w-30-filter-slam`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-08-19`
- Started: `2026-08-19`
- Finished: `2026-08-19`
- Branch: `feature/riotbox-1446-p023-integrate-and-qualify-the-frozen-w-30-filter-slam`
- Linear branch: `feature/riotbox-1446-p023-integrate-and-qualify-the-frozen-w-30-filter-slam`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`
- PR: `#1412 (https://github.com/marang/riotbox/pull/1412)`
- Merge commit: `76164353b58343c993dab2aa11c33387d6564650`
- Deleted from Linear: `2026-08-19`
- Verification: `just ci: passed`; `GitHub Rust CI: passed`; `RBX-305 four-source Development matrix: passed`; `formal representative listening review: keep`
- Docs touched: `docs/benchmarks/w30_filter_slam_product_qualification_v1.json`, `docs/reviews/riotbox_1446_w30_filter_slam_product_qualification_2026-08-19.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_core_spec.md`, `docs/specs/replay_model_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

## Outcome

Rebuild the exact source-blind `w30_filter_slam_v1` contract frozen by RBX-304 as a real performer-owned Riotbox action, then qualify that unchanged implementation on fresh registered Development sources and one formal human review.

## Frozen boundary

* Contract: `docs/benchmarks/w30_filter_slam_development_v1.json`
* Decision: RBX-304
* Gesture: `w30.filter_slam`, quantized to `NextBar`
* Target: current focused promoted W-30 capture
* Duration and curve: exactly eight beats with the frozen cutoff/Q timeline and 20 ms filtered-to-dry return
* Render order: after existing source-backed W-30 control, before shared mix/limiting
* No Holdout or commercial-reference access
* Qualification may reject v1 but may not shorten, lengthen, retune, reorder, or remap it; any such change requires a new version and Decision

## Required product surfaces

1. Queue path: typed `ActionCommand` and `NextBar` scheduling.
2. Commit/audio path: exact RBX-304 RuntimeMix behavior on the focused promoted W-30 capture.
3. Session/replay: typed deterministic gesture state, restore, and replay consequence.
4. Observer/UI: clear queued/active/completed state and visible refusal when source state is missing or inapplicable.
5. Test/QA: unit, replay/restore, observer, runtime-audio, refusal, and unchanged-other-lanes proof.

## Acceptance criteria

* Implement source-blind before opening any qualification source.
* Missing/inapplicable source refuses visibly and produces no fallback audio.
* Ordinary W-30 output is sample-exact after the frozen 20 ms return.
* Source PCM, playback rate, gate/reverse, grit, bus level, Source Monitor, and other lanes remain unchanged.
* Fresh source-diverse Development qualification follows exact registered paths and a bounded access log, fail-closed on any contract breach.
* One technically preflighted formal human review determines musician-facing fitness.
* No Golden Path, Holdout, universal source-quality, hardness, demo, or release claim is implied.

## What Shipped

- Added performer-owned w30.filter_slam with NextBar queue/commit and visible refusal for unavailable source state.
- Persisted typed FilterSlamV1 capture identity and start beat through Session, restore, replay, observer, UI, and exact RuntimeMix projection.
- Passed the immutable four-source Development matrix and one formal human keep review without retuning RBX-304.

## Notes

- The effect uses one prepared per-output-channel Direct Form II Transposed RBJ low-pass state and allocates nothing in the realtime callback.
- No Holdout or commercial-reference audio was opened; claims exclude hardness, universal source quality, complete Golden Path, demo, and release readiness.

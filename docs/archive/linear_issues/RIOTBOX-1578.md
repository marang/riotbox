# `RIOTBOX-1578` — Add isolated loaded-recorder observation adapter

- Ticket: `RIOTBOX-1578`
- Title: `Add isolated loaded-recorder observation adapter`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1578/add-isolated-loaded-recorder-observation-adapter`
- Project: `P017 | Live Performance Readiness`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-15`
- Started: `2026-10-15`
- Finished: `2026-10-15`
- Branch: `feature/riotbox-1578-add-isolated-loaded-recorder-observation-adapter`
- Linear branch: `feature/riotbox-1578-add-isolated-loaded-recorder-observation-adapter`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: [#1681](https://github.com/marang/riotbox/pull/1681)
- Merge commit: `92a89ca1d7abb2092713f8d9ea5ec5974a5d65c2`
- Deleted from Linear: `2026-10-16`
- Verification: Full local `just ci` PASS; final 165 existing silent + 74 loaded controls PASS; 18 shared-controller lifecycle controls and overlapping 80-test live-master family PASS; fresh generated six-window preparation, three sibling copies and metadata-only Core restores PASS; native PR CI 38706255441 and post-merge CI 38707331483 PASS.
- Docs touched: `docs/benchmarks/loaded_recorder_observation_v1.json`, `docs/benchmarks/loaded_recorder_observation_v1.md`, `docs/engineering/module_policy.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1578_loaded_recorder_adapter_2026-10-15.md`, `docs/specs/audio_core_spec.md`, `docs/specs/audio_qa/automated_qa.md`
- Follow-ups: RIOTBOX-1579 separately authorized V1 observation subsequently FAILED/consumed; RIOTBOX-1580 versioned correction and RIOTBOX-1581 separate V2 subsequently completed; RIOTBOX-1041/1036 remain open; original ALSA cause unexplained.

## Why This Ticket Existed

RIOTBOX-1577 supplied an identity-bound generated Session, but the isolated operator only understood silent interval drivers. One complete adapter was needed for the existing actual live-master CLI without creating a second recorder or granting host execution.

## What Shipped

- Added an optional typed completion observation to the existing production controller: timestamp in memory, immediate existing stop, verified Stopped, then observer write before finalization/publication; observer failures remain visible and no-observer behavior is preserved.

- Admitted exact generated V1 identities and exclusive sibling run Sessions with one-use copy/output rechecks before hydration; baseline Session/source/graph/capture identities remain distinct from Core pre-capture hashes.

- Added a closed 2/8/16-bar workload reusing the owned sink, process/watchdog/signal supervision, routes, deadlines, ALSA diagnostics and cleanup. Query-start/end intervals distinguish post-completion absence from premature or ambiguous disappearance.

- Correlated final JSON, observer, WAV/proof, Session/action/receipt/Core identities, unchanged baseline and diagnostics only after verified process/stream removal; retained computed failed metrics.

- Froze loaded observation V1 / RBX-444, owning specs and generated source-free controls without changing DSP, recording policy, callbacks or source policy.

## Notes

- Exact reviewed head: `a1a6d7180585d8d8b9176d2b86083b31ab65e9c5`; complete merge tree was identical. Independent Spec/Evidence and Maintainer/Rust/adversarial reviews plus self-review retained zero findings after P1 pre-hydration copy integrity, P2 computed-failure retention and P3 clipped RMS fixes.

- Coverage families overlap and are not additive; final late-repair generated rechecks are recorded separately. Authors' own code was excluded from independent-review credit.

- Only generated preparation/admission/copy/metadata restore ran; no actual host/service/device/runtime or old-attempt/source access, DAW or playback. The optional milestone is post-stop and adds no pre-stop/callback I/O.

- This adapter required a separate exact reviewed prospective fence, binding and fresh explicit authorization before any actual loaded run. Ordinary execution CLI remained blocked; all historical phases stayed consumed.

- Historical main/reference synchronization and exact feature-branch removal were verified without switching or modifying then-active RIOTBOX-1579 work.

- This archive preserves historical scope, verification and outcomes only. It grants no new source, host, device, consumed-attempt, playback or trial access.

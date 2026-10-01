# `RIOTBOX-1550` Freeze source-free limiter baseline protocol and exact RuntimeMix control proof

- Ticket: `RIOTBOX-1550`
- Title: `Freeze source-free limiter baseline protocol and exact RuntimeMix control proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1550/freeze-source-free-limiter-baseline-protocol-and-exact-runtimemix`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1550-limiter-baseline-protocol`
- Linear branch: `feature/riotbox-1550-freeze-source-free-limiter-baseline-protocol-and-exact`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1646 (https://github.com/marang/riotbox/pull/1646)`
- Merge commit: `bda080871c1437a7c496b4fd742c3ee54e75112e`
- Deleted from Linear: `2026-10-01`
- Verification: `Final just ci actual zero/warning-free; Debug/Release same 133 executed plus same ignored benchmark, all 129 original names/statuses and 11 old complete functions preserved; 19 computed final serial reports unchanged. fmt/diff/zero-include and solo Rust/spec/evidence/self review pass. Native run 36867443299 both Rust/Windows SUCCESS, fresh reviews/comments empty, CLEAN, exact-head matched merge. Full feature candidate equals merged main; normal fast-forward verified.`
- Docs touched: `docs/benchmarks/master_bus_limiter_baseline_protocol_v1.md`, `docs/reviews/riotbox_1550_limiter_baseline_protocol_2026-10-01.md`, `docs/specs/audio_core_spec.md`, `docs/engineering/audio_numeric_values.md`, `docs/engineering/audio_numeric_inventory_2026-09-22.md`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1501 remains incomplete: preregister exact authorized Development inputs/gestures, bounded alternatives and stopping budget before source results; artifact preflight and fresh structured human review before calibration decision. No indefinite framework chain.`

## Why This Ticket Existed

RIOTBOX-1501 lacked a preregistered source-free baseline/control contract. Existing 0.92/0.985 protection values were provisional, not calibrated; actual EPSILON write semantics and all five current mix contributors needed honest baseline evidence.

## What Shipped

- Versioned generated in-memory baseline protocol/RBX-417 plus five tests in two existing runtime owners. Signed-neighbor, overload, sparse/sustained, partition/repeat and five-owner isolated/leave-one-out controls preserve existing DSP/constants/APIs and all prior tests. Owning audio spec/numeric guide/inventory link the bounded baseline, not calibration.

## Notes

- No production DSP/runtime/API/constants/Core/Session/replay/Cargo/Stage-A changes, new limiter/executor/framework, gain compensation or automatic selection. No real Development/Holdout/commercial audio, directory discovery, playback, DAW/device or subagents. Intentional overload is protection-only and fails clean-product gates; no musical/hardness/human/true-peak/device/hearing-safety/release pass. Keep failed fixture/Clippy diagnostics distinct from final accepted proof. Archive preserves context before exact cleanup.

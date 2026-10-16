# `RIOTBOX-1501` — Calibrate and document the inherited master-bus limiter headroom contract

- Ticket: `RIOTBOX-1501`
- Title: `Calibrate and document the inherited master-bus limiter headroom contract`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1501/calibrate-and-document-the-inherited-master-bus-limiter-headroom`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-09-22`
- Started: `2026-10-02`
- Finished: `2026-10-02`
- Branch: `feature/riotbox-1501-retain-baseline-decision`
- Linear branch: `feature/riotbox-1501-calibrate-and-document-the-inherited-master-bus-limiter`
- Assignee: `Markus`
- Labels: `review-followup`, `Audio`
- PR: [#1649](https://github.com/marang/riotbox/pull/1649), [#1650](https://github.com/marang/riotbox/pull/1650), [#1651](https://github.com/marang/riotbox/pull/1651), [#1652](https://github.com/marang/riotbox/pull/1652), [#1653](https://github.com/marang/riotbox/pull/1653) (final decision: #1653)
- Merge commit: `92f4f9b954f405e4949b83a230b6eb93a66dfac6`
- Deleted from Linear: `2026-10-16`
- Verification: Historical: source-free `just ci`, generated limiter controls, bounded Development measurements and final structured-review validation; final native CI 37010210724 passed. Current archival check: Linear Done and merged PR metadata only; no tests or audio rerun.
- Docs touched: `docs/benchmarks/master_bus_limiter_calibration_protocol_v1_draft.md`, `docs/engineering/audio_numeric_values.md`, `docs/benchmarks/master_bus_limiter_calibration_protocol_v1.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1501_limiter_development_comparison_2026-10-02.md`, `docs/specs/audio_core_spec.md`, `docs/benchmarks/master_bus_limiter_calibration_protocol_v2.md`, `docs/reviews/riotbox_1501_limiter_overload_v2_2026-10-02.md`, `docs/benchmarks/master_bus_limiter_review_artifact_protocol_v1.md`, `docs/reviews/riotbox_1501_limiter_review_artifacts_2026-10-02.md`, `docs/engineering/audio_numeric_inventory_2026-09-22.md`, `docs/reviews/riotbox_1501_limiter_retention_decision_2026-10-02.md`
- Follow-ups: RIOTBOX-1550 supplies the prior source-free baseline; RIOTBOX-1509 Windows work was deferred at implementation closeout. No further indistinguishable listening or tuning is required; any materially new failure needs a new bounded contract and access authorization.

## Why This Ticket Existed

RIOTBOX-1420 found regression-tested but historically uncalibrated master-bus limiter values: knee 0.92 and ceiling 0.985. Neither an observed Fill peak nor a passing synthetic test established an optimal setting. P023 needed an explicitly bounded sample-protection decision, separate from clean-path QA, perceived loudness/hardness and true-peak/device safety.

## What Shipped

- RBX-421 retains unchanged production A: f32 knee `0.92`, ceiling `0.985`, the existing samplewise tanh function and actual-write predicate. This closes the explicitly accepted baseline-retention path, not a perceptual-optimum calibration.
- The source-free protocol draft (#1649) preceded accepted RBX-418 Development V1 (#1650), RBX-419 fixed-overload V2 (#1651), separately authorized RBX-420 fixed-window artifacts (#1652), and the final retention decision (#1653). Production limiter policy, live callback and frozen historical evidence remain unchanged.
- A default-off diagnostic seam shares the production limiter and existing offline RuntimeMix. One admitted byte buffer owns ordinary preparation/Session/capture evidence; no second mixer, action or persistence truth was introduced.
- The owning Audio spec and numeric guide/inventory now distinguish finite sample protection from exact clean-product QA, which still requires zero limiter writes and zero pre/post clips. No compensating gain or weak-output boost is permitted.

## Notes

- The original acceptance required explicit headroom/equality semantics, all-owner overload controls, preregistered alternatives and Development cases, artifact-bound human evidence when useful, and either a justified change or measured baseline retention. The final report maps all five criteria.
- Nominal knee/ceiling are −0.7242/−0.1313 dBFS with 0.015 linear ceiling headroom. `abs(x) <= knee` passes; shaped samples count as writes only when the change exceeds `f32::EPSILON`. Prior RIOTBOX-1550 five-owner overload reduced sample peak 1.5052301 / 1,364 pre-clips to 0.985 / zero sample clips, with 1,652 writes; this intentionally fails clean-path QA.
- V1's one authorized three-source Linux session passed admission, repeat/partition and clean gates. Even fixed 2x Dense/Tonal/Sparse peaks (0.799794/0.537297/0.783375) stayed below 0.92, so all 18 A/B/C outputs were byte-identical with zero writes. Protection and policy preference were unobserved; there was no retry or useful human comparison.
- V2 was separately frozen before a fresh three-file session: nine mix passes, 27 outputs and three ordinary captures. Historical clean/2x identities stayed exact; fixed 4x exercised every policy with zero output sample clips. A/B/C write counts were Dense 4984/3208/5024, Tonal 104/32/110 and Sparse 964/728/972. This is finite diagnostic protection, not a quality or preference verdict.
- The separately authorized artifact phase used one new exact Sparse source delivery, reproduced all V2 identities, and published exactly three two-second 48-kHz stereo float32 files from Sparse stress_4x frames `[0,96000)`. A single shared presentation attenuation was not a production gain change. Original source budgets for V1, V2 and artifacts are consumed and closed; no Holdout/commercial access or source-directory discovery occurred.
- Human evidence: the listener perceived no A/B/C difference and selected no preference (`human_verdict: inconclusive`, comparative policy selection only). The requested unchanged replay repeated each two-second file five times, ten seconds per candidate, A → B → C; normal exits and monitor-confirmed silence were reported. A separate original-source audition was orientation only, not a source-recognition or quality verdict.
- The earlier pumping observation and pumping-driven follow-up were explicitly withdrawn. They are superseded provenance, not a current defect, diagnosis, tuning target or blocker. Withdrawal proves neither presence nor absence of pumping and does not grant a quality pass.
- Review history: four V1 pre-access P2 findings (executable binding, limiter failure observations, preparation provenance and inherited clean gates) were fixed and independently rechecked. V2 size/pin/import and delta-RMS wording issues, artifact foreign-checkout fixture failures and final raw-FFT/presented-ffmpeg attribution wording were corrected without post-access algorithm changes or extra renders. Final independent review and self-review retained no open findings.
- The unattenuated reviewed crops measured about +2.518/+2.517/+2.376 dBTP with the specified conservative estimator, despite sample ceilings. Thus this decision is expressly not true-peak, device or hearing-safety protection. Three familiar Development cases and one Sparse listening interval add no source-general, hardness, demo, musical-quality, Windows, release or P023-completion claim.
- Earlier issue notes saying Todo / no archive described incomplete intermediate slices. The final 2026-10-02 user-authorized retention decision superseded them; the current issue is Done. Its original closeout retained recent history temporarily, with archive-before-deletion still required.

- Archive-only verification on 2026-10-16 re-read the full current Linear issue and all comments, checked attached GitHub PR bodies/branches/merge commits and successful retained native checks, and used tracked review metadata where cited. All implementation tests, benchmarks, source access, host and human outcomes above are historical evidence, not checks rerun by this cleanup. Linear's attachment reader does not fetch external GitHub URLs; those links were verified directly with `gh pr view`. Metadata dates use the Linear UTC timestamps.

## Implementation PR History

| PR | Actual implementation branch | Merge commit |
| --- | --- | --- |
| [#1649](https://github.com/marang/riotbox/pull/1649) | `feature/riotbox-1501-calibration-protocol-draft` | `da1227edc917609c5e0704c8665219642fbf67db` |
| [#1650](https://github.com/marang/riotbox/pull/1650) | `feature/riotbox-1501-development-comparison` | `c4472c286469ce7f89f2d0b35d7df611007ae4ec` |
| [#1651](https://github.com/marang/riotbox/pull/1651) | `feature/riotbox-1501-overload-comparison-v2` | `91ad8c46ab689feef418b1e79a32f23a48b2e95a` |
| [#1652](https://github.com/marang/riotbox/pull/1652) | `feature/riotbox-1501-fixed-window-review-artifacts` | `08a87c480d905b83efc59167612cf93ad00be3f1` |
| [#1653](https://github.com/marang/riotbox/pull/1653) | `feature/riotbox-1501-retain-baseline-decision` | `92f4f9b954f405e4949b83a230b6eb93a66dfac6` |

## Retained Human Evidence Identity

- A WAV: `5a346e98ad5c647e87894e3570e27a38d6c960710ca16b7ce148ea0a4ca9d6a1`.
- B WAV: `7a4fac4c4c03a71d4741708380faaea46e49fd1f05d997ab1130f49853a4628f`.
- C WAV: `5d7fff7b0b90ab05c8660c51236adf65d3004d3d9212f05680e3337054a431c4`.
- Final structured review: `0fc5eaf3869314cd0f121ca846b3163e7a044ce690bc7b1f484da32a11440df8`.
- Superseded pre-withdrawal review: `192948cb913ad34fb89375e8a34259b851be1794e9056b39b7e9e30fc98c6d65`.
- Unchanged pre-feedback assessment: `8dfa48ed44872dd57a934a5c31ccdf23fa1be913189063ffeb990eab93f5cfb0`.
- These are historical identities recorded in the merged retention report, not artifacts reopened or rehashed during archival.

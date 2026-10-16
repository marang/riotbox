# RIOTBOX-1594 — bound the existing restore-history join

Date: 2026-10-16. Maintenance protecting P023 recall responsiveness and existing
recording/export lineage under RIOTBOX-1041. Base: RIOTBOX-1593 merge
`1d190c582fcc97e29cce4125acf12d231c7cb87b`.

## Risk and repair

The existing private App `mc202_phrase_replay_inputs` scans the action prefix for
each commit record before testing MC-202 eligibility. For aligned unique
n-action/n-record histories, even all-TransportPlay controls perform
`n(n+1)/2` ID comparisons. Construction and successful target/payload restore
invoke this collector on the control side, not the realtime callback.

One transient BTreeMap now indexes the clamped prefix. First-entry-wins insertion
preserves the old scan's first match, including an earlier ineligible duplicate
masking a later eligible action in unvalidated `from_parts` inputs. Every stored
record still traverses the same filters in the same order, with the same action,
boundary and persisted-plan clones. No reconstruction, last-plan shortcut,
timing-trust predicate or fallback behavior changes.

For prefix length k and m records, the join now performs k insertions and m
lookups, each bounded by the tree lookup cost, rather than m prefix scans.
Additional transient storage is O(k). This is a source-level complexity argument,
not a new machine-dependent CI timing assertion or another public test hook.
No new domain contract, algorithm threshold or architectural decision is needed
for this semantics-preserving implementation detail; RBX-454 remains authority.

## Verification

Four characterization tests passed against the original collector, both before
and after integrating RIOTBOX-1593. They also pass after replacement, including
all three declared replay policies and every prefix through one-past-end.
The private old-scan oracle compares complete action/boundary/plan values for
nonmonotonic IDs, backward/permuted records, missing/future/repeated records,
status/result combinations and duplicate-action masking. Reconstruction controls
retain trusted/untrusted/missing last-plan, missing-graph and empty-input rules.
These adversarial low-level inputs do not claim Core admission or valid V2 logs.

Logs: `/tmp/riotbox-1594-join-characterization.log`,
`/tmp/riotbox-1594-characterization-after-1593.log`,
`/tmp/riotbox-1594-join-indexed.log`. Strict all-target/all-feature Clippy passes
(`/tmp/riotbox-1594-clippy-early.log`). Full App library: 901 passed, one existing
informational benchmark ignored (`/tmp/riotbox-1594-app-suite.log`). Independent
Rust/adversarial and spec/evidence reviews retain no P0–P3 findings. Complete
local/native CI remain mandatory PR gates;
their final status belongs to PR/Linear closeout, not an assumed focused pass.

The identical standalone generated public App restore diagnostic was separately
linked to baseline/candidate libraries. Each row is the median of three calls;
both retain every action and apply the complete requested history. Sequential
Debug observations, not an isolated collector benchmark or host qualification.

The retained baseline was built on RIOTBOX-1593 implementation head
`e5b97ff1baca0cc2d0a66e5b600b5f89e633cc0e`, not pre-1593 main. Its production
tree matches the integrated base above: only two test-style files differ between
those commits. The candidate was rebuilt from that integrated base with this
join change. Therefore no old-versus-new Core chronology implementation is
substituted into this comparison.

| Actions | Baseline microseconds | Indexed microseconds |
| ---: | ---: | ---: |
| 1,000 | 16,599 | 18,208 |
| 4,000 | 117,812 | 78,477 |
| 16,000 | 1,183,351 | 364,087 |

The small case is not faster in this observation. No universal speedup,
production latency, memory limit or hard realtime claim follows.
Source `/tmp/riotbox-restore-scaling-probe.rs`, SHA-256
`b2b4fd0784135500e9f72c5c96a4326505be569fcd309b499e766a7381ff83d7`.
Retained baseline binary SHA-256
`503e30801ebc488f6c788ba273a74a8708c8e9c4d7df569429a5175cfa550f13`;
candidate `4c70a9a74e6a9513d37a4f3b3cf9ee5cd8346f3e0785fbc8bc07f7d796575ec2`.
Logs: `/tmp/riotbox-1594-scaling-{baseline-repeat,candidate}.log`.

## Boundaries

One existing App owner and one Core replay authority. No public API/test
constructor, ActionCommand, persistent/App state, cache, source/DSP algorithm,
sound recipe/threshold, Session/recording schema or dependency change. No actual
source/capture/Holdout/commercial/ignored-attempt access, host/device/service
observation/runtime, DAW or human playback. Generated metadata and regression
evidence are not musical/source-general/human/hardness/latency/endurance/release
verdicts. No Windows/TUI expansion; RIOTBOX-1041/1036 remain open.

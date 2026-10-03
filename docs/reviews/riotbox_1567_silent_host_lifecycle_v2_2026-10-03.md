# Silent-host lifecycle V2 repair — RIOTBOX-1567

Date: 2026-10-03. Maintenance/regression only. Both established V1 tooling
defects are repaired and covered by generated metadata/process tests. **No new
host attempt, audio-service access or host pass.** Independent integrated
review is clear and local source-free CI passed. Exact-head native PR CI is
still a merge gate, pending at this report boundary.

## Scope and frozen identity

- Branch: `feature/riotbox-1567-silent-host-lifetime-v2`.
- Review base: `37403fdfcb79fc09dc6965e44dd674a9767de750` (PR #1669 merge).
- Owner: P017 / RIOTBOX-1041 observation seam on the accepted P023 callback
  path; repair the failed measuring tool, not a new musical mechanism.
- [V2 contract](../benchmarks/silent_host_observation_v2.md), frozen before
  implementation under RBX-437. JSON SHA-256:
  `9b8517f13285e36a6082fb57131602949c89758199d57ccef42a26f4a542bcea`.
- V1 JSON SHA-256 remains
  `ba26bc50db85a6b745738917100689a7ca5ac3c2aa95cc7c699db2ef404358d3`.
  Every existing numeric budget and non-version field is preserved in V2.
- V1 JSON, protocol, [failed report](riotbox_1566_silent_host_observation_2026-10-03.md)
  and original ignored attempt/cleanup evidence remain unchanged. The executed
  implementation remains available at
  `cf4f76664a50ba8f054e5e2a9fc19016092c89dd`; no parallel V1 copy is added.

## Behavior and regression evidence

1. **P2 recycled object ID falsely blocked teardown — fixed.** Route admission
   now records typed Node ID plus `object.serial`, alongside Client and stereo
   links. Active serial replacement fails admission continuity. Removal accepts
   a recycled Client ID or a genuinely different Node lifetime, but still
   rejects the original orphan Node after Client, links and target disappear.
   Missing/malformed identity, inconsistent serial relocation, target Nodes,
   foreign links and late removal remain failures. Generated RED/GREEN pairs
   reproduce the false rejection and cover these negative controls; complete
   generated child-process runs verify both ID-reuse shapes and child reaping.
2. **P2 nonexistent Pulse module JSON index — fixed.** A cohesive module adapter
   reads explicit short text (index/name/arguments) instead of inferring a JSON
   field or array position. It preserves empty tab columns, validates index
   range/uniqueness and exact null-sink/stereo arguments, rejects ambiguity and
   observed replacements, and unloads at most once. Module and Node absence
   must fit the original total deadline, including queries and unload. Seventeen
   generated module tests and integrated attempt/deadline controls pass. The
   original actual-shaped no-index JSON reproduces `KeyError('index')` before
   replacement; the supported text format is tied to upstream implementation
   evidence in the V2 contract, not a newly queried host.
3. **Execution remains closed.** The old explicit CLI opt-in is rejected before
   file creation, host inspection or subprocess launch. The internal generated
   attempt seam requires a caller-supplied temporary owner; no real attempt
   owner or alternative launch command is added. V2 acceptance still consumes
   the unchanged Rust driver's `riotbox.silent_host_sample.v1` records with
   identical schedule/budgets. Existing independent watchdog, signal, route-log
   failure, uncertain-cleanup and result-publication behavior is retained.

Focused reproduction logs are ephemeral `/tmp/riotbox-1567-*.log` files;
committed tests and contracts are the repeatable evidence. No log is a human
listening or real-host result.

## Review and disposition

The first independent Spec And Evidence audit identified a stale signal-test
entry path after the CLI was disabled. It was reproduced: TERM/HUP/INT could
not reach the generated process. The test now enters `OperatorInterrupts` and
the generated orchestration directly, without reopening the CLI. All four
signal test functions pass; their subprocess loop includes all three signals.

The integrated Spec And Evidence audit found one P2 in the extracted module
cleanup: the old filter could treat malformed PipeWire metadata (`{}`, an empty
string, or a Node without required fields) as proof of sink absence. The
coordinator reproduced it with generated inputs, then reused strict snapshot
validation and required well-formed object types and Node names before absence
can pass. Malformed metadata now produces an explicit evidence failure; empty
valid snapshots and unrelated valid objects remain accepted. Dedicated
RED/GREEN and the full generated suite verify this correction. This was an
inherited validation hole, not evidence of a new real-host failure.

Final independent Spec And Evidence rereview has zero retained P0–P3 findings
and ran 75 focused generated tests. The Adversarial Implementation and
Performance/Operations pass independently ran 28 identity/version/module tests
and 31 operator/deadline/signal/attempt tests, with no retained findings. Its
Devil's Advocate assessment retains the documented sampled/non-atomic limit;
the source-free-only scope and disabled CLI do not claim to solve that limit.
The coordinator's Maintainer/Product Pragmatist pass checked all changed
callers, explicit V2 result shape, module ownership and drift: no new
ActionCommand, JamAppState, Session/Core/replay state, dependencies, Rust or
audio-producing behavior. The Pulse extraction replaces the old functions;
there is no second lifecycle path or general test framework.
The follow-up coordinator self-review found zero remaining findings. All
retained findings were fixed; none were deferred or rejected to clear the PR.

## Verification and limits

- `just silent-host-contract-fixtures`: **89 tests passed**, generated only,
  with ResourceWarnings treated as errors.
- Module integration RED/GREEN: three focused attempt/cleanup/deadline tests.
- `git diff --check`: passed; V1 tracked artifacts and `crates/` have no diff.
- Coordinator verification: exact original attempt result and both cleanup
  JSON hashes still match the V1 report; no original evidence was rewritten.
- `just ci`: passed. The final review correction also passed the separate
  complete 89-test fixture rerun; source-free PR gate only, not `ci-broad`.
- Exact-head native PR CI: pending; this report does not preclaim its result.

Sampling and later module unload are not an atomic compare-and-unload protocol;
observed replacements fail, but a malicious inter-command replacement is not
covered. No current host compatibility, endurance, latency/xrun, loaded-runtime,
physical-device, musical, hardness or release qualification is established.
RIOTBOX-1041 remains open. A new real-host observation needs its own prospective
phase/owner authorization; neither this repair nor green fixtures grant it.

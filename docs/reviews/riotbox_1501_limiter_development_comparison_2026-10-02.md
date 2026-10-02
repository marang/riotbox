# RIOTBOX-1501 — bounded limiter Development comparison

Date: 2026-10-02. Base: `da1227edc917609c5e0704c8665219642fbf67db`.
Classification: diagnostic calibration, no production policy change.

## Before source access

The user approved continuing with at most three exact Development sources
after complete preregistration. Windows compatibility work was separately
deferred; RIOTBOX-1509 is Backlog / Low, not resolved. Existing CI is unchanged.

The [new protocol](../benchmarks/master_bus_limiter_calibration_protocol_v1.md)
and its JSON bind the familiar Dense/Tonal/Sparse identities, historical
Graph/Session metadata, exact committed owners, identical pre-PCM, A/B/C,
fixed 2x diagnostic stress, measurement windows, access budget and stopping
rule. Holdouts, commercial references, source discovery and playback remain
closed. The source-free baseline and design draft are unchanged.

Implementation reuses the production limiter and offline RuntimeMix loop;
the default-off feature retains pre-PCM without touching the live callback.
The App adapter constructs an ordinary fresh Session from admitted bytes and
an existing Graph, preserving artifact-backed capture and queue/commit truth.
No original-file reopen, analysis pass, new ActionCommand or persistent app
state was added. Source timing is explicitly confirmed: Dense's historical
130-BPM Session must not silently become its approximately 130.285-BPM probe.

Tonal/Sparse native bit depth was absent from the inspected metadata. Their
finite PCM16/PCM24 admission is declared before source access, not inferred
from presentation WAVs. Exact-width Stage-A callers remain unchanged. The
current v3 registry is used only for protected identity/path/hash exclusion;
these three MusicRadar/SampleRadar cases belong to a separate registered corpus.

## Review findings and disposition

Independent reviewers inspected the Audio and App seams in separate passes,
then the executor, guard, preparation and measurement contracts. All review
work used code, declared metadata or generated fixtures, never original audio.

| Finding | Origin | Correction and verification |
| --- | --- | --- |
| P2: a present ignored executable was hashed without binding it to the reviewed build | Adversarial Implementation Reviewer | Explicit locked native build before source access; compiler-artifact feature/path and unchanged Git identity checked. Generated build-binding test rejects a missing feature. |
| P2: rejected child execution discarded computed limiter observations | Spec And Evidence Auditor | Bounded structured failure diagnostics and parent-side retention before rejecting exit status. Generated hot control records pre-overload, writes and post-protection, without proceeding to stress. |
| P2: failed preparation/comparison lost available Action/Commit/Capture provenance | Spec And Evidence Auditor | Retain the actual Session snapshot on preparation/evidence failure and validated preparation on comparison failure. Generated six-commit/capture regression proves retained records, timing and window below the diagnostic size bound; independent re-review closed the finding. |
| P2: peak-nonzero clean gate was weaker than the inherited W30 activity and Sparse mix-RMS gates | Spec And Evidence Auditor, coordinator self-review | Retain the existing positive active-sample count and Sparse whole-mix minimum before stress; generated regression checks. No added isolated-lane/full-journey claim. |

The Audio independent review retained no findings: production parameter values,
f32 operation order and actual-write predicate are unchanged. The App
independent review retained no findings: same-byte hash/decode, rounded-duration
compatibility, explicit BPM confirmation and ordinary capture semantics are
covered by generated tests. A known bounded size remains in the cohesive
diagnostic runner plus tests; no mechanical include shards or generic validator
framework were added.

Final independent re-review and coordinator self-review retain zero unresolved
findings. The coordinator additionally checked the exact preselected listening
window: changes later in a render cannot count as an observed difference inside
that window. No policy or source access was selected from test outcomes.

## Verification snapshot

- Metadata-only preflight: all three bound Graph/Session/corpus records pass;
  no original source audio opened.
- Full source-free `just ci`: initial pass in
  `/tmp/riotbox-1501-comparison-ci.log`, followed by final accepted-protocol pass
  in `/tmp/riotbox-1501-final-preaccess-ci.log` after all implementation fixes.
- Audio feature tests: nine passed, including exact inherited stateless
  controls for A/B/C. Eighteen computed reports precede assertions in
  `/tmp/riotbox-1501-abc-frozen-stateless-catalog.log`.
- The five-owner hot composite remains the unchanged A-only baseline control;
  it is not claimed as a completed A/B/C composite catalog.
- App feature tests: eight passed, independently repeated.
- Runner feature tests: ten passed, including the complete synthetic Sparse
  eleven-action path at beat 17, capture/roles, nonzero output, 128/128/257
  bit parity, failure provenance and inherited clean gates.
- Final independent `just limiter-calibration-fixtures`: all four test groups
  passed (Audio 9, App 8, runner 10, Python 10); log
  `/tmp/riotbox-1501-final-independent-review-fixtures.log`.
- Default-off builds and feature-enabled Clippy passed for the affected crates.
- Python metric/execution regressions: ten passed, including degenerate
  windows, exact alignment, inherited gates, stale-build rejection, retained
  child failure and exclusion of policy differences outside the preselected
  future listening window.

## Execution state

Not yet executed. The reviewed protocol is accepted under RBX-418 and final
source-free verification passed; execution still requires a clean commit. No real source,
candidate output, listening artifact, device or human review was used. This
report is not a calibration decision; RIOTBOX-1501 remains In Progress.

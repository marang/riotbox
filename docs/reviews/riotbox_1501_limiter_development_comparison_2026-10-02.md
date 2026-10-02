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

## One bounded execution

Execution succeeded on Linux at 2026-10-02 09:09:46–09:10:06 UTC, after the
reviewed implementation was committed with a clean worktree:

- Implementation: `2b24c44d974f58487aa3f23b3653debc23d1d6d2`.
- Protocol SHA-256: `c9d72c910aea77045a25c30ecc8a5bfec2e66481a9762aa2ba8827aa0f94a42d`.
- Native feature-build binary SHA-256: `13fe903bc262617b97eb6c09c2370b29ef77cc8d469da94c8d7b210961d3d557`.
- Access session: `08c1a823-3d62-40aa-acdb-4221e469b13f`.
- Local report: `artifacts/development/riotbox-1501/calibration-v1/comparison.json`;
  SHA-256 `7f8cecb384357b2ea725ef916707320c51299f1d26adf577df2e644b4de206d8`.
- Local access log: `artifacts/development/riotbox-1501/calibration-v1/development-access.json`;
  SHA-256 `00fb0f950e311151cfb223ffa36afd58b691562414d7920aca46a0dc0e35f414`.
- Command exited 0; `/tmp/riotbox-1501-development-execution.log`.

The access log records exactly the three declared originals, every expected
hash matched, and no directory discovery. Dense was PCM24 / 162831 native
frames; Tonal and Sparse were PCM16 / 176400 frames each, all 44.1-kHz stereo.
These actual widths are results, not retrospective changes to admission.
The access layer deliberately says `not_evaluated_by_access_layer`; technical
comparison success comes from the separate owner report, not source admission.

All three cases passed 128-frame repeat, 257-frame partition and production-A
bit parity. Dense/Tonal each retained six committed actions and the declared
capture window; Sparse retained eleven and beat-17 W-30/TR-909/MC-202 ownership.
Three ordinary derived captures were produced. Nine RuntimeMix passes and
eighteen in-memory policy outputs stayed within the budget; no comparison WAV,
original reopen, analyzer pass, Holdout/commercial access or playback occurred.

## Measured result and limits

Values below are the original Rust f32 reports, rounded for display. Full f64
descriptors, local windows and raw-f32 output identities remain in the report.

| Case | Clean peak | Clean RMS | Fixed 2x peak | Actual writes A/B/C, clean and 2x |
| --- | ---: | ---: | ---: | --- |
| Dense | 0.399897 | 0.069313 | 0.799794 | 0 / 0 / 0 |
| Tonal | 0.268649 | 0.107563 | 0.537297 | 0 / 0 / 0 |
| Sparse | 0.391687 | 0.064179 | 0.783375 | 0 / 0 / 0 |

Every clean path had positive activity, zero pre/post clips and zero modified
samples; Sparse retained its minimum RMS gate. In **every** source/condition,
A/B/C outputs were bit-identical, had identical raw-f32 hashes and zero deltas.
Even the fixed 2x challenges remained below the lowest knee of 0.92. Thus this
run supports clean-path preservation for these exact gestures, but **does not
exercise source-backed protection or discriminate between policies**. The
synthetic overload controls do not fill that musical-evidence gap.

The preregistered Sparse two-second future window likewise has no protection
or policy difference. No listening artifact or human test is warranted for
identical PCM. No gain, source, recipe, window, threshold or measurement was
adjusted after these results; there was no retry.

Final technical status is `technical_comparison_complete_no_policy_selection`;
`human_verdict=unverified`, `quality_proof=false`. Production remains provisional
A, not an optimum selected by this run. RIOTBOX-1501 remains incomplete and
must not be archived as calibrated. Any further source-backed protection
experiment needs a new version, Decision and explicit bounded source-access
phase; it cannot silently extend this consumed one-session budget.

Independent post-result evidence audit retained zero findings: exact protocol,
report and access-log hashes; all three admitted identities; nine parity flags;
6/6/11 accepted commit records; eighteen zero-write/zero-clip outputs with
input-identical hashes; and the unobserved fixed listening window were checked.
Only the two permitted result JSONs and code/docs were read, never originals or
capture audio. The only post-execution repository changes are this evidence
summary and its numeric-guide route; the accepted algorithm/contract is intact.

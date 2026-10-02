# RIOTBOX-1501 — fixed-window review artifact preparation

Date: 2026-10-02. Base: `91ad8c46ab689feef418b1e79a32f23a48b2e95a`.
Classification: diagnostic artifact preparation, not instrument progress.
RIOTBOX-1501 moved to In Progress / High before creating this branch.

## Authority and purpose

The user approved preparing three A/B/C comparison files from the fixed Sparse
4x interval with one new original access, explicitly without playback. The
[separate protocol](../benchmarks/master_bus_limiter_review_artifact_protocol_v1.md)
must be accepted, pinned, reviewed and committed before access. V1/V2 contracts,
historical results, production DSP and Stage-A boundaries stay immutable.

This removes the precise P023/RIOTBOX-1501 handoff blocker: the technical V2
comparison has policy differences but no exact review WAVs. It adds no musical
mechanism or production policy. The target is a one-bar, two-second composite
of W-30 source transform, TR-909 transient lead and MC-202 punctuation; Source
Monitor and resample tap are silent, bass ownership is unassigned. No isolated
lane, new-hook, development, hardness or bass-pressure claim is made.

## Pre-access design

Reuse the unchanged Rust V2 executor once for Sparse and require all nine
full-render policy hashes, input identities and complete committed preparation
to match its pinned historical report before publishing any comparison WAV.
Crop only `stress_4x` frames [0,96000); all three float32 WAVs use one common
attenuation-only presentation gain. Raw, cropped, presented and file identities
are distinct. No additional original, candidate, composite WAV or playback.

Independent source-free design review (Spec And Evidence Auditor / Product
Pragmatist) identified these concrete reuse constraints before implementation:

- The legacy Dense WAV writer fixes 44100 Hz/PCM16 and adds clipping; it is not
  valid for this 48000-Hz float32 comparison. Only its pure peak/gain helpers
  are reused, with new exact-file readback and format proof.
- Full V2 execution would open three originals. The new owner selects only the
  one authorized identity, and metadata checks need only Sparse's Graph/Session.
- The generic listening skeleton checks presence/size only and its two-bar hook
  prompt does not fit one bar. Keep its structured unverified verdict, supply
  no original-source argument, attach exact hashes, and write the specific
  factual diagnostic prompt without a new verdict system.

These are design constraints, not source-result tuning. No new original or
capture audio was accessed for design or implementation.

Main pre-access review found one P2 in the draft's initial exact-preparation
requirement (Spec And Evidence Auditor / Test Evidence): the existing capture
writer uses a random six-character basename, and promotion action ID 5 embeds it
in `result.summary`. Comparing the already consumed V1/V2 Sparse metadata proves
this is their only preparation difference (`capture-TU2u9F.wav` versus
`capture-Ba93eR.wav`); no original/capture audio was read for this check.
`capture_identity.rs` owns the existing tempfile prefix/suffix. The draft now
permits only that token in the fully anchored ID-5/index-4 promotion summary to
normalize in a comparison copy, retaining raw provenance. Every other field,
capture content identity, commit and timing value remains exact. Mutation tests
must verify this narrow exception before freeze; no production change is needed.

## Implementation and source-free evidence

The new phase owner reuses the existing registered-identity, single-Graph and
bounded child-transport helpers. V1/V2 entrypoints retain their defaults and
behavior. A separate artifact publisher owns only fixed-window float32 WAV
encoding, exact-file analysis and failure evidence; it cannot open a source or
launch playback. The existing listening-review skeleton owns human verdicts.
There is no new product state, renderer, policy API or parallel verdict model.

Publisher self-review closed a P2 failure-evidence issue (Adversarial
Implementation Reviewer / Test Evidence): nonfinite external loudness output
must not insert NaN/Infinity into a strict-JSON failure report. Preserve the
tool's textual summary with null numeric fields, then fail closed; a generated
regression proves serialization and no retry. This does not change normal
metrics or accepted peak gates.

Main review also closes the remote regression gap (Spec And Evidence Auditor /
Test Evidence): Linux Actions explicitly installs ffmpeg and system NumPy and
runs the limiter Python fixtures with the paired system interpreter. The local
Justfile includes both new suites. No source preflight/executor enters CI and
the existing Windows job is unchanged.

The real metadata-only phase preflight passes on the exact parent report and
Sparse Graph/Session without opening audio. Generated ffmpeg/ffprobe roundtrips
prove float32/48000/stereo/96000 frames and bit-exact decoding. Tests cover all
twelve historical PCM identity mutations, narrow filename-token normalization,
source-admission/output budgets, retained child and partial-file failures,
shared gain, fixed crops, safety gates, tool deadlines/byte limits, exclusive
destinations, and report-bound unverified review creation. Prior V1/V2 tests
remain green. Final frozen validation and independent review are recorded below.

## Freeze and independent review

RBX-420 accepts this separate phase. Raw JSON SHA-256:
`d7c66a7885133830c06251980075a3d3aeeb8e47a10b671017b42027e776b41d`.
The execution owner pins these bytes before metadata/source admission.

Independent full implementation review against base `91ad8c46` retains zero
open findings under Spec And Evidence Auditor, Adversarial Implementation
Reviewer and Product Pragmatist. The reviewer independently ran all 47 generated
Python tests successfully. Main's complete-function/self-review also retains
zero open findings after the documented repairs. Rust/Cargo and both consumed
V1/V2 contracts/results have no diff. No source audio was used in review.

Full local `just ci` passes (`/tmp/riotbox-1501-artifacts-ci.log`), including
synthetic audio QA, Rust tests, contract checks and all-feature strict Clippy.
Its Python step ran 46 tests before the final additional pin/status-rejection
regression; the final 47-test suite passed independently afterward. The frozen
fixture/preflight rerun passes before the clean pre-access commit:
`just limiter-calibration-fixtures` records 9 Audio, 8 App, 18 executor and
47 Python tests plus registry fixtures in
`/tmp/riotbox-1501-artifacts-frozen-fixtures.log`. The accepted pinned metadata
preflight also passes without audio access. No protocol/algorithm tuning follows
this freeze; execution results are recorded separately below.

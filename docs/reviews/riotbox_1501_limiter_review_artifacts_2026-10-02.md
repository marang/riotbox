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

## Single execution and exact artifacts

Execution occurred from clean pre-access commit
`a38aef8199c659573cfd6df6ea78f87c5f55ecb0`; its bound native binary SHA-256
`573faddf9e9e418b6f7a0509de10e847112ac410ae7e30c09fd0574244d4dde0`
is unchanged from V2. The one session
`55d17a71-6186-4590-827e-39265c3a39db` ran
2026-10-02 10:54:07.826718–10:54:16.799420 UTC and completed successfully.
The access log records exactly one verified Sparse file delivered to the owner,
no directory discovery and no Holdout audio reads. The unchanged executor used
one source decode, three full mix passes, nine policy outputs and one ordinary
derived capture. Exactly three review WAVs were written, with zero playbacks.
No execution implementation, contract, algorithm or threshold was edited after
access. Subsequent changes are evidence documentation and the source-free test
fixture correction described below; the artifact session is never rerun.

Retained ignored evidence under
`artifacts/development/riotbox-1501/review-artifacts-v1/`:

- `artifact-report.json` SHA-256
  `d043da4727c8f4f3751e0305ee37bf16ed8876f1666d6ddf513c32aff6a24ea0`.
- `development-access.json` SHA-256
  `e243a1867fc1cd9e9574e85eca29d32eeb9034d4ccb7abdaa74dd376b6e52a58`.
- Existing structured `listening-review/review.json` binds the report and all
  three WAV identities; `prompt.md` supplies the neutral one-bar review purpose.
  After adding only the completed independent-assessment reference, review SHA-256
  is `8dfa48ed44872dd57a934a5c31ccdf23fa1be913189063ffeb990eab93f5cfb0`
  (initial generated review: `d95bc0a9c18c97b4b67e5d1841825dfab2bde817d96f3e2478804971b4e27014`).
  Raw report/WAV bytes and every human verdict field remain unchanged.

All three full input hashes and nine output hashes exactly reproduce V2;
repeat/partition/baseline controls pass. Complete preparation also matches after
only the declared comparison-copy filename token normalization. The capture's
encoded content identity remains
`sha256:396744f6e6574204e98d3d59bb921617a9bc3a2a54cfabf60c337de3da9e7bdc`.
Neither an old capture nor another original was hydrated.

| File | Exact WAV SHA-256 |
| --- | --- |
| `A.wav` | `5a346e98ad5c647e87894e3570e27a38d6c960710ca16b7ce148ea0a4ca9d6a1` |
| `B.wav` | `7a4fac4c4c03a71d4741708380faaea46e49fd1f05d997ab1130f49853a4628f` |
| `C.wav` | `5d7fff7b0b90ab05c8660c51236adf65d3004d3d9212f05680e3337054a431c4` |

All files are 768092 bytes, float32/48000 Hz/stereo/96000 frames/2.0 seconds;
ffprobe and decoding operate on each exact hashed file buffer. Readback is
sample-bit-identical to the corresponding presented crop. All files are finite,
active, with zero clipped or exactly-zero samples. Tool versions are ffmpeg and
ffprobe n9.0.2; no player or device is opened.

One shared f32 gain `0.6517586708068848` (bits `3f26d9a8`, −3.718264 dB)
protects the presentation; no per-policy gain or production setting changes.
The maximum estimated raw-crop intersample peak was +2.518264 dBTP, which is
why sample-peak-only protection would have been insufficient for presentation.
The finite estimators remain diagnostic, not hearing/device-safety guarantees.

| Policy | Sample peak | RMS | LUFS | FFT estimate dBTP | ffmpeg peak dBFS |
| --- | ---: | ---: | ---: | ---: | ---: |
| A | 0.641982 | 0.174520 | −15.8 | −1.200000 | −1.3 |
| B | 0.641982 | 0.174543 | −15.8 | −1.201114 | −1.3 |
| C | 0.620800 | 0.174363 | −15.8 | −1.342408 | −1.4 |

Full two-second candidate-minus-A RMS is 0.000150273 for B and 0.000863421
for C; relative delta RMS is 0.000861066 / 0.004947417, with correlation
0.999999638 / 0.999988155. Fixed attack/body/recovery windows and each channel
are measured separately, including spectral-power-fraction deltas. These are
real numeric differences, not proof of audibility, preference or hardness.
The report's status is `artifact_preflight_complete_human_unverified`;
all human/taste/demo fields remain unverified and production remains A.

## Independent artifact-bound pre-listen assessment

Recorded before any playback or listener feedback, against exactly the three
WAV hashes in the table above and report `d043da47…6a24ea0`. An independent
reviewer audited only the authorized result/access/review metadata and retained
zero findings. The exact-file technical evidence comes from the publisher's
hash-bound probe/decode/measurement of those WAV bytes, not from agent listening.
No audio was opened by the reviewer and no earlier human verdict is transferred.

Technical validity passes. Musical statements below are predictions, not human
evidence, and stay out of the factual listener brief:

- Assignment and role: correctly assigned one-bar composite of W-30 source
  transformation, TR-909 transient lead and MC-202 punctuation. Monitor/tap are
  silent; no bass role is assigned. The intended composite role is retained.
- Strongest expected element: TR-909-led drum/transient character follows from
  the declared recipe, not an independently heard judgment.
- Source/hook: exact source/capture identity and preparation survive; audible
  recognizability and hook quality are unverified. One bar cannot prove the
  generic two-bar hook criterion.
- Groove: identical preparation/timing predicts rhythmic equivalence; musical
  feel remains unobserved.
- Clarity: small spectral-fraction changes predict broadly similar coloration;
  there is no measured basis for calling a version clearer.
- Dynamics/impact: C reduces sample peaks more than B, which does not establish
  improved impact. Full-window delta RMS relative to A is B 0.0861%, C 0.4947%;
  fixed attack-window values are 0.2000% and 0.8868%.
- Changed: limiter policy. Unchanged: source, committed actions/timing, window,
  contributors and common presentation gain.
- Likely failure mechanism: perceptual indistinguishability or inconclusive
  preference, particularly A/B; the composite can mask localized differences.

The reviewer judges a neutral diagnostic comparison useful, with **weak/uncertain
expected discrimination**. This is neither a human `weak` verdict nor a policy
winner. Accept “no clear difference” without forcing a preference or regenerating
the files. The next dependent step needs a fresh readiness confirmation after
the factual brief, then bounded playback and verified stop/silence. No playback
is authorized by this preparation phase. Human verdict, demo readiness, musical
quality, hardness and product calibration remain unverified/open.

## Native CI test-fixture correction

PR #1652 run `36998596637` passes Rust tests but exposes a P2 Test Evidence
finding in the new synthetic phase suite (Spec And Evidence Auditor): its
in-memory contract retained the real `/home/markus/Dev/riotbox` workspace pin.
On GitHub's different checkout, the correct production guard rejects it before
the intended synthetic metadata/status assertions. Two tests fail and a generic
historical-rejection assertion can pass for the wrong reason. The real local
single execution and artifact identities are not affected.

Repair only the generated test fixtures: bind their in-memory workspace identity
to the test checkout, explicitly exercise a different temporary workspace, and
require historical-rejection tests to reach both pinned metadata loads and fail
for the intended identity reason. Do not relax the real protocol's workspace
pin, edit accepted execution code or regenerate any source-derived artifact.
This is CI-driven test isolation, not source-result-driven algorithm tuning or
a new access phase. The native rerun is required before merge.

Both main and independent reruns pass all 47 generated tests after this repair
(`/tmp/riotbox-1501-artifacts-portable-tests.log`,
`/tmp/riotbox-1501-ci-fixture-independent.log`). Independent review of the
test/docs-only delta against `871eed0d` retains zero findings. Its additional
generated negative check confirms that the unchanged production preflight still
rejects a wrong workspace before any metadata load. Main self-review and
`git diff --check` also pass. No execution code or artifact is changed or rerun.

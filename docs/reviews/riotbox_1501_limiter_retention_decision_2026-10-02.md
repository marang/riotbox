# RIOTBOX-1501 — retain the measured master-bus baseline

Date: 2026-10-02. Base: `08a87c480d905b83efc59167612cf93ad00be3f1`.
Classification: maintenance / evidence and contract closeout, not audible
instrument progress. Decision: RBX-421. The user explicitly authorized closing
RIOTBOX-1501 through the documented baseline-retention acceptance path.

## Decision and rationale

Keep production A exactly unchanged: f32 knee `0.92`, ceiling `0.985`, the
existing samplewise tanh function and actual-write predicate. No makeup gain,
new limiter, lane gain, action, Session state, runtime branch or recipe change.
This is a bounded engineering retention decision, not a claim that these two
values are a perceptual optimum. It resolves the provisional decision state,
not the limits of the evidence.

The measured baseline leaves clean paths untouched and protects the declared
finite overloads. B raises only the knee to `0.9525`; C lowers only the ceiling
to `0.9525`. Neither alternative offers a demonstrated perceptual advantage in
the exact reviewed comparison. C's smaller sample ceiling is a measured tradeoff,
not a demonstrated true-peak solution or a reason by itself to change the
product. Keeping A avoids an unsubstantiated global sound change; it does not
prove A better than B/C or equate a null preference with a human quality pass.

## Acceptance evidence

| Issue criterion | Evidence and bounded conclusion |
| --- | --- |
| Intended headroom and equality semantics | [Source-free baseline](../benchmarks/master_bus_limiter_baseline_protocol_v1.md), RBX-417: nominal knee −0.7242 dBFS, ceiling −0.1313 dBFS, headroom 0.015 linear amplitude. `abs(x) <= knee` passes; a shaped sample is written/counts only when its difference exceeds `f32::EPSILON`. |
| Quiet, transient, sustained overload and all-owner behavior | [RIOTBOX-1550 result](riotbox_1550_limiter_baseline_protocol_2026-10-01.md): quiet/knee controls unchanged; signed/transient/sustained/partition controls bounded, no boost; named five-owner RuntimeMix pre peak 1.5052301 / 1,364 pre-clips becomes post peak 0.985 / zero sample clips with 1,652 actual writes. This intentional overload fails clean-path QA, as it should. |
| Preregistered alternatives and authorized Development cases | [V1 result](riotbox_1501_limiter_development_comparison_2026-10-02.md), RBX-418, then [V2 result](riotbox_1501_limiter_overload_v2_2026-10-02.md), RBX-419: exact Dense/Tonal/Sparse clean/2x controls are preserved; fixed 4x inputs exercise all three policies on every case, with finite bounded output and zero output sample clips. All repeat/partition/historical identity gates passed. No gain compensation or source-result tuning. |
| Exact artifact and human evidence | [Artifact preparation](riotbox_1501_limiter_review_artifacts_2026-10-02.md), RBX-420, and the hash-bound final review below. Human result: A/B/C sound the same; no preferred candidate. No source-recognition, hook, strongest-element or overall-quality verdict was requested or inferred. |
| Defensible retention decision and owning contracts | RBX-421 and this report explain why the existing baseline is retained, distinguish sample protection from clean-path QA and perceptual quality, and update `audio_core_spec.md` and `audio_numeric_values.md`. No frozen experiment is amended. |

Source budgets and immutable V1/V2/artifact protocols remain consumed and closed.
This decision uses existing code and result/review metadata only: no original,
historical capture, Holdout or commercial audio, source discovery, new
source-backed candidate render or playback. Closeout validation may execute
the normal source-free generated controls; it does not reopen those budgets.

## Final human record

The three reviewed files remain the same Sparse `stress_4x` crop, frames
`[0,96000)` at 48 kHz stereo. Each two-second file was repeated five times for
the explicitly requested ten-second A → B → C replay, not extended with new
musical material. Every player exited normally; monitor checks afterward showed
left/right/overall peak and RMS at digital silence, with no final sink inputs.
The separately requested original-source audition was orientation only and is
not an additional policy candidate or source-recognition verdict.

| Artifact | SHA-256 |
| --- | --- |
| A.wav | `5a346e98ad5c647e87894e3570e27a38d6c960710ca16b7ce148ea0a4ca9d6a1` |
| B.wav | `7a4fac4c4c03a71d4741708380faaea46e49fd1f05d997ab1130f49853a4628f` |
| C.wav | `5d7fff7b0b90ab05c8660c51236adf65d3004d3d9212f05680e3337054a431c4` |
| artifact-report.json | `d043da4727c8f4f3751e0305ee37bf16ed8876f1666d6ddf513c32aff6a24ea0` |
| Final listening-review/review.json | `0fc5eaf3869314cd0f121ca846b3163e7a044ce690bc7b1f484da32a11440df8` |

These are local ignored artifacts under
`artifacts/development/riotbox-1501/review-artifacts-v1/`. The final structured
record uses `riotbox.listening_review.v1`, reviewer Markus,
`human_verdict: inconclusive`, no failure reason and no policy preference.
`inconclusive` is limited to policy selection; unrelated rubric fields mean
unjudged, not negative ratings. It does not require repeating this comparison.

The earlier pumping judgment was explicitly withdrawn by the listener. It is
excluded from current evidence, diagnosis, tuning and blockers; withdrawal
neither proves nor disproves pumping. Preserve history without promoting it:
`review-before-pumping-withdrawal.json` SHA
`192948cb913ad34fb89375e8a34259b851be1794e9056b39b7e9e30fc98c6d65` is superseded.
The unchanged pre-feedback assessment remains bound to
`review-before-human.json` SHA
`8dfa48ed44872dd57a934a5c31ccdf23fa1be913189063ffeb990eab93f5cfb0`; it was not
rewritten to agree with later human feedback. No agent claims to have listened.

## Explicit limits and stopping rule

- Only three familiar Development sources and one selected Sparse listening
  interval were compared. Repetition adds listening time, not source diversity,
  statistical power or proof of perceptual equivalence elsewhere.
- Sample protection is not true-peak protection. The unattenuated reviewed
  A/B/C crops measured approximately +2.518/+2.517/+2.376 dBTP with the specified
  conservative estimator. Their common presentation-only attenuation made the
  review files pass the separate safety gate; it did not change the product.
- No device, hearing-safety, Windows, arbitrary/nonfinite-input, release,
  source-general, hardness, demo-bank or overall musical-quality pass is added.
  Existing clean-product gates still require zero limiter writes and zero
  pre/post clips; the limiter may not hide an overloaded mix or boost weak sound.
- Do not rerun indistinguishable comparisons or keep tuning to manufacture a
  preference. A later concrete requirement or materially different failure may
  justify a new bounded issue/decision and applicable versioned experiment.

The issue's baseline-retention acceptance path is complete once this decision
and owning-contract updates pass review/CI and merge. Closeout removes no audio
quality/release gates and does not claim P023 completion. No new human playback
is needed for this documentation-only change.

## Closeout verification and review

Full local source-free `just ci` passes at this closure
(`/tmp/riotbox-1501-retention-ci.log`): formatting, Rust tests, synthetic
limiter/audio/sidecar/contract checks, tracked JSON and strict all-feature
Clippy. The final structured review validates; metadata-only SHA checks match
the frozen V1/V2/artifact contracts, V2 result, artifact report and final human
record. Production code and all frozen experimental files have no diff.

Independent Spec And Evidence Auditor review covers all five issue criteria,
the five changed/new documents and final/historical review provenance. One P3
was fixed: raw true-peak observations are attributed only to the conservative
FFT estimator; ffmpeg measured the attenuated files. Independent follow-up and
main self-review retain zero open findings; `git diff --check` passes. Native
PR CI and merge remain separate required gates, recorded in the PR/Linear
closeout rather than claimed by this local result.

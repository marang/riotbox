# RIOTBOX-1469 W-30 Stereo Pad Development Stop

Date: 2026-08-26

Work class: `audible_vertical_slice / development_exploration`

Outcome: stereo-preserving W-30 playback is technically feasible on the exact
Development case, but musical usefulness remains human-inconclusive under the
available notebook-speaker monitoring; no provisional keep was awarded and
the temporary Runtime seam was removed

## Frozen Boundary

The source-blind v1 contract compared the unchanged W-30 mono fold against one
candidate that retained the exact existing Mid window and stored symmetric
Side information at the same projection indices. The candidate reconstructed
left and right before applying the unchanged W-30 source-backed character
formula independently per channel. Cursor, retriggers, slices, direction,
rate, gate, crossfade, envelope, grit, bus level, other lanes, and arrangement
remained unchanged. Widening, Haas delay, panning, Mid/Side enhancement,
EQ/gain repair, dry blending, added lanes, and fallback music were forbidden.

Generated source-blind tests proved mono compatibility, anti-phase Side
retention, independent per-channel character state, one shared cursor,
sample-exact 128/257-frame callback partitioning, sample-exact restart,
missing-source silence, and an unchanged existing mono control. The
Development side window remained heap-owned outside the normal product render
snapshot, and normal product callbacks never enabled it.

## Bounded Development Result

One fresh exclusive session opened only registered Development case
`dense_beat03_130` by its exact path and SHA-256. It performed no source
directory discovery and accessed neither Holdout audio nor commercial
references. The complete preflight passed all eleven frozen checks before
human playback.

The objective result establishes the mechanism, not a taste verdict:

- candidate Side/Mid RMS: `-20.8035 dB` versus source `-27.1915 dB`;
- candidate-minus-source Side/Mid delta: `+6.3879 dB`, inside the frozen
  `+/-12 dB` range;
- candidate/control stereo delta RMS: `0.0046083`, above the frozen `0.001`
  minimum;
- candidate/control complete-Mid correlation: `0.9999926`;
- candidate/control complete-Mid RMS delta: `-0.0075 dB`;
- center-attack deltas: `-0.0108` to `-0.0002 dB`;
- callback partitions and restart: sample-exact;
- clipped integer samples and limiter interventions: zero;
- source/control/candidate true peaks: `-1.2003`, `-10.5673`, and
  `-10.5647 dBTP` respectively.

The candidate therefore retains real stereo information while leaving the
center, timing, restart, and existing mono path effectively unchanged. This is
valid technical feasibility evidence.

## Human Usefulness Check

After exact-artifact preflight, the listener heard source, mono control, and
stereo candidate in the frozen order with one-second pauses. Playback ended at
the announced boundary and silence was verified. The listener understood the
mono/stereo comparison but reported that the ThinkPad P52
built-in speakers did not permit a reliable audible distinction.

This is recorded as `inconclusive_due_to_monitoring`, not rejection and not a
musical pass. Automated spatial metrics cannot award the required musician-
facing usefulness verdict. The v1 one-review budget is exhausted, so no repeat,
second source, threshold change, or result-driven candidate revision is
admitted under this contract.

## Consequence

RIOTBOX-1469 stops without `freeze_for_qualification`. The source-blind
implementation is preserved in commit `9d9e5b0e`; the final tree removes its
Development-only renderer, Runtime activation seam, tests, and source runner.
The frozen contract and this evidence remain to prevent rediscovery and to
support a materially justified future slice with suitable stereo monitoring.

Dense remains non-demo-ready and the sole missing positive demo family. No
product, source-general, quality, Holdout, demo, release, or P023-completion
claim follows.

## Evidence Identities

- frozen contract: `0a7f6c84254910d567685186661a991b76ff5b21123669eb0c8b6a7b2437abf5`
- registered source corpus: `67b5b8b2882575cf70fa61aacf25ae282c17714fe51ffcb13f905458e025d552`
- exclusive access log: `e34eaa74fb7d89f8efd2cf581de01d760a6bf3822d951320c590cd69f9d4a4e1`
- exploration result: `3d9275cbbcfabe0618d446fec97586e97fcddc6b5c434172f406bf3ea97bf32d`
- Runtime report: `1d761aedf7f19386a98a149611a42bb15b542552aa84a62317d41f2a8d20a7df`
- technical preflight: `e6b7eae36ed49b74df2f2563329bcc113f00dc8326f5b9aec5433c41ddeb5190`
- source presentation WAV: `0a807b38b198447a607013239f727b1bc61727ebfc4b41203b80c754d2b27961`
- mono-control WAV: `c4838082902ef6784d0e99b07a25420d2f30c18067c64c2239bf55073bef0f81`
- stereo-candidate WAV: `5fbb1ad08c8a36bf6f85d6f3eb82113a01a466438e37e922a8d7bcf8bc7b54d5`

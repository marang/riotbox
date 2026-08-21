# RIOTBOX-1449 MC-202 Answer Development Rejection

Date: 2026-08-21

Work class: `development_exploration`

Outcome: stopped fail-closed; no product behavior retained

## Scope

RIOTBOX-1449 tested a musical owner materially different from the rejected
RIOTBOX-1417 instigator mechanisms: a source-derived MC-202 low-end answer that
would punctuate the existing W-30/TR-909 groove without cutting the supporting
bed. The bounded exploration used only exact registered Development identities
through fresh exclusive access logs. No source directory was discovered, and
no Holdout or commercial-reference audio was opened.

An initial current-behavior audit on
`oga_cinameng_can_be_so_beautiful` found that explicit `Pressure` rejected
`sub_pressure_shove` for insufficient phrase-level low-band pressure but then
selected `fill_pickup_instigator`. That role mismatch was not played. The
audible exploration then used `oga_frosty_ham_osdrums`, SHA-256
`7e412dd16e701d1f2b3a8c0d66fbb24ec0164691e6761a93eca8b4bb60d32bb2`.
Its exact explicit `Answer` path selected the source-derived
`sparse_offbeat_answer` family.

## Technical Result

The controlled exact RuntimeMix diagnostic passed with no clips or limiter
intervention. The final review artifact was
`artifacts/development/riotbox-1449/review-2026-08-21-a/03_source_then_A_then_B.wav`,
SHA-256
`4ee0d3bb861113a1db8204d32e17150fcd471b38bde471844a07e5f8c0bb9c89`.
Its 23.841-second assignment was:

1. exact registered source, resampled to 48 kHz and repeated twice;
2. one second of digital silence;
3. A: 8.533 seconds of the exact W-30 plus TR-909 stems with MC-202 omitted;
4. one second of digital silence;
5. B: the same W-30/TR-909 state plus the source-derived MC-202 answer;
6. 0.5 seconds of digital end silence.

A and B both measured `-21.5 LUFS`. A/B waveform correlation was `0.98857`;
the exact delta matched the rendered MC-202 stem with correlation above
`0.999999` and at most one PCM16 LSB error. The delta was confined almost
entirely to sixteenth-note phases 11 and 12 and changed only about 2.4% of
frames. During the intended answer slots it sat approximately 17 dB below the
existing mix in the 90-160 Hz bass band, 26 dB below it in the 160-800 Hz
low-mid band, and 36 dB below it in the 35-90 Hz sub band. Phase 11 was
effectively absent on alternating bars. This explains why a technically valid
isolated lane did not establish perceptual ownership in the complete mix.

## Human Result

After exact-artifact preflight and fresh readiness, one bounded playback was
completed and the host returned to silence. The musician judged the general
source transformation useful but reported no audible distinction between A
and B.

Durable interpretation:

- `human_verdict: reject` for the additional MC-202 answer;
- the positive comment belongs to the shared W-30/TR-909 transformation in
  both sides, not to the tested MC-202 delta;
- the answer is too temporally sparse and too strongly masked in its intended
  low-frequency owner bands to be worth triggering;
- no provisional keep, frozen mechanism, source-diversity matrix, formal
  product review, Holdout access, or quality claim is authorized.

## Stop And Follow-up

The temporary Development role selector was removed. The exploration does not
continue through scalar gain, EQ, or threshold tuning, and the unchanged
artifact is not replayed merely to search for a preference.

RIOTBOX-1450 separately owns the product-contract defect exposed by the
unplayed pressure audit: an explicit MC-202 performer intent must fail closed
instead of committing a different audible candidate family. Fixing that
contract does not promote or repair the rejected answer. Any later audible
MC-202 attempt requires a new Linear-first slice and a materially different
musical topology with sustained perceptual ownership in the full mix.

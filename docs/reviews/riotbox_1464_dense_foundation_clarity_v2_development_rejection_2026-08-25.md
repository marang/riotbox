# RIOTBOX-1464 Dense Foundation Clarity v2 Development Rejection

Date: 2026-08-25

Work class: `audible_vertical_slice / development_exploration`

Outcome: frozen v2 rejected before render or listening because no registered
source-native answer cell satisfied its clarity and contrast contract

## Frozen Boundary

RIOTBOX-1464 did not tune rejected v1. Before fresh source access, RBX-325
froze `riotbox.dense_break_foundation_chop.v2` as a materially different
source-native topology. Beats zero through five remain sample-exact. Beats six
through seven may be replaced only by one coherent two-beat cell from beats
zero through one, two through three, or four through five. The selected cell
would retain its internal timing and receive a 2 ms linear crossfade only at
answer entry and phrase-end return.

Eligibility requires bounded whole-cell and corresponding half-beat low-band
and broadband-level differences together with material waveform contrast. The
selector has no EQ or filter repair, gain compensation, source-specific
exception, additive layer, support instrument, pitch/time change, bus effect,
or fallback music.

## Source-Blind Falsification

Deterministic generated stereo fixtures pass before source access. They prove
that the selector:

- chooses an energy-matched but materially different coherent cell;
- preserves the six-beat anchor sample-exactly;
- copies the chosen cell exactly away from its two boundary fades;
- rejects candidates whose whole-cell energy matches while a local half-beat
  collapses;
- rejects near-identical candidates; and
- resolves exact ties deterministically.

The implementation uses the supplied sample rate and per-channel low-band
filtering before stereo RMS aggregation, avoiding a false weight result from
hardcoded rate assumptions or stereo cancellation.

## Bounded Development Result

After the contract and Decision were hash-pinned, one fresh exclusive
`DevelopmentSourceAccessSession` opened only registered case
`dense_beat03_130` by its exact path and expected SHA-256. The access layer
performed no directory discovery and opened no Holdout audio. The verified
bytes were delivered in process to the qualification owner; no direct source
path read occurred in the renderer.

All three permitted coherent two-beat cells were ineligible under the frozen
combination of whole-cell clarity, corresponding half-beat clarity, and
material-contrast gates. The owner therefore raised
`no clarity-preserving two-beat answer cell`; the access session recorded an
audited abort and did not create the output directory, source loop, candidate,
or report.

This is the intended fail-closed result. The session did not relax a threshold,
substitute a source, repair the audio after selection, or start another access
log.

## Human Listening

No listening review occurred. There is no technically eligible candidate, so
playing the unchanged source alone or fabricating a comparison would add no
musical evidence and would violate the frozen stopping rule. No human verdict
or perceptual claim is recorded for v2.

## Consequence

Frozen v2 is rejected for `no_eligible_answer_cell`. It grants no product
behavior, source-general success, quality, hardness, Holdout, demo, release, or
P023-completion claim. Its contract, focused synthetic fixtures, and concise
negative record remain to prevent scalar threshold loosening or accidental
reuse of the same three-cell selector.

Any successor requires a new Linear-first slice, a materially different causal
topology, and a new version and Decision before fresh source access. It may not
reinterpret this result as permission to tune v2 or make a Beat03 exception.
Dense remains non-demo-ready.

## Evidence Identities

- frozen v2 contract: `b0338bd87e9c536f80f725e9a1f891f4df67b43a370de10564f2fa60884b0020`
- source-blind renderer/fixture implementation:
  `b1d4ea9139da38eab83f5594177907df34545a21b4514cc59f988043b9582ccb`
- failed-closed Development access log:
  `8d3f827039f70aede5d46b57ee97cf0c91b80a4d3df1c33b62056e7057ab2e26`

# RIOTBOX-1481 Source-Matched Stem Multitrack Utility Review — 2026-08-27

## Scope

- Phase: P016 / Pro Workflow / Export
- Issue: RIOTBOX-1481
- Frozen qualification contract:
  `riotbox.source_matched_stem_multitrack_utility_qualification.v1`, raw
  SHA-256 `a5526812b060a41cd942676be8c615aeb8f44c72d7a4f64870c9b46137c5d4a7`
- Input boundary: only the exact RIOTBOX-1480 Dense full mix and byte-identical
  Session Drums, Music, and Bass stems
- Claim boundary: whether those unchanged post-bus contribution stems remain
  musically useful when exposed through one concrete mute, balance, and
  arrangement task
- Explicit non-goals: new source access, a V2 producer/source-renderer rerun or
  stem tuning, a second candidate, TUI/Ghost controls, DAW import, live
  recording, Holdout evidence, commercial-reference evidence, source
  generality, or release readiness

## Frozen Presentation

A repeats the exact qualified four-bar full mix twice. B uses only the exact
qualified contribution stems over the same eight-bar duration: Music alone for
two bars, Music and Bass for two bars, then all three roles for four bars. The
fixed role gains are Music `-3 dB`, Bass `+12 dB`, and Drums `+6 dB`. One
whole-artifact scalar matches A's RMS unless the frozen PCM16-safe peak ceiling
would bind first. No limiter, dither, replacement audio, synthetic support, or
additional lane is present.

The exact A and B SHA-256 values are
`11eb956983ea722c9c4434c8b83e766846c470439bd505e5d9e661349c8e467d` and
`c677577b5e9aa00205e0cdcebacd042fd58285cb32c667d75ea273e031fe5b63`.

## Technical Qualification

The technical report has SHA-256
`bae4a09b52738d13e43ceeb89b660eceeab2956ed5800f2001d7e0416038e3a5` and
records `pass`. Every input hash, manifest/proof binding, regular-file check,
and 44.1 kHz stereo PCM16 grid passed. No original source, source directory,
new registered Development source, Holdout, commercial reference, or producer
rerun was accessed. The frozen access field that forbids a producer or renderer
rerun refers to the V2 product producer/source renderer; the separate
qualification presentation render is the operation explicitly declared by the
same contract.

A and B each contain 649898 frames or 14.736916 seconds. Their normalized RMS
values are `0.0957171455` and `0.0957171470`, so loudness does not decide the
comparison. Their peaks are `0.725647` and `0.758118`; neither clips. Two
independent B renders are byte-identical. The Music-only, Music-plus-Bass, and
full-role sections differ from A with normalized delta RMS values `0.01795`,
`0.03342`, and `0.03866`, confirming that every declared operation reaches the
output.

## Human Review

The structured review JSON has SHA-256
`8f42e9bb49e4261d0cb61d10031f5b85de4e638c1b250a9de3c604e64c2f70f4`
and records `human_verdict: reject`.

As a musician-facing loop or stem result, B is a clear rejection. The listener
would not loop it because its musical result is not good enough and does not
approach Riotbox's aggressive sample-based rave-punk direction. The passive
presentation also does not establish what musician action or workflow this
manipulation represents inside Riotbox. No more detailed taste diagnosis is
required: the exact candidate fails both the musical-loop bar and the
in-product utility question.

No human musical verdict applies to possible internal intermediate use. The
listener was not shown a downstream transformation or final output that uses
these contributions, and therefore cannot assess whether they are useful
inside a later mechanism.

## Product Interpretation

The current package remains strong reconstruction and lineage evidence. Its
three files are symmetric Shapley contributions allocated after the shared
nonlinear mix bus, not ordinary isolated pre-bus instrument lanes. Exact
summation therefore reconstructs the approved full mix, but independently
muting or reweighting a contribution does not rerun that shared bus and need
not preserve a musically self-contained role.

It is a technically grounded inference that the rejected result exposes this
context dependence: the Music contribution dominates, while boosted Drums and
Bass contain attribution shaped by the complete nonlinear mixture. This one
Dense review does not prove that every possible contribution edit must fail,
but it is sufficient to reject promotion of the current V2 contribution set as
musician-remixable stems.

## Consequence

Keep the V2 contribution handoff bounded to exact reconstruction, lineage, and
internal proof. This review does not reject that internal role; it rejects the
current files and arrangement as musician-facing or loop-ready material. Do
not add musician UI around them, retune this candidate, replay it, or open
another source. The one-off qualification renderer is not retained as product
infrastructure.

Before another musician-facing review, a new Linear-first slice must freeze the
product-role boundary. If the current contributions feed a later musical
mechanism, review only that mechanism's final audible result after explaining
their causal role. A future musician export instead requires a versioned
musician-semantic stem contract inside the existing export Action, Session,
receipt, and observer spine. That contract must distinguish editable musical
roles from reconstructable post-bus attribution without creating a second
export or persistence system. Its output must qualify musically before any
TUI/Ghost or broader DAW surface is built.

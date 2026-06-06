# Dense-Break Performance Pack v1

`riotbox.dense_break_performance_pack.v1` is the first controlled Riotbox
sound-quality Golden Path.

It renders an 8-bar source-backed rave-punk break performance from a local
dense-break source, currently `data/test_audio/examples/Beat03_130BPM(Full).wav`.
The source file is a short loop; the pack treats it as source material and
arranges a longer performance from generated Riotbox stems.

Target structure:

- Bars 1-2: source character plus W-30-style chop motif.
- Bars 3-4: W-30 source chop becomes the main hook.
- Bars 5-6: TR-909 and MC-202 pressure lift.
- Bar 7: dropout followed by source-chop stutter.
- Bar 8: restore hit where break transient and bass pressure land together.

Generated artifacts:

- `00_source_window.wav`
- `01_chop_hook.wav`
- `02_pressure_lift.wav`
- `03_dropout_stutter.wav`
- `04_restore_hit.wav`
- `05_full_performance.wav`
- `performance-report.json`
- `agent-review.json`
- `agent-review.md`
- `visuals/*.waveform.png`
- `visuals/*.spectrogram.png`
- `README.md`

The JSON reports expose the shared evidence boundary:

```json
{
  "evidence_role": "diagnostic",
  "source_backed": true,
  "source_timing_backed": true,
  "scripted_generation": true,
  "quality_proof": false,
  "human_verdict": "unverified"
}
```

The report emits `agent_verdict: agent_promising` only when the pack avoids the
known bad-output modes for this Golden Path: weak W-30 hook presence, missing
bass-pressure lift, weak dropout/stutter contrast, weak restore transient,
near-static bars, source-copy collapse, or buried bass pressure.

The current pro-pressure render also gates against the older too-polite pass
shape. The full performance must be assertive relative to the source window, the
opening hook must carry real break transient, the pressure section must be
louder than the hook, and the restore hit must be bigger than the pressure
section. This keeps the Golden Path aimed at a room-moving source-backed
instrument instead of a technically valid but underpowered fixture.

`agent_promising` is not a final musical pass, and this pack is not technical or
musical quality proof for the product. The current pack now records a bounded
`source_policy` decision object. A nested `pressure_lift_policy` classifies the
source as dense-break, tonal-hook, sparse-bass-pressure, pad/noise, bad-timing,
or thin/uncertain from the source profile and source-timing confidence, then
chooses lift shape, source/hook bleed, TR-909 / MC-202 / bass drive, bar-4 /
bar-5 intensity, and bass frequencies. The same policy also chooses stutter
density, grain offset, restore snap gain, and bad-timing confirmation cue
strength. A sibling `arrangement_policy` chooses the first six hook/chop/pressure
roles from source/W-30 bar candidates while preserving the bounded destructive
dropout/restore tail. Bad-timing sources take a manual-confirm cautious cut that
avoids confident bar-locked claims until timing is confirmed. The allowed role
grammar is still scripted and bounded to `hook`, `chop`, `pressure`, `dropout`,
and `restore`, so this remains smoke, regression, and diagnostic evidence. It
proves the harness can render source-backed stems, apply visible source-aware
pressure-lift/stutter/restore, arrangement, and mix-treatment decisions, and
reject known weak-output shapes. The pressure policy also has a bounded
`pad_noise` family for thin low-band, high-noise material: that path gates
pad/noise as texture and must not promote it to dense-break proof. The pack also
writes `06_rebuild_only_performance.wav`, a
source-layer-off diagnostic render:
raw source bleed is removed while source-derived chops, transient snaps, bass
pressure, drums, and restore behavior remain active. The report gates that
rebuild-only output for non-silence, useful RMS against the full/source mix,
low source correlation, and a distinct source-on/source-off waveform. It does
not yet prove fully source-aware production quality. The report must keep
`human_verdict: unverified` until a structured listening review or the future
P021 calibrated audio judge supplies stronger verdict evidence.

For dense-break and tonal-hook sources, the W-30 hook/chop path now uses a
bounded `hook_chop_policy` instead of only taking the strongest grain from the
first bar. The policy scans multiple source/W-30 candidates, records separate
hook and chop offsets, and reports `hook_chop_selection_source_derived`,
`hook_chop_static_distance_frames`, and `hook_chop_offset_distance_frames`.
Dense/tonal reports must fail if hook/chop selection is not source-derived,
does not scan enough candidates, collapses back to the old static first-bar
choice, or selects hook/chop offsets without enough contrast. This is still a
diagnostic selection proof, not a human musical pass.

For dense-break, tonal-hook, and sparse-bass-pressure sources, the first six
arrangement bars now expose source-derived role-order proof. The
`arrangement_policy` compares the selected hook/chop/pressure order against the
old source-family scripted order, reports candidate count and scripted-role
distance, and must fail when eligible sources fall back to the fixed family
recipe. The role vocabulary plus dropout/restore tail remain bounded, so this
is stronger arrangement evidence, not product-quality proof.

For dense-break, tonal-hook, and sparse-bass-pressure sources, the mix bus now
exposes bounded source-derived treatment through `mix_treatment_policy`. The
policy scans source/W-30 bar candidates, derives hook/chop/pressure/restore bus
drive, slam, W-30 gain, break-snap gain, pressure peak, and restore bass gain
from source energy, low-band, high-band, transient, and W-30 response, then
reports candidate count, distance from the old fixed mix recipe, source-energy
span, and output contrast. Eligible reports must fail if mix treatment is not
source-derived, does not scan enough candidates, collapses back to the old fixed
recipe, or loses pressure/restore contrast. This makes the rendered output react
more to the source material, but it is still bounded diagnostic mix evidence,
not product-quality proof.

For dense-break and tonal-hook sources, the destructive dropout/stutter/restore
path now also exposes a bounded `destructive_gesture_policy`. The policy scans
source/W-30 candidates for separate stutter and restore cues instead of using
only the old fixed stutter beat offset and `source[:hit]` restore transient.
Dense/tonal reports must fail if destructive gesture selection is not
source-derived, does not scan enough candidates, collapses back to the old fixed
choice, or selects stutter/restore offsets without enough contrast. This proves
stronger source-derived destructive material, not a human musical pass.

For sparse-bass-pressure sources, the bass-pressure render now derives its
pressure/restore frequencies from the source low-band envelope and timing
centroid instead of using only the fixed policy contour. The report exposes
`bass_movement_source_derived`,
`sparse_bass_movement_static_distance_hz`, and
`sparse_bass_movement_frequency_span_hz`; sparse reports must fail if movement
collapses back to the old fixed contour or does not vary enough. This is still a
bounded diagnostic proof, not a claim that the bass-pressure line is musically
approved.

`dense-break-performance-pack-smoke` validates the generated report and mutates
it as negative rebuild-only fixtures. The smoke must fail if rebuild-only RMS is
zeroed, if rebuild-only/source correlation is pushed into source-masked
territory, or if source-on and rebuild-only correlation collapses to identical.
These negative checks protect the diagnostic gate from quietly accepting
silence, raw-source masking, or a no-op source-layer toggle.

Run:

```bash
just dense-break-performance-pack
just dense-break-performance-pack-smoke
just agent-musical-review-pack-smoke
just pro-pressure-source-matrix-smoke
just professional-source-wav-pack-smoke
just edge-source-professional-diagnostics-smoke
just non-dense-professional-proof-pack-smoke
just professional-output-listening-pack-smoke
just destructive-variation-professional-smoke
just rendered-weak-professional-output-fixtures
just professional-output-suite-smoke
```

The source-matrix smoke renders the same pro-pressure contract across multiple
local examples (`Beat03`, `Beat08`, `Beat20`, and `DH_BeatC`) and writes
`source-matrix-report.json` with per-source proof plus an `arrangement_summary`.
The matrix now requires source-derived role-order proof for eligible cases, at
least two distinct role-order signatures across source families, routes weak
arrangement results to concrete fix categories, and checks rebuild-only metrics
for every rendered case. This prevents Beat03 from being the only passing
example while other local break sources quietly regress, collapse back to one
fixed arrangement, or pass only because the raw source layer masks weak
generated/rebuilt material.

`professional-source-wav-pack-smoke` renders family-aware audible WAV packs for
tonal and sparse local sources (`DH_RushArp` and `DH_BeatC_KickSnr`). Tonal
material must carry a `tonal_hook` pressure-lift policy and is allowed to keep
the hook more readable, while sparse material must carry a
`sparse_bass_pressure` pressure-lift policy, use a sparse-specific arrangement
signature, prove pressure stronger than the hook, and include
`06_rebuild_only_performance.wav` so the pack can compare source-layer-on vs
source-layer-off output. Both cases still write `human_verdict: unverified`.
Because these cases currently reuse the bounded performance generator and
bounded role vocabulary / destructive tail, they are cross-source diagnostics
rather than source-family quality proof.

`non-dense-professional-proof-pack-smoke` adds a bounded bridge between the
rendered tonal/sparse Professional Source WAVs and the stricter
`tonal_hook_professional` / `sparse_bass_pressure_professional` validator
families. It writes WAV hashes, source-family validator hashes, and review
prompts for tonal-hook and sparse-bass-pressure cases, plus the source-family
fixture manifest hashes that fed the validators. It is still diagnostic: the
current rendered WAV path is scripted, so the pack must keep
`quality_proof: false` until source-family production decisions are owned by the
engine rather than by a rehearsed render recipe.

`edge-source-professional-diagnostics-smoke` renders bounded pad/noise and
bad-timing diagnostic cases from the local source corpus. The pad/noise case
uses `DH_Fadapad_120_A.wav` and must stay `degraded` / `unavailable` in
source-timing while taking the explicit `pad_noise` pressure policy path. That
path treats the source as a gated texture candidate rather than pretending it is
a breakbeat. The bad-timing case uses `Beat20_128BPM(Full).wav` and must stay
`candidate_ambiguous` / `manual_confirm_only` so bar-locked decisions route to a
timing/UI fix instead of pretending the grid is proven. That case must take the
explicit `bad_timing` pressure policy, expose a
`manual_confirm_cautious_arrangement` timing policy, reject bar-locked policy,
and render an audible confirmation cue. The smoke mutates the generated report
to prove silence, identical output, fallback collapse, missing source-family
metadata, missing bad-timing policy, and bad-timing bar-lock regression are
rejected. This is diagnostic evidence only: passing means Riotbox detected and
routed the weak/risky output, not that the output is demo-ready.

`professional-output-listening-pack-smoke` prepares structured human-review
packs for dense, tonal, and sparse professional-output WAVs. It records candidate
WAV hashes, source-report hashes, review prompts, and explicit
`human_verdict: unverified` placeholders so taste review can happen without
claiming a musical pass from metrics alone.

`destructive-variation-professional-smoke` validates the dense-break dropout /
stutter / restore behavior as a first-class professional-output contract. It
rejects flat cuts, weak stutters, static bars, source-copy collapse, and restores
that do not recover with enough level after the destructive gesture.

`professional-output-suite-smoke` renders the current professional-output child
reports together and writes `riotbox.professional_output_suite.v1`. The suite is
the central deterministic status surface for dense-break, pro-pressure source
matrix, tonal/sparse WAV packs, structured listening packs, and destructive
variation proof. It also includes the non-dense proof pack and edge-source
diagnostics so tonal-hook, sparse-bass-pressure, pad/noise, and bad-timing
source-family risks are visible in the aggregate status surface. It checks child
report hashes and listening-pack file identity, but still keeps
`human_verdict: unverified`.

`rendered-weak-professional-output-fixtures` adds a rendered negative example:
real WAV files for a dense/destructive flat-stutter case whose metrics prove the
stutter is too flat and the restore is too weak. The destructive variation gate
must reject that case, so a hand-written JSON-only negative is no longer the
only weak-output proof.

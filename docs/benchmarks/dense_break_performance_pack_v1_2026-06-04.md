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
musical quality proof for the product. The current pack still contains scripted
arrangement behavior, including fixed section roles such as `pressure_lift`.
Treat it as smoke, regression, and diagnostic evidence: it proves the harness
can render source-backed stems and reject known weak-output shapes, not that the
generator has made a source-aware production decision. The report must keep
`human_verdict: unverified` until a structured listening review or the future
P021 calibrated audio judge supplies stronger verdict evidence.

Run:

```bash
just dense-break-performance-pack
just dense-break-performance-pack-smoke
just agent-musical-review-pack-smoke
just pro-pressure-source-matrix-smoke
just professional-source-wav-pack-smoke
just non-dense-professional-proof-pack-smoke
just professional-output-listening-pack-smoke
just destructive-variation-professional-smoke
just rendered-weak-professional-output-fixtures
just professional-output-suite-smoke
```

The source-matrix smoke renders the same pro-pressure contract across multiple
local examples (`Beat03`, `Beat08`, `Beat20`, and `DH_BeatC`) and writes
`source-matrix-report.json` with per-source proof. This prevents Beat03 from
being the only passing example while other local break sources quietly regress.

`professional-source-wav-pack-smoke` renders family-aware audible WAV packs for
tonal and sparse local sources (`DH_RushArp` and `DH_BeatC_KickSnr`). Tonal
material is allowed to keep the hook stronger than the pressure section, while
sparse material must prove pressure stronger than the hook. Both cases still
write `human_verdict: unverified`. Because these cases currently reuse the
dense-break performance generator, they are cross-source diagnostics rather than
source-family quality proof.

`non-dense-professional-proof-pack-smoke` adds a bounded bridge between the
rendered tonal/sparse Professional Source WAVs and the stricter
`tonal_hook_professional` / `sparse_bass_pressure_professional` validator
families. It writes WAV hashes, source-family validator hashes, and review
prompts for tonal-hook and sparse-bass-pressure cases, plus the source-family
fixture manifest hashes that fed the validators. It is still diagnostic: the
current rendered WAV path is scripted, so the pack must keep
`quality_proof: false` until source-family production decisions are owned by the
engine rather than by a rehearsed render recipe.

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
variation proof. It also includes the non-dense proof pack so tonal-hook and
sparse-bass-pressure source-family validators are visible in the aggregate
status surface. It checks child report hashes and listening-pack file identity,
but still keeps `human_verdict: unverified`.

`rendered-weak-professional-output-fixtures` adds a rendered negative example:
real WAV files for a dense/destructive flat-stutter case whose metrics prove the
stutter is too flat and the restore is too weak. The destructive variation gate
must reject that case, so a hand-written JSON-only negative is no longer the
only weak-output proof.

# RIOTBOX-1398 Human Listening Review

Date: 2026-07-11
Reviewer: Markus
Verdict source: real user-session playback through the configured PipeWire
output, not fixture or agent approval

## Evidence Boundary

The reviewed WAVs come from the existing
local-riotbox-1363-professional-output-suite pack. They are source-backed,
source-timing-backed, scripted offline listening scaffolds. They are not the
exact live riotbox-app mixer path and remain quality_proof: false.

The checked-in label corpus stores artifact hashes rather than local WAV files:
docs/benchmarks/p023_human_listening_label_corpus_2026-07-11.json.

## Dense Break

- Source: Beat03_130BPM(Full).wav
- Human verdict: weak
- Source character: transformed but still recognizable
- Hook: weak; no element convincingly leads
- Bass pressure: missing in practice
- Main failure: MC-202 bass is practically inaudible, while hook and drum
  support remain too weak to improve the source

## Tonal Hook

- Source: DH_RushArp_120_A.wav
- Human verdict: weak
- Source character: transformed but still recognizable
- Hook: clear and good; the melodic transformation sounds advanced
- Bass pressure: intended but not audible
- Main direction: preserve the successful transformed arpeggio hook while
  adding tight, clearly audible bass support

## Sparse Bass Pressure

- Source: DH_BeatC_KickSnr_120-01.wav
- Human verdict: weak
- Source character: transformed and still somewhat recognizable
- Hook: clear, recognizable drum hook
- Bass pressure: audible, but droning and still too weak
- Main direction: retain the drum hook while making bass tighter, more physical,
  and less droning

## Product Decision

No reviewed candidate earns a human pass. Riotbox must not claim demo-ready or
musical-alpha quality from this pack.

The strongest positive evidence is the tonal W-30-style melodic transformation.
The repeated highest-value failure across all three source families is MC-202
bass translation: missing in dense and tonal material, audible but droning and
weak in sparse material.

Route one sound-policy priority into RIOTBOX-1400: preserve the successful
source-backed hook transformations while making MC-202 bass pressure clearly
audible, tight, physical, and role-aware. The live implementation order remains
RIOTBOX-1330 then RIOTBOX-1333 and RIOTBOX-1335; the verdict informs the shared
performance policy rather than opening another report or validator.

## Review UX Note

The isolated TR-909 and MC-202 stems are diagnostic lane solos, not musician
loops or presets. Their purpose was not obvious without reading pack metadata.
Future listening instructions should state that solo stems diagnose timing,
glitches, and lane contribution; musical value must be judged by adding them to
the W-30/source-led mix.

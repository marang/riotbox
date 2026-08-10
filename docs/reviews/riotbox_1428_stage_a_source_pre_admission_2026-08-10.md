# RIOTBOX-1428 Stage-A Source Pre-Admission Review

Date: 2026-08-10
Reviewer role: `project_musician`
Work class: internal prerequisite of the existing RIOTBOX-1428
`audible_vertical_slice`
Quality proof: `false`

## Purpose

This record closes only the narrow raw-source suitability lane preregistered in
`docs/engineering/percussive_force_and_beat_impact.md`. It is an internal
prerequisite of the already-owned audible slice, not a chained contract
enabler. The directly enabled outcome is the exact audible F1--F3
source-backed render plus bounded human directional comparison. This record
does not qualify an event, select F1--F3, render a force candidate, prove a
harder hit, or improve the live instrument.

## Holdout-safe preflight

The two declared candidate identities were compared with the active
`holdout_a` plus `holdout_b` metadata union from
`docs/benchmarks/source_holdout_rotation_v1.json`. The comparison used exactly
`case_id`, `source_path`, and `sha256`; neither candidate overlaps the nine
active holdout entries on any field.

During this original bounded preflight, the canonical predecessor manifest SHA-256 was
`dd017080f311dcb2a8eda2fac63d8da372a356f0fc2cc33d5c97d3fd2ea34cfc`.
No active holdout audio was opened, read, hashed, rendered, classified, or
played. No source directory was globbed or discovered in that session. This
scoped statement remains true; the separate post-review operational audit
below does not retroactively change it.

## Raw-source suitability verdicts

### William Hector — Horde War Drums loop

- Case: `oga_william_hector_horde_war_drums`
- Declared source SHA-256:
  `a4d95514029dd928e5637c3b9edd659b8eaf14fa78d8afb2ab7ec4da064e4417`
- Source suitability: **yes**
- Source-suitability verdict owner: `human_review`
- At least two usable percussive hits: **yes**
- Technical taxonomy: `dense_break`
- Taxonomy verdict owner: `technical_review`
- Original-rate format retained: stereo 44.1 kHz PCM24 WAV
- License/provider: CC0-1.0, OpenGameArt

The verdict means the raw source can participate in the positive development
corpus. It does not assert that any specific onset is already a frozen or
force-qualified event.

### frosty ham — Drumming / osdrums

- Case: `oga_frosty_ham_osdrums`
- Declared decoded-source SHA-256:
  `7e412dd16e701d1f2b3a8c0d66fbb24ec0164691e6761a93eca8b4bb60d32bb2`
- Source suitability: **yes**; the raw material was described as good drums
- Source-suitability verdict owner: `human_review`
- At least two distinct raw attacks visible and audible: **yes**; this is not
  mechanism-blind event qualification
- Technical taxonomy: `electronic_drums`
- Taxonomy verdict owner: `technical_review`
- Taxonomy evidence: the creator identifies the source as made in Online
  Sequencer; the source is programmed/synthetic and drum-led
- Original-rate format retained: stereo 44.1 kHz PCM16 WAV decoded from the
  registered OGG
- License/provider: CC0-1.0, OpenGameArt

The six-repeat, 12.822993-second presentation was audition-only. Its SHA-256 is
`735754338e50b19dbdbaf8a58e8391f97ab6de56a6678eefb5d67d14cd910a9f`.
It is not a source identity, event catalog, candidate render, fixture, or
quality artifact and remains `quality_proof: false`.

## Admission decision

Both cases may be admitted as development-only entries in
`riotbox.source_holdout_rotation.v2`. All v1 entries remain unchanged as JSON
values, the active holdout ID/path/SHA strings remain unchanged, and inherited
sources retain the implicit v1 48 kHz PCM16 file contract. The two admitted
cases carry explicit original-rate `source_format` records and derivation
provenance.

The next allowed action is the versioned registry/matrix freeze followed by
mechanism-blind source and event qualification. Candidate DSP, event rendering,
and human force comparison remain forbidden until that freeze is complete.

## Post-review operational note — 2026-08-10

A later operational audit used the following bounded shell shape:

```bash
ls -ld data data/test_audio data/test_audio/external \
  data/test_audio/external/RIOTBOX-1423 \
  data/test_audio/external/RIOTBOX-1423/wav && \
  find data/test_audio/external/RIOTBOX-1423/wav -maxdepth 1 -type l -print | head -20
```

That command statted the named directories and enumerated entry metadata at
depth one. The symlink probe printed nothing; `ls` printed only the named
directory metadata. It opened, read, hashed, rendered, and played no audio.
Because it was a later filesystem audit, it cannot establish source
freshness, source quality, event eligibility, or holdout quality. Actual
RIOTBOX-1428 Stage-A qualification must restart in a fresh, bounded
development-access-log session; this audit cannot be reused as qualification
evidence.

After the musician requested an independent technical opinion on the admitted
`oga_frosty_ham_osdrums` source, a read-only technical suitability pass reopened
that exact registered development WAV with `ffprobe` / `ffmpeg` and Python
`wave` / NumPy, and inspected already-generated waveform and spectrogram PNGs.
It measured format, level, stereo balance, broad spectral regions, and possible
attack locations. It opened no holdout, produced no candidate, selected no
event, and changed no mechanism, threshold, identity, or renderer. These
measurements are source-family suitability context only and are forbidden as
Stage-A event-catalog or algorithm evidence. The later mechanism-blind
qualification still must begin in a fresh bounded development access-log
session.

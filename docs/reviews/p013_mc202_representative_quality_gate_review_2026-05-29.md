# P013 MC-202 Representative Quality Gate Review

Date: 2026-05-29

## Scope

RIOTBOX-1023 tightens the representative showcase path so the MC-202 lane is a
required musical-depth proof, not only a reported manifest metric. It also fixes
two showcase-gate drift points found while proving the slice: full-mix
source-diversity no longer treats similar loudness as a failure when spectral
evidence proves the sources differ, and the showcase generator now chooses an
observer probe whose Source Timing profile matches the manifest being
correlated. This slice does not add a new `ActionCommand`, Session model,
realtime audio path, live TUI state, or `JamAppState` state.

## Drift Check

- New `ActionCommand`: no
- Queue path covered: n/a
- Commit or side-effect path covered: n/a
- Session/replay consequence covered: n/a
- User-visible or observer surface covered: yes, via showcase validation JSON,
  markdown, observer/audio summary, and failure issues
- Test/QA proof included: yes
- Added `JamAppState` state: no
- Added or changed audio-producing behavior: yes, bounded offline MC-202
  bass-pressure output is stronger inside the representative showcase pack
- Shadow-system risk reviewed: yes, this reuses the existing representative
  showcase manifest and source-grid metrics

## Gate Change

The representative musical-quality gate now rejects a candidate when the MC-202
bass lane is missing audible bass pressure, lacks phrase/bar variation, exceeds
the static bar-similarity budget, misses the source-grid hit-ratio floor, or has
peak offsets beyond the source-grid window.

The existing `invalid_mc202_drift` fixture now proves the previous false
positive cannot pass: a candidate with good aggregate musical metrics but
`mc202_source_grid_alignment.hit_ratio = 0.0` and
`mc202_source_grid_alignment.max_peak_offset_ms = 999.0` fails with
`mc202_source_grid_alignment_too_weak` and
`mc202_source_grid_peak_offset_too_high`.

The source-diversity validator still rejects identical full-mix hashes, high
full-mix waveform correlation, and low normalized RMS delta when no spectral
evidence exists or when spectral distance is also low. It now allows low
normalized RMS delta when spectral-energy distance is clearly different. The
representative showcase case that motivated this was
`break_low_drive_128bpm.wav` versus `hat_cut_pressure_132bpm.wav`:

- Full-mix normalized signal RMS delta: `0.02811195550575623`
- Full-mix spectral-energy distance: `0.8868246366633687`
- Full-mix waveform correlation: `0.0013584023839486015`

## Audio Proof

Fresh representative showcase command:

```bash
scripts/run_compact.sh /tmp/riotbox-1023-showcase-fixed3.log scripts/generate_representative_source_showcase.sh /tmp/riotbox-1023-showcase-fixed3 local-riotbox-1023-fixed3 8.0 4
```

Selected musical candidate:

- Case/window: `break_low_drive/late`
- Result: `pass`
- Full mix RMS: `0.041796066`
- Full mix low-band RMS: `0.040449157`
- Support generated/source RMS ratio: `0.16191177`
- Source-first generated/source RMS ratio: `0.03601381`
- MC-202 bass-pressure RMS: `0.004488326`
- MC-202 bass-pressure low-band RMS: `0.0029217238`
- MC-202 source-grid hit ratio: `1.0`
- MC-202 source-grid max peak offset: `6.802721 ms`
- W-30 preview RMS: `0.23140997`

The observer/audio correlation block used a fallback-grid observer profile
against the representative manifest with `source_timing.grid_use =
"unavailable"`, producing aligned Source Timing evidence with no issues.

## Validation

```bash
python3 -m py_compile scripts/validate_representative_showcase_musical_quality.py scripts/validate_source_showcase_diversity.py
just representative-source-showcase-musical-quality-fixtures
just source-showcase-diversity-validator-fixtures
just source-showcase-diversity-report-fixtures
cargo test -p riotbox-audio --bin feral_grid_pack
scripts/run_compact.sh /tmp/riotbox-1023-showcase-fixed3.log scripts/generate_representative_source_showcase.sh /tmp/riotbox-1023-showcase-fixed3 local-riotbox-1023-fixed3 8.0 4
```

All listed commands passed locally.

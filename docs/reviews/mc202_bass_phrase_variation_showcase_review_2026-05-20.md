# MC-202 Bass Phrase Variation Showcase Review

Date: 2026-05-20

## Scope

RIOTBOX-807 deepens the bounded MC-202 bass-pressure lane inside the existing
`feral_grid_pack` representative showcase path. It does not add a new
`ActionCommand`, Session model, realtime mixer path, live TUI path, or
`JamAppState` state.

## Drift Check

- New `ActionCommand`: no
- Queue path covered: n/a
- Commit or side-effect path covered: n/a
- Session/replay consequence covered: n/a
- User-visible or observer surface covered: yes, via generated stem, manifest,
  report, and showcase musical-quality artifacts
- Test/QA proof included: yes
- Added `JamAppState` state: no
- Added or changed audio-producing behavior: yes, offline showcase output only
- Shadow-system risk reviewed: yes, this reuses the existing MC-202 offline
  render state and only sequences bounded render profiles per bar

## Audio Proof

Representative showcase command:

```bash
scripts/run_compact.sh /tmp/riotbox-807-showcase.log scripts/generate_representative_source_showcase.sh /tmp/riotbox-807-showcase local-riotbox-807 4.0 4
```

Selected candidate:

- Case/window: `break_low_drive/late`
- Result: `pass`
- Full mix RMS: `0.041497074`
- Full mix low-band RMS: `0.04014892`
- Support generated/source RMS ratio: `0.17172423`
- Source-first generated/source RMS ratio: `0.03711394`
- TR-909 kick-pressure low-band ratio: `1.3182278`
- MC-202 phrase variation applied: `true`
- MC-202 distinct bar profiles: `3`
- MC-202 bar similarity: `0.20694506`
- MC-202 bass-pressure RMS: `0.0057738633`
- MC-202 bass-pressure low-band RMS: `0.0037195687`
- W-30 offbeat triggers: `4`
- W-30 unique slice offsets: `6`
- Full mix bar similarity: `0.4862349`
- Event density per bar: `105.5`

The representative musical-quality validator now rejects MC-202 support when
phrase variation is missing, fewer than two distinct bar profiles are present,
or the MC-202 stem remains above the static bar-similarity budget.

## Validation

```bash
scripts/run_compact.sh /tmp/riotbox-807-feral-grid-tests.log cargo test -p riotbox-audio --bin feral_grid_pack
scripts/run_compact.sh /tmp/riotbox-807-fixtures.log just representative-source-showcase-musical-quality-fixtures
scripts/run_compact.sh /tmp/riotbox-807-showcase.log scripts/generate_representative_source_showcase.sh /tmp/riotbox-807-showcase local-riotbox-807 4.0 4
scripts/run_compact.sh /tmp/riotbox-807-syncopated.log just syncopated-source-showcase-smoke
python3 -m py_compile scripts/validate_representative_showcase_musical_quality.py
```

All listed commands passed locally.

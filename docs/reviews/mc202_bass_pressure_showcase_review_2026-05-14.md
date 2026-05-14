# MC-202 Bass-Pressure Showcase Review

Date: 2026-05-14

## Scope

RIOTBOX-806 adds a bounded MC-202 bass-pressure lane to the existing
`feral_grid_pack` representative showcase path. It does not add a new
`ActionCommand`, Session model, realtime mixer architecture, or live TUI path.

## Drift Check

- New `ActionCommand`: no
- Queue path covered: n/a
- Commit or side-effect path covered: n/a
- Session/replay consequence covered: n/a
- User-visible or observer surface covered: yes, via generated stem, manifest,
  report, and showcase README artifacts
- Test/QA proof included: yes
- Added `JamAppState` state: no
- Added or changed audio-producing behavior: yes
- Shadow-system risk reviewed: yes, this uses the existing MC-202 offline render
  state instead of a second bass-render path

## Audio Proof

Representative showcase command:

```bash
scripts/generate_representative_source_showcase.sh /tmp/riotbox-806-showcase local-riotbox-806 4.0 4
```

Selected candidate:

- Case/window: `break_low_drive/late`
- Result: `pass`
- Full mix RMS: `0.041565027`
- Full mix low-band RMS: `0.04023073`
- Support generated/source RMS ratio: `0.1700395`
- Source-first generated/source RMS ratio: `0.037191734`
- TR-909 kick-pressure low-band ratio: `1.3182278`
- MC-202 bass-pressure RMS: `0.005062084`
- MC-202 bass-pressure low-band RMS: `0.0032124845`
- W-30 offbeat triggers: `4`
- W-30 unique slice offsets: `6`
- Full mix bar similarity: `0.48656666`
- Event density per bar: `103.5`

The MC-202 lane is now rejected if it is missing, below `0.003` RMS, or below
`0.001` low-band RMS in the representative showcase validator and syncopated
smoke path.

## Validation

```bash
scripts/run_compact.sh /tmp/riotbox-806-feral-grid-tests.log cargo test -p riotbox-audio --bin feral_grid_pack
scripts/run_compact.sh /tmp/riotbox-806-fixtures.log just representative-source-showcase-musical-quality-fixtures
scripts/run_compact.sh /tmp/riotbox-806-showcase.log scripts/generate_representative_source_showcase.sh /tmp/riotbox-806-showcase local-riotbox-806 4.0 4
scripts/run_compact.sh /tmp/riotbox-806-pycompile.log python3 -m py_compile scripts/validate_representative_showcase_musical_quality.py scripts/validate_source_showcase_diversity.py scripts/validate_stage_style_stability_proof.py
scripts/run_compact.sh /tmp/riotbox-806-syncopated-smoke.log just syncopated-source-showcase-smoke
scripts/run_compact.sh /tmp/riotbox-806-fmt.log cargo fmt --check
scripts/run_compact.sh /tmp/riotbox-806-clippy-audio.log cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings
scripts/run_compact.sh /tmp/riotbox-806-audio-qa-ci.log just audio-qa-ci
git diff --check
```

All listed commands passed locally.

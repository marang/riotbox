# P013 W-30 Source Accent Dynamics Review

Date: 2026-05-29

## Scope

RIOTBOX-1024 deepens W-30 behavior in the representative Feral-grid showcase by
deriving trigger velocity from selected source-slice energy and source-offset
position. The slice stays inside the existing offline showcase path and does not
add a new `ActionCommand`, Session model, realtime audio callback path, live TUI
state, or `JamAppState` state.

## Drift Check

- New `ActionCommand`: no
- Queue path covered: n/a
- Commit or side-effect path covered: n/a
- Session/replay consequence covered: n/a
- User-visible or observer surface covered: yes, via W-30 manifest metrics,
  report text, and representative showcase validation artifacts
- Test/QA proof included: yes
- Added `JamAppState` state: no
- Added or changed audio-producing behavior: yes, bounded offline W-30
  source-chop trigger velocity now varies from source-slice evidence, with
  source-energy span reported independently from the final velocity span
- Shadow-system risk reviewed: yes, this reuses the existing source window,
  slice-choice, trigger-variation, source-grid, and musical-quality gates

## Audio Proof

Fresh representative showcase command:

```bash
scripts/run_compact.sh /tmp/riotbox-1024-showcase-after-energy-proof.log scripts/generate_representative_source_showcase.sh /tmp/riotbox-1024-showcase-after-energy-proof local-riotbox-1024-energy 8.0 4
```

Selected musical candidate:

- Case/window: `tonal_hook_chop/head`
- Result: `pass`
- Full mix RMS: `0.03191345`
- Support generated/source RMS ratio: `0.21667686`
- W-30 preview RMS: `0.16920625`
- W-30 unique slice offsets: `6`
- W-30 accent distinct velocities: `5`
- W-30 accent velocity span: `0.2982387`
- W-30 accent source-energy span: `0.9400456`
- MC-202 bass-pressure RMS: `0.0044834786`

The source-diversity summary passed with no failures, and observer/audio
correlation kept Source Timing alignment at `status = "aligned"` with no issues.

## Validation

```bash
cargo test -p riotbox-audio --bin feral_grid_pack w30 -- --nocapture
cargo test -p riotbox-audio --bin feral_grid_pack
python3 -m py_compile scripts/validate_representative_showcase_musical_quality.py
just representative-source-showcase-musical-quality-fixtures
scripts/run_compact.sh /tmp/riotbox-1024-showcase-after-energy-proof.log scripts/generate_representative_source_showcase.sh /tmp/riotbox-1024-showcase-after-energy-proof local-riotbox-1024-energy 8.0 4
scripts/run_compact.sh /tmp/riotbox-1024-ci-final.log just ci
scripts/run_compact.sh /tmp/riotbox-1024-audio-qa-ci-final.log just audio-qa-ci
```

All listed commands passed locally.

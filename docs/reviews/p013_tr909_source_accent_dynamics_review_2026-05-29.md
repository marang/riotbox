# P013 TR-909 Source Accent Dynamics Review

Date: 2026-05-29

## Scope

RIOTBOX-1026 deepens the representative Feral-grid TR-909 support lane by
applying source-shaped accent factors to the existing kick-pressure anchors and
recording a manifest proof that those accents are not flat. The slice stays
inside the offline showcase path and does not add a new `ActionCommand`, Session
model, realtime callback path, live TUI state, or `JamAppState` state.

## Drift Check

- New `ActionCommand`: no
- Queue path covered: n/a
- Commit or side-effect path covered: n/a
- Session/replay consequence covered: n/a
- User-visible or observer surface covered: yes, via TR-909 manifest metrics,
  report text, and representative showcase validation artifacts
- Test/QA proof included: yes
- Added `JamAppState` state: no
- Added or changed audio-producing behavior: yes, bounded offline TR-909
  kick-pressure anchors now use source-shaped accent factors
- Shadow-system risk reviewed: yes, this reuses the existing TR-909 source
  profile, kick-pressure, source-grid, and musical-quality gates

## Audio Proof

Fresh representative showcase command:

```bash
scripts/run_compact.sh /tmp/riotbox-1026-showcase-final.log scripts/generate_representative_source_showcase.sh /tmp/riotbox-1026-showcase-final local-riotbox-1026-final 8.0 4
```

Selected musical candidate:

- Case/window: `tonal_hook_chop/head`
- Result: `pass`
- Full mix RMS: `0.03214914`
- Support generated/source RMS ratio: `0.22443631`
- TR-909 accent distinct accents: `3`
- TR-909 accent span: `1.0313376`
- TR-909 accent source-energy span: `0.7535631`
- TR-909 kick-pressure low-band ratio: `1.4053622`
- W-30 accent velocity span: `0.2982387`
- MC-202 bass-pressure RMS: `0.0044834786`

The source-diversity summary passed with no failures, and observer/audio
correlation kept Source Timing alignment at `status = "aligned"` with no issues.

## Validation

```bash
cargo test -p riotbox-audio --bin feral_grid_pack
python3 -m py_compile scripts/validate_representative_showcase_musical_quality.py
just representative-source-showcase-musical-quality-fixtures
scripts/run_compact.sh /tmp/riotbox-1026-showcase-final.log scripts/generate_representative_source_showcase.sh /tmp/riotbox-1026-showcase-final local-riotbox-1026-final 8.0 4
scripts/run_compact.sh /tmp/riotbox-1026-audio-qa-ci-final.log just audio-qa-ci
scripts/run_compact.sh /tmp/riotbox-1026-ci-final.log just ci
```

All listed commands passed locally.

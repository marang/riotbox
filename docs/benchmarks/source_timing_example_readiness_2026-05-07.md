# Source Timing Example Readiness

Date: 2026-05-07
Rechecked: 2026-05-10

Status: local real-example readiness check for Feral grid packs after the
Source Timing readiness-to-pack bridge and explicit dance-loop auto-readiness
policy.

## Purpose

This benchmark records what the current P012 timing path can honestly say about
the example WAV files users are likely to try first.

It is not a detector-quality claim. The example WAV files are intentionally not
committed to the repo, so this is a local evidence note for the current machine's
available `data/test_audio/examples/` corpus.

## Command

Each present source was rendered with the current auto-BPM Feral grid path:

```bash
just feral-grid-pack "data/test_audio/examples/<source>.wav" <date> auto 2 1.0 0.0
```

The Feral grid auto path uses `dance_loop_auto_readiness`, not the wider
`broad_research` timing policy. The broader policy is still useful for detector
diagnostics because it preserves half-time / double-time alternatives more
aggressively.

Each generated manifest records that decision as
`source_timing.policy_profile: dance_loop_auto_readiness`.

The report values below come from each generated `manifest.json`. Newer Feral grid
packs also surface the same compact readiness, downbeat, phrase, and warning
evidence in `README.md` and `grid-report.md` for human QA:

- `grid_bpm_source`
- `grid_bpm_decision_reason`
- `bpm`
- `source_timing.policy_profile`
- `source_timing.primary_bpm`
- `source_timing_bpm_delta`
- `source_timing.bpm_agrees_with_grid`
- `source_timing.readiness`
- `source_timing.requires_manual_confirm`
- `source_timing.anchor_evidence`
- `source_timing.warning_codes`

## Results

The 2026-05-10 recheck uses the real `source_timing_probe --json` CLI directly
against local example WAVs. The Feral grid pack manifest can still choose a
fallback grid policy independently, so this table records probe readiness rather
than pack BPM decision fields.

| Source | Cue | Readiness | Manual confirm | Primary BPM | Beat | Downbeat | Phrase | Warnings | Practical guidance |
| --- | --- | --- | --- | ---: | --- | --- | --- | --- | --- |
| `Beat03_130BPM(Full).wav` | `needs confirm` | `ready` | yes | 130.285 | stable | stable | not_enough_material | `PhraseUncertain` | BPM, beat, and downbeat are useful, but short-loop phrase absence still blocks automatic trust. |
| `Beat08_128BPM(Full).wav` | `needs confirm` | `ready` | yes | 128.397 | stable | stable | not_enough_material | `PhraseUncertain` | Same: usable grid evidence, still needs explicit confirmation for phrase-level confidence. |
| `Beat20_128BPM(Full).wav` | `needs confirm` | `weak` | yes | 128.397 | stable | weak | not_enough_material | `PhraseUncertain`, `AmbiguousDownbeat` | Keep conservative; downbeat evidence is not stable enough. |
| `DH_BeatC_120-01.wav` | `needs confirm` | `ready` | yes | 120.185 | stable | stable | not_enough_material | `PhraseUncertain` | Useful short-loop timing candidate; do not treat as long-phrase locked yet. |
| `DH_BeatC_KickSnr_120-01.wav` | `needs confirm` | `ready` | yes | 120.185 | stable | stable | not_enough_material | `PhraseUncertain` | Useful short-loop timing candidate; do not treat as long-phrase locked yet. |
| `DH_Fadapad_120_A.wav` | `needs confirm` | `unavailable` | yes | none | unavailable | unavailable | unavailable | `LowTimingConfidence`, `WeakKickAnchor` | Correctly not a drum-timing source for this probe path. |
| `DH_RushArp_120_A.wav` | `needs confirm` | `unavailable` | yes | none | unavailable | unavailable | unavailable | `LowTimingConfidence`, `WeakKickAnchor` | Needs a melodic/source-chop path, not TR-style timing trust. |

## Interpretation

The current examples are more auditable than before, but not automatically better
for every source:

- the CLI now finds useful BPM and often stable downbeat evidence for several
  short drum loops
- `readiness=ready` can still pair with `requires_manual_confirm=true`; this is
  safe but confusing and should be refined before musicians depend on it
- short-loop phrase absence is currently reported as `not_enough_material` /
  `PhraseUncertain`, even when beat and downbeat are stable
- sources with a known BPM should still use explicit BPM in musician-facing
  examples when the cue remains `needs confirm`
- melodic or non-drum sources can still fail the current TR-909 support render
  gate and should not be presented as Feral grid examples yet

This directly explains why generated examples can still feel like the previous
ones unless the user chooses an explicit BPM or the detector reaches a more
clearly trusted short-loop state.

## Follow-Up

Near-term follow-up should not loosen readiness just to make examples look better.
Instead:

- refine short-loop readiness semantics so stable BPM/beat/downbeat evidence can
  be used conservatively without pretending long-phrase confidence exists
- keep explicit-BPM examples documented where the current detector is still weak
- add a separate melodic/source-chop showcase path for non-drum sources such as
  `DH_RushArp_120_A.wav`

# Source Timing Example Readiness

Date: 2026-05-07
Rechecked: 2026-05-08

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

| Source | Grid BPM source | Grid BPM | Primary BPM | Delta | Agrees | Readiness | Manual confirm | Warnings | Practical guidance |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | --- | --- |
| `Beat03_130BPM(Full).wav` | `static_default` | 128.000 | 130.285 | 2.285 | no | `weak` | yes | `PhraseUncertain`, `AmbiguousDownbeat` | Use explicit `130 BPM` for listening examples until readiness improves. |
| `Beat08_128BPM(Full).wav` | `static_default` | 128.000 | 128.397 | 0.397 | yes | `weak` | yes | `PhraseUncertain`, `AmbiguousDownbeat` | Timing is BPM-close, but still not auto-trusted because downbeat/phrase evidence is weak. |
| `Beat20_128BPM(Full).wav` | `static_default` | 128.000 | 128.397 | 0.397 | yes | `weak` | yes | `PhraseUncertain`, `AmbiguousDownbeat` | Same as Beat08: acceptable BPM agreement, not enough readiness for automatic trust. |
| `DH_BeatC_120-01.wav` | `static_default` | 128.000 | 120.185 | 7.815 | no | `weak` | yes | `PhraseUncertain`, `AmbiguousDownbeat` | Use explicit `120 BPM`; auto fallback is musically misleading here. |
| `DH_RushArp_120_A.wav` | no manifest | unknown | unknown | unknown | unknown | unknown | unknown | render failed: `tr909 rendered near silence` | Not a good Feral grid pack source for this drum-support path yet. Needs a separate melodic/source-chop example path. |

## Interpretation

The current examples are more auditable than before, but not automatically better
for every source:

- the pack now exposes when auto mode fell back to `static_default`
- the Feral grid auto policy no longer reports generic half-time ambiguity for
  these examples, but still refuses auto-trust while downbeat / phrase evidence
  is weak
- BPM agreement is visible even when readiness remains weak
- sources with a known BPM should still use explicit BPM in musician-facing
  examples when readiness is not `ready`
- melodic or non-drum sources can still fail the current TR-909 support render
  gate and should not be presented as Feral grid examples yet

This directly explains why generated examples can still feel like the previous
ones unless the user chooses an explicit BPM or the detector reaches `ready`.

## Follow-Up

Near-term follow-up should not loosen readiness just to make examples look better.
Instead:

- improve downbeat and phrase evidence so stable real sources can become `ready`
- keep explicit-BPM examples documented where the current detector is still weak
- add a separate melodic/source-chop showcase path for non-drum sources such as
  `DH_RushArp_120_A.wav`

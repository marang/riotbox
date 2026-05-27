# P012 Beat20 Downbeat Feasibility Guardrail - 2026-05-27

## Scope

Focused P012 check for `RIOTBOX-1014` after the source transport and
real-source reporting slices closed.

Reviewed:

- `docs/execution_roadmap.md`
- `docs/specs/source_timing_intelligence_spec.md`
- `docs/reviews/p012_real_source_timing_confidence_review_2026-05-22.md`
- `docs/reviews/p012_post_source_transport_spine_review_2026-05-26.md`
- current local report from
  `just source-timing-example-probe-report-local /tmp/riotbox-next-source-timing-report.md`
- current downbeat and anchor scoring in
  `crates/riotbox-core/src/source_graph/timing_probe_candidates/`

This review does not change analyzer, Session, action, JamAppState, UI,
observer schema, realtime audio, or audio-output behavior.

## Current Beat20 Evidence

`Beat20_128BPM(Full).wav` remains a useful but ambiguous real-source timing
case:

| Field | Current value |
| --- | ---: |
| BPM | `128.397` |
| Beat status | `stable` |
| Beat score / match / median | `0.992` / `1.000` / `0.006` |
| Downbeat status | `ambiguous` |
| Selected downbeat offset | `2` |
| Downbeat score | `0.273` |
| Downbeat margin | `0.005` |
| Alternate downbeat phases | `3` |
| Alternate evidence count | `6` |
| Anchor evidence | `11/0/0/11` |
| Grid use | `manual_confirm_only` |
| Warnings | `phrase_uncertain,ambiguous_downbeat` |

The important result is that Beat20 has strong beat-period evidence, but no
trusted kick or backbeat anchors and no reliable bar-one phase margin.

## Low-Band Probe Check

A bounded local low-band check was run against the same 1024-frame / 512-hop
windowing used by the current probe. It compared full-band flux/RMS and a simple
165 Hz one-pole low-pass flux/RMS phase score over the detected onset windows.

Observed phase-margin summary:

| Scoring input | Best offset | Best score | Margin |
| --- | ---: | ---: | ---: |
| Full-band RMS, current effective signal | `2` | `0.273` | `0.005` |
| Full-band positive flux | `1` | `0.271` | `0.015` |
| Low-band positive flux | `2` | `0.264` | `0.007` |
| Low-band RMS | `2` | `0.283` | `0.027` |

The best low-band variant improves the margin, but only to about `0.027`. That
is still below the current downbeat ambiguity margin of `0.05` and does not
create kick/backbeat anchor support.

## Decision

Do not promote Beat20-like rows to `locked_grid`, `short_loop_manual_confirm`,
or stable downbeat evidence from the current full-band or simple low-band
scoring alone.

Future detector work may target Beat20, but it must first add stronger musical
evidence, such as trusted kick/backbeat anchors, a clearly larger phase margin,
or another documented downbeat cue that survives fixture and local example
expectations. Until then, the correct behavior is the current
`manual_confirm_only` / `ambiguous_downbeat` result.

## Verification

```bash
scripts/run_compact.sh /tmp/riotbox-next-source-timing-report.log just source-timing-example-probe-report-local /tmp/riotbox-next-source-timing-report.md
git diff --check
```

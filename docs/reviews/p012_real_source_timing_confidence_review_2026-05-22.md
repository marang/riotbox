# P012 Real Source Timing Confidence Review - 2026-05-22

## Scope

Focused review for `RIOTBOX-929` to decide the next bounded P012 real-source
timing-confidence implementation slice from current local example evidence.

Reviewed:

- `docs/execution_roadmap.md`
- `docs/reviews/p012_current_source_timing_spine_review_2026-05-21.md`
- `just source-timing-example-probe-report-local`
- local source examples under `data/test_audio/examples/`

This review did not change analyzer, Session, action, JamAppState, UI, observer,
or audio-output behavior.

## Current Example Rows

The current local example report matches its expectation gate for every present
row.

| Source | Readiness | Grid use | Confidence | Beat | Downbeat | Phrase | Key blocker |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `Beat03_130BPM(Full).wav` | `needs_review` | `short_loop_manual_confirm` | `candidate_cautious` | `stable` | `stable` | `not_enough_material` | short phrase evidence |
| `Beat08_128BPM(Full).wav` | `needs_review` | `short_loop_manual_confirm` | `candidate_cautious` | `stable` | `stable` | `not_enough_material` | short phrase evidence |
| `Beat20_128BPM(Full).wav` | `needs_review` | `manual_confirm_only` | `candidate_ambiguous` | `stable` | `ambiguous` | `not_enough_material` | ambiguous downbeat |
| `DH_BeatC_120-01.wav` | `needs_review` | `short_loop_manual_confirm` | `candidate_cautious` | `stable` | `stable` | `not_enough_material` | short phrase evidence |
| `DH_BeatC_KickSnr_120-01.wav` | `needs_review` | `short_loop_manual_confirm` | `candidate_cautious` | `stable` | `stable` | `not_enough_material` | short phrase evidence |
| `DH_Fadapad_120_A.wav` | `unavailable` | `unavailable` | `degraded` | `unavailable` | `unavailable` | `unavailable` | sparse onsets / weak anchors |
| `DH_RushArp_120_A.wav` | `unavailable` | `unavailable` | `degraded` | `unavailable` | `unavailable` | `unavailable` | sparse onsets / weak anchors |

## Findings

No row currently justifies a blind promotion to locked or ready source timing.
The conservative behavior is working: stable short-loop rows get a useful
manual-confirm path, weak melodic/pad rows stay unavailable, and the ambiguous
drum-loop row does not pretend to know the downbeat.

`Beat20_128BPM(Full).wav` is the best next implementation target because it is
musically plausible but below the desired readiness boundary for a specific,
explainable reason:

- BPM and beat evidence are stable: `128.397`, beat score `0.992`, beat match
  `1.000`, beat median `0.006`, and no beat alternates.
- Downbeat evidence is not stable enough: downbeat margin `0.005`, downbeat
  alternates `3`, alternate evidence `6`, and warning `ambiguous_downbeat`.
- Anchor evidence is transient-only: `11/0/0/11`, so there are no kick or
  backbeat anchors to support a stronger downbeat claim.

The correct near-term product move is not to make this row sound more confident.
It is to make the reason for caution visible through the shared source-timing
surface, so a musician can tell whether they are confirming a short loop, a weak
source, or a genuinely ambiguous downbeat.

## Recommended Next Slice

Create a bounded implementation ticket to surface downbeat ambiguity evidence in
the shared Source Timing summary used by Jam, Source, and observer-facing paths.

Minimum behavior:

- preserve current analyzer classification and fallback behavior
- expose compact downbeat ambiguity facts when present:
  - downbeat status
  - alternate count
  - margin or equivalent confidence gap
  - whether the source is manual-confirm-only because of ambiguity
- keep stable short-loop rows phrased differently from ambiguous rows
- prove the result through a focused fixture or snapshot using the current
  `Beat20_128BPM(Full).wav` evidence shape, without requiring production-grade
  arbitrary-audio detection

Musician-facing effect:

- A musician no longer sees all `needs confirm` cases as the same problem.
- Stable short loops read as "confirm this plausible grid first."
- Beat20-like rows read as "the beat is steady, but bar one is ambiguous."
- Riotbox stays honest instead of falsely locking generated lanes to an
  uncertain downbeat.

## Deferred Follow-Up

After the ambiguity surface is visible, a later fixture-backed implementation can
try to improve downbeat phase evidence for one Beat20-like row. That follow-up
must prove a musically defensible anchor and keep the ambiguous/manual-confirm
fallback intact for sources without kick/backbeat support.

## Verification Commands

```bash
scripts/run_compact.sh /tmp/riotbox-929-source-timing-report.log just source-timing-example-probe-report-local /tmp/riotbox-929-source-timing-example-report.md
sed -n '1,220p' /tmp/riotbox-929-source-timing-example-report.md
```

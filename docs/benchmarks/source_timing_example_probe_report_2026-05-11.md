# Source Timing Example Probe Report - 2026-05-11

## Purpose

This note captures the current optional local Source Timing example baseline for
the documented Beat/DH example WAVs.

The source WAV files are intentionally not committed. This benchmark note is a
local review baseline, not a mandatory CI input and not a production arbitrary
audio detector claim.

## Command

```bash
just source-timing-example-probe-report-local artifacts/audio_qa/local/source_timing_example_probe_report_expected.md
```

This command uses:

- `scripts/source_timing_example_probe_report.py`
- `scripts/fixtures/source_timing_example_probe_report/local_example_expectations.json`

Missing example WAV rows are reported as `missing` / `skipped` instead of
failing, so the command stays safe for fresh clones.

## Captured Local Result

Captured from this checkout with the local example WAVs present.

| Source | Status | Cue | Readiness | Manual confirm | Grid use | BPM | Confidence | Drift | Beat | Beat score | Downbeat | Downbeat score | Phrase | Alternate evidence | Warnings | Anchors total/kick/backbeat/transient | Groove residuals | Expectation |
| --- | --- | --- | --- | --- | --- | ---: | --- | --- | --- | ---: | --- | ---: | --- | ---: | --- | --- | ---: | --- |
| Beat03_130BPM(Full).wav | probed | needs confirm | needs_review | yes | short_loop_manual_confirm | 130.285 | candidate_cautious | not_enough_material | stable | 0.902 | stable | 0.356 | not_enough_material | 0 | phrase_uncertain | 11/4/3/4 | 4 | ok |
| Beat08_128BPM(Full).wav | probed | needs confirm | needs_review | yes | short_loop_manual_confirm | 128.397 | candidate_cautious | not_enough_material | stable | 0.992 | stable | 0.315 | not_enough_material | 0 | phrase_uncertain | 9/3/2/4 | 4 | ok |
| Beat20_128BPM(Full).wav | probed | needs confirm | weak | yes | manual_confirm_only | 128.397 | candidate_ambiguous | not_enough_material | stable | 0.992 | weak | 0.273 | not_enough_material | 6 | phrase_uncertain,ambiguous_downbeat | 11/0/0/11 | 4 | ok |
| DH_BeatC_120-01.wav | probed | needs confirm | needs_review | yes | short_loop_manual_confirm | 120.185 | candidate_cautious | not_enough_material | stable | 0.997 | stable | 0.304 | not_enough_material | 0 | phrase_uncertain | 8/2/4/2 | 4 | ok |
| DH_BeatC_KickSnr_120-01.wav | probed | needs confirm | needs_review | yes | short_loop_manual_confirm | 120.185 | candidate_cautious | not_enough_material | stable | 0.997 | stable | 0.304 | not_enough_material | 0 | phrase_uncertain | 8/2/4/2 | 4 | ok |
| DH_Fadapad_120_A.wav | probed | needs confirm | unavailable | yes | unavailable | none | degraded | unavailable | unavailable | none | unavailable | none | unavailable | 0 | low_timing_confidence,weak_kick_anchor | 0/0/0/0 | 0 | ok |
| DH_RushArp_120_A.wav | probed | needs confirm | unavailable | yes | unavailable | none | degraded | unavailable | unavailable | none | unavailable | none | unavailable | 0 | low_timing_confidence,weak_kick_anchor | 0/0/0/0 | 0 | ok |

## Interpretation

- Current short drum-loop examples have stable beat/downbeat evidence but still
  require manual confirmation because phrase evidence is not long enough.
- `Beat20_128BPM(Full).wav` remains intentionally weaker because downbeat
  evidence is ambiguous.
- Pad/arp examples remain unavailable instead of being promoted as drum timing
  sources.
- The evidence columns make the readiness decision reviewable without opening
  the raw probe JSON: short drum loops show cautious confidence and
  not-enough-material drift/phrase evidence, while the weaker Beat20 row shows
  ambiguous confidence and alternate evidence.
- This is a conservative P012 review surface. It should catch accidental shifts
  in the current timing contract without pretending the detector is finished.

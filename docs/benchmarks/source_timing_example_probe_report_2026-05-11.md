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

| Source | Status | Cue | Readiness | Manual confirm | BPM | Beat | Downbeat | Phrase | Warnings | Anchors total/kick/backbeat/transient | Groove residuals | Expectation |
| --- | --- | --- | --- | --- | ---: | --- | --- | --- | --- | --- | ---: | --- |
| Beat03_130BPM(Full).wav | probed | needs confirm | needs_review | yes | 130.285 | stable | stable | not_enough_material | phrase_uncertain | 11/4/3/4 | 4 | ok |
| Beat08_128BPM(Full).wav | probed | needs confirm | needs_review | yes | 128.397 | stable | stable | not_enough_material | phrase_uncertain | 9/3/2/4 | 4 | ok |
| Beat20_128BPM(Full).wav | probed | needs confirm | weak | yes | 128.397 | stable | weak | not_enough_material | phrase_uncertain,ambiguous_downbeat | 11/0/0/11 | 4 | ok |
| DH_BeatC_120-01.wav | probed | needs confirm | needs_review | yes | 120.185 | stable | stable | not_enough_material | phrase_uncertain | 8/2/4/2 | 4 | ok |
| DH_BeatC_KickSnr_120-01.wav | probed | needs confirm | needs_review | yes | 120.185 | stable | stable | not_enough_material | phrase_uncertain | 8/2/4/2 | 4 | ok |
| DH_Fadapad_120_A.wav | probed | needs confirm | unavailable | yes | none | unavailable | unavailable | unavailable | low_timing_confidence,weak_kick_anchor | 0/0/0/0 | 0 | ok |
| DH_RushArp_120_A.wav | probed | needs confirm | unavailable | yes | none | unavailable | unavailable | unavailable | low_timing_confidence,weak_kick_anchor | 0/0/0/0 | 0 | ok |

## Interpretation

- Current short drum-loop examples have stable beat/downbeat evidence but still
  require manual confirmation because phrase evidence is not long enough.
- `Beat20_128BPM(Full).wav` remains intentionally weaker because downbeat
  evidence is ambiguous.
- Pad/arp examples remain unavailable instead of being promoted as drum timing
  sources.
- This is a conservative P012 review surface. It should catch accidental shifts
  in the current timing contract without pretending the detector is finished.

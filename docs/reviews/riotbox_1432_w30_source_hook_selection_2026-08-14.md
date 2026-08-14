# RIOTBOX-1432 W-30 Source Hook Selection

Date: 2026-08-14  
Result: `no_winner_fail_closed`  
Human playback: not authorized because the three-source technical gate did not pass

## Frozen boundary

RBX-285 and source-blind commit `52003511` froze the complete analysis,
eligibility, scoring, tie-breaking, score-lift, Session-lineage, and activation
contract before any selected Development WAV was opened. The comparison reused
the existing `CaptureBarGroup -> W30AuditionRawCapture ->
PromoteCaptureToPad -> W30TriggerPad` product spine and exact RuntimeMix
renderer. No Holdout audio, commercial reference, or source directory was
opened or searched.

Exactly these registered Development files were opened, using their existing
timing-confirmation contracts:

- `Beat03_130BPM(Full).wav` at 130 BPM
- `DH_RushArp_120_A.wav` at the established musician-owned 120 BPM / 0-second manual grid
- `DH_BeatC_KickSnr_120-01.wav` at 120 BPM confirmation

Each source received exactly the baseline plus the two frozen policies. No
feature, constant, threshold, weight, eligibility rule, or render behavior was
changed after observing a result.

## Development result

| Family | Baseline | `attack_body_contrast_v1` | `repetition_salience_v1` | Consequence |
| --- | --- | --- | --- | --- |
| dense break | range `0.000000–1.842116s`; WAV `a7326f8a…` | retained baseline; insufficient eligible bars; same WAV | retained baseline; insufficient eligible bars; same WAV | primary bar confidence `0.196067` is below frozen `0.35`; no candidate |
| tonal riff | bar 1, `0.000000–2.000000s`; WAV `516c0bc3…` | selected bar 2, `2.000000–4.000000s`; score/lift `0.65/0.65`; WAV `0f2ce0b3…` | selected the same bar with the same score/lift and byte-identical WAV | real audible-input change exists, but the two candidate families collapse to one decision |
| sparse drums | range `0.000000–1.996916s`; WAV `89204640…` | retained baseline; insufficient eligible bars; same WAV | retained baseline; insufficient eligible bars; same WAV | primary bar confidence `0.167215` is below frozen `0.35`; no candidate |

Full live-hook SHA-256 values:

- dense baseline / attack / repetition:
  `a7326f8a16fd76d4b9d7959c3a871724a763555f05b0edc4f6fb1ac69d6c31f7`
- tonal baseline:
  `516c0bc3fb2ae2ba4e77adbc864933e9000f54026529f2596d8e9a04a203d42d`
- tonal attack / repetition:
  `0f2ce0b34f4c6f3276d8c4fc331ff2dde6d5478f55fe09651cac54b728b63bd1`
- sparse baseline / attack / repetition:
  `89204640f8717f0721215f3666dca737492c2172c5d94d95a118c25c5505463d`

All nine exact RuntimeMix renders are non-silent and have zero clipped or
near-clipped samples. Normal-path peak/RMS pairs are `0.399897/0.067608`
(dense), `0.268649/0.104270` baseline and `0.287453/0.106262` selected
(tonal), and `0.369202/0.081763` (sparse).

## Product verdict

Neither frozen policy qualifies as a product winner. Both fail to change two of
the three required source families, and they are byte-identical to one another
on the only family they change. That fails the required cross-source diversity
and does not justify changing `FeralBreakAlphaV2` away from
`transport_boundary_v1`.

The Beat03 baseline/candidate human gate was not opened: its candidate bytes are
identical to baseline, and the prerequisite three-source technical gate failed.
Playback would add no evidence. The typed analysis and fail-closed capture
lineage remain useful infrastructure, but this slice makes no musical-quality
claim and leaves current product behavior unchanged.

# RIOTBOX-1443 W-30 Pitch-Dive Transfer Observation

Date: 2026-08-18
Partition: Development only
Result: `informative_positive_transfer_observation`
Mechanism changes: none
Holdout access: none
Commercial-reference access: none

## Question

Does unchanged frozen `w30_pitch_dive_v1` remain musically useful and
source-recognizable on four additional registered Development sources before
Riotbox pays for product-spine integration?

## Access And Version Record

The first source set stopped before pitch-dive rendering when its fourth case
failed the unchanged one-BPM tempo-admission tolerance. The v2 replacement also
stopped before pitch-dive rendering for a declared/detected tempo mismatch. No
candidate or human playback from either failed session was used as evidence.

The fresh v3 session verified exactly four registered hashes before decode and
completed without source-directory discovery. Its completed access-log SHA-256
is `d20a9162b09f32992176acadf296226a89ab24af866b3825c9f2cb5e4536dc1e`.
No Holdout or commercial-reference audio was opened.

## Technical Evidence

Each case used the existing product-path W-30 live-hook render as A. B applied
the exact RBX-299 continuous pitch curve after eight sample-exact control beats.
Every B contained only one frozen topology, had a non-silent intended
four-beat delta, produced no clipped samples, and ended with a silent terminal
frame.

| Case | Family | Product BPM | A SHA-256 | B SHA-256 | Final-four-beat delta RMS |
| --- | --- | ---: | --- | --- | ---: |
| `freesound_alastair_pursloe_183441` | dense break | `135.11032` | `e549271d651be5cb2a4310cf20d8e78ee2a07d560d27497fecf22125d30eefd7` | `89c6cee900033c6384a972d8fb1b8d183a1ed09e199b38cbd6bc1d19696e3cb1` | `0.160840` |
| `freesound_dabromusic_266735` | dense break | `172.26566` | `2cf1eb27a6b0836bd31629d18eae0375b70a961becd73fc129e198de5a6cd7af` | `1d76a5a6e4e527c865f72ec605588ce763eb5eb34c23ba71bdc650b62b0f038d` | `0.091311` |
| `freesound_dr_skitz_353853` | sparse drums | `119.68088` | `343dd11d0f2bb7ebbe1f266cf9e32bc1620e1699c516dcb2882348ab2baa83ce` | `70d6dcfe5b8cc45b2fd6f04de198bc3bf77a6ccae3dd1e10f8b28425b3e42716` | `0.110687` |
| `tonal_rusharp_120` | tonal riff | `120.0` | `c03db0c7fb4465ad86806a35a1281758fd52fb36ac706dbfa473f9ce08b0113d` | `63ff96fdcc870f06a387128928756e6f884dfa0fbe07cf219b07cf08f7f25bf1` | `0.141834` |

The source-first review composites used one-second role pauses, two A
presentations, and two B presentations. Source context was attenuated by 6 dB
for playback safety; repeated-role joins used a 20 ms crossfade and endpoint
fade. These presentation edits do not alter the separately hash-bound A/B
evidence above.

| Case | Composite SHA-256 | Duration | Integrated loudness | True peak |
| --- | --- | ---: | ---: | ---: |
| `freesound_alastair_pursloe_183441` | `d46d919c813b7145aee9771748a6165f4b63cba44fc6b795bbc0c31468da2ebf` | `37.498146` s | `-16.6` LUFS | `-6.0` dBFS |
| `freesound_dabromusic_266735` | `86ca5a45a0f4449ca3654d569b42395d3b2aefab0ef5331036b9cbc2a6161ee1` | `21.469042` s | `-18.9` LUFS | `-7.0` dBFS |
| `freesound_dr_skitz_353853` | `7fb266c6c41e9c5a4fab9b8ff5581b12edb3196a0522a9bcf51182f3ca13dae4` | `30.044000` s | `-22.6` LUFS | `-6.2` dBFS |
| `tonal_rusharp_120` | `65cbc2e5f0663fd4443aa943cdc2f0f64db8a012af381e5d666d7e8d3f1c11c2` | `29.960000` s | `-17.1` LUFS | `-7.6` dBFS |

All four composites were stereo PCM16 at 48 kHz, stopped at their announced
endpoints, and left no active playback stream.

## Human Result

The project musician gave the same positive source-bound assessment in all
four cases: A was already a successful transformation, while B improved it
further through the pitch-dive exit. No loss of groove, clarity, recognizable
source identity, or musical usefulness was reported. The tonal case matched
the three drum-led cases rather than exposing a family-specific failure.

The durable aggregate is therefore:

- A was useful on four of four additional sources
- B was preferred over A on four of four sources
- the unchanged Pitch Dive transferred across dense break, sparse drum, and
  tonal-riff material
- no heard case justified per-source tuning or rejection

## Product Boundary

This is strong positive Development transfer evidence, not product
qualification. B came from a temporary post-control renderer, which has been
removed. Product work must rebuild RBX-299 through the canonical W-30
queue/commit, Session/replay, observer/UI, and exact RuntimeMix path, then pass
fresh automated source-diversity gates and one bounded exact-product human
review. It must not retune the accepted curve, timing, fade, or ownership and
does not yet authorize Holdout, hardness, Golden Path, demo, or release claims.

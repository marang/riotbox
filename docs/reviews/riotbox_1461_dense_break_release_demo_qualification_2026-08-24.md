# RIOTBOX-1461 Dense-Break Release-Demo Qualification

- Date: 2026-08-24
- Source family: `dense_break`
- Development case: `dense_beat03_130`
- Human verdict: `reject`
- Demo readiness: `not_demo_ready`
- Release readiness: `blocked`
- Quality claim allowed: `false`

## Evidence Boundary

The historical RIOTBOX-1402 live-review WAV existed only in temporary local
storage and was no longer available. RIOTBOX-1461 did not reconstruct, relabel,
or infer a new verdict from that missing artifact. Instead, it opened exactly
the registered Development source
`data/test_audio/examples/Beat03_130BPM(Full).wav` in one bounded access
session and generated a fresh current-state professional diagnostic pack.

The source SHA-256 was
`e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`.
The registry identity, format, and hash matched. Active Holdout metadata was
checked only for identity collision; no Holdout audio, source-directory
discovery, or commercial reference was opened. The ignored access log is
`artifacts/development/riotbox-1461/access-log-20260824T184434Z.json`, SHA-256
`edfac9d289c3e0239982b2f9ff1236a2dc6e7c020c2e3665f78256a5ee35ac63`.

The pack is a scripted diagnostic rendering of the current dense-break
mechanism, not an exact live-TUI journey and not product-quality proof. It
contains an approximately eight-bar source presentation and a rebuild-only
W-30 / TR-909 / MC-202 performance arc. It adds no new mechanism and cannot
substitute for exact live output, replay, or restart evidence.

## Pre-Playback Safety Correction

The first fresh candidate passed the old sample-peak check but independently
measured `+6.0 dBTP`. Its SHA-256 was
`a43ae8a43535d6d00cf3a74f9e9c5f84eda5d8adf980a5398311328091e7dcae`.
It was rejected and never played.

RBX-319 freezes `riotbox.audio_presentation_true_peak_safety.v1`: a
conservative four-times band-limited estimate, `-1.0 dBTP` hard ceiling,
`-1.2 dBTP` normalization target, and one uniform presentation-only gain for
the source window and every emitted transformed section. This changes neither
the musical render nor its internal balances and grants no quality claim.

The safe exact artifacts used for review were:

- source presentation SHA-256
  `5e3779c7dd33baa2ac381b75e5cdc7e91fd9b72635f3a0f3b62fefa44c8e3749`,
  independently measured at `-23.0 LUFS`, `0.8 LU` LRA, and `-6.8 dBTP`;
- transformed candidate SHA-256
  `23da8815a2083c3929cd2ad6bc7d9f7e63b398f67dcb667fadeb1bbea86736b2`,
  independently measured at `-13.9 LUFS`, `7.5 LU` LRA, and `-2.1 dBTP`.

All emitted presentation WAVs remained below `-1.0 dBTP`. The presentation
safety result is a technical listening precondition only.

## Human Review

The exact safe source presentation was played first, followed by a one-second
pause and the exact safe transformed candidate. Playback ended normally and
no player remained active.

The listener understood the intended sectional development, but rejected the
overall transformation. The foundational transformed break was not musically
usable, which made the value of the added modified element difficult to assess
fairly. The higher-level intention remains promising and could become useful
if the base transformation first preserves source identity, groove, and
clarity.

The structured review records:

- `human_verdict: reject`;
- `strongest_element: none`;
- `source_recognition: source_transformed_but_present`;
- `hook_after_two_bars: weak`;
- review SHA-256
  `f9bb8cb948f106a095778f6644d9db1b398adc0054759501ad89e2fdb705be31`.

Automated source-character and pressure gates therefore remain diagnostic and
do not override the human musical-fitness failure.

## Readiness Consequence

The result is promoted into the local live evidence bank as a reviewed
non-demo outcome, not as a demo-ready success. The demo-bank SHA-256 is
`ca6c141f78589015f67f42edb3180f3a9663df409c6199260568ca35105d78c9`.

RIOTBOX-1461 also corrects aggregate readiness routing so a current structured
human failure retains its concrete fix category even when older automated
gates classify the same category's fixtures as stale controls. The resulting
report routes:

- `dense_break` to one bounded foundational `chop_policy` correction;
- `sparse_drums` and `tonal_riff` to missing real-source candidate work.

It no longer asks for a second Dense source-selection pass after the existing
candidate has already been reviewed and rejected. The local readiness report
SHA-256 is
`2d800d285ef0635e6b8c2a71d02cb49620a8de94c35d6229e5d8d69a161b26ee`.

RIOTBOX-1461 closes fail-closed: the candidate is not demo-ready, release and
quality claims remain blocked, and no Holdout or source-general claim is made.
The next audible product slice must improve the foundational dense-break
transformation before re-evaluating any higher-level modification or seeking
demo promotion.

## Validation

- professional output listening-pack smoke and mutation fixtures: pass;
- destructive-variation professional smoke: pass;
- true-peak estimates independently checked with FFmpeg: pass for safe pack;
- structured review import and demo-bank promotion: pass;
- source-family coverage and live human-review queue: pass;
- sound-quality readiness smoke, including current-human-failure routing: pass;
- full repository `just ci`: pass;
- no unsafe candidate playback, Holdout audio access, commercial reference,
  or source-directory discovery occurred.

The broad CI run reused its explicit registered fixture-audio gates. Those
regressions create no additional RIOTBOX-1461 source, musical, or readiness
claim.

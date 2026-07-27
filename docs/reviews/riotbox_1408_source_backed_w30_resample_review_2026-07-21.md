# RIOTBOX-1408 Source-Backed W-30 Resample Review

Date: 2026-07-21

Structured listening verdict: 2026-07-23

Classification: `audible_vertical_slice`

## Outcome

The W-30 internal resample tap now renders hydrated audio from the committed capture lineage. The former fixed-frequency oscillator, shimmer voice, and fixed transport-tempo behavior are absent from the product renderer. A missing or unhydrated capture has typed `source_audio_unavailable` state, silent routing, a runtime/observer warning, and no synthetic fallback.

The slice preserves the existing product spine:

- `CaptureRef`, its artifact, and lineage remain Session/replay truth.
- The app derives a bounded, heap-owned runtime grain from the existing capture cache outside the callback.
- The coherent shared state carries fixed-size PCM and identity into the callback without I/O or allocation there.
- The live callback and exact offline RuntimeMix seam receive the same transport tempo and render position.
- `promote.resample` continues through its existing queue, commit, capture-artifact, observer, and replay paths.

## Review Findings Resolved

1. The first source-window projection averaged across the full capture and removed too much attack and upper-mid detail. It now deterministically selects the strongest energy/transient `4096`-frame original-PCM region.
2. Keeping the grain inline in copied render plans overflowed the stack in the dense-break live-path smoke. Public runtime state now heap-owns the grain while the callback snapshot remains bounded and allocation-free.
3. The first missing-source QA control removed PCM but retained a contradictory ready route. It now uses typed unavailable state and silent routing, matching the product contract.
4. The renderer initially coerced a missing tempo to `1 BPM`. It now advances the chop grid only with a finite positive transport tempo, avoiding another hidden musical fallback.
5. The source projection initially coerced a zero channel count to mono. Invalid channel/sample-rate metadata and empty PCM now fail projection and therefore remain unavailable/silent.

## Exact Audio Evidence

Exact isolated RuntimeMix renders use silent TR-909, MC-202, W-30 preview, and generated source-monitor contributors. Only the committed W-30 resample tap is audible.

| Source | Tap SHA-256 | Peak | RMS | Missing-source control |
| --- | --- | ---: | ---: | --- |
| Beat03 130 BPM | `3ed25cdef27fec000d4ab35304667c85b1d4f48e986b86c3ba5e236b0fd96ada` | `0.20599318` | `0.016432744` | digital silence |
| Beat08 128 BPM | `1569516ffc194779244bd0915566efb5cb08a4018e3c5ccc2fc7e1f86c3d6a5f` | `0.25394` | `0.024507` | digital silence |
| Beat20 80 BPM | `d2f1657163ae01c723e51f685b13ce489d885b856f9857c93ef77b3963420d0b` | `0.26245` | `0.021704` | digital silence |

Beat03 reproduced byte-for-byte in a separate run. Pairwise source deltas are non-collapsed: correlations range from `-0.168341` to `-0.011637`, with RMS deltas from `0.027515` to `0.035376`.

The focused callback regression also rejects collapse to raw source PCM and proves that inverting the prepared source grain changes the rendered output with opposite aggregate polarity. The retired oscillator could not satisfy that source-dependence proof, and its fixed frequencies and state fields no longer exist in the renderer.

## Validation

- `just ci`: pass
- `just audio-qa-ci`: pass
- `cargo fmt`: pass
- focused app/audio resample tests: pass
- dense-break exact live-path smoke: pass after the heap-ownership fix
- `git diff --check`: pass
- structured listening artifact: `/tmp/riotbox-1408-listening-review/review.json`
- `human_verdict`: `technically_ok_but_musically_weak`

## Human Listening Verdict

Owner first described the isolated element as interesting. A direct raw-level
A/B then made the musical limitation clear:

- source: the complete `3.692313`-second Beat03 source at `-14.8 LUFS`
- separator: `250 ms` digital silence
- candidate: the first `3.692313` seconds of the exact tap at `-29.8 LUFS`
- verdict: `technically_ok_but_musically_weak`
- strongest element: `chop`
- source recognition: `source_lost`
- hook after two bars: `weak`
- listener description: a very timid, gentle tap

The technical proof therefore establishes real PCM dependence, deterministic
replay, diversity, silence without material, and an exact callback-owned
render, but not sufficient source character, level, or stage usefulness. Bass
pressure is `unassigned` for this tap and was not part of the failure. The
concrete audible follow-up is `RIOTBOX-1422`: retain the accepted source-backed
seam, raise usable active level, preserve more recognizable source character,
and add a performer-triggered hard variation. This weak verdict blocks
demo-ready promotion and must not be rewritten as a sound-quality pass.

## Remaining Limits

`render_tr909_w30_preview.rs` is above the soft review-size range, but the resample renderer remains a coherent callback-owned sibling of the existing W-30 preview renderer. A future semantic extraction may reduce review cost; this slice avoids an unrelated module-ownership rewrite. The shared state copies a bounded `4096`-sample grain per callback snapshot, consistent with the existing W-30 fixed-window design. Callback timing/regression gates remain the acceptance boundary for that cost.

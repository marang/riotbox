# RIOTBOX-1430 Stage-A-v5 Qualification Pass and Matrix-v6 Freeze

Date: 2026-08-11

The single fresh Development-only qualification passed. All thirteen exact
bound source files were opened once through bounded no-follow access and
analyzed with the unchanged Detector, Anatomy, source-feature, and source-
contrast implementation. Holdout audio and commercial references remained
untouched.

Nine sources qualified individually. The first lexicographic combination that
also passed author, three-family, and unique source-contrast partition gates is:

- `freesound_dabromusic_266735` — pool 5, `dense_break`;
- `freesound_dr_skitz_353853` — pool 9, `sparse_drums`;
- `freesound_garzul_213512` — pool 12, `electronic_drums`;
- `freesound_aikighost_19059` — pool 13, `electronic_drums`.

The valid three-cluster partition is `{5}`, `{9,12}`, `{13}`. This is source-
contrast evidence, not a taste ranking.

Matrix v6 freezes both eligible events from each source and every F1/F2/F3
condition, for exactly 24 renders. Its Rust runner calls the existing public
mechanism implementations and records policy plus basic reject-only screens.
Full-source candidate reanalysis remains mandatory before playback.

No candidate had been rendered when RBX-269 and Matrix v6 were frozen.
`quality_proof=false`, `hardness_proof=false`, and
`human_verdict=unverified` remain unchanged.

## Matrix execution and advanced-screen freeze

All 24 conditions executed. Five passed the renderer and first strict screens;
nineteen were rejected for typed renderer refusal or basic safety/identity/body
failure. Before opening any surviving candidate WAV for reanalysis, RBX-270
pins one general advanced-screen implementation and the exact five condition
IDs. Playback remains forbidden until this screen completes.

## Advanced-screen v1 terminal result and v2 correction freeze

The frozen v1 advanced screen completed once. Its raw result SHA-256 is
`d78461412f98e145e9767a111c3c5654cdaf8f0b54f84e31d2e56dcf7c9cd406`.
It retained zero of five pre-survivors, so its own contract forbids playback.
Four Aikighost conditions were rejected because their exact static-EQ split
basis was unavailable. Those fail-closed rejections remain terminal.

The DABRO F2 condition passed every raw screen, but the attenuation-matched
view re-ran the amplitude-sensitive detector after uniform gain matching and
therefore changed both event counts from four to zero. That contradicts
RBX-270's already-qualified, source-frozen event identity. No threshold or
candidate evidence is being relaxed: advanced-screen v2 freezes the raw
source/candidate event classification across both views while continuing to
compute all signal-domain metrics at each view's declared gains. The v1 result
remains immutable evidence. V2 must be committed and pass source-blind
`--validate-only` before it may reopen the exact five candidate WAVs.

## Advanced-screen v2 result and listening handoff

Committed advanced-screen v2 completed once and retained exactly
`f2_freesound_dabromusic_266735_event1`. Its exclusive ignored result has raw
SHA-256 `a0339027cf0b194dfa95a5e6baa9b1833ba2102e55fee7524f11aa4d4a11231b`.
The other four v1 rejections remain unchanged.

The survivor is a full-length, 44.1 kHz stereo Float32 render of the exact
Development source. It preserves `1.0x` playback and modifies only frozen event
1 through F2 `f2_exact_complementary_three_band_v1`; all samples outside the
event support remain bit-identical. Raw and attenuation-matched views both
retain four source/candidate events and pass every frozen identity, body,
boundary, and confound screen. This authorizes one bounded structured human
comparison, not a hardness or quality claim.

## Structured human verdict

The listener requested a longer presentation after the initial bounded event
comparison. The final verdict is bound only to the transparent looped A/B WAV
with raw SHA-256
`384d7a977f7c1f4ccca84a24b2e64790ed3074d13fccdff9d46de4078b8cb368`:
10.0 seconds of repeated A, 0.5 seconds of silence, 10.0 seconds of repeated B,
and 0.2 seconds of exact-zero endpoint silence. A and B use the same clean
121475-frame loop boundary. The presentation is 20.7 seconds, 44.1 kHz stereo
Float32, has no clipped samples, and contains no product, monitor, support, or
diagnostic voice.

The exact structured review SHA-256 is
`84e3cd67d0e764a33595ae0c880922b8bbf97a1dfc7cd3fb8969e6c3692b37ce`.
Human verdict: `reject`. The listener found no meaningful perceptual difference
that established greater percussive hardness. Source recognition remained
clear. F2 v1 is
therefore frozen as a negative human result; no scalar retuning or replay of
the unchanged candidate is allowed.

RIOTBOX-1430 has now delivered the corrected source-pool objective: fifteen
new lawful metadata candidates, thirteen admitted and freshly qualified,
four selected sources across the frozen family/author/contrast contract, the
complete 24-condition matrix, exact candidate artifacts, and a structured
human result. It does not deliver a positive hardness pass. RIOTBOX-1428 Stage
B remains blocked and must continue from a newly versioned causal mechanism
hypothesis rather than reinterpret this result.

## Post-execution code hygiene

Full branch CI later exposed one Rust-1.97 Clippy spelling requirement in the
offline Matrix runner. The expression `source.len() % channels != 0` was
replaced by the semantically identical
`!source.len().is_multiple_of(channels)`. The render-time source remains bound
to SHA-256
`534f960b5b0afe98ece921d260f4fd30b8c616da8d3b7ab21cfe841f32bd4362`;
the lint-clean current source has SHA-256
`f686532c0860e73af9cd9bbd6805777a172b8cb816a1eb95d838aded13edc2c4`.
No renderer policy, candidate bytes, metric, result, or human evidence changed,
and no rerender or replay occurred.

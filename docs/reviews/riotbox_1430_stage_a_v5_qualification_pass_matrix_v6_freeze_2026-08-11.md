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

# RIOTBOX-1402 Human Listening Review

Date: 2026-07-18
Reviewer: Markus
Verdict source: real user-session playback through the configured PipeWire
output

## Evidence Boundary

The reviewed candidate is the source-backed eight-bar Feral Break Alpha v2
render from the exact `RuntimeMix` offline simulation:

- Source: `Beat03_130BPM(Full).wav`
- Artifact: `alpha/05_feral_break_alpha_eight_bar.wav`
- Artifact SHA-256:
  `2d97345be46eb8c35e90d4018e964237f4a325713bc180e52a863f8ca6ee5117`
- Local review pack: `/tmp/riotbox-1402-bite-v2/beat03`

The artifact exercises the production mixer implementation but was not captured
from the interactive audio-device path. Its generated manifest therefore
correctly remains `quality_proof: false` and `human_verdict: unverified`.
This review records human product direction separately and does not upgrade
those machine-readable claims.

## Technical Preflight

- The exact artifact was assigned before playback.
- Peak was approximately -1.44 dBFS, with no clipping or limiter activation.
- Full-mix RMS was approximately -21.89 dBFS.
- W-30 source-backed bite increased high-frequency edge energy without changing
  the clean `grit = 0` path.
- The typed bass owner was `unassigned`; absent bass pressure was therefore not
  a failure criterion for this review.
- A four-source exact-path diagnostic matrix passed without collapsing to one
  timing envelope. Its maximum time-normalized envelope correlation was
  `0.492000`; this is diversity evidence, not musical quality proof.

## Human Verdict

Human direction verdict: **accepted for this iteration**

Reviewer wording: "ja schon besser, lassen wir es so mal"

The candidate is audibly improved enough to freeze rather than continue
parameter iteration. This is not a claim that Riotbox has reached musical-alpha
quality, nor a final structured pass for the interactive live instrument path.
The accepted sound must not be changed during ticket closeout unless a
correctness defect requires it; an audible change would require fresh listening.

## Exact Live Stop Follow-Up

On 2026-07-19 the frozen session was loaded through the real interactive
PipeWire/device path with source monitoring disabled and only the W-30 raw
capture audition plus internal resample tap active. The source capture was
`cap-01`, duration `1.842116` seconds, SHA-256
`19100072eca31f94c76c1ce941521881d200f8598cc58cae6d4ed7836db70e8e`.

The bounded stop proof recorded:

- observer start with `is_playing: true`
- `Space` / `toggle_transport` after `2.953` seconds
- the same observer event reported `is_playing: false` and
  `status_message: transport paused`
- intentional `q` after a further `1.001` seconds
- the persisted Session retained `is_playing: false`
- no Riotbox runtime remained active after exit

Markus confirmed that the output became completely silent after the announced
stop. This is a real human and exact-device-path pass for transport-stop
semantics. It does not by itself upgrade the musical verdict or prove the
remaining preset, gesture, restart, recall, and trigger journey.

## Remaining Gate

RIOTBOX-1402 still needs exact interactive live-path verification of the frozen
behavior. Capture material and exact transport stop are now proven. The
remaining check should verify preset activation, gestures, mixer output, and
intentional quit/restart/recall/trigger behavior without using this offline
render as a substitute. The next playback request must identify the exact live
candidate, state the intended role, and wait for fresh listener readiness.

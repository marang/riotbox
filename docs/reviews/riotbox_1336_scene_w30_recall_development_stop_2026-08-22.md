# RIOTBOX-1336 Scene W-30 Recall Development Stop

Date: 2026-08-22
Scope: bounded Development exploration only
Result: stop recall v1; retain explicit Scene W-30 `pin` ownership

## Boundary

The exploration used only registered Development case
`oga_cinameng_can_be_so_beautiful` at
`data/test_audio/external/RIOTBOX-1423/wav/dense_oga_cinameng_can_be_so_beautiful.wav`
with SHA-256
`bf5fa8c5bc15e39d79cb51a08a54ccc4d663ab4996149b29153bd0e1febebd6f`.
The fresh exclusive access session is recorded locally at
`artifacts/development/riotbox-1336/access-log-v1.json`. It opened one exact
Development file, performed no directory discovery, and opened no Holdout or
commercial-reference audio.

This was an exploration of `scene_owned_w30_material_recall_then_restore`, not
product qualification. Source Monitor and all non-W-30 lanes were silent so
the review isolated the claimed material change.

## Exact Artifact And Technical Result

The bounded source / A / B review artifact was
`artifacts/development/riotbox-1336/scene-recall-v1/03_scene_recall_review_source_A_B_v1.wav`
with SHA-256
`741e5d8e3c6681421814e9d4682547a60c184086129dd9a57343e6d83af9ba98`.
It is 23.130667 seconds of 48 kHz stereo PCM16, measures -19.5 LUFS and
-3.3 dBFS peak, and stopped at the announced endpoint after each playback.

A retained the focused W-30 capture throughout. B replaced only the middle
eight-beat segment with a second scene-owned capture and then restored the
original capture sample-exactly. The changed middle measured delta RMS
`0.141144` with peak absolute delta `0.427933`, so the mechanism was clearly
active rather than masked or collapsed.

Aggregate middle-section chroma remained close (cosine `0.9870`), but the
time-local result did not: framewise chroma cosine averaged `0.5323`, reached a
10th percentile of `0.2456`, and changed dominant pitch class in `84.8%` of
frames. RMS-envelope correlation was `-0.0815`, onset-flux correlation was
`0.1201`, and mean spectral centroid moved from approximately `162.9 Hz` in A
to `131.1 Hz` in B. The second capture therefore preserved broadly related
pitch material while disrupting local pitch, phrase, and groove continuity.

## Listening And Engineering Verdict

The listener judged A musically usable, but not aligned with personal taste and
therefore not a preferred transformation. This is not a usability rejection.

B was clearly distinguishable, but its middle recall was musically unusable:
the added transformation sounded broken and harmonically incoherent, with the
perception of an instrument entering on a wrong note. The independent technical
review agrees with that verdict. The strong frame-local pitch-class mismatch
and decorrelated amplitude/onset contours explain why B can share similar
aggregate pitch content yet still interrupt the phrase as an unrelated event.

## Consequence

Stop arbitrary scene-section W-30 recall v1. Do not tune its window, source,
thresholds, or timing against this consumed result, and do not run a v2 inside
RIOTBOX-1336. The temporary recall renderer is removed.

The useful product correction is narrower: Scene movement explicitly owns W-30
as `pin`. `scene.launch` and `scene.restore` keep the focused source-backed
capture, pad assignment, articulation, and damage/resample state unchanged
while TR-909 and MC-202 express the current Scene contrast. Session, replay,
observer, Jam view, UI cues, and exact mixed-output tests expose and prove that
ownership. A future W-30 `recall` or `resample` Scene role requires a new typed,
versioned contract, Decision, bounded exploration, and output/listening proof.

No source-general, hardness, automatic-composition, Holdout, demo, release, or
overall P023 completion claim follows.

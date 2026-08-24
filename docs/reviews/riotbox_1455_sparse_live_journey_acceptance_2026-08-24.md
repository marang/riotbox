# RIOTBOX-1455 Sparse Live Journey Acceptance

- Date: 2026-08-24
- Partition: Development only
- Decisions: RBX-316, RBX-317
- Result: bounded exact-live journey keep

## Scope

RIOTBOX-1455 integrates the accepted sparse-pressure behavior into one exact
product journey: source monitor, capture, raw audition, promotion, committed
live-policy projection, held sparse state, capture-scoped `transient_bite`,
explicit bypass and ordinary re-entry, Session save, process reconstruction,
recall, and trigger.

The representative source is the registered Development-only
`DH_BeatC_KickSnr_120-01.wav`, SHA-256
`8a970e5d7bd9b29771aba85f75e697c7510940d4404714bfb1e55e210c15f46c`,
with manual `120 BPM` and zero-second downbeat. The completed bounded access log
has SHA-256
`263141d7cc0f612964e815fd2b1927292b1b07c1bf1bd5ef2c9e03bc00fa5916`;
it records one exact Development file, no directory discovery, and no Holdout
or commercial-reference access.

## Versioned Contracts

RBX-316 freezes the existing capture-scoped `w30.apply_damage_profile` action
as an explicit Apply/Bypass contract. For this sparse journey, intensity `0.82`
applies `transient_bite` to the active promoted capture and intensity `0.0`
bypasses it. The ordinary state is not a second preset or hidden app-local
mode, and damage state does not spill to another capture.

The first exact run failed closed because MC-202 output differed between 128-
and 257-frame callback partitions. RBX-317 versions the MC-202 source-phrase
renderer to v2: transport time snaps to the nearest absolute audio frame and
each sample derives from that integer frame. The musical phrase, pitch,
envelope, levels, Sparse policy, and Transient Bite parameters remain
unchanged. The rejected v1 manifest remains negative evidence.

## Exact Output Proof

`just sparse-pressure-live-journey` passed with no manifest failures. The exact
RuntimeMix proof establishes:

- capture, raw audition, promotion, held performance, Apply, Bypass, save,
  reconstruction, recall, and trigger all complete;
- held, Transient Bite, ordinary re-entry, and restart-recall stages match their
  named W-30, TR-909, and MC-202 contributors sample-exactly;
- 128- and 257-frame callback partitions match exactly for all three generated
  lanes;
- Source Monitor is silent throughout every generated stage;
- `transient_bite` gates each W-30 trigger after fraction `0.3608`, derived from
  committed intensity `0.82` and the frozen `0.44` full-gate fraction;
- the TR-909 owns the hardest transient while bounded MC-202 punctuation remains
  present;
- explicit bypass clears the capture-scoped articulation before ordinary
  re-entry, and restart recall remains ordinary rather than damaged;
- all stages avoid clipping and limiter intervention; and
- save/restart preserves the promoted capture and preset before recall.

The final manifest SHA-256 is
`0d8359819210acd99cc2f49aeef999e80adca5fd9ef1d41f7994624c83fbc80d`.

## Human Review

The final stereo 48 kHz PCM16 review artifact is exactly `31.0` seconds,
measures `-22.5 LUFS` with `-2.1 dBFS` true peak, contains no clipping, and ends
with one second of silence. Its SHA-256 is
`64bb983b5fccdeced71b03c8d07bd031726a52995a60a6a89aeab8cda8f1c69d`.
The order is source context, held sparse state, Transient Bite, explicit
bypass/ordinary re-entry, restart recall, and final silence.

After exact-artifact preflight and fresh readiness, two bounded playbacks
reached the announced endpoint and stopped silently. The listener judged both
the held sparse transformation and its Transient Bite variation musically
successful. The structured review records `human_verdict: keep`, strongest
element `chop`, `source_transformed_but_present`, and clear differentiation.
Its SHA-256 is
`7091d1699500857e5cde043fba0930409ede3848d170f999597507f20bd30184`.
The review does not claim a human preference between the two useful states;
ordinary re-entry and restart identity remain exact technical evidence rather
than a separate taste verdict.

## Claim Boundary

This is a bounded Development-only integration pass for one sparse capture and
restart journey. It grants no universal source, hardness, bass, Holdout,
release, demo-readiness, or zero-downtime restart claim. `transient_bite` is a
capture-scoped rhythmic choke: it shortens W-30 trigger sustain while retaining
the sparse TR-909 transient owner and MC-202 punctuation. Future changes to its
pinned algorithm, thresholds, or live-policy contract require a new version
and durable decision rather than post-result tuning.

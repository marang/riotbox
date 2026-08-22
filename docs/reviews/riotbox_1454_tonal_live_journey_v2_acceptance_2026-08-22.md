# RIOTBOX-1454 Tonal Live Journey v2 Acceptance

Date: 2026-08-22  
Partition: Development only  
Decision: RBX-315  
Result: bounded exact-live journey keep

## Scope

RIOTBOX-1454 integrates already-qualified tonal behavior into one exact product
journey: source monitor, capture, raw audition, promotion, committed live-policy
projection, held W-30 hook, `w30.pitch_dive`, ordinary re-entry, Session save,
process reconstruction, recall, and trigger. It does not requalify or retune the
Pitch Dive mechanism already covered by RIOTBOX-1442 through RIOTBOX-1444.

The representative source is the registered Development-only
`DH_RushArp_120_A.wav`, SHA-256
`ec2a0c930eb338bf81cd5cb4b5fef487e07c140ad40181e1d92b2a0990334e0e`,
with manual `120 BPM` and zero-second downbeat. The completed bounded access log
has SHA-256
`918a9f70db2c43dad8b824fc9577dccc881215ebbdd664b8253a9849775edcde`;
it records one exact Development file, no directory discovery, and no Holdout
or commercial-reference access.

## Versioned Correction

The rejected v1 journey exposed a weak tonal TR-909 support pulse and an
overlong presentation after the W-30 destructive exit. RBX-315 versions the
shared policy as `riotbox.live_performance_policy.v2`: tonal held state assigns
TR-909 and generic MC-202 to typed `stay_out`, while explicit performer Fill,
Takeover, Slam, and Scene movement remain valid overrides. The qualified
`w30_pitch_dive_v1` curve and thresholds remain unchanged.

## Exact Output Proof

`just tonal-hook-live-journey` passed with no manifest failures. The exact
RuntimeMix callback proof establishes:

- generated held, Pitch Dive, and ordinary-re-entry stages are sample-exact to
  their W-30-only sequence;
- restart recall is sample-exact to W-30-only output;
- TR-909, MC-202, and Source Monitor each have maximum journey RMS `0.0`;
- the first eight Pitch Dive beats remain sample-exact to held W-30 output;
- the active four-beat tail has delta RMS `0.141857`;
- 128- and 257-frame callback partitions match exactly;
- all stages avoid clipping and limiter intervention;
- ordinary re-entry clears the timed articulation; and
- save/restart preserves the promoted capture and preset before recall.

The final manifest SHA-256 is
`28a95aae429361de50b3590e0feabf99f4426e35e1bdb43a818c006a2fe0b27d`.

## Cross-Character Regression

The existing controlled Development matrix passed after one fail-closed run
exposed and removed an obsolete dense-only `BreakReinforce` precondition from
the tonal preparation path. No audio algorithm or threshold changed. The fresh
three-source rerun proved exact duplicate stability for tonal and sparse cases,
distinct dense/tonal/sparse envelopes, silent tonal TR-909 and MC-202, unchanged
dense TR-909 lead plus MC-202 instigation, and unchanged sparse TR-909 lead plus
MC-202 punctuation. The matrix report SHA-256 is
`8f995e83438a31db08d041e7f7ae6f88ccfbfd7e13262139309c3f2eb6301072`;
its completed bounded access-log SHA-256 is
`852c388abd57f5cd29281e84250ecd989ee2f2b9a868f893d4e1eb33b6f3796c`.

## Human Review

The final stereo 48 kHz PCM16 review artifact is exactly `29.0` seconds,
measures `-16.0 LUFS` with `-2.7 dBFS` true peak, contains no clipping, and
ends with one second of silence. Its SHA-256 is
`24eca9572537d81d6ed87c61c13806a0c679092d8f8f73723e2015bfff490e6b`.
The order is source context, held W-30 hook, Pitch Dive, direct ordinary
re-entry, immediately presented restart recall, and final silence. The direct
review join does not claim zero-downtime audio across application restart; it
avoids adding a fake musical gap to the evidence presentation.

After exact-artifact preflight and fresh readiness, playback reached the
announced endpoint and stopped silently. The listener confirmed that the
Pitch Dive remained successful, the unwanted timing/support pulse was absent,
and the corrected journey no longer contained the broken-sounding gap. The
structured review records `human_verdict: keep`, strongest element `chop`,
`source_transformed_but_present`, and a clear hook. Its SHA-256 is
`8c67d9a45c21e0e061906e1310c2fc64f790c9590aba4e3f51e687420c5365ea`.

## Claim Boundary

This is a bounded Development-only integration pass for one tonal capture and
restart journey. It grants no universal source, hardness, Holdout, release,
demo-readiness, or zero-downtime restart claim. Future changes to the pinned
Pitch Dive algorithm, thresholds, or live-policy contract require a new
version and durable decision rather than post-result tuning.

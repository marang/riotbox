# RIOTBOX-1442 W-30 Pitch-Dive Development Exploration

Status: completed with provisional keep
Partition: Development only  
Variant budget: three review-ready candidates maximum  
Holdout access: prohibited  
Commercial-reference access: prohibited

## Narrow Question

Can a source-backed W-30 phrase end with an immediately obvious downward pitch
gesture that remains recognizable, creates a useful transition, and feels worth
triggering live?

This is reversible musical discovery, not product implementation,
qualification, hardness evidence, or a new action contract.

## Exact Source Boundary

Open only the already registered Development source:

- case: `dense_beat03_130`
- path: `data/test_audio/examples/Beat03_130BPM(Full).wav`
- SHA-256: `e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`
- declared tempo: `130.0` BPM

Create the access log before hashing or decoding this file. Do not discover the
source directory. Reuse the existing W-30 live-hook render as control material;
do not add generated replacement audio.

## Bounded Variants

Each candidate keeps the first eight beats identical to the ordinary W-30
control and changes only the final four-beat exit. Source PCM, preceding W-30
transformation, grit, and level remain the same.

1. `continuous_tape_brake`: one continuous four-beat fall from ordinary rate to
   a deep but still articulated ending, followed by a short click-safe fade.
2. `stepped_machine_fall`: four clearly stepped rate plateaus, one per beat,
   with continuous source cursor and short transition smoothing.
3. `late_plunge_choke`: two ordinary beats followed by a fast two-beat plunge
   and deliberate terminal choke.

Produce each topology once. Do not make scalar retries after hearing a result.
Stop after an explicit keep or after all three variants fail.

## Technical Preflight

Before any playback, verify the exact source hash and report for every rendered
artifact:

- PCM format, sample rate, channels, and duration
- SHA-256, peak, RMS, active-sample count, and clip count
- control/candidate duration equality
- sample-exact first eight beats
- non-silent intended final four-beat delta
- click-safe terminal fade and silence after the announced endpoint

The only audible contributor after source playback is the source-backed W-30
control or its named pitch-dive candidate.

## Human Review

Play the exact source first. Then compare the unchanged W-30 control with only
one candidate at a time, separated by one second. Each pair may be replayed
unchanged on direct request.

Ask whether the pitch movement is obvious, preserves enough source identity,
creates a musically useful exit, avoids accidental mud/clicks/tail leakage, and
would be triggered live. A keep is provisional Development direction only. If
no candidate earns a keep, remove temporary rendering behavior and preserve one
concise negative record.

## Result

The first topology, `continuous_tape_brake`, earned a provisional Development
keep. The listener judged the presented material useful, the pitch dive
especially successful, and the candidate clearly transformed while retaining
recognizable source identity. Per the stopping rule, the two remaining rendered
topologies were not played and carry no human evidence. The exact kept recipe is
frozen in `docs/benchmarks/w30_pitch_dive_development_v1.json`; transfer
observation, product integration, and source-diverse qualification remain
separate work.

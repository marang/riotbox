# Master-bus limiter calibration protocol — v1 draft

Owner: RIOTBOX-1501. Status: **design draft; execution blocked**.
Classification: contract enabler for RIOTBOX-1501's bounded audible calibration.
This is not an accepted protocol, source-access grant, calibration result or
production-policy decision. No source, candidate, device or playback execution
is authorized by merging this document. `human_verdict` remains `unverified`.

## Purpose and invariant baseline

Determine whether either of two declared parameter changes offers a useful
sample-protection trade-off while retaining already-clean product behavior.
The musician-facing question is preservation of attack, clarity and the
source/hook during a protection event, not increased loudness or hardness.

[RBX-379's numeric ownership](../engineering/audio_numeric_values.md) and
[baseline protocol v1 / RBX-417](master_bus_limiter_baseline_protocol_v1.md)
remain unchanged: production knee `0.92`, ceiling `0.985`, existing f32 tanh
operations, inclusive knee bypass and actual write/count predicate
`abs(shaped - input) > f32::EPSILON`. Preserve the original baseline reports,
recipes, thresholds and independently frozen Stage-A contracts.

This draft proposes parameter-only comparisons of the existing stateless
sample limiter. No lookahead, oversampling, release envelope, channel-linking,
new ActionCommand, automatic policy selection, makeup gain or runtime knob.
Sample-peak headroom is not true-peak, device or hearing protection. No
source-general, family-readiness, hardness, demo or release claim is proposed.

## Proposed comparison set

Exactly three policies; no search grid or combined B+C candidate:

| Policy | Knee t | Ceiling c | Question |
| --- | --- | --- | --- |
| A: inherited baseline | 0.92 | 0.985 | Preserve the actual product control. |
| B: later onset | 0.9525 | 0.985 | What changes when protection begins later at the same ceiling? |
| C: lower ceiling | 0.92 | 0.9525 | What changes when the nominal ceiling is lower at the same onset? |

`0.9525` is the arithmetic midpoint of the inherited linear-amplitude interval,
not an optimal value, geometric midpoint, equal-dB step or perceptual midpoint.
Each alternative changes one explicit parameter; both also reduce the derived
knee width `c - t`. These are two directed sensitivity comparisons, not proof
of independently isolated knee-width effects or an optimum over all settings.
Nominal linear headroom `1 - c` is 0.015 for A/B and 0.0475 for C. Freeze the
actual f32 bit patterns and computed coordinates in the accepted successor;
do not infer bit-identical tanh results across operating systems.

The current public limiter/report APIs accept no alternate parameters and the
RuntimeMix report contains aggregate pre/post metrics, not paired PCM buffers.
This draft adds neither API nor executor. Before execution, review the smallest
offline/test-only seam needed to reuse the actual algorithm and obtain the same
pre-limiter input for every policy. Prove baseline parity first. Do not implement
a second independent DSP renderer, add callback logging or expose test policy
state through Session, presets or the public runtime. Never apply B/C to A's
already-limited output or attempt to reconstruct the input from that output.

## Separate clean-product and protection-control questions

1. **Clean regression:** unchanged product preparation, committed gestures,
   gains, source timing and contributor inventory. The original A control and
   each tested policy must independently have zero actual modified samples and
   zero pre/post clips. Preserve the exact input/output length and samples.
   A later knee cannot rescue a case rejected by A. Existing stricter recipe
   gates remain in force; no new pass threshold is inferred from these results.
   Zero writes make these outputs bit-identical to their shared input: this
   proves nonregression, not a musical gain. Do not replace the write predicate
   with the stricter, nonequivalent `peak <= knee` assertion.
2. **Synthetic stress:** reuse the fixed source-free baseline catalog as an
   immutable A control. A separately approved comparison may apply B/C to those
   same input buffers. Record expected overload and actual modifications before
   checking finite output, bounded magnitude and no post-clips. Overload is a
   protection observation, never clean-product or musical success.
3. **Source-backed stress proposal:** for each authorized clean product buffer,
   one fixed diagnostic input at exactly `2.0 * input`, identical for A/B/C.
   It is a declared overload challenge, not a supported performer gain, remedy
   for weak output or loudness compensation. Never import it as a product recipe.
   Record actual pre-overload rather than assuming that every source reaches
   the knee. If the preselected listening case does not exercise protection,
   mark that question unobserved; do not increase gain or choose another window.

All analyses use unclamped in-memory samples before WAV conversion. If a needed
pre-limiter observation is unavailable, stop; post-only zero clips cannot stand
in for it. Retain failed reports without adjusting the policy or pass criteria.

## Proposed Development batch and execution lock

At most three exact registered Development cases: one existing `dense_break`,
one `tonal_riff` and one `sparse_drums` product journey. This is a bounded
regression/stress comparison, not the broader source-general readiness matrix.
Use only already-supported, explicitly committed musician preparation and
gestures. No new musical mechanism, support lane or source substitution.

The accepted successor must fill **every** field below before any source open:

- registry version and identity, exact case IDs, paths, content hashes, rights,
  format/frame bounds and Development partition; protected-partition checks
  must use metadata and exclude identity/path/hash collisions before opening;
- exact source window, stored timing authority, Session/Graph/recipe identities,
  action order and commit boundaries, monitor route and all audible owners;
- fixed callback format/partition, render duration and sample-index windows;
- exact safe-access entry point, one bounded access log, one read/hash/decode
  from the same bytes per admitted file, and the total read/render budget;
- preselected case and local event window for listening, with deterministic
  sample-index selection frozen before new audio evidence;
- the reviewed comparison seam, f32 policy identities, command/build identity,
  artifact/report paths and the named source-access authorization.

These bindings are currently **unset**. Neither a wildcard, placeholder nor
registry membership constitutes permission. Do not discover source directories,
open/hash protected audio, borrow a consumed Stage-A session, acquire substitute
files or grant access through a validator default. Holdouts and commercial
references remain closed for this proposal. No new acquisition, access or
validator framework is part of the drafting slice.

The proposed execution budget is three sources read once each, two conditions
(clean and fixed stress), and three policies: at most 18 distinct outputs,
computed in memory. Necessary baseline repeat/partition controls must be named
and counted separately before acceptance; they must reuse admitted bytes rather
than silently expanding file access. Persist metrics, not 18 listening packs.
Stop on the first identity, access, baseline-parity or clean-regression failure.

## Measurements to bind before execution

Use existing `MasterBusLimiterReport`/`OfflineAudioMetrics` semantics for input,
pre/post peak, RMS, DC, clip/near-clip counts, linear headroom, actual modified
sample count and policy values. Require equal nonempty frame-aligned buffers,
format, channel order and sample count before a comparison; preserve the
baseline's separate empty-buffer control. Reject nonfinite values first. The
existing `signal_delta_metrics` pads unequal lengths, so it must not serve as
the alignment check. Report signed/absolute sample deltas, delta RMS and actual
sample equality. Relative delta is delta RMS / A RMS; report it as undefined
when A RMS is zero, not as an epsilon-normalized pass. Do not substitute above-knee
counts for writes or presentation-level metrics for raw product metrics.

For every preregistered local window, report per-channel peak/RMS, modified
sample count, crest factor, peak change and sample-delta RMS. A zero-RMS window
has no defined crest factor. Include a fixed event-attack
window and its adjacent body/recovery context; their exact sample intervals
must be bound before access, not selected around the largest observed gain.
Correlation and role-relevant spectral comparisons must declare their window,
normalization and zero-energy handling before use. They are descriptive evidence,
not a new numerical taste gate. No threshold is learned from the comparison.

A source-free pre-execution review must resolve any metric unavailable at the
current seam and specify its algorithm and degenerate-input behavior. If it
cannot be observed without a new architecture, stop and reshape the proposal.
RuntimeMix callback, repeat/partition and restart proof remain necessary for
any later claim about the actual live instrument; offline protection evidence
alone does not close that claim.

## Bounded human comparison and selection

Only the preselected, technically valid stress case may become an initial
comparison artifact. Proposed order: A/B/A/C from the identical two-second
input window, separated by 250 ms of silence: 8.75 seconds total. No concatenation
may truncate the preregistered event/context; if two seconds cannot contain it,
the draft must be revised before source access, not cropped after observation.
Clean bit-identical controls do not need another listening request.

Apply the full [listening contract](../specs/audio_qa/listening_review.md): exact
artifact preflight/assignment, independent pre-listen assessment, factual brief,
fresh explicit readiness, bounded playback and verified silence. The brief names
the full composite and diagnostic overload; it asks about attack, distortion,
clarity and source/hook preservation, not a product hardness pass. Agent predictions
and human verdicts remain distinct. Obvious failure or no observable policy
difference stops before requesting an unnecessary taste verdict.

Preserve raw measurements. Any required presentation attenuation is one shared
gain across every section, with the existing applicable true-peak presentation
safety evidence; require the linked presentation-safety contract for this whole
comparison even when the selected source is not Dense. This is not a claim that
the production limiter is true-peak calibrated. The attenuation is not
candidate-specific loudness normalization or product
gain compensation. Never play the unprotected pre-limiter overload as a control.
No second normalization variant or extra review-ready generation is budgeted.
An explicit immediate unchanged replay follows the existing replay rule; no
automatic repeated audition, source substitution or scalar retuning follows.

Selection remains blocked until technical evidence and the exact structured
human verdict exist. A positive stress verdict still cannot grant a clean-product
pass. A reject/inconclusive result cannot be repaired by another
unbudgeted setting. A retained A needs measured rationale and explicit limits;
none of the three policies earns an optimality claim. An alternative can inform
only a later separately frozen product decision, owning audio-core spec and
numeric-passport update, source-blind implementation and applicable exact-output
regression/review. No production default changes in this drafting slice.

## Exit from this drafting slice

Review the comparison logic, ownership, measurement gaps, budgets and access
locks. Merge only the design document, with RIOTBOX-1501 still incomplete.
Before any execution, complete the missing bindings, resolve the comparison
seam and metric definitions, record the accepted version and targeted Decision,
and obtain the exact required source authorization. Human readiness is separate
and cannot be supplied by that authorization. The accepted successor must state
its own stopping/version rules; this draft cannot silently become executable.

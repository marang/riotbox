# Master-bus limiter fixed-overload Development comparison v2

Owner: RIOTBOX-1501. Classification: bounded diagnostic follow-up, not a new
musical mechanism. **Accepted before source access under RBX-419. Execution
requires the raw JSON pin and independently reviewed clean implementation commit.**

The [JSON](master_bus_limiter_calibration_protocol_v2.json) owns exact inputs,
identities, conditions, metrics and budgets. This document owns interpretation
and operation. V1 [protocol](master_bus_limiter_calibration_protocol_v1.md),
JSON, Decision RBX-418 and [result](../reviews/riotbox_1501_limiter_development_comparison_2026-10-02.md)
remain immutable historical evidence. No Stage-A contract changes.

## Why this distinct version exists

V1 established bit-identical clean paths but did not exercise protection, even
at its fixed 2x challenge. Its consumed Development evidence motivates exactly
one added 4x challenge, not an adaptive search. This is **not source-blind or
fresh generalization evidence**. The user's subsequent approval authorizes one
separate bounded phase on the same three exact Development originals after
preregistration; it does not extend or retry V1's consumed access session.

Preserve A (0.92/0.985), B (0.9525/0.985), C (0.92/0.9525), the same production
f32 operations and actual-write predicate. Production remains provisional A.
No normalization, makeup, adaptive target peak, new limiter architecture,
musical recipe, public control or persistent product state is introduced.
4x is a diagnostic input challenge, never a remedy for weak product sound or a
hardness claim. Technical metrics cannot select the perceptually best policy.

## Exact inherited preparation and access

Use V1's unchanged committed preparation: explicit historical BPM confirmation,
six-action Dense/Tonal W-30 owner at beat 8, eleven-action Sparse prefix at beat
17, the same capture windows, contributor roles, silent monitor/resample owners
and all existing clean activity/Sparse RMS gates. The V1 preparation tables are
normative by the pinned parent contract, not recreated or retuned here.

The JSON repeats the exact case/corpus/Graph/Session pins. Before source access,
validate the pinned V1 protocol and exact comparison JSON as **metadata only**,
including their referenced compiler and historical control fingerprints. Never
hydrate V1's capture audio. The current compiler must equal the declared Linux
native compiler identity; this is not a portable cross-platform hash promise.
An unsupported build or later fingerprint mismatch fails closed, not into an
epsilon comparison or freshly learned reference.

Use the same protected v3 registry for identity/path/hash exclusion before any
selected file open. The three familiar MusicRadar/SampleRadar examples are not
CC0 holdout sources. Their known widths from V1 now bind exact PCM24 for Dense
and PCM16 for Tonal/Sparse; no new header reconnaissance is needed. One fresh
exclusive access session reads each original at most once, no-follow and
bounded, with both owners hashing the same payload and only Rust decoding the
source PCM once. No source directory discovery, Holdout or commercial audio.

## Fixed order and historical control gate

One prepared plan per source produces the existing three RuntimeMix passes:
128-frame primary, 128-frame repeat and 257-frame partition. Require identical
pre/post PCM and baseline-A reports as in V1. Retain the primary raw pre-buffer.

Process in this exact order:

1. `clean`: same A/B/C zero-write, zero-clip and activity/RMS gates.
2. `stress_2x`: original pre-buffer directly multiplied by `2.0f32`; unchanged
   A/B/C protection/finite/output checks.
3. Before **any** 4x computation, require the original pre/clean PCM and every
   2x A/B/C output to match the pinned V1 f32-little-endian SHA-256 fingerprints.
   Rust enforces this before the next condition; Python independently verifies
   the returned measurements. No file is opened for these buffer fingerprints.
4. `stress_4x`: original pre-buffer directly multiplied by `4.0f32`
   (`0x40800000`), then independently processed by A/B/C. Never multiply a 2x or
   already limited output. No gain ladder or additional condition is authorized.

V1's entrypoint retains its old two-condition behavior. V2 has an explicit
versioned entrypoint and response identity, but reuses the same source owner,
preparation, mixer, limiter and descriptive measurement implementation. No
arbitrary-gain CLI or second renderer is created.

## Budget, evidence and stopping

The full maximum is three original reads/decodes, nine RuntimeMix passes,
twenty-seven in-memory policy outputs and three ordinary derived captures.
Zero comparison WAVs and zero playbacks. The V2-only stdout limit is 128 MiB:
ten sample arrays replace V1's seven, and f32 values serialized through JSON
numbers may require long decimal representations. Test the maximum declared
frame count with generated buffers before any source access. All other input,
diagnostic and child-time limits remain unchanged; V1 retains its 64-MiB bound.

Reuse V1's fixed per-channel attack/body/recovery windows and descriptive f64
metrics, retaining original Rust f32 reports separately. Whole-render writes
or overload do not imply intervention in any particular local window. The
preselected possible later listening window remains Sparse frames [0,96000),
but explicitly names `stress_4x`, not the retained second condition. Record
protection/difference there independently; no source/window substitution.

Stop on the first access, identity, format, preparation, parity, historical
control, clean, finite or protection-bound violation. Retain available
clean/2x/4x reports and Action/Commit/Capture/timing provenance before returning
the failure. No remaining originals, retry or deletion of failed output.
If 4x still does not exercise protection, record unobserved; it does not permit
another gain or changed gate. Post-access changes need another version and
Decision, not a patch to these contracts.

Run `python3 scripts/run_master_bus_limiter_calibration_v2.py` for metadata-only
preflight. `--execute` additionally requires the accepted raw JSON pin, reviewed
clean commit, explicit locked native feature build and fresh `calibration-v2`
output directory. Record the actual executable, compiler, commit and report
identities. No playback is authorized; any future listening artifact and human
comparison require their own bounded phase and full listening-review gate.
RIOTBOX-1501 remains incomplete until a defensible calibration decision; no
source-general, musical, hardness, true-peak, device, hearing-safety or release
claim follows from this diagnostic. Windows work stays deferred.

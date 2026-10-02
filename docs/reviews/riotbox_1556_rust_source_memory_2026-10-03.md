# RIOTBOX-1556 — bounded Rust WAV/PCM admission

Date: 2026-10-03. Base: `996471448e9f763300e1cc44659ceefb7953ac25`.
Classification: maintenance/regression, P017 protection of the accepted P023
source-load/Session restore path. Contract: RBX-425.

## Purpose and ownership

Original-source `read_to_end` and PCM `collect` previously imposed no explicit
payload budget. Rust V1 now rejects encoded WAVs beyond 256 MiB and decoded
PCM16/24 beyond 67,108,864 interleaved `f32` samples (256 MiB). These are samples,
not frames. Each limit is fixed before any real-source evidence and has no
public override; changing it needs a new version and Decision.

One Audio owner reads an already admitted regular descriptor, checks metadata
before payload allocation and bounds actual reads regardless of understated
metadata or file growth. The single-byte overrun probe is stack-only; it never
returns truncated success. Requested growth is capped and uses fallible
reservation. Decoding checks sample count before exact fallible reservation;
the sample conversion functions and WAV format rules remain unchanged.

The original-source opener still supports symlinks to regular files and Unix
nonblocking admission. Capture hydration retains its own no-follow opener and
not-regular diagnostic, then uses the same descriptor byte guard. Hash and
decode continue to use one returned buffer. Typed Audio resource-limit and
allocation failures project through existing unavailable diagnostics; Core/
Session identity, queue, replay and saved files are not rewritten.

This intentionally tightens compatibility for oversized historical assets:
they become unavailable, never truncated, silently migrated or substituted.
Other independently valid capture assets may still hydrate. No DSP, live audio,
musical policy, ActionCommand, persistence schema, dependency or TUI control
changes. README remains unchanged.

## Evidence and boundaries

Behavioral RED on the two small-budget regressions proves the old reader and
decoder accepted a fifth byte/sample under a four-unit limit:
`/tmp/riotbox-1556-memory-red.log`. With guards, both pass:
`/tmp/riotbox-1556-memory-green.log`. The initial staging warning belongs only
to the superseded RED implementation, not a successful final gate.

Further small-budget tests cover exact boundary/empty input, metadata precheck
without any read, short reads, Interrupted retry at start and overrun probe,
multi-chunk byte identity/capped capacity, ordinary I/O failure, ignored WAV
chunks, PCM16/24 mono/stereo sample counts and unchanged malformed-WAV errors.
Fallible capacity-overflow testing returns a typed failure without requesting
real huge memory. Sparse production-limit-plus-one files exercise the actual
source/capture interfaces; no test reads or hashes their oversized contents.
App restore tests require no trusted cache, visible unavailable state and
unchanged saved Session bytes. Final test/CI results follow below.

Only generated synthetic controls and temporary sparse files are used. No real
Development/Holdout/commercial audio or source-directory discovery, runtime,
device, DAW or human playback. There is no new human/musical/source-general/
hardness/release or physical-device/endurance qualification.

The contract is per Rust read/decode requested payload, not a global process or
cache ceiling. Encoded and decoded buffers coexist; multiple captures accumulate,
and allocator overhead/rounding and overcommit may still matter. Caller-owned
WAV buffers already exist; `from_interleaved_samples` adopts an existing Vec.
Writers and unrelated render allocations are outside this slice. Import calls
the Python sidecar before Rust enrichment; Python's earlier full-file/frame/
sample-list allocations remain outside the contract. The independent contract
review traced that ordering explicitly; no end-to-end import-memory or OOM/
deadline/power-loss guarantee is claimed.

## Review and closeout

Independent pre-implementation contract review found no blocking objection;
its concrete boundary clarifications above were frozen before final validation.
Separate independent Maintainer/Adversarial/Rust and Spec/Evidence reviews,
found one P3 evidence gap: the existing capture no-follow opener was correct,
but no direct capture-path symlink regression protected it. Disposition: fixed
by `capture_no_follow_survives_shared_descriptor_budget_reader`, which checks
successful direct capture read, supported original-source symlink read, rejected
capture-symlink read and unavailable verified hydration with matching identity.
The Spec and Evidence Auditor independently rechecked the correction; no
remaining finding. The separate Maintainer/Adversarial/Performance/API Rust
review retained zero actionable findings. No finding is deferred or waived.
Root's follow-up self-review finds no additional defect and confirms metadata,
actual-read and allocation ordering, identical conversion, typed errors,
source/capture policy separation and unchanged Core/Session identity. Final
full source-free local `just ci` passes on the sealed source/test tree
(`/tmp/riotbox-1556-ci-sealed.log`, exit 0), including the added no-follow
regression. Exact-head native CI remains the merge gate.

Focused results: 33 Audio source tests pass in both Debug and Release; 39
distinct App source/capture/preflight cases pass, including four new sparse
regressions and the additional no-follow test. The latter also passes alone
(`/tmp/riotbox-1556-capture-no-follow.log`). An earlier full CI pass preceded
that test addition and is not used as the final-tree verification.
Logs: `/tmp/riotbox-1556-audio-source-{debug,release}.log` and
`/tmp/riotbox-1556-app-{persistence-runtime-view,capture-identity,source-file-admission,artifact-hydration-preflight}.log`.

Architecture cadence: RIOTBOX-1551 was the latest current-state checkpoint;
RIOTBOX-1552/1555 were substantive successors, while 1553/1554 were host evidence
and mechanical presentation. This is the third substantive successor; no whole
repository audit is claimed. RIOTBOX-1556 is first in the approved three-task sequence;
measured snapshot/undo scaling and callback allocation evidence remain separate
Linear-first slices, not silently folded into this change.
